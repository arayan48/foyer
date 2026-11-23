// Pack joueurs : un .mrpack (format Modrinth) que Modrinth App ou Prism installe en un clic.
// Le pack ne contient que des liens Modrinth, plus la config Iris et la fiche du serveur.

export type PackItem = { id: string; title: string; kind: 'mod' | 'shader' }

const API = 'https://api.modrinth.com/v2'
const get = (u: string) => fetch(u).then((r) => (r.ok ? r.json() : Promise.reject(new Error(`Modrinth : ${r.status}`))))

// Iris fait tourner les shaders, Sodium est requis par Iris
const SHADER_MODS = ['iris', 'sodium']

type Entry = { path: string; hashes: { sha1: string; sha512: string }; env: { client: string; server: string }; downloads: string[]; fileSize: number }

/** Construit le .mrpack. Renvoie les octets et les titres introuvables pour cette version. */
export async function buildPack(items: PackItem[], mc: string, name: string, server: { name: string; ip: string }) {
  const files: Entry[] = []
  const done = new Set<string>()
  const missing: string[] = []
  let shader = ''

  async function add(id: string, kind: 'mod' | 'shader', title = id) {
    if (done.has(id)) return
    done.add(id)
    const qs = new URLSearchParams({ loaders: JSON.stringify([kind === 'shader' ? 'iris' : 'fabric']), game_versions: JSON.stringify([mc]) })
    const v = (await get(`${API}/project/${id}/version?${qs}`))[0]
    if (!v) return void missing.push(title)
    const f = v.files.find((f: any) => f.primary) ?? v.files[0]
    const path = `${kind === 'shader' ? 'shaderpacks' : 'mods'}/${f.filename}`
    if (files.some((e) => e.path === path)) return
    if (kind === 'shader' && !shader) shader = f.filename
    files.push({ path, hashes: { sha1: f.hashes.sha1, sha512: f.hashes.sha512 }, env: { client: 'required', server: 'unsupported' }, downloads: [f.url], fileSize: f.size })
    for (const d of v.dependencies ?? []) if (d.dependency_type === 'required' && d.project_id) await add(d.project_id, 'mod')
  }

  if (items.some((i) => i.kind === 'shader')) for (const m of SHADER_MODS) await add(m, 'mod')
  for (const i of items) await add(i.id, i.kind, i.title)

  const loader = (await fetch(`https://meta.fabricmc.net/v2/versions/loader/${mc}`).then((r) => r.json()))[0]?.loader?.version
  if (!loader) throw new Error(`Fabric ne gère pas encore Minecraft ${mc}`)

  const enc = new TextEncoder()
  const index = { formatVersion: 1, game: 'minecraft', versionId: '1', name, files, dependencies: { minecraft: mc, 'fabric-loader': loader } }
  const zip: [string, Uint8Array][] = [
    ['modrinth.index.json', enc.encode(JSON.stringify(index, null, 2))],
    ['overrides/servers.dat', serversDat(server.name, server.ip)],
  ]
  if (shader) zip.push(['overrides/config/iris.properties', enc.encode(`enableShaders=true\nshaderPack=${shader}\n`)])
  return { bytes: makeZip(zip), missing }
}

// servers.dat : NBT non compressé, { servers: [ { name, ip } ] }
function serversDat(name: string, ip: string) {
  const out: number[] = []
  const str = (s: string) => { const b = new TextEncoder().encode(s); out.push(b.length >> 8, b.length & 255, ...b) }
  out.push(10); str('')
  out.push(9); str('servers'); out.push(10, 0, 0, 0, 1)
  out.push(8); str('name'); str(name)
  out.push(8); str('ip'); str(ip)
  out.push(0, 0)
  return new Uint8Array(out)
}

const CRC = Array.from({ length: 256 }, (_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0 })
const crc32 = (b: Uint8Array) => { let c = ~0; for (const x of b) c = CRC[(c ^ x) & 255] ^ (c >>> 8); return ~c >>> 0 }

/** Zip sans compression (fichiers minuscules, inutile de compresser). */
export function makeZip(entries: [string, Uint8Array][]) {
  const parts: Uint8Array[] = []
  const central: Uint8Array[] = []
  let off = 0
  for (const [name, data] of entries) {
    const n = new TextEncoder().encode(name)
    const crc = crc32(data)
    const head = (sig: number, extra: number) => {
      const h = new DataView(new ArrayBuffer(extra))
      h.setUint32(0, sig, true)
      return h
    }
    const l = head(0x04034b50, 30)
    l.setUint16(4, 20, true); l.setUint16(6, 0x800, true); l.setUint32(14, crc, true)
    l.setUint32(18, data.length, true); l.setUint32(22, data.length, true); l.setUint16(26, n.length, true)
    const c = head(0x02014b50, 46)
    c.setUint16(4, 20, true); c.setUint16(6, 20, true); c.setUint16(8, 0x800, true); c.setUint32(16, crc, true)
    c.setUint32(20, data.length, true); c.setUint32(24, data.length, true); c.setUint16(28, n.length, true); c.setUint32(42, off, true)
    parts.push(new Uint8Array(l.buffer), n, data)
    central.push(new Uint8Array(c.buffer), n)
    off += 30 + n.length + data.length
  }
  const size = central.reduce((s, b) => s + b.length, 0)
  const e = new DataView(new ArrayBuffer(22))
  e.setUint32(0, 0x06054b50, true); e.setUint16(8, entries.length, true); e.setUint16(10, entries.length, true)
  e.setUint32(12, size, true); e.setUint32(16, off, true)
  const all = [...parts, ...central, new Uint8Array(e.buffer)]
  const out = new Uint8Array(all.reduce((s, b) => s + b.length, 0))
  let p = 0
  for (const b of all) out.set(b, p), (p += b.length)
  return out
}
