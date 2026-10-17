<script lang="ts">
  // Hardware key: fires on pointerdown, optional hold-repeat, visible pressed state.
  let {
    x, y, w = 34, h = 34, rx = 3, label, family = 'yellow',
    onDown, onUp, repeat = false, dark = false, lit = false, play = false,
  }: {
    x: number; y: number; w?: number; h?: number; rx?: number; label?: string
    family?: 'yellow' | 'cyan' | 'orange' | 'none'; dark?: boolean
    onDown?: () => void; onUp?: () => void; repeat?: boolean
    lit?: boolean; play?: boolean
  } = $props()

  let held = $state(false)
  let repeatTimer: ReturnType<typeof setInterval> | undefined
  let minTimer: ReturnType<typeof setTimeout> | undefined

  const fam = $derived(
    family === 'orange' ? 'var(--orange)'
    : family === 'cyan' ? 'var(--led-cyan)'
    : family === 'none' ? 'transparent'
    : 'var(--led-yellow)'
  )

  function down(e: PointerEvent) {
    ;try { (e.target as Element).setPointerCapture(e.pointerId) } catch {}
    held = true
    minTimer = setTimeout(() => {}, 90)
    onDown?.()
    if (repeat) {
      setTimeout(() => {
        if (held) repeatTimer = setInterval(() => onDown?.(), 125)
      }, 300)
    }
  }

  function up() {
    if (!held) return
    held = false
    clearInterval(repeatTimer)
    onUp?.()
  }
</script>

<g transform={`translate(${x},${y})`} style={`--fam:${fam}`} class="key" role="button" tabindex="0" aria-label={label ?? 'key'} data-label={label} class:held class:dark class:lit class:play onpointerdown={down} onpointerup={up} onpointercancel={up}>
  <rect width={w} height={h} rx={rx} />
  {#if label}
    <text x={w / 2} y={h / 2 + 3.5} text-anchor="middle">{label}</text>
  {/if}
  <style>
    .key { cursor: pointer; touch-action: none; }
    rect {
      fill: var(--panel);
      stroke: var(--panel-well);
      stroke-width: 1;
      transition: fill 60ms linear;
    }
    .dark rect { fill: var(--ink); stroke: #000; }
    .key.held rect { fill: var(--fam); stroke: var(--fam); }
    .dark.held rect { fill: var(--fam); }
    /* lit = stored step; play = transport playhead on this key */
    .key.lit rect {
      fill: color-mix(in srgb, var(--fam) 55%, var(--panel));
      stroke: var(--fam);
    }
    .dark.lit rect { fill: var(--fam); }
    .key.play rect { stroke-width: 2.5; stroke: var(--ink); }
    .key.play.lit rect { stroke: var(--ink); }
    text {
      fill: var(--ink);
      font-size: 9px;
      font-weight: 600;
      letter-spacing: 0.06em;
      user-select: none;
      pointer-events: none;
    }
    .dark text { fill: var(--panel); }
    .key.held text { fill: var(--ink); }
  </style>
</g>
