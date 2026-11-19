<script lang="ts">
  import { Network, ShieldCheck, Globe, Copy, Check } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import SaveBar from './SaveBar.svelte'
  import { patchServer, type Conn, type Server } from './api'

  let { conn, server, address }: { conn: Conn; server: Server; address: string } = $props()

  let saved = $state(server.port)
  let port = $state(server.port)
  let busy = $state(false)
  let error = $state('')
  let copied = $state(false)
  const mc = server.game === 'minecraft'
  const idle = $derived(server.status === 'stopped' || server.status === 'error')
  const ports = $derived(mc ? [{ port, proto: 'TCP', what: 'Jeu' }] : [{ port: server.port, proto: 'TCP + UDP', what: 'Jeu' }, { port: server.admin_port, proto: 'TCP', what: 'txAdmin' }])

  async function save() {
    busy = true
    error = ''
    try {
      await patchServer(conn, server.id, { port })
      saved = port
    } catch (e) {
      error = (e as Error).message
    } finally {
      busy = false
    }
  }

  async function copy() {
    await navigator.clipboard.writeText(`${address}:${server.port}`)
    copied = true
    setTimeout(() => (copied = false), 1500)
  }
  const pages = [
    { id: 'address', label: 'Adresse', icon: Network },
    { id: 'firewall', label: 'Pare-feu', icon: ShieldCheck },
    { id: 'internet', label: 'Internet', icon: Globe },
  ]
  let page = $state('address')
</script>

<SubPages {pages} bind:page label="Réglages réseau">
  {#if page === 'address'}
    <h2>Adresse et port</h2>
    <div class="card">
      <div class="row">
        <div><strong>Adresse sur le réseau local</strong><small>Pour les joueurs connectés à la même box.</small></div>
        <div class="val"><span class="addr">{address}:{server.port}</span><button class="btn icon" onclick={copy} aria-label="Copier l'adresse">{#if copied}<Check size={14} />{:else}<Copy size={14} />{/if}</button></div>
      </div>
      <label class="row">
        <div>
          <strong>Port du jeu</strong>
          <small>{#if !mc}Pour FiveM, il se règle dans txAdmin.{:else if !idle}Arrête le serveur pour le changer.{:else}Entre 1024 et 65535.{/if}</small>
        </div>
        <input class="input" type="number" min="1024" max="65535" bind:value={port} disabled={!mc || !idle} />
      </label>
    </div>

  {:else if page === 'firewall'}
    <h2>Pare-feu</h2>
    <p class="lead">Foyer ouvre ces ports pour le réseau local seulement, jamais pour Internet.</p>
    <div class="card">
      {#each ports as p}
        <div class="row">
          <div><strong>{p.what}</strong><small>{p.proto}</small></div>
          <div class="val"><span class="addr">{p.port}</span><span class="ok">Ouvert</span></div>
        </div>
      {/each}
    </div>

  {:else if page === 'internet'}
    <h2>Jouer depuis Internet <span class="soon">Bientôt</span></h2>
    <div class="card">
      <label class="row">
        <div><strong>Tunnel playit.gg</strong><small>Donne une adresse publique au serveur.</small></div>
        <input type="checkbox" class="switch" disabled />
      </label>
      <div class="row">
        <div><strong>Tester la joignabilité</strong><small>Vérifie que le port répond depuis l'extérieur.</small></div>
        <div class="val"><button class="btn" disabled>Tester</button></div>
      </div>
    </div>
  {/if}
</SubPages>

<SaveBar dirty={port !== saved} {busy} {error} onsave={save} onreset={() => { port = saved; error = '' }} />


<style>
  .addr { font-variant-numeric: tabular-nums; color: var(--ink); font-weight: 500; }
  .ok { display: inline-flex; align-items: center; gap: 6px; color: var(--ok); font-size: 0.85rem; }
  .ok::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
</style>
