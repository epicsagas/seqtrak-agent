# Developer Quickstart

English | [한국어](QUICKSTART.ko.md)

Shortest path to setting up, controlling, and recording patterns onto the Yamaha SEQTRAK from the CLI, an AI agent, or the web GUI. For the full protocol reference, see `.claude/skills/seqtrak/SKILL.md`. For the hardware faceplate mapping, see `docs/DEVICE_UX.md` ([한국어](DEVICE_UX.ko.md)).

## 0. Hardware checklist

- Requirements: Yamaha SEQTRAK running firmware **OS 2.00 or later** and a USB data cable (plus Rust and Node.js if building from source). All commands and SysEx addresses were implemented and tested on **OS 2.00**; other firmware versions may partially or completely fail to respond. If you run into version-specific issues, please open an [issue](https://github.com/epicsagas/seqtrak-agent/issues) with the exact symptoms or send a [pull request](../CONTRIBUTING.md) so we can review and merge it.
- Turn on the SEQTRAK and plug it in over USB. Running `st list` should list `SEQTRAK` under both outputs and inputs.
  - If no port appears, the unit may have auto-powered off. Turn it back on and run `st list` again.
- Enable USB SysEx transmission once on the unit:
  ```bash
  st param 00:00:10 01      # SYSTEM General: USB MIDI Transmit SysEx = On
  ```
- Set MIDI Sync on the unit to `External` or `Auto`. If MIDI Sync is set to `Internal`, transport commands and `rec-steps` will not work properly.
- Arming `REC` on the unit itself requires pressing the physical `REC` button on the hardware panel (no SysEx command exists for arming it).

## 1. Install or build

### Install prebuilt release (no Rust required)

```bash
# macOS & Linux
curl -fsSL https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/epicsagas/seqtrak-agent/main/install.ps1 | iex
```

### Build from source (for local development)

```bash
cargo build --release
cargo install --path . --locked                   # installs st and st-mcp onto PATH
cd gui && npm install && npm run build && cd ..   # builds gui/dist
st serve &                                        # serves http://127.0.0.1:8787
```

- `st serve` starts even if the SEQTRAK is unplugged and connects automatically once the unit appears. On macOS, it calls `MIDIRestart` if a CoreMIDI client gets stuck after replugging.
- Whenever you modify the Rust CLI or MCP code, run `cargo install --path . --locked` again so the installed binaries stay current.

## 2. Connect an AI agent (MCP + skill)

### Copy-paste prompt for your agent

Paste this into your AI coding agent (Claude Code, Codex CLI, Grok, Antigravity, Cursor, or similar) and it will install the binaries, MCP server, and skill for your setup:

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

JSON (`.mcp.json`, Claude Desktop, Grok, Antigravity, Cursor):

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

TOML (Codex CLI `~/.codex/config.toml`):

```toml
[mcp_servers.seqtrak]
command = "st-mcp"
args = []
```

- `st-mcp` exposes 15 tools, including `st_rec_steps` for recording patterns into the unit.
- Smoke test the MCP server from the shell:
  ```bash
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | st-mcp
  ```

## 3. Recording patterns into the unit

Because no SysEx message writes step notes directly, pattern composition uses the unit's real-time recorder:

```bash
st load 1                                  # select target project
# press the physical REC button on the SEQTRAK panel
st rec-steps kick   --steps 1,5,9,13 --keep-playing
st rec-steps snare  --steps 5,13 --vel 90 --keep-playing
st rec-steps synth1 --steps 1,5,9,13 --note 60 --gate 2    # final take stops transport
```

- Notes fire in lockstep with the unit's transmitted MIDI clock (96 PPQ, 6 ticks per 16th note) and land in the pattern for each track's MIDI channel (`kick` = channel 1).
- Stopping the transport (`0xFC`) ends the recording session and disarms `REC`. Pass `--keep-playing` on every take except the last one.
- To verify a take, run `st transport start` followed by `st monitor --timeout 3` and check the echoed `note_on` events.
- Projects save automatically on power-off or when switching projects (User Guide 4.2, no manual WRITE button exists).

## 4. Track, sound, and effect reference

- 11 fixed tracks on channels 1 to 11: `kick`, `snare`, `clap`, `hat1`, `hat2`, `perc1`, `perc2`, `synth1`, `synth2`, `dx`, `sampler`.
- Per-track pattern select (1 to 6): `st param 30:5p:0F <0-5>` (`p` = part 0 to 10, so `kick` is `30:50:0F`).
- Pattern length (1 to 128 steps): `st param 30:5p:16 <MSB> <LSB>` (16 steps = `00 10`).
- Named CCs: `volume`, `pan`, `mute`, `solo`, `filter_cutoff`, `filter_resonance`, `eg_attack`, `eg_decay_release`, `reverb_send`, `delay_send`, `fx_master_1..3`, `fx_single_1..3`, `arp_template`, `arp_gate`, `arp_speed`, `drum_pitch`, `fm_algorithm`, `fm_mod_amount`, `fm_mod_frequency`, `fm_mod_feedback`.
- Full SysEx block map: see "SysEx address map" in `.claude/skills/seqtrak/SKILL.md`.
- Project tempo: `st get` / `st param 30:40:76` (2-byte MSB/LSB, `00 78` = 120 BPM). Read and write verified on hardware.

## 5. Web GUI development (Prototype)

> **Note:** The web GUI is currently in prototype status and may not work accurately in all situations.

```bash
cd gui && npm run check && npm run build   # svelte-check + vite build (gui/dist)
st serve                                   # static file server + /api/*
```

- Two views: `DEVICE` (hardware faceplate replica with clock-synced step sequencer) and `MIXER` (strip layout for live control).
- GUI step sequencer API: `GET /api/seq`, `POST /api/seq {op: step|rec|clear|note}`. This layer runs in `st serve` and plays over MIDI without altering the unit's own pattern memory.
- Click the `HELP` button in the top bar for the in-app manual (supports English and Korean).

### Browser E2E testing (`browser-harness`)

1. Start `st serve &` and open `http://127.0.0.1:8787` in Chrome.
2. Dispatch synthetic `PointerEvent` (`pointerdown` followed by `pointerup`) on SVG controls for deterministic clicks.
3. Verify backend state via `/api/seq`, `/api/param?addr=...`, and `/api/health`.
4. Verify note output with `st monitor --timeout 3`.

## 6. Code map and adding features

```text
src/model.rs     channel, CC, bank, and SysEx address constants
src/sysex.rs     SysEx framing, parsing, checksum, version formatting
src/midi.rs      midir wrapper, dump stitching, clock port, MIDIRestart
src/recsteps.rs  clock-synced step recording (rec-steps)
src/serve.rs     axum HTTP server and GUI step sequencer
src/cli.rs       st subcommands
src/mcp.rs       st-mcp JSON-RPC server
gui/             Svelte 5 + TypeScript frontend
```

To add a new CLI and MCP command: add the variant to `Command` in `src/cli.rs`, handle it in `run()`, register the `Tool` and dispatch arm in `src/mcp.rs`, document it in `.claude/skills/seqtrak/SKILL.md`, and re-run `cargo install --path . --locked`.

## 7. Common troubleshooting

- `st get` or `st dump` returns nothing: make sure the USB SysEx transmit switch (`st param 00:00:10 01`) is enabled.
- `rec-steps` fails with `no MIDI clock`: the unit's MIDI Sync is set to `Internal`; change it to `External` or `Auto`.
- Device disappears from `st list`: the unit auto-powered off; press the power button to wake it up.
- Unfinished takes saved on project switch: `st load N` triggers the unit's auto-save for the current project buffer, so finish or clean up your take before loading another project.
