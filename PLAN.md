# SEQTRAK Control Project Plan

English | [한국어](PLAN.ko.md)

Goal: Build a Rust CLI, MCP server, agent skill, and web GUI that control the Yamaha SEQTRAK over USB MIDI and SysEx, including recording patterns into the unit's own memory. Every milestone below is shipped and verified on hardware.

## Verified hardware facts

### Device
- USB: Yamaha VID `0x0499`, PID `0x174C`, class-compliant USB MIDI, no driver needed.
- Firmware reference: the official Yamaha SEQTRAK Data List for OS V2.00 (YJ-D0, 01/2026), available from Yamaha. The manual is not included in this repository.
- MIDI clock: The unit transmits 96 PPQ timing clock while playing (6 ticks per 16th note, MIDI Clock Out defaults to On).
- Power behavior: Auto power-off removes the USB device completely; `st serve` polls and reconnects automatically when it comes back.

### Fixed MIDI channels
Channels 1 to 7 are drums (`kick`, `snare`, `clap`, `hat1`, `hat2`, `perc1`, `perc2`), channels 8 and 9 are AWM2 synths (`synth1`, `synth2`), channel 10 is the 4-op FM synth (`dx`), and channel 11 is the sampler (`sampler`).

### SysEx framing (Model ID `1C 0C`)
- Parameter Change: `F0 43 1n 7F 1C 0C ah am al dd.. F7`
- Parameter Request: `F0 43 3n 7F 1C 0C ah am al F7`
- Dump Request: `F0 43 2n 7F 1C 0C ah am al F7`
- Bulk Dump: `F0 43 0n 7F 1C bh bl 0C ah am al dd.. cc F7` (checksum covers Model ID through the last data byte; device number `n = 0`).

### Key SysEx addresses (full map in `.claude/skills/seqtrak/SKILL.md`)
- System (`00 00 00`): offset `10` = SysEx TX switch, `2D` = Clock Out, `2F` = Pattern Select Sensitivity.
- Project Common (`30 40 00`): reads return 0 over SysEx; tempo cannot be read or written via SysEx.
- Send and Master FX: Send Reverb `30 41`, Send Delay `30 42`, Master FX 1 to 4 `30 43` to `30 46`, Master EQ `30 47`.
- Audio and scale: A/D `30 49` to `30 4B`, USB Audio `30 4C`, Scale `30 4D`.
- Project Track (`30 5p 00`): `00` volume, `01` pan, `0F` Pattern Select (0 to 5 for patterns 1 to 6), `16..` pattern lengths.
- Sound (`31 0p` name, `31 1p` general, `31 2p`/`31 3p` insertions, `31 4p` common) and Arpeggiator (`31 5p`, synth parts).
- AWM elements (`41 ep`, `42 ep`) and Sampler (`48 09 00` common, `50 eA 00` for 7 sample elements).

### Control Change numbers (channels 1 to 11)
Volume 7, pan 10, mute 23, solo 24, drum pitch 25, mono/poly/chord 26, arp template 27, arp gate 28, arp speed 29, EQ high/low 20/21, cutoff 74, resonance 71, attack 73, decay 75, reverb/delay sends 91/94, assigned FX 102 to 109, FM 116 to 119.

### Writing patterns into the unit
The Data List defines no SysEx for step note data. Instead, the unit's real-time recorder captures incoming USB MIDI notes while REC is armed on the hardware panel. `st rec-steps` (and MCP `st_rec_steps`) starts transport and fires notes on the unit's 96 PPQ clock grid. Because stopping transport ends the recording session, multi-track takes chain together with `--keep-playing` on all takes except the last.

## Architecture

```text
src/model.rs     channel, CC, bank, and SysEx address tables
src/sysex.rs     SysEx builder, parser, checksum, version formatter
src/midi.rs      midir wrapper, bulk dump stitching, clock port, MIDIRestart nudge
src/recsteps.rs  clock-synced step recording
src/serve.rs     axum HTTP server and MIDI-clock step sequencer
src/cli.rs       st commands (list, identity, note, cc, pc, sysex, param, dump,
                 monitor, transport, load, get, snapshot, serve, rec-steps)
src/mcp.rs       st-mcp stdio JSON-RPC server (14 tools)
gui/             Svelte 5 + TS web controller (DEVICE view, MIXER view, HELP overlay)
```

## Milestones
- [x] Protocol research: USB IDs, MIDI implementation, CC/bank/SysEx tables
- [x] Core library (`midi`, `sysex`, `model`) and unit tests
- [x] CLI commands and hardware verification
- [x] `st-mcp` server and JSON-RPC smoke tests
- [x] Claude skill, documentation, and local installation
- [x] Web GUI mixer view on `st serve` and browser E2E checks
- [x] Hardware-accurate DEVICE faceplate view (pattern knobs, 2x8 step grid, REC + scale keys, FX knob, latching scope switch, momentary sliders)
- [x] Backend step sequencer synced to hardware MIDI clock (`/api/seq`)
- [x] Hardware pattern composition via `rec-steps` with chained takes
- [x] Offline startup, background reconnect, and CoreMIDI `MIDIRestart` recovery
- [x] Bilingual documentation (English default + Korean `.ko.md`) and in-GUI manual

## Known protocol limits
- Step notes cannot be written via SysEx; use `st rec-steps` with the unit's recorder armed.
- Project tempo cannot be read or written over SysEx, and arming REC requires the physical panel button.
- Projects auto-save on power-off or project switch (no manual WRITE button exists).
- Song Position Pointer and song mode are not supported by the hardware.
