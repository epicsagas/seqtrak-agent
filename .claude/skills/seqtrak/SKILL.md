# SEQTRAK Control
Rust CLI st and MCP server st-mcp live in this repo. Build: cargo build --release. Prefer the st CLI via Bash. If an MCP client is connected, st-mcp exposes the same tools (15).
## Reference material (all shipped in this repo)
- docs/seqtrak_user_guide_D0.pdf: official User Guide, OS V2.00, 109 pages (feature manual).
- docs/seqtrak_data_list_D0.pdf: official Data List, YJ-D0 01/2026, 163 pages. Catalog pp.1-110 (sounds/waves/effects), MIDI Data Table pp.117-163 (all SysEx addresses). Ground truth for every address below.
- docs/sound-library.tsv: 2421 preset sounds (drum 855, synth 1077, dx 100, sampler 389) with type/no/name/category/lsb/pc.
- docs/awm-wave.tsv: 2019 of 2024 AWM waveforms (no/name/main/sub) for element wave selection via the 41:ep oscillator block.
- docs/fx-types.tsv: effect Type MSB/LSB for every track/send/master effect (reverb 15, delay 9, filter 3, compressor 6, distortion 17, modulation 27, other 17). Two ANALOG DELAY rows have unresolved hex (see file note).
## Device facts
- Yamaha SEQTRAK over class-compliant USB MIDI. Ports named "SEQTRAK".
- 11 fixed tracks, channels 1-11: kick snare clap hat1 hat2 perc1 perc2 (drum, ch1-7), synth1 synth2 (ch8-9), dx (FM, ch10), sampler (ch11). Channels cannot be remapped.
- SysEx model: F0 43 1n 7F 1C 0C  <data..> F7 (n = 0). Checksum auto-computed by st param.
- Reading state: st get (one parameter), st snapshot (load project + capture echo stream, best full readout), st dump (bulk, big and slow).
- One-shot knobs: CC values center at 64 = zero offset (filter, EG, drum pitch, EQ gain).
- The unit transmits MIDI clock while playing: 96 PPQ, 6 clocks per 16th note (MIDI Clock Out defaults On).
## Composing into the unit (the real write path)
The Data List documents no SysEx to write step data. The unit's recorder is the write path: arm REC on the unit panel, then the unit records incoming MIDI notes into the selected project's patterns. st rec-steps automates the note side: it restarts the transport, fires the notes exactly on the unit's clock grid, and stops at the end of the window.
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
- The record session ENDS when the transport stops: chain takes with --keep-playing on every step except the last (each new run restarts the loop from step 1 and overdubs; existing notes replay internally and are not re-recorded).
- Verified on hardware: kick 1,5,9,13 / snare 5,13 vel90 / synth1 note60 gate2 all landed in the unit's pattern and replayed on playback.
- The unit records per MIDI channel: notes land on the track owning that channel (kick = ch1), so no track selection needed on the panel.
- Verify a take by watching st monitor while it plays back: the unit replays recorded notes as note_on on the same channel.
## Sound catalog: 2421 preset sounds
- Loading a synth or DX sound onto a track: st pc -m 63 -l <lsb> -p <pc> -c <track>. Example: Acid3 (no 858) = st pc -m 63 -l 8 -p 25 -c synth1. MSB is always 63 for sounds.
- Preset banks: LSB 2-31 = Preset 1-30 (LSB 0-1 = User slots). Formula from the list number: lsb = (no-1)//128 + 2, pc = (no-1)%128. Verified against the panel display.
- Drum kits use a different target (MSB 32+p, LSB 2p per the bank table below); the drum entries in the TSV carry the list numbers and names but the kit-slot mapping is not verified yet.
- UNRESOLVED: loading drum kits via st pc -m 32 -l <0|1> -p <0-127> -c kick produced no audible or panel change in tests (OS 2.00); the Data List documents MSB 32-38 / LSB = preset kit bank / PC = kit, but the mapping from the Drum Sound list numbers to kits is unknown. Do not promise drum kit switching until this is solved; per-pad sound assignment inside a kit is a DrumKit-edit (panel/app) feature.
- SAMPLER sounds are factory samples; the SAMPLER track's 7 element slots are normally loaded with user WAVs through the official app, not bank/PC.
- Natural-language flow: user asks for a character ("warm pad", "acid bass", "punchy kick") -> grep the TSV by category/name keywords -> pick candidates -> load with st pc -> confirm. Sound names cannot be read back over SysEx; the panel display or the user's ear is the only verification.
- Loading a sound is a live-buffer edit on that track; the usual auto-save rules apply.
- AWM waves (synth element waveform numbers): docs/awm-wave.tsv, written into 41:ep oscillator block.
## Patterns: 6 slots per track
Every one of the 11 tracks holds 6 independent pattern slots. The panel exposes 3 of them as pattern keys; slots 4-6 are reachable only over SysEx (or the official app). Key facts for composing:
- Pattern Select per track: st param 30:5p:0F <0-5> selects the slot the track plays AND the slot its recorder writes into. Example: st param 30:50:0F 01 puts the kick on pattern 2.
- Per-slot lengths: st param 30:5p:<16,18,1A,1C,1E,20> <MSB> <LSB> sets the length of patterns 1-6 independently (1-128 steps). One track can run a 16-step loop while another runs 48.
- Song mode exists on the device (User Guide ch.13, scenes) but the Data List documents NO SysEx address for song/scene data: it is panel/app only. Over MIDI the arrangement is whatever pattern slots you switch to live. Do not claim song/scene control.
- The recorder always writes the CURRENTLY selected pattern, so select the slot before firing takes. st rec-steps --pattern N (and st seq --pattern N) does the select automatically, and the selection stays afterwards.
- Pattern count mode 3 vs 6 slots: project common 30:40:79 (0 = normal 3, 1 = advanced 6).
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
st bulk 11:00:00 dump.hex                # send a bulk block back (restore; UNVERIFIED)
st monitor --timeout 5                   # watch incoming MIDI
st transport start|stop                  # sequencer (unit sync = External/Auto)
```
- st bulk sends the payload hex file (data bytes only, as written by st dump -o) framed as a bulk dump the unit is documented to receive. byte_count = data+4 per spec. VERIFIED on hardware: system general 00:00:00 dump -> bulk write -> re-dump came back byte-identical.
## Project Common 30:40 (offsets; per Data List pp.123-124)
- 00-57 Project Name (UTF-8, 7-bit encoded, 88 bytes)
- 63 Sampler Element Mute (bit6-0 = elements 7-1)
- 65 Pan (01-7F, 40 center), 66 Volume
- 68 Solo Track (0 off, 1-11)
- 75 ARP Synchro Quantize Value (5-300)
- 76 Tempo MSB + 77 LSB (2-byte big-endian; 00:78 = 120). Read AND write verified on hardware (120 to 126 to 120 round-trip).
- 7C-7D Swing Offset (2-byte signed, -58..+58; default 0100 = 0)
- 79 Pattern Select Mode (0 = 3 patterns, 1 = 6)
- 7A-7B Pattern Master Step (2-byte, 1-128)
- 7E Scale (0-7 = Scale 1-8), 7F Key (40-4B, 40 = C)
Examples: `st param 30:40:76 00 78` = tempo 120; `st param 30:40:7E 02` = Scale 3; `st param 30:40:79 01` = 6-pattern mode.
## System 00:00 (offsets; per Data List p.123)
- 00 Speaker Master Volume (0-127), 01 Headphone Master Volume
- 02 Master Tune (4-byte)
- 06 Local Switch, 07 Bank Select Rx/Tx, 08 Program Change Rx/Tx (0/1)
- 0A-13, 0F/10 MIDI I/O + message-filter switches per port (legacy/USB/Bluetooth; Sysex Tx ships OFF: offset 10 = USB MIDI Transmit SysEx, enable once with st param 00:00:10 01)
- 14 Bluetooth MIDI Tx SysEx, 15/18 Audio Output Gain (-6/0/+6/+12 dB)
- 19 Sampling Source (0 resampling, 1 mic/line, 2 USB audio), 1B Sampling Count-In (0/1), 1C Auto Normalize (0/1)
- 1F Record Quantize Value (0 off, then 60/80/120/160/240)
- 20 Pad Velocity (1-127), 21 Sound Design Mode (0 normal, 1 advanced)
- 22-24 MIDI Thru per port (0 off, 1 channel, 2 realtime, 3 both)
- 25 Click Switch (metronome), 26 Click Type (1-10), 27 Click Volume
- 2A MIDI Sync (0 Internal, 1 MIDI/Auto). Set this BEFORE st transport/rec-steps if the unit ignores them.
- 2B Transmit Sequencer Control, 2D MIDI Clock Out, 2E Sink Current, 2F Pattern Select Sensitivity (1-3)
Examples: `st param 00:00:2A 01` = MIDI sync Auto; `st param 00:00:25 01` = metronome on; `st param 00:00:00 7F` = speaker volume max.
## Project Track 30:5p (p = part 0-10)
- 00 Volume, 01 Pan
- 02/03 Velocity Limit Low/High, 04/05 Note Limit Low/High (C-2..G8)
- 08 Group Number (0 off, 1-127)
- 0C Octave (-2..+2, 40 = 0; NOT for drum/sampler tracks)
- 0F Pattern Select (0-5 = slots 1-6)
- 16,18,1A,1C,1E,20 Pattern 1-6 Step (2-byte MSB/LSB, 1-128)
- 29 Mute (0 off, 7D on)
## Chord notes (synth parts 7-9 only)
- Scale 1-4: 30:6p, Scale 5-8: 30:7p. Per offset: Pad 1-7, Chord Note 1-4 per scale (0 off, 24-60 = C1..C6). Full byte map: Data List pp.135-137.
## Effects
- Send Reverb 30:41 / Send Delay 30:42: 00-01 Type MSB/LSB (see docs/fx-types.tsv), 02 Template Number, 03-32 Parameters 1-24 (2-byte each, odd MSB offsets), 44 Return, 45 Pan, 46 On/Off. Example: `st param 30:41:00 01 20` = send reverb R3 PLATE.
- Master Effect 1-4: 30:43 / 30:44 / 30:45 / 30:46 (00 type MSB, 01 type LSB, parameters follow same 2-byte layout). Master EQ 30:47.
- Master/single effect slider CCs: fx_master_1..3 (102-104 = Master Effect 1 params 1-3), 105 = Master Effect 2 param 1, 106 = Master Effect 3 param 1, fx_single_1..3 (107-109), send reverb 110-112, send delay 113-115.
- A/D: insertion A 30:49, insertion B 30:4A, general 30:4B; USB audio input 30:4C; Scale 30:4D.
## Sound blocks (p = part 0-10)
- Name 31:0p (7-bit encoded), General 31:1p, Insertion A 31:2p, Insertion B 31:3p
- LFO 31:4p (NOT "sound common")
- Arpeggiator 31:5p (synth parts 7-9 only; 06 ARP Loop, 07 ARP Hold, 0D Key Mode, 10 Quantize, 16 Swing Offset MSB+LSB)
- AWM elements: 41 ep 00 oscillator/amplitude/pitch, 42 ep 00 filter/EQ/LFO (p part, e element 0-7; wave numbers from docs/awm-wave.tsv)
- Sound DX: Common 48:09, Operator 49:o9 (o = operator 0-3)
- SAMPLER: element blocks 41:eA / 42:eA (per element) and element general 50:eA (e = element 0-6). There is no separate "sample common" beyond 48:09 which belongs to DX.
## Not controllable over MIDI (panel/app only, no SysEx documented)
- Song mode / scenes (User Guide ch.13), launch quantize (5.1.3)
- Pattern copy/paste/delete, track step copy/paste (5.4/5.5/5.9)
- Step attributes: micro timing, substep, probability, parameter lock, motion recording (6.3-6.6, 9.2-9.3); step data exists only inside bulk dumps (read-only)
- Sampling execution: record/resample/monitor trigger (8.2/8.4/8.7). Settings (source/count-in/auto-normalize) ARE controllable: system offsets 19/1B/1C
- Track type change Drum/DrumKit/Synth (6.7), sound save/delete/import (9.5-9.7), WAV load, undo/redo (9.4), knob sensitivity (16.2), factory reset (16.1)
- Project backup/restore via app/USB (4.3): st dump + st bulk replaces this in principle, but bulk write is UNVERIFIED on hardware.
## CC quick reference (channels = tracks)
- volume(7) pan(10) mute(23, 0-63 off / 64-127 on) solo(24, 0 off / 1-11 track)
- filter_cutoff(74) filter_resonance(71) eg_attack(73) eg_decay_release(75): 0..64..127 = -64..0..+63 offset
- reverb_send(91) delay_send(94) eq_high(20) eq_low(21) fx_master_1..3(102-104) fx_single_1..3(107-109)
- synth/dx only: portamento_time(5) portamento_switch(65) mono_poly(26: 0 mono 1 poly 2 chord) arp_template(27, 0-15, 0=off) arp_gate(28) arp_speed(29)
- dx only: fm_algorithm(116, 0-11) fm_mod_amount(117) fm_mod_frequency(118) fm_mod_feedback(119)
- drum only: drum_pitch(25)
- Receive-only CCs the unit accepts (no need to send unless asked): expression(11), sustain(64), sostenuto(66), data entry(6/38/96/97); RPN 0/1/2 = pitch bend sensitivity / master fine tune / master coarse tune
## Bank select + program change
[1] Target:Project User 1-8, MSB:64, LSB:0-7, PC:any (load triggers on PC; PC 8 = Temporary Project) [2] Target:Sound Preset 1-32, MSB:63, LSB:0-31, PC:0-127 [3] Target:Sound User 1-16, MSB:63, LSB:32-47, PC:0-127 [4] Target:Sampler Element, MSB:62, LSB:0-11, PC:0-127 [5] Target:DrumKit (drum track p0-6), MSB:32+p, LSB:2p, PC:0-127
## Web GUI step layer (optional, needs st serve)
st serve runs the web controller on 127.0.0.1:8787 and hosts a GUI-side step sequencer synced to the unit's clock: GET /api/seq (state), POST /api/seq {op: step|rec|clear|note, track, step, on, note}. The GUI layer sounds over MIDI; the unit's own pattern memory is untouched by it. Use rec-steps/seq instead when the goal is writing into the unit.
## Gotchas
- Device SysEx transmit ships OFF. If st get/dump return nothing: st param 00:00:10 01 once.
- Direct st dump only works for System 00:00:00 (and Format Version 00:7F:00). Project/sound blocks refuse per-block requests (verified: 30:40/30:41/30:50/31:10 all return 0 bytes); request whole data through the Bulk Header addresses instead: 11:00:00 = project edit buffer, 11:01:nn = project user, 02:nn = sound edit buffer part, etc. (Parameter Change Table, Bulk Control.)
- Bulk dump can be ignored while the unit is busy; st dump retries 3 times internally.
- The unit only accepts program/bank for loading; there is no "current project" query, use st snapshot.
- Projects auto-save on power-off or project switch (user guide 4.2); there is no manual WRITE key on the panel and no store-to-flash SysEx. Auto-save on st load N commits whatever is in the live buffer, including half-finished takes, so finish or repair a pattern before switching projects.
- MIDI sync: st transport and rec-steps need the unit's MIDI Sync set to External/Auto. If it is on Internal, fix it remotely: st param 00:00:2A 01.
- The unit auto-powers-off; if ports vanish from st list, power it back on.
- Long-lived processes can stop seeing replugged CoreMIDI devices; st serve nudges MIDIRestart and reconnects on its own.
- Firmware 2.0, chained takes (--keep-playing) on patterns longer than 16 steps: the loop restart between takes does not reliably return to step 1. Takes land at shifted positions or nowhere, non-deterministically. Prefer st seq (one sweep, no restarts) for anything beyond a single pitch; in follow mode it locks onto a unique replayed note so verify-then-refire stays deterministic. Verify by counting playback note_ons per pitch against the design (st monitor --timeout, count events; fresh pitches give clean signals). Wrong-slot ghosts accumulate and can only be cleaned on the unit panel.
- A transport stop ENDS the record session (REC disarms). A rec-steps run after a stop only performs the notes live and records nothing; re-arm REC before every batch. Keep one session alive per batch (keep-playing on every take but the last).
- st dump 11:00:00 reads committed project memory and does not reflect real-time-recorder writes; byte-diffing dumps cannot verify rec-steps takes. Verify by playback counting instead.
- Pattern length writes (30:5p:16) belong before arming REC. Length edits while a record session is running correlate with scrambled landings.
## Setup as MCP server (optional)
```
cargo install --path . --locked
claude mcp add seqtrak --scope user -- "$(command -v st-mcp)"
mkdir -p ~/.claude/skills/seqtrak && cp .claude/skills/seqtrak/SKILL.md ~/.claude/skills/seqtrak/
```
st/st-mcp already installed via cargo install in this environment; the MCP server is registered user-scope as seqtrak and the skill is copied to ~/.claude/skills/seqtrak. Re-run cargo install --path . --locked after CLI changes. Re-copy SKILL.md after doc changes.
