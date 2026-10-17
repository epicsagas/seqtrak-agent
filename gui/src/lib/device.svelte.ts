// Shared state for the device faceplate view.

export const device = $state({
  selected: 'synth1',
  arm: null as 'mute' | 'solo' | null,
  screenLine1: 'SEQTRAK',
  screenLine2: 'ready',
  playing: false,
  masterVol: 100,
})

export function say(line1: string, line2 = '') {
  device.screenLine1 = line1
  device.screenLine2 = line2
}

export const DRUM_TRACKS = ['kick', 'snare', 'clap', 'hat1', 'hat2', 'perc1', 'perc2']
export const SYNTH_TRACKS = ['synth1', 'synth2', 'dx', 'sampler']
export const ALL_TRACKS = [...DRUM_TRACKS, ...SYNTH_TRACKS]

export const TRACK_LABEL: Record<string, string> = {
  kick: 'KICK', snare: 'SNARE', clap: 'CLAP', hat1: 'HAT 1', hat2: 'HAT 2',
  perc1: 'PERC 1', perc2: 'PERC 2', synth1: 'SYNTH 1', synth2: 'SYNTH 2',
  dx: 'DX', sampler: 'SAMPLER',
}
