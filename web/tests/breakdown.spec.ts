import { barY, nameY, NARROW_SCREEN } from '../src/lib/fee-breakdown'
import {
  brokers,
  compare,
  exchangeName,
  expectedRows,
  feeKinds,
  inputsOnPage,
  listedPlans,
  purchasePhrase,
  rowOfKey,
} from './core'
import {
  away,
  canvasPicture,
  choice,
  expect,
  recordDrawnText,
  test,
  tickBrokers,
  type Page,
} from './fixtures'
import { t } from '../src/lib/text'

// The fee breakdown: comparing by one fee, and picking a plan to see year by
// year. The bars' options are checked in src/lib/fee-breakdown.test.ts.

test.beforeEach(async ({ page }) => {
  await choice(page, t.chart, t.breakdownView).click()
  await away(page)
})

const overTime = (page: Page) => page.locator('h4', { hasText: t.yearByYear })

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
      .getByRole('group', { name: t.compareBy })
      .getByRole('button', { name: feeKinds().find(({ value }) => value === 'Account')!.name })
    await custody.click()
    await away(page)
    await expect(custody).toHaveAttribute('aria-pressed', 'true')
    await expect.poll(() => canvasPicture(bars)).not.toBe(all)
    await custody.click()
    await expect(custody).toHaveAttribute('aria-pressed', 'false')
  },
)

/** Places on the second row (fewest fees first): its name, its bar a little
 * after it starts, the room beside the bar, and its total. */
async function secondRow(page: Page) {
  // In view, or the click lands outside the screen.
  await page.locator('.bars').scrollIntoViewIfNeeded()
  const box = (await page.locator('.bars').boundingBox())!
  const narrow = await page.evaluate((query) => matchMedia(query).matches, NARROW_SCREEN)
  const [name, bar] = [box.y + nameY(1, narrow), box.y + barY(1, narrow)]
  return {
    name: { x: box.x + 30, y: name },
    bar: { x: box.x + 24, y: bar },
    beside: { x: box.x + box.width * 0.8, y: bar },
    total: { x: box.x + box.width - 20, y: bar },
  }
}

test('clicking a bar pins its plan and shows it over time', async ({ page }) => {
  const second = (await byFees(page))[1]
  const { bar } = await secondRow(page)
  await page.mouse.click(bar.x, bar.y)
  await away(page)
  await expect(rowOfKey(page, second.key)).toHaveClass(/pinned/)
  await expect(overTime(page)).toContainText(second.label)
})

test('tapping a bar pins its plan and shows it over time', { tag: '@touch' }, async ({ page }) => {
  const second = (await byFees(page))[1]
  const { bar } = await secondRow(page)
  await page.touchscreen.tap(bar.x, bar.y)
  await expect(rowOfKey(page, second.key)).toHaveClass(/pinned/)
  await expect(overTime(page)).toContainText(second.label)
})

/** Presses each part of the second row twice: once pins its plan, and the
 * second time unpins it. The whole row is the plan's, as its outline shows. */
async function pressEachPart(page: Page, press: (x: number, y: number) => Promise<void>) {
  const row = rowOfKey(page, (await byFees(page))[1].key)
  for (const [part, { x, y }] of Object.entries(await secondRow(page))) {
    await press(x, y)
    await expect(row, `pinned from its ${part}`).toHaveClass(/pinned/)
    await press(x, y)
    await expect(row, `unpinned from its ${part}`).not.toHaveClass(/pinned/)
  }
}

test('clicking anywhere on a row pins its plan, and again unpins it', async ({ page }) => {
  await pressEachPart(page, (x, y) => page.mouse.click(x, y))
})

test('tapping anywhere on a row pins its plan, and again unpins it', { tag: '@touch' }, async ({ page }) => {
  await pressEachPart(page, (x, y) => page.touchscreen.tap(x, y))
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
  await choice(page, t.exchange, exchangeName('Europe')).click()
  const unoffered = compare(await inputsOnPage(page))
    .plans.filter(({ outcome }) => !outcome)
    .map(({ key }) => listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.label)
  expect(unoffered.length).toBeGreaterThan(0)
  const hint = page.getByText(`${t.notOfferedFor(purchasePhrase('Etf', 'Europe'))}:`)
  for (const label of unoffered) await expect(hint).toContainText(label)
})

// Each name is drawn whole, on its line above the bar: none is cut to fit
// the box (see `fitName`), on this screen and in the page's font.
test("every plan's name is drawn whole, on one line", { tag: '@phone' }, async ({ page }) => {
  const drawn = await recordDrawnText(page)
  await tickBrokers(
    page,
    brokers().map(({ name }) => name),
  )
  const names = (await byFees(page)).map(({ label }) => label)
  await expect.poll(drawn).toEqual(expect.arrayContaining(names))
})
