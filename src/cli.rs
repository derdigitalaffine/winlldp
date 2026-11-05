//! Console interface, sharing the exact same parser/capture/export implementation as the GUI.
use crate::{
    capture::{self, CaptureBackend, Engine, Notice},
    model::Monitor,
    protocol::{self, TxProfile},
};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
const HELP: &str = r#"WinLLDP 1.1.0 — command line

WinLLDP.exe --cli [options]
  --help                    Show this help
  --list-adapters           Print adapters as JSON (no Npcap required)
  --doctor                  Check Windows Packet Monitor, Npcap and adapters (JSON)
  --backend auto|windows|npcap
                            Receive backend, default auto (native first)
  --adapter NAME-OR-ID      Select adapter by exact name/ID; repeat for receive
  --duration SECONDS        Live capture duration, default 30; 0 until Ctrl+C
  --read FILE.pcap          Read an offline Ethernet PCAP instead of live capture
  --format json|csv|pcap    Final export format, default json
  --output FILE            Export file; otherwise stdout (PCAP requires a file)
  --send PROFILE.json      Explicitly advertise on ONE selected adapter
  --write-profile FILE     Write editable defaults for ONE selected adapter

No sending is enabled by default. Windows Packet Monitor receives LLDP
without Npcap (administrator required). Npcap is optional for receiving
and required for LLDP advertisement; its permissions govern standard access. Ctrl+C stops capture
and attempts an LLDP shutdown when advertising. Offline files are snapshots.
GUI-subsystem EXE: in cmd.exe use start /wait "" WinLLDP.exe --cli ...
Exit codes: 0 success; 2 invalid arguments, I/O or capture error.
"#;
#[derive(Debug, Default)]
struct Options {
    help: bool,
    list: bool,
    doctor: bool,
    backend: CaptureBackend,
    adapters: Vec<String>,
    duration: u64,
    read: Option<String>,
    format: String,
    output: Option<String>,
    send: Option<String>,
    write_profile: Option<String>,
}
fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        duration: 30,
        format: "json".into(),
        ..Default::default()
    };
    let mut a = args.iter();
    while let Some(key) = a.next() {
        match key.as_str() {
            "--cli" => {}
            "--help" | "-h" => o.help = true,
            "--list-adapters" => o.list = true,
            "--doctor" => o.doctor = true,
            "--backend" => {
                o.backend = match a.next().map(String::as_str) {
                    Some("auto") => CaptureBackend::Auto,
                    Some("windows") => CaptureBackend::Windows,
                    Some("npcap") => CaptureBackend::Npcap,
                    _ => return Err("--backend must be auto, windows or npcap".into()),
                };
            }
            "--adapter" => o
                .adapters
                .push(a.next().ok_or("--adapter requires a value")?.clone()),
            "--duration" => {
                o.duration = a
                    .next()
                    .ok_or("--duration requires seconds")?
                    .parse()
                    .map_err(|_| "Invalid duration")?
            }
            "--read" => o.read = Some(a.next().ok_or("--read requires a file")?.clone()),
            "--format" => {
                o.format = a
                    .next()
                    .ok_or("--format requires json, csv or pcap")?
                    .clone()
            }
            "--output" => o.output = Some(a.next().ok_or("--output requires a file")?.clone()),
            "--send" => o.send = Some(a.next().ok_or("--send requires a profile")?.clone()),
            "--write-profile" => {
                o.write_profile = Some(a.next().ok_or("--write-profile requires a file")?.clone())
            }
            _ => return Err(format!("Unknown option: {key}")),
        }
    }
    if !["json", "csv", "pcap"].contains(&o.format.as_str()) {
        return Err("Use --format json, csv or pcap".into());
    }
    if o.format == "pcap" && o.output.is_none() {
        return Err("PCAP requires --output FILE".into());
    }
    if o.send.is_some() && o.backend == CaptureBackend::Windows {
        return Err("LLDP advertising requires Npcap; use --backend npcap or auto".into());
    }
    if o.read.is_some() && o.send.is_some() {
        return Err("--read cannot be combined with --send".into());
    }
    Ok(o)
}
pub fn run(args: &[String]) -> i32 {
    match execute(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("WinLLDP: {e}");
            2
        }
    }
}
fn output(bytes: &[u8], path: Option<&String>) -> Result<(), String> {
    if let Some(p) = path {
        std::fs::write(p, bytes).map_err(|e| e.to_string())
    } else {
        std::io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|e| e.to_string())
    }
}
fn execute(args: &[String]) -> Result<(), String> {
    let o = options(args)?;
    if o.help {
        print!("{HELP}");
        return Ok(());
    }
    if o.doctor {
        let report = serde_json::json!({
            "npcap": capture::inspect_npcap(),
            "windows_packet_monitor": capture::inspect_packet_monitor(),
            "selected_default_backend": "auto",
            "adapters": capture::adapters(),
            "note": "Npcap is a separately installed driver; successful probing does not prove that a remote switch is advertising LLDP."
        });
        return output(
            &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
            o.output.as_ref(),
        );
    }
    if o.list {
        return output(
            &serde_json::to_vec_pretty(&capture::adapters()?).map_err(|e| e.to_string())?,
            o.output.as_ref(),
        );
    }
    let mut monitor = Monitor::default();
    if let Some(path) = &o.read {
        let packets =
            crate::export::read_pcap(std::fs::File::open(path).map_err(|e| e.to_string())?)?;
        for p in packets {
            let _ = monitor.ingest("PCAP", p.bytes, p.at);
        }
    } else {
        let mut adapters = capture::adapters()?;
        if o.adapters.is_empty() {
            adapters.retain(|a| a.up);
        } else {
            for selected in &o.adapters {
                if !adapters
                    .iter()
                    .any(|a| a.id == *selected || a.name == *selected)
                {
                    return Err(format!("Unknown adapter: {selected}; use --list-adapters"));
                }
            }
            adapters.retain(|a| o.adapters.iter().any(|v| v == &a.id || v == &a.name));
        }
        if (o.send.is_some() || o.write_profile.is_some())
            && (adapters.len() != 1 || o.adapters.len() != 1)
        {
            return Err(
                "Sending/profile generation requires exactly one explicit --adapter".into(),
            );
        }
        if let Some(path) = &o.write_profile {
            let a = &adapters[0];
            let p = TxProfile {
                chassis_value: a
                    .mac
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(":"),
                port_value: a.name.clone(),
                port_description: a.description.clone(),
                ..Default::default()
            };
            return output(
                &serde_json::to_vec_pretty(&p).map_err(|e| e.to_string())?,
                Some(path),
            );
        }
        let profile = if let Some(path) = &o.send {
            if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
                return Err("Profile exceeds 1 MiB".into());
            }
            let p: TxProfile =
                serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            protocol::encode(&p, adapters[0].mac, false)?;
            Some(p)
        } else {
            None
        };
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst))
            .map_err(|e| e.to_string())?;
        let mut engine = Engine::default();
        let backend = if o.send.is_some() && o.backend == CaptureBackend::Auto {
            CaptureBackend::Npcap
        } else {
            o.backend
        };
        engine.start_with(&adapters, backend)?;
        let started = Instant::now();
        let mut had_ready = false;
        let mut active = std::collections::BTreeSet::new();
        let mut failed = false;
        while !stop.load(Ordering::SeqCst)
            && (o.duration == 0 || started.elapsed() < Duration::from_secs(o.duration))
        {
            for event in engine.poll() {
                match event {
                    Notice::Frame(p) => {
                        let _ = monitor.ingest(&p.adapter, p.bytes, p.at);
                    }
                    Notice::Error(id, e) => {
                        eprintln!("{id}: {e}");
                        failed = true;
                    }
                    Notice::Status(id, s) => {
                        eprintln!("{id}: {s}");
                        if s == "capture_started" {
                            had_ready = true;
                            active.insert(id.clone());
                            if let Some(p) = &profile {
                                engine.advertise(&id, Some(p.clone()))?;
                            }
                        } else {
                            active.remove(&id);
                        }
                    }
                    Notice::Tx(id, v) => {
                        eprintln!("{id}: advertising={v}");
                    }
                    Notice::LocalFrame(_) => {
                        // Local advertising echoes are not remote neighbors.
                    }
                    Notice::Backend(id, selected) => {
                        eprintln!("{id}: backend={selected}");
                    }
                    Notice::Diagnostic(id, detail) => {
                        eprintln!("{id}: {detail}");
                    }
                }
            }
            monitor.expire(chrono::Utc::now());
            if started.elapsed() > Duration::from_secs(3) && active.is_empty() {
                failed = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        engine.stop();
        if !had_ready || failed {
            return Err("Capture or transmission failed (see diagnostics)".into());
        }
    }
    let data = match o.format.as_str() {
        "csv" => crate::export::csv(&monitor).into_bytes(),
        "pcap" => {
            let mut b = vec![];
            crate::export::write_pcap(&monitor.packets, &mut b).map_err(|e| e.to_string())?;
            b
        }
        _ => crate::export::json(&monitor).map_err(|e| e.to_string())?,
    };
    output(&data, o.output.as_ref())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argument_validation() {
        let a = |s: &str| s.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        assert!(options(&a("--cli --duration nope")).is_err());
        assert!(options(&a("--cli --format pcap")).is_err());
        assert!(options(&a("--cli --read a --send b")).is_err());
        assert!(options(&a("--cli --unknown")).is_err());
        assert!(options(&a("--backend anything")).is_err());
        assert!(options(&a("--backend windows --send p.json")).is_err());
        assert_eq!(
            options(&a("--backend windows")).unwrap().backend,
            CaptureBackend::Windows
        );
        assert_eq!(
            options(&a("--backend npcap")).unwrap().backend,
            CaptureBackend::Npcap
        );
        assert_eq!(
            options(&a("--cli --duration 5 --format csv"))
                .unwrap()
                .duration,
            5
        );
    }
}
