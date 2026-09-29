import {
  brokers,
  compare,
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

test('the best plan is the first that can be opened with these deposits', async ({ page }) => {
  const best = async () => {
    const { plans } = compare(await inputsOnPage(page))
    const best = plans.find(({ outcome, warning }) => outcome && !warning)!
    return listedPlans.find(({ key }) => JSON.stringify(key) === JSON.stringify(best.key))!.label
  }
  await expect(page.getByText(`Best: ${await best()}`)).toBeVisible()
  // A plan that needs a bigger first deposit is warned, and passed over.
  await page.getByLabel('One-time deposit').fill('1000')
  const warned = compare(await inputsOnPage(page)).plans.filter(({ warning }) => warning)
  expect(warned.length).toBeGreaterThan(0)
  for (const { key, warning } of warned) await expect(rowOfKey(page, key)).toContainText(`⚠ ${warning}`)
  await expect(page.getByText(`Best: ${await best()}`)).toBeVisible()
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
    await checkbox(page, usual.plan.name).check()
    await expect(rows(page)).toHaveCount(usualPlans.length)
    await expect(rowOf(page, usual.label)).not.toHaveClass(/pinned/)
  },
)

test('every plan can be compared at once', async ({ page }) => {
  for (const broker of brokers()) await checkbox(page, broker.name).check()
  await expect(rows(page)).toHaveCount(listedPlans.length)
  expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  await choice(page, 'Chart', 'Breakdown').click()
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

test(
  'keeping the holdings drops the sold column; inflation restates the amounts; the yearly cost is the core’s',
  { tag: '@phone' },
  async ({ page }) => {
    await expect(page.locator('th', { hasText: 'Yearly cost' })).toBeVisible()
    await page.getByLabel('More options', { exact: true }).check()
    await choice(page, 'At the end', 'Keep').click()
    await expect(page.locator('th', { hasText: 'Value if sold' })).toBeHidden()
    await page.getByLabel('Inflation', { exact: true }).fill('2')
    await expect(page.getByText('over 20 years, in today’s money')).toBeVisible()
    const expected = expectedRows(await inputsOnPage(page))
    await expect.poll(() => rowsOnPage(page)).toEqual(expected)
    // The best plan's yearly cost is in the summary too.
    await expect(page.locator('.stat.best .note')).toContainText(`· ${expected[0].amounts.at(-1)} a year`)
  },
)
