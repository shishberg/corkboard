import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { useFontsStore } from './stores/fonts'
import App from './App.vue'
import './style.css'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.mount('#app')

// Load fonts after pinia is active so @font-face rules are injected once.
// Tolerant — loadFontManifest already falls back on failure.
useFontsStore().load()
