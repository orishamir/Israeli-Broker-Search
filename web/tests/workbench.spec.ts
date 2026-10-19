import { compareWays, expectedRows, inputsOnPage, rowsOnPage, securityName, ways } from './core'
import type { Page } from './fixtures'
import {
  answerCard,
  canvasPicture,
  choice,
  enter,
  expect,
  expectAnswerFor,
  expertSwitch,
  setExpert,
  test,
  workbench,
  away,
} from './fixtures'
import { t } from '../src/lib/text'

// The workbench ("all the options"): the switch, what is bought, the ways of
// holding the S&P 500, the full table and the charts. What the numbers are
// is the core's; the switch must change nothing but what is shown.

/** The ways ticked on the page. */
async function waysOnPage(page: Page) {
  const ticked = page.getByRole('list', { name: t.waysTitle }).getByRole('button', { pressed: true })
  const names = await ticked.allTextContents()
  return ways()
    .filter(({ name }) => names.includes(name))
    .map(({ way }) => way)
}

const money = { now: 10_000, monthly: 2000, horizon: 20 } as const

test(
  'the switch shows the workbench and changes nothing about the answer',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { ...money, horizon: 5, flexibility: 'CanWait' })
    await expectAnswerFor(page, answers)
    const before = (await answerCard(page).textContent())!
    await expect(page.locator('section.workbench')).toHaveCount(0)

    await setExpert(page)
    await expect(page.locator('section.workbench')).toBeVisible()
    await expect(page.locator('#chart')).toBeVisible()
    await expectAnswerFor(page, answers)
    await expect(answerCard(page)).toHaveText(before)
    // The questions are as they were.
    await expect(page.locator('#now')).toHaveValue('10,000')

    await setExpert(page, false)
    await expect(page.locator('section.workbench')).toHaveCount(0)
    await expect(answerCard(page)).toHaveText(before)
  },
)

test('the switch is remembered', async ({ page }) => {
  await setExpert(page)
  await page.reload()
  await expect(expertSwitch(page)).toBeChecked()
  await expect(page.locator('section.workbench')).toBeVisible()
})

test('without the switch the table shows three rows and a way to all of them', async ({ page }) => {
  await enter(page, { ...money, horizon: 10 })
  await expect(page.locator('section.within tbody tr')).toHaveCount(3)
  await page
    .locator('section.within')
    .getByRole('button', { name: /^כל \d+/ })
    .click()
  await expect(expertSwitch(page)).toBeChecked()
  await expect(page.locator('section.within tbody tr').nth(3)).toBeVisible()
})

test('the S&P 500 compares the ways, as the core ranks them', { tag: '@phone' }, async ({ page }) => {
  await enter(page, money)
  await workbench(page, t.thingSp500)
  const expected = compareWays({ inputs: await inputsOnPage(page), ways: await waysOnPage(page) })
  // A row for each way, best first; a way held by several plans folds them.
  const rows = page.locator('section.within tbody tr')
  await expect(rows).toHaveCount(expected.ways.length)
  await expect(page.locator('section.within tbody tr .names > span:first-child')).toHaveText(
    expected.ways.map(({ name }) => name),
  )
  const some = expected.ways.find(({ plans }) => plans.length > 1)!
  const row = rows.filter({ hasText: some.name })
  await row.getByRole('button', { name: t.showWayPlans(some.plans.length) }).click()
  await expect(rows).toHaveCount(expected.ways.length + some.plans.length)
})

test('a way can be left out of the comparison and put back', async ({ page }) => {
  await enter(page, money)
  await workbench(page, t.thingSp500)
  const all = await waysOnPage(page)
  const rows = page.locator('section.within tbody tr')
  await expect(rows).toHaveCount(all.length)
  const chips = page.getByRole('list', { name: t.waysTitle }).getByRole('button')
  await chips.nth(0).click()
  await expect(chips.nth(0)).toHaveAttribute('aria-pressed', 'false')
  await expect(rows).toHaveCount(all.length - 1)
  await chips.nth(0).click()
  await expect(rows).toHaveCount(all.length)
})

test("the fund's own cost is a part of the fee breakdown, with its own color", async ({ page }) => {
  await enter(page, money)
  await workbench(page, t.thingSp500)
  await choice(page, t.chart, t.breakdownView).click()
  const bars = page.locator('#chart .bars')
  await expect.poll(() => canvasPicture(bars)).toBeTruthy()
  const parts = page.getByRole('group', { name: t.compareBy })
  const product = parts.getByRole('button', { name: t.productItself })
  await expect(product).toBeVisible()
  const all = await canvasPicture(bars)
  await product.click()
  await away(page)
  await expect(product).toHaveAttribute('aria-pressed', 'true')
  await expect.poll(() => canvasPicture(bars)).not.toBe(all)
})

test('every view of the chart draws, the chart by deposit only for plans on one exchange', async ({
  page,
}) => {
  await enter(page, money)
  await workbench(page, t.thingSp500)
  const views = page.getByRole('radiogroup', { name: t.chart })
  await expect(views.getByText(t.byDepositView, { exact: true })).toHaveCount(0)
  for (const view of [t.lostView, t.valueView, t.breakdownView]) {
    await choice(page, t.chart, view).click()
    await expect(page.locator('#chart canvas:visible').first()).toBeVisible()
  }
  await workbench(page, t.thingOther)
  await expect(views.getByText(t.byDepositView, { exact: true })).toBeVisible()
  await choice(page, t.chart, t.byDepositView).click()
  await expect(page.locator('#chart .by-deposit canvas')).toBeVisible()
})

test("the full table is the core's, for the plans ticked", async ({ page }) => {
  await enter(page, money)
  await workbench(page)
  const expected = expectedRows(await inputsOnPage(page))
  await expect.poll(() => rowsOnPage(page)).toEqual(expected)
})

test('what is bought and where changes the table, and the core still agrees', async ({ page }) => {
  await enter(page, money)
  await workbench(page)
  await choice(page, t.security, securityName('Bond')).click()
  const inputs = await inputsOnPage(page)
  expect(inputs.security).toBe('Bond')
  await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(inputs))
})
