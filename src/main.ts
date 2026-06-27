import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { useFontsStore } from './stores/fonts'
import { usePagesStore } from './stores/pages'
import { getDocument } from './lib/deviceApi'
import App from './App.vue'
import './style.css'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.mount('#app')

// Load fonts after pinia is active so @font-face rules are injected once.
// Tolerant — loadFontManifest already falls back on failure.
useFontsStore().load()

// Hydrate editor from device on startup. Tolerant — keeps local default if device is unreachable.
getDocument().then((doc) => {
  if (doc) usePagesStore().hydrate(doc)
})
