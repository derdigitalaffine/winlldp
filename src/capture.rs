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
#[derive(Clone, Debug)]
pub enum Notice {
    Frame(Packet),
    Status(String, String),
    Error(String, String),
    Tx(String, bool),
    LocalFrame(String),
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
        self.stop();
        if devices.is_empty() {
            return Err("No adapter selected".into());
        }
        let api = Arc::new(Api::load()?);
        let (tx, rx) = mpsc::sync_channel(512);
        self.rx = Some(rx);
        for adapter in devices {
            let adapter = adapter.clone();
            let api = api.clone();
            let tx = tx.clone();
            let (ctx, crx) = mpsc::channel();
            let id = adapter.id.clone();
            let join = thread::spawn(move || run(api, adapter, crx, tx));
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
