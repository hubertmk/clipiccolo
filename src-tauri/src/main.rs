// Clipiccolo: drop, paste or copy any video (file or link) in, get a small MP4 that uploads and shares easily —
// next to the original, or in a folder chosen in Settings — and back on the clipboard. The UI lives in ../ui;
// clipboard and drag-out are in clip.rs; the installer (or the portable launcher) puts ffmpeg next to this exe,
// and on macOS it is the sidecar in Contents/MacOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

mod clip;
#[cfg(target_os = "macos")]
use clip::mac as native;
#[cfg(windows)]
use clip::win as native;

/// Saved in %APPDATA%\io.github.hubertmk.clipiccolo\settings.json
/// (macOS: ~/Library/Application Support/io.github.hubertmk.clipiccolo/settings.json).
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct UserSettings {
    /// None = next to the original video (videos from links: Downloads).
    output_dir: Option<String>,
    /// Convert a newly copied video (file or link) when the window gets focus.
    auto_paste: bool,
    /// Put finished videos on the clipboard when a batch is done (now if focused, else on the next focus).
    auto_copy: bool,
    /// Quality last chosen: "small", "balanced" or "sharp".
    preset: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self { output_dir: None, auto_paste: true, auto_copy: true, preset: "balanced".into() }
    }
}

#[derive(Default)]
struct AppState {
    settings: Mutex<UserSettings>,
    /// Clipboard sequence number last looked at (or written by us), so auto-paste only reacts to new copies.
    clip_seq: Mutex<u32>,
    ffmpeg: Mutex<Option<PathBuf>>,
    child: Arc<Mutex<Option<Child>>>,
    cancelled: Arc<AtomicBool>,
    /// The very first start (no settings file existed): the UI shows its one-time tip.
    first_run: AtomicBool,
    /// Files the OS asked us to open before the UI was listening (macOS: Finder, a drop on the Dock icon).
    opened: Mutex<Vec<String>>,
    /// Set once the UI has asked for `initial_files`; from then on opened files go straight to it.
    ui_ready: AtomicBool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Preset {
    Small,
    Balanced,
    Sharp,
}

struct Settings {
    max_side: u32,
    fps_max: u32,
    crf: u32,
    audio_kbps: u32,
}

fn settings(p: Preset) -> Settings {
    match p {
        Preset::Small => Settings { max_side: 1280, fps_max: 30, crf: 30, audio_kbps: 64 },
        Preset::Balanced => Settings { max_side: 1920, fps_max: 30, crf: 26, audio_kbps: 96 },
        Preset::Sharp => Settings { max_side: 2560, fps_max: 60, crf: 22, audio_kbps: 128 },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Done {
    output: String,
    in_bytes: u64,
    out_bytes: u64,
}

#[derive(Clone, Serialize)]
struct Progress {
    id: u32,
    pct: f64,
    /// "download" (a link being fetched) or "convert".
    stage: &'static str,
}

/// `C:\a\clip.mov` → `<dir>\clip - piccolo.mp4`, then `clip - piccolo (2).mp4`, … if taken.
/// `dir` is the Settings override, or the original's folder.
fn output_path(input: &Path, override_dir: Option<&Path>) -> PathBuf {
    let dir = override_dir.unwrap_or_else(|| input.parent().unwrap_or_else(|| Path::new(".")));
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "video".into());
    output_in(dir, &stem)
}

/// `<dir>\<stem> - piccolo.mp4`, numbered if taken.
fn output_in(dir: &Path, stem: &str) -> PathBuf {
    let mut n = 1;
    loop {
        let name = if n == 1 { format!("{stem} - piccolo.mp4") } else { format!("{stem} - piccolo ({n}).mp4") };
        let p = dir.join(name);
        if !p.exists() {
            return p;
        }
        n += 1;
    }
}

fn ffmpeg_args(input: &str, output: &Path, s: &Settings) -> Vec<String> {
    let m = s.max_side;
    // Fit inside an m×m box (long side ≤ m, portrait too), never upscale, even dimensions for H.264.
    let vf = format!(
        "scale='min({m},iw)':'min({m},ih)':force_original_aspect_ratio=decrease:force_divisible_by=2,format=yuv420p"
    );
    [
        "-hide_banner", "-nostdin", "-y",
        "-i", input,
        "-map", "0:v:0", "-map", "0:a:0?", "-sn", "-dn",
        "-vf", &vf,
        "-fpsmax", &s.fps_max.to_string(),
        "-c:v", "libx264", "-preset", "veryfast", "-crf", &s.crf.to_string(),
        // Stereo at most: a 5.1 track squeezed into 64-128 kb/s sounds worse than a clean downmix.
        "-c:a", "aac", "-ac", "2", "-b:a", &format!("{}k", s.audio_kbps),
        "-movflags", "+faststart",
        "-progress", "pipe:1", "-nostats",
        &output.to_string_lossy(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// "  Duration: 00:00:44.99, start: …" → 44.99
fn parse_duration(line: &str) -> Option<f64> {
    let rest = line.trim_start().strip_prefix("Duration:")?.trim_start();
    let hms = rest.split(',').next()?.trim();
    let mut parts = hms.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

/// Turns ffmpeg's last words into something the user can act on. `output` is the file we tried to write.
fn friendly_error(log: &[String], output: &Path) -> String {
    let all = log.join("\n");
    let low = all.to_lowercase();
    if low.contains("matches no streams") || low.contains("does not contain any stream") {
        return "This file has no video in it.".into();
    }
    if low.contains("invalid data found") || low.contains("moov atom not found") {
        return "This doesn't look like a video ffmpeg can read, or the file is damaged.".into();
    }
    if low.contains("permission denied") {
        let dir = output.parent().map(|p| p.display().to_string()).unwrap_or_default();
        return format!("Can't write to {dir}. Pick a folder you can write to under Settings → Save converted videos, and try again.");
    }
    if low.contains("no space left") {
        return "The disk is full.".into();
    }
    log.iter().rev().take(4).rev().cloned().collect::<Vec<_>>().join("\n")
}

/// The installer puts ffmpeg.exe next to this app; so does the portable launcher. On macOS the bundle carries
/// `ffmpeg` next to the executable in Contents/MacOS.
fn ensure_ffmpeg(_app: &AppHandle) -> Result<PathBuf, String> {
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = me.with_file_name(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    if exe.is_file() {
        return Ok(exe);
    }
    Err(format!("ffmpeg is missing from {}. Reinstall Clipiccolo.", me.parent().map(|p| p.display().to_string()).unwrap_or_default()))
}

/// Finds ffmpeg and returns its version ("8.0") for the Settings footer.
#[tauri::command]
async fn prepare(app: AppHandle) -> Result<String, String> {
    let a = app.clone();
    let (path, version) = tauri::async_runtime::spawn_blocking(move || {
        let path = ensure_ffmpeg(&a)?;
        let version = ffmpeg_version(&path);
        Ok::<_, String>((path, version))
    })
    .await
    .map_err(|e| e.to_string())??;
    *app.state::<AppState>().ffmpeg.lock().unwrap() = Some(path);
    Ok(version)
}

/// `ffmpeg -version` starts with "ffmpeg version 8.0-essentials_build-www.gyan.dev ..."; keep the "8.0".
fn ffmpeg_version(ffmpeg: &Path) -> String {
    let mut cmd = Command::new(ffmpeg);
    cmd.arg("-version").stdin(Stdio::null()).stderr(Stdio::null());
    no_window(&mut cmd)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            let v = s.lines().next()?.strip_prefix("ffmpeg version ")?.split_whitespace().next()?;
            Some(v.split(['-', '_']).next().unwrap_or(v).to_string())
        })
        .unwrap_or_default()
}

/// Files dropped onto the .exe itself arrive as command-line arguments. On macOS, files opened with the app
/// before the UI was up are waiting in `opened`; later ones arrive as the `open-files` event.
#[tauri::command]
fn initial_files(state: State<'_, AppState>) -> Vec<String> {
    state.ui_ready.store(true, Ordering::SeqCst);
    let mut files: Vec<String> = std::env::args().skip(1).filter(|a| Path::new(a).is_file()).collect();
    files.append(&mut state.opened.lock().unwrap());
    files
}

/// Files the OS asks us to open (Finder "Open with", a drop on the Dock icon).
#[cfg(target_os = "macos")]
fn open_files(app: &AppHandle, files: Vec<String>) {
    if files.is_empty() {
        return;
    }
    let st = app.state::<AppState>();
    if st.ui_ready.load(Ordering::SeqCst) {
        let _ = app.emit("open-files", files);
    } else {
        st.opened.lock().unwrap().extend(files);
    }
}

#[tauri::command]
async fn pick_files(app: AppHandle) -> Vec<String> {
    use tauri_plugin_dialog::DialogExt;
    let files = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Choose videos to shrink")
            .add_filter("Videos", &["mp4", "mov", "mkv", "avi", "webm", "wmv", "m4v", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "3gp", "gif", "ogv"])
            .add_filter("All files", &["*"])
            .blocking_pick_files()
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_default();
    files.into_iter().filter_map(|f| f.into_path().ok()).map(|p| p.to_string_lossy().into_owned()).collect()
}

/// Opens Explorer (Finder on macOS) in the output folder with the converted video selected.
#[tauri::command]
fn reveal(path: String) {
    #[cfg(windows)]
    {
        // Explorer only understands `/select,"C:\path with spaces\x.mp4"` — the quotes must come after the
        // comma, which std's own argument quoting can't produce, so pass the raw command line.
        use std::os::windows::process::CommandExt;
        let _ = Command::new("explorer.exe").raw_arg(format!("/select,\"{path}\"")).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = path;
    }
}

fn settings_file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

/// Builds up to 0.4.1 used the identifier com.oqton.clip-shrinker; carry their settings over once.
fn migrate_old_settings(new: &Path) {
    if new.exists() {
        return;
    }
    let Some(root) = new.parent().and_then(Path::parent) else { return };
    let old = root.join("com.oqton.clip-shrinker").join("settings.json");
    if old.is_file() {
        if let Some(d) = new.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::copy(&old, new);
    }
}

/// The saved settings, and whether a file was there to read (after migration). No file means a first start.
fn load_settings(app: &AppHandle) -> (UserSettings, bool) {
    let saved = settings_file(app)
        .inspect(|f| migrate_old_settings(f))
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|s| serde_json::from_str(&s).ok());
    match saved {
        Some(s) => (s, true),
        None => (UserSettings::default(), false),
    }
}

fn save_settings(app: &AppHandle, s: &UserSettings) -> Result<(), String> {
    let f = settings_file(app).ok_or("No settings folder")?;
    if let Some(d) = f.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::write(&f, serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> UserSettings {
    state.settings.lock().unwrap().clone()
}

/// True on the very first start only, before any settings were saved.
#[tauri::command]
fn first_run(state: State<'_, AppState>) -> bool {
    state.first_run.load(Ordering::SeqCst)
}

#[tauri::command]
fn set_preset(app: AppHandle, state: State<'_, AppState>, preset: String) -> Result<UserSettings, String> {
    let mut s = state.settings.lock().unwrap().clone();
    s.preset = preset;
    save_settings(&app, &s)?;
    *state.settings.lock().unwrap() = s.clone();
    Ok(s)
}

/// Folder picker for the output override. Returns the (possibly unchanged) settings.
#[tauri::command]
async fn pick_output_dir(app: AppHandle) -> Result<UserSettings, String> {
    use tauri_plugin_dialog::DialogExt;
    let a = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        a.dialog().file().set_title("Save converted videos to").blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    if let Some(dir) = picked.and_then(|f| f.into_path().ok()) {
        let mut s = state.settings.lock().unwrap().clone();
        s.output_dir = Some(dir.to_string_lossy().into_owned());
        save_settings(&app, &s)?;
        *state.settings.lock().unwrap() = s;
    }
    let s = state.settings.lock().unwrap().clone();
    Ok(s)
}

/// Back to "next to the original".
#[tauri::command]
fn reset_output_dir(app: AppHandle, state: State<'_, AppState>) -> Result<UserSettings, String> {
    let mut s = state.settings.lock().unwrap().clone();
    s.output_dir = None;
    save_settings(&app, &s)?;
    *state.settings.lock().unwrap() = s.clone();
    Ok(s)
}

#[tauri::command]
fn set_auto_paste(app: AppHandle, state: State<'_, AppState>, on: bool) -> Result<UserSettings, String> {
    let mut s = state.settings.lock().unwrap().clone();
    s.auto_paste = on;
    save_settings(&app, &s)?;
    *state.settings.lock().unwrap() = s.clone();
    Ok(s)
}

#[tauri::command]
fn set_auto_copy(app: AppHandle, state: State<'_, AppState>, on: bool) -> Result<UserSettings, String> {
    let mut s = state.settings.lock().unwrap().clone();
    s.auto_copy = on;
    save_settings(&app, &s)?;
    *state.settings.lock().unwrap() = s.clone();
    Ok(s)
}

/// Videos on the clipboard: files copied in Explorer or Finder, or paths/links copied as text.
/// `auto` (window got focus): only a copy made since we last looked, only things that look like videos, never
/// our own outputs, and only with "Convert copied videos" on. Ctrl+V / ⌘V (`auto` = false) takes whatever is there.
#[tauri::command]
fn clipboard_sources(state: State<'_, AppState>, auto: bool) -> Vec<String> {
    #[cfg(any(windows, target_os = "macos"))]
    {
        let seq = native::clipboard_seq();
        {
            let mut last = state.clip_seq.lock().unwrap();
            if auto && (!state.settings.lock().unwrap().auto_paste || *last == seq) {
                return vec![];
            }
            *last = seq;
        }
        let found = match native::read_clipboard() {
            native::Clip::Files(f) => f,
            native::Clip::Text(t) => clip::sources_from_text(&t),
            native::Clip::Empty => vec![],
        };
        if auto {
            found.into_iter().filter(|s| clip::looks_like_video(s) && !clip::is_own_output(s)).collect()
        } else {
            found
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (state, auto);
        vec![]
    }
}

/// Converted videos onto the clipboard, the same way Explorer's Ctrl+C (Finder's ⌘C) does, so a paste anywhere
/// takes them. Not async: Tauri runs sync commands on the main thread, which OLE needs.
#[tauri::command]
fn copy_files(state: State<'_, AppState>, paths: Vec<String>) -> Result<(), String> {
    #[cfg(any(windows, target_os = "macos"))]
    {
        let p: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        #[cfg(windows)]
        native::copy_files(&p).map_err(|e| e.message().to_string())?;
        #[cfg(target_os = "macos")]
        native::copy_files(&p)?;
        // Our own copy is not a new video to convert on the next focus.
        *state.clip_seq.lock().unwrap() = native::clipboard_seq();
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (state, paths);
        Ok(())
    }
}

/// Drags converted videos out of the window (to browsers, Teams, Explorer, Finder …). Main thread; on Windows
/// it blocks until the drop, on macOS AppKit runs the drag and sends `drag-ended` when it is over.
#[tauri::command]
fn drag_files(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    #[cfg(any(windows, target_os = "macos"))]
    {
        let w = app.get_webview_window("main").ok_or("No window")?;
        let p: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        #[cfg(windows)]
        {
            let hwnd = w.hwnd().map_err(|e| e.to_string())?;
            native::drag_files(hwnd, &p).map_err(|e| e.message().to_string())
        }
        #[cfg(target_os = "macos")]
        {
            let view = w.ns_view().map_err(|e| e.to_string())?;
            native::drag_files(&app, view, &p)
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (app, paths);
        Ok(())
    }
}

#[tauri::command]
fn cancel(state: State<'_, AppState>) {
    state.cancelled.store(true, Ordering::SeqCst);
    if let Some(c) = state.child.lock().unwrap().as_mut() {
        let _ = c.kill();
    }
}

#[tauri::command]
async fn convert(app: AppHandle, state: State<'_, AppState>, id: u32, path: String, preset: Preset) -> Result<Done, String> {
    let ffmpeg = state.ffmpeg.lock().unwrap().clone().ok_or("Still getting ready, try again in a moment.")?;
    let out_dir = state.settings.lock().unwrap().output_dir.clone().map(PathBuf::from);
    let work = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("work");
    let downloads = app.path().download_dir().ok();
    let child_slot = state.child.clone();
    let cancelled = state.cancelled.clone();
    cancelled.store(false, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        if clip::is_url(&path) {
            // Links: saved to the output folder, or Downloads.
            let dir = out_dir.or(downloads).ok_or("No folder to save to. Pick one in Settings.")?;
            run_link(&app, &ffmpeg, id, &path, &dir, &work, preset, &child_slot, &cancelled)
        } else {
            run(&app, &ffmpeg, id, Path::new(&path), out_dir.as_deref(), preset, &child_slot, &cancelled)
        }
    })
        .await
        .map_err(|e| e.to_string())?
}

fn no_window(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// A link: HLS playlists go straight to ffmpeg; anything else is downloaded with the system's own curl first.
#[allow(clippy::too_many_arguments)]
fn run_link(
    app: &AppHandle,
    ffmpeg: &Path,
    id: u32,
    url: &str,
    dir: &Path,
    work: &Path,
    preset: Preset,
    child_slot: &Arc<Mutex<Option<Child>>>,
    cancelled: &AtomicBool,
) -> Result<Done, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Can't use the output folder {}: {e}", dir.display()))?;
    let output = output_in(dir, &clip::url_stem(url));
    if clip::is_hls(url) {
        let out_bytes = encode(app, ffmpeg, id, url, &output, preset, child_slot, cancelled)?;
        return Ok(Done { output: output.to_string_lossy().into_owned(), in_bytes: 0, out_bytes });
    }
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let tmp = work.join(format!("download-{}-{id}.{}", std::process::id(), clip::url_extension(url)));
    let result = download(app, id, url, &tmp, child_slot, cancelled)
        .and_then(|_| encode(app, ffmpeg, id, &tmp.to_string_lossy(), &output, preset, child_slot, cancelled));
    let _ = std::fs::remove_file(&tmp);
    let out_bytes = result?;
    Ok(Done { output: output.to_string_lossy().into_owned(), in_bytes: 0, out_bytes })
}

/// curl -L with its progress bar on stderr ("####   45.3%", lines ended by \r).
fn download(
    app: &AppHandle,
    id: u32,
    url: &str,
    dest: &Path,
    child_slot: &Arc<Mutex<Option<Child>>>,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let _ = app.emit("progress", Progress { id, pct: 0.0, stage: "download" });
    let mut cmd = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    cmd.args(["-L", "--fail", "--retry", "2", "-#", "-o"])
        .arg(dest)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = no_window(&mut cmd).spawn().map_err(|e| format!("Can't start the download: {e}"))?;
    let mut stderr = child.stderr.take().unwrap();
    *child_slot.lock().unwrap() = Some(child);

    let mut buf = Vec::new();
    let mut last_err = String::new();
    let mut last = -1.0;
    let mut chunk = [0u8; 512];
    loop {
        let n = std::io::Read::read(&mut stderr, &mut chunk).unwrap_or(0);
        if n == 0 {
            break;
        }
        for &b in &chunk[..n] {
            if b == b'\r' || b == b'\n' {
                let line = String::from_utf8_lossy(&buf).trim().to_string();
                buf.clear();
                if let Some(pct) = clip::curl_percent(&line) {
                    if pct - last >= 0.5 {
                        last = pct;
                        let _ = app.emit("progress", Progress { id, pct: pct.min(100.0), stage: "download" });
                    }
                } else if !line.is_empty() {
                    last_err = line;
                }
            } else {
                buf.push(b);
            }
        }
    }
    let status = child_slot.lock().unwrap().take().map(|mut c| c.wait());
    if cancelled.load(Ordering::SeqCst) {
        return Err("Cancelled".into());
    }
    match status {
        Some(Ok(s)) if s.success() => Ok(()),
        _ => Err(if last_err.contains("404") {
            "The link doesn't point to a video (404).".into()
        } else if last_err.is_empty() {
            "Couldn't download the video.".into()
        } else {
            format!("Couldn't download the video: {}", last_err.trim_start_matches("curl: "))
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    app: &AppHandle,
    ffmpeg: &Path,
    id: u32,
    input: &Path,
    out_dir: Option<&Path>,
    preset: Preset,
    child_slot: &Arc<Mutex<Option<Child>>>,
    cancelled: &AtomicBool,
) -> Result<Done, String> {
    let in_bytes = std::fs::metadata(input).map_err(|_| "File not found.".to_string())?.len();
    if let Some(d) = out_dir {
        std::fs::create_dir_all(d).map_err(|e| format!("Can't use the output folder {}: {e}", d.display()))?;
    }
    let output = output_path(input, out_dir);
    let out_bytes = encode(app, ffmpeg, id, &input.to_string_lossy(), &output, preset, child_slot, cancelled)?;
    Ok(Done { output: output.to_string_lossy().into_owned(), in_bytes, out_bytes })
}

/// Runs ffmpeg from `input` (path or URL) to `output`, reporting progress. Returns the output size.
#[allow(clippy::too_many_arguments)]
fn encode(
    app: &AppHandle,
    ffmpeg: &Path,
    id: u32,
    input: &str,
    output: &Path,
    preset: Preset,
    child_slot: &Arc<Mutex<Option<Child>>>,
    cancelled: &AtomicBool,
) -> Result<u64, String> {
    let _ = app.emit("progress", Progress { id, pct: 0.0, stage: "convert" });
    let mut cmd = Command::new(ffmpeg);
    cmd.args(ffmpeg_args(input, output, &settings(preset)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = no_window(&mut cmd).spawn().map_err(|e| format!("Can't start ffmpeg: {e}"))?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    *child_slot.lock().unwrap() = Some(child);

    let duration = Arc::new(Mutex::new(None::<f64>));
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let err_thread = {
        let duration = duration.clone();
        let log = log.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(d) = parse_duration(&line) {
                    duration.lock().unwrap().get_or_insert(d);
                }
                let mut l = log.lock().unwrap();
                l.push(line);
                if l.len() > 40 {
                    l.remove(0);
                }
            }
        })
    };

    let mut last = -1.0;
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Some(us) = line.strip_prefix("out_time_us=").and_then(|v| v.trim().parse::<f64>().ok()) {
            if let Some(d) = *duration.lock().unwrap() {
                if d > 0.0 {
                    let pct = (us / 1e6 / d * 100.0).clamp(0.0, 99.5);
                    if pct - last >= 0.5 {
                        last = pct;
                        let _ = app.emit("progress", Progress { id, pct, stage: "convert" });
                    }
                }
            }
        }
    }
    let status = child_slot.lock().unwrap().take().map(|mut c| c.wait());
    let _ = err_thread.join();

    let ok = matches!(status, Some(Ok(s)) if s.success());
    if cancelled.load(Ordering::SeqCst) || !ok {
        let _ = std::fs::remove_file(output);
        if cancelled.load(Ordering::SeqCst) {
            return Err("Cancelled".into());
        }
        return Err(friendly_error(&log.lock().unwrap(), output));
    }
    Ok(std::fs::metadata(output).map(|m| m.len()).unwrap_or(0))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let (s, had_file) = load_settings(app.handle());
            let st = app.state::<AppState>();
            if had_file {
                // Whatever is on the clipboard at start counts as new: the app may have been opened right after copying.
                *st.clip_seq.lock().unwrap() = u32::MAX;
            } else {
                // First start ever: don't convert something copied before the user knew what this app does. Save the
                // defaults so the next start is a normal one.
                st.first_run.store(true, Ordering::SeqCst);
                #[cfg(any(windows, target_os = "macos"))]
                {
                    *st.clip_seq.lock().unwrap() = native::clipboard_seq();
                }
                let _ = save_settings(app.handle(), &s);
            }
            *st.settings.lock().unwrap() = s;
            Ok(())
        })
        .on_window_event(|w, e| {
            // However the window goes away, don't leave ffmpeg running in the background.
            if let tauri::WindowEvent::Destroyed = e {
                let st = w.state::<AppState>();
                st.cancelled.store(true, Ordering::SeqCst);
                let mut slot = st.child.lock().unwrap();
                if let Some(c) = slot.as_mut() {
                    let _ = c.kill();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            prepare, initial_files, pick_files, reveal, cancel, convert, get_settings, pick_output_dir,
            reset_output_dir, set_auto_paste, set_auto_copy, set_preset, first_run, clipboard_sources, copy_files,
            drag_files
        ])
        .build(tauri::generate_context!())
        .expect("error while running Clipiccolo")
        .run(|_app, _event| {
            // macOS: files opened with the app (Finder "Open with", a drop on the Dock icon) arrive here, not as
            // command-line arguments.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let files = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .filter(|p| p.is_file())
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                open_files(_app, files);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration() {
        assert_eq!(parse_duration("  Duration: 00:00:44.99, start: 0.000000, bitrate: 23377 kb/s"), Some(44.99));
        assert_eq!(parse_duration("  Duration: 01:02:03.50, start"), Some(3723.5));
        assert_eq!(parse_duration("  Duration: N/A, start"), None);
    }

    #[test]
    fn names() {
        let d = std::env::temp_dir().join("cs-test-names");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let input = d.join("Screen Recording (1).mov");
        assert_eq!(output_path(&input, None), d.join("Screen Recording (1) - piccolo.mp4"));
        std::fs::write(d.join("Screen Recording (1) - piccolo.mp4"), b"x").unwrap();
        assert_eq!(output_path(&input, None), d.join("Screen Recording (1) - piccolo (2).mp4"));
        let other = d.join("out");
        assert_eq!(output_path(&input, Some(&other)), other.join("Screen Recording (1) - piccolo.mp4"));
    }
}
