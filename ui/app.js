// Clipiccolo UI: a queue of dropped files, converted one at a time by the Rust side.
const T = window.__TAURI__;
const invoke = T.core.invoke;
const win = T.window.getCurrentWindow();
const $ = (id) => document.getElementById(id);

// macOS: the system's traffic lights instead of our own buttons, ⌘ instead of Ctrl, Finder instead of Explorer.
const MAC = /Mac/.test(navigator.platform) || /Macintosh/.test(navigator.userAgent);
if (MAC) document.body.classList.add("mac");
const platformText = (s) => (MAC ? s.replace(/Ctrl\+/g, "⌘").replace(/Explorer/g, "Finder") : s);

const ICON = {
  wait: '<svg class="i" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>',
  busy: '<svg class="i spin" viewBox="0 0 24 24"><path d="M21 12a9 9 0 1 1-6.22-8.56" /></svg>',
  ok: '<svg class="i" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" /><path d="m8.5 12 2.5 2.5 4.5-5" /></svg>',
  err: '<svg class="i" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16.5v.01" /></svg>',
  info: '<svg class="i" viewBox="0 0 24 24"><rect x="8" y="2" width="8" height="4" rx="1" /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" /></svg>',
};

const PRESETS = {
  small: "Up to 1280 px · 30 fps · smallest file, fine for quick repros",
  balanced: "Up to 1920 px · 30 fps · UI text stays readable",
  sharp: "Up to 2560 px · 60 fps · for fine detail and smooth motion",
};

let preset = "balanced"; // the saved choice arrives with the settings
let settings = { outputDir: null, autoPaste: true, autoCopy: true, preset };
let showTip = false; // first start only
let pendingCopy = []; // finished since the last auto-copy
let ready = false;
let running = false;
let draggingOut = false; // our own drag passes over our own drop zone
let nextId = 1;
const items = []; // {id, path, name, state: wait|running|done|failed|cancelled, pct, result, error, preset}

// ---------- helpers ----------
const isLink = (p) => /^https?:\/\//i.test(p);
const baseName = (p) => p.split(/[\\/]/).pop();
// A link shows as its file name (or the site if it has none).
function displayName(p) {
  if (!isLink(p)) return baseName(p);
  try {
    const u = new URL(p);
    const last = u.pathname.split("/").filter(Boolean).pop();
    return (last && decodeURIComponent(last)) || u.host;
  } catch { return p; }
}
function fmtBytes(n) {
  if (n >= 1e9) return (n / 1e9).toFixed(2) + " GB";
  if (n >= 1e6) return (n / 1e6).toFixed(n >= 1e8 ? 0 : 1) + " MB";
  return Math.max(1, Math.round(n / 1e3)) + " KB";
}
function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

const runningText = (it) =>
  `${it.stage === "download" ? "Downloading" : isLink(it.path) && !it.stage ? "Connecting" : "Shrinking"} · ${Math.floor(it.pct)} %`;

// ---------- render ----------
function renderPreset() {
  for (const b of $("preset").querySelectorAll("button")) {
    const on = b.dataset.preset === preset;
    b.classList.toggle("on", on);
    b.setAttribute("aria-checked", on);
  }
  $("presetHint").textContent = PRESETS[preset];
}

function renderItem(it) {
  let li = document.getElementById("item-" + it.id);
  if (!li) {
    li = document.createElement("li");
    li.id = "item-" + it.id;
    $("queue").appendChild(li);
  }
  li.className = "item " + it.state;
  li.title = it.state === "done" ? "Drag into any app, an email or a folder" : "";
  let icon = ICON.wait, sub = "Waiting", extra = "", action = "";
  if (it.state === "running") {
    icon = ICON.busy;
    sub = runningText(it);
    extra = `<div class="bar"><b style="width:${it.pct}%"></b></div>`;
  } else if (it.state === "done") {
    icon = ICON.ok;
    const r = it.result;
    if (r.inBytes > 0) {
      const saved = Math.round((1 - r.outBytes / r.inBytes) * 100);
      sub = `${fmtBytes(r.inBytes)} → ${fmtBytes(r.outBytes)}` + (saved > 0 ? ` <b>${saved} % smaller</b>` : "");
    } else {
      sub = `Saved · ${fmtBytes(r.outBytes)}`; // from a link: no original size to compare with
    }
    action = `<span class="actions"><button class="btn small" data-copy="${it.id}" title="Copy the video to the clipboard">Copy</button>`
      + `<button class="btn ghost small" data-reveal="${it.id}" title="${platformText("Show in Explorer")}">Show</button></span>`;
  } else if (it.state === "failed") {
    icon = ICON.err;
    sub = esc(it.error);
    action = `<button class="btn ghost small" data-retry="${it.id}">Retry</button>`;
  } else if (it.state === "cancelled") {
    icon = ICON.err;
    sub = "Cancelled";
    action = `<button class="btn ghost small" data-retry="${it.id}">Retry</button>`;
  }
  li.innerHTML = `${icon}<div class="grow"><div class="name" title="${esc(it.path)}">${esc(it.name)}</div><div class="sub">${sub}</div>${extra}</div>${action}`;
}

function renderChrome() {
  const has = items.length > 0;
  $("queue").classList.toggle("hidden", !has);
  $("drop").classList.toggle("compact", has);
  const done = items.filter((i) => i.state === "done");
  const pending = items.filter((i) => i.state === "wait" || i.state === "running").length;
  $("cancelBtn").classList.toggle("hidden", !running);
  $("clearBtn").classList.toggle("hidden", running || !has);
  $("copyAllBtn").classList.toggle("hidden", done.length < 2);
  $("tip").classList.toggle("hidden", !showTip || has);
  for (const b of $("preset").querySelectorAll("button")) b.disabled = false;

  const idle = "Output: MP4 (H.264) — plays everywhere";
  let s = idle;
  if (running) s = `${items.length - pending + 1} of ${items.length}`;
  else if (done.length) {
    const inB = done.reduce((a, i) => a + i.result.inBytes, 0);
    const outB = done.reduce((a, i) => a + i.result.outBytes, 0);
    s = done.every((i) => i.result.inBytes > 0)
      ? `${done.length} done · ${fmtBytes(inB)} → ${fmtBytes(outB)}`
      : `${done.length} done · ${fmtBytes(outB)}`;
  }
  $("summary").textContent = s;
  $("summary").classList.toggle("nums", s !== idle); // counts and sizes read like a receipt
}

const folderName = (p) => p.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || p;
function renderSettings() {
  if (PRESETS[settings.preset] && settings.preset !== preset) { preset = settings.preset; renderPreset(); }
  const d = settings.outputDir;
  $("outDirText").textContent = d || "Next to the original video (links: Downloads)";
  $("outDirReset").classList.toggle("hidden", !d);
  $("autoPaste").checked = settings.autoPaste !== false;
  $("autoCopy").checked = settings.autoCopy !== false;
  $("dropText").textContent = d ? `Any format · saved to ${folderName(d)}` : "Any format · saved next to the original";
  $("dropText").title = d || "";
}

function showSettings(on) {
  $("home").classList.toggle("hidden", on);
  $("footer").classList.toggle("hidden", on);
  $("settings").classList.toggle("hidden", !on);
  $("backBtn").classList.toggle("hidden", !on);
  $("settingsBtn").classList.toggle("hidden", on);
  $("subtitle").textContent = on ? "Settings" : "Small MP4s, ready to share";
}

function setStatus(kind, title, text = "") {
  const el = $("status");
  if (!kind) { el.classList.add("hidden"); return; }
  el.className = "status " + kind;
  $("statusIcon").outerHTML = `<span id="statusIcon">${kind === "busy" ? ICON.busy : kind === "info" ? ICON.info : ICON.err}</span>`;
  $("statusTitle").textContent = title;
  $("statusText").textContent = text;
}

// Small toast above the footer, gone after a moment.
let toastTimer = 0;
function toast(text) {
  $("toastText").textContent = text;
  $("toast").classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => $("toast").classList.remove("show"), 2600);
}

// A hint that goes away by itself (unless something more important replaced it meanwhile).
let flashTimer = 0;
function flash(title, text) {
  setStatus("info", title, text);
  clearTimeout(flashTimer);
  flashTimer = setTimeout(() => { if ($("status").classList.contains("info")) setStatus(null); }, 4000);
}

// ---------- queue ----------
function add(paths) {
  // Our own results coming back (dragged onto ourselves, or copied and pasted) aren't new work.
  const outputs = new Set(items.filter((i) => i.result).map((i) => i.result.output.toLowerCase()));
  for (const path of paths) {
    if (outputs.has(path.toLowerCase())) continue;
    if (items.some((i) => i.path === path && (i.state === "wait" || i.state === "running"))) continue;
    const it = { id: nextId++, path, name: displayName(path), state: "wait", pct: 0 };
    items.push(it);
    renderItem(it);
  }
  renderChrome();
  pump();
}

async function pump() {
  if (running || !ready) return;
  const it = items.find((i) => i.state === "wait");
  if (!it) return;
  running = true;
  it.state = "running";
  it.pct = 0;
  it.stage = "";
  renderItem(it);
  renderChrome();
  try {
    it.result = await invoke("convert", { id: it.id, path: it.path, preset });
    it.state = "done";
    pendingCopy.push(it.result.output);
  } catch (e) {
    const msg = String(e);
    it.state = msg === "Cancelled" ? "cancelled" : "failed";
    it.error = msg;
  }
  running = false;
  renderItem(it);
  renderChrome();
  pump();
  autoCopy();
}

// ---------- events ----------
T.event.listen("progress", ({ payload }) => {
  const it = items.find((i) => i.id === payload.id);
  if (!it || it.state !== "running") return;
  it.pct = payload.pct;
  it.stage = payload.stage;
  const li = document.getElementById("item-" + it.id);
  const bar = li && li.querySelector(".bar b");
  const sub = li && li.querySelector(".sub");
  if (bar) bar.style.width = it.pct + "%";
  if (sub) sub.textContent = runningText(it);
});

T.webviewWindow.getCurrentWebviewWindow().onDragDropEvent((e) => {
  if (draggingOut) return;
  const t = e.payload.type;
  if (t === "enter" || t === "over") $("drop").classList.add("over");
  else if (t === "leave") $("drop").classList.remove("over");
  else if (t === "drop") {
    $("drop").classList.remove("over");
    add(e.payload.paths || []);
  }
});

async function browse() {
  const files = await invoke("pick_files");
  if (files.length) add(files);
}
$("drop").addEventListener("click", browse);
$("drop").addEventListener("keydown", (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); browse(); } });

$("preset").addEventListener("click", (e) => {
  const b = e.target.closest("button[data-preset]");
  if (!b) return;
  preset = b.dataset.preset;
  renderPreset();
  invoke("set_preset", { preset }).then((s) => { settings = s; }).catch(() => {});
});

// ---------- clipboard in: Ctrl+V, and on focus ----------
// Returns how many new videos it found.
let checkingClipboard = false;
async function fromClipboard(auto) {
  if (checkingClipboard) return 0;
  checkingClipboard = true;
  let n = 0;
  try {
    const found = await invoke("clipboard_sources", { auto });
    n = found.length;
    if (n) add(found);
    else if (!auto) flash("Nothing to paste", platformText("Copy a video file in Explorer, or a link to a video, then press Ctrl+V."));
  } catch {}
  checkingClipboard = false;
  return n;
}

// On focus: a video copied meanwhile comes first (it joins the batch); otherwise finished videos go to the clipboard.
async function onFocus() {
  if ((await fromClipboard(true)) === 0) autoCopy();
}
win.onFocusChanged(({ payload: focused }) => { if (focused) onFocus(); });

// ---------- converted files out: Copy, Ctrl+C, drag ----------
const outputsOf = (list) => list.filter((i) => i.state === "done" && i.result).map((i) => i.result.output);

async function copyOut(paths, button) {
  if (!paths.length) return false;
  try {
    await invoke("copy_files", { paths });
    if (button) {
      const label = button.textContent;
      button.textContent = "Copied";
      setTimeout(() => { button.textContent = label; }, 1500);
    }
    toast(paths.length === 1 ? "Fatto. Copied to clipboard." : `Fatto. ${paths.length} videos copied to clipboard.`);
    return true;
  } catch (e) {
    setStatus("err", "Couldn't copy", String(e));
    return false;
  }
}

// When a batch is done: finished videos onto the clipboard — now if the window has focus, else on the next focus.
let copying = false;
async function autoCopy() {
  if (copying || !pendingCopy.length || running || items.some((i) => i.state === "wait")) return;
  if (settings.autoCopy === false) { pendingCopy = []; return; }
  let focused = false;
  try { focused = await win.isFocused(); } catch {}
  if (!focused) return;
  copying = true;
  const paths = pendingCopy.filter((p) => items.some((i) => i.result && i.result.output === p));
  pendingCopy = [];
  if (paths.length && !(await copyOut(paths))) pendingCopy = paths.concat(pendingCopy);
  copying = false;
}

// Press on a finished row and move a few pixels: a real file drag that browsers, Teams, Explorer … accept.
let press = null;
$("queue").addEventListener("pointerdown", (e) => {
  if (e.button !== 0 || e.target.closest("button")) return;
  const li = e.target.closest("li.item.done");
  if (li) press = { x: e.clientX, y: e.clientY, id: +li.id.slice(5) };
});
window.addEventListener("pointermove", async (e) => {
  if (!press) return;
  if (!(e.buttons & 1)) { press = null; return; }
  if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < 6) return;
  const it = items.find((i) => i.id === press.id);
  press = null;
  if (!it || !it.result) return;
  draggingOut = true;
  let started = true;
  try { await invoke("drag_files", { paths: [it.result.output] }); }
  catch (err) { started = false; setStatus("err", "Couldn't drag the video", String(err)); }
  // Windows blocks until the drop; macOS returns at once and sends "drag-ended" when it is over.
  if (!MAC || !started) dragEnded();
});
window.addEventListener("pointerup", () => { press = null; });
function dragEnded() {
  draggingOut = false;
  $("drop").classList.remove("over");
}
T.event.listen("drag-ended", dragEnded);

$("queue").addEventListener("click", (e) => {
  const c = e.target.closest("[data-copy]");
  if (c) {
    const it = items.find((i) => i.id === +c.dataset.copy);
    if (it) copyOut(outputsOf([it]), c);
    return;
  }
  const r = e.target.closest("[data-reveal]");
  if (r) {
    const it = items.find((i) => i.id === +r.dataset.reveal);
    if (it && it.result) invoke("reveal", { path: it.result.output });
    return;
  }
  const t = e.target.closest("[data-retry]");
  if (t) {
    const it = items.find((i) => i.id === +t.dataset.retry);
    if (it) { it.state = "wait"; it.error = ""; renderItem(it); renderChrome(); pump(); }
  }
});

$("cancelBtn").addEventListener("click", () => {
  for (const i of items) if (i.state === "wait") { i.state = "cancelled"; renderItem(i); }
  invoke("cancel");
});
$("clearBtn").addEventListener("click", () => {
  items.length = 0;
  pendingCopy = [];
  $("queue").innerHTML = "";
  renderChrome();
});
$("copyAllBtn").addEventListener("click", (e) => copyOut(outputsOf(items), e.currentTarget));

function copyAll() {
  const out = outputsOf(items);
  if (out.length) copyOut(out);
  else flash("Nothing to copy yet", platformText("Finished videos go to the clipboard on their own; Ctrl+C copies them again later."));
}
document.addEventListener("keydown", (e) => {
  const k = e.key.toLowerCase();
  const mod = MAC ? e.metaKey : e.ctrlKey;
  if ((mod && k === "v") || (e.shiftKey && k === "insert")) {
    e.preventDefault();
    fromClipboard(false);
  } else if (mod && k === "c" && !String(window.getSelection())) {
    e.preventDefault();
    copyAll();
  }
});
if (MAC) {
  // ⌘V / ⌘C that went to the Edit menu instead of the key handler still arrive as paste/copy events.
  document.addEventListener("paste", (e) => { e.preventDefault(); fromClipboard(false); });
  document.addEventListener("copy", (e) => { if (!String(window.getSelection())) { e.preventDefault(); copyAll(); } });
}

$("autoPaste").addEventListener("change", async (e) => {
  try { settings = await invoke("set_auto_paste", { on: e.target.checked }); }
  catch (err) { setStatus("err", "Couldn't save the setting", String(err)); }
  renderSettings();
});
$("autoCopy").addEventListener("change", async (e) => {
  try { settings = await invoke("set_auto_copy", { on: e.target.checked }); }
  catch (err) { setStatus("err", "Couldn't save the setting", String(err)); }
  renderSettings();
});
// The tip shrinks away instead of vanishing (the Piccolo Echo), then leaves the layout.
$("tipClose").addEventListener("click", () => {
  const tip = $("tip");
  tip.classList.add("leaving");
  setTimeout(() => { showTip = false; tip.classList.remove("leaving"); renderChrome(); }, 200);
});
$("settingsBtn").addEventListener("click", () => showSettings(true));
$("backBtn").addEventListener("click", () => showSettings(false));
$("outDirBtn").addEventListener("click", async () => {
  try { settings = await invoke("pick_output_dir"); renderSettings(); } catch (e) { setStatus("err", "Couldn't save the setting", String(e)); }
});
$("outDirReset").addEventListener("click", async () => {
  try { settings = await invoke("reset_output_dir"); renderSettings(); } catch (e) { setStatus("err", "Couldn't save the setting", String(e)); }
});
$("minBtn").addEventListener("click", () => win.minimize());
$("closeBtn").addEventListener("click", () => { invoke("cancel"); win.close(); });

// Block the webview's own file-drop navigation.
window.addEventListener("dragover", (e) => e.preventDefault());
window.addEventListener("drop", (e) => e.preventDefault());

// Settings footer: the app version from the build, ffmpeg's from `prepare`.
async function showVersion(ffmpegVersion) {
  let v = "";
  try { v = await T.app.getVersion(); } catch {}
  $("version").textContent = ["Clipiccolo" + (v ? " v" + v : ""), ffmpegVersion ? "ffmpeg " + ffmpegVersion : ""].filter(Boolean).join(" · ");
}

// ---------- start ----------
if (MAC) {
  for (const el of document.querySelectorAll("[title]")) el.title = platformText(el.title);
  for (const el of document.querySelectorAll(".row .desc, #tip .grow")) el.textContent = platformText(el.textContent);
}
// macOS: files opened with the app while it runs (Finder "Open with", a drop on the Dock icon).
T.event.listen("open-files", ({ payload }) => add(payload || []));
renderPreset();
renderChrome();
renderSettings();
(async () => {
  try { settings = await invoke("get_settings"); renderSettings(); } catch {}
  try { showTip = await invoke("first_run"); renderChrome(); } catch {}
  const t = setTimeout(() => setStatus("busy", "Getting ready"), 400);
  try {
    const ffmpegVersion = await invoke("prepare");
    clearTimeout(t);
    showVersion(ffmpegVersion);
    setStatus(null);
    ready = true;
  } catch (e) {
    clearTimeout(t);
    setStatus("err", "Couldn't start the video engine", String(e));
    return;
  }
  const initial = await invoke("initial_files");
  if (initial.length) add(initial);
  else pump();
  // Started right after copying a video? The focus event may have fired before we were listening.
  if (await win.isFocused()) onFocus();
})();
