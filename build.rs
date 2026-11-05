pub fn main() {
    slint_build::compile("ui/app-window.slint").expect("compile Slint UI");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/winlldp.ico")
            .set("ProductName", "WinLLDP")
            .set("FileDescription", "WinLLDP — Network discovery")
            .set(
                "LegalCopyright",
                "Copyright 2026 Dominik Tröster, Verbandsgemeinde Otterbach-Otterberg",
            );
        res.compile().expect("Windows icon and version resource");
    }
    println!("cargo:rerun-if-changed=assets/winlldp.ico");
    println!("cargo:rerun-if-changed=ui/app-window.slint");
    println!("cargo:rerun-if-env-changed=WINLLDP_SOURCE_REV");
}
