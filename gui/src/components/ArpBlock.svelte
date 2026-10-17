<script lang="ts">
  import { api } from '../lib/api'

  let target = $state('synth1')
  let template = $state(0)
  let gate = $state(64)
  let speed = $state(3)
  let mode = $state(1)
  let pswitch = $state(0)
  let ptime = $state(40)

  const TEMPLATES = ['off', 'up', 'up 2oct', 'down', 'down 2oct', 'rand', 'rand 2oct', 'updown a', 'updown a 2oct', 'updown b', 'updown b 2oct', 'thumb', 'unison', 'chord1', 'chord2', 'as played']

  function send(cc: string, val: number) {
    api.cc(target, cc, val).catch(() => {})
  }
</script>

<div class="arp">
  <div class="row target">
    <span class="lbl">TARGET</span>
    <div class="keys">
      {#each ['synth1', 'synth2', 'dx'] as t (t)}
        <button class="k" class:on={target === t} onclick={() => (target = t)}>{t}</button>
      {/each}
    </div>
  </div>

  <label class="row">
    <span class="lbl">TEMPLATE</span>
    <select bind:value={template} onchange={() => send('arp_template', template)}>
      {#each TEMPLATES as name, i (name)}
        <option value={i}>{i}: {name}</option>
      {/each}
    </select>
  </label>

  <label class="row">
    <span class="lbl">GATE</span>
    <input type="range" min="0" max="127" bind:value={gate} oninput={() => send('arp_gate', gate)} />
    <output>{gate}</output>
  </label>

  <label class="row">
    <span class="lbl">SPEED</span>
    <input type="range" min="0" max="9" bind:value={speed} oninput={() => send('arp_speed', speed)} />
    <output>{speed}</output>
  </label>

  <div class="row">
    <span class="lbl">MODE</span>
    <div class="keys">
      {#each ['mono', 'poly', 'chord'] as label, i (label)}
        <button class="k" class:on={mode === i} onclick={() => { mode = i; send('mono_poly', i) }}>{label}</button>
      {/each}
      <button class="k" class:on={pswitch === 1} onclick={() => { pswitch = pswitch ? 0 : 1; send('portamento_switch', pswitch) }}>port</button>
      <input class="ptime" type="range" min="0" max="127" bind:value={ptime} oninput={() => send('portamento_time', ptime)} aria-label="portamento time" />
    </div>
  </div>
</div>

<style>
  .arp {
    margin-top: 12px;
    background: var(--panel-deep);
    border: 1px solid var(--panel-well);
    border-radius: var(--radius);
    padding: 10px;
    display: grid;
    gap: 8px;
  }
  .row {
    display: grid;
    grid-template-columns: 74px 1fr auto;
    gap: 8px;
    align-items: center;
  }
  .row.target {
    grid-template-columns: 74px 1fr;
  }
  .lbl {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    color: var(--ink-soft);
  }
  .keys {
    display: flex;
    gap: 4px;
  }
  .k {
    border: 1px solid var(--panel-well);
    background: var(--panel);
    color: var(--ink);
    border-radius: var(--radius);
    padding: 4px 8px;
    font-size: 12px;
    font-weight: 600;
  }
  .k.on {
    background: var(--led-cyan);
    border-color: var(--led-cyan);
  }
  output {
    font-variant-numeric: tabular-nums;
    font-size: 13px;
    min-width: 3ch;
    text-align: right;
    color: var(--ink-soft);
  }
  input[type='range'] {
    accent-color: var(--ink);
    width: 100%;
  }
  .ptime {
    width: 64px;
  }
  select {
    background: var(--panel);
    border: 1px solid var(--panel-well);
    border-radius: var(--radius);
    padding: 4px;
    color: var(--ink);
  }
</style>
