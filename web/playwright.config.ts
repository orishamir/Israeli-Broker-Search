import { defineConfig, devices } from '@playwright/test'

/** Every test runs on these. */
const core = [
  { name: 'desktop', use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 } } },
  { name: 'phone', use: { ...devices['Galaxy S24'] } },
  // Safari's engine, which is where real iPhones differ.
  { name: 'iphone', use: { ...devices['iPhone 15'] } },
]

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
].map((project) => ({ ...project, testMatch: 'layout.spec.ts' }))

export default defineConfig({
  testDir: 'tests',
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
  projects: [...core, ...layoutOnly],
  // Serves the app while testing; the WebAssembly must already be built
  // (`npm run wasm`).
  webServer: {
    command: 'npx vite --port 5173 --strictPort',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
  },
})
