<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { api } from '../lib/api'

  let lines = $state<{ label: string; hex: string }[]>([])
  let polling = $state(false)
  let timer: ReturnType<typeof setInterval> | undefined

  async function poll() {
    if (polling) return
    polling = true
    try {
      const r = await api.monitor(1, false)
      if (r.events.length) {
        lines = [...lines, ...r.events].slice(-8)
      }
    } catch {
      /* keep old lines on failure */
    }
    polling = false
  }

  onMount(() => {
    timer = setInterval(poll, 4000)
  })
  onDestroy(() => clearInterval(timer))
</script>

<div class="log">
  <h3>IN</h3>
  {#if lines.length === 0}
    <p class="empty">No MIDI in yet. Play the pads or press play.</p>
  {:else}
    <ul>
      {#each lines as l, i (i)}
        <li><span>{l.label}</span><span>{l.hex}</span></li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .log {
    margin-top: 12px;
  }
  h3 {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.22em;
    color: var(--ink-soft);
  }
  .empty {
    margin: 0;
    font-size: 13px;
    color: var(--ink-soft);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }
  li {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    background: var(--panel-deep);
    padding: 3px 8px;
    border-radius: var(--radius);
  }
  li span:first-child {
    color: var(--ink);
    font-weight: 600;
  }
  li span:last-child {
    color: var(--ink-soft);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
