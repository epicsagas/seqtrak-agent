# SEQTRAK GUI Design

English | [한국어](DESIGN.ko.md)

Subject: Web controller for the Yamaha SEQTRAK, served by the local `st serve` backend.

Audience: A musician or developer sitting at a desk with the SEQTRAK plugged in over USB, controlling transport, patterns, mixer levels, mute/solo, arpeggiator settings, and effects from a single browser window.

## Visual concept: "second faceplate"

The SEQTRAK is a two-tone hardware instrument: a warm charcoal chassis holding light-grey molded panel sections with tight silkscreen labels, lit by yellow and cyan pattern LEDs and the signature orange of the SEQTRAK ORANGE edition. The web controller follows the same visual language instead of looking like a generic dark DAW or SaaS dashboard. Controls resemble the physical keys, knobs, and touch strips they correspond to.

## Design tokens

| Token | Hex | Role |
|---|---|---|
| `chassis` | `#23211E` | Page background, warm charcoal |
| `panel` | `#D8D5CE` | Molded light-grey panel sections |
| `panel-deep` | `#C7C4BC` | Recessed control wells inside a panel |
| `ink` | `#211F1B` | Primary silkscreen label print |
| `ink-soft` | `#6E6A62` | Secondary labels and inactive readouts |
| `orange` | `#E8590C` | Primary accent for transport play, REC, and active views |
| `led-yellow` | `#FFC400` | Drum track family (tracks 1 to 7) and warm active states |
| `led-cyan` | `#12C7D6` | Synth and sampler family (tracks 8 to 11) and cool active states |
| `led-red` | `#E03A2F` | Error banner and warning states |

Typography uses `Noto Sans KR` (`@fontsource/noto-sans-kr`) throughout, with tabular numerals (`font-variant-numeric: tabular-nums`) for numeric readouts. Weight 600 is used for silkscreen labels and weight 400 for values.

## Layout and views

The interface provides two main views (`DEVICE` and `MIXER`) plus a toggleable `HELP` overlay:

- **DEVICE view** (`DeviceFace.svelte`): Full SVG replica of the hardware top and side panels (`1560x430` viewBox) with side keys, 11 track pattern knobs, `[ALL]` pattern knob, 2x8 drum step grid synced to the unit's MIDI clock, `[REC]` plus 7 scale keys, 4 Sound Design knobs, `[FX]` rotary, latching `MASTER/SINGLE` switch, and 3 touch sliders.
- **MIXER view**: Single-screen three-zone layout matching the hardware's zone dividers: `DRUMS` on the left (7 track strips and 7 drum trigger pads), `SYNTH / SAMPLER` in the center (4 track strips and the arpeggiator block), and `MASTER` on the right (`ALL` volume rotary, transport controls, project loader `P1` to `P8`, and incoming MIDI log).

ASCII wireframe (`MIXER` view):

```text
+-----------------------------------------------------------------------+
| SEQTRAK  OS 2.0  connected  link up            [DEVICE][MIXER][HELP]  |
+----------------------+---------------------------+--------------------+
| DRUMS                | SYNTH / SAMPLER           | MASTER             |
| [kick][snare][clap]  | [synth1][synth2]          |      _____         |
| [hat1][hat2]         | [dx    ][sampler]         |    /  ALL  \       |
| [perc1][perc2]       |  arp: template gate speed |   |  knob   |      |
| 7-pad drum triggers  |  mono/poly  portamento    |    \_______/       |
|                      |                           | [PLAY/PAUSE][STOP] |
|                      |                           | project P1-P8      |
|                      |                           | event log          |
+----------------------+---------------------------+--------------------+
```

## Core principles

1. Mirror the hardware rather than inventing extra chrome. If the hardware uses a rotary knob, draw a knob; if it uses a key, draw a key.
2. Format values like a hardware readout: tabular numerals, minimal units.
3. Use color strictly for zone identity and active state: yellow for drums, cyan for synths, orange for live transport/recording state.
4. Fit on a single desktop screen (`1440x900`) without vertical scrolling; stack zones cleanly on narrow viewports.
5. Limit animation to hardware-style LED feedback: transport pulse while playing and brief flashes on note triggers.

## Component inventory

- `DeviceFace` (`src/components/device/`): SVG hardware faceplate (`DeviceKnob`, `DeviceKey`, `TouchSlider`), real SysEx Pattern Select (1 to 6), MIDI-clock step grid, `REC` + scale keys, and FX controls.
- `AllKnob`: SVG rotary with 40 LED ticks; dragging or pressing arrow keys broadcasts CC 7 volume across all 11 tracks.
- `TrackStrip`: Track label, channel badge, volume and pan sliders, and `M` (mute), `S` (solo), and `HIT` (audition) buttons.
- `PadGrid`: 7 trigger pads for the 7 drum tracks (`kick` through `perc2`) with visual hit flash.
- `Transport`: `PLAY`/`PAUSE` and `STOP` buttons driving MIDI Start (`0xFA`) and Stop (`0xFC`).
- `ArpBlock`: Target track selector (`synth1`, `synth2`, `dx`), 16 arpeggiator templates, gate, speed, `mono`/`poly`/`chord` mode, and portamento controls.
- `EventLog`: Rolling 8-line view of incoming MIDI messages from `/api/monitor`.
- `HelpPanel`: Bilingual (`EN` / `KO`) in-app manual drawer for the active view.

## Copy rules

Button labels match the hardware silkscreen (`PATTERN`, `MUTE`, `SOLO`, `PLAY`, `STOP`). Error banners state the fix plainly: `"SEQTRAK not connected. Plug in USB and power on; st serve reconnects automatically."`
