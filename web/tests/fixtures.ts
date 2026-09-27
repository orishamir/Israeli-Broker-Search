import { expect, test as base } from '@playwright/test'

export { expect }
export type { Page } from '@playwright/test'

/** Every test starts on the app, with fixed exchange rates instead of today's,
 * so results don't change daily. It starts once the app is ready: the page
 * loads before the WebAssembly does, and a half-drawn page is still enough
 * for a screenshot to look settled. */
export const test = base.extend({
  page: async ({ page }, use) => {
    await page.route('https://api.frankfurter.dev/**', (route) =>
      route.fulfill({ json: { date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } } }),
    )
    await page.goto('/')
    await expect(page.locator('.chart canvas')).toBeVisible()
    await expect(page.getByText('· 2026-09-25')).toBeVisible()
    await use(page)
  },
})
