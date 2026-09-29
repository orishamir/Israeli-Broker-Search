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

/** Only the layout checks run on these: the extremes of screen sizes, to
 * catch layouts that break at a width the core devices don't have. Adding a
 * device here is all it takes to cover it. */
const layoutOnly = [
  { name: 'iphone-se', use: devices['iPhone SE (3rd gen)'] },
  { name: 'iphone-pro-max', use: devices['iPhone 17 Pro Max'] },
  { name: 'galaxy-a55', use: devices['Galaxy A55'] },
  { name: 'ipad-mini', use: devices['iPad Mini'] },
  { name: 'fold-open', use: devices['Galaxy Z Fold 7'] },
  { name: 'laptop', use: { ...devices['Desktop Chrome'], viewport: { width: 1024, height: 700 } } },
].map((project) => ({
  ...project,
  testMatch: 'layout.spec.ts',
  // The pictures are taken on the narrowest phone and the desktop only.
  grepInvert: project.name === 'iphone-se' ? undefined : /@pictures/,
}))

export default defineConfig({
  testDir: 'tests',
  // The performance tests have their own config, playwright.perf.config.ts.
  testIgnore: 'perf/**',
  // Screenshots and traces, for looking at what the tests saw.
  outputDir: 'test-results',
  // Baselines of `toHaveScreenshot` and `toMatchAriaSnapshot`, per device.
  snapshotPathTemplate: '{testDir}/__snapshots__/{testFilePath}/{arg}-{projectName}{ext}',
  fullyParallel: true,
  use: {
    baseURL: 'http://localhost:5173',
    // The app then skips its animations, so tests needn't wait for them.
    reducedMotion: 'reduce',
  },
  expect: {
    // Anti-aliasing differs a little between runs.
    // Stored at one pixel per CSS pixel, not the phones' 2–3x, to keep them small.
    toHaveScreenshot: { maxDiffPixelRatio: 0.002, scale: 'css' },
  },
  projects: [desktop, ...phones, ...layoutOnly],
  // Serves the app while testing; the WebAssembly must already be built
  // (`npm run wasm`).
  webServer: {
    command: 'npx vite --port 5173 --strictPort',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
  },
})
