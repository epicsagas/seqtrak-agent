<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../../lib/api'
  import { TRACKS } from '../../lib/tracks'
  import { ALL_TRACKS, DRUM_TRACKS, SYNTH_TRACKS, TRACK_LABEL, device, say } from '../../lib/device.svelte'
  import DeviceKnob from './DeviceKnob.svelte'
  import DeviceKey from './DeviceKey.svelte'
  import TouchSlider from './TouchSlider.svelte'

  const SD_PARAMS = [
    { cc: 'filter_cutoff', label: 'CUTOFF' },
    { cc: 'filter_resonance', label: 'RESO' },
    { cc: 'eg_attack', label: 'ATTACK' },
    { cc: 'eg_decay_release', label: 'DECAY' },
  ]
  // Synth keys: [REC] + the 7 scale degrees (User Guide panel item 10/11).
  const SCALE = [60, 62, 64, 65, 67, 69, 71]
  const SCALE_LABELS = ['C4', 'D4', 'E4', 'F4', 'G4', 'A4', 'B4']
  const STEP_LABELS = (page: number) =>
    Array.from({ length: 16 }, (_, i) => String(page * 16 + i + 1))

  // Pattern selection per track, 1-6 (panel: track knobs change patterns; the
  // [ALL] knob changes patterns for all 11 tracks). Sent as SysEx Pattern Select.
  const ptns = $state<Record<string, number>>(Object.fromEntries(ALL_TRACKS.map((t) => [t, 1])))
  let allPtn = $state(1)
  const sdVals = $state<Record<string, number>>(Object.fromEntries(SD_PARAMS.map((p) => [p.cc, 64])))
  const muted = $state<Record<string, boolean>>({})
  const soloed = $state<Record<string, boolean>>({})

  // GUI step layer, synced to the device's MIDI clock by the serve backend.
  let patterns = $state<number[][]>(Array.from({ length: 11 }, () => Array(32).fill(0)))
  let playStep = $state(0)
  let recArm = $state(false)
  const pending = new Set<string>()

  let fxVal = $state(100)
  let highPass = $state(0)
  let repeater = $state(0)
  let fxKnob = $state(0)
  let scope = $state<'master' | 'single'>('master')
  let fxPage = $state<1 | 2>(1)
  let stepPage = $state<0 | 1>(0)
  let selected = $derived(device.selected)
  let lastSent = $state('')
  const isSynth = $derived(SYNTH_TRACKS.includes(selected))
  const trackIdx = $derived(ALL_TRACKS.indexOf(selected))
  const drumX = (i: number) => 200 + i * 60
  const synthX = (i: number) => 690 + i * 60

  function sendCc(t: string, cc: string, v: number) {
    lastSent = `${TRACK_LABEL[t]} ${cc} ${v}`
    api.cc(t, cc, v).catch(() => {})
  }

  const partAddr = (t: string, off: string) => {
    const ch = TRACKS.find((x) => x.name === t)?.ch ?? 1
    return `30:${(0x50 + ch - 1).toString(16)}:${off}`
  }

  function patternSet(t: string, p: number) {
    p = Math.max(1, Math.min(6, p))
    if (ptns[t] === p) return
    ptns[t] = p
    api.param(partAddr(t, '0F'), [p - 1]).catch(() => {})
    say('PATTERN', `${TRACK_LABEL[t]}  ${p}`)
  }

  function allPattern(p: number) {
    p = Math.max(1, Math.min(6, p))
    if (allPtn === p) return
    allPtn = p
    for (const t of ALL_TRACKS) {
      ptns[t] = p
      api.param(partAddr(t, '0F'), [p - 1]).catch(() => {})
    }
    say('PATTERN ALL', String(p))
  }

  function knobPress(t: string) {
    if (device.arm === 'mute') {
      device.arm = null
      muted[t] = !muted[t]
      say(muted[t] ? 'MUTE ON' : 'MUTE OFF', TRACK_LABEL[t])
      sendCc(t, 'mute', muted[t] ? 100 : 0)
      return
    }
    if (device.arm === 'solo') {
      device.arm = null
      soloed[t] = !soloed[t]
      say(soloed[t] ? 'SOLO ON' : 'SOLO OFF', TRACK_LABEL[t])
      sendCc(t, 'solo', soloed[t] ? ALL_TRACKS.indexOf(t) + 1 : 0)
      return
    }
    device.selected = t
    say('SELECT', TRACK_LABEL[t])
    // sync the knob with the pattern the unit is actually on
    api.paramRead(partAddr(t, '0F')).then((r) => { ptns[t] = (r.bytes[0] ?? 0) + 1 }).catch(() => {})
    // hardware auditions on press only while playback is stopped
    if (!device.playing) api.note(t, 60, 100, 200).catch(() => {})
  }

  function knobHold(t: string) {
    sendCc(t, 'mute', 100)
    say('MUTE ON', TRACK_LABEL[t])
  }

  function panic() {
    say('PANIC', 'all notes off')
    for (const t of ALL_TRACKS) api.cc(t, '123', 0).catch(() => {})
  }

  async function playToggle() {
    device.playing = !device.playing
    say(device.playing ? 'PLAY' : 'STOP')
    await api.transport(device.playing ? 'start' : 'stop').catch(() => {})
  }

  function armMode(mode: 'mute' | 'solo') {
    device.arm = device.arm === mode ? null : mode
    say(device.arm ? `${mode.toUpperCase()} MODE` : 'EXIT', device.arm ? 'touch a track knob' : '')
  }

  function unavailable(what: string) {
    say(what, 'not available over MIDI')
  }

  // ---- step layer -----------------------------------------------------------

  async function pollSeq() {
    try {
      const s = await api.seq()
      device.playing = s.playing
      playStep = s.step
      recArm = s.rec
      if (pending.size === 0) {
        patterns = s.patterns
      } else {
        const next = patterns.map((r) => r.slice())
        s.patterns.forEach((row, i) =>
          row.forEach((v, j) => {
            if (!pending.has(`${i}:${j}`)) next[i][j] = v
          })
        )
        patterns = next
      }
    } catch {}
  }

  onMount(() => {
    const t = setInterval(pollSeq, 120)
    return () => clearInterval(t)
  })

  // Drum keys toggle the selected track's steps on/off (panel item 9).
  function gridStep(gs: number) {
    const idx = trackIdx
    if (idx < 0) return
    const on = !patterns[idx][gs]
    const next = patterns.map((r) => r.slice())
    next[idx][gs] = on ? (idx < 7 ? 1 : 60) : 0
    patterns = next
    const key = `${idx}:${gs}`
    pending.add(key)
    api.seqOp({ op: 'step', track: selected, step: gs, on })
      .catch(() => {})
      .finally(() => pending.delete(key))
    say(on ? 'STEP ON' : 'STEP OFF', `${gs + 1}  ${TRACK_LABEL[selected]}`)
  }

  // [REC] arms the step layer; scale keys audition and, while REC + playing,
  // record into the layer at the current step (the device records the same
  // notes itself when its own REC is up, since these are real Note Ons).
  function toggleRec() {
    recArm = !recArm
    api.seqOp({ op: 'rec', on: recArm }).catch(() => {})
    say(recArm ? 'REC ARM' : 'REC OFF', recArm ? 'keys record while playing' : '')
  }

  function synthNote(note: number) {
    const t = isSynth ? selected : 'synth1'
    api.seqOp({ op: 'note', track: t, note }).catch(() => {})
    say(recArm && device.playing ? 'REC' : 'NOTE', `${TRACK_LABEL[t]}  ${SCALE_LABELS[SCALE.indexOf(note)]}`)
  }

  // ---- sound design & fx ----------------------------------------------------

  function sdDelta(cc: string, d: number) {
    sdVals[cc] = Math.max(0, Math.min(127, sdVals[cc] + d))
    sendCc(selected, cc, sdVals[cc])
    const label = SD_PARAMS.find((p) => p.cc === cc)?.label ?? cc
    say(label, `${TRACK_LABEL[selected]}  ${sdVals[cc]}`)
  }

  // [FX] knob: effect type = category (MSB 0-7) x preset (LSB 0-7) via SysEx.
  function fxKnobDelta(d: number) {
    fxKnob = Math.max(0, Math.min(63, fxKnob + d))
    const cat = Math.floor(fxKnob / 8)
    const preset = fxKnob % 8
    const addr = scope === 'master' ? '30:43:00' : `31:${0x20 + ALL_TRACKS.indexOf(selected)}:00`
    api.param(addr, [cat, preset]).catch(() => {})
    say(`${scope.toUpperCase()} FX TYPE`, `cat ${cat + 1} preset ${preset + 1}`)
  }

  function fxTargets(momentary: boolean): string[] {
    if (momentary && fxPage === 1) return ALL_TRACKS
    if (scope === 'single') return [selected]
    return ['kick']
  }

  function levelInput(v: number) {
    say('FX LEVEL', String(v))
    const cc = scope === 'master' ? '102' : '107'
    for (const t of fxTargets(false)) sendCc(t, cc, v)
  }

  function highPassInput(v: number) {
    say('HIGH PASS', String(v))
    const sweep = 64 + Math.round((v / 127) * 63)
    for (const t of fxTargets(true)) sendCc(t, 'filter_cutoff', sweep)
  }

  function highPassRelease() {
    highPass = 0
    say('HIGH PASS', 'off')
    for (const t of fxTargets(true)) sendCc(t, 'filter_cutoff', 64)
  }

  function repeaterInput(v: number) {
    say('REPEATER', String(v))
    for (const t of SYNTH_TRACKS) sendCc(t, 'arp_gate', v)
  }

  function repeaterRelease() {
    repeater = 0
    say('REPEATER', 'off')
    for (const t of SYNTH_TRACKS) sendCc(t, 'arp_gate', 64)
  }

  function page2Input(kind: 'hp' | 'rpt', v: number) {
    const cc = scope === 'master' ? (kind === 'hp' ? '103' : '104') : kind === 'hp' ? '108' : '109'
    say(kind === 'hp' ? 'FX PARAM 2' : 'FX PARAM 3', String(v))
    for (const t of fxTargets(false)) sendCc(t, cc, v)
  }

  function clearFx() {
    fxVal = 0
    highPass = 0
    repeater = 0
    const cc = scope === 'master' ? '102' : '107'
    for (const t of fxTargets(false)) sendCc(t, cc, 0)
    for (const t of fxTargets(true)) sendCc(t, 'filter_cutoff', 64)
    for (const t of SYNTH_TRACKS) sendCc(t, 'arp_gate', 64)
    say('CLEAR FX', 'level minimized')
  }

  function toggleScope() {
    scope = scope === 'master' ? 'single' : 'master'
    say('SCOPE', scope.toUpperCase())
  }

  function toggleFxPage() {
    fxPage = fxPage === 1 ? 2 : 1
    say('FX PAGE', String(fxPage))
  }
</script>

<div class="wrap">
<svg viewBox="0 0 1560 430" role="img" aria-label="SEQTRAK device faceplate">
  <rect x="2" y="2" width="1556" height="426" rx="18" class="chassis" />
  <rect x="96" y="12" width="1446" height="406" rx="10" class="topPanel" />
  <rect x="12" y="12" width="76" height="406" rx="10" class="sidePanel" />

  <!-- left side buttons (1-6) -->
  <text x="50" y="30" class="zoneLabel" text-anchor="middle">SIDE</text>
  <DeviceKey x={16} y={38} w={32} h={22} label="VOL+" onDown={() => { device.masterVol = Math.min(127, device.masterVol + 5); say('MONITOR VOL', String(device.masterVol)) }} />
  <DeviceKey x={52} y={38} w={32} h={22} label="VOL-" onDown={() => { device.masterVol = Math.max(0, device.masterVol - 5); say('MONITOR VOL', String(device.masterVol)) }} />
  <DeviceKey x={16} y={64} w={32} h={22} label="MUTE" family="orange" onDown={() => armMode('mute')} />
  <DeviceKey x={52} y={64} w={32} h={22} label="SOLO" family="orange" onDown={() => armMode('solo')} />
  <DeviceKey x={16} y={90} w={68} h={22} label={`PG${stepPage + 1}`} onDown={() => { stepPage = stepPage === 0 ? 1 : 0; say('PAGE', String(stepPage + 1)) }} />
  <DeviceKey x={16} y={116} w={68} h={30} label="DELETE" onDown={() => unavailable('DELETE')} />
  <DeviceKey x={16} y={150} w={68} h={30} family="orange" label={device.playing ? 'STOP' : 'PLAY'} onDown={playToggle} />
  <text x="50" y="200" class="sideNote" text-anchor="middle">{'⏻'}/PLAY</text>
  <DeviceKnob x={50} y={280} r={30} value={((allPtn - 1) / 5) * 127} label="ALL" family="orange" onDelta={(d) => allPattern(allPtn + d)} onPress={panic} />
  <text x="50" y="345" class="sideNote" text-anchor="middle">pattern all 1-6</text>

  <!-- track knobs (8): turn = pattern 1-6, press = select + audition (stopped) -->
  {#each DRUM_TRACKS as t, i (t)}
    <DeviceKnob x={drumX(i)} y={64} r={22} value={((ptns[t] - 1) / 5) * 127} label={TRACK_LABEL[t]} family="yellow" selected={selected === t} onDelta={(d) => patternSet(t, ptns[t] + d)} onPress={() => knobPress(t)} onHold={() => knobHold(t)} />
  {/each}
  {#each SYNTH_TRACKS as t, i (t)}
    <DeviceKnob x={synthX(i)} y={64} r={22} value={((ptns[t] - 1) / 5) * 127} label={TRACK_LABEL[t]} family="cyan" selected={selected === t} onDelta={(d) => patternSet(t, ptns[t] + d)} onPress={() => knobPress(t)} onHold={() => knobHold(t)} />
  {/each}

  <!-- drum keys (9): 2x8 step on/off grid for the selected track -->
  <text x={110} y={146} class="zoneLabel">DRUM · PAGE {stepPage + 1}</text>
  {#each STEP_LABELS(stepPage) as label, i (label)}
    {@const gs = stepPage * 16 + i}
    <DeviceKey x={110 + (i % 8) * 58} y={160 + Math.floor(i / 8) * 46} w={54} h={40} label={label} family="yellow" lit={(patterns[trackIdx]?.[gs] ?? 0) > 0} play={device.playing && playStep === gs} onDown={() => gridStep(gs)} />
  {/each}

  <!-- synth section (10/11): REC + 2x4 keys, first key arms recording -->
  <text x={690} y={146} class="zoneLabel">SYNTH / SAMPLER</text>
  <DeviceKey x={690} y={160} w={46} h={40} rx={6} label="REC" family="orange" dark lit={recArm} onDown={toggleRec} />
  {#each SCALE as n, i (n)}
    {@const pos = i + 1}
    <DeviceKey x={690 + (pos % 4) * 52} y={160 + Math.floor(pos / 4) * 46} w={46} h={40} label={SCALE_LABELS[i]} family="cyan" onDown={() => synthNote(n)} />
  {/each}
  <DeviceKey x={908} y={160} w={64} h={22} label="BAR LEN" onDown={() => unavailable('BAR LENGTH')} />
  <DeviceKey x={908} y={186} w={64} h={22} label="OCTAVE" onDown={() => unavailable('OCTAVE')} />
  <DeviceKey x={908} y={212} w={64} h={22} label="SCALE" onDown={() => unavailable('SCALE')} />
  <DeviceKey x={908} y={238} w={64} h={22} label="KEY" onDown={() => unavailable('KEY')} />
  <DeviceKey x={908} y={264} w={64} h={22} label="REC SMP" onDown={() => unavailable('REC SAMPLE')} />

  <line x1={990} y1={20} x2={990} y2={410} class="divider" />

  <!-- sound design & fx (17-27) -->
  <text x={1020} y={112} class="zoneLabel">SOUND DESIGN &amp; FX</text>
  <rect x={1020} y={20} width={230} height={58} rx="4" class="screen" />
  <text x={1032} y={44} class="screenText">{device.screenLine1}</text>
  <text x={1032} y={64} class="screenSub">{device.screenLine2 || (device.arm ? `${device.arm.toUpperCase()} MODE` : `SEL ${TRACK_LABEL[selected]} · P${ptns[selected]} · FX${fxPage} · ${scope.slice(0, 3).toUpperCase()}`)}</text>
  <text x={1240} y={40} class="screenSub" text-anchor="end">{lastSent}</text>
  <DeviceKey x={1258} y={26} w={62} h={22} label="UNDO" onDown={() => unavailable('UNDO/REDO')} />
  <DeviceKey x={1258} y={52} w={62} h={22} label="REDO" onDown={() => unavailable('UNDO/REDO')} />

  {#each SD_PARAMS as p, i (p.cc)}
    <DeviceKnob x={1020 + i * 62} y={140} r={22} value={sdVals[p.cc]} label={p.label} family={isSynth ? 'cyan' : 'yellow'} onDelta={(d) => sdDelta(p.cc, d)} />
  {/each}
  <DeviceKey x={1020} y={182} w={72} h={22} label="SD PAGE" onDown={() => unavailable('SD PAGE')} />
  <DeviceKey x={1100} y={182} w={72} h={22} label="CLEAR FX" onDown={clearFx} />
  <DeviceKnob x={1258} y={140} r={24} value={fxKnob} label="FX" family="orange" onDelta={fxKnobDelta} />

  <!-- [MASTER/SINGLE] slide switch (24) -->
  <g class="switch" onpointerdown={toggleScope} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleScope()} tabindex="0" role="switch" aria-checked={scope === 'single'} aria-label="master or single">
    <rect x={1330} y={104} width={116} height={24} rx="12" class="switchTrack" />
    <rect x={scope === 'master' ? 1333 : 1330 + 116 - 57} y={107} width={54} height={18} rx="9" class="switchNub" />
    <text x={1333 + 24} y={119} text-anchor="middle" class="switchText" opacity={scope === 'master' ? 1 : 0.35}>MST</text>
    <text x={1330 + 116 - 26} y={119} text-anchor="middle" class="switchText" opacity={scope === 'single' ? 1 : 0.35}>SINGLE</text>
  </g>
  <text x={1330} y={96} class="zoneLabel">SCOPE</text>

  <!-- touch sliders (25). page 1: FX LEVEL / HIGH PASS / REPEATER; page 2: params 1/2/3 -->
  {#if fxPage === 1}
    <TouchSlider x={1330} y={140} label="FX LVL" bind:value={fxVal} color="cyan" onInput={levelInput} />
    <TouchSlider x={1374} y={140} label="HI PASS" bind:value={highPass} spring color="yellow" onInput={highPassInput} onRelease={highPassRelease} />
    <TouchSlider x={1418} y={140} label="RPT" bind:value={repeater} spring color="orange" onInput={repeaterInput} onRelease={repeaterRelease} />
  {:else}
    <TouchSlider x={1330} y={140} label="P1" bind:value={fxVal} color="cyan" onInput={levelInput} />
    <TouchSlider x={1374} y={140} label="P2" bind:value={highPass} color="yellow" onInput={(v) => page2Input('hp', v)} />
    <TouchSlider x={1418} y={140} label="P3" bind:value={repeater} color="orange" onInput={(v) => page2Input('rpt', v)} />
  {/if}
  <DeviceKey x={1330} y={286} w={118} h={22} label={`FX PAGE ${fxPage}`} onDown={toggleFxPage} />

  <!-- speaker (27 area) -->
  {#each Array.from({ length: 4 }, (_, row) => row) as row (row)}
    {#each Array.from({ length: 3 }, (_, col) => col) as col (col)}
      <circle cx={1480 + col * 22} cy={330 + row * 22} r="7" class="grill" />
    {/each}
  {/each}

  <text x={110} y={406} class="hint">track knobs + ALL = pattern 1-6 (real) . drum keys = step on/off, fires on the unit's clock while playing . REC + scale keys record while playing . VOL+- = monitor only</text>
</svg>
</div>

<style>
  .wrap {
    background: var(--chassis-deep);
    border-radius: 6px;
    padding: 8px;
    overflow-x: auto;
  }
  svg {
    display: block;
    width: 100%;
    min-width: 1100px;
  }
  .chassis { fill: #35322c; stroke: #211f1b; }
  .topPanel { fill: var(--panel); }
  .sidePanel { fill: var(--panel-deep); }
  .divider { stroke: var(--panel-well); stroke-width: 2; }
  .zoneLabel { font-size: 10px; font-weight: 600; letter-spacing: 0.18em; fill: var(--ink-soft); }
  .sideNote { font-size: 8.5px; fill: var(--ink-soft); }
  .screen { fill: var(--chassis-deep); stroke: #000; }
  .screenText { fill: var(--led-yellow); font-size: 17px; font-weight: 600; letter-spacing: 0.1em; font-variant-numeric: tabular-nums; }
  .screenSub { fill: #9aa; font-size: 11px; font-variant-numeric: tabular-nums; }
  .grill { fill: var(--panel-well); }
  .hint { font-size: 11px; fill: var(--ink-soft); }
  .switch { cursor: pointer; }
  .switchTrack { fill: var(--panel-deep); stroke: var(--panel-well); }
  .switchNub { fill: var(--chassis-deep); stroke: #000; }
  .switchText { font-size: 9px; font-weight: 600; fill: var(--ink); pointer-events: none; }
</style>
