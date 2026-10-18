# seqtrak agent

English | [한국어](README.ko.md)

Control the Yamaha SEQTRAK from the terminal, an AI agent, or a browser. This repository builds a Rust CLI (`st`), an MCP server (`st-mcp`, 14 tools), a Claude skill, and a web GUI (`st serve`) that mirrors the hardware panel. It includes a MIDI-clock step sequencer and a recording helper that writes step patterns directly into the unit's pattern memory.

Communication uses class-compliant USB MIDI: notes, control changes, bank and program changes, and Yamaha SysEx parameter changes and bulk dumps.

> **Firmware compatibility (OS 2.00+):** This project requires Yamaha SEQTRAK firmware **OS 2.00 or later** and was built and tested against **OS 2.00** (SEQTRAK Data List D0). On other firmware versions, some or all features may not work as expected. If you encounter any compatibility issues, please open an [issue](https://github.com/epicsagas/seqtrak-agent/issues) with the exact symptoms (and output of `st identity`), or submit a [pull request](CONTRIBUTING.md) with a fix and we will gladly review and merge it.

## Install

### Prebuilt binaries (macOS, Linux, Windows)

Install the latest release (`st`, `st-mcp`, and the `seqtrak` skill) without Rust or Cargo:

```bash
# macOS & Linux
curl -fsSL https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.ps1 | iex
```

### Build from source

```bash
cargo build --release            # builds target/release/st and target/release/st-mcp
cargo install --path . --locked  # installs st and st-mcp onto PATH
```

## Connect an AI agent

### Copy-paste prompt for your agent

Paste this into your AI coding agent (Claude Code, Codex CLI, Grok, Antigravity, Cursor, or similar) and it will clone, build, and register the MCP server and skill for your setup:

```text
Read https://github.com/epicsagas/seqtrak-agent and install the CLI (`st`), the MCP server (`st-mcp`), and the `.claude/skills/seqtrak` skill for my current agent environment.
```

### Claude Code & Codex CLI

```bash
# Claude Code
claude mcp add seqtrak --scope user -- "$(command -v st-mcp)"
mkdir -p ~/.claude/skills && cp -r .claude/skills/seqtrak ~/.claude/skills/

# OpenAI Codex CLI
codex mcp add seqtrak -- "$(command -v st-mcp)"
mkdir -p ~/.codex/skills && cp -r .claude/skills/seqtrak ~/.codex/skills/
```

### MCP configuration files (JSON / TOML)

If your client configures MCP servers via a JSON file (such as `.mcp.json`, Claude Desktop, Grok, Antigravity, or Cursor):

```json
{
  "mcpServers": {
    "seqtrak": {
      "command": "st-mcp",
      "args": []
    }
  }
}
```

If your client uses TOML (such as Codex CLI `~/.codex/config.toml`):

```toml
[mcp_servers.seqtrak]
command = "st-mcp"
args = []
```

## What the MCP can do: from one note to a full arrangement

The MCP server (`st-mcp`) exposes the same 14 tools as the `st` CLI, so an agent can start with a one-line smoke test and end with a multi-bar, multi-track arrangement without leaving the conversation. The examples below are real tool calls; the agent composes them into a plan and the unit records the result.

### You talk, the agent plays

In practice you never call the tools yourself. You describe an outcome in your own language, and the agent picks the tools, plans the takes, drives the hardware, and verifies what actually landed:

- "트랙 재생해줘" or "hit the kick": a single `st_note`
- "키트 바꿔줘", "put a warmer pad on synth2": `st_pc` and maybe `st_cc`
- "리버브 좀 줄여줘": one `st_cc` with `reverb_send`
- "비트 하나 만들어줘": project load, pattern length, `st_rec_steps` takes, playback verification
- "Thriller 느낌으로 3마디 작곡해줘": a full session like Level 5 below. The agent plans each track, records, counts playback events per pitch, and refires only what is missing

The [seqtrak skill](.claude/skills/seqtrak/SKILL.md) teaches the agent the device model, the verified SysEx address map, and the firmware quirks, so the natural-language path stays safe: the agent plans, you listen, and you keep what you like.

### The 14 tools

| Tool | What it does |
|------|--------------|
| `st_list_ports` | List MIDI ports; SEQTRAK ports contain `SEQTRAK` |
| `st_identity` | Query device family and firmware version |
| `st_note` | Play one note on a track |
| `st_cc` | Control change by name or number: volume, pan, mute, solo, filter, EG, FX sends, drum pitch, arpeggiator |
| `st_pc` | Bank select + program change: project, preset and user sounds, drum kits, sampler elements |
| `st_load_project` | Load project 1 to 8 |
| `st_get` | Read one SysEx parameter |
| `st_param` | Write one SysEx parameter, checksum automatic |
| `st_sysex` | Send a raw SysEx hex string |
| `st_dump` | Bulk dump a memory block |
| `st_snapshot` | Load a project and capture its full parameter echo as TSV |
| `st_monitor` | Listen to incoming MIDI for N seconds, clock flood filtered |
| `st_transport` | Start or stop the sequencer |
| `st_rec_steps` | Record steps into the track's pattern through the real-time recorder |

### Level 1: see the device, hear one note

Check the connection and firmware, trigger a single hit, and watch the MIDI stream:

```text
st_identity    {}
st_note        {"track": "kick", "note": 36, "vel": 110, "dur_ms": 150}
st_monitor     {"seconds": 5}
```

### Level 2: sounds and mixer

Load a project, put a preset sound on a track, and mix it:

```text
st_load_project {"project": 4}
st_pc           {"msb": 63, "lsb": 0, "prog": 5, "track": "synth1"}
st_cc           {"track": "synth1", "cc": "volume", "val": 100}
st_cc           {"track": "synth1", "cc": "reverb_send", "val": 90}
```

### Level 3: deep parameters over SysEx

Any documented parameter is readable and writable: pattern lengths, effect types, send levels, arpeggiator, scale. Extend the kick pattern to 48 steps and check it stuck:

```text
st_get         {"address": "30:50:16"}                  # read pattern 1 length
st_param       {"address": "30:50:16", "data": [0, 48]} # write MSB+LSB = 48 steps
```

### Level 4: transport and step recording

Arm REC on the hardware panel (no SysEx exists for that), then the agent records steps onto the unit's own clock grid:

```text
st_rec_steps   {"track": "kick", "steps": "1,7,9,13", "vel": 110, "keep_playing": true}
st_rec_steps   {"track": "snare", "steps": "5,13", "vel": 100}
st_transport   {"action": "start"}
```

`st_monitor` during playback shows the recorded notes coming back, which is how the agent verifies a take landed.

### Level 5: a whole arrangement

A real session: a 3-bar, 7-track groove (drums, chromatic bass line, chord stabs, an FM lead) recorded into project 5. Drums are one call per track because a drum track fires one pitch everywhere:

```text
st_rec_steps   {"track": "kick",   "bars": 3, "steps": "1,7,9,13,17,23,25,29,33,39,41,45", "vel": 110, "keep_playing": true}
st_rec_steps   {"track": "snare",  "bars": 3, "steps": "5,13,21,29,37,45,46,47", "vel": 100, "keep_playing": true}
st_rec_steps   {"track": "hat1",   "bars": 3, "steps": "1,3,5,7,9,11,13,15,17,19,21,23,25,27,29,31,33,35,37,39,41,43,45,47", "vel": 70, "keep_playing": true}
st_rec_steps   {"track": "dx",     "bars": 3, "steps": "33", "note": 73, "gate": 2, "vel": 90, "keep_playing": true}
```

Melodic tracks need many pitches, so the CLI sweeps the whole line in one clock-synced pass (multi-pitch lines cannot be expressed in a single `st_rec_steps` call):

```bash
st seq synth1 --bars 3 --gate 2 --follow --anchor-ch 10 --anchor-note 73 --anchor-step 33 --loop-steps 48 \
  --events "1:42,3:42,5:47,7:47,9:46,11:46,13:45,15:44,17:42,19:42,21:40,23:40,25:42,27:42,29:44,31:42,33:49,35:49,37:47,39:47,41:46,43:46,45:44,47:42"
```

`--follow` anchors the sweep on a note the unit itself plays back (here the one-off DX C#5 on step 33), so the agent never touches the transport while recording. The agent then verifies the result by counting playback events per pitch against the plan and refires only what is missing. The full workflow, verified SysEx address map, and firmware quirks live in [`.claude/skills/seqtrak/SKILL.md`](.claude/skills/seqtrak/SKILL.md).

## Record patterns into the unit

The SEQTRAK does not expose a SysEx message for writing step notes directly. Instead, its real-time recorder captures incoming USB MIDI notes whenever REC is armed on the hardware panel. Run `st rec-steps` while REC is armed:

```bash
st rec-steps kick   --steps 1,5,9,13 --keep-playing
st rec-steps snare  --steps 5,13 --vel 90 --keep-playing
st rec-steps synth1 --steps 1,5,9,13 --note 60 --gate 2   # final take stops transport
```

`st rec-steps` locks onto the unit's 96 PPQ MIDI clock (6 ticks per 16th note) and sends notes on each track's channel so they land in that track's pattern. Projects save automatically on power-off or project switch (User Guide 4.2).

See [`.claude/skills/seqtrak/SKILL.md`](.claude/skills/seqtrak/SKILL.md) for the full workflow and verified SysEx address map, or [Developer Quickstart (`docs/QUICKSTART.md`)](docs/QUICKSTART.md) ([한국어](docs/QUICKSTART.ko.md)) for the step-by-step setup guide.

## Web GUI (Prototype)

> **Note:** The web GUI is currently in prototype status and some controls or states may not work accurately yet. For reliable control and pattern recording, use the `st` CLI or `st-mcp`.

```bash
st serve        # http://127.0.0.1:8787, serves the built web GUI
```

`st serve` looks for the GUI in this order: `$ST_GUI_DIR`, `./gui/dist` (source checkout), `gui/` next to the `st` binary (extracted release archive), then `~/.local/share/seqtrak-agent/gui` (installed by `install.sh` / `install.ps1`).

The web GUI loads the Noto Sans KR font from Google Fonts at runtime (no font files are bundled). Offline, it falls back to system fonts.

- **DEVICE view**: Hardware-accurate faceplate. Track knobs and `[ALL]` select patterns 1 to 6 via SysEx (`30:5p:0F`), drum keys toggle steps on a 2x8 grid, `[REC]` plus the 7 scale keys record live while playing, the `[FX]` knob switches effect types, `MASTER/SINGLE` is a latching slide switch, and `HIGH PASS` and `REPEATER` spring back to zero on release.
- **MIXER view**: Per-track mixer strips (volume, pan, mute, solo, audition hit), drum pads, arpeggiator controls, transport, project loader (P1 to P8), and incoming MIDI log.
- **Auto-reconnect**: Starts even when the unit is unplugged or powered off, polls for the USB port in the background, and calls `MIDIRestart` on macOS if a CoreMIDI client gets stuck after replugging.

The GUI step sequencer is exposed at `GET /api/seq` and `POST /api/seq` (`step`, `rec`, `clear`, `note`). See [Device Faceplate UX (`docs/DEVICE_UX.md`)](docs/DEVICE_UX.md) ([한국어](docs/DEVICE_UX.ko.md)), [Design System (`docs/DESIGN.md`)](docs/DESIGN.md) ([한국어](docs/DESIGN.ko.md)), and [GUI Architecture (`docs/GUI_PLAN.md`)](docs/GUI_PLAN.md) ([한국어](docs/GUI_PLAN.ko.md)).

## Quick start & CLI guide

For initial hardware setup and end-to-end workflows, read the [Developer Quickstart (`docs/QUICKSTART.md`)](docs/QUICKSTART.md) ([한국어](docs/QUICKSTART.ko.md)). For every subcommand, flag, named CC, bank table, and SysEx address, see the [CLI Reference (`docs/CLI.md`)](docs/CLI.md) ([한국어](docs/CLI.ko.md)).

```bash
st identity                    # check SEQTRAK connection and firmware version
st note -c kick -n 36          # trigger kick
st cc -c synth1 -n volume -v 100
st load 2                      # load project 2
st param 30:50:0F 02           # switch kick to pattern 3
st transport start             # start sequencer (unit MIDI Sync = External/Auto)
st snapshot --project 2        # capture full parameter state as TSV
st dump 11:00:00 -o project.hex
```

## Source layout

```text
src/model.rs     track channels, CC table, banks, SysEx address constants
src/sysex.rs     SysEx framing, parser, checksum, version formatter
src/midi.rs      midir wrapper, bulk dump stitching, clock listener, MIDIRestart
src/recsteps.rs  clock-synced step recording into the hardware recorder
src/serve.rs     axum HTTP server and MIDI-clock step sequencer
src/cli.rs       st CLI subcommands
src/mcp.rs       st-mcp stdio JSON-RPC MCP server
gui/             Svelte 5 + TypeScript web controller
docs/            quickstart, CLI reference, UX/design specs, review report
```

## Tests

```bash
cargo test && (cd gui && npm run check && npm run build)
```

## Trademarks

Yamaha and SEQTRAK are trademarks or registered trademarks of Yamaha Corporation. This project is an independent, unofficial open-source tool and is not affiliated with, endorsed by, or sponsored by Yamaha Corporation.
