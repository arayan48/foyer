// Réglages de l'appli, gardés sur ce PC.
export interface Settings {
  closeToTray: boolean
}

const KEY = 'foyer.settings'
const DEFAULTS: Settings = { closeToTray: true }

export function loadSettings(): Settings {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) ?? '{}') }
  } catch {
    return { ...DEFAULTS }
  }
}

export async function applySettings(s: Settings) {
  try {
    localStorage.setItem(KEY, JSON.stringify(s))
  } catch {}
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('set_close_to_tray', { on: s.closeToTray })
  } catch {} // hors Tauri
}
