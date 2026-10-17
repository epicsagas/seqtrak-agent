//! `st` CLI: control the Yamaha SEQTRAK over USB MIDI.

use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};

use crate::model;
use crate::sysex::{self, SeqtrakSysex};

#[derive(Parser)]
#[command(name = "st", about = "Control the Yamaha SEQTRAK over USB MIDI")]
pub struct Cli {
    /// MIDI device number (0 unless changed on the unit)
    #[arg(long, default_value_t = 0)]
    pub device: u8,
    /// Output/input port index when several MIDI devices exist
    #[arg(long)]
    pub port: Option<usize>,
    /// Emit JSON
    #[arg(long)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TrackArg {
    Kick,
    Snare,
    Clap,
    Hat1,
    Hat2,
    Perc1,
    Perc2,
    Synth1,
    Synth2,
    Dx,
    Sampler,
}

impl TrackArg {
    fn ch(self) -> u8 {
        match self {
            TrackArg::Kick => 0,
            TrackArg::Snare => 1,
            TrackArg::Clap => 2,
            TrackArg::Hat1 => 3,
            TrackArg::Hat2 => 4,
            TrackArg::Perc1 => 5,
            TrackArg::Perc2 => 6,
            TrackArg::Synth1 => 7,
            TrackArg::Synth2 => 8,
            TrackArg::Dx => 9,
            TrackArg::Sampler => 10,
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// List MIDI ports
    List,
    /// Universal identity request; prints model + firmware version
    Identity,
    /// Send a note (default: one hit with fixed duration)
    Note {
        #[arg(short = 'c', long, value_enum)]
        track: TrackArg,
        #[arg(short = 'n', long)]
        note: u8,
        #[arg(short = 'v', long, default_value_t = 100)]
        vel: u8,
        /// Hold time in ms before note off
        #[arg(short = 'd', long, default_value_t = 200)]
        dur: u64,
    },
    /// Send a control change. Use a name (volume, pan, mute, solo, arp_template, ...)
    /// or a raw number.
    Cc {
        #[arg(short = 'c', long, value_enum)]
        track: TrackArg,
        /// CC name or number
        #[arg(short = 'n', long)]
        cc: String,
        #[arg(short = 'v', long)]
        val: u8,
    },
    /// Bank select + program change. Loads a project, sound, sampler element or drum kit.
    Pc {
        /// Bank MSB (64 project, 63 sound, 62 sampler element, 32-38 drumkit)
        #[arg(short = 'm', long)]
        msb: u8,
        #[arg(short = 'l', long, default_value_t = 0)]
        lsb: u8,
        #[arg(short = 'p', long)]
        prog: u8,
        #[arg(short = 'c', long, value_enum, default_value = "kick")]
        track: TrackArg,
    },
    /// Send raw SysEx hex bytes, e.g. "F0 43 10 7F 1C 0C 30 50 00 40 F7"
    Sysex {
        hex: String,
        /// Reply wait window in ms
        #[arg(long, default_value_t = 500)]
        wait: u64,
    },
    /// Parameter change with auto checksum: st param 30:50:00 40
    Param {
        /// Address hex triple, e.g. 30:50:00 (colon, dot or space separated)
        address: String,
        /// Data bytes hex
        data: Vec<String>,
    },
    /// Bulk dump request; writes payload hex (or file with -o)
    Dump {
        address: String,
        #[arg(short = 'o', long)]
        out: Option<String>,
        /// Timeout seconds
        #[arg(short = 't', long, default_value_t = 10)]
        timeout: u64,
    },
    /// Listen to incoming MIDI from the device
    Monitor {
        /// Exit after N seconds (omit: run until Ctrl-C)
        #[arg(long)]
        timeout: Option<u64>,
        /// Include clock and active-sense flood
        #[arg(long)]
        clock: bool,
    },
    /// Sequencer transport (device must be in MIDI sync External mode)
    Transport {
        #[arg(value_enum)]
        action: TransportAction,
        /// BPM for the clock action
        #[arg(long, default_value_t = 120.0)]
        bpm: f64,
        /// Seconds to run the clock (omit: until Ctrl-C)
        #[arg(long)]
        seconds: Option<u64>,
    },
    /// Load project 1-8 onto the unit (shorthand for pc -m 64)
    Load {
        /// Project number 1-8
        #[arg(value_parser = clap::value_parser!(u8).range(1..=8))]
        project: u8,
    },
    /// Read one parameter via SysEx parameter request
    Get { address: String },
    /// HTTP backend for the web GUI (serves gui/dist on 127.0.0.1:8787)
    Serve,
    /// Load a project and capture the parameter-change echo stream as TSV
    /// ("hh:mm:ll<TAB>vv" per line). Best full-state readout.
    Snapshot {
        /// Project number 1-8 to load first (0: no load, just listen)
        #[arg(long, default_value_t = 1)]
        project: u8,
        /// Capture seconds
        #[arg(long, default_value_t = 5)]
        seconds: u64,
    },
    /// Fire notes on the unit's clock so its real-time recorder captures them.
    /// Arm REC on the unit first; notes land in the selected track's pattern.
    RecSteps {
        #[arg(value_enum)]
        track: TrackArg,
        /// 1-based 16th steps, comma separated: 1,5,9,13
        #[arg(long)]
        steps: String,
        /// MIDI note number (drum kits trigger on any note; 60 = the pad)
        #[arg(long, default_value_t = 60)]
        note: u8,
        #[arg(long, default_value_t = 100)]
        vel: u8,
        /// Bars to run (16 steps each)
        #[arg(long, default_value_t = 1)]
        bars: u8,
        /// Note length in 16th ticks
        #[arg(long, default_value_t = 1)]
        gate: u8,
        /// Leave the transport running after the window
        #[arg(long)]
        keep_playing: bool,
    },
    /// Fire many notes in ONE clock-synced sweep from a stopped transport
    /// (single Start, no mid-sweep restarts). Arm REC on the unit first.
    /// Events are 1-based step:note pairs: 1:42,3:42,5:47
    Seq {
        #[arg(value_enum)]
        track: TrackArg,
        #[arg(long)]
        events: String,
        #[arg(long, default_value_t = 100)]
        vel: u8,
        /// Bars to run (16 steps each)
        #[arg(long, default_value_t = 1)]
        bars: u8,
        /// Note length in 16th ticks
        #[arg(long, default_value_t = 1)]
        gate: u8,
        /// Leave the transport running after the sweep
        #[arg(long)]
        keep_playing: bool,
        /// Don't touch the transport; anchor on the unit's own loop-start
        /// note replay (needs a drum note on step 1 and playback running)
        #[arg(long)]
        follow: bool,
        /// Shift the fire grid by this many clocks into each step (0-5);
        /// boundary-timed notes can be dropped by the unit's recorder
        #[arg(long, default_value_t = 0)]
        clock_offset: usize,
        /// Follow anchor: MIDI channel of the unit's own replayed note
        #[arg(long, default_value_t = 1)]
        anchor_ch: u8,
        /// Follow anchor: note number (0 = any note on the channel)
        #[arg(long, default_value_t = 60)]
        anchor_note: u8,
        /// Follow anchor: the loop step that note plays on
        #[arg(long, default_value_t = 1)]
        anchor_step: usize,
        /// Unit loop length in steps (for follow step conversion)
        #[arg(long, default_value_t = 48)]
        loop_steps: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TransportAction {
    Start,
    Stop,
    /// Send timing clock at given BPM until stopped (needs --bpm)
    Clock,
}

pub fn run(cli: &Cli) -> Result<String, String> {
    match &cli.command {
        Command::List => {
            let ports = crate::midi::list_ports();
            Ok(if cli.json {
                serde_json::json!({ "outputs": ports.outputs, "inputs": ports.inputs }).to_string()
            } else {
                format!("outputs:\n{}", fmt_list(&ports.outputs)) + &format!("inputs:\n{}", fmt_list(&ports.inputs))
            })
        }
        Command::Identity => {
            let mut link = open(cli)?;
            let req = [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7];
            link.send(&req)?;
            let evs = link.recv_window(Duration::from_secs(2), false);
            let reply = evs.iter().find(|e| e.bytes.first() == Some(&0xF0));
            match reply {
                Some(ev) => match sysex::parse(&ev.bytes) {
                    Some(SeqtrakSysex::Identity { family, version }) => {
                        let ver = sysex::format_version(version);
                        Ok(format!("identity family={:02X}{:02X}{:02X} version={ver} raw={}", family[0], family[1], family[2], ev.hex()))
                    }
                    _ => Ok(format!("sysex reply: {}", ev.hex())),
                },
                None => Err("no identity reply (is the SEQTRAK on?)".into()),
            }
        }
        Command::Note { track, note, vel, dur } => {
            let mut link = open(cli)?;
            let ch = track.ch();
            link.send(&[0x90 | ch, *note, *vel])?;
            std::thread::sleep(Duration::from_millis(*dur));
            link.send(&[0x80 | ch, *note, 0x40])?;
            Ok(format!("note ch{} note{note} vel{vel} dur{dur}ms", ch + 1))
        }
        Command::Cc { track, cc, val } => {
            let mut link = open(cli)?;
            let ch = track.ch();
            let num = model::cc_number(cc).ok_or_else(|| format!("unknown cc '{cc}'; names: see --help or use a number"))?;
            link.send(&[0xB0 | ch, num, *val])?;
            Ok(format!("cc ch{} {num}({}) val{val}", ch + 1, model::cc_name(num)))
        }
        Command::Pc { msb, lsb, prog, track } => {
            let mut link = open(cli)?;
            let ch = track.ch();
            link.send(&[0xB0 | ch, 0, *msb])?;
            link.send(&[0xB0 | ch, 32, *lsb])?;
            link.send(&[0xC0 | ch, *prog])?;
            Ok(format!("bank msb{msb} lsb{lsb} prog{prog} on ch{}", ch + 1))
        }
        Command::Sysex { hex, wait } => {
            let mut link = open(cli)?;
            let bytes = parse_hex(hex)?;
            link.send(&bytes)?;
            let evs = link.recv_window(Duration::from_millis(*wait), false);
            Ok(format!("sent {} bytes{}", bytes.len(), fmt_replies(&evs)))
        }
        Command::Param { address, data } => {
            let mut link = open(cli)?;
            let addr = parse_addr(address)?;
            let mut bytes: Vec<u8> = Vec::new();
            for s in data {
                bytes.append(&mut parse_hex(s)?);
            }
            let msg = sysex::parameter_change(cli.device, addr, &bytes).to_bytes();
            link.send(&msg)?;
            let evs = link.recv_window(Duration::from_millis(500), false);
            Ok(format!("param {:02X}:{:02X}:{:02X} <- {}{}",
                addr[0], addr[1], addr[2],
                bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "),
                fmt_replies(&evs)))
        }
        Command::Dump { address, out, timeout } => {
            let mut link = open(cli)?;
            let addr = parse_addr(address)?;
            let req = sysex::dump_request(cli.device, addr).to_bytes();
            // The unit intermittently ignores bulk requests (busy TX or gate);
            // retry a few times before giving up.
            let per_try = Duration::from_secs((*timeout).max(1) / 3 + 1);
            let mut data = Vec::new();
            for _ in 0..3 {
                data = link.request_dump(&req, per_try)?;
                if !data.is_empty() {
                    break;
                }
            }
            if let Some(path) = out {
                let hexs: String = data.iter().map(|b| format!("{b:02X} ")).collect();
                std::fs::write(path, hexs.trim_end()).map_err(|e| format!("write failed: {e}"))?;
            }
            Ok(format!("dump {:02X}:{:02X}:{:02X} -> {} bytes{}", addr[0], addr[1], addr[2], data.len(),
                out.as_deref().map(|p| format!(" saved to {p}")).unwrap_or_default()))
        }
        Command::Monitor { timeout, clock } => {
            let link = open(cli)?;
            let wait = timeout.map(Duration::from_secs);
            let deadline = wait.map(|w| std::time::Instant::now() + w);
            let mut lines = Vec::new();
            loop {
                let evs = link.recv_window(Duration::from_millis(300), *clock);
                for ev in &evs {
                    let line = format!("{} {}", ev.label(), ev.hex());
                    println!("{line}");
                    lines.push(line);
                }
                if deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                    break;
                }
            }
            Ok(format!("monitor: {} events", lines.len()))
        }
        Command::Transport { action, bpm, seconds } => {            let mut link = open(cli)?;
            match action {
                TransportAction::Start => link.send(&[0xFA])?,
                TransportAction::Stop => link.send(&[0xFC])?,
                TransportAction::Clock => {
                    // 24 pulses per quarter note
                    let interval = Duration::from_secs_f64(60.0 / bpm / 24.0);
                    link.send(&[0xFA])?;
                    if let Some(s) = seconds {
                        let deadline = std::time::Instant::now() + Duration::from_secs(*s);
                        while std::time::Instant::now() < deadline {
                            link.send(&[0xF8])?;
                            std::thread::sleep(interval);
                        }
                        link.send(&[0xFC])?;
                    } else {
                        loop {
                            link.send(&[0xF8])?;
                            std::thread::sleep(interval);
                        }
                    }
                }
            }
            Ok(format!("transport {action:?} bpm={bpm}"))
        }
        Command::Load { project } => {
            let mut link = open(cli)?;
            let ch = 0;
            link.send(&[0xB0 | ch, 0, 64])?;
            link.send(&[0xB0 | ch, 32, project - 1])?;
            link.send(&[0xC0 | ch, 0])?;
            Ok(format!("load project {project}"))
        }
        Command::Get { address } => {
            let mut link = open(cli)?;
            let addr = parse_addr(address)?;
            // Parameter request: F0 43 3n 7F 1C 0C ah am al F7
            let req = [0xF0, 0x43, 0x30 | (cli.device & 0x0F), 0x7F, 0x1C, 0x0C, addr[0], addr[1], addr[2], 0xF7];
            link.send(&req)?;
            let evs = link.recv_window(Duration::from_secs(2), false);
            for ev in &evs {
                if let Some(SeqtrakSysex::ParameterChange { address: a, data, .. }) = sysex::parse(&ev.bytes)
                    && a == addr {
                        return Ok(format!("{:02X}:{:02X}:{:02X} = {}",
                            a[0], a[1], a[2], data.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")));
                    }
            }
            Err(format!("no reply for {:02X}:{:02X}:{:02X}", addr[0], addr[1], addr[2]))
        }
        Command::Serve => {
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("tokio runtime: {e}"))?;
            rt.block_on(crate::serve::run())
                .map(|_| "server stopped".into())
        }
        Command::Snapshot { project, seconds } => {
            let mut link = open(cli)?;
            if *project > 0 {
                let ch = 0;
                link.send(&[0xB0 | ch, 0, 64])?;
                link.send(&[0xB0 | ch, 32, project - 1])?;
                link.send(&[0xC0 | ch, 0])?;
            }
            let mut pairs: Vec<String> = Vec::new();
            let mut sysex_count = 0usize;
            let end = std::time::Instant::now() + Duration::from_secs(*seconds);
            while std::time::Instant::now() < end {
                for ev in link.recv_window(Duration::from_millis(200), false) {
                    if ev.bytes.first() != Some(&0xF0) {
                        continue;
                    }
                    sysex_count += 1;
                    if let Some(SeqtrakSysex::ParameterChange { address: a, data, .. }) = sysex::parse(&ev.bytes) {
                        pairs.push(format!("{:02X}:{:02X}:{:02X}\t{}",
                            a[0], a[1], a[2], data.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")));
                    }
                }
            }
            Ok(format!("snapshot: {sysex_count} sysex, {} params\n{}", pairs.len(), pairs.join("\n")))
        }
        Command::RecSteps { track, steps, note, vel, bars, gate, keep_playing } => {
            let parsed: Result<Vec<usize>, _> = steps
                .split(',')
                .map(|s| s.trim().parse::<usize>().map(|n| n.saturating_sub(1)))
                .collect();
            let steps = parsed.map_err(|_| "bad --steps; use comma separated 1-based steps like 1,5,9,13".to_string())?;
            crate::recsteps::record_steps(
                track.ch(),
                &steps,
                *note,
                *vel,
                *bars as usize,
                *gate as usize,
                *keep_playing,
            )
        }
        Command::Seq { track, events, vel, bars, gate, keep_playing, follow, clock_offset, anchor_ch, anchor_note, anchor_step, loop_steps } => {
            let parsed: Result<Vec<(usize, u8)>, String> = events
                .split(',')
                .map(|s| {
                    let p: Vec<&str> = s.trim().split(':').collect();
                    if p.len() != 2 {
                        return Err(format!("bad event `{s}`; use step:note like 1:42"));
                    }
                    let step = p[0]
                        .parse::<usize>()
                        .map(|n| n.saturating_sub(1))
                        .map_err(|_| format!("bad step in `{s}`"))?;
                    let note = p[1].parse::<u8>().map_err(|_| format!("bad note in `{s}`"))?;
                    Ok((step, note))
                })
                .collect();
            let events = parsed?;
            let ev = events.into_iter().map(|(s, n)| (s, n, *vel)).collect();
            crate::recsteps::seq(
                track.ch(),
                ev,
                *bars as usize,
                *gate as usize,
                *keep_playing,
                *follow,
                *clock_offset,
                // CLI channels are 1-based like the monitor prints; raw MIDI is 0-based
                anchor_ch.saturating_sub(1),
                *anchor_note,
                *anchor_step,
                *loop_steps,
            )
        }
    }
}

fn open(cli: &Cli) -> Result<crate::midi::Link, String> {
    let (link, ports) = crate::midi::Link::open(cli.port)?;
    let _ = ports;
    Ok(link)
}

fn fmt_list(v: &[String]) -> String {
    v.iter().enumerate().map(|(i, n)| format!("  [{i}] {n}\n")).collect()
}

fn fmt_replies(evs: &[crate::midi::MidiEvent]) -> String {
    if evs.is_empty() {
        String::new()
    } else {
        format!(" replies:{}", evs.iter().map(|e| format!(" {}", e.hex())).collect::<String>())
    }
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace() && *c != ',').collect();
    let cleaned = cleaned.strip_prefix("0x").unwrap_or(&cleaned);
    if !cleaned.len().is_multiple_of(2) {
        return Err(format!("hex string '{s}' must have an even number of digits"));
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).map_err(|e| format!("bad hex '{s}': {e}")))
        .collect()
}

fn parse_addr(s: &str) -> Result<[u8; 3], String> {
    let parts: Vec<&str> = s.split([':', '.', ' ']).filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return Err(format!("address needs 3 hex bytes, got '{s}'"));
    }
    let mut out = [0u8; 3];
    for (o, p) in out.iter_mut().zip(parts) {
        *o = u8::from_str_radix(p, 16).map_err(|e| format!("bad address byte '{p}': {e}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_addr_formats() {
        assert_eq!(parse_addr("30:50:0F"), Ok([0x30, 0x50, 0x0F]));
        assert_eq!(parse_addr("11.00.00"), Ok([0x11, 0x00, 0x00]));
        assert!(parse_addr("30:50").is_err());
    }

    #[test]
    fn parse_hex_inputs() {
        assert_eq!(parse_hex("F0 43 10"), Ok(vec![0xF0, 0x43, 0x10]));
        assert_eq!(parse_hex("0x02"), Ok(vec![0x02]));
        assert!(parse_hex("2").is_err());
    }
}
