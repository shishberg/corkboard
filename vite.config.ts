import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  server: { allowedHosts: ["kodama.local"] },
  build: {
    rollupOptions: {
      // Two entry points: the editor (index.html) and the status dashboard
      // (dashboard.html), served by the device at `/` and `/dashboard`.
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        dashboard: fileURLToPath(new URL('./dashboard.html', import.meta.url)),
      },
      // Silence Rolldown's INVALID_ANNOTATION noise from @vueuse/core's prebuilt
      // bundle (misplaced `/* #__PURE__ */` comments — a third-party artifact,
      // nothing in our code). Let every other warning through.
      onLog(level, log, handler) {
        if (log.code === 'INVALID_ANNOTATION' && log.message?.includes('@vueuse/core')) return
        handler(level, log)
      },
    },
  },
})
