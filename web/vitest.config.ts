import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vitest/config'

// Unit tests of the TypeScript side: `npm run test:unit`. They run in Node
// with a browser-like document (jsdom) and the real Rust core, loaded once by
// src/test-setup.ts. What needs a real browser is Playwright's, in tests/.
export default defineConfig({
  plugins: [svelte()],
  // Svelte's browser build, with working runes; the default under Node is
  // its server build, where effects never run.
  resolve: { conditions: ['browser'] },
  test: {
    // A test of a module with runes is itself a `.svelte.ts` module.
    include: ['src/**/*.test.ts', 'src/**/*.test.svelte.ts'],
    environment: 'jsdom',
    setupFiles: ['src/test-setup.ts'],
    coverage: {
      include: ['src/**/*.ts', 'src/**/*.svelte'],
      exclude: ['src/lib/core/**', 'src/**/*.test.ts'],
    },
  },
})
