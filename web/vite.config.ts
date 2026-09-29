import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig, type Plugin } from 'vite'

/** Preloads the WebAssembly. The browser otherwise asks for it only once the
 * JavaScript has downloaded and run, one after the other. */
function preloadWasm(): Plugin {
  return {
    name: 'preload-wasm',
    transformIndexHtml: {
      order: 'post',
      handler: (_html, { bundle }) => {
        const wasm = Object.keys(bundle ?? {}).find((file) => file.endsWith('.wasm'))
        return wasm
          ? [
              {
                tag: 'link',
                attrs: {
                  rel: 'preload',
                  href: `./${wasm}`,
                  as: 'fetch',
                  type: 'application/wasm',
                  crossorigin: true,
                },
                injectTo: 'head',
              },
            ]
          : []
      },
    },
  }
}

export default defineConfig({
  plugins: [svelte(), preloadWasm()],
  // Relative paths, so the built site works both locally and under
  // GitHub Pages' /Israeli-Broker-Search/ path.
  base: './',
})
