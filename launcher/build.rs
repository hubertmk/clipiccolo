fn main() {
    println!("cargo:rerun-if-changed=payload");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../src-tauri/icons/icon.ico")
            .set("ProductName", "Clipiccolo")
            .set("FileDescription", "Clipiccolo — shrink any video to a small MP4")
            .set("LegalCopyright", "Copyright (c) 2026 Hubert Maria Kerschbaum. MIT License.");
        res.compile().expect("resource compile failed");
    }
}
