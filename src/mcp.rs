//! `st-mcp`: minimal MCP server over stdio (JSON-RPC 2.0, no runtime deps).
//! Hand-rolled because the tool surface is small and stable; swap to rmcp if
//! resources/prompts are ever needed (ponytail: tools only).

use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::cli::{TrackArg, TransportAction};

struct Tool {
    name: &'static str,
    description: &'static str,
    schema: Value,
}

fn tools() -> Vec<Tool> {
    vec![
    Tool {
        name: "st_list_ports",
        description: "List MIDI output/input ports. SEQTRAK ports contain 'SEQTRAK'.",
        schema: json!({"type": "object", "properties": {}}),
    },
    Tool {
        name: "st_identity",
        description: "Query the SEQTRAK identity (family + firmware version) via SysEx.",
        schema: json!({"type": "object", "properties": {}}),
    },
    Tool {
        name: "st_note",
        description: "Play one note on a track. Track: kick,snare,clap,hat1,hat2,perc1,perc2,synth1,synth2,dx,sampler.",
        schema: json!({"type": "object", "required": ["track"], "properties": {
            "track": {"type": "string"}, "note": {"type": "integer", "minimum": 0, "maximum": 127, "default": 60},
            "vel": {"type": "integer", "minimum": 1, "maximum": 127, "default": 100},
            "dur_ms": {"type": "integer", "default": 200}}}),
    },
    Tool {
        name: "st_cc",
        description: "Send a control change. cc is a name (volume,pan,eq_high,eq_low,mute,solo,drum_pitch,mono_poly,arp_template,arp_gate,arp_speed,portamento_time,portamento_switch,filter_resonance,eg_attack,filter_cutoff,eg_decay_release,reverb_send,delay_send,fx_master_1,fx_master_2,fx_master_3,fx_single_1,fx_single_2,fx_single_3,fm_algorithm,fm_mod_amount,fm_mod_frequency,fm_mod_feedback) or a raw number.",
        schema: json!({"type": "object", "required": ["track", "cc", "val"], "properties": {
            "track": {"type": "string"}, "cc": {"type": "string"},
            "val": {"type": "integer", "minimum": 0, "maximum": 127}}}),
    },
    Tool {
        name: "st_load_project",
        description: "Load project 1-8 into the unit (bank MSB 64 + PC).",
        schema: json!({"type": "object", "required": ["project"], "properties": {
            "project": {"type": "integer", "minimum": 1, "maximum": 8}}}),
    },
    Tool {
        name: "st_pc",
        description: "Bank select + program change. msb: 64 project, 63 sound, 62 sampler element, 32-38 drumkit(part). lsb: sound user 32-47, drumkit 2*part.",
        schema: json!({"type": "object", "required": ["msb", "prog"], "properties": {
            "track": {"type": "string", "default": "kick"},
            "msb": {"type": "integer"}, "lsb": {"type": "integer", "default": 0},
            "prog": {"type": "integer", "minimum": 0, "maximum": 127}}}),
    },
    Tool {
        name: "st_param",
        description: "SEQTRAK SysEx parameter change. address 'hh:mm:ll', data hex bytes. Checksum added automatically.",
        schema: json!({"type": "object", "required": ["address", "data"], "properties": {
            "address": {"type": "string"}, "data": {"type": "array", "items": {"type": "integer"}}}}),
    },
    Tool {
        name: "st_dump",
        description: "Bulk dump request (e.g. project edit buffer 11:00:00). Returns payload size; optionally writes hex to file.",
        schema: json!({"type": "object", "required": ["address"], "properties": {
            "address": {"type": "string"}, "out_file": {"type": "string"},
            "timeout_s": {"type": "integer", "default": 10}}}),
    },
    Tool {
        name: "st_sysex",
        description: "Send raw SysEx hex string like 'F0 43 10 7F 1C 0C 30 50 00 40 F7'.",
        schema: json!({"type": "object", "required": ["hex"], "properties": {"hex": {"type": "string"}}}),
    },
    Tool {
        name: "st_monitor",
        description: "Listen to MIDI events from the device for N seconds (clock flood filtered). Returns labeled events.",
        schema: json!({"type": "object", "properties": {
            "seconds": {"type": "integer", "default": 3}, "include_clock": {"type": "boolean", "default": false}}}),
    },
    Tool {
        name: "st_get",
        description: "Read one parameter via SysEx parameter request. Address 'hh:mm:ll'.",
        schema: json!({"type": "object", "required": ["address"], "properties": {
            "address": {"type": "string"}}}),
    },
    Tool {
        name: "st_snapshot",
        description: "Load a project (1-8, or 0 to listen without loading) and capture the parameter-change echo stream. Returns 'hh:mm:ll\\tvalue' lines. Best full-state readout.",
        schema: json!({"type": "object", "properties": {
            "project": {"type": "integer", "minimum": 0, "maximum": 8, "default": 1},
            "seconds": {"type": "integer", "default": 5}}}),
    },
    Tool {
        name: "st_transport",
        description: "Transport: start or stop the SEQTRAK sequencer (device MIDI sync must be External/Auto).",
        schema: json!({"type": "object", "required": ["action"], "properties": {
            "action": {"type": "string", "enum": ["start", "stop"]}}}),
    },
    Tool {
        name: "st_rec_steps",
        description: "Compose into the unit: fire notes on the unit's clock on the given 1-based 16th steps so its real-time recorder captures them into the track's pattern. Arm REC on the unit first. steps like '1,5,9,13'. Optionally select a pattern slot (1-6) on the track first; each track holds 6 slots and the recorder writes the currently selected one.",
        schema: json!({"type": "object", "required": ["track", "steps"], "properties": {
            "track": {"type": "string", "enum": ["kick","snare","clap","hat1","hat2","perc1","perc2","synth1","synth2","dx","sampler"]},
            "steps": {"type": "string"},
            "note": {"type": "integer", "minimum": 0, "maximum": 127, "default": 60},
            "vel": {"type": "integer", "minimum": 1, "maximum": 127, "default": 100},
            "bars": {"type": "integer", "minimum": 1, "maximum": 8, "default": 1},
            "gate": {"type": "integer", "minimum": 1, "maximum": 16, "default": 1},
            "keep_playing": {"type": "boolean", "default": false},
            "pattern": {"type": "integer", "minimum": 1, "maximum": 6,
                "description": "Pattern slot 1-6 to select on the track before recording (each track holds 6 slots; the recorder writes the currently selected one)"}}}),
    },
    ]
}

fn run_tool(name: &str, args: &Value) -> Result<Value, String> {
    use crate::cli::{run, Cli, Command};
    let mk = |command| Cli { device: 0, port: None, json: false, command };
    let call = |command| run(&mk(command)).map(Value::String);
    match name {
        "st_list_ports" => call(Command::List),
        "st_identity" => call(Command::Identity),
        "st_note" => call(Command::Note {
            track: parse_track(&args["track"])?,
            note: args.get("note").and_then(Value::as_u64).unwrap_or(60) as u8,
            vel: args.get("vel").and_then(Value::as_u64).unwrap_or(100) as u8,
            dur: args.get("dur_ms").and_then(Value::as_u64).unwrap_or(200),
        }),
        "st_cc" => call(Command::Cc {
            track: parse_track(&args["track"])?,
            cc: args["cc"].as_str().ok_or("cc required")?.to_string(),
            val: args.get("val").and_then(Value::as_u64).unwrap_or(0) as u8,
        }),
        "st_load_project" => call(Command::Load {
            project: args.get("project").and_then(Value::as_u64).ok_or("project required")? as u8,
        }),
        "st_pc" => call(Command::Pc {
            msb: args.get("msb").and_then(Value::as_u64).ok_or("msb required")? as u8,
            lsb: args.get("lsb").and_then(Value::as_u64).unwrap_or(0) as u8,
            prog: args.get("prog").and_then(Value::as_u64).ok_or("prog required")? as u8,
            track: match args.get("track") {
                Some(v) if !v.is_null() => parse_track(v)?,
                _ => TrackArg::Kick,
            },
        }),
        "st_param" => call(Command::Param {
            address: args["address"].as_str().ok_or("address required")?.to_string(),
            data: args["data"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64()).map(|v| format!("{v:02X}")).collect()).unwrap_or_default(),
        }),
        "st_dump" => call(Command::Dump {
            address: args["address"].as_str().ok_or("address required")?.to_string(),
            out: args.get("out_file").and_then(Value::as_str).map(String::from),
            timeout: args.get("timeout_s").and_then(Value::as_u64).unwrap_or(10),
        }),
        "st_sysex" => call(Command::Sysex {
            hex: args["hex"].as_str().ok_or("hex required")?.to_string(),
            wait: args.get("wait_ms").and_then(Value::as_u64).unwrap_or(500),
        }),
        "st_monitor" => call(Command::Monitor {
            timeout: Some(args.get("seconds").and_then(Value::as_u64).unwrap_or(3)),
            clock: args.get("include_clock").and_then(Value::as_bool).unwrap_or(false),
        }),
        "st_get" => call(Command::Get {
            address: args["address"].as_str().ok_or("address required")?.to_string(),
        }),
        "st_snapshot" => call(Command::Snapshot {
            project: args.get("project").and_then(Value::as_u64).unwrap_or(1) as u8,
            seconds: args.get("seconds").and_then(Value::as_u64).unwrap_or(5),
        }),
        "st_transport" => call(Command::Transport {
            action: match args["action"].as_str() {
                Some("start") => TransportAction::Start,
                Some("stop") => TransportAction::Stop,
                other => return Err(format!("bad action {other:?}")),
            },
            bpm: 120.0,
            seconds: None,
        }),
        "st_rec_steps" => call(Command::RecSteps {
            track: parse_track(&args["track"])?,
            steps: args["steps"].as_str().ok_or("steps required")?.to_string(),
            note: args.get("note").and_then(Value::as_u64).unwrap_or(60) as u8,
            vel: args.get("vel").and_then(Value::as_u64).unwrap_or(100) as u8,
            bars: args.get("bars").and_then(Value::as_u64).unwrap_or(1) as u8,
            gate: args.get("gate").and_then(Value::as_u64).unwrap_or(1) as u8,
            keep_playing: args.get("keep_playing").and_then(Value::as_bool).unwrap_or(false),
            pattern: args.get("pattern").and_then(Value::as_u64).map(|p| p as usize),
        }),
        other => Err(format!("unknown tool {other}")),
    }
}

fn parse_track(v: &Value) -> Result<TrackArg, String> {
    use TrackArg::*;
    match v.as_str() {
        Some("kick") => Ok(Kick),
        Some("snare") => Ok(Snare),
        Some("clap") => Ok(Clap),
        Some("hat1") => Ok(Hat1),
        Some("hat2") => Ok(Hat2),
        Some("perc1") => Ok(Perc1),
        Some("perc2") => Ok(Perc2),
        Some("synth1") => Ok(Synth1),
        Some("synth2") => Ok(Synth2),
        Some("dx") => Ok(Dx),
        Some("sampler") => Ok(Sampler),
        other => Err(format!("bad track {other:?}")),
    }
}

pub fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(_) => break,
        };
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2024-11-05"),
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "seqtrak-mcp", "version": env!("CARGO_PKG_VERSION")}
                }
            }),
            "notifications/initialized" | "notifications/cancelled" => continue,
            "ping" => json!({"jsonrpc": "2.0", "id": id, "result": {}}),
            "tools/list" => json!({
                "jsonrpc": "2.0", "id": id,
                "result": {"tools": tools().iter().map(|t| json!({
                    "name": t.name, "description": t.description, "inputSchema": t.schema
                })).collect::<Vec<_>>()}
            }),
            "tools/call" => {
                let name = msg["params"]["name"].as_str().unwrap_or("");
                let args = msg["params"]["arguments"].as_object().map(|m| Value::Object(m.clone())).unwrap_or(json!({}));
                match run_tool(name, &args) {
                    Ok(text) => json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {"content": [{"type": "text", "text": text}], "isError": false}
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {"content": [{"type": "text", "text": e}], "isError": true}
                    }),
                }
            }
            other => json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": -32601, "message": format!("method not found: {other}")}
            }),
        };
        let _ = writeln!(stdout, "{response}");
        let _ = stdout.flush();
    }
}

/// Exposed for tests and smoke checks.
pub fn tool_names() -> Vec<&'static str> {
    tools().iter().map(|t| t.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tools_registered() {
        let names = tool_names();
        assert_eq!(names.len(), 14);
        assert!(names.contains(&"st_rec_steps"));
        assert!(names.contains(&"st_pc"));
    }

    #[test]
    fn parse_track_variants() {
        assert!(matches!(parse_track(&json!("synth1")), Ok(TrackArg::Synth1)));
        assert!(matches!(parse_track(&json!("sampler")), Ok(TrackArg::Sampler)));
        assert!(parse_track(&json!("unknown")).is_err());
    }
}
