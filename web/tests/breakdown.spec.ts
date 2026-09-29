import { nameWidth } from '../src/lib/fee-breakdown'
import { brokers, compare, expectedRows, inputsOnPage, listedPlans, purchasePhrase, rowOfKey } from './core'
import { away, canvasPicture, checkbox, choice, expect, test, type Page } from './fixtures'

// The fee breakdown: comparing by one fee, and picking a plan to see year by
// year. The bars' options are checked in src/lib/fee-breakdown.test.ts.

test.beforeEach(async ({ page }) => {
  await choice(page, 'Chart', 'Breakdown').click()
  await away(page)
})

const overTime = (page: Page) => page.locator('h4', { hasText: 'Year by year' })

/** The compared plans by their fees, least first: the order of the bars. */
async function byFees(page: Page) {
  const { plans } = compare(await inputsOnPage(page))
  return plans
    .filter(({ outcome }) => outcome)
    .toSorted((a, b) => a.outcome!.fees.total - b.outcome!.fees.total)
    .map(({ key }) => listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!)
}

test(
  'shows the best plan year by year at first, and compares by one fee on request',
  { tag: '@phone' },
  async ({ page }) => {
    const best = expectedRows(await inputsOnPage(page))[0]
    await expect(overTime(page)).toContainText(best.name)
    const bars = page.locator('.bars')
    const all = await canvasPicture(bars)
    const custody = page
      .getByRole('group', { name: 'Compare the plans by' })
      .getByRole('button', { name: 'Custody' })
    await custody.click()
    await away(page)
    await expect(custody).toHaveAttribute('aria-pressed', 'true')
    await expect.poll(() => canvasPicture(bars)).not.toBe(all)
    await custody.click()
    await expect(custody).toHaveAttribute('aria-pressed', 'false')
  },
)

/** Where the second bar is (fewest fees first), just right of the names. */
async function secondBar(page: Page) {
  // In view, or the click lands outside the screen.
  await page.locator('.bars').scrollIntoViewIfNeeded()
  const box = (await page.locator('.bars').boundingBox())!
  const narrow = page.viewportSize()!.width < 560
  return { x: box.x + (narrow ? 104 : 120) + 24, y: box.y + 4 + (narrow ? 40 : 34.5) * 1.5 }
}

test('clicking a bar pins its plan and shows it over time', async ({ page }) => {
  const second = (await byFees(page))[1]
  const { x, y } = await secondBar(page)
  await page.mouse.click(x, y)
  await away(page)
  await expect(rowOfKey(page, second.key)).toHaveClass(/pinned/)
  await expect(overTime(page)).toContainText(second.label)
})

test('tapping a bar pins its plan and shows it over time', { tag: '@touch' }, async ({ page }) => {
  const second = (await byFees(page))[1]
  const { x, y } = await secondBar(page)
  await page.touchscreen.tap(x, y)
  await expect(rowOfKey(page, second.key)).toHaveClass(/pinned/)
  await expect(overTime(page)).toContainText(second.label)
})

test(
  "scrolling the page with a finger on the chart doesn't pick a plan",
  { tag: ['@touch', '@chromium'] },
  async ({ page }) => {
    const best = expectedRows(await inputsOnPage(page))[0]
    await page.locator('.bars').scrollIntoViewIfNeeded()
    const box = (await page.locator('.bars').boundingBox())!
    const [x, y] = [box.x + box.width / 2, box.y + box.height / 2]
    // Real touch events: `tap()` can't scroll.
    const cdp = await page.context().newCDPSession(page)
    const touch = (type: string, touchPoints: { x: number; y: number }[]) =>
      cdp.send('Input.dispatchTouchEvent', { type: type as 'touchStart', touchPoints })
    await touch('touchStart', [{ x, y }])
    for (let step = 1; step <= 8; step++) await touch('touchMove', [{ x, y: y - step * 25 }])
    await touch('touchEnd', [])
    await expect(overTime(page)).toContainText(best.name)
    await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
  },
)

test('plans without a price are named, not just left out', { tag: '@phone' }, async ({ page }) => {
  await choice(page, 'Exchange', 'Europe').click()
  const unoffered = compare(await inputsOnPage(page))
    .plans.filter(({ outcome }) => !outcome)
    .map(({ key }) => listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.label)
  expect(unoffered.length).toBeGreaterThan(0)
  const hint = page.getByText(`Not offered for ${purchasePhrase('Etf', 'Europe')}:`)
  for (const label of unoffered) await expect(hint).toContainText(label)
})

// A name column too narrow for a word breaks the word ("Excellenc/e"): every
// word of every plan's name must fit a line of the column, in the page's font
// at ECharts' size, on this screen's column width.
test("every word of a plan's name fits the bars' name column", { tag: '@phone' }, async ({ page }) => {
  for (const broker of brokers()) await checkbox(page, broker.name).check()
  const room = nameWidth(page.viewportSize()!.width < 560) - 2 * 5
  const words = [...new Set(listedPlans.flatMap(({ label }) => label.split(' ')))]
  const widths = await page.evaluate((words) => {
    const context = document.createElement('canvas').getContext('2d')!
    context.font = `12px ${getComputedStyle(document.documentElement).fontFamily}`
    return words.map((word) => context.measureText(word).width)
  }, words)
  const tooWide = words.filter((_, index) => widths[index] > room)
  expect(tooWide, `wider than the ${room}px column`).toEqual([])
})
