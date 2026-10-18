---
name: seqtrak
description: Control the Yamaha SEQTRAK (USB MIDI) from the agent. Use when the user asks to play notes, change sounds, load projects, edit mixer/effects/arp parameters, read device state, dump or snapshot the unit, drive the sequencer transport, or COMPOSE: record step patterns into the unit's own pattern memory. Triggers on "seqtrak", "트랙 재생", "키트 바꿔", "프로젝트 로드", "볼륨/뮤트/솔로", "리버브/딜레이", "아르페지오", "작곡", "비트 만들어", MIDI/SysEx control of the unit.
---

# SEQTRAK Control

Rust CLI st and MCP server st-mcp live in this repo. Build: cargo build --release. Prefer the st CLI via Bash. If an MCP client is connected, st-mcp exposes the same tools (14).

## Device facts
- Yamaha SEQTRAK over class-compliant USB MIDI. Ports named "SEQTRAK".
- 11 fixed tracks, channels 1-11: kick snare clap hat1 hat2 perc1 perc2 (drum, ch1-7), synth1 synth2 (ch8-9), dx (FM, ch10), sampler (ch11). Channels cannot be remapped.
- SysEx model: F0 43 1n 7F 1C 0C <addr hi mid lo> <data..> F7 (n = 0). Checksum auto-computed by st param.
- Reading state: st get (one parameter), st snapshot (load project + capture echo stream, best full readout), st dump (bulk, big and slow).
- One-shot knobs: CC values center at 64 = zero offset (filter, EG, drum pitch, EQ gain).
- The unit transmits MIDI clock while playing: 96 PPQ, 6 clocks per 16th note (MIDI Clock Out defaults On).

## Composing into the unit (the real write path)

The Data List documents no SysEx to write step notes. The unit's recorder is
the write path: arm REC on the unit panel, then the unit records incoming MIDI
notes into the selected project's patterns. `st rec-steps` automates the note
side: it restarts the transport, fires the notes exactly on the unit's clock
grid, and stops at the end of the window.

```
1. st load 1                       # pick the target project
2. user arms REC on the unit       # panel key; no SysEx for this exists
3. st rec-steps kick  --steps 1,5,9,13
4. st rec-steps snare --steps 5,13 --vel 90
5. st rec-steps synth1 --steps 1,5,9,13 --note 60 --gate 2
6. st transport stop               # if rec-steps was run with --keep-playing
7. done: projects auto-save on power-off or project switch (user guide 4.2); no manual save exists
```

- steps are 1-based 16ths; --bars widens the window (step 17 needs --bars 2).
- --note matters for synth/sampler tracks (pitch); drum kits trigger on any note, 60 = the pad.
- --gate 2 = note length 2 steps (default 1). --keep-playing leaves the transport running for chained takes.
- While REC is not armed the same command just performs the notes live (harmless audition).
- The record session ENDS when the transport stops: chain takes with --keep-playing on every step except the last (each new run restarts the loop from step 1 and overdubs; existing pattern notes replay internally and are not re-recorded).
- Verified on hardware: kick 1,5,9,13 / snare 5,13 vel90 / synth1 note60 gate2 all landed in the unit's pattern and replayed on playback.
- The unit records per MIDI channel: notes land on the track owning that channel (kick = ch1), so no track selection needed on the panel.
- Verify a take by watching st monitor while it plays back: the unit replays recorded notes as note_on on the same channel.

## Sound catalog: 2421 preset sounds

`docs/sound-library.tsv` (shipped in this repo, extracted from the Data List Sound List) indexes every built-in sound: `type` (drum/synth/dx/sampler), `no` (global list number), `name`, `category`, and for synth/dx the `lsb`/`pc` for loading. Counts: drum 855, synth 1077, dx 100, sampler 389.

- Loading a synth or DX sound onto a track: `st pc -m 63 -l <lsb> -p <pc> -c <track>`. Example: Acid3 (no 858) = `st pc -m 63 -l 8 -p 25 -c synth1`. MSB is always 63 for sounds.
- Preset banks: LSB 2-31 = Preset 1-30 (LSB 0-1 = User slots). Formula from the list number: `lsb = (no-1)//128 + 2`, `pc = (no-1)%128`. Verified against the panel display.
- Drum kits use a different target (MSB 32+p, LSB 2p per the bank table below); the drum entries in the TSV carry the list numbers and names but the kit-slot mapping is not verified yet.
- SAMPLER sounds are factory samples; the SAMPLER track's 7 element slots are normally loaded with user WAVs through the official app, not bank/PC.
- Natural-language flow: user asks for a character ("warm pad", "acid bass", "punchy kick") → grep the TSV by category/name keywords → pick candidates → load with `st pc` → confirm. Sound names cannot be read back over SysEx; the panel display or the user's ear is the only verification.
- Loading a sound is a live-buffer edit on that track; the usual auto-save rules apply.

## Track patterns: 6 slots per track

Every one of the 11 tracks holds 6 independent pattern slots. The panel exposes 3 of them as pattern keys; slots 4-6 are reachable only over SysEx (or the official app). Key facts for composing:

- Pattern Select per track: `st param 30:5p:0F <0-5>` selects the slot the track plays AND the slot its recorder writes into. Example: `st param 30:50:0F 01` puts the kick on pattern 2.
- Per-slot lengths: `st param 30:5p:<16,18,1A,1C,1E,20> <MSB> <LSB>` sets the length of pattern 1-6 independently (1-128 steps). One track can run a 16-step pattern while another runs 48.
- There is no song mode: pattern flow is whatever the player switches to live. An arrangement = record each slot, then switch slots on the panel or over SysEx while playing.
- The recorder always writes the CURRENTLY selected pattern, so select the slot before firing takes. `st rec-steps --pattern N` (and `st seq --pattern N`) does the select automatically, and the selection stays afterwards.
- Selecting a slot is a live-buffer edit: switching slots mid-session does not lose other slots' content, and the usual auto-save rules apply.

## Command map
```
st list                                  # enumerate MIDI ports
st identity                              # firmware check
st note -c kick -n 36 -v 110 -d 150      # one hit; -c accepts track names above
st cc   -c synth1 -n volume -v 100       # named CC or number
st pc   -m 63 -l 0 -p 5 -c synth1        # bank+program: load sound onto track
st load 3                                # load project 3
st get  30:50:00                         # read one SysEx parameter
st param 30:50:00 40                     # write (checksum auto)
st rec-steps -c kick --steps 1,5,9,13    # record steps into the unit (see above)
st seq -c synth1 --events "1:42,3:47" --bars 3 --gate 2
                                         # many notes in ONE sweep; --follow anchors on the
                                         # unit's own replayed note (--anchor-ch/-note/-step,
                                         # --loop-steps) and never touches the transport
st snapshot --project 1 --seconds 5      # full state echo -> TSV
st dump 11:00:00 -o dump.hex             # bulk project dump (21KB+)
st monitor --timeout 5                   # watch incoming MIDI
st transport start|stop                  # sequencer (unit sync = External/Auto)
```

## CC quick reference (channels = tracks)
- volume(7) pan(10) mute(23, 0-63 off / 64-127 on) solo(24, 0 off / 1-11 track)
- filter_cutoff(74) filter_resonance(71) eg_attack(73) eg_decay_release(75): 0..64..127 = -64..0..+63 offset
- reverb_send(91) delay_send(94) eq_high(20) eq_low(21) fx_master_1..3(102-104) fx_single_1..3(107-109)
- synth/dx only: portamento_time(5) portamento_switch(65) mono_poly(26: 0 mono 1 poly 2 chord) arp_template(27, 0-15, 0=off) arp_gate(28) arp_speed(29)
- dx only: fm_algorithm(116, 0-11) fm_mod_amount(117) fm_mod_frequency(118) fm_mod_feedback(119)
- drum only: drum_pitch(25)

## Bank select + program change
[1] Target:Project User 1-8, MSB:64, LSB:0-7, PC:any (load triggers on PC)
[2] Target:Sound Preset 1-32, MSB:63, LSB:0-31, PC:0-127
[3] Target:Sound User 1-16, MSB:63, LSB:32-47, PC:0-127
[4] Target:Sampler Element, MSB:62, LSB:0-11, PC:0-127
[5] Target:DrumKit (drum track p0-6), MSB:32+p, LSB:2p, PC:0-127

## SysEx address map (verified bases, p = part 0-10)
- System general 00 00 00 (offset 10 = USB MIDI Transmit SysEx switch; 2D = MIDI Clock Out; 2F = Pattern Select Sensitivity)
- Project Common general 30 40 00 (reads return 0 over SysEx; tempo not reachable)
- Send Reverb 30 41 00, Send Delay 30 42 00
- Master Effect 1-4: 30 43 / 30 44 / 30 45 / 30 46 (offset 00 = type MSB, 01 = type LSB), Master EQ 30 47 00
- A/D: insertion A 30 49 00, insertion B 30 4A 00, general 30 4B 00; USB audio input 30 4C 00; Scale 30 4D 00
- Project Track (p = part): general 30 5p 00 (00 volume, 01 pan, 0F Pattern Select 0-5 = pattern 1-6, 16,18,1A,1C,1E,20 = pattern 1-6 length MSB+LSB, 1-128 steps), chord notes 30 6p 00 (synth parts only)
- Sound (p = part): name 31 0p 00, general 31 1p 00, insertion A 31 2p 00, insertion B 31 3p 00, sound common 31 4p 00
- Arpeggiator 31 5p 00 (synth parts 7-9 only; 06 ARP Loop, 07 ARP Hold, 0D Key Mode, 10 Quantize, 16 Swing Offset MSB+LSB)
- AWM elements: 41 ep 00 oscillator/amplitude/pitch, 42 ep 00 filter/EQ/LFO (p part, e element 0-7)
- SAMPLER: sample common 48 09 00, sample general 50 eA 00 (e = element 0-6, 7 slots)
- Full tables: the official Yamaha SEQTRAK Data List for OS V2.00 (YJ-D0, 01/2026), MIDI Data Table pp. 117-163 (download from Yamaha; not included in this repository)
- Pattern Select example: st param 30:50:0F 02 = kick plays pattern 3

## Web GUI step layer (optional, needs st serve)
`st serve` runs the web controller on 127.0.0.1:8787 and hosts a GUI-side step
sequencer synced to the unit's clock: GET /api/seq (state), POST /api/seq
{op: step|rec|clear|note, track, step, on, note}. The GUI layer sounds over
MIDI; the unit's own pattern memory is untouched by it. Use rec-steps instead
when the goal is writing into the unit.

## Gotchas
- Device SysEx transmit ships OFF. If st get/dump return nothing: st param 00:00:10 01 once.
- Bulk dump can be ignored while the unit is busy; st dump retries 3 times internally.
- The unit only accepts program/bank for loading; there is no "current project" query, use st snapshot.
- Projects auto-save on power-off or project switch (user guide 4.2); there is no manual WRITE key on the panel and no store-to-flash SysEx. Auto-save on st load N commits whatever is in the live buffer, including half-finished takes, so finish or repair a pattern before switching projects.
- MIDI sync: st transport and rec-steps need the unit's MIDI Sync set to External/Auto, otherwise Start/Stop and the clock are ignored (rec-steps fails with "no MIDI clock" in that case).
- The unit auto-powers-off; if ports vanish from st list, power it back on.
- Long-lived processes can stop seeing replugged CoreMIDI devices; st serve nudges MIDIRestart and reconnects on its own.
- Firmware 2.0, chained takes (--keep-playing) on patterns longer than 16 steps: the loop restart between takes does not reliably return to step 1. Takes land at shifted positions or nowhere, non-deterministically. Prefer `st seq` (one sweep, no restarts) for anything beyond a single pitch; in follow mode it locks onto a unique replayed note so verify-then-refire stays deterministic. Verify by counting playback note_ons per pitch against the design (st monitor --timeout, count events; fresh pitches give clean signals). Wrong-slot ghosts accumulate and can only be cleaned on the unit panel.
- A transport stop ENDS the record session (REC disarms). A rec-steps run after a stop only performs the notes live and records nothing; re-arm REC before every batch. Keep one session alive per batch (keep-playing on every take but the last).
- st dump 11:00:00 reads committed project memory and does not reflect real-time-recorder writes; byte-diffing dumps cannot verify rec-steps takes. Verify by playback counting instead.
- Pattern length writes (30:5p:16) belong before arming REC. Length edits while a record session is running correlate with scrambled landings.

## Setup as MCP server (optional)
```
cargo install --path . --locked
claude mcp add seqtrak --scope user -- "$(command -v st-mcp)"
mkdir -p ~/.claude/skills && cp -r .claude/skills/seqtrak ~/.claude/skills/
```
st/st-mcp already installed via cargo install in this environment; the MCP server is registered user-scope as seqtrak and the skill is copied to ~/.claude/skills/seqtrak. Re-run cargo install --path . --locked after CLI changes.
