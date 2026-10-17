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
