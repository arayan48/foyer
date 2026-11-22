// Faux agent pour travailler l'interface sans serveur : node mock-agent.mjs (jeton : test)
import http from 'node:http'

const servers = [
  { id: 'a1', name: 'Survie entre potes', game: 'minecraft', port: 25565, ram_mb: 4096, status: 'running', cpu: 23, mem: 2.6e9, admin_port: null, disk: 1.8e9, players: ['Bob', 'Eve_1'], version: 'Paper 1.21.10', auto_start: false, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
  { id: 'b2', name: 'RP Los Santos', game: 'fivem', port: 30120, ram_mb: 4096, status: 'stopped', cpu: 0, mem: 0, admin_port: 40120, version: 'Build 17000', auto_start: true, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
  { id: 'c3', name: 'Créatif', game: 'minecraft', port: 25566, ram_mb: 2048, status: 'installing', cpu: 0, mem: 0, admin_port: null, auto_start: false, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
]
// fichiers : chemin → texte (null = dossier)
const files = new Map([['world', null], ['world/level.dat', 'a\0b'], ['plugins', null], ['server.properties', 'motd=Chez nous\r\nmax-players=10\r\n'], ['eula.txt', 'eula=true\n']])
const players = { online: ['Bob', 'Eve_1'], ops: ['Bob'], whitelist: ['Bob', 'Max'], banned: ['Troll'] }
const trash = []
const journal = []
let props = { motd: 'Chez nous', difficulty: 'normal', 'max-players': '10' }
const plugins = [{ file: 'EssentialsX-2.21.jar', enabled: true, size: 3.2e6 }, { file: 'Dynmap-3.7.jar.disabled', enabled: false, size: 8e6 }]
const sys = { name: 'WIN-RVSDEAUH9KJ', os: 'Windows Server 2025 Standard', cpu: 18, mem_used: 6.1e9, mem_total: 17.1e9, disk_free: 4.6e11, disk_total: 5e11, cores: 4, threads: 8, cpu_model: 'Intel(R) Xeon(R) E-2236 CPU @ 3.40GHz', arch: 'x86_64', kernel: '26100', uptime: 412345, agent: '0.2.0' }

http.createServer(async (q, r) => {
  r.setHeader('Access-Control-Allow-Origin', '*')
  r.setHeader('Access-Control-Allow-Headers', '*')
  r.setHeader('Access-Control-Allow-Methods', '*')
  if (q.method === 'OPTIONS') return r.end()
  if (q.headers.authorization !== 'Bearer test') return (r.statusCode = 401), r.end()
  const json = (v) => (r.setHeader('Content-Type', 'application/json'), r.end(JSON.stringify(v)))
  const u = new URL(q.url, 'http://x')
  const body = async () => { const b = []; for await (const c of q) b.push(c); return Buffer.concat(b) }
  if (q.method !== 'GET') journal.unshift({ at: Math.floor(Date.now() / 1000), ip: '127.0.0.1', action: `${q.method} ${q.url}`, status: 204 })
  if (u.pathname === '/api/journal') return json(journal)
  const tr = u.pathname.match(/^\/api\/trash(?:\/(\w+))?$/)
  if (tr) {
    if (!tr[1]) return json(trash)
    const [s] = trash.splice(trash.findIndex((s) => s.id === tr[1]), 1)
    if (q.method === 'DELETE') return r.writeHead(204).end()
    servers.push(s)
    return json(s)
  }
  const pm = u.pathname.match(/^\/api\/servers\/\w+\/players(?:\/(.+)\/(\w+))?$/)
  if (pm) {
    const [, n, a] = pm
    const drop = (k) => (players[k] = players[k].filter((x) => x !== n))
    const put = (k) => players[k].includes(n) || players[k].push(n)
    if (!n) return json(players)
    ;({ kick: () => drop('online'), ban: () => (drop('online'), put('banned')), pardon: () => drop('banned'), op: () => put('ops'), deop: () => drop('ops'), whitelist_add: () => put('whitelist'), whitelist_remove: () => drop('whitelist') })[a]()
    return r.writeHead(204).end()
  }
  const fm = u.pathname.match(/^\/api\/servers\/\w+\/files(\/content)?$/)
  if (fm) {
    const path = u.searchParams.get('path') ?? ''
    if (fm[1]) {
      if (q.method === 'PUT') return files.set(path, (await body()).toString()), r.writeHead(204).end()
      return files.has(path) ? r.end(files.get(path)) : r.writeHead(404).end('Fichier introuvable')
    }
    const under = [...files.keys()].filter((k) => k === path || k.startsWith(path + '/'))
    if (q.method === 'POST') return files.has(path) ? r.writeHead(409).end('Ce nom existe déjà') : (files.set(path, null), r.writeHead(201).end())
    if (q.method === 'DELETE') return under.forEach((k) => files.delete(k)), r.writeHead(204).end()
    if (q.method === 'PATCH') {
      const { to } = JSON.parse(await body())
      if (files.has(to)) return r.writeHead(409).end('Ce nom existe déjà')
      under.forEach((k) => (files.set(to + k.slice(path.length), files.get(k)), files.delete(k)))
      return r.writeHead(204).end()
    }
    const pre = path ? path + '/' : ''
    return json([...files].filter(([k]) => k.startsWith(pre) && !k.slice(pre.length).includes('/')).map(([k, v]) => ({ name: k.slice(pre.length), dir: v === null, size: v?.length ?? 0, modified: 1795376593 })).sort((a, b) => b.dir - a.dir || a.name.localeCompare(b.name)))
  }
  if (q.url === '/api/system') return json({ ...sys, cpu: 10 + Math.random() * 20 })
  if (q.url === '/api/catalog') return json({ minecraft: [{ id: '26.3', java: 25 }, { id: '1.21.11', java: 21 }, { id: '1.20.4', java: 17 }, { id: '1.16.5', java: 8 }], java: [8, 11, 17, 21, 22, 23, 24, 25], fivem: [{ id: 'recommended', build: '38435' }, { id: 'optional', build: '7290' }, { id: 'latest', build: '38781' }] })
  if (/^\/api\/catalog\/\w+$/.test(q.url)) return json([{ id: '26.3', java: 25 }, { id: '1.21.11', java: 21 }, { id: '1.21.10', java: 21 }, { id: '1.20.4', java: 17 }])
  const sw = q.url.match(/^\/api\/servers\/(\w+)\/software$/)
  if (sw) {
    let b = ''
    for await (const c of q) b += c
    b = JSON.parse(b)
    const s = servers.find((s) => s.id === sw[1])
    if (s.status !== 'stopped') return r.writeHead(409).end("Arrête le serveur d'abord")
    s.status = 'installing'
    setTimeout(() => Object.assign(s, { status: 'stopped', version: `${{ vanilla: 'Vanilla', paper: 'Paper', purpur: 'Purpur', fabric: 'Fabric', forge: 'Forge', neoforge: 'NeoForge' }[b.software]} ${b.version}` }), 3000)
    return r.writeHead(204).end()
  }
  if (q.url === '/api/servers' && q.method === 'POST') {
    let body = ''
    q.on('data', (c) => (body += c))
    return q.on('end', () => {
      const b = JSON.parse(body)
      const s = { id: Math.random().toString(16).slice(2, 10), name: b.name, game: b.game, port: b.port, ram_mb: b.ram_mb, status: 'installing', cpu: 0, mem: 0, admin_port: b.game === 'fivem' && b.txadmin ? b.port + 10000 : null, auto_start: false, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false, java: b.java, txadmin: b.game !== 'fivem' || b.txadmin, framework: b.framework ?? null }
      servers.push(s)
      console.log('création', b)
      json(s)
    })
  }
  if (q.url === '/api/servers') return json(servers.map((s) => (s.status === 'running' ? { ...s, cpu: 15 + Math.random() * 20 } : s)))
  if (/^\/api\/servers\/\w+\/properties$/.test(q.url)) {
    if (q.method === 'PUT') { let b = ''; for await (const c of q) b += c; props = { ...props, ...JSON.parse(b) }; return r.writeHead(204).end() }
    return json(props)
  }
  const pl = q.url.match(/^\/api\/servers\/(\w+)\/plugins(?:\/(.+))?$/)
  if (pl) {
    const f = pl[2] && decodeURIComponent(pl[2])
    if (f?.startsWith('search')) return json({ total: 137, per_page: 20, hits: Array.from({ length: 20 }, (_, i) => ({ id: 'p' + i, title: ['LuckPerms', 'WorldEdit', 'EssentialsX', 'Chunky', 'CoreProtect'][i % 5] + (i > 4 ? ' ' + i : ''), description: 'Un plugin très utile pour ton serveur, avec une description assez longue pour tenir sur deux lignes.', author: 'Auteur', downloads: 9100000 / (i + 1), follows: 4200 / (i + 1), updated: new Date(Date.now() - i * 9e8).toISOString(), icon: null })) })
    if (q.method === 'POST') return plugins.push({ file: 'LuckPerms-5.4.jar', enabled: true, size: 1.4e6 }), json('LuckPerms-5.4.jar')
    const i = plugins.findIndex((p) => p.file === f)
    if (q.method === 'DELETE') return plugins.splice(i, 1), r.writeHead(204).end()
    if (q.method === 'PATCH') return (plugins[i] = { ...plugins[i], enabled: !plugins[i].enabled, file: plugins[i].enabled ? f + '.disabled' : f.replace('.disabled', '') }), r.writeHead(204).end()
    return json(plugins)
  }
  const p = q.url.match(/^\/api\/servers\/(\w+)$/)
  if (p && q.method === 'DELETE') {
    const i = servers.findIndex((s) => s.id === p[1])
    if (i < 0) return r.writeHead(404).end()
    trash.unshift({ ...servers.splice(i, 1)[0], status: 'stopped', deleted_at: Math.floor(Date.now() / 1000) })
    return r.writeHead(204).end()
  }
  if (p && q.method === 'PATCH') {
    let body = ''
    q.on('data', (c) => (body += c))
    return q.on('end', () => {
      const s = servers.find((s) => s.id === p[1])
      Object.assign(s, JSON.parse(body))
      json(s)
    })
  }
  const m = q.url.match(/^\/api\/servers\/(\w+)\/(start|stop|restart)$/)
  if (m) {
    const s = servers.find((s) => s.id === m[1])
    s.status = m[2] === 'stop' ? 'stopped' : 'running'
    r.statusCode = 204
    return r.end()
  }
  r.statusCode = 404
  r.end()
}).listen(Number(process.env.PORT) || 7878, '127.0.0.1')
