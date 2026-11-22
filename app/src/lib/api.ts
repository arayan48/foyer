export type Game = 'minecraft' | 'fivem' | 'valheim' | 'terraria'
export type Status = 'installing' | 'stopped' | 'starting' | 'running' | 'stopping' | 'error'

export interface Server {
  id: string
  name: string
  game: Game
  port: number
  ram_mb: number
  auto_start: boolean
  auto_restart: boolean
  cpu_limit?: number
  cores?: number[]
  ram_cap?: boolean
  status: Status
  cpu: number
  mem: number
  admin_port: number | null
  version?: string
  java?: number
  txadmin?: boolean
  framework?: 'vanilla' | 'esx' | 'qbcore' | null
  // à venir côté agent : la console les affiche dès qu'ils existent
  players?: string[]
  max_players?: number
  uptime_s?: number
}

export interface Catalog {
  minecraft: { id: string; java: number }[]
  java: number[]
  fivem: { id: string; build: string }[]
}

export interface Host {
  name: string
  os: string
  cpu: number
  mem_used: number
  mem_total: number
  disk_free: number
  disk_total: number
  cores?: number
  threads?: number
  cpu_model?: string
  arch?: string
  kernel?: string
  uptime?: number
  agent?: string
}

export interface Conn {
  url: string
  token: string
  /** empreinte SHA-256 du certificat de l'agent, vérifiée à la connexion */
  fp?: string
}

/** Dans l'appli : passage par Rust (certificat épinglé). Dans un navigateur : fetch direct (faux agent de test). */
export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const tauri = () => import('@tauri-apps/api/core')

const KEY = 'foyer.conn'

export function loadConn(): Conn | null {
  try {
    const c: Conn | null = JSON.parse(localStorage.getItem(KEY) ?? 'null')
    // ancienne connexion en HTTP sans empreinte : on se reconnecte
    return inTauri && !c?.fp ? null : c
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
  if (!/^https?:\/\//.test(u)) u = 'https://' + u
  if (!/:\d+$/.test(u)) u += ':7878'
  return u
}

export class AuthError extends Error {}

export async function call<T>(c: Conn, path: string, init: RequestInit = {}): Promise<T> {
  let status: number, text: string
  if (inTauri) {
    const { invoke } = await tauri()
    try {
      ;[status, text] = await invoke<[number, string]>('agent_call', {
        url: c.url, fp: c.fp ?? '', token: c.token, method: init.method ?? 'GET', path, body: init.body ?? null,
      })
    } catch (e) {
      if (e === 'EMPREINTE') throw new AuthError('Le certificat du serveur a changé. Reconnecte-toi et vérifie l\'empreinte.')
      throw new Error(String(e))
    }
  } else {
    const r = await fetch(c.url + path, {
      ...init,
      headers: { Authorization: `Bearer ${c.token}`, 'Content-Type': 'application/json', ...init.headers },
    })
    ;[status, text] = [r.status, await r.text()]
  }
  if (status === 401) throw new AuthError('Jeton refusé')
  if (status < 200 || status >= 300) throw new Error(text || `Erreur ${status}`)
  return status === 204 || !text ? (undefined as T) : JSON.parse(text)
}

/** Empreinte du certificat présenté par l'agent, lue sans envoyer le jeton. */
export async function probe(url: string): Promise<string> {
  const { invoke } = await tauri()
  return invoke<string>('agent_probe', { url })
}

export interface Console {
  send(text: string): void
  close(): void
}

/** Console en direct d'un serveur. onclose est appelé si la connexion tombe. */
export async function openConsole(c: Conn, id: string, online: (l: string) => void, onclose: () => void): Promise<Console> {
  if (!inTauri) {
    const ws = new WebSocket(`${c.url.replace(/^http/, 'ws')}/api/servers/${id}/console`)
    ws.onmessage = (e) => online(e.data)
    ws.onclose = onclose
    await new Promise((ok, ko) => ((ws.onopen = ok), (ws.onerror = ko)))
    return { send: (t) => ws.send(t), close: () => ((ws.onclose = null), ws.close()) }
  }
  const { invoke } = await tauri()
  const { listen } = await import('@tauri-apps/api/event')
  const key = crypto.randomUUID()
  const stop = await listen<{ key: string; line: string | null }>('console', (e) => {
    if (e.payload.key !== key) return
    if (e.payload.line === null) (stop(), onclose())
    else online(e.payload.line)
  })
  try {
    await invoke('console_open', { key, url: c.url, fp: c.fp ?? '', token: c.token, id })
  } catch (e) {
    stop()
    throw e
  }
  return {
    send: (text) => invoke('console_send', { key, text }),
    close: () => (stop(), invoke('console_close', { key })),
  }
}

export function patchServer(c: Conn, id: string, body: Partial<Server>) {
  return call<Server>(c, `/api/servers/${id}`, { method: 'PATCH', body: JSON.stringify(body) })
}

export function host(c: Conn) {
  return new URL(c.url).hostname
}

export const GAMES: Record<Game, { label: string; hint: string; port: number; min: string; cats: string[]; soon?: boolean }> = {
  minecraft: { label: 'Minecraft', hint: 'Java Edition', port: 25565, min: '1 cœur · 1 Go RAM · 5 Go disque', cats: ['Survie', 'Bac à sable'] },
  fivem: { label: 'FiveM', hint: 'GTA V roleplay, ESX ou QBCore', port: 30120, min: '1 cœur · 2 Go RAM · 10 Go disque', cats: ['Jeu de rôle'], soon: true },
  valheim: { label: 'Valheim', hint: 'Survie viking en coopération', port: 2456, min: '2 cœurs · 4 Go RAM · 2 Go disque', cats: ['Survie', 'Coopération'], soon: true },
  terraria: { label: 'Terraria', hint: 'Aventure 2D, serveur TShock', port: 7777, min: '1 cœur · 512 Mo RAM · 500 Mo disque', cats: ['Aventure', 'Bac à sable'], soon: true },
}

export type Software = 'vanilla' | 'paper' | 'purpur' | 'fabric' | 'forge' | 'neoforge'

/** Logiciels serveur Minecraft. addons : dossier des extensions, null si aucune. */
export const SOFTWARE: Record<Software, { label: string; hint: string; addons: 'plugins' | 'mods' | null }> = {
  vanilla: { label: 'Vanilla', hint: 'Le jeu officiel de Mojang', addons: null },
  paper: { label: 'Paper', hint: 'Plugins, plus rapide', addons: 'plugins' },
  purpur: { label: 'Purpur', hint: 'Plugins, plus de réglages que Paper', addons: 'plugins' },
  fabric: { label: 'Fabric', hint: 'Mods légers et récents', addons: 'mods' },
  forge: { label: 'Forge', hint: 'Mods, gros modpacks', addons: 'mods' },
  neoforge: { label: 'NeoForge', hint: 'Mods, suite de Forge', addons: 'mods' },
}

/** « NeoForge 1.21.1 » → { id: 'neoforge', mc: '1.21.1' }. Sans version : Paper, comme l'agent. */
export function software(s: Server): { id: Software; mc: string } {
  const [name, mc = ''] = (s.version ?? '').split(' ')
  const id = name.toLowerCase() as Software
  return { id: id in SOFTWARE ? id : 'paper', mc }
}

/** Compare deux versions numérotées (1.9 < 1.21 < 26.3). */
export function cmpVersion(a: string, b: string) {
  return a.localeCompare(b, 'en', { numeric: true })
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
