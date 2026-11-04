#[cfg(windows)]
pub struct Tray {
    pub _icon: tray_icon::TrayIcon,
    pub show: tray_icon::menu::MenuItem,
    pub exit: tray_icon::menu::MenuItem,
}
#[cfg(windows)]
impl Tray {
    pub fn new() -> Result<Self, String> {
        use tray_icon::{
            menu::{Menu, MenuItem, PredefinedMenuItem},
            TrayIconBuilder,
        };
        let menu = Menu::new();
        let show = MenuItem::new("WinLLDP öffnen", true, None);
        let exit = MenuItem::new("Beenden", true, None);
        menu.append(&show).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| e.to_string())?;
        menu.append(&exit).map_err(|e| e.to_string())?;
        let im = image::load_from_memory(include_bytes!("../assets/winlldp-otter.png"))
            .map_err(|e| e.to_string())?
            .resize(32, 32, image::imageops::FilterType::Lanczos3)
            .into_rgba8();
        let icon = tray_icon::Icon::from_rgba(im.into_raw(), 32, 32).map_err(|e| e.to_string())?;
        let icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("WinLLDP")
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            _icon: icon,
            show,
            exit,
        })
    }
    pub fn poll(&self) -> (bool, bool) {
        use tray_icon::{menu::MenuEvent, TrayIconEvent};
        let mut show = false;
        let mut exit = false;
        while let Ok(e) = MenuEvent::receiver().try_recv() {
            if e.id == *self.show.id() {
                show = true;
            }
            if e.id == *self.exit.id() {
                exit = true;
            }
        }
        while let Ok(e) = TrayIconEvent::receiver().try_recv() {
            if matches!(e, TrayIconEvent::DoubleClick { .. }) {
                show = true;
            }
        }
        (show, exit)
    }
}
pub fn is_admin() -> bool {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}
pub fn elevate() -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let file: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
        let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
        let result = windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        );
        if result as isize <= 32 {
            Err("Administrator restart cancelled or denied".into())
        } else {
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        Err("Administrator restart is available on Windows".into())
    }
}

pub fn standard_access() -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::{
            Foundation::CloseHandle, Security::*, System::Threading::*, UI::WindowsAndMessaging::*,
        };
        let shell = GetShellWindow();
        if shell.is_null() {
            return Err("Windows shell unavailable".into());
        }
        let mut pid = 0;
        GetWindowThreadProcessId(shell, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut token = std::ptr::null_mut();
        let opened = OpenProcessToken(process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut token);
        CloseHandle(process);
        if opened == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let mut primary = std::ptr::null_mut();
        let duplicated = DuplicateTokenEx(
            token,
            TOKEN_ASSIGN_PRIMARY | TOKEN_DUPLICATE | TOKEN_QUERY,
            std::ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut primary,
        );
        CloseHandle(token);
        if duplicated == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let path = std::env::current_exe().map_err(|e| e.to_string())?;
        let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut cmd: Vec<u16> = format!("\"{}\"", path.display())
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let result = CreateProcessWithTokenW(
            primary,
            0,
            file.as_ptr(),
            cmd.as_mut_ptr(),
            0,
            std::ptr::null(),
            std::ptr::null(),
            &si,
            &mut pi,
        );
        CloseHandle(primary);
        if result == 0 {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        Err("Available on Windows".into())
    }
}

pub fn launch_npcap_installer(path: &std::path::Path) -> Result<(), String> {
    // The user obtains this installer directly from npcap.com and selects it.
    // Windows UAC and the vendor's interactive setup remain visible.
    let filename = path.file_name().and_then(|s| s.to_str()).ok_or("Invalid filename")?;
    let name = filename.to_ascii_lowercase();
    if !name.starts_with("npcap-") || !name.ends_with(".exe") || !path.is_file() {
        return Err("Select an official npcap-*.exe installer downloaded from npcap.com".into());
    }
    #[cfg(windows)]
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        let absolute = path.canonicalize().map_err(|e| e.to_string())?;
        let exe: Vec<u16> = absolute.as_os_str().encode_wide().chain(Some(0)).collect();
        let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
        let result = windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            exe.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        );
        if result as isize <= 32 {
            Err(format!("Npcap installer launch cancelled/failed (code {})", result as isize))
        } else {
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        Err("Npcap installation is only available on Windows".into())
    }
}
