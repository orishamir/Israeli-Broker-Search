import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [svelte()],
  // Relative paths, so the built site works both locally and under
  // GitHub Pages' /Israeli-Broker-Search/ path.
  base: './',
})
