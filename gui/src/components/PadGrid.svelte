<script lang="ts">
  import { api } from '../lib/api'

  // The seven drum tracks; a pad sends the kit's pad-1 note on that channel.
  const pads = [
    { name: 'kick', label: 'KICK', vel: 110 },
    { name: 'snare', label: 'SNARE', vel: 100 },
    { name: 'clap', label: 'CLAP', vel: 100 },
    { name: 'hat1', label: 'HAT 1', vel: 90 },
    { name: 'hat2', label: 'HAT 2', vel: 90 },
    { name: 'perc1', label: 'PERC 1', vel: 95 },
    { name: 'perc2', label: 'PERC 2', vel: 95 },
  ]

  let lit = $state<string | null>(null)
  let timer: ReturnType<typeof setTimeout> | undefined

  async function hit(name: string) {
    lit = name
    clearTimeout(timer)
    timer = setTimeout(() => (lit = null), 130)
    await api.note(name, 60, 100, 150).catch(() => {})
  }
</script>

<div class="pads" role="group" aria-label="drum pads">
  {#each pads as p (p.name)}
    <button class="pad" class:lit={lit === p.name} onclick={() => hit(p.name)}>{p.label}</button>
  {/each}
</div>

<style>
  .pads {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    margin-top: 10px;
  }
  .pad {
    background: var(--panel-deep);
    border: 1px solid var(--panel-well);
    border-radius: var(--radius);
    padding: 14px 0;
    font-weight: 600;
    font-size: 12px;
    letter-spacing: 0.05em;
    color: var(--ink);
  }
  .pad:active,
  .pad.lit {
    background: var(--led-yellow);
    border-color: var(--led-yellow);
  }
</style>
