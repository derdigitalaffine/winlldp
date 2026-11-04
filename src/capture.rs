//! Dynamic Npcap capture. Never loads DLLs from the working directory.
use crate::{
    model::Packet,
    protocol::{self, TxProfile},
};
use chrono::Utc;
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adapter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mac: [u8; 6],
    pub ips: Vec<String>,
    pub up: bool,
}
pub fn adapters() -> Result<Vec<Adapter>, String> {
    #[cfg(windows)]
    {
        let list = ipconfig::get_adapters().map_err(|e| e.to_string())?;
        Ok(list
            .into_iter()
            .filter(|a| a.physical_address().is_some_and(|p| p.len() == 6))
            .map(|a| {
                let mut mac = [0; 6];
                mac.copy_from_slice(a.physical_address().expect("filtered six-byte MAC"));
                Adapter {
                    id: format!("\\Device\\NPF_{}", a.adapter_name()),
                    name: a.friendly_name().into(),
                    description: a.description().into(),
                    mac,
                    ips: a.ip_addresses().iter().map(ToString::to_string).collect(),
                    up: a.oper_status() == ipconfig::OperStatus::IfOperStatusUp,
                }
            })
            .collect())
    }
    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}
/// Receive backend. Auto prefers Windows Packet Monitor and falls back to Npcap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureBackend {
    #[default]
    Auto,
    Windows,
    Npcap,
}
impl CaptureBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic (Windows first, Npcap fallback)",
            Self::Windows => "Windows Packet Monitor",
            Self::Npcap => "Npcap",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Notice {
    Frame(Packet),
    Status(String, String),
    Error(String, String),
    Tx(String, bool),
    LocalFrame(String),
    Backend(String, String),
    Diagnostic(String, String),
}
#[derive(Clone)]
enum Command {
    Stop,
    Advertise(Option<TxProfile>),
}
struct Worker {
    id: String,
    command: Sender<Command>,
    join: Option<JoinHandle<()>>,
}
#[derive(Default)]
pub struct Engine {
    workers: Vec<Worker>,
    rx: Option<Receiver<Notice>>,
}
impl Engine {
    pub fn start(&mut self, devices: &[Adapter]) -> Result<(), String> {
        self.start_with(devices, CaptureBackend::Auto)
    }
    pub fn start_with(&mut self, devices: &[Adapter], backend: CaptureBackend) -> Result<(), String> {
        self.stop();
        if devices.is_empty() {
            return Err("No adapter selected".into());
        }
        // Only explicit Npcap requires loading a third-party DLL at startup.
        // Native receive is usable with no Npcap installed.
        let api = if backend == CaptureBackend::Npcap {
            Some(Arc::new(Api::load()?))
        } else {
            None
        };
        let (tx, rx) = mpsc::sync_channel(512);
        self.rx = Some(rx);
        for adapter in devices {
            let adapter = adapter.clone();
            let api = api.clone();
            let tx = tx.clone();
            let (ctx, crx) = mpsc::channel();
            let id = adapter.id.clone();
            let join = thread::spawn(move || {
                if let Some(api) = api {
                    notice(&tx, Notice::Backend(adapter.id.clone(), "Npcap".into()));
                    run(api, adapter, crx, tx);
                } else {
                    run_auto(adapter, backend, crx, tx);
                }
            });
            self.workers.push(Worker {
                id,
                command: ctx,
                join: Some(join),
            });
        }
        Ok(())
    }
    pub fn poll(&self) -> Vec<Notice> {
        self.rx
            .as_ref()
            .map(|r| r.try_iter().take(1024).collect())
            .unwrap_or_default()
    }
    pub fn advertise(&self, id: &str, profile: Option<TxProfile>) -> Result<(), String> {
        let worker = self
            .workers
            .iter()
            .find(|w| w.id == id)
            .ok_or("Adapter is not being monitored")?;
        worker
            .command
            .send(Command::Advertise(profile))
            .map_err(|_| "Adapter worker stopped".into())
    }
    pub fn stop(&mut self) {
        for w in &self.workers {
            let _ = w.command.send(Command::Stop);
        }
        for mut w in self.workers.drain(..) {
            if let Some(j) = w.join.take() {
                let _ = j.join();
            }
        }
        self.rx = None;
    }
    pub fn running(&self) -> bool {
        !self.workers.is_empty()
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Inspect the actual Npcap DLL, device enumeration and admin-only setting.
/// Never assumes that DLL presence alone means capture is working.
pub fn inspect_npcap() -> Result<NpcapReport, String> {
    let api = Api::load()?;
    let devices = unsafe { api.devices()? };
    let version = unsafe {
        let p = (api.version)();
        if p.is_null() { "Npcap (unknown version)".into() }
        else { CStr::from_ptr(p).to_string_lossy().into_owned() }
    };
    #[cfg(windows)]
    let admin_only = {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SYSTEM\CurrentControlSet\Services\npcap\Parameters")
            .ok()
            .and_then(|key| key.get_value::<u32, _>("AdminOnly").ok())
            .map(|value| value != 0)
    };
    #[cfg(not(windows))]
    let admin_only = None;
    Ok(NpcapReport { version, devices, admin_only })
}

pub fn is_local_source(frame: &[u8], mac: [u8; 6]) -> bool {
    mac != [0; 6] && frame.get(6..12) == Some(mac.as_slice())
}

#[derive(Clone, Debug, Serialize)]
pub struct NpcapReport {
    pub version: String,
    pub devices: Vec<String>,
    pub admin_only: Option<bool>,
}
#[repr(C)]
struct PcapIf {
    next: *mut PcapIf,
    name: *mut c_char,
    _description: *mut c_char,
    _addresses: *mut c_void,
    _flags: u32,
}
#[repr(C)]
struct Header {
    sec: i32,
    usec: i32,
    caplen: u32,
    len: u32,
}
#[repr(C)]
struct Bpf {
    len: u32,
    insns: *mut c_void,
}
type Handle = *mut c_void;
struct Api {
    _library: Library,
    open: unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, *mut c_char) -> Handle,
    next: unsafe extern "C" fn(Handle, *mut *const Header, *mut *const u8) -> c_int,
    close: unsafe extern "C" fn(Handle),
    send: unsafe extern "C" fn(Handle, *const u8, c_int) -> c_int,
    error: unsafe extern "C" fn(Handle) -> *const c_char,
    nonblock: unsafe extern "C" fn(Handle, c_int, *mut c_char) -> c_int,
    compile: unsafe extern "C" fn(Handle, *mut Bpf, *const c_char, c_int, u32) -> c_int,
    filter: unsafe extern "C" fn(Handle, *mut Bpf) -> c_int,
    free: unsafe extern "C" fn(*mut Bpf),
    datalink: unsafe extern "C" fn(Handle) -> c_int,
    version: unsafe extern "C" fn() -> *const c_char,
    findalldevs: unsafe extern "C" fn(*mut *mut PcapIf, *mut c_char) -> c_int,
    freealldevs: unsafe extern "C" fn(*mut PcapIf),
    direction: Option<unsafe extern "C" fn(Handle, c_int) -> c_int>,
}
impl Api {
    fn load() -> Result<Self, String> {
        #[cfg(not(windows))]
        {
            Err("Live capture requires Windows and Npcap; PCAP import is available".into())
        }
        #[cfg(windows)]
        unsafe {
            use std::os::windows::ffi::OsStringExt;
            let mut path = [0u16; 32768];
            let n = windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(
                path.as_mut_ptr(),
                path.len() as u32,
            );
            if n == 0 || n as usize >= path.len() {
                return Err("Windows system directory unavailable".into());
            }
            let path = std::path::PathBuf::from(std::ffi::OsString::from_wide(&path[..n as usize]))
                .join("Npcap")
                .join("wpcap.dll");
            let library =
                libloading::os::windows::Library::load_with_flags(path, 0x00000100 | 0x00000800)
                    .map_err(|e| {
                        format!("Npcap unavailable: {e}. Install Npcap from https://npcap.com/")
                    })?;
            let l: Library = library.into();
            macro_rules! symbol {
                ($n:literal) => {
                    *l.get(concat!($n, "\0").as_bytes())
                        .map_err(|e| e.to_string())?
                };
            }
            Ok(Self {
                open: symbol!("pcap_open_live"),
                next: symbol!("pcap_next_ex"),
                close: symbol!("pcap_close"),
                send: symbol!("pcap_sendpacket"),
                error: symbol!("pcap_geterr"),
                nonblock: symbol!("pcap_setnonblock"),
                compile: symbol!("pcap_compile"),
                filter: symbol!("pcap_setfilter"),
                free: symbol!("pcap_freecode"),
                datalink: symbol!("pcap_datalink"),
                version: symbol!("pcap_lib_version"),
                findalldevs: symbol!("pcap_findalldevs"),
                freealldevs: symbol!("pcap_freealldevs"),
                direction: l.get(b"pcap_setdirection\0").ok().map(
                    |f: libloading::Symbol<unsafe extern "C" fn(Handle, c_int) -> c_int>| *f
                ),
                _library: l,
            })
        }
    }
    unsafe fn devices(&self) -> Result<Vec<String>, String> {
        let mut head: *mut PcapIf = std::ptr::null_mut();
        let mut err = [0i8; 256];
        if (self.findalldevs)(&mut head, err.as_mut_ptr()) != 0 {
            return Err(format!("Npcap adapter enumeration: {}",
                CStr::from_ptr(err.as_ptr()).to_string_lossy()));
        }
        let mut result = Vec::new();
        let mut current = head;
        while !current.is_null() && result.len() < 1024 {
            let device = &*current;
            if !device.name.is_null() {
                result.push(CStr::from_ptr(device.name).to_string_lossy().into_owned());
            }
            current = device.next;
        }
        if !head.is_null() {
            (self.freealldevs)(head);
        }
        Ok(result)
    }
    unsafe fn err(&self, h: Handle) -> String {
        let p = (self.error)(h);
        if p.is_null() {
            "Npcap error".into()
        } else {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}
fn notice(tx: &SyncSender<Notice>, n: Notice) {
    let _ = tx.try_send(n);
}

/// Report Packet Monitor availability separately from Npcap. The actual live
/// open still performs privilege and adapter-mapping checks.
pub fn inspect_packet_monitor() -> Result<String, String> {
    #[cfg(windows)]
    {
        let capabilities = winptap::capabilities(winptap::Backend::Auto)
            .map_err(|e| e.to_string())?;
        let devices = winptap::CaptureDevice::list().map_err(|e| e.to_string())?;
        let names = devices.iter().filter(|d| d.network_interface)
            .map(|d| format!("{}: {}", d.id, d.description)).collect::<Vec<_>>();
        Ok(format!("{:?}; {} NIC components: {}", capabilities.backend,
            names.len(), names.join(" | ")))
    }
    #[cfg(not(windows))]
    {
        Err("Windows Packet Monitor is only available on Windows".into())
    }
}

/// Auto never requires Npcap, and only falls back when the selected native
/// capture really fails. Explicit native mode never silently selects Npcap.
fn run_auto(a: Adapter, backend: CaptureBackend, rx: Receiver<Command>, tx: SyncSender<Notice>) {
    let native_error: String;
    #[cfg(windows)]
    {
        match run_windows(&a, &rx, &tx) {
            Ok(()) => return,
            Err(e) => native_error = e,
        }
    }
    #[cfg(not(windows))]
    {
        native_error = "Windows Packet Monitor is not available on this OS".into();
    }
    if backend == CaptureBackend::Windows {
        notice(&tx, Notice::Error(a.id, format!("Windows Packet Monitor: {native_error}")));
        return;
    }
    notice(&tx, Notice::Diagnostic(a.id.clone(),
        format!("Windows Packet Monitor unavailable ({native_error}); trying Npcap")));
    match Api::load() {
        Ok(api) => {
            notice(&tx, Notice::Backend(a.id.clone(), "Npcap fallback".into()));
            run(Arc::new(api), a, rx, tx);
        }
        Err(npcap) => {
            notice(&tx, Notice::Error(a.id,
                format!("Windows Packet Monitor: {native_error}; Npcap: {npcap}. Run elevated for native receive or install Npcap.")));
        }
    }
}

#[cfg(windows)]
fn adapter_identity(a: &Adapter) -> winptap::InterfaceIdentity<'_> {
    winptap::InterfaceIdentity {
        // The OS adapter ID is a GUID wrapped in the historical Npcap prefix.
        name: a.id.strip_prefix("\\Device\\NPF_").unwrap_or(&a.id),
        description: Some(&a.description),
        mac_address: Some(a.mac),
    }
}

#[cfg(windows)]
fn is_remote_lldp_frame(
    bytes: &[u8],
    local_mac: [u8; 6],
    link_type: winptap::LinkType,
    direction: winptap::Direction,
) -> bool {
    link_type == winptap::LinkType::Ethernet
        && direction != winptap::Direction::Transmit
        && !is_local_source(bytes, local_mac)
        && protocol::parse(bytes).is_ok()
}

#[cfg(windows)]
fn run_windows(
    a: &Adapter,
    rx: &Receiver<Command>,
    tx: &SyncSender<Notice>,
) -> Result<(), String> {
    use winptap::{
        CaptureConfig, CaptureDevice, Error as WinError, Filter,
        LinkType, PacketReceiver,
    };
    // OS interface IDs are NPF-prefixed GUIDs for historical Npcap use.
    // Never guess a pktmon component from an enumeration index.
    let all = adapters()?;
    let interfaces: Vec<_> = all.iter().map(adapter_identity).collect();
    let capture_devices = CaptureDevice::list().map_err(|e| e.to_string())?;
    let capture_device = CaptureDevice::for_interface(
        adapter_identity(a), &interfaces, &capture_devices,
    ).map_err(|e| format!("adapter mapping: {e}"))?;

    // Driver and application filters both restrict capture to LLDP EtherType.
    // The library understands one or more 802.1Q VLAN tags.
    let lldp = Filter { ether_type: Some(0x88cc), ..Default::default() };
    let config = CaptureConfig::builder()
        .device(capture_device.clone())
        .link_type(LinkType::Ethernet)
        .snaplen(2048)
        .queue_capacity(256)
        .capture_filter(lldp)
        .build().map_err(|e| e.to_string())?;
    let mut receiver = PacketReceiver::open(config).map_err(|e| e.to_string())?;
    notice(tx, Notice::Backend(a.id.clone(), format!(
        "Windows Packet Monitor ({:?})", capture_device.backend,
    )));
    notice(tx, Notice::Status(a.id.clone(), "capture_started".into()));
    let result = (|| -> Result<(), String> {
        'receive: loop {
            for command in rx.try_iter() {
                match command {
                    Command::Stop => break 'receive,
                    Command::Advertise(Some(_)) => notice(tx, Notice::Error(
                        a.id.clone(),
                        "Windows Packet Monitor cannot inject LLDP. Select Npcap to advertise.".into(),
                    )),
                    Command::Advertise(None) => notice(tx, Notice::Tx(a.id.clone(), false)),
                }
            }
            for _ in 0..64 {
                let frame = match receiver.try_next_packet() {
                    Ok(Some(p)) => p,
                    Ok(None) => return Err("Packet Monitor stream closed".into()),
                    Err(WinError::Timeout) => break,
                    Err(e) => return Err(e.to_string()),
                };
                let bytes = frame.data();
                if is_local_source(bytes, a.mac) {
                    notice(tx, Notice::LocalFrame(a.id.clone()));
                    continue;
                }
                if !is_remote_lldp_frame(
                    bytes, a.mac, frame.metadata().link_type, frame.metadata().direction
                ) {
                    continue;
                }
                notice(tx, Notice::Frame(Packet {
                    at: Utc::now(),
                    adapter: a.id.clone(),
                    bytes: bytes.to_vec(),
                }));
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    })();
    let stopped = receiver.stop().map_err(|e| e.to_string());
    notice(tx, Notice::Status(a.id.clone(), "capture_stopped".into()));
    result.and(stopped)
}

fn run(api: Arc<Api>, a: Adapter, rx: Receiver<Command>, tx: SyncSender<Notice>) {
    unsafe {
        let device = match CString::new(a.id.as_str()) {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut err = [0i8; 256];
        let h = (api.open)(device.as_ptr(), 65535, 0, 100, err.as_mut_ptr());
        if h.is_null() {
            notice(
                &tx,
                Notice::Error(
                    a.id.clone(),
                    CStr::from_ptr(err.as_ptr()).to_string_lossy().into_owned(),
                ),
            );
            return;
        }
        // RAII guarantees close, even if configuration fails.
        struct Guard<'a>(&'a Api, Handle);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                unsafe { (self.0.close)(self.1) }
            }
        }
        let _guard = Guard(&api, h);
        if (api.datalink)(h) != 1 {
            notice(
                &tx,
                Notice::Error(
                    a.id.clone(),
                    "Adapter does not expose Ethernet frames".into(),
                ),
            );
            return;
        }
        // PCAP_D_IN=1. If unsupported, local-MAC filtering still protects the UI.
        if let Some(set_direction) = api.direction {
            let _ = set_direction(h, 1);
        }
        if (api.nonblock)(h, 1, err.as_mut_ptr()) != 0 {
            notice(&tx, Notice::Error(a.id.clone(), api.err(h)));
            return;
        }
        let mut bpf = Bpf {
            len: 0,
            insns: std::ptr::null_mut(),
        };
        let expression=c"ether proto 0x88cc or (ether proto 0x8100 and ether[16:2] = 0x88cc) or (ether proto 0x88a8 and ether[20:2] = 0x88cc) or (ether proto 0x8100 and ether[20:2] = 0x88cc)";
        if (api.compile)(h, &mut bpf, expression.as_ptr(), 1, u32::MAX) != 0 {
            notice(&tx, Notice::Error(a.id.clone(), api.err(h)));
            return;
        }
        let result = (api.filter)(h, &mut bpf);
        (api.free)(&mut bpf);
        if result != 0 {
            notice(&tx, Notice::Error(a.id.clone(), api.err(h)));
            return;
        }
        notice(&tx, Notice::Status(a.id.clone(), "capture_started".into()));
        let mut profile: Option<TxProfile> = None;
        let mut due = Instant::now();
        let send = |p: &TxProfile, shutdown: bool| -> Result<(), String> {
            let frame = protocol::encode(p, a.mac, shutdown)?;
            if (api.send)(h, frame.as_ptr(), frame.len() as c_int) != 0 {
                Err(api.err(h))
            } else {
                Ok(())
            }
        };
        'worker: loop {
            for cmd in rx.try_iter() {
                match cmd {
                    Command::Stop => break 'worker,
                    Command::Advertise(next) => {
                        if let Some(p) = profile.take() {
                            if let Err(e) = send(&p, true) {
                                notice(&tx, Notice::Error(a.id.clone(), e));
                            }
                        }
                        if let Some(p) = next {
                            match send(&p, false) {
                                Ok(()) => {
                                    due = Instant::now() + Duration::from_secs(p.interval as u64);
                                    profile = Some(p);
                                    notice(&tx, Notice::Tx(a.id.clone(), true));
                                }
                                Err(e) => {
                                    notice(&tx, Notice::Error(a.id.clone(), e));
                                    notice(&tx, Notice::Tx(a.id.clone(), false));
                                }
                            }
                        } else {
                            notice(&tx, Notice::Tx(a.id.clone(), false));
                        }
                    }
                }
            }
            if let Some(p) = &profile {
                if Instant::now() >= due {
                    if let Err(e) = send(p, false) {
                        notice(&tx, Notice::Error(a.id.clone(), e));
                        profile = None;
                        notice(&tx, Notice::Tx(a.id.clone(), false));
                    } else {
                        due = Instant::now() + Duration::from_secs(p.interval as u64);
                    }
                }
            }
            // Drain a bounded burst; worker stays responsive under load.
            for _ in 0..64 {
                let mut header = std::ptr::null();
                let mut bytes = std::ptr::null();
                match (api.next)(h, &mut header, &mut bytes) {
                    1 if !header.is_null() && !bytes.is_null() => {
                        let head = &*header;
                        if head.caplen <= 65535 && head.caplen >= 14 {
                            let bytes =
                                std::slice::from_raw_parts(bytes, head.caplen as usize).to_vec();
                            if protocol::parse(&bytes).is_ok() {
                                if is_local_source(&bytes, a.mac) {
                                    notice(&tx, Notice::LocalFrame(a.id.clone()));
                                    continue;
                                }
                                notice(
                                    &tx,
                                    Notice::Frame(Packet {
                                        at: Utc::now(),
                                        adapter: a.id.clone(),
                                        bytes,
                                    }),
                                );
                            }
                        }
                    }
                    0 => break,
                    _ => {
                        notice(&tx, Notice::Error(a.id.clone(), api.err(h)));
                        break 'worker;
                    }
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        if let Some(p) = profile {
            if let Err(e) = send(&p, true) {
                notice(&tx, Notice::Error(a.id.clone(), e));
            }
        }
        notice(&tx, Notice::Tx(a.id.clone(), false));
        notice(&tx, Notice::Status(a.id, "capture_stopped".into()));
    }
}

#[cfg(test)]
mod own_frame_tests {
    use super::*;
    #[test]
    fn never_treats_remote_or_truncated_frames_as_local() {
        let mac = [0, 17, 34, 51, 68, 85];
        let mut frame = vec![0u8; 60];
        frame[6..12].copy_from_slice(&mac);
        assert!(is_local_source(&frame, mac));
        frame[6] = 7;
        assert!(!is_local_source(&frame, mac));
        assert!(!is_local_source(&frame[..10], mac));
        assert!(!is_local_source(&frame, [0; 6]));
    }
}

#[cfg(all(test, windows))]
mod native_packet_tests {
    use super::*;
    use winptap::{Direction, LinkType};

    #[test]
    fn native_receive_requires_incoming_ethernet_lldp_from_other_mac() {
        let self_mac = [2, 1, 1, 1, 1, 1];
        let switch_mac = [2, 10, 20, 30, 40, 50];
        let profile = TxProfile {
            chassis_value: "02:0A:14:1E:28:32".into(),
            port_value: "Ethernet1/1".into(),
            ..Default::default()
        };
        let frame = protocol::encode(&profile, switch_mac, false).unwrap();
        assert!(is_remote_lldp_frame(&frame, self_mac, LinkType::Ethernet, Direction::Receive));
        assert!(!is_remote_lldp_frame(&frame, switch_mac, LinkType::Ethernet, Direction::Receive));
        assert!(!is_remote_lldp_frame(&frame, self_mac, LinkType::Ethernet, Direction::Transmit));
        assert!(!is_remote_lldp_frame(&frame, self_mac, LinkType::RawIp, Direction::Receive));
        assert!(!is_remote_lldp_frame(&frame[..12], self_mac, LinkType::Ethernet, Direction::Receive));
    }
}
