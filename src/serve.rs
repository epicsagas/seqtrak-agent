//! `st serve`: HTTP backend for the web GUI. Owns one MIDI link, exposes /api/*,
//! serves the built SPA from gui/dist.

use std::sync::Mutex;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::midi::Link;
use crate::model;
use crate::sysex;

struct AppState {
    link: Mutex<Option<Link>>,
    seq: std::sync::Arc<Seq>,
}

type Shared = std::sync::Arc<AppState>;

// ------------------------------------------------------------- step sequencer

/// GUI-owned step layer synced to the device's transmitted MIDI clock. The unit
/// streams F8 while running (MIDI Clock Out defaults On) at 96 PPQ, so 6 clocks
/// make one 16th. Steps fire notes through a dedicated output connection so the
/// tick loop never contends with HTTP handlers on the `Link` mutex.
struct Seq {
    /// Per track (index = channel - 1): 0 = off, drums 1, synth tracks note number.
    patterns: Mutex<Vec<[u8; 32]>>,
    /// (channel, note) fired on the current step, for the next tick's note-offs.
    active: Mutex<Vec<(u8, u8)>>,
    rec: std::sync::atomic::AtomicBool,
    playing: std::sync::atomic::AtomicBool,
    step: std::sync::atomic::AtomicUsize,
    sub: std::sync::atomic::AtomicUsize,
}

impl Seq {
    fn new() -> Self {
        Self {
            patterns: Mutex::new(vec![[0u8; 32]; model::TRACKS.len()]),
            active: Mutex::new(Vec::new()),
            rec: std::sync::atomic::AtomicBool::new(false),
            playing: std::sync::atomic::AtomicBool::new(false),
            step: std::sync::atomic::AtomicUsize::new(0),
            sub: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

/// Spawn the sequencer thread. It reopens its MIDI connections until the device
/// shows up, ticks on the device clock, and re-opens them again after the
/// device disappears (the clock reader returns once the source goes silent).
fn spawn_seq(seq: std::sync::Arc<Seq>) {
    std::thread::spawn(move || {
        let mut fails = 0usize;
        loop {
        let mut out = match crate::midi::open_out() {
            Ok(o) => o,
            Err(e) => {
                fails += 1;
                eprintln!("st seq: waiting for device ({e})");
                if fails.is_multiple_of(4) {
                    crate::midi::midi_restart();
                }
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        fails = 0;
        eprintln!("st seq: clock synced");
        let s2 = seq.clone();
        let _ = crate::midi::run_clock_port(
            move |bytes| {
            use std::sync::atomic::Ordering;
            match bytes.first() {
                Some(0xFA) | Some(0xFB) => {
                    s2.step.store(0, Ordering::Relaxed);
                    s2.sub.store(0, Ordering::Relaxed);
                    s2.playing.store(true, Ordering::Relaxed);
                    true
                }
                Some(0xFC) => {
                    s2.playing.store(false, Ordering::Relaxed);
                    let mut active = s2.active.lock().unwrap();
                    for (ch, note) in active.drain(..) {
                        let _ = out.send(&[0x80 | ch, note, 0x40]);
                    }
                    true
                }
                Some(0xF8) if s2.playing.load(Ordering::Relaxed) => {
                    if !s2.sub.fetch_add(1, Ordering::Relaxed).is_multiple_of(6) {
                        return true;
                    }
                    let step = s2.step.load(Ordering::Relaxed);
                    let mut active = s2.active.lock().unwrap();
                    for (ch, note) in active.drain(..) {
                        let _ = out.send(&[0x80 | ch, note, 0x40]);
                    }
                    let patterns = s2.patterns.lock().unwrap();
                    for (i, pat) in patterns.iter().enumerate() {
                        let v = pat[step];
                        if v == 0 {
                            continue;
                        }
                        let ch = i as u8;
                        let note = if i < 7 { 60 } else { v };
                        let _ = out.send(&[0x90 | ch, note, 100]);
                        active.push((ch, note));
                    }
                    s2.step.store((step + 1) % 32, Ordering::Relaxed);
                    true
                }
                _ => true,
            }
        },
        None,
    );
        seq.playing.store(false, std::sync::atomic::Ordering::Relaxed);
        eprintln!("st seq: clock lost, waiting for device");
        std::thread::sleep(Duration::from_secs(2));
        }
    });
}

fn track_index(name: &str) -> Result<usize, String> {
    model::TRACKS
        .iter()
        .position(|(n, _)| *n == name)
        .ok_or_else(|| format!("unknown track {name}"))
}

#[derive(Deserialize)]
struct SeqReq {
    op: String,
    #[serde(default)]
    track: String,
    #[serde(default)]
    step: i32,
    #[serde(default)]
    on: bool,
    #[serde(default)]
    note: u8,
}

async fn seq_get(State(state): State<Shared>) -> impl IntoResponse {
    use std::sync::atomic::Ordering;
    let seq = &state.seq;
    let patterns = seq.patterns.lock().unwrap();
    Json(json!({
        "playing": seq.playing.load(Ordering::Relaxed),
        "rec": seq.rec.load(Ordering::Relaxed),
        "step": seq.step.load(Ordering::Relaxed),
        "patterns": *patterns,
    }))
}

fn bad_req(e: String) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": e })))
}

async fn seq_post(State(state): State<Shared>, Json(req): Json<SeqReq>) -> impl IntoResponse {
    use std::sync::atomic::Ordering;
    let seq = &state.seq;
    match req.op.as_str() {
        "rec" => {
            seq.rec.store(req.on, Ordering::Relaxed);
            Ok(Json(json!({ "rec": req.on })))
        }
        "clear" => {
            let idx = track_index(&req.track).map_err(bad_req)?;
            seq.patterns.lock().unwrap()[idx] = [0; 32];
            Ok(Json(json!({ "cleared": req.track })))
        }
        "step" => {
            let idx = track_index(&req.track).map_err(bad_req)?;
            let step = usize::try_from(req.step).map_err(|_| bad_req("bad step".into()))?;
            if step >= 32 {
                return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "step 0-31" }))));
            }
            let v = if req.on {
                if idx < 7 {
                    1
                } else if req.note > 0 {
                    req.note
                } else {
                    60
                }
            } else {
                0
            };
            seq.patterns.lock().unwrap()[idx][step] = v;
            Ok(Json(json!({ "on": req.on })))
        }
        // Live input: audition the note now; while REC is armed and the
        // sequencer runs, also store it into the GUI layer at the current step
        // (the device records the same Note On itself when its own REC is up).
        "note" => {
            let idx = track_index(&req.track).map_err(bad_req)?;
            let ch = model::track_channel(&req.track).ok_or("unknown track").map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))))?;
            let note = if req.note > 0 { req.note } else { 60 };
            let recorded = seq.rec.load(Ordering::Relaxed) && seq.playing.load(Ordering::Relaxed);
            if recorded {
                let step = seq.step.load(Ordering::Relaxed);
                seq.patterns.lock().unwrap()[idx][step] = note;
            }
            let _ = with_link(&state, |link| link.send(&[0x90 | ch, note, 100]))?;
            tokio::time::sleep(Duration::from_millis(120)).await;
            with_link(&state, |link| {
                link.send(&[0x80 | ch, note, 0x40])?;
                Ok(json!({ "recorded": recorded }))
            })
        }
        other => Err((StatusCode::BAD_REQUEST, Json(json!({ "error": format!("unknown op {other}") })))),
    }
}

fn with_link<T: serde::Serialize>(
    state: &Shared,
    f: impl FnOnce(&mut Link) -> Result<T, String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut guard = state.link.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "link poisoned"}))))?;
    let link = guard.as_mut().ok_or_else(|| {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "seqtrak not connected"})))
    })?;
    f(link)
        .map(|v| Json(serde_json::to_value(v).unwrap_or(Value::Null)))
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({"error": e}))))
}

async fn with_link_blocking<T, F>(
    state: Shared,
    f: F,
) -> Result<Json<Value>, (StatusCode, Json<Value>)>
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&mut Link) -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(move || with_link(&state, f))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
}

#[derive(Deserialize)]
struct NoteReq {
    track: String,
    #[serde(default = "default_note")]
    note: u8,
    #[serde(default = "default_vel")]
    vel: u8,
    #[serde(default = "default_dur")]
    dur: u64,
}

fn default_note() -> u8 {
    60
}
fn default_vel() -> u8 {
    100
}
fn default_dur() -> u64 {
    200
}

#[derive(Deserialize)]
struct CcReq {
    track: String,
    cc: String,
    val: u8,
}

#[derive(Deserialize)]
struct PcReq {
    #[serde(default)]
    track: String,
    msb: u8,
    #[serde(default)]
    lsb: u8,
    prog: u8,
}

#[derive(Deserialize)]
struct LoadReq {
    project: u8,
}

#[derive(Deserialize)]
struct TransportReq {
    action: String,
}

#[derive(Deserialize)]
struct ParamWriteReq {
    addr: String,
    #[serde(default)]
    bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct ParamReadReq {
    addr: String,
}

#[derive(Deserialize)]
struct SnapshotReq {
    #[serde(default = "default_project")]
    project: u8,
    #[serde(default = "default_seconds")]
    seconds: u64,
}

fn default_project() -> u8 {
    1
}
fn default_seconds() -> u64 {
    4
}

#[derive(Deserialize)]
struct MonitorReq {
    #[serde(default = "default_seconds")]
    seconds: u64,
    #[serde(default)]
    clock: bool,
}

async fn health(State(state): State<Shared>) -> impl IntoResponse {
    let ports = crate::midi::list_ports();
    let link_ok = state.link.lock().map(|g| g.is_some()).unwrap_or(false);
    Json(json!({ "ok": true, "link": link_ok, "outputs": ports.outputs, "inputs": ports.inputs }))
}

async fn identity(State(state): State<Shared>) -> impl IntoResponse {
    with_link_blocking(state, |link| {
        link.send(&[0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7])?;
        let evs = link.recv_window(Duration::from_secs(2), false);
        match evs.iter().find(|e| e.bytes.first() == Some(&0xF0)) {
            Some(ev) => match sysex::parse(&ev.bytes) {
                Some(sysex::SeqtrakSysex::Identity { version, .. }) => {
                    Ok(json!({ "version": sysex::format_version(version), "raw": ev.hex() }))
                }
                _ => Ok(json!({ "raw": ev.hex() })),
            },
            None => Err("no identity reply".into()),
        }
    })
    .await
}

async fn note(State(state): State<Shared>, Json(req): Json<NoteReq>) -> impl IntoResponse {
    let ch = model::track_channel(&req.track)
        .ok_or_else(|| bad_req(format!("unknown track {}", req.track)))?;
    let _ = with_link(&state, |link| link.send(&[0x90 | ch, req.note, req.vel]))?;
    tokio::time::sleep(Duration::from_millis(req.dur.min(4000))).await;
    with_link(&state, |link| {
        link.send(&[0x80 | ch, req.note, 0x40])?;
        Ok(json!({ "sent": true }))
    })
}

async fn cc(State(state): State<Shared>, Json(req): Json<CcReq>) -> impl IntoResponse {
    with_link(&state, |link| {
        let ch = model::track_channel(&req.track).ok_or_else(|| format!("unknown track {}", req.track))?;
        let num = model::cc_number(&req.cc).ok_or_else(|| format!("unknown cc {}", req.cc))?;
        link.send(&[0xB0 | ch, num, req.val])?;
        Ok(json!({ "sent": true }))
    })
}

async fn pc(State(state): State<Shared>, Json(req): Json<PcReq>) -> impl IntoResponse {
    with_link(&state, |link| {
        let ch = if req.track.is_empty() { 0 } else { model::track_channel(&req.track).ok_or_else(|| format!("unknown track {}", req.track))? };
        link.send(&[0xB0 | ch, 0, req.msb])?;
        link.send(&[0xB0 | ch, 32, req.lsb])?;
        link.send(&[0xC0 | ch, req.prog])?;
        Ok(json!({ "sent": true }))
    })
}

async fn load(State(state): State<Shared>, Json(req): Json<LoadReq>) -> impl IntoResponse {
    with_link(&state, |link| {
        if !(1..=8).contains(&req.project) {
            return Err("project must be 1-8".into());
        }
        link.send(&[0xB0, 0, 64])?;
        link.send(&[0xB0, 32, req.project - 1])?;
        link.send(&[0xC0, 0])?;
        Ok(json!({ "loaded": req.project }))
    })
}

async fn transport(State(state): State<Shared>, Json(req): Json<TransportReq>) -> impl IntoResponse {
    with_link(&state, |link| {
        match req.action.as_str() {
            "start" => link.send(&[0xFA])?,
            "stop" => link.send(&[0xFC])?,
            other => return Err(format!("unknown action {other}")),
        }
        Ok(json!({ "action": req.action }))
    })
}

async fn param_write(State(state): State<Shared>, Json(req): Json<ParamWriteReq>) -> impl IntoResponse {
    with_link(&state, |link| {
        let addr = parse_addr(&req.addr)?;
        let msg = sysex::parameter_change(0, addr, &req.bytes).to_bytes();
        link.send(&msg)?;
        Ok(json!({ "sent": true }))
    })
}

async fn param_read(Query(q): Query<ParamReadReq>, State(state): State<Shared>) -> impl IntoResponse {
    with_link_blocking(state, move |link| {
        let addr = parse_addr(&q.addr)?;
        let req = [0xF0, 0x43, 0x30, 0x7F, 0x1C, 0x0C, addr[0], addr[1], addr[2], 0xF7];
        link.send(&req)?;
        let evs = link.recv_window(Duration::from_secs(2), false);
        for ev in &evs {
            if let Some(sysex::SeqtrakSysex::ParameterChange { address: a, data, .. }) = sysex::parse(&ev.bytes)
                && a == addr
            {
                return Ok(json!({ "addr": format!("{:02X}:{:02X}:{:02X}", a[0], a[1], a[2]), "bytes": data }));
            }
        }
        Err("no reply".into())
    })
    .await
}

async fn snapshot(State(state): State<Shared>, Json(req): Json<SnapshotReq>) -> impl IntoResponse {
    with_link_blocking(state, move |link| {
        if req.project > 0 {
            link.send(&[0xB0, 0, 64])?;
            link.send(&[0xB0, 32, req.project - 1])?;
            link.send(&[0xC0, 0])?;
        }
        let mut pairs = Vec::new();
        let end = std::time::Instant::now() + Duration::from_secs(req.seconds);
        while std::time::Instant::now() < end {
            for ev in link.recv_window(Duration::from_millis(200), false) {
                if let Some(sysex::SeqtrakSysex::ParameterChange { address: a, data, .. }) = sysex::parse(&ev.bytes) {
                    pairs.push(json!({
                        "addr": format!("{:02X}:{:02X}:{:02X}", a[0], a[1], a[2]),
                        "bytes": data,
                    }));
                }
            }
        }
        Ok(json!({ "params": pairs }))
    })
    .await
}

async fn monitor(State(state): State<Shared>, Json(req): Json<MonitorReq>) -> impl IntoResponse {
    with_link_blocking(state, move |link| {
        let evs = link.recv_window(Duration::from_secs(req.seconds), req.clock);
        Ok(json!({ "events": evs.iter().map(|e| json!({"label": e.label(), "hex": e.hex()})).collect::<Vec<_>>() }))
    })
    .await
}

fn parse_addr(s: &str) -> Result<[u8; 3], String> {
    let parts: Vec<&str> = s.split([':', '.']).filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return Err("addr needs 3 hex bytes like 30:50:00".into());
    }
    let mut out = [0u8; 3];
    for (o, p) in out.iter_mut().zip(parts) {
        *o = u8::from_str_radix(p, 16).map_err(|e| format!("bad addr byte: {e}"))?;
    }
    Ok(out)
}

/// Locate the built web GUI. Order: `$ST_GUI_DIR`, `./gui/dist` (source checkout),
/// `<exe dir>/gui` (extracted release archive), `<exe dir>/../share/seqtrak-agent/gui`
/// and `~/.local/share/seqtrak-agent/gui` (install.sh / install.ps1 layout).
/// Falls back to `./gui/dist` when none contains an `index.html`.
pub fn gui_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(d) = std::env::var_os("ST_GUI_DIR") {
        cands.push(PathBuf::from(d));
    }
    let cwd_dist = std::env::current_dir().unwrap_or_default().join("gui/dist");
    cands.push(cwd_dist.clone());
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        cands.push(exe_dir.join("gui"));
        cands.push(exe_dir.join("../share/seqtrak-agent/gui"));
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        cands.push(PathBuf::from(home).join(".local/share/seqtrak-agent/gui"));
    }
    cands.into_iter().find(|d| d.join("index.html").is_file()).unwrap_or(cwd_dist)
}

/// Run the server. Binds 127.0.0.1:8787, serves the web GUI (see `gui_dir`). Starts even without
/// the device attached and reconnects in the background.
pub async fn run() -> Result<(), String> {
    let link = match Link::open(None) {
        Ok((l, _)) => Some(l),
        Err(e) => {
            eprintln!("st serve: {e}; serving anyway, retrying in background");
            None
        }
    };
    let state: Shared =
        std::sync::Arc::new(AppState { link: Mutex::new(link), seq: std::sync::Arc::new(Seq::new()) });
    spawn_seq(state.seq.clone());

    // Reconnect the main link whenever the device (re)appears.
    let st2 = state.clone();
    tokio::spawn(async move {
        let mut fails = 0usize;
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let need = st2.link.lock().map(|g| g.is_none()).unwrap_or(false);
            if need {
                match Link::open(None) {
                    Ok((l, _)) => {
                        eprintln!("st serve: seqtrak connected");
                        fails = 0;
                        if let Ok(mut g) = st2.link.lock() {
                            *g = Some(l);
                        }
                    }
                    Err(e) => {
                        fails += 1;
                        eprintln!("st serve: reconnect failed ({e})");
                        if fails.is_multiple_of(4) {
                            crate::midi::midi_restart();
                        }
                    }
                }
            }
        }
    });

    let static_dir = gui_dir();
    eprintln!("st serve: web GUI from {}", static_dir.display());
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/identity", get(identity))
        .route("/api/note", post(note))
        .route("/api/cc", post(cc))
        .route("/api/pc", post(pc))
        .route("/api/load", post(load))
        .route("/api/transport", post(transport))
        .route("/api/param", post(param_write).get(param_read))
        .route("/api/snapshot", post(snapshot))
        .route("/api/monitor", post(monitor))
        .route("/api/seq", get(seq_get).post(seq_post))
        .fallback_service(
            tower_http::services::ServeDir::new(&static_dir)
                .append_index_html_on_directories(true),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787").await.map_err(|e| e.to_string())?;
    eprintln!("st serve: http://127.0.0.1:8787 (static: {})", static_dir.display());
    axum::serve(listener, app).await.map_err(|e| e.to_string())
}
