import {
  brokers,
  compare,
  expectedAroundLine,
  expectedRows,
  inputsOnPage,
  listed,
  listedPlans,
  purchasePhrase,
  rowOf,
  rowOfKey,
  rowsOnPage,
  usualPlans,
} from './core'
import { away, checkbox, choice, expect, rows, test } from './fixtures'

// The results table and the summary above it: what the core computes for
// the inputs, shown right.

test(
  'the table shows what the core computes for the inputs, best first',
  { tag: '@phone' },
  async ({ page }) => {
    expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))

    await choice(page, 'Security', 'Bond').click()
    await page.getByLabel('Every month').fill('3500')
    await page.getByLabel('Buy every', { exact: true }).selectOption({ label: '3 months' })
    await checkbox(page, 'Bank Leumi').check()
    const expected = expectedRows(await inputsOnPage(page))
    await expect.poll(() => rowsOnPage(page)).toEqual(expected)
  },
)

test("the best plan is the table's first, with its warning when it needs a bigger first deposit", async ({
  page,
}) => {
  const best = async () => {
    const { plans } = compare(await inputsOnPage(page))
    const best = plans.find(({ outcome }) => outcome)!
    const label = listedPlans.find(({ key }) => JSON.stringify(key) === JSON.stringify(best.key))!.label
    return { label, warning: best.warning }
  }
  const card = page.locator('.stat.best')
  await expect(card).toContainText(`Best: ${(await best()).label}`)
  // A plan that needs a bigger first deposit is warned, in the table and,
  // if it's the best, in the summary too: it's still the cheapest.
  await page.getByLabel('One-time deposit').fill('1000')
  const warned = compare(await inputsOnPage(page)).plans.filter(({ warning }) => warning)
  expect(warned.length).toBeGreaterThan(0)
  for (const { key, warning } of warned) await expect(rowOfKey(page, key)).toContainText(`⚠ ${warning}`)
  await expect(card).toContainText(`Best: ${(await best()).label}`)
  await expect(card).not.toContainText('⚠')
  // On Tel Aviv, with nothing to start, the cheapest plans all need a deposit.
  await page.getByRole('button', { name: 'Monthly, Tel Aviv' }).click()
  const { label, warning } = await best()
  expect(warning).toBeTruthy()
  await expect(card).toContainText(`Best: ${label}`)
  await expect(card).toContainText(`⚠ ${warning}`)
})

// Worked out off the page's thread (sweeper.ts), so it follows the inputs a
// moment later; `expect` retries cover that.
test('under the best plan, where another plan becomes cheaper at other deposits, as the core says', async ({
  page,
}) => {
  const line = page.locator('.stat.best .around')
  await expect(line).toHaveText(expectedAroundLine(await inputsOnPage(page))!)
  await expect(line).toBeVisible()
  // Bonds on Tel Aviv: another plan is cheaper past a crossing.
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await choice(page, 'Security', 'Bond').click()
  const crossing = expectedAroundLine(await inputsOnPage(page))!
  expect(crossing).toMatch(/cheaper/)
  await expect(line).toHaveText(crossing)
  await expect(line).toBeVisible()
  // With nothing monthly, the one-time deposit is the one varied.
  await page.getByLabel('Every month').fill('0')
  const once = expectedAroundLine(await inputsOnPage(page))!
  expect(once).toMatch(/one-time/)
  await expect(line).toHaveText(once)
})

test("a plan's notes and flags are the core's: a track, a standing order, fees beyond the deposits, a number that may be too low", async ({
  page,
}) => {
  for (const broker of brokers()) await checkbox(page, broker.name).check()
  await choice(page, 'Security', 'Index fund').click()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await page.getByLabel('Buy every', { exact: true }).selectOption({ label: '3 months' })
  await page.getByLabel('One-time deposit').fill('0')
  await page.getByLabel('Every month').fill('50')
  const results = compare(await inputsOnPage(page)).plans
  const kinds = { note: new Set<string>(), warning: new Set<string>(), mayCostMore: new Set<string>() }
  for (const result of results) {
    const texts = (['note', 'warning', 'mayCostMore'] as const).flatMap((kind) =>
      result[kind] ? [[kind, result[kind]] as const] : [],
    )
    for (const [kind, text] of texts) {
      kinds[kind].add(text)
      await expect(rowOfKey(page, result.key), kind).toContainText(text)
    }
  }
  // The inputs were chosen to bring out each kind.
  expect(kinds.note.size).toBeGreaterThan(0)
  expect(kinds.warning.size).toBeGreaterThan(0)

  await choice(page, 'Security', 'ETF').click()
  await choice(page, 'Exchange', 'USA').click()
  await page.getByLabel('One-time deposit').fill('10000')
  await page.getByLabel('Every month').fill('2000')
  const abroad = compare(await inputsOnPage(page)).plans
  const flagged = abroad.filter(({ mayCostMore }) => mayCostMore)
  expect(flagged.length).toBeGreaterThan(0)
  for (const { key, mayCostMore } of flagged) {
    await expect(rowOfKey(page, key)).toContainText(mayCostMore!)
    // Flagged, but still ranked.
    await expect(rowOfKey(page, key).locator('.rank')).not.toHaveText('–')
  }
  for (const { key } of abroad.filter(({ mayCostMore }) => !mayCostMore)) {
    await expect(rowOfKey(page, key)).not.toContainText('May cost more')
  }
})

test('plans without a price go last, named, and say why', { tag: '@phone' }, async ({ page }) => {
  await choice(page, 'Exchange', 'Europe').click()
  const unoffered = compare(await inputsOnPage(page)).plans.filter(({ outcome }) => !outcome)
  expect(unoffered.length).toBeGreaterThan(0)
  await expect(rows(page).last()).toHaveClass(/not-offered/)
  await expect(rows(page).first()).not.toHaveClass(/not-offered/)
  await expect(rows(page).last()).toContainText(`Not offered for ${purchasePhrase('Etf', 'Europe')}`)
})

test(
  'ticking a broker adds its plans; unticking removes them, and their pins',
  { tag: '@phone' },
  async ({ page }) => {
    const leumi = brokers().find(({ name }) => name === 'Bank Leumi')!
    const usual = usualPlans.find(({ broker }) => broker.name === leumi.name)!
    await rowOf(page, usual.label).click()
    await expect(rowOf(page, usual.label)).toHaveClass(/pinned/)
    await expect(checkbox(page, leumi.name)).toHaveJSProperty('indeterminate', true)
    await checkbox(page, leumi.name).check()
    await expect(rows(page)).toHaveCount(usualPlans.length + leumi.plans.length - 1)
    await checkbox(page, leumi.name).uncheck()
    await expect(rows(page)).toHaveCount(usualPlans.length - 1)
    // Three banks have an "Online" plan: the checkbox in Leumi's list.
    await page
      .getByRole('list', { name: leumi.name })
      .getByRole('checkbox', { name: usual.plan.name, exact: true })
      .check()
    await expect(rows(page)).toHaveCount(usualPlans.length)
    await expect(rowOf(page, usual.label)).not.toHaveClass(/pinned/)
  },
)

test('every plan can be compared at once', async ({ page }) => {
  for (const broker of brokers()) await checkbox(page, broker.name).check()
  await expect(rows(page)).toHaveCount(listedPlans.length)
  expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  await choice(page, 'Chart', 'Fee breakdown').click()
  // Away from the plans, or the one under the pointer shows year by year.
  await away(page)
  const best = expectedRows(await inputsOnPage(page))[0]
  await expect(page.locator('h4', { hasText: 'Year by year' })).toContainText(best.name)
})

test('hovering a plan in the sidebar highlights it in the table, and previews it', async ({ page }) => {
  const ticked = usualPlans.find(({ broker }) => broker.name === 'Bank Leumi')!
  const inSidebar = (name: string) =>
    page
      .getByRole('list', { name: ticked.broker.name })
      .getByRole('listitem')
      .filter({ has: page.getByText(name, { exact: true }) })
  await inSidebar(ticked.plan.name).hover()
  await expect(rowOf(page, ticked.label)).toHaveClass(/highlighted/)
  const preview = page.locator('.preview')
  await expect(preview).toContainText(`For ${purchasePhrase('Etf', 'Usa')}:`)
  await expect(preview).toHaveCSS('opacity', '1') // after fading in
  await away(page)
  await expect(rowOf(page, ticked.label)).not.toHaveClass(/highlighted/)
  await expect(preview).toBeHidden()
  // Any plan previews, ticked or not.
  const { plan } = listed('Leumi · Pepper')
  await inSidebar(plan.name).hover()
  await expect(preview).toContainText(plan.description.slice(0, 20))
})

test("a plan's preview opens beside it, moved up to fit a short window", async ({ page }) => {
  // A 1366×768 laptop's, less the taskbar and the browser's bars: shorter
  // than two previews.
  await page.setViewportSize({ width: 1366, height: 600 })
  const plan = page.getByRole('list', { name: 'Bank Leumi' }).getByRole('listitem').first()
  // Just above the middle: a preview hanging down from it would run off the bottom.
  await plan.evaluate((row) => row.scrollIntoView({ block: 'center' }))
  await plan.hover()
  const preview = page.locator('.preview')
  await expect(preview).toBeInViewport({ ratio: 1 })
  const [planBox, previewBox] = [(await plan.boundingBox())!, (await preview.boundingBox())!]
  expect(previewBox.x).toBeGreaterThan(planBox.x + planBox.width)
})

// Over the list, it would hide the plans the mouse goes to next.
test('in one column there is no room beside a plan, so hovering it previews nothing', async ({ page }) => {
  // Half of a 1366×768 laptop screen.
  await page.setViewportSize({ width: 683, height: 768 })
  const ticked = usualPlans.find(({ broker }) => broker.name === 'Bank Leumi')!
  await page
    .getByRole('list', { name: ticked.broker.name })
    .getByRole('listitem')
    .filter({ has: page.getByText(ticked.plan.name, { exact: true }) })
    .hover()
  await expect(rowOf(page, ticked.label)).toHaveClass(/highlighted/)
  await expect(page.locator('.preview')).toHaveCount(0)
})

test(
  'keeping the holdings drops the tax and what is left after it; today’s money restates the amounts; the yearly fees are the core’s',
  { tag: '@phone' },
  async ({ page }) => {
    const tax = page.locator('th', { hasText: /^Tax/ })
    await expect(page.locator('th', { hasText: 'Left after tax' })).toBeVisible()
    await expect(tax).toBeVisible()
    await page.getByLabel('More options', { exact: true }).check()
    await choice(page, 'At the end', 'Keep').click()
    await expect(page.locator('th', { hasText: 'Left after tax' })).toBeHidden()
    await expect(tax).toBeHidden()
    // Nothing is taken out, so how it's taken out isn't asked.
    await expect(page.getByRole('radiogroup', { name: 'Taking the money out' })).toBeHidden()
    await page.getByRole('checkbox', { name: "Show amounts in today's money" }).check()
    await expect(page.getByText('over 20 years, in today’s money')).toBeVisible()
    const expected = expectedRows(await inputsOnPage(page))
    await expect.poll(() => rowsOnPage(page)).toEqual(expected)
    // The best plan's yearly fees are in the summary too: held, lost, yearly.
    await expect(page.locator('.stat.best .note').first()).toContainText(`· ${expected[0].amounts[2]} a year`)
  },
)

const FUND = 'Provident fund · Average fee'

test(
  'as a pension from 60 the provident fund pays no tax; how the money is taken out is asked only while it is ticked',
  { tag: '@phone' },
  async ({ page }) => {
    const wayOut = page.getByRole('radiogroup', { name: 'Taking the money out' })
    const taxOf = (label: string) => rowOf(page, label).locator('td.amount').nth(2)
    await expect(taxOf(FUND)).not.toHaveText('none')
    // Its row says why its tax is what it is; a broker's row has nothing to say.
    await expect(rowOf(page, FUND).locator('.tax-note')).toHaveText(
      'Taxed like a broker; no tax as a pension from 60',
    )
    await expect(rows(page).locator('.tax-note')).toHaveCount(1)
    // The age is asked only with the pension.
    await expect(page.getByLabel('Your age today', { exact: true })).toBeHidden()
    await wayOut.getByText('As a pension').click()
    await expect(page.getByLabel('Your age today', { exact: true })).toHaveValue('45')
    await expect(page.getByText("You'd be 65 at the end: old enough for the pension")).toBeVisible()
    await expect(taxOf(FUND)).toHaveText('none')
    await expect(rowOf(page, FUND).locator('.tax-note')).toHaveText('No tax: taken as a pension from 60')
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
    // The best plan, with nothing to pay.
    await expect(page.locator('.stat.best .label')).toHaveText(`Best: ${FUND}`)
    await expect(page.locator('.stat.best .note').first()).toHaveText('left, with no tax to pay')

    // 39 today is 59 after the 20 years: the fund is taxed like the rest.
    await page.getByLabel('Your age today', { exact: true }).fill('39')
    await expect(page.getByText("You'd be 59 at the end. The pension opens at 60")).toBeVisible()
    await expect(taxOf(FUND)).not.toHaveText('none')
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))

    // No fund ticked, nothing to ask.
    await page
      .getByRole('list', { name: 'Provident fund for investment' })
      .getByRole('checkbox', { name: 'Average fee' })
      .uncheck()
    await expect(wayOut).toBeHidden()
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  },
)

test(
  'a study fund says its tax rule and what became of its tax; kept for less than six years, that it is still locked',
  { tag: '@phone' },
  async ({ page }) => {
    const average = page
      .getByRole('list', { name: 'Study fund' })
      .getByRole('checkbox', { name: 'Average fee' })
    await expect(page.locator('aside').getByText('קרן השתלמות', { exact: true })).toBeVisible()
    // Its rule is said under its name, before it's ticked.
    await expect(
      page.locator('aside').getByText('No tax on gains after 6 years, on up to ₪20,566 deposited a year'),
    ).toBeVisible()
    await average.check()
    const row = rowOf(page, 'Study fund · Average fee')
    await expect(row).not.toHaveClass(/not-offered/)
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
    // ₪10,000 and ₪2,000 a month is more than the tax-free amount; ₪1,500
    // a month alone isn't.
    await expect(row.locator('.tax-note')).toHaveText('Taxed only on what you deposit over ₪20,566 a year')
    await page.getByLabel('One-time deposit').fill('0')
    await page.getByLabel('Every month').fill('1500')
    await expect(row.locator('.tax-note')).toHaveText('No tax: your deposits are within ₪20,566 a year')
    await expect(row.locator('td.amount').nth(2)).toHaveText('none')
    // It pays no pension, so it's the provident fund the way out is asked for.
    await expect(page.getByRole('radiogroup', { name: 'Taking the money out' })).toBeVisible()

    await page.locator('#years').fill('5')
    await expect(row).toHaveClass(/not-offered/)
    await expect(row).toContainText('Its money is still locked when your years are up')
    await row.getByRole('button', { name: 'What “The lock” means' }).click()
    await expect(
      page.getByRole('tooltip').filter({ hasText: 'only 6 years after the first deposit' }),
    ).toBeVisible()
  },
)

test(
  'deposits over a fund’s yearly ceiling leave it last, saying by how much',
  { tag: '@phone' },
  async ({ page }) => {
    await page.getByLabel('Every month').fill('8000')
    const row = rowOf(page, FUND)
    await expect(row).toHaveClass(/not-offered/)
    await expect(row).toContainText('Your deposits are over its yearly ceiling')
    await expect(rows(page).last()).toContainText('Provident fund for investment')
    await row.getByRole('button', { name: 'What “The ceiling” means' }).click()
    const { plans } = compare(await inputsOnPage(page))
    const reason = plans.at(-1)!.notOffered!
    expect(reason).toContain('No more than ₪83,641 can be deposited in a year')
    await expect(page.getByRole('tooltip').filter({ hasText: reason })).toBeVisible()
  },
)
