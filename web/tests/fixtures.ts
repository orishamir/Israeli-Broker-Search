import { expect, test as base, type Locator, type Page } from '@playwright/test'
import { dateText } from '../src/lib/format'
import { t } from '../src/lib/text'
import { listed, RATES } from './core'

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

/** The list of plans to compare, a dialog over the inputs, opened by "Add
 * or remove". While it's open the rest of the page can't be used. */
export const plansList = (page: Page) => page.getByRole('dialog', { name: t.whatsCompared })

/** The details dialog (`DetailsDialog`): a plan's or broker's details, the
 * editor, the page about the numbers. It may open over the list of plans. */
export const details = (page: Page) =>
  page.getByRole('dialog').filter({ hasNot: page.getByRole('heading', { name: t.whatsCompared }) })

/** Opens the list of plans, unless it's open. */
export async function openPlans(page: Page) {
  if (!(await plansList(page).isVisible())) await page.getByRole('button', { name: t.addOrRemove }).click()
  await expect(plansList(page)).toBeVisible()
}

/** Closes the list of plans with Esc, and waits until it's gone. */
export async function closePlans(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(plansList(page)).toBeHidden()
}

/** A broker's tick, for all its plans, in the list of plans, which it opens. */
export async function brokerTick(page: Page, name: string): Promise<Locator> {
  await openPlans(page)
  return plansList(page).getByRole('checkbox', { name, exact: true })
}

/** The plan called `englishLabel` in the list of plans, which it opens,
 * with the plan's broker unfolded: its tick, ✎ and ℹ are in it. */
export async function planInList(page: Page, englishLabel: string): Promise<Locator> {
  const { broker, plan } = listed(englishLabel)
  await openPlans(page)
  const line = plansList(page).getByRole('button', { name: broker.name, exact: true })
  if ((await line.getAttribute('aria-expanded')) === 'false') await line.click()
  return plansList(page)
    .getByRole('list', { name: broker.name })
    .getByRole('listitem')
    .filter({ has: page.getByRole('checkbox', { name: plan.name, exact: true }) })
}

/** Ticks, or unticks, all the plans of each broker named, in the list of
 * plans, and closes it. */
export async function tickBrokers(page: Page, names: string[], ticked = true) {
  await openPlans(page)
  for (const name of names)
    await plansList(page).getByRole('checkbox', { name, exact: true }).setChecked(ticked)
  await closePlans(page)
}

/** Ticks, or unticks, the plan called `englishLabel` in the list of plans,
 * and closes it. */
export async function tickPlan(page: Page, englishLabel: string, ticked = true) {
  await (await planInList(page, englishLabel)).getByRole('checkbox').setChecked(ticked)
  await closePlans(page)
}

/** Opens the plan called `englishLabel`'s details with its ℹ, in the list
 * of plans: over it, which stays open under them. */
export async function aboutPlan(page: Page, englishLabel: string) {
  const plan = await planInList(page, englishLabel)
  await plan.getByRole('button', { name: t.about(listed(englishLabel).label), exact: true }).click()
  await expect(details(page)).toBeVisible()
}

/** Opens a copy of the plan called `englishLabel` in the editor with its
 * ✎, in the list of plans: over it, which stays open under the editor. */
export async function copyPlan(page: Page, englishLabel: string) {
  const plan = await planInList(page, englishLabel)
  await plan.getByRole('button', { name: t.changeACopy(listed(englishLabel).label), exact: true }).click()
  await expect(details(page)).toBeVisible()
}

/** Moves the mouse to the corner, so no hover tip is open: before a
 * screenshot, and before Esc in a dialog, or Esc closes the tip instead. */
export const away = (page: Page) => page.mouse.move(0, 0)

/** Closes the open dialog (not the list of plans under it) with Esc, and
 * waits until it's gone. */
export async function closeDialog(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(details(page)).toBeHidden()
}

/** A picture of the chart's canvas, to compare before and after a change
 * without a stored baseline: zooming must change it, resetting restore it. */
export const canvasPicture = (chart: Locator) =>
  chart
    .locator('canvas')
    .first()
    .evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL())

/** Records every text drawn on a canvas from now on (what a chart draws,
 * which a picture can't read); the function returned lists them. */
export async function recordDrawnText(page: Page) {
  await page.evaluate(() => {
    const drawn: string[] = []
    Object.assign(window, { drawn })
    const fill = CanvasRenderingContext2D.prototype.fillText
    CanvasRenderingContext2D.prototype.fillText = function (text, ...rest) {
      drawn.push(text)
      fill.call(this, text, ...rest)
    }
  })
  return () => page.evaluate(() => (window as unknown as { drawn: string[] }).drawn)
}

/** Makes the screen as tall as the page, before a picture of all of it.
 * Chromium's phones lose their touch emulation while capturing past the
 * bottom of the screen, and draw the page as for a mouse ("Click" rather
 * than "Tap"). */
export async function fitScreenToPage(page: Page) {
  const { width, height } = page.viewportSize()!
  const pageHeight = await page.evaluate(() => document.documentElement.scrollHeight)
  if (pageHeight > height) await page.setViewportSize({ width, height: pageHeight })
}
