import { expect, test as base, type Locator, type Page } from '@playwright/test'
import { dateText } from '../src/lib/format'
import { RATES } from './core'

export { expect, type Locator, type Page }

/** Every test starts on the app, with fixed exchange rates instead of
 * today's, so results don't change daily. It starts once the app is ready:
 * the page loads before the WebAssembly does, and a half-drawn page is still
 * enough for a screenshot to look settled. Any error thrown in the page
 * fails the test. */
export const test = base.extend({
  page: async ({ page }, use) => {
    const errors: string[] = []
    page.on('pageerror', (error) => errors.push(error.message))
    // What the app makes of this is RATES.
    await page.route('https://api.frankfurter.dev/**', (route) =>
      route.fulfill({ json: { date: RATES.date, rates: { ILS: 3.4594, USD: 1.1403 } } }),
    )
    await page.goto('/')
    await expect(page.locator('.chart canvas')).toBeVisible()
    await expect(page.getByText(`· ${dateText(RATES.date)}`)).toBeVisible()
    await use(page)
    expect(errors, 'errors thrown in the page').toEqual([])
  },
})

/** The results table's rows, best first. */
export const rows = (page: Page) => page.locator('tbody tr')

/** The row of the plan called `name` (its plan name, not its broker's). */
export const row = (page: Page, name: string) =>
  rows(page).filter({ has: page.getByText(name, { exact: true }) })

/** A security, exchange, chart or editor view button, by its group and name. */
export const choice = (within: Page | Locator, group: string, name: string) =>
  within.getByRole('radiogroup', { name: group }).getByText(name, { exact: true })

/** A broker's or plan's tick in the sidebar. */
export const checkbox = (page: Page, name: string) => page.getByRole('checkbox', { name, exact: true })

/** Moves the mouse to the corner, so no hover tip is open: before a
 * screenshot, and before Esc in a dialog, or Esc closes the tip instead. */
export const away = (page: Page) => page.mouse.move(0, 0)

/** Closes the open dialog with Esc, and waits until it's gone. */
export async function closeDialog(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).toBeHidden()
}

/** A picture of the chart's canvas, to compare before and after a change
 * without a stored baseline: zooming must change it, resetting restore it. */
export const canvasPicture = (chart: Locator) =>
  chart
    .locator('canvas')
    .first()
    .evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL())

/** Makes the screen as tall as the page, before a picture of all of it.
 * Chromium's phones lose their touch emulation while capturing past the
 * bottom of the screen, and draw the page as for a mouse ("Click" rather
 * than "Tap"). */
export async function fitScreenToPage(page: Page) {
  const { width, height } = page.viewportSize()!
  const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight)
  if (pageHeight > height) await page.setViewportSize({ width, height: pageHeight })
}
