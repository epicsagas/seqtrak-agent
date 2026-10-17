<script lang="ts">
  import { api } from '../lib/api'

  let value = $state(100)
  let dragging = $state(false)

  // Broadcast to all 11 track channels; the closest thing to master volume.
  function apply() {
    for (const ch of ['kick', 'snare', 'clap', 'hat1', 'hat2', 'perc1', 'perc2', 'synth1', 'synth2', 'dx', 'sampler']) {
      api.cc(ch, 'volume', value).catch(() => {})
    }
  }

  function onDown(e: PointerEvent) {
    dragging = true
    ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
  }

  function onMove(e: PointerEvent) {
    if (!dragging) return
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const dx = e.clientX - (rect.left + rect.width / 2)
    const dy = e.clientY - (rect.top + rect.height / 2)
    let deg = (Math.atan2(dx, -dy) * 180) / Math.PI
    deg = Math.max(-135, Math.min(135, deg))
    value = Math.round(((deg + 135) / 270) * 127)
  }

  function onUp() {
    if (dragging) {
      dragging = false
      apply()
    }
  }

  const ticks = Array.from({ length: 40 }, (_, i) => i)
  const angle = $derived((value / 127) * 270 - 135)
</script>

<div class="knob" role="slider" tabindex="0" aria-label="all track volumes" aria-valuemin="0" aria-valuemax="127"
  aria-valuenow={value}
  onpointerdown={onDown}
  onpointermove={onMove}
  onpointerup={onUp}
  onkeydown={(e) => {
    if (e.key === 'ArrowUp' || e.key === 'ArrowRight') { value = Math.min(127, value + 1); apply() }
    if (e.key === 'ArrowDown' || e.key === 'ArrowLeft') { value = Math.max(0, value - 1); apply() }
  }}
>
  <svg viewBox="0 0 120 120">
    {#each ticks as i (i)}
      <line
        x1="60"
        y1="8"
        x2="60"
        y2="14"
        class="tick"
        class:on={i * 10.35 <= value * 3.15}
        transform={`rotate(${i * 10.35 - 157.5 + (157.5 - 135) * 0 - 0} 60 60) rotate(${(i / 39) * 270 - 135} 60 60)`}
      />
    {/each}
    <circle cx="60" cy="60" r="34" class="cap" />
    <line x1="60" y1="60" x2="60" y2="32" class="pointer" transform={`rotate(${angle} 60 60)`} />
  </svg>
  <output>{String(value).padStart(3, '0')}</output>
</div>

<style>
  .knob {
    display: grid;
    justify-items: center;
    gap: 2px;
    touch-action: none;
    user-select: none;
    cursor: grab;
  }
  .knob:active {
    cursor: grabbing;
  }
  svg {
    width: 148px;
  }
  .tick {
    stroke: var(--panel-well);
    stroke-width: 3;
    stroke-linecap: round;
  }
  .tick.on {
    stroke: var(--orange);
  }
  .cap {
    fill: var(--panel-deep);
    stroke: var(--ink-soft);
    stroke-width: 1;
  }
  .pointer {
    stroke: var(--ink);
    stroke-width: 4;
    stroke-linecap: round;
  }
  output {
    font-variant-numeric: tabular-nums;
    color: var(--ink-soft);
    font-size: 14px;
  }
</style>
