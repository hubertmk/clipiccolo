pub const APP: &str = "ClipiccoloApp.exe";
pub static FILES: &[(&str, &[u8], u64)] = &[
    ("ClipiccoloApp.exe", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/ClipiccoloApp.exe.zst")), 3614336),
    ("ffmpeg.exe", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/ffmpeg.exe.zst")), 98738304),
    ("FFMPEG-LICENSE.txt", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/FFMPEG-LICENSE.txt.zst")), 35147),
    ("LICENSE.txt", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/LICENSE.txt.zst")), 1080),
    ("THIRD-PARTY-NOTICES.md", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/THIRD-PARTY-NOTICES.md.zst")), 3430),
    ("THIRD-PARTY-LICENSES.html", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/THIRD-PARTY-LICENSES.html.zst")), 211897),
];
