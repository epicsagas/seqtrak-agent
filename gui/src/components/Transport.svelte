<script lang="ts">
  import { api } from '../lib/api'

  let playing = $state(false)

  async function toggle() {
    playing = !playing
    await api.transport(playing ? 'start' : 'stop').catch(() => {})
  }

  async function stop() {
    playing = false
    await api.transport('stop').catch(() => {})
  }
</script>

<div class="transport">
  <button class="tbtn play" class:on={playing} onclick={toggle} aria-pressed={playing}>
    {playing ? 'PAUSE' : 'PLAY'}
  </button>
  <button class="tbtn" onclick={stop}>STOP</button>
</div>

<style>
  .transport {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }
  .tbtn {
    flex: 1;
    border: 1px solid var(--panel-well);
    background: var(--panel-deep);
    color: var(--ink);
    border-radius: var(--radius);
    padding: 10px 0;
    font-weight: 600;
    letter-spacing: 0.08em;
  }
  .tbtn.play.on {
    background: var(--orange);
    border-color: var(--orange-deep);
    color: #fff;
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      box-shadow: 0 0 10px var(--orange);
    }
  }
</style>
