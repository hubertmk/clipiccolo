// Getting videos in through the clipboard and getting converted ones out again (clipboard and drag).
// The parsing is plain Rust; the Win32 parts live in `win`, the AppKit parts in `mac`.

use std::path::Path;

/// Extensions we treat as video when we have to guess from a name (auto-convert from the clipboard).
/// Dropping, pasting with Ctrl+V and Browse accept anything and let ffmpeg decide.
pub const VIDEO_EXTS: &[&str] = &[
    "mp4", "mov", "mkv", "avi", "webm", "wmv", "m4v", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "3gp", "gif", "ogv",
    "m3u8",
];

pub fn is_url(s: &str) -> bool {
    let l = s.get(..8).unwrap_or("").to_ascii_lowercase();
    (l.starts_with("http://") || l.starts_with("https://")) && s.len() > 8 && !s.chars().any(char::is_whitespace)
}

/// `https://host/a/My%20clip.mp4?x=1#t` → `My clip.mp4` (empty if the path ends in `/`).
fn url_file_name(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let path = rest.split(['?', '#']).next().unwrap_or("");
    let last = match path.split_once('/') {
        Some((_, p)) => p.rsplit('/').next().unwrap_or(""),
        None => "",
    };
    percent_decode(last)
}

fn percent_decode(s: &str) -> String {
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h << 4 | l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn extension(name: &str) -> String {
    Path::new(name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// HLS playlists are many small files: ffmpeg streams those itself instead of us downloading one file.
pub fn is_hls(url: &str) -> bool {
    extension(&url_file_name(url)) == "m3u8"
}

/// Extension for the downloaded copy of a link, so ffmpeg gets a hint (`video` if the link has none).
pub fn url_extension(url: &str) -> String {
    let e = extension(&url_file_name(url));
    if !e.is_empty() && e.len() <= 5 && e.bytes().all(|c| c.is_ascii_alphanumeric()) { e } else { "video".into() }
}

/// `######      45.3%` (curl's progress bar) → 45.3
pub fn curl_percent(line: &str) -> Option<f64> {
    line.split_whitespace().last()?.strip_suffix('%')?.parse().ok()
}

pub fn looks_like_video(src: &str) -> bool {
    let name = if is_url(src) { url_file_name(src) } else { src.to_string() };
    VIDEO_EXTS.contains(&extension(&name).as_str())
}

/// `clip - piccolo.mp4`, `clip - piccolo (2).mp4`: something we made, not worth shrinking again on its own.
/// ` - small` is what earlier versions wrote, still recognised.
pub fn is_own_output(src: &str) -> bool {
    let stem = Path::new(src).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    [" - piccolo", " - small"].iter().any(|suffix| {
        if stem.ends_with(suffix) {
            return true;
        }
        match stem.rsplit_once(&format!("{suffix} (")) {
            Some((_, n)) => n.strip_suffix(')').is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit())),
            None => false,
        }
    })
}

/// File name for a video that came from a link: the link's file name without extension, made safe for Windows.
pub fn url_stem(url: &str) -> String {
    let name = url_file_name(url);
    // Not Path::file_stem: on Windows `b:c.mp4` would read as a drive letter.
    let stem = match name.rsplit_once('.') {
        Some((s, _)) if !s.is_empty() => s,
        _ => name.as_str(),
    };
    let clean: String = stem
        .chars()
        .map(|c| if c.is_control() || r#"<>:"/\|?*"#.contains(c) { '_' } else { c })
        .take(80)
        .collect();
    let clean = clean.trim_matches(|c: char| c == ' ' || c == '.').to_string();
    if clean.is_empty() { "video".into() } else { clean }
}

/// Windows: `file:///C:/a%20b/x.mp4` → `C:\a b\x.mp4`, `file://server/share/x.mp4` → `\\server\share\x.mp4`.
/// Elsewhere: `file:///Users/a%20b/x.mp4` (or `file://localhost/...`) → `/Users/a b/x.mp4`.
fn file_uri_to_path(s: &str) -> Option<String> {
    let rest = s.get(..7).filter(|p| p.eq_ignore_ascii_case("file://")).map(|_| &s[7..])?;
    #[cfg(windows)]
    {
        let rest = percent_decode(rest).replace('/', "\\");
        match rest.strip_prefix('\\') {
            Some(local) => Some(local.to_string()), // \C:\x → C:\x
            None => Some(format!("\\\\{rest}")),
        }
    }
    #[cfg(not(windows))]
    {
        let rest = rest.strip_prefix("localhost").unwrap_or(rest);
        let path = percent_decode(rest.split(['?', '#']).next().unwrap_or(rest));
        path.starts_with('/').then_some(path)
    }
}

/// Clipboard text → the files and links in it, one per line. "Copy as path" quotes are fine.
/// Anything that isn't an existing file or an http(s) link is ignored, and so is a long text.
pub fn sources_from_text(text: &str) -> Vec<String> {
    if text.len() > 16_384 {
        return vec![];
    }
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.len() > 50 {
        return vec![];
    }
    let mut out = Vec::new();
    for line in lines {
        let l = line.trim_matches(|c| c == '"' || c == '\'' || c == '<' || c == '>').trim();
        if is_url(l) {
            out.push(l.to_string());
            continue;
        }
        let p = file_uri_to_path(l).unwrap_or_else(|| l.to_string());
        if Path::new(&p).is_absolute() && Path::new(&p).is_file() {
            out.push(p);
        }
    }
    out
}

#[cfg(windows)]
pub mod win {
    use std::os::windows::ffi::OsStrExt;
    use std::path::PathBuf;
    use windows::core::{Result, PCWSTR};
    use windows::Win32::Foundation::{E_FAIL, HGLOBAL, HWND};
    use windows::Win32::System::Com::{IBindCtx, IDataObject};
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows::Win32::System::Ole::{
        IDropSource, OleFlushClipboard, OleInitialize, OleSetClipboard, CF_HDROP, CF_UNICODETEXT, DROPEFFECT_COPY,
    };
    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::Win32::UI::Shell::{
        DragQueryFileW, ILCreateFromPathW, ILFree, SHCreateShellItemArrayFromIDLists, SHDoDragDrop, BHID_DataObject, HDROP,
    };

    /// Changes every time anything is put on the clipboard.
    pub fn clipboard_seq() -> u32 {
        unsafe { GetClipboardSequenceNumber() }
    }

    pub enum Clip {
        /// Copied in Explorer (or any app that copies files).
        Files(Vec<String>),
        Text(String),
        Empty,
    }

    pub fn read_clipboard() -> Clip {
        // Another app may hold the clipboard open for a moment.
        for _ in 0..10 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                let clip = unsafe { read_open_clipboard() };
                let _ = unsafe { CloseClipboard() };
                return clip;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        Clip::Empty
    }

    unsafe fn read_open_clipboard() -> Clip {
        unsafe {
            if let Ok(h) = GetClipboardData(CF_HDROP.0 as u32) {
                let drop = HDROP(h.0);
                let n = DragQueryFileW(drop, u32::MAX, None);
                let files: Vec<String> = (0..n)
                    .filter_map(|i| {
                        let len = DragQueryFileW(drop, i, None);
                        let mut buf = vec![0u16; len as usize + 1];
                        let got = DragQueryFileW(drop, i, Some(&mut buf)) as usize;
                        (got > 0).then(|| String::from_utf16_lossy(&buf[..got]))
                    })
                    .collect();
                if !files.is_empty() {
                    return Clip::Files(files);
                }
            }
            if let Ok(h) = GetClipboardData(CF_UNICODETEXT.0 as u32) {
                let g = HGLOBAL(h.0);
                let p = GlobalLock(g) as *const u16;
                if !p.is_null() {
                    let max = GlobalSize(g) / 2;
                    let s = std::slice::from_raw_parts(p, max);
                    let len = s.iter().position(|&c| c == 0).unwrap_or(max);
                    let text = String::from_utf16_lossy(&s[..len]);
                    let _ = GlobalUnlock(g);
                    return Clip::Text(text);
                }
            }
            Clip::Empty
        }
    }

    /// The same data object Explorer hands out for these files, so every app that takes files copied or
    /// dragged from Explorer (browsers, Teams, Outlook, Explorer itself) takes ours.
    fn shell_data_object(paths: &[PathBuf]) -> Result<IDataObject> {
        unsafe {
            let wide: Vec<Vec<u16>> = paths.iter().map(|p| p.as_os_str().encode_wide().chain(Some(0)).collect()).collect();
            let pidls: Vec<*mut ITEMIDLIST> = wide.iter().map(|w| ILCreateFromPathW(PCWSTR(w.as_ptr()))).collect();
            let result = if pidls.iter().any(|p| p.is_null()) {
                Err(E_FAIL.into())
            } else {
                let list: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| p.cast_const()).collect();
                SHCreateShellItemArrayFromIDLists(&list).and_then(|a| a.BindToHandler(None::<&IBindCtx>, &BHID_DataObject))
            };
            for p in pidls.into_iter().filter(|p| !p.is_null()) {
                ILFree(Some(p.cast_const()));
            }
            result
        }
    }

    /// Puts the files on the clipboard like Ctrl+C in Explorer. Must run on the UI (OLE) thread.
    pub fn copy_files(paths: &[PathBuf]) -> Result<()> {
        unsafe {
            let _ = OleInitialize(None); // already done by the window; harmless if so
            let data = shell_data_object(paths)?;
            OleSetClipboard(&data)?;
            // Hand the data over to Windows so it stays on the clipboard after Clipiccolo closes.
            OleFlushClipboard()
        }
    }

    /// Drags the files out of the window until the mouse button goes up. Must run on the UI (OLE) thread
    /// while the button is held; blocks until the drop (or Esc).
    pub fn drag_files(hwnd: HWND, paths: &[PathBuf]) -> Result<()> {
        unsafe {
            let _ = OleInitialize(None);
            let data = shell_data_object(paths)?;
            SHDoDragDrop(Some(hwnd), &data, None::<&IDropSource>, DROPEFFECT_COPY)?;
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
pub mod mac {
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};

    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, ProtocolObject};
    use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession, NSDraggingSource,
        NSPasteboard, NSPasteboardTypeFileURL, NSPasteboardTypeString, NSPasteboardWriting, NSView, NSWorkspace,
    };
    use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL};
    use tauri::{AppHandle, Emitter};

    pub enum Clip {
        /// Copied in Finder (or any app that copies files).
        Files(Vec<String>),
        Text(String),
        Empty,
    }

    /// NSPasteboard's change count: goes up every time anything is put on the clipboard.
    pub fn clipboard_seq() -> u32 {
        NSPasteboard::generalPasteboard().changeCount() as u32
    }

    /// Looking at the types costs nothing; reading content is what macOS may ask the user about once ("access
    /// data from other apps"), so that only happens when there are files or text to read.
    pub fn read_clipboard() -> Clip {
        let pb = NSPasteboard::generalPasteboard();
        // SAFETY: reading AppKit constants.
        let (file_type, text_type) = unsafe { (NSPasteboardTypeFileURL, NSPasteboardTypeString) };
        match pb.availableTypeFromArray(&NSArray::from_slice(&[file_type, text_type])) {
            Some(t) if t.isEqualToString(file_type) => {
                let items = pb.pasteboardItems().map(|i| i.to_vec()).unwrap_or_default();
                let files: Vec<String> = items
                    .iter()
                    .filter_map(|item| item.stringForType(file_type))
                    .filter_map(|url| file_url_path(&url.to_string()))
                    .collect();
                if files.is_empty() { Clip::Empty } else { Clip::Files(files) }
            }
            Some(_) => pb.stringForType(text_type).map(|s| Clip::Text(s.to_string())).unwrap_or(Clip::Empty),
            None => Clip::Empty,
        }
    }

    /// `file:///.file/id=…` (the file-reference form Finder copies) or `file:///Users/…` → the path.
    fn file_url_path(url: &str) -> Option<String> {
        let u = NSURL::URLWithString(&NSString::from_str(url))?;
        let path = u.filePathURL().and_then(|f| f.path()).or_else(|| u.path())?;
        Some(path.to_string())
    }

    fn file_url(path: &Path) -> Retained<NSURL> {
        NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
    }

    /// Puts the files on the clipboard the way Cmd+C in Finder does, so Cmd+V in Finder, Mail, Slack … pastes them.
    pub fn copy_files(paths: &[PathBuf]) -> Result<(), String> {
        let urls: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> =
            paths.iter().map(|p| ProtocolObject::from_retained(file_url(p))).collect();
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        if pb.writeObjects(&NSArray::from_retained_slice(&urls)) {
            Ok(())
        } else {
            Err("The clipboard didn't take the files.".into())
        }
    }

    struct Ivars {
        app: AppHandle,
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements and DragSource has no Drop.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "ClipiccoloDragSource"]
        #[ivars = Ivars]
        struct DragSource;

        unsafe impl NSObjectProtocol for DragSource {}

        unsafe impl NSDraggingSource for DragSource {
            #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
            fn operation_mask(&self, _session: &NSDraggingSession, _context: NSDraggingContext) -> NSDragOperation {
                NSDragOperation::Copy
            }

            #[unsafe(method(draggingSession:endedAtPoint:operation:))]
            fn ended(&self, _session: &NSDraggingSession, _point: NSPoint, _operation: NSDragOperation) {
                // The UI ignores its own drag passing over the drop zone until this arrives.
                let _ = self.ivars().app.emit("drag-ended", ());
            }
        }
    );

    impl DragSource {
        fn new(mtm: MainThreadMarker, app: AppHandle) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(Ivars { app });
            // SAFETY: plain NSObject init.
            unsafe { msg_send![super(this), init] }
        }
    }

    thread_local! {
        /// Kept alive for the length of the drag session.
        static SOURCE: RefCell<Option<Retained<DragSource>>> = const { RefCell::new(None) };
    }

    /// Drags the files out of the window like Finder does (browsers, Mail, Slack, Finder … take them).
    /// Must run on the main thread while the mouse button is down. Returns at once: AppKit runs the drag and
    /// `drag-ended` is emitted when it is over.
    pub fn drag_files(app: &AppHandle, ns_view: *mut c_void, paths: &[PathBuf]) -> Result<(), String> {
        let mtm = MainThreadMarker::new().ok_or("A drag has to start on the main thread")?;
        // SAFETY: Tauri hands out the window's content NSView, which lives as long as the window.
        let view: &NSView = unsafe { &*(ns_view as *const NSView) };
        let event = NSApplication::sharedApplication(mtm).currentEvent().ok_or("No mouse event to drag with")?;
        let at = view.convertPoint_fromView(event.locationInWindow(), None);
        let workspace = NSWorkspace::sharedWorkspace();
        let size = 64.0;
        let items: Vec<Retained<NSDraggingItem>> = paths
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let writer: Retained<ProtocolObject<dyn NSPasteboardWriting>> = ProtocolObject::from_retained(file_url(p));
                let item = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), &writer);
                let icon = workspace.iconForFile(&NSString::from_str(&p.to_string_lossy()));
                let image: &AnyObject = &icon;
                let off = i as f64 * 6.0; // a small stack when several files go at once
                let frame = NSRect::new(NSPoint::new(at.x - size / 2.0 + off, at.y - size / 2.0 - off), NSSize::new(size, size));
                // SAFETY: `contents` is an NSImage, which is what AppKit expects.
                unsafe { item.setDraggingFrame_contents(frame, Some(image)) };
                item
            })
            .collect();
        let source = DragSource::new(mtm, app.clone());
        let _session = view.beginDraggingSessionWithItems_event_source(
            &NSArray::from_retained_slice(&items),
            &event,
            ProtocolObject::from_ref(&*source),
        );
        SOURCE.with(|s| *s.borrow_mut() = Some(source));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() {
        assert!(is_url("https://x.io/a.mp4"));
        assert!(is_url("HTTP://x.io/"));
        assert!(!is_url("https://"));
        assert!(!is_url("https://x.io/a b.mp4"));
        assert!(!is_url("C:\\a.mp4"));
        assert!(looks_like_video("https://cdn.x.io/v/My%20clip.MP4?sig=1#t=3"));
        assert!(looks_like_video("https://x.io/live/index.m3u8"));
        assert!(!looks_like_video("https://issues.x.io/browse/QA-12"));
        assert!(!looks_like_video("https://x.io/watch?v=clip.mp4"));
        assert_eq!(url_stem("https://cdn.x.io/v/My%20clip.mp4?sig=1"), "My clip");
        assert_eq!(url_stem("https://x.io/a/b:c*d.webm"), "b_c_d");
        assert_eq!(url_stem("https://x.io/"), "video");
        assert_eq!(url_stem("https://x.io"), "video");
        assert!(is_hls("https://x.io/live/index.m3u8?t=1"));
        assert!(!is_hls("https://x.io/a.mp4"));
        assert_eq!(url_extension("https://x.io/a.MOV?x"), "mov");
        assert_eq!(url_extension("https://x.io/get?id=3"), "video");
        assert_eq!(curl_percent("#######                                    12.5%"), Some(12.5));
        assert_eq!(curl_percent("curl: (22) The requested URL returned error: 404"), None);
        assert_eq!(percent_decode("a%2"), "a%2");
        assert_eq!(percent_decode("%e2%82%ac%"), "€%");
    }

    /// Overwrites the clipboard, so only on request: `cargo test clipboard_round_trip -- --ignored`
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn clipboard_round_trip() {
        let d = std::env::temp_dir().join("cs-test-copy");
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("clip - piccolo.mp4");
        std::fs::write(&f, b"x").unwrap();
        let before = win::clipboard_seq();
        win::copy_files(&[f.clone()]).unwrap();
        assert_ne!(win::clipboard_seq(), before);
        match win::read_clipboard() {
            win::Clip::Files(got) => assert_eq!(got, vec![f.to_string_lossy().into_owned()]),
            _ => panic!("no files on the clipboard"),
        }
    }

    #[test]
    fn own_output() {
        assert!(is_own_output("C:\\v\\clip - piccolo.mp4"));
        assert!(is_own_output("C:\\v\\clip - piccolo (12).mp4"));
        assert!(!is_own_output("C:\\v\\clip - piccolo (x).mp4"));
        assert!(!is_own_output("C:\\v\\piccolo talk.mp4"));
        // Written by earlier versions.
        assert!(is_own_output("C:\\v\\clip - small.mp4"));
        assert!(is_own_output("C:\\v\\clip - small (12).mp4"));
        assert!(!is_own_output("C:\\v\\clip - small (x).mp4"));
        assert!(!is_own_output("C:\\v\\small talk.mp4"));
    }

    #[test]
    fn text() {
        let d = std::env::temp_dir().join("cs-test-clip");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("a b.mov");
        std::fs::write(&f, b"x").unwrap();
        let fs = f.to_string_lossy().into_owned();

        assert_eq!(sources_from_text(&format!("\"{fs}\"")), vec![fs.clone()]);
        assert_eq!(sources_from_text(&format!("  {fs}\r\n\r\nhttps://x.io/v.mp4\nnot a thing\n")), vec![fs.clone(), "https://x.io/v.mp4".into()]);
        assert!(sources_from_text("hello world").is_empty());
        assert!(sources_from_text(&d.to_string_lossy()).is_empty(), "folders are not videos");
        #[cfg(windows)]
        {
            let uri = format!("file:///{}", fs.replace('\\', "/").replace(' ', "%20"));
            assert_eq!(sources_from_text(&uri), vec![fs.clone()]);
            assert_eq!(file_uri_to_path("file://srv/share/x%20y.mp4").unwrap(), r"\\srv\share\x y.mp4");
        }
        #[cfg(not(windows))]
        {
            let uri = format!("file://{}", fs.replace(' ', "%20"));
            assert_eq!(sources_from_text(&uri), vec![fs.clone()]);
            assert_eq!(file_uri_to_path("file://localhost/Users/a%20b/x.mp4").unwrap(), "/Users/a b/x.mp4");
            assert_eq!(file_uri_to_path("file:///a/b.mp4?x#y").unwrap(), "/a/b.mp4");
            assert!(file_uri_to_path("file://host/share/x.mp4").is_none());
        }
    }
}
