import {
  brokers,
  compare,
  examples,
  exchangeName,
  expectedAroundLine,
  expectedRows,
  inputsOnPage,
  listed,
  listedPlans,
  purchasePhrase,
  rowOf,
  rowOfKey,
  rowsOnPage,
  securityName,
  usualPlans,
} from './core'
import { t } from '../src/lib/text'
import {
  away,
  brokerTick,
  choice,
  expect,
  openPlans,
  planInList,
  plansList,
  rows,
  test,
  tickBrokers,
  tickPlan,
} from './fixtures'

// The results table and the summary above it: what the core computes for
// the inputs, shown right.

test(
  'the table shows what the core computes for the inputs, best first',
  { tag: '@phone' },
  async ({ page }) => {
    expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))

    await choice(page, t.security, securityName('Bond')).click()
    await page.getByLabel(t.everyMonth).fill('3500')
    await page.getByLabel(t.buyEvery, { exact: true }).selectOption('3')
    await tickBrokers(page, [listed('Leumi · Online').broker.name])
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
  await expect(card).toContainText(t.best((await best()).label))
  // A plan that needs a bigger first deposit is warned, in the table and,
  // if it's the best, in the summary too: it's still the cheapest.
  await page.getByLabel(t.oneTimeDeposit).fill('1000')
  const warned = compare(await inputsOnPage(page)).plans.filter(({ warning }) => warning)
  expect(warned.length).toBeGreaterThan(0)
  for (const { key, warning } of warned) await expect(rowOfKey(page, key)).toContainText(`⚠ ${warning}`)
  await expect(card).toContainText(t.best((await best()).label))
  await expect(card).not.toContainText('⚠')
  // On Tel Aviv, with nothing to start (saving from a salary), the cheapest
  // plans all need a deposit.
  const fromSalary = examples().find(
    ({ exchange, firstDeposit }) => exchange === 'Tlv' && firstDeposit === 0,
  )!
  await page.getByRole('button', { name: fromSalary.name, exact: true }).click()
  const { label, warning } = await best()
  expect(warning).toBeTruthy()
  await expect(card).toContainText(t.best(label))
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
  await choice(page, t.exchange, exchangeName('Tlv')).click()
  await choice(page, t.security, securityName('Bond')).click()
  const crossing = expectedAroundLine(await inputsOnPage(page))!
  expect(crossing).toMatch(/זול יותר/)
  await expect(line).toHaveText(crossing)
  await expect(line).toBeVisible()
  // With nothing monthly, the one-time deposit is the one varied.
  await page.getByLabel(t.everyMonth).fill('0')
  const once = expectedAroundLine(await inputsOnPage(page))!
  expect(once).toMatch(/חד-פעמית/)
  await expect(line).toHaveText(once)
})

test("a plan's notes and flags are the core's: a track, a standing order, fees beyond the deposits, a number that may be too low", async ({
  page,
}) => {
  await tickBrokers(
    page,
    brokers().map(({ name }) => name),
  )
  await choice(page, t.security, securityName('IndexFund')).click()
  await choice(page, t.exchange, exchangeName('Tlv')).click()
  await page.getByLabel(t.buyEvery, { exact: true }).selectOption('3')
  await page.getByLabel(t.oneTimeDeposit).fill('0')
  await page.getByLabel(t.everyMonth).fill('50')
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

  await choice(page, t.security, securityName('Etf')).click()
  await choice(page, t.exchange, exchangeName('Usa')).click()
  await page.getByLabel(t.oneTimeDeposit).fill('10000')
  await page.getByLabel(t.everyMonth).fill('2000')
  const abroad = compare(await inputsOnPage(page)).plans
  const flagged = abroad.filter(({ mayCostMore }) => mayCostMore)
  expect(flagged.length).toBeGreaterThan(0)
  for (const { key, mayCostMore } of flagged) {
    await expect(rowOfKey(page, key)).toContainText(mayCostMore!)
    // Flagged, but still ranked.
    await expect(rowOfKey(page, key).locator('.rank')).not.toHaveText('–')
  }
  for (const { key } of abroad.filter(({ mayCostMore }) => !mayCostMore)) {
    await expect(rowOfKey(page, key)).not.toContainText('עשוי לעלות יותר')
  }
})

test('plans without a price go last, named, and say why', { tag: '@phone' }, async ({ page }) => {
  await choice(page, t.exchange, exchangeName('Europe')).click()
  const unoffered = compare(await inputsOnPage(page)).plans.filter(({ outcome }) => !outcome)
  expect(unoffered.length).toBeGreaterThan(0)
  await expect(rows(page).last()).toHaveClass(/not-offered/)
  await expect(rows(page).first()).not.toHaveClass(/not-offered/)
  await expect(rows(page).last()).toContainText(t.notOfferedFor(purchasePhrase('Etf', 'Europe')))
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

test('every plan can be compared at once', async ({ page }) => {
  await tickBrokers(
    page,
    brokers().map(({ name }) => name),
  )
  await expect(rows(page)).toHaveCount(listedPlans.length)
  expect(await rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  await choice(page, t.chart, t.breakdownView).click()
  // Away from the plans, or the one under the pointer shows year by year.
  await away(page)
  const best = expectedRows(await inputsOnPage(page))[0]
  await expect(page.locator('h4', { hasText: t.yearByYear })).toContainText(best.name)
})

test('hovering a plan in the list of plans highlights it in the table, and previews it', async ({ page }) => {
  const ticked = usualPlans.find(({ broker }) => broker.englishName === 'Bank Leumi')!
  await (await planInList(page, ticked.englishLabel)).hover()
  await expect(rowOf(page, ticked.englishLabel)).toHaveClass(/highlighted/)
  const preview = page.locator('.preview')
  await expect(preview).toContainText(t.forPurchase(purchasePhrase('Etf', 'Usa')))
  await expect(preview).toHaveCSS('opacity', '1') // after fading in
  await away(page)
  await expect(rowOf(page, ticked.englishLabel)).not.toHaveClass(/highlighted/)
  await expect(preview).toBeHidden()
  // Any plan previews, ticked or not.
  const { plan } = listed('Leumi · Pepper')
  await (await planInList(page, 'Leumi · Pepper')).hover()
  await expect(preview).toContainText(plan.description.slice(0, 20))
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

test(
  'keeping the holdings drops the tax and what is left after it; today’s money restates the amounts; the yearly fees are the core’s',
  { tag: '@phone' },
  async ({ page }) => {
    const tax = page.getByRole('columnheader', { name: `${t.tax} ${t.whatMeans(t.tax)}` })
    await expect(page.locator('th', { hasText: t.leftAfterTax })).toBeVisible()
    await expect(tax).toBeVisible()
    await page.getByLabel(t.moreOptions, { exact: true }).check()
    await choice(page, t.atTheEnd, t.keep).click()
    await expect(page.locator('th', { hasText: t.leftAfterTax })).toBeHidden()
    await expect(tax).toBeHidden()
    // Nothing is taken out, so how it's taken out isn't asked.
    await expect(page.getByRole('radiogroup', { name: t.takingTheMoneyOut })).toBeHidden()
    await page.getByRole('checkbox', { name: t.todaysMoney }).check()
    await expect(page.getByText(t.overYears(20, true))).toBeVisible()
    const expected = expectedRows(await inputsOnPage(page))
    await expect.poll(() => rowsOnPage(page)).toEqual(expected)
    // The best plan's yearly fees are in the summary too: held, lost, yearly.
    const [, lost, yearly] = expected[0].amounts
    await expect(page.locator('.stat.best .note').first()).toHaveText(t.lostAndYearly(lost!, yearly!))
  },
)

const FUND = 'Provident fund · Average fee'

test(
  'as a pension from 60 the provident fund pays no tax; how the money is taken out is asked only while it is ticked',
  { tag: '@phone' },
  async ({ page }) => {
    const wayOut = page.getByRole('radiogroup', { name: t.takingTheMoneyOut })
    const taxOf = (label: string) => rowOf(page, label).locator('td.amount').nth(2)
    await expect(taxOf(FUND)).not.toHaveText(t.none)
    // Its row says why its tax is what it is; a broker's row has nothing to say.
    await expect(rowOf(page, FUND).locator('.tax-note')).toHaveText(
      'ממוסה כמו חשבון מסחר; אין מס בקצבה מגיל 60',
    )
    await expect(rows(page).locator('.tax-note')).toHaveCount(1)
    // The age is asked only with the pension.
    await expect(page.getByLabel(t.yourAge, { exact: true })).toBeHidden()
    await wayOut.getByText(t.asAPension).click()
    await expect(page.getByLabel(t.yourAge, { exact: true })).toHaveValue('45')
    await expect(page.getByText(t.oldEnough(65, 60))).toBeVisible()
    await expect(taxOf(FUND)).toHaveText(t.none)
    await expect(rowOf(page, FUND).locator('.tax-note')).toHaveText('אין מס: נמשכת כקצבה מגיל 60')
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
    // The best plan, with nothing to pay.
    await expect(page.locator('.stat.best .label')).toHaveText(t.best(listed(FUND).label))
    await expect(page.locator('.stat.best .note').first()).toHaveText(t.leftAfter(undefined))

    // 39 today is 59 after the 20 years: the fund is taxed like the rest.
    await page.getByLabel(t.yourAge, { exact: true }).fill('39')
    await expect(page.getByText(t.tooYoung(59, 60))).toBeVisible()
    await expect(taxOf(FUND)).not.toHaveText(t.none)
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))

    // No fund ticked, nothing to ask.
    await tickPlan(page, FUND, false)
    await expect(wayOut).toBeHidden()
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  },
)

test(
  'a study fund says its tax rule and what became of its tax; kept for less than six years, that it is still locked',
  { tag: '@phone' },
  async ({ page }) => {
    // Its rule is said under its name, before it's ticked.
    await openPlans(page)
    await expect(
      plansList(page).getByText('אין מס על הרווחים אחרי 6 שנים, על עד ₪20,566 שהופקדו בשנה'),
    ).toBeVisible()
    await tickPlan(page, 'Study fund · Average fee')
    const row = rowOf(page, 'Study fund · Average fee')
    await expect(row).not.toHaveClass(/not-offered/)
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
    // ₪10,000 and ₪2,000 a month is more than the tax-free amount; ₪1,500
    // a month alone isn't.
    await expect(row.locator('.tax-note')).toHaveText('המס הוא רק על מה שמופקד מעל ₪20,566 בשנה')
    await page.getByLabel(t.oneTimeDeposit).fill('0')
    await page.getByLabel(t.everyMonth).fill('1500')
    await expect(row.locator('.tax-note')).toHaveText('אין מס: ההפקדות שלכם בתוך ₪20,566 בשנה')
    await expect(row.locator('td.amount').nth(2)).toHaveText(t.none)
    // It pays no pension, so it's the provident fund the way out is asked for.
    await expect(page.getByRole('radiogroup', { name: t.takingTheMoneyOut })).toBeVisible()

    await page.locator('#years').fill('5')
    await expect(row).toHaveClass(/not-offered/)
    await expect(row).toContainText(t.stillLocked)
    await row.getByRole('button', { name: t.whatMeans(t.theLock) }).click()
    await expect(page.getByRole('tooltip').filter({ hasText: 'רק 6 שנים אחרי ההפקדה הראשונה' })).toBeVisible()
  },
)

test(
  'deposits over a fund’s yearly ceiling leave it last, saying by how much',
  { tag: '@phone' },
  async ({ page }) => {
    await page.getByLabel(t.everyMonth).fill('8000')
    const row = rowOf(page, FUND)
    await expect(row).toHaveClass(/not-offered/)
    await expect(row).toContainText(t.overTheCeiling)
    await expect(rows(page).last()).toContainText(listed(FUND).broker.name)
    await row.getByRole('button', { name: t.whatMeans(t.theCeiling) }).click()
    const { plans } = compare(await inputsOnPage(page))
    const reason = plans.at(-1)!.notOffered!
    expect(reason).toContain('אי אפשר להפקיד יותר מ-₪83,641 בשנה')
    await expect(page.getByRole('tooltip').filter({ hasText: reason })).toBeVisible()
  },
)
