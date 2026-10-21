import { defineConfig, devices } from '@playwright/test'

// Which tests run where is decided by tags (see tests/CLAUDE.md): every test
// runs on the desktop unless it's touch-only (@touch), and tests that need a
// phone as well say so (@phone).

/** Every test but the touch-only ones. */
const desktop = {
  name: 'desktop',
  use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 } },
  grepInvert: /@touch/,
}

/** The tests tagged @phone or @touch: how the page is used with a finger,
 * and what differs on a small screen. */
const phones = [
  { name: 'phone', use: { ...devices['Galaxy S24'] } },
  // Safari's engine, which is where real iPhones differ.
  { name: 'iphone', use: { ...devices['iPhone 15'] } },
].map((project) => ({
  ...project,
  grep: /@phone|@touch/,
  // What needs Chrome's debugging protocol (real touch events) can't run on WebKit.
  grepInvert: project.name === 'iphone' ? /@chromium/ : undefined,
}))

/** Only the layout checks and the phone's flow run on this: the narrowest
 * phone, where layouts break first. Adding a device here is all it takes to cover it. */
const layoutOnly = [{ name: 'iphone-se', use: devices['iPhone SE (3rd gen)'] }].map((project) => ({
  ...project,
  testMatch: ['layout.spec.ts', 'phone.spec.ts'],
}))

/** The dev server's port: another worktree's tests may hold the usual one. */
const port = Number(process.env.PORT ?? 5173)

export default defineConfig({
  testDir: 'tests',
  // The performance tests have their own config, playwright.perf.config.ts.
  testIgnore: 'perf/**',
  // Screenshots and traces, for looking at what the tests saw.
  outputDir: 'test-results',
  fullyParallel: true,
  use: {
    baseURL: `http://localhost:${port}`,
    // The app then skips its animations, so tests needn't wait for them.
    reducedMotion: 'reduce',
  },
  projects: [desktop, ...phones, ...layoutOnly],
  // Serves the app while testing; the WebAssembly must already be built
  // (`npm run wasm`).
  webServer: {
    command: `npx vite --port ${port} --strictPort`,
    url: `http://localhost:${port}`,
    reuseExistingServer: true,
  },
})
