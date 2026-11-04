use crate::model::{Monitor, Packet};
use chrono::{TimeZone, Utc};
use serde::Serialize;
use std::io::{Read, Write};

fn csv_cell(s: &str) -> String {
    let safe = if s.trim_start().starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.into()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}
pub fn csv(m: &Monitor) -> String {
    let mut s =
        "adapter,system_name,chassis_id,port_id,port_description,last_seen,expires,tlvs_json\r\n"
            .to_owned();
    for n in &m.neighbors {
        let a = &n.advertisement;
        let row = [
            n.adapter.clone(),
            a.system_name.clone(),
            a.chassis_id.clone(),
            a.port_id.clone(),
            a.port_description.clone(),
            n.last_seen.to_rfc3339(),
            n.expires.to_rfc3339(),
            serde_json::to_string(&a.tlvs).unwrap(),
        ];
        s.push_str(
            &row.iter()
                .map(|c| csv_cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        s.push_str("\r\n");
    }
    s
}
pub fn json(m: &Monitor) -> Result<Vec<u8>, serde_json::Error> {
    #[derive(Serialize)]
    struct Snapshot<'a> {
        version: &'static str,
        exported_at: String,
        neighbors: &'a [crate::model::Neighbor],
        events: &'a std::collections::VecDeque<crate::model::Event>,
    }
    serde_json::to_vec_pretty(&Snapshot {
        version: env!("CARGO_PKG_VERSION"),
        exported_at: Utc::now().to_rfc3339(),
        neighbors: &m.neighbors,
        events: &m.events,
    })
}
pub fn write_pcap<W: Write>(
    packets: &std::collections::VecDeque<Packet>,
    mut w: W,
) -> std::io::Result<()> {
    w.write_all(&[
        0xd4, 0xc3, 0xb2, 0xa1, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0, 0, 1, 0, 0, 0,
    ])?;
    for p in packets {
        for n in [
            p.at.timestamp() as u32,
            p.at.timestamp_subsec_micros(),
            p.bytes.len() as u32,
            p.bytes.len() as u32,
        ] {
            w.write_all(&n.to_le_bytes())?;
        }
        w.write_all(&p.bytes)?;
    }
    Ok(())
}
pub fn read_pcap<R: Read>(mut r: R) -> Result<Vec<Packet>, String> {
    let mut h = [0u8; 24];
    r.read_exact(&mut h).map_err(|_| "Missing PCAP header")?;
    let (le, nano) = match &h[..4] {
        [0xd4, 0xc3, 0xb2, 0xa1] => (true, false),
        [0xa1, 0xb2, 0xc3, 0xd4] => (false, false),
        [0x4d, 0x3c, 0xb2, 0xa1] => (true, true),
        [0xa1, 0xb2, 0x3c, 0x4d] => (false, true),
        _ => return Err("Use classic Ethernet PCAP (convert PCAPNG with Wireshark)".into()),
    };
    let word = |b: &[u8]| {
        if le {
            u32::from_le_bytes(b.try_into().unwrap())
        } else {
            u32::from_be_bytes(b.try_into().unwrap())
        }
    };
    if word(&h[20..24]) != 1 {
        return Err("PCAP must contain Ethernet frames".into());
    }
    let mut out = vec![];
    let mut total = 0usize;
    loop {
        let mut ph = [0u8; 16];
        match r.read(&mut ph[..1]) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        };
        r.read_exact(&mut ph[1..])
            .map_err(|_| "Truncated packet header")?;
        let n = word(&ph[8..12]) as usize;
        total += n;
        if n > 65535 || total > 32 * 1024 * 1024 || out.len() >= 32768 {
            return Err("PCAP size limit (32 MiB / 32768 packets)".into());
        }
        let mut bytes = vec![0; n];
        r.read_exact(&mut bytes).map_err(|_| "Truncated packet")?;
        let fraction = word(&ph[4..8]);
        let nanos = if nano {
            fraction
        } else {
            fraction.checked_mul(1000).ok_or("Invalid timestamp")?
        };
        let at = Utc
            .timestamp_opt(word(&ph[..4]) as i64, nanos)
            .single()
            .ok_or("Invalid timestamp")?;
        out.push(Packet {
            at,
            adapter: "PCAP".into(),
            bytes,
        });
    }
    Ok(out)
}
