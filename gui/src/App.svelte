<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from './lib/api'
  import { TRACKS } from './lib/tracks'
  import AllKnob from './components/AllKnob.svelte'
  import TrackStrip from './components/TrackStrip.svelte'
  import PadGrid from './components/PadGrid.svelte'
  import Transport from './components/Transport.svelte'
  import ArpBlock from './components/ArpBlock.svelte'
  import EventLog from './components/EventLog.svelte'
  import DeviceFace from './components/device/DeviceFace.svelte'
  import HelpPanel from './components/HelpPanel.svelte'

  let view = $state<'mixer' | 'device'>('device')
  let help = $state(false)
  let health = $state<{ ok: boolean; link: boolean; outputs: string[] } | null>(null)
  let fw = $state('')
  let error = $state('')
  let project = $state(1)
  let loadingProject = $state(0)

  onMount(async () => {
    for (;;) {
      try {
        health = await api.health()
        error = ''
        if (health.link && !fw) fw = (await api.identity().catch(() => null))?.version ?? ''
      } catch {
        error = 'SEQTRAK not connected. Plug in USB and power on; st serve reconnects automatically.'
      }
      await new Promise((r) => setTimeout(r, 3000))
    }
  })

  async function loadProject(n: number) {
    loadingProject = n
    try {
      await api.load(n)
      project = n
    } catch (e) {
      error = `load failed: ${e}`
    }
    loadingProject = 0
  }

  const drums = TRACKS.filter((t) => t.zone === 'drums')
  const synths = TRACKS.filter((t) => t.zone === 'synth')
</script>

{#if error}
  <div class="error">{error}</div>
{/if}

<div class="face">
  <header class="top">
    <span class="brand">SEQTRAK AGENT</span>
    <span class="sub">
      {#if fw}OS {fw}{:else}--{/if}
      {#if health?.link}
        connected
      {:else}
        offline
      {/if}
    </span>
    <span class="ticker" aria-live="polite">{error ? 'link down' : 'link up'}</span>
    <nav class="views">
      <button class:von={view === 'device'} onclick={() => (view = 'device')}>DEVICE</button>
      <button class:von={view === 'mixer'} onclick={() => (view = 'mixer')}>MIXER</button>
      <button class="helpbtn" onclick={() => (help = !help)} aria-expanded={help}>HELP</button>
    </nav>
  </header>

  {#if help}
    <HelpPanel view={view} onclose={() => (help = false)} />
  {/if}

  {#if view === 'device'}
    <DeviceFace />
  {:else}
  <p class="mhint">strips: volume/pan, M mute, S solo, HIT audition . pads trigger drums . ARP controls synth1/synth2/dx . MASTER: ALL volume, transport, project P1-P8, MIDI log . HELP = manual (EN/KO)</p>
  <main class="zones">
    <section class="zone" aria-label="drums">
      <h2>DRUMS</h2>
      <div class="strips drums">
        {#each drums as t (t.name)}
          <TrackStrip track={t} />
        {/each}
      </div>
      <PadGrid />
    </section>

    <section class="zone" aria-label="synth and sampler">
      <h2>SYNTH / SAMPLER</h2>
      <div class="strips synth">
        {#each synths as t (t.name)}
          <TrackStrip track={t} />
        {/each}
      </div>
      <ArpBlock />
    </section>

    <section class="zone" aria-label="master">
      <h2>MASTER</h2>
      <AllKnob />
      <Transport />
      <div class="projects">
        <h3>PROJECT</h3>
        <div class="pkeys">
          {#each [1, 2, 3, 4, 5, 6, 7, 8] as n (n)}
            <button
              class="pkey"
              class:on={project === n}
              disabled={loadingProject === n}
              onclick={() => loadProject(n)}>P{n}</button
            >
          {/each}
        </div>
      </div>
      <EventLog />
    </section>
  </main>
  {/if}
</div>

<style>
  .error {
    position: fixed;
    inset: auto 16px 16px;
    background: var(--led-red);
    color: #fff;
    padding: 10px 14px;
    border-radius: var(--radius);
    z-index: 10;
  }
  .face {
    max-width: 1240px;
    margin: 0 auto;
    padding: 18px 20px 26px;
  }
  .top {
    display: flex;
    align-items: baseline;
    gap: 14px;
    padding-bottom: 14px;
  }
  .brand {
    font-size: 21px;
    font-weight: 600;
    letter-spacing: 0.14em;
    color: var(--orange);
  }
  .sub {
    color: var(--ink-soft) !important;
    filter: brightness(1.9);
    font-variant-numeric: tabular-nums;
  }
  .ticker {
    font-size: 13px;
    color: var(--led-cyan);
  }
  .views {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .views button {
    border: 1px solid var(--chassis-deep);
    background: transparent;
    color: var(--panel);
    padding: 4px 12px;
    border-radius: var(--radius);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.1em;
  }
  .views button.von {
    background: var(--orange);
    border-color: var(--orange-deep);
    color: #fff;
  }
  .views .helpbtn {
    border-style: dashed;
  }
  .mhint {
    margin: 0 0 10px;
    font-size: 12px;
    color: #97937f;
  }
  .zones {
    display: grid;
    grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr) minmax(0, 0.95fr);
    gap: 14px;
    align-items: start;
  }
  .zone {
    min-width: 0;
  }
  .zone {
    background: var(--panel);
    color: var(--ink);
    border-radius: var(--radius);
    padding: 12px 12px 14px;
  }
  h2 {
    margin: 0 0 10px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.22em;
    color: var(--ink-soft);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.22em;
    color: var(--ink-soft);
  }
  .strips.drums {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 5px;
  }
  .strips.drums :global(.ch) {
    display: none;
  }
  .strips.synth {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 5px;
  }
  .projects {
    margin-top: 12px;
  }
  .pkeys {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 4px;
  }
  .pkey {
    border: 1px solid var(--panel-well);
    background: var(--panel-deep);
    color: var(--ink);
    padding: 6px 0;
    border-radius: var(--radius);
    font-weight: 600;
    font-size: 12px;
  }
  .pkey.on {
    background: var(--orange);
    border-color: var(--orange-deep);
    color: #fff;
  }
  @media (max-width: 980px) {
    .zones {
      grid-template-columns: 1fr;
    }
    .strips.drums {
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }
  }
</style>
