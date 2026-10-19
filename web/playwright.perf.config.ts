import { defineConfig, devices } from '@playwright/test'

// How fast and smooth the app is: `npm run test:perf`. Separate from the
// other tests because it measures rather than checks: it needs a production
// build, animations on, one test at a time on a quiet machine, and a phone's
// CPU (throttled in the spec). The numbers are printed as a table; the
// budgets in tests/perf/speed.spec.ts fail the run when one is missed.
/** The preview's port: another worktree's run may hold the usual one. */
const port = Number(process.env.PORT ?? 4173)

export default defineConfig<{ throttle: number }>({
  testDir: 'tests/perf',
  outputDir: 'test-results/perf',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 180_000,
  reporter: [['list']],
  use: {
    baseURL: `http://localhost:${port}`,
    // Animations on: their smoothness is part of what's measured.
    reducedMotion: 'no-preference',
  },
  projects: [
    {
      name: 'desktop',
      use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 }, throttle: 1 },
    },
    // Chromium, so the CPU can be throttled: a real Galaxy is about 4× slower.
    { name: 'phone', use: { ...devices['Galaxy S24'], throttle: 4 } },
  ],
  // The built app, as users get it: the dev server serves modules one by one.
  webServer: {
    command: `npm run build && npx vite preview --port ${port} --strictPort`,
    url: `http://localhost:${port}`,
    reuseExistingServer: true,
    timeout: 300_000,
  },
})
