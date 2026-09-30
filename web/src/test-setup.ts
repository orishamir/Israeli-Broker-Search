// Runs before each unit test file: loads the Rust core from the built
// WebAssembly (`npm run wasm`), so the modules under test call the same
// functions the browser does, in Hebrew as on the page, and stubs what jsdom
// lacks.
import { readFileSync } from 'node:fs'
import { vi } from 'vitest'
import { initSync, setLang } from './lib/core/core'

// Relative to web/, where vitest runs.
initSync({ module: readFileSync('src/lib/core/core_bg.wasm') })
setLang('He')

// jsdom has no matchMedia; nothing here asks for less motion or a touch screen.
vi.stubGlobal('matchMedia', () => ({
  matches: false,
  addEventListener: () => {},
  removeEventListener: () => {},
}))
