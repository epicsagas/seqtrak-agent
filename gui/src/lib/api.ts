// Thin fetch wrapper over the st serve API.

async function post(path: string, body: unknown): Promise<unknown> {
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const data = (await res.json().catch(() => null)) as { error?: string } | null
    throw new Error(data?.error ?? `HTTP ${res.status}`)
  }
  return res.json()
}

export async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(path)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json() as Promise<T>
}

export interface SeqState {
  playing: boolean
  rec: boolean
  step: number
  patterns: number[][]
}

export const api = {
  health: () => getJson<{ ok: boolean; link: boolean; outputs: string[]; inputs: string[] }>('/api/health'),
  identity: () => getJson<{ version?: string; raw?: string }>('/api/identity'),
  note: (track: string, note = 60, vel = 100, dur = 150) =>
    post('/api/note', { track, note, vel, dur }),
  cc: (track: string, cc: string, val: number) => post('/api/cc', { track, cc, val }),
  pc: (track: string, msb: number, lsb: number, prog: number) =>
    post('/api/pc', { track, msb, lsb, prog }),
  load: (project: number) => post('/api/load', { project }),
  transport: (action: 'start' | 'stop') => post('/api/transport', { action }),
  param: (addr: string, bytes: number[]) => post('/api/param', { addr, bytes }),
  paramRead: (addr: string) =>
    getJson<{ addr: string; bytes: number[] }>(`/api/param?addr=${encodeURIComponent(addr)}`),
  monitor: (seconds: number, clock = false) =>
    post('/api/monitor', { seconds, clock }) as Promise<{ events: { label: string; hex: string }[] }>,
  seq: () => getJson<SeqState>('/api/seq'),
  seqOp: (body: Record<string, unknown>) => post('/api/seq', body),
}
