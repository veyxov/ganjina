import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// Dev-only: proxies API/blob requests to the Rust backend so the frontend can
// run on its own port (vite dev) while still hitting the real API — no CORS
// setup needed, same pattern the production build doesn't need at all since
// Axum serves both the built frontend and the API from one origin.
export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3000',
      '/blobs': 'http://127.0.0.1:3000',
    },
  },
})
