export type Game = 'minecraft' | 'fivem'
export type Status = 'installing' | 'stopped' | 'starting' | 'running' | 'stopping' | 'error'

export interface Server {
  id: string
  name: string
  game: Game
  port: number
  ram_mb: number
  status: Status
  cpu: number
  mem: number
  admin_port: number | null
}

export interface Host {
  name: string
  os: string
  cpu: number
  mem_used: number
  mem_total: number
  disk_free: number
  disk_total: number
}

export interface Conn {
  url: string
  token: string
}

const KEY = 'foyer.conn'

export function loadConn(): Conn | null {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? 'null')
  } catch {
    return null
  }
}

export function saveConn(c: Conn | null) {
  try {
    c ? localStorage.setItem(KEY, JSON.stringify(c)) : localStorage.removeItem(KEY)
  } catch {}
}

export function normalizeUrl(u: string) {
  u = u.trim().replace(/\/+$/, '')
  if (!/^https?:\/\//.test(u)) u = 'http://' + u
  if (!/:\d+$/.test(u)) u += ':7878'
  return u
}

export class AuthError extends Error {}

export async function call<T>(c: Conn, path: string, init: RequestInit = {}): Promise<T> {
  const r = await fetch(c.url + path, {
    ...init,
    headers: { Authorization: `Bearer ${c.token}`, 'Content-Type': 'application/json', ...init.headers },
  })
  if (r.status === 401) throw new AuthError('Jeton refusé')
  if (!r.ok) throw new Error((await r.text()) || `Erreur ${r.status}`)
  return r.status === 204 ? (undefined as T) : r.json()
}

export function consoleUrl(c: Conn, id: string) {
  return `${c.url.replace(/^http/, 'ws')}/api/servers/${id}/console?token=${encodeURIComponent(c.token)}`
}

export function host(c: Conn) {
  return new URL(c.url).hostname
}

export const GAMES: Record<Game, { label: string; hint: string; port: number }> = {
  minecraft: { label: 'Minecraft', hint: 'Java Edition, serveur Paper', port: 25565 },
  fivem: { label: 'FiveM', hint: 'GTA V roleplay, avec txAdmin', port: 30120 },
}

export const STATUS: Record<Status, string> = {
  installing: 'Installation',
  stopped: 'Arrêté',
  starting: 'Démarrage',
  running: 'En ligne',
  stopping: 'Arrêt',
  error: 'Erreur',
}

export function bytes(n: number) {
  if (n >= 1024 ** 3) return (n / 1024 ** 3).toFixed(1).replace('.', ',') + ' Go'
  return Math.round(n / 1024 ** 2) + ' Mo'
}
