# Device Faceplate UX

English | [한국어](DEVICE_UX.ko.md)

Source of truth: SEQTRAK User Guide D0 section 1.2 (configuration) and section 2 (panel items 1 to 27), plus hardware verification on OS 2.00. Goal: make the `DEVICE` view in the web GUI behave like the physical instrument under a mouse or touch pointer.

> **Note:** The web GUI is currently in prototype status and some interactions or hardware states may not work accurately yet.

## Hardware panel to MIDI mapping

| Panel control | Hardware function (User Guide) | GUI behavior |
|---|---|---|
| `[VOL+]` / `[VOL-]` (1) | Speaker and headphone monitor level | Updates local monitor readout only; sends no MIDI |
| `[DELETE]` (2) | Delete pattern or project | Not exposed over MIDI (displays status hint) |
| `[MUTE]` / `[SOLO]` (3/4) | Arm mode, then press a track knob | Same arm-then-press workflow using CC 23 and CC 24 |
| `[PAGE]` (5) | Toggle drum step page 1-16 or 17-32 | Flips the 2x8 step grid between steps 1-16 and 17-32 |
| `[PLAY]` (6) | Start or stop the project sequencer | Sends MIDI Start/Stop; follows hardware PLAY button via incoming MIDI |
| `[ALL]` knob (7) | Switch patterns 1 to 6 across all 11 tracks | Stepped 1 to 6 rotary; broadcasts SysEx Pattern Select (`30:5p:0F`) |
| Track knobs (8) | Turn to select pattern 1 to 6; press to select track (and audition when stopped) | Same behavior; pressing also reads back the track's current pattern via SysEx |
| Drum keys (9) | Toggle 16th-note steps on or off for the selected track | Toggles steps in the GUI step layer, triggered on the unit's 96 PPQ MIDI clock |
| `[REC]` key (10) | Arm real-time recording | Arms the GUI step layer so scale keys record at the playhead while playing. If hardware REC is also armed, the unit records those same Note On messages into its own pattern |
| Synth keys (11) | Play the selected synth or sampler track in scale | `[REC]` plus 7 scale keys (`C4` to `B4`); auditions immediately, or records while `REC` is armed and playing |
| `[BAR LENGTH]` / `[OCTAVE]` / `[SCALE]` / `[KEY]` / `[REC SAMPLE]` (12-16) | Pattern length, octave, scale, key, sampling | Not mapped in the faceplate view (displays status hint; pattern length is available via SysEx `30:5p:16`) |
| Sound Design knobs (20) | Adjust 4 sound parameters of the selected track | Sends cutoff, resonance, attack, and decay offsets via CC 74, 71, 73, and 75 |
| `[FX]` knob (23) | Choose effect type | Selects category (MSB 0-7) and preset (LSB 0-7) via SysEx `30:43:00` (master) or `31:2p:00` (single) |
| `[MASTER/SINGLE]` (24) | Latching slide switch for FX target scope | Latching slide switch toggling between `MST` and `SINGLE` |
| `FX LEVEL` / `HIGH PASS` / `REPEATER` (25) | Touch performance strips | Page 1: `FX LEVEL` (CC 102/107), plus momentary `HIGH PASS` and `REPEATER` strips that spring back to 0 on release (sending neutral 64). Page 2: assigned FX parameters 1, 2, and 3 (CC 102-104 or 107-109) |
| `[CLEAR FX]` (22) | Minimize active effect level | Resets FX level to 0 and returns cutoff and gate to neutral 64 |

## Step layer and hardware clock sync

While playing, the SEQTRAK transmits MIDI timing clock at 96 PPQ (6 clocks per 16th note, MIDI Clock Out defaults to On). The `st serve` backend listens on a dedicated MIDI input connection, divides the clock into 16th-note ticks, and fires active steps from the GUI layer over a dedicated MIDI output connection. The frontend polls `GET /api/seq` every 120 ms so the playhead highlight and PLAY state follow the hardware panel even when playback is started on the unit itself.

The GUI step layer lives in `st serve` memory and leaves the unit's internal pattern memory untouched. Drum steps trigger note 60 on channels 1 to 7; synth steps store the recorded MIDI note number. Meanwhile, turning a track knob writes SysEx `30:5p:0F` (Pattern Select, 0 to 5) directly to the hardware.

## Hardware-verified corrections

- `MASTER/SINGLE` is a two-position latching slide switch, not two momentary buttons.
- `HIGH PASS` and `REPEATER` on FX Page 1 are momentary: releasing the strip snaps the value back to 0 and sends neutral CC 64, and Page 1 affects all tracks regardless of the `MASTER/SINGLE` switch.
- Turning the track knobs or `[ALL]` selects patterns 1 to 6 rather than changing volume. Hardware knob sensitivity is controlled by system offset `2F` (Pattern Select Sensitivity).
- The 16 drum keys form a 2x8 step grid for the currently selected track.
- The leftmost key in the synth key block is `[REC]`; the remaining 7 keys play scale notes.
- `[VOL+]` and `[VOL-]` only adjust speaker/headphone monitor level and do not send MIDI CC.

## Interaction rules

- **Knobs**: Drag vertically or scroll the mouse wheel (`Shift` for 5x steps). Pattern knobs step from 1 to 6. A quick click (< 150 ms without dragging) triggers the press action (select track and audition when stopped). Holding for 420 ms or longer mutes the track.
- **Keys**: Fire immediately on `pointerdown`. Lit cells indicate active steps; a dark border marks the current playhead step.
- **Touch sliders**: Drag vertically to sweep; double-click to reset; spring sliders (`HIGH PASS` and `REPEATER` on Page 1) return to 0 on release.
- **LED color language**: Yellow for drum tracks (1 to 7), cyan for synth and sampler tracks (8 to 11), and orange for transport, `REC`, and master controls.
- **Status display**: The virtual LCD in the upper right displays the current mode, selected track, pattern number, FX page, and last transmitted parameter.

## Rendering and resilience

The faceplate is built as a single responsive SVG (`1560x430` viewBox) with two-tone flat shading and crisp hit targets, avoiding WebGL overhead. `st serve` launches even when the hardware is disconnected, retries the MIDI connection every 2 seconds, and calls `MIDIRestart` on macOS if CoreMIDI stops reporting a replugged unit.
