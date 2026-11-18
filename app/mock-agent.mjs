// Faux agent pour travailler l'interface sans serveur : node mock-agent.mjs (jeton : test)
import http from 'node:http'

const servers = [
  { id: 'a1', name: 'Survie entre potes', game: 'minecraft', port: 25565, ram_mb: 4096, status: 'running', cpu: 23, mem: 2.6e9, admin_port: null },
  { id: 'b2', name: 'RP Los Santos', game: 'fivem', port: 30120, ram_mb: 4096, status: 'stopped', cpu: 0, mem: 0, admin_port: 40120 },
  { id: 'c3', name: 'Créatif', game: 'minecraft', port: 25566, ram_mb: 2048, status: 'installing', cpu: 0, mem: 0, admin_port: null },
]
const sys = { name: 'WIN-RVSDEAUH9KJ', os: 'Windows Server 2025 Standard', cpu: 18, mem_used: 6.1e9, mem_total: 17.1e9, disk_free: 4.6e11, disk_total: 5e11 }

http.createServer((q, r) => {
  r.setHeader('Access-Control-Allow-Origin', '*')
  r.setHeader('Access-Control-Allow-Headers', '*')
  r.setHeader('Access-Control-Allow-Methods', '*')
  if (q.method === 'OPTIONS') return r.end()
  if (q.headers.authorization !== 'Bearer test') return (r.statusCode = 401), r.end()
  const json = (v) => (r.setHeader('Content-Type', 'application/json'), r.end(JSON.stringify(v)))
  if (q.url === '/api/system') return json({ ...sys, cpu: 10 + Math.random() * 20 })
  if (q.url === '/api/servers') return json(servers.map((s) => (s.status === 'running' ? { ...s, cpu: 15 + Math.random() * 20 } : s)))
  const m = q.url.match(/^\/api\/servers\/(\w+)\/(start|stop|restart)$/)
  if (m) {
    const s = servers.find((s) => s.id === m[1])
    s.status = m[2] === 'stop' ? 'stopped' : 'running'
    r.statusCode = 204
    return r.end()
  }
  r.statusCode = 404
  r.end()
}).listen(7878, '127.0.0.1')
