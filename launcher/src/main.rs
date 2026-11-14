// Clipiccolo.exe: the portable single file. Unpacks the app, WebView2Loader.dll and ffmpeg
// into %LOCALAPPDATA%\Clipiccolo\<version> once, then starts the app with the same arguments
// (so dropping videos onto this exe works too).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::Command;

include!("../payload/payload.rs"); // FILES: &[(name, zst bytes, unpacked size)]

fn unpack(dir: &Path, name: &str, zst: &[u8], size: u64) -> std::io::Result<()> {
    let dest = dir.join(name);
    if std::fs::metadata(&dest).map(|m| m.len() == size).unwrap_or(false) {
        return Ok(());
    }
    let tmp = dir.join(format!("{name}.{}.part", std::process::id()));
    {
        let mut src = zst;
        let mut dec = ruzstd::decoding::StreamingDecoder::new(&mut src)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let mut out = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        std::io::copy(&mut dec, &mut out)?;
    }
    if std::fs::rename(&tmp, &dest).is_err() {
        // Another launch won the race, or the old file is in use; keep whatever is there if it is complete.
        let _ = std::fs::remove_file(&tmp);
        if !std::fs::metadata(&dest).map(|m| m.len() == size).unwrap_or(false) {
            return Err(std::io::Error::other(format!("could not write {}", dest.display())));
        }
    }
    Ok(())
}

/// Older versions' folders next to ours (each ~100 MB) are of no use once this one runs. One still in use
/// (an old launcher started a moment ago) can't be removed; it goes on a later start. Only folders holding
/// our payload are touched: the installer puts its own files in the parent folder.
fn remove_old_versions(dir: &Path) {
    let Some(parent) = dir.parent() else { return };
    let Ok(entries) = std::fs::read_dir(parent) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p != dir && p.is_dir() && p.join(APP).is_file() {
            let _ = std::fs::remove_dir_all(&p);
        }
    }
}

fn base_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Clipiccolo")
        .join(env!("CARGO_PKG_VERSION"))
}

#[cfg(windows)]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
fn msgbox(msg: &str, flags: u32) -> i32 {
    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: *mut core::ffi::c_void, text: *const u16, caption: *const u16, ty: u32) -> i32;
    }
    unsafe { MessageBoxW(std::ptr::null_mut(), wide(msg).as_ptr(), wide("Clipiccolo").as_ptr(), flags) }
}

#[cfg(windows)]
fn alert(msg: &str) {
    msgbox(msg, 0x10); // MB_ICONERROR
}

/// Microsoft's Evergreen WebView2 runtime registers itself under EdgeUpdate\Clients\{F3017226-...}; a missing
/// key or a "pv" of 0.0.0.0 means it is not installed. Without it the app would exit silently.
#[cfg(windows)]
fn webview2_installed() -> bool {
    #[link(name = "advapi32")]
    extern "system" {
        fn RegGetValueW(hkey: isize, sub_key: *const u16, value: *const u16, flags: u32, kind: *mut u32, data: *mut core::ffi::c_void, len: *mut u32) -> i32;
    }
    const HKCU: isize = 0x8000_0001u32 as i32 as isize;
    const HKLM: isize = 0x8000_0002u32 as i32 as isize;
    const RRF_RT_REG_SZ: u32 = 0x2;
    const CLIENT: &str = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    let keys = [
        (HKCU, format!(r"Software\{CLIENT}")),
        (HKLM, format!(r"SOFTWARE\WOW6432Node\{CLIENT}")),
        (HKLM, format!(r"SOFTWARE\{CLIENT}")),
    ];
    let value = wide("pv");
    keys.iter().any(|(root, key)| {
        let mut buf = [0u16; 64];
        let mut len = (buf.len() * 2) as u32;
        let mut kind = 0u32;
        let r = unsafe { RegGetValueW(*root, wide(key).as_ptr(), value.as_ptr(), RRF_RT_REG_SZ, &mut kind, buf.as_mut_ptr().cast(), &mut len) };
        if r != 0 {
            return false;
        }
        let v = String::from_utf16_lossy(&buf);
        let v = v.trim_end_matches('\0');
        !v.is_empty() && v != "0.0.0.0"
    })
}
#[cfg(not(windows))]
fn alert(msg: &str) {
    eprintln!("{msg}");
}

fn main() {
    #[cfg(windows)]
    if !webview2_installed() {
        let yes = msgbox(
            "Clipiccolo needs Microsoft's WebView2 runtime, which this PC doesn't have (Windows 11 and up-to-date Windows 10 include it).\n\nOpen the download page now? Install it, then start Clipiccolo again.",
            0x4 | 0x30, // MB_YESNO | MB_ICONWARNING
        ) == 6; // IDYES
        if yes {
            let _ = Command::new("rundll32").args(["url.dll,FileProtocolHandler", "https://go.microsoft.com/fwlink/p/?LinkId=2124703"]).spawn();
        }
        std::process::exit(1);
    }
    let dir = base_dir();
    let result = std::fs::create_dir_all(&dir)
        .and_then(|_| FILES.iter().try_for_each(|(n, z, s)| unpack(&dir, n, z, *s)));
    if let Err(e) = result {
        alert(&format!("Couldn't unpack Clipiccolo into\n{}\n\n{e}", dir.display()));
        std::process::exit(1);
    }
    if let Err(e) = Command::new(dir.join(APP)).args(std::env::args_os().skip(1)).spawn() {
        alert(&format!("Couldn't start Clipiccolo:\n{e}"));
        std::process::exit(1);
    }
    remove_old_versions(&dir);
}
