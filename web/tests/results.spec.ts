import {
  compare,
  examples,
  expectedRows,
  inputsOnPage,
  listed,
  listedPlans,
  rowOf,
  rowsOnPage,
  usualPlans,
} from './core'
import { t } from '../src/lib/text'
import { away, brokerTick, choice, expect, planInList, rows, test, tickPlan } from './fixtures'

// The results table and the summary above it, as they respond to the user.
// What they say is the core's, tested in Rust.

test('the table shows what the core computes for the inputs', { tag: '@phone' }, async ({ page }) => {
  expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
})

// A line of its own would change the card's height and move the table.
test("when no plan can be opened, the best one's warning is a sign that says it on hover", async ({
  page,
}) => {
  const fromSalary = examples().find(
    ({ exchange, firstDeposit }) => exchange === 'Tlv' && firstDeposit === 0,
  )!
  await page.getByRole('button', { name: fromSalary.name, exact: true }).click()
  const ibi = listed('IBI · Typical offer')
  const ticked = (await inputsOnPage(page)).plans.map(
    (key) => listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.englishLabel,
  )
  for (const englishLabel of ticked.filter((label) => label !== ibi.englishLabel))
    await tickPlan(page, englishLabel, false)
  const { warning } = compare(await inputsOnPage(page)).plans[0]
  const card = page.locator('.stat.best')
  await card.getByRole('button', { name: `⚠ ${warning}`, exact: true }).hover()
  await expect(card.getByRole('tooltip')).toHaveText(warning!)
})

test(
  'ticking a broker adds its plans; unticking removes them, and their pins',
  { tag: '@phone' },
  async ({ page }) => {
    const leumi = listed('Leumi · Online').broker
    const usual = usualPlans.find(({ broker }) => broker.name === leumi.name)!
    await rowOf(page, usual.englishLabel).click()
    await expect(rowOf(page, usual.englishLabel)).toHaveClass(/pinned/)
    const all = await brokerTick(page, leumi.name)
    await expect(all).toHaveJSProperty('indeterminate', true)
    await all.check()
    await expect(rows(page)).toHaveCount(usualPlans.length + leumi.plans.length - 1)
    await all.uncheck()
    await expect(rows(page)).toHaveCount(usualPlans.length - 1)
    await (await planInList(page, usual.englishLabel)).getByRole('checkbox').check()
    await expect(rows(page)).toHaveCount(usualPlans.length)
    await expect(rowOf(page, usual.englishLabel)).not.toHaveClass(/pinned/)
  },
)

test('hovering a plan in the list of plans highlights it in the table, and previews it', async ({ page }) => {
  const ticked = usualPlans.find(({ broker }) => broker.englishName === 'Bank Leumi')!
  await (await planInList(page, ticked.englishLabel)).hover()
  await expect(rowOf(page, ticked.englishLabel)).toHaveClass(/highlighted/)
  const preview = page.locator('.preview')
  await expect(preview).toHaveCSS('opacity', '1') // after fading in
  await away(page)
  await expect(rowOf(page, ticked.englishLabel)).not.toHaveClass(/highlighted/)
  await expect(preview).toBeHidden()
  // Any plan previews, ticked or not.
  await (await planInList(page, 'Leumi · Pepper')).hover()
  await expect(preview).toBeVisible()
})

// Beside the plan, on the side of the results: left of it, on this
// right-to-left page.
test("a plan's preview opens left of it, moved up to fit a short window", async ({ page }) => {
  // A 1366×768 laptop's, less the taskbar and the browser's bars: shorter
  // than two previews.
  await page.setViewportSize({ width: 1366, height: 600 })
  const plan = await planInList(page, 'Leumi · Online')
  // Just above the middle: a preview hanging down from it would run off the bottom.
  await plan.evaluate((row) => row.scrollIntoView({ block: 'center' }))
  await plan.hover()
  const preview = page.locator('.preview')
  await expect(preview).toBeInViewport({ ratio: 1 })
  const [planBox, previewBox] = [(await plan.boundingBox())!, (await preview.boundingBox())!]
  expect(previewBox.x + previewBox.width).toBeLessThan(planBox.x)
})

// Over the list, it would hide the plans the mouse goes to next.
test('in one column there is no room beside a plan, so hovering it previews nothing', async ({ page }) => {
  // Half of a 1366×768 laptop screen.
  await page.setViewportSize({ width: 683, height: 768 })
  const ticked = usualPlans.find(({ broker }) => broker.englishName === 'Bank Leumi')!
  await (await planInList(page, ticked.englishLabel)).hover()
  await expect(rowOf(page, ticked.englishLabel)).toHaveClass(/highlighted/)
  await expect(page.locator('.preview')).toHaveCount(0)
})

test('keeping the holdings drops the tax columns, and asks nothing about taking the money out', async ({
  page,
}) => {
  const tax = page.getByRole('columnheader', { name: `${t.tax} ${t.whatMeans(t.tax)}` })
  await expect(tax).toBeVisible()
  await page.getByLabel(t.moreOptions, { exact: true }).check()
  await choice(page, t.atTheEnd, t.keep).click()
  await expect(page.locator('th', { hasText: t.leftAfterTax })).toBeHidden()
  await expect(tax).toBeHidden()
  await expect(page.getByRole('radiogroup', { name: t.takingTheMoneyOut })).toBeHidden()
})

test('the age is asked only for a pension, and only while a fund that pays one is ticked', async ({
  page,
}) => {
  const wayOut = page.getByRole('radiogroup', { name: t.takingTheMoneyOut })
  await expect(page.getByLabel(t.yourAge, { exact: true })).toBeHidden()
  await wayOut.getByText(t.asAPension).click()
  await expect(page.getByLabel(t.yourAge, { exact: true })).toHaveValue('45')
  await tickPlan(page, 'Provident fund · Average fee', false)
  await expect(wayOut).toBeHidden()
})
