<script lang="ts">
  import { Network, ShieldCheck, Globe, Copy, Check, ExternalLink } from 'lucide-svelte'
  import SubPages from './SubPages.svelte'
  import SaveBar from './SaveBar.svelte'
  import Confirm from './Confirm.svelte'
  import { patchServer, playit, openLink, type Conn, type Server, type Tunnel } from './api'
  import { onDestroy } from 'svelte'

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
  // playit.gg
  let linked = $state<boolean | null>(null)
  let tunnel = $state<Tunnel | null>(null)
  let linking = $state(false)
  let netBusy = $state(false)
  let netError = $state('')
  let pubCopied = $state(false)
  let askUnlink = $state(false)
  let timer: ReturnType<typeof setInterval> | undefined
  onDestroy(() => clearInterval(timer))

  async function refresh() {
    try {
      linked = (await playit.account(conn)).linked
      tunnel = linked ? await playit.get(conn, server.id) : null
      netError = ''
    } catch (e) {
      netError = (e as Error).message
    }
  }

  async function link() {
    netError = ''
    linking = true
    try {
      await openLink((await playit.link(conn)).url)
    } catch (e) {
      linking = false
      return (netError = (e as Error).message)
    }
    clearInterval(timer)
    timer = setInterval(async () => {
      const st = await playit.linkState(conn).catch(() => 'waiting' as const)
      if (st === 'waiting') return
      clearInterval(timer)
      linking = false
      if (st === 'done') refresh()
      else netError = st === 'rejected' ? 'Liaison refusée sur playit.gg.' : 'Le lien a expiré, recommence.'
    }, 2000)
  }

  async function toggle(on: boolean) {
    netBusy = true
    netError = ''
    try {
      await (on ? playit.add(conn, server.id) : playit.remove(conn, server.id))
      await refresh()
    } catch (e) {
      netError = (e as Error).message
    } finally {
      netBusy = false
    }
  }

  async function unlink() {
    netBusy = true
    try {
      await playit.unlink(conn)
      await refresh()
    } finally {
      netBusy = false
    }
  }

  // l'adresse arrive quelques secondes après la création du tunnel
  $effect(() => {
    if (page !== 'internet') return
    refresh()
    const t = setInterval(() => { if (tunnel && !tunnel.address) refresh() }, 3000)
    return () => clearInterval(t)
  })

  async function copyPublic() {
    await navigator.clipboard.writeText(tunnel!.address!)
    pubCopied = true
    setTimeout(() => (pubCopied = false), 1500)
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
    <h2>Jouer depuis Internet</h2>
    <p class="lead">Le tunnel playit.gg donne une adresse publique au serveur, sans rien ouvrir sur la box.</p>
    {#if linked === false}
      <div class="card">
        <div class="row">
          <div><strong>Compte playit.gg</strong><small>{linking ? 'Valide la liaison dans la page ouverte.' : 'Gratuit. À relier une seule fois pour tous les serveurs.'}</small></div>
          <div class="val"><button class="btn primary" onclick={link} disabled={linking}><ExternalLink size={14} /> {linking ? 'En attente…' : 'Relier un compte'}</button></div>
        </div>
      </div>
    {:else if linked}
      <div class="card">
        <label class="row">
          <div><strong>Tunnel playit.gg</strong><small>{mc ? 'Minecraft Java, TCP.' : 'Port du jeu, TCP + UDP.'}</small></div>
          <input type="checkbox" class="switch" checked={!!tunnel} disabled={netBusy} onchange={(e) => toggle(e.currentTarget.checked)} />
        </label>
        {#if tunnel}
          <div class="row">
            <div><strong>Adresse publique</strong><small>{tunnel.disabled ? `Désactivé par playit.gg : ${tunnel.disabled}` : 'À donner aux joueurs.'}</small></div>
            <div class="val">
              {#if tunnel.address}
                <span class="addr">{tunnel.address}</span><button class="btn icon" onclick={copyPublic} aria-label="Copier l'adresse publique">{#if pubCopied}<Check size={14} />{:else}<Copy size={14} />{/if}</button>
              {:else}
                <small>Préparation…</small>
              {/if}
            </div>
          </div>
        {/if}
        <div class="row">
          <div><strong>Compte playit.gg</strong><small>Relié à cette machine.</small></div>
          <div class="val"><button class="btn" onclick={() => (askUnlink = true)} disabled={netBusy}>Délier</button></div>
        </div>
      </div>
    {/if}
    {#if netError}<p class="err-text" role="alert">{netError}</p>{/if}
  {/if}
</SubPages>

<Confirm bind:open={askUnlink} title="Délier le compte playit.gg ?" text="Les adresses publiques de tous les serveurs sont supprimées." action="Délier" onconfirm={unlink} />

<SaveBar dirty={port !== saved} {busy} {error} onsave={save} onreset={() => { port = saved; error = '' }} />


<style>
  .addr { font-variant-numeric: tabular-nums; color: var(--ink); font-weight: 500; }
  .ok { display: inline-flex; align-items: center; gap: 6px; color: var(--ok); font-size: 0.85rem; }
  .ok::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
</style>
