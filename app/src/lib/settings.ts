// Réglages de l'appli, gardés sur ce PC.
export interface Settings {
  closeToTray: boolean
  theme: 'dark' | 'light' | 'system'
  accent: 'braise' | 'bleu' | 'vert'
}

const KEY = 'foyer.settings'
const DEFAULTS: Settings = { closeToTray: true, theme: 'dark', accent: 'braise' }

export function loadSettings(): Settings {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) ?? '{}') }
  } catch {
    return { ...DEFAULTS }
  }
}

const sysLight = matchMedia('(prefers-color-scheme: light)')

// Pose thème et accent sur <html> ; les jetons de app.css font le reste.
export function applyTheme(s: Settings) {
  const light = s.theme === 'light' || (s.theme === 'system' && sysLight.matches)
  document.documentElement.dataset.theme = light ? 'light' : 'dark'
  document.documentElement.dataset.accent = s.accent
  // barre de titre de la fenêtre : suit le thème choisi (null = système)
  import('@tauri-apps/api/window')
    .then(({ getCurrentWindow }) => getCurrentWindow().setTheme(s.theme === 'system' ? null : s.theme))
    .catch(() => {}) // hors Tauri
}
sysLight.addEventListener('change', () => applyTheme(loadSettings()))

export async function applySettings(s: Settings) {
  applyTheme(s)
  try {
    localStorage.setItem(KEY, JSON.stringify(s))
  } catch {}
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('set_close_to_tray', { on: s.closeToTray })
  } catch {} // hors Tauri
}
