<script lang="ts">
  // Vertical touch slider strip (FX LEVEL / HIGH PASS / REPEATER).
  let {
    x, y, w = 34, h = 120, label, value = $bindable(64), spring = false, color = 'cyan',
    onInput, onRelease,
  }: {
    x: number; y: number; w?: number; h?: number; label: string
    value?: number; spring?: boolean; color?: 'cyan' | 'yellow' | 'orange'
    onInput?: (v: number) => void; onRelease?: () => void
  } = $props()

  let dragging = $state(false)
  const accent = $derived(color === 'orange' ? 'var(--orange)' : color === 'yellow' ? 'var(--led-yellow)' : 'var(--led-cyan)')

  function pos(e: PointerEvent, el: HTMLElement) {
    const rect = el.getBoundingClientRect()
    const rel = rect.height ? 1 - (e.clientY - rect.top) / rect.height : 0
    return Math.max(0, Math.min(127, Math.round(rel * 127)))
  }

  function down(e: PointerEvent) {
    ;try { (e.currentTarget as Element).setPointerCapture(e.pointerId) } catch {}
    dragging = true
    value = pos(e, e.currentTarget as HTMLElement)
    onInput?.(value)
  }

  function move(e: PointerEvent) {
    if (!dragging) return
    value = pos(e, e.currentTarget as HTMLElement)
    onInput?.(value)
  }

  function up() {
    if (!dragging) return
    dragging = false
    if (spring) value = 0
    onRelease?.()
  }

  function dbl() {
    value = spring ? 0 : 64
    onInput?.(value)
  }

  const fillY = $derived(h - (value / 127) * h)
</script>

<g transform={`translate(${x},${y})`} class="slider" role="slider" tabindex="0" aria-label={label} aria-valuemin={0} aria-valuemax={127} aria-valuenow={value} data-label={label} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} ondblclick={dbl} style={`--accent:${accent}`}>
  <rect width={w} height={h} rx="3" class="well" />
  <rect y={fillY} width={w} height={h - fillY} rx="3" class="fill" />
  <line x1="0" y1={fillY} x2={w} y2={fillY} class="cap-line" />
  <text x={w / 2} y={h + 12} text-anchor="middle" class="label">{label}</text>
  <style>
    .slider { cursor: ns-resize; touch-action: none; }
    .well { fill: var(--chassis-deep); stroke: #000; }
    .fill { fill: var(--accent); opacity: 0.85; pointer-events: none; }
    .cap-line { stroke: #fff; stroke-width: 2; pointer-events: none; }
    .label { font-size: 8.5px; font-weight: 600; letter-spacing: 0.06em; fill: var(--ink-soft); user-select: none; pointer-events: none; }
  </style>
</g>
