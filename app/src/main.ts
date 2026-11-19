import '@fontsource-variable/figtree'
import '@fontsource/unbounded/600.css'
import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { applyTheme, loadSettings } from './lib/settings'

applyTheme(loadSettings())

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
