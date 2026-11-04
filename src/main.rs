#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod platform;
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|s| s == "--cli") {
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
        }
        std::process::exit(winlldp::cli::run(&args[1..]));
    }
    if args.iter().any(|s| s == "--self-test") {
        let p = winlldp::protocol::TxProfile {
            chassis_value: "00:11:22:33:44:55".into(),
            port_value: "self-test".into(),
            ..Default::default()
        };
        let b = winlldp::protocol::encode(&p, [0, 17, 34, 51, 68, 85], false).unwrap();
        assert_eq!(winlldp::protocol::parse(&b).unwrap().port_id, "self-test");
        if let Some(i) = args.iter().position(|s| s == "--report") {
            if let Some(p) = args.get(i + 1) {
                std::fs::write(p, "WinLLDP Windows self-test OK\n").unwrap();
            }
        }
        return Ok(());
    }
    let im = image::load_from_memory(include_bytes!("../assets/winlldp-otter.png"))
        .expect("embedded icon")
        .into_rgba8();
    let (w, h) = im.dimensions();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(if args.iter().any(|s| s == "--compact") {
                [860., 580.]
            } else {
                [980., 640.]
            })
            .with_min_inner_size([740., 480.])
            .with_title("WinLLDP — Network discovery")
            .with_icon(eframe::egui::IconData {
                rgba: im.into_raw(),
                width: w,
                height: h,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "WinLLDP",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
