//! CoreMIDI (midir) wrapper: enumerate, open paired SEQTRAK ports, send, collect.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub struct PortPair {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}

pub fn list_ports() -> PortPair {
    let outputs = MidiOutput::new("seqtrak-list")
        .map(|o| {
            o.ports()
                .iter()
                .map(|p| o.port_name(p).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    let inputs = MidiInput::new("seqtrak-list")
        .map(|i| {
            i.ports()
                .iter()
                .map(|p| i.port_name(p).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    PortPair { outputs, inputs }
}

/// Open connection to the SEQTRAK. Picks the only SEQTRAK port, or the sole port.
pub struct Link {
    pub out: MidiOutputConnection,
    pub rx: mpsc::Receiver<MidiEvent>,
    _in_conn: MidiInputConnection<()>,
}

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub bytes: Vec<u8>,
    pub at: Instant,
}

impl MidiEvent {
    /// Hex string like "F0 43 10 ..." or "90 3C 64".
    pub fn hex(&self) -> String {
        self.bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
    pub fn label(&self) -> String {
        let b = &self.bytes;
        match b.first() {
            Some(0xF0) => "sysex".into(),
            Some(st) if st & 0xF0 == 0x90 && b.get(2) != Some(&0) => format!(
                "note_on ch{} note{} vel{}",
                (st & 0x0F) + 1,
                b[1],
                b.get(2).copied().unwrap_or(0)
            ),
            Some(st) if st & 0xF0 == 0x90 => format!("note_off ch{} note{}", (st & 0x0F) + 1, b[1]),
            Some(st) if st & 0xF0 == 0x80 => format!("note_off ch{} note{}", (st & 0x0F) + 1, b[1]),
            Some(st) if st & 0xF0 == 0xB0 => format!(
                "cc ch{} cc{} val{}",
                (st & 0x0F) + 1,
                b[1],
                b.get(2).copied().unwrap_or(0)
            ),
            Some(st) if st & 0xF0 == 0xC0 => format!("pc ch{} prog{}", (st & 0x0F) + 1, b[1]),
            Some(0xF8) => "clock".into(),
            Some(0xFA) => "start".into(),
            Some(0xFB) => "continue".into(),
            Some(0xFC) => "stop".into(),
            Some(0xFE) => "active_sense".into(),
            _ => "raw".into(),
        }
    }
}

fn port_index(names: Vec<String>, kind: &str) -> Result<usize, String> {
    names
        .iter()
        .position(|n| n.to_ascii_lowercase().contains("seqtrak"))
        .or(if names.len() == 1 { Some(0) } else { None })
        .ok_or_else(|| format!("no SEQTRAK {kind} port found; ports: {names:?}"))
}

/// Open an independent output connection for the sequencer tick thread, so its
/// note scheduling never contends with the shared `Link` mutex.
pub fn open_out() -> Result<MidiOutputConnection, String> {
    let m = MidiOutput::new("seqtrak-seq").map_err(|e| e.to_string())?;
    let ports = m.ports();
    let names: Vec<String> = ports
        .iter()
        .map(|p| m.port_name(p).unwrap_or_default())
        .collect();
    let idx = port_index(names, "output")?;
    m.connect(&ports[idx], "seqtrak-seq")
        .map_err(|e| e.to_string())
}

/// Open an input connection and block the calling thread, forwarding every raw
/// message (clock and realtime bytes included) to `on_event`. The callback
/// returns false to stop. Returns early when the port goes quiet after having
/// carried traffic (device unplugged / powered off). `first_event_timeout`
/// bounds how long to wait for the very first message; None waits forever.
pub fn run_clock_port(
    on_event: impl FnMut(&[u8]) -> bool + Send + 'static,
    first_event_timeout: Option<Duration>,
) -> Result<(), String> {
    run_clock_port_hooked(on_event, first_event_timeout, None)
}

/// Same as `run_clock_port`, but `on_connect` runs AFTER the input connection
/// is established and before the first message is awaited. Transport Start
/// must be sent from here: sending it before the port opens loses the first
/// clocks while midir connects, and the tick counter lags the unit's grid.
pub fn run_clock_port_after_connect(
    on_event: impl FnMut(&[u8]) -> bool + Send + 'static,
    first_event_timeout: Option<Duration>,
    on_connect: impl FnOnce() + Send + 'static,
) -> Result<(), String> {
    run_clock_port_hooked(on_event, first_event_timeout, Some(Box::new(on_connect)))
}

fn run_clock_port_hooked(
    mut on_event: impl FnMut(&[u8]) -> bool + Send + 'static,
    first_event_timeout: Option<Duration>,
    on_connect: Option<Box<dyn FnOnce() + Send>>,
) -> Result<(), String> {
    let inm = MidiInput::new("seqtrak-clock").map_err(|e| e.to_string())?;
    let ports = inm.ports();
    let names: Vec<String> = ports
        .iter()
        .map(|p| inm.port_name(p).unwrap_or_default())
        .collect();
    let idx = port_index(names, "input")?;
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let _conn = inm
        .connect(
            &ports[idx],
            "seqtrak-clock",
            move |_stamp, bytes, ()| {
                let _ = tx.send(bytes.to_vec());
            },
            (),
        )
        .map_err(|e| e.to_string())?;
    if let Some(hook) = on_connect {
        hook();
    }
    let started = Instant::now();
    let mut had_traffic = false;
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(bytes) => {
                had_traffic = true;
                if !on_event(&bytes) {
                    return Ok(());
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if had_traffic {
                    return Ok(()); // went silent: source is gone
                }
                if let Some(limit) = first_event_timeout
                    && started.elapsed() > limit
                {
                    return Err("no MIDI clock from the device".into());
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

/// Nudge CoreMIDI when a long-lived process stops seeing devices that a fresh
/// process does see (classic stuck-client after replug). No-op elsewhere.
pub fn midi_restart() {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            fn MIDIRestart() -> i32;
        }
        let _ = MIDIRestart();
    }
}

impl Link {
    /// Open the first SEQTRAK-looking pair. `prefer` indexes into outputs if needed.
    pub fn open(prefer: Option<usize>) -> Result<(Link, Vec<String>), String> {
        let outm = MidiOutput::new("seqtrak-out").map_err(|e| e.to_string())?;
        let out_ports = outm.ports();
        let out_names: Vec<String> = out_ports
            .iter()
            .map(|p| outm.port_name(p).unwrap_or_default())
            .collect();
        let out_idx = prefer
            .or_else(|| {
                out_names
                    .iter()
                    .position(|n| n.to_ascii_lowercase().contains("seqtrak"))
            })
            .or(if out_ports.len() == 1 { Some(0) } else { None })
            .ok_or_else(|| format!("no SEQTRAK output port found; ports: {out_names:?}"))?;

        let inm = MidiInput::new("seqtrak-in").map_err(|e| e.to_string())?;
        let in_ports = inm.ports();
        let in_names: Vec<String> = in_ports
            .iter()
            .map(|p| inm.port_name(p).unwrap_or_default())
            .collect();
        let in_idx = in_names
            .iter()
            .position(|n| n.to_ascii_lowercase().contains("seqtrak"))
            .or(if in_ports.len() == 1 { Some(0) } else { None })
            .ok_or_else(|| format!("no SEQTRAK input port found; ports: {in_names:?}"))?;

        let (tx, rx) = mpsc::channel();
        let in_conn = inm
            .connect(
                &in_ports[in_idx],
                "seqtrak-monitor",
                move |_stamp, bytes, ()| {
                    let _ = tx.send(MidiEvent {
                        bytes: bytes.to_vec(),
                        at: Instant::now(),
                    });
                },
                (),
            )
            .map_err(|e| e.to_string())?;

        let out = outm
            .connect(&out_ports[out_idx], "seqtrak-out")
            .map_err(|e| e.to_string())?;
        Ok((
            Link {
                out,
                rx,
                _in_conn: in_conn,
            },
            out_names,
        ))
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.out.send(bytes).map_err(|e| e.to_string())
    }

    /// Drain events received within `wait` after now. Clock floods filtered out
    /// unless `include_clock`.
    pub fn recv_window(&self, wait: Duration, include_clock: bool) -> Vec<MidiEvent> {
        let deadline = Instant::now() + wait;
        let mut out = Vec::new();
        while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            match self
                .rx
                .recv_timeout(remaining.min(Duration::from_millis(50)))
            {
                Ok(ev) => {
                    if !include_clock && matches!(ev.bytes.first(), Some(0xF8) | Some(0xFE)) {
                        continue;
                    }
                    out.push(ev);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        out
    }

    /// Send sysex then wait for the bulk reply, stitching multi-block dumps.
    /// Envelope per chunk: [43 0n 7F 1C bh bl 0C ah am al dd..dd cc]
    pub fn request_dump(&mut self, sysex: &[u8], timeout: Duration) -> Result<Vec<u8>, String> {
        self.send(sysex)?;
        let deadline = Instant::now() + timeout;
        let mut data: Vec<u8> = Vec::new();
        let mut got_any = false;
        while Instant::now() < deadline {
            let wait = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(wait.min(Duration::from_millis(200))) {
                Ok(ev) if ev.bytes.first() == Some(&0xF0) => {
                    got_any = true;
                    let inner = &ev.bytes[1..ev.bytes.len().saturating_sub(1)];
                    if inner.len() > 10 && inner[0] == 0x43 && inner[6] == 0x0C {
                        data.extend_from_slice(&inner[10..inner.len() - 1]); // drop checksum
                        if inner[7] == 0x12 {
                            break; // bulk footer block 12 00 00
                        }
                    }
                }
                Ok(_) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if got_any {
                        break; // quiet gap after partial data
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hex_format() {
        let ev = super::MidiEvent {
            bytes: vec![0x90, 0x3C, 0x64],
            at: std::time::Instant::now(),
        };
        assert_eq!(ev.hex(), "90 3C 64");
        assert_eq!(ev.label(), "note_on ch1 note60 vel100");
    }
}
