import { expect, test as base, type Page } from '@playwright/test'

export { expect, type Page }

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

/** Makes the screen as tall as the page, before a picture of all of it.
 * Chromium's phones lose their touch emulation while capturing past the
 * bottom of the screen, and draw the page as for a mouse ("Click" rather
 * than "Tap"). */
export async function fitScreenToPage(page: Page) {
  const { width, height } = page.viewportSize()!
  const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight)
  if (pageHeight > height) await page.setViewportSize({ width, height: pageHeight })
}
