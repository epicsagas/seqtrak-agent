<script lang="ts">
  // Hardware-style rotary: drag vertical, wheel for steps, click = press, hold = alternate.
  let {
    x, y, r = 26, value, label, family = 'yellow', selected = false,
    onPress, onHold, onDelta, onValue,
  }: {
    x: number; y: number; r?: number; value: number; label: string
    family?: 'yellow' | 'cyan' | 'orange'; selected?: boolean
    onPress?: () => void; onHold?: () => void
    onDelta?: (d: number) => void; onValue?: (v: number) => void
  } = $props()

  let dragging = $state(false)
  let pressed = $state(false)
  let startY = 0
  let moved = 0
  let holdTimer: ReturnType<typeof setTimeout> | undefined
  let holdFired = false

  const famColor = $derived(
    family === 'orange' ? 'var(--orange)' : family === 'cyan' ? 'var(--led-cyan)' : 'var(--led-yellow)'
  )

  function down(e: PointerEvent) {
    ;try { (e.target as Element).setPointerCapture(e.pointerId) } catch {}
    dragging = true
    pressed = true
    startY = e.clientY
    moved = 0
    holdFired = false
    if (onHold) {
      holdTimer = setTimeout(() => {
        holdFired = true
        onHold?.()
      }, 420)
    }
  }

  function move(e: PointerEvent) {
    if (!dragging) return
    const dy = startY - e.clientY
    if (Math.abs(e.clientY - startY) > 3) {
      moved = Math.max(moved, Math.abs(e.clientY - startY))
      clearTimeout(holdTimer)
      const steps = Math.trunc(dy / 6)
      if (steps !== 0) {
        startY = e.clientY
        if (onDelta) onDelta(steps)
        else onValue?.(Math.max(0, Math.min(127, value + steps)))
      }
    }
  }

  function up() {
    if (!dragging) return
    dragging = false
    pressed = false
    clearTimeout(holdTimer)
    if (!holdFired && moved < 4) onPress?.()
  }

  function wheel(e: WheelEvent) {
    e.preventDefault()
    const d = (e.deltaY > 0 ? -1 : 1) * (e.shiftKey ? 5 : 1)
    if (onDelta) onDelta(d)
    else onValue?.(Math.max(0, Math.min(127, value + d)))
  }

  const ticks = Array.from({ length: 11 }, (_, i) => i)
  const angle = $derived((value / 127) * 280 - 140)
</script>

<g transform={`translate(${x},${y})`} style={`--famc:${famColor}`} class="knob" class:selected onpointerdown={down} onpointermove={move} onpointerup={up} onwheel={wheel} tabindex="0" role="slider" aria-label={label} aria-valuenow={value}>
  <circle r={r + 7} fill="none" stroke="transparent" />
  {#each ticks as i (i)}
    <line
      x1="0" y1={-r - 7} x2="0" y2={-r - 3}
      transform={`rotate(${(i / 10) * 280 - 140})`}
      class="tick"
      class:on={i * 12.7 <= value}
    />
  {/each}
  <circle r={r} class="cap" />
  <circle r={r - 5} class="capTop" />
  <line x1="0" y1={-r + 9} x2="0" y2={-r + 2} class="pointer" transform={`rotate(${angle})`} />
  <text y={r + 14} class="label" text-anchor="middle">{label}</text>
  <style>
    .knob { cursor: grab; touch-action: none; outline: none; }
    .knob:active { cursor: grabbing; }
    .knob:focus-visible .cap { stroke: var(--orange); stroke-width: 2; }
    .tick { stroke: var(--panel-well); stroke-width: 2; stroke-linecap: round; }
    .tick.on { stroke: var(--orange); }
    .knob.selected .tick.on { stroke: var(--orange); }
    .cap { fill: var(--ink); stroke: #000; stroke-width: 1; }
    .capTop { fill: #2e2c28; }
    .knob.selected .capTop { fill: #3a382f; }
    .pointer { stroke: var(--panel); stroke-width: 2.5; stroke-linecap: round; }
    .knob.selected .pointer { stroke: var(--famc); }
    .label { font-size: 9px; font-weight: 600; letter-spacing: 0.08em; fill: var(--ink-soft); user-select: none; }
  </style>
</g>
