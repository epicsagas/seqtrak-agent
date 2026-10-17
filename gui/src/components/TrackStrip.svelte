<script lang="ts">
  import { api } from '../lib/api'

  type Strip = { name: string; label: string; ch: number; family: 'yellow' | 'cyan' }

  let { track }: { track: Strip } = $props()

  let volume = $state(100)
  let pan = $state(64)
  let mute = $state(false)
  let solo = $state(false)
  let lit = $state(false)

  const fam = $derived(track.family === 'yellow' ? 'var(--led-yellow)' : 'var(--led-cyan)')

  function setVolume(e: Event) {
    volume = Number((e.target as HTMLInputElement).value)
    api.cc(track.name, 'volume', volume).catch(() => {})
  }

  function setPan(e: Event) {
    pan = Number((e.target as HTMLInputElement).value)
    api.cc(track.name, 'pan', pan).catch(() => {})
  }

  async function toggleMute() {
    mute = !mute
    await api.cc(track.name, 'mute', mute ? 100 : 0).catch(() => {})
  }

  async function toggleSolo() {
    solo = !solo
    await api.cc(track.name, 'solo', solo ? track.ch : 0).catch(() => {})
  }

  async function tap() {
    lit = true
    setTimeout(() => (lit = false), 150)
    await api.note(track.name, 60, 110, 120).catch(() => {})
  }
</script>

<section class="strip" class:lit style={`--fam:${fam}`}>
  <header>
    <span class="name">{track.label}</span>
    <span class="ch">CH{track.ch}</span>
  </header>
  <label class="ctl">
    <span class="sr">volume</span>
    <input type="range" min="0" max="127" bind:value={volume} oninput={setVolume} />
    <output>{String(volume).padStart(3, '0')}</output>
  </label>
  <label class="ctl">
    <span class="sr">pan</span>
    <input type="range" min="1" max="127" bind:value={pan} oninput={setPan} />
  </label>
  <div class="keys">
    <button class="key" class:on={mute} onclick={toggleMute}>M</button>
    <button class="key" class:on={solo} onclick={toggleSolo}>S</button>
    <button class="key pad" onclick={tap}>HIT</button>
  </div>
</section>

<style>
  .strip {
    background: var(--panel);
    color: var(--ink);
    border-radius: var(--radius);
    padding: 10px 10px 8px;
    display: grid;
    gap: 6px;
    border-top: 3px solid var(--fam);
  }
  .strip.lit {
    box-shadow: 0 0 12px var(--fam);
  }
  header {
    display: flex;
    justify-content: space-between;
    gap: 6px;
    min-width: 0;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .name {
    font-weight: 600;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }
  header {
    gap: 6px;
  }
  .ch {
    color: var(--ink-soft);
    font-variant-numeric: tabular-nums;
  }
  .ctl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
    align-items: center;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
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
  .keys {
    display: flex;
    gap: 4px;
  }
  .key {
    flex: 1;
    border: 1px solid var(--panel-well);
    background: var(--panel-deep);
    color: var(--ink);
    border-radius: var(--radius);
    padding: 5px 0;
    font-weight: 600;
    font-size: 12px;
  }
  .key.on {
    background: var(--ink);
    color: var(--fam);
    border-color: var(--ink);
  }
  .key.pad {
    background: var(--ink);
    color: var(--panel);
  }
</style>
