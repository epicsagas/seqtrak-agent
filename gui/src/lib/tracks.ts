export type Zone = 'drums' | 'synth'
export type Family = 'yellow' | 'cyan'

export interface Track {
  name: string
  label: string
  ch: number
  zone: Zone
  family: Family
}

export const TRACKS: Track[] = [
  { name: 'kick', label: 'KICK', ch: 1, zone: 'drums', family: 'yellow' },
  { name: 'snare', label: 'SNARE', ch: 2, zone: 'drums', family: 'yellow' },
  { name: 'clap', label: 'CLAP', ch: 3, zone: 'drums', family: 'yellow' },
  { name: 'hat1', label: 'HAT 1', ch: 4, zone: 'drums', family: 'yellow' },
  { name: 'hat2', label: 'HAT 2', ch: 5, zone: 'drums', family: 'yellow' },
  { name: 'perc1', label: 'PERC 1', ch: 6, zone: 'drums', family: 'yellow' },
  { name: 'perc2', label: 'PERC 2', ch: 7, zone: 'drums', family: 'yellow' },
  { name: 'synth1', label: 'SYNTH 1', ch: 8, zone: 'synth', family: 'cyan' },
  { name: 'synth2', label: 'SYNTH 2', ch: 9, zone: 'synth', family: 'cyan' },
  { name: 'dx', label: 'DX', ch: 10, zone: 'synth', family: 'cyan' },
  { name: 'sampler', label: 'SAMPLER', ch: 11, zone: 'synth', family: 'cyan' },
]
