// Faux agent pour travailler l'interface sans serveur : node mock-agent.mjs (jeton : test)
import http from 'node:http'

const servers = [
  { id: 'a1', name: 'Survie entre potes', game: 'minecraft', port: 25565, ram_mb: 4096, status: 'running', cpu: 23, mem: 2.6e9, admin_port: null, version: 'Paper 1.21.10', auto_start: false, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
  { id: 'b2', name: 'RP Los Santos', game: 'fivem', port: 30120, ram_mb: 4096, status: 'stopped', cpu: 0, mem: 0, admin_port: 40120, version: 'Build 17000', auto_start: true, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
  { id: 'c3', name: 'Créatif', game: 'minecraft', port: 25566, ram_mb: 2048, status: 'installing', cpu: 0, mem: 0, admin_port: null, auto_start: false, auto_restart: true, cpu_limit: 0, cores: [], ram_cap: false },
]
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
  if (q.url === '/api/system') return json({ ...sys, cpu: 10 + Math.random() * 20 })
  if (q.url === '/api/catalog') return json({ minecraft: [{ id: '26.3', java: 25 }, { id: '1.21.11', java: 21 }, { id: '1.20.4', java: 17 }, { id: '1.16.5', java: 8 }], java: [8, 11, 17, 21, 22, 23, 24, 25], fivem: [{ id: 'recommended', build: '38435' }, { id: 'optional', build: '7290' }, { id: 'latest', build: '38781' }] })
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
    servers.splice(i, 1)
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
