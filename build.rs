fn main() {
    println!("cargo:rerun-if-changed=assets/winlldp.ico");
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
}
