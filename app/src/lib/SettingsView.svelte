<script lang="ts">
  import { LogOut, RefreshCw, Cpu, AppWindow, Plug, Palette, Bell, Archive, Keyboard, Languages, Users, Info, Globe } from 'lucide-svelte'
  import GithubMark from './GithubMark.svelte'
  import SubPages from './SubPages.svelte'
  import { applySettings, loadSettings } from './settings'
  import { bytes, type Conn, type Host } from './api'

  let { conn, sys, onlogout }: { conn: Conn; sys: Host | null; onlogout: () => void } = $props()

  // appli installée sur le serveur lui-même : se déconnecter de soi n'a pas de sens
  const local = new URL(conn.url).hostname === '127.0.0.1'
  const pages = [
    { id: 'app', label: 'Application', icon: AppWindow },
    { id: 'look', label: 'Apparence', icon: Palette },
    { id: 'notif', label: 'Notifications', icon: Bell },
    { id: 'backup', label: 'Sauvegardes', icon: Archive },
    { id: 'keys', label: 'Raccourcis', icon: Keyboard },
    { id: 'lang', label: 'Langue', icon: Languages },
    { id: 'team', label: 'Équipe', icon: Users },
    { id: 'system', label: 'Système', icon: Cpu },
    ...(local ? [] : [{ id: 'conn', label: 'Connexion', icon: Plug }]),
    { id: 'about', label: 'À propos', icon: Info },
  ]
  let page = $state('app')
  const mac = navigator.userAgent.includes('Mac')

  function since(sec = 0) {
    const d = Math.floor(sec / 86400), h = Math.floor((sec % 86400) / 3600), m = Math.floor((sec % 3600) / 60)
    return d ? `${d} j ${h} h` : h ? `${h} h ${m} min` : `${m} min`
  }
  const pct = (a: number, b: number) => (b ? Math.round((a / b) * 100) : 0)

  const inApp = '__TAURI_INTERNALS__' in window
  let s = $state(loadSettings())
  let autostart = $state(false)
  let version = $state('')
  let upd = $state<'idle' | 'checking' | 'none' | 'found' | 'installing' | 'error'>('idle')
  let newVersion = $state('')
  let install: (() => Promise<void>) | null = null

  $effect(() => {
    if (!inApp) return
    import('@tauri-apps/api/app').then(({ getVersion }) => getVersion()).then((v) => (version = v))
    import('@tauri-apps/plugin-autostart').then(({ isEnabled }) => isEnabled()).then((v) => (autostart = v))
  })

  function toggleTray() {
    s.closeToTray = !s.closeToTray
    applySettings(s)
  }

  async function toggleAutostart() {
    const a = await import('@tauri-apps/plugin-autostart')
    autostart ? await a.disable() : await a.enable()
    autostart = await a.isEnabled()
  }

  async function checkUpdate() {
    upd = 'checking'
    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const u = await check()
      if (!u) return (upd = 'none')
      newVersion = u.version
      install = async () => {
        await u.downloadAndInstall()
        const { relaunch } = await import('@tauri-apps/plugin-process')
        await relaunch()
      }
      upd = 'found'
    } catch {
      upd = 'error'
    }
  }

  async function doInstall() {
    upd = 'installing'
    try {
      await install?.()
    } catch {
      upd = 'error'
    }
  }
</script>

<header><h1>Paramètres</h1></header>

<SubPages {pages} bind:page label="Rubriques">
  {#if page === 'system'}
    <h2>Système</h2>
    {#if sys}
      <h3>Machine</h3>
      <div class="card">
        <div class="row"><div><strong>Nom</strong></div><div class="val">{sys.name || '—'}</div></div>
        <div class="row"><div><strong>Système</strong></div><div class="val">{sys.os || '—'}</div></div>
        {#if sys.kernel}<div class="row"><div><strong>Noyau / build</strong></div><div class="val num">{sys.kernel}</div></div>{/if}
        {#if sys.arch}<div class="row"><div><strong>Architecture</strong></div><div class="val">{sys.arch}</div></div>{/if}
        {#if sys.uptime != null}<div class="row"><div><strong>Allumée depuis</strong></div><div class="val num">{since(sys.uptime)}</div></div>{/if}
        <div class="row"><div><strong>Adresse de l'agent</strong></div><div class="val num">{conn.url}</div></div>
        {#if sys.agent}<div class="row"><div><strong>Version de l'agent</strong></div><div class="val num">{sys.agent}</div></div>{/if}
      </div>
      <h3>Matériel</h3>
      <div class="card">
        {#if sys.cpu_model}<div class="row"><div><strong>Processeur</strong></div><div class="val">{sys.cpu_model}</div></div>{/if}
        {#if sys.threads}<div class="row"><div><strong>Cœurs</strong></div><div class="val num">{sys.cores ?? '—'} cœurs physiques · {sys.threads} logiques</div></div>{/if}
        <div class="row"><div><strong>Charge processeur</strong></div><div class="val num">{Math.round(sys.cpu)} %</div></div>
        <div class="row"><div><strong>Mémoire</strong></div><div class="val num">{bytes(sys.mem_used)} utilisés sur {bytes(sys.mem_total)} ({pct(sys.mem_used, sys.mem_total)} %)</div></div>
        <div class="row"><div><strong>Disque des serveurs</strong></div><div class="val num">{bytes(sys.disk_free)} libres sur {bytes(sys.disk_total)}</div></div>
      </div>
    {:else}
      <p class="hint">Chargement…</p>
    {/if}

  {:else if page === 'app'}
    <h2>Application</h2>
    <h3>Fenêtre</h3>
    <div class="card">
      <label class="row">
        <div><strong>Rester en arrière-plan à la fermeture</strong><small>La croix cache la fenêtre. Pour quitter : clic droit sur l'icône près de l'horloge, puis « Quitter Foyer ».</small></div>
        <input type="checkbox" class="switch" checked={s.closeToTray} onchange={toggleTray} disabled={!inApp} />
      </label>
      <label class="row">
        <div><strong>Lancer Foyer au démarrage de l'ordinateur</strong><small>Foyer démarre caché, prêt à recevoir les alertes.</small></div>
        <input type="checkbox" class="switch" checked={autostart} onchange={toggleAutostart} disabled={!inApp} />
      </label>
    </div>
    {#if !inApp}<p class="hint">Ces réglages ne marchent que dans l'application installée.</p>{/if}
    <h3>Mises à jour</h3>
    <div class="card">
      <div class="row">
        <div>
          <strong>Version {version || '—'}</strong>
          <small>
            {#if upd === 'checking'}Recherche…
            {:else if upd === 'none'}Foyer est à jour.
            {:else if upd === 'found'}Foyer {newVersion} est disponible.
            {:else if upd === 'installing'}Installation, Foyer va redémarrer…
            {:else if upd === 'error'}Impossible de vérifier. Réessaie plus tard.
            {:else}Vérifiée au lancement puis toutes les 6 h.{/if}
          </small>
        </div>
        <div class="val">
          {#if upd === 'found' || upd === 'installing'}
            <button class="btn primary" onclick={doInstall} disabled={upd === 'installing'}>Mettre à jour</button>
          {:else}
            <button class="btn" onclick={checkUpdate} disabled={!inApp || upd === 'checking'}><RefreshCw size={14} /> Vérifier</button>
          {/if}
        </div>
      </div>
    </div>

  {:else if page === 'look'}
    <h2>Apparence</h2>
    <div class="card">
      <label class="row"><div><strong>Thème</strong></div><select class="input" bind:value={s.theme} onchange={() => applySettings(s)}><option value="dark">Sombre</option><option value="light">Clair</option><option value="system">Comme le système</option></select></label>
      <label class="row"><div><strong>Couleur d'accent</strong></div><select class="input" bind:value={s.accent} onchange={() => applySettings(s)}><option value="braise">Braise</option><option value="bleu">Bleu</option><option value="vert">Vert</option></select></label>
      <div class="row"><div><strong>Zoom</strong><small>Dès maintenant : {mac ? '⌘' : 'Ctrl'} + et {mac ? '⌘' : 'Ctrl'} − pour agrandir ou réduire, {mac ? '⌘' : 'Ctrl'} 0 pour revenir à 100 %.</small></div></div>
      <label class="row"><div><strong>Affichage compact</strong><small>Bientôt</small></div><input type="checkbox" class="switch" disabled /></label>
    </div>

  {:else if page === 'notif'}
    <h2>Notifications <span class="soon">Bientôt</span></h2>
    <div class="card">
      <label class="row"><div><strong>Serveur arrêté sur une erreur</strong></div><input type="checkbox" class="switch" checked disabled /></label>
      <label class="row"><div><strong>Disque presque plein</strong></div><input type="checkbox" class="switch" checked disabled /></label>
      <label class="row"><div><strong>Mise à jour du jeu disponible</strong></div><input type="checkbox" class="switch" disabled /></label>
      <label class="row"><div><strong>Envoyer aussi sur Discord</strong></div><input class="input" placeholder="https://discord.com/api/webhooks/…" disabled /></label>
    </div>

  {:else if page === 'backup'}
    <h2>Sauvegardes <span class="soon">Bientôt</span></h2>
    <div class="card">
      <label class="row"><div><strong>Fréquence</strong></div><select class="input" disabled><option>Chaque nuit</option><option>Toutes les 6 h</option><option>Chaque semaine</option></select></label>
      <label class="row"><div><strong>Copies gardées</strong></div><input class="input" type="number" value="7" disabled /></label>
      <label class="row"><div><strong>Dossier</strong><small>Un autre disque est plus sûr.</small></div><input class="input" placeholder="D:\Sauvegardes" disabled /></label>
    </div>

  {:else if page === 'keys'}
    <h2>Raccourcis clavier <span class="soon">Bientôt</span></h2>
    <div class="card">
      <div class="row"><div><strong>Rechercher un serveur</strong></div><div class="val"><kbd>{mac ? '⌘' : 'Ctrl'}</kbd><kbd>K</kbd></div></div>
      <div class="row"><div><strong>Nouveau serveur</strong></div><div class="val"><kbd>{mac ? '⌘' : 'Ctrl'}</kbd><kbd>N</kbd></div></div>
      <div class="row"><div><strong>Paramètres</strong></div><div class="val"><kbd>{mac ? '⌘' : 'Ctrl'}</kbd><kbd>,</kbd></div></div>
    </div>

  {:else if page === 'lang'}
    <h2>Langue <span class="soon">Bientôt</span></h2>
    <div class="card">
      <label class="row"><div><strong>Langue</strong></div><select class="input" disabled><option>Français</option><option>English</option></select></label>
    </div>

  {:else if page === 'team'}
    <h2>Équipe <span class="soon">Bientôt</span></h2>
    <div class="card">
      <div class="row"><div><strong>Inviter quelqu'un</strong><small>Il pourra démarrer et arrêter les serveurs, sans les supprimer.</small></div><div class="val"><button class="btn" disabled><Users size={14} /> Inviter</button></div></div>
    </div>

  {:else if page === 'about'}
    <h2>À propos</h2>
    <p class="lead">Foyer gère tes serveurs de jeu, chez toi. Libre et gratuit (licence MIT).</p>
    <div class="card">
      <div class="row"><div><strong>Version</strong></div><div class="val num">{version || '—'}</div></div>
      <div class="row"><div><strong>Code source</strong></div><div class="val"><a class="btn" href="https://github.com/arayan48/foyer" target="_blank" rel="noreferrer"><GithubMark size={14} /> GitHub</a></div></div>
      <div class="row"><div><strong>Site web <span class="soon">Bientôt</span></strong></div><div class="val"><button class="btn" disabled><Globe size={14} /> Site</button></div></div>
    </div>

  {:else}
    <h2>Connexion</h2>
    <div class="card">
      <div class="row">
        <div><strong>Agent</strong><small class="num">{conn.url}</small></div>
        <div class="val"><button class="btn danger" onclick={onlogout}><LogOut size={14} /> Se déconnecter</button></div>
      </div>
    </div>
  {/if}
</SubPages>

<style>
  header { margin-bottom: 20px; }
  .num { font-variant-numeric: tabular-nums; }
  .hint { color: var(--muted); font-size: 0.85rem; margin-top: 8px; }
  kbd { display: inline-grid; place-items: center; min-width: 1.6rem; height: 1.6rem; padding: 0 6px; border-radius: 6px; border: 1px solid var(--line); border-bottom-width: 2px; background: var(--surface-2); font: inherit; font-size: 0.8rem; color: var(--ink); }
  a.btn { text-decoration: none; }
</style>
