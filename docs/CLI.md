# CLI Reference (`st`)

English | [한국어](CLI.ko.md)

Complete command-line reference for `st`, the Yamaha SEQTRAK USB MIDI controller and pattern recorder. For first-time setup and hardware configuration, see [Developer Quickstart](QUICKSTART.md) ([한국어](QUICKSTART.ko.md)).

## Global options

Place global options before the subcommand:

```bash
st [--device <0-15>] [--port <INDEX>] [--json] <SUBCOMMAND>
```

| Option | Default | Description |
|---|---|---|
| `--device <N>` | `0` | Yamaha SysEx device number (`0` to `15`, maps to `10` to `1F` in the SysEx header) |
| `--port <INDEX>` | auto | Explicit MIDI output/input port index when multiple MIDI interfaces are connected |
| `--json` | `false` | Format output as single-line JSON for scripting |

## Track names (`<TRACK>`)

Every track-targeted command accepts one of these 11 fixed track names:

| Track name | MIDI channel (1-based) | Part index (`p` in hex) | Engine |
|---|---|---|---|
| `kick` | 1 | `0` | Drum (AWM2) |
| `snare` | 2 | `1` | Drum (AWM2) |
| `clap` | 3 | `2` | Drum (AWM2) |
| `hat1` | 4 | `3` | Drum (AWM2) |
| `hat2` | 5 | `4` | Drum (AWM2) |
| `perc1` | 6 | `5` | Drum (AWM2) |
| `perc2` | 7 | `6` | Drum (AWM2) |
| `synth1` | 8 | `7` | AWM2 poly synth |
| `synth2` | 9 | `8` | AWM2 poly synth |
| `dx` | 10 | `9` | 4-operator FM synth |
| `sampler` | 11 | `A` (10) | 7-slot user sampler |

## Subcommands

### 1. Connection & inspection

#### `st list`

Lists available MIDI output and input ports on the system.

```bash
st list
st --json list
```

#### `st identity`

Sends a Universal Device Inquiry (`F0 7E 7F 06 01 F7`) and prints the SEQTRAK family/member codes and firmware version (for example, `2.0` on OS 2.00).

```bash
st identity
```

#### `st monitor`

Prints incoming MIDI events from the SEQTRAK in real time. Timing clock (`0xF8`) and active sensing (`0xFE`) are filtered out by default unless `--clock` is passed.

```bash
st monitor --timeout 5          # listen for 5 seconds and exit
st monitor --timeout 2 --clock  # include 0xF8 MIDI clock bytes
```

### 2. Notes, Control Changes, and Sound / Project loading

#### `st note`

Sends a `Note On`, holds for `--dur` milliseconds, and sends `Note Off`.

```bash
st note -c kick -n 36
st note -c synth1 -n 60 -v 110 -d 500
```

- `-c, --track <TRACK>`: Target track (`kick` to `sampler`).
- `-n, --note <0-127>`: MIDI note number (drum tracks trigger on any note; sampler uses `60` to `66` for its 7 sample slots).
- `-v, --vel <1-127>`: Note velocity (default: `100`).
- `-d, --dur <MS>`: Hold duration in milliseconds (default: `200`).

#### `st cc`

Sends a Control Change on the track's channel using either a named CC or a raw number (`0` to `127`).

```bash
st cc -c synth1 -n volume -v 100
st cc -c dx -n fm_mod_amount -v 90
st cc -c kick -n mute -v 127    # mute kick (0 = unmute, >= 64 = mute)
```

See the [Named CC table](#named-cc-table) below for all 29 supported names.

#### `st load`

Loads project `1` to `8` on the unit (shorthand for Bank Select `MSB=64, LSB=0` + Program Change on channel 1). Switching projects triggers the unit's automatic save for the previously active project buffer.

```bash
st load 1
st load 4
```

#### `st pc`

Sends Bank Select (`CC 0` MSB, `CC 32` LSB) followed by `Program Change` (`1`-based program number `1` to `128`) on the selected track.

```bash
st pc -c synth1 -m 63 -l 1 -p 12   # load synth sound from category bank
st pc -c kick -m 32 -l 0 -p 5      # load kick variation
```

| Target | MSB (`-m`) | LSB (`-l`) | Program (`-p`) |
|---|---|---|---|
| Project (1 to 8) | `64` | `0` | `1` to `8` |
| Sound (track tone) | `63` | `0` to `16` (category) | `1` to `128` |
| Sampler element | `62` | `0` to `1` | `1` to `128` |
| Drum kit / piece | `32` to `38` | `0` to `1` | `1` to `128` |

### 3. SysEx parameters, dumps, and snapshots

#### `st param`

Builds and sends a Yamaha SEQTRAK Parameter Change SysEx (`F0 43 1n 7F 1C 0C ah am al dd... F7`). Address bytes can be separated by `:`, `.`, or spaces.

```bash
st param 00:00:10 01         # enable USB MIDI SysEx transmission
st param 30:50:0F 02         # switch kick (part 0) to pattern 3 (0-based: 00..05)
st param 30:57:16 00 20      # set synth1 (part 7) pattern length to 32 steps
```

#### `st get`

Sends a Parameter Request (`F0 43 3n 7F 1C 0C ah am al F7`) and prints the returned value bytes. Requires `st param 00:00:10 01` to be enabled once on the unit.

```bash
st get 00:7F:00              # read format version
st get 30:50:0F              # read active pattern index (00..05) on kick
```

#### `st dump`

Sends a Bulk Dump Request (`F0 43 2n 7F 1C 0C ah am al F7`), stitches multi-packet replies, verifies checksums, and outputs the payload hex to stdout or a file.

```bash
st dump 30:41:00                        # dump Send Reverb block
st dump 11:00:00 -o project.hex -t 15   # full bulk dump to file
```

#### `st sysex`

Sends arbitrary raw SysEx hex bytes (`F0 ... F7`) and prints any SysEx reply received within `--wait` milliseconds (default: `500`).

```bash
st sysex "F0 7E 7F 06 01 F7" --wait 500
```

#### `st snapshot`

Loads a project (`1` to `8`, or `0` to listen without loading) and captures the parameter-change stream echoed by the unit as tab-separated `hh:mm:ll<TAB>hex` lines.

```bash
st snapshot --project 1 --seconds 5 > project1.tsv
st snapshot --project 0 --seconds 3     # capture panel turns without reloading
```

### 4. Transport and pattern recording

#### `st transport`

Controls the sequencer transport. The unit's MIDI Sync setting must be `External` or `Auto`.

```bash
st transport start                      # send MIDI Start (0xFA)
st transport stop                       # send MIDI Stop (0xFC)
st transport clock --bpm 124 --seconds 8
```

#### `st rec-steps`

Records a 16th-note step pattern for a single note into the unit's real-time recorder, locked to the unit's 96 PPQ MIDI clock (6 clocks per 16th note). Arm the physical `REC` button on the SEQTRAK first.

```bash
# Multi-track session: pass --keep-playing on every take except the last
st rec-steps kick   --steps 1,5,9,13 --keep-playing
st rec-steps snare  --steps 5,13 --vel 95 --keep-playing
st rec-steps hat1   --steps 1,3,5,7,9,11,13,15 --vel 75
```

- `--steps <LIST>`: Comma-separated 1-based 16th-note steps (`1` to `16` for 1 bar, up to `128` for 8 bars).
- `--note <0-127>`: MIDI note number (default: `60`).
- `--vel <1-127>`: Velocity (default: `100`).
- `--bars <1-8>`: Number of 16-step bars to run (default: `1`).
- `--gate <TICKS>`: Note duration in 16th-note steps (default: `1`).
- `--keep-playing`: Do not send `Stop` (`0xFC`) at the end of the sweep so `REC` stays armed for the next take.

#### `st seq`

Fires multiple `(step, note)` events in a single clock-synced sweep from a stopped transport (or anchored to a running loop with `--follow`). Ideal for recording melodic synth lines, basslines, or chords in one pass without mid-session restarts.

```bash
# Record a 1-bar synth1 bassline in a single sweep
st seq synth1 --events 1:48,5:48,7:55,9:48,13:53,15:55 --vel 100 --bars 1 --gate 2
```

- `--events <STEP:NOTE,...>`: Comma-separated `step:note` pairs (`1`-based steps). Multiple pairs with the same step fire simultaneously as a chord (for example, `1:60,1:64,1:67`).
- `--vel <1-127>`: Velocity (default: `100`).
- `--bars <1-8>`: Number of bars in the sweep (default: `1`).
- `--gate <TICKS>`: Note length in 16th-note steps (default: `1`).
- `--keep-playing`: Keep transport running after the sweep finishes.
- `--clock-offset <0-5>`: Shift note firing by `0` to `5` MIDI clocks into each 16th-note step (default: `0`).

### 5. Web GUI server

#### `st serve`

Starts the local HTTP backend and static asset server (`gui/dist`) on `http://127.0.0.1:8787`.

```bash
st serve
```

## Named CC table

| Name | CC # | Range / behavior | Tracks |
|---|---|---|---|
| `portamento_time` | `5` | `0` to `127` | `synth1`, `synth2`, `dx`, `sampler` |
| `volume` | `7` | `0` to `127` | All (`1` to `11`) |
| `pan` | `10` | `1` to `127` (`64` = center) | All (`1` to `11`) |
| `eq_high` | `20` | `40` to `88` (`64` = 0 dB) | All (`1` to `11`) |
| `eq_low` | `21` | `40` to `88` (`64` = 0 dB) | All (`1` to `11`) |
| `mute` | `23` | `0`-`63` off, `64`-`127` muted | All (`1` to `11`) |
| `solo` | `24` | `0`-`63` off, `64`-`127` solo | All (`1` to `11`) |
| `drum_pitch` | `25` | `40` to `88` (`64` = center) | Drum tracks (`1` to `7`) |
| `mono_poly` | `26` | `0` mono, `64` poly, `127` chord | `synth1`, `synth2`, `dx` |
| `arp_template` | `27` | `0` off, `1` to `16` type | `synth1`, `synth2`, `dx`, `sampler` |
| `arp_gate` | `28` | `0` to `127` | `synth1`, `synth2`, `dx`, `sampler` |
| `arp_speed` | `29` | `0` to `10` | `synth1`, `synth2`, `dx`, `sampler` |
| `portamento_switch` | `65` | `0`-`63` off, `64`-`127` on | `synth1`, `synth2`, `dx`, `sampler` |
| `filter_resonance` | `71` | `0` to `127` (`64` = center offset) | All |
| `eg_attack` | `73` | `0` to `127` (`64` = center offset) | All |
| `filter_cutoff` | `74` | `0` to `127` (`64` = center offset) | All |
| `eg_decay_release` | `75` | `0` to `127` (`64` = center offset) | All |
| `reverb_send` | `91` | `0` to `127` | All (`1` to `11`) |
| `delay_send` | `94` | `0` to `127` | All (`1` to `11`) |
| `fx_master_1` | `102` | `0` to `127` (Master FX Param 1 / Level) | Channel 1 |
| `fx_master_2` | `103` | `0` to `127` (Master FX Param 2) | Channel 1 |
| `fx_master_3` | `104` | `0` to `127` (Master FX Param 3) | Channel 1 |
| `fx_single_1` | `107` | `0` to `127` (Single FX Param 1 / Level) | Active track |
| `fx_single_2` | `108` | `0` to `127` (Single FX Param 2) | Active track |
| `fx_single_3` | `109` | `0` to `127` (Single FX Param 3) | Active track |
| `fm_algorithm` | `116` | `0` to `127` | `dx` (channel 10) |
| `fm_mod_amount` | `117` | `0` to `127` | `dx` (channel 10) |
| `fm_mod_frequency` | `118` | `0` to `127` | `dx` (channel 10) |
| `fm_mod_feedback` | `119` | `0` to `127` | `dx` (channel 10) |

## Frequently used SysEx addresses

In the table below, `p` is the 0-based part index in hex (`0` for `kick` through `A` for `sampler`):

| Address | Data bytes | Meaning |
|---|---|---|
| `00:00:10` | `00` / `01` | USB MIDI SysEx transmit switch (`01` = On) |
| `00:00:26` | `00`..`7F` | Metronome volume (`00` = silent) |
| `30:5p:0F` | `00`..`05` | Select pattern `1` to `6` on part `p` |
| `30:5p:16` | `<MSB> <LSB>` | Pattern length in steps (`00 10` = 16, `00 20` = 32, `00 40` = 64, `01 00` = 128) |
| `30:41:00` | `MSB LSB` | Send Reverb type |
| `30:42:00` | `MSB LSB` | Send Delay type |
| `30:43:00` | `MSB LSB` | Master Effect type (category `0` to `7`, preset `0` to `7`) |
| `31:1p:00` | `28`..`58` | Part pitch / octave offset (`40` = center) |
| `31:2p:00` | `MSB LSB` | Track Single Effect (Insertion A) type |
| `31:4p:00` | 9 bytes | Sound LFO block (waveform, speed, depth, destinations) |
