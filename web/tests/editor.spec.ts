import { formatNumber } from '../src/lib/numbers'
import {
  copyOf,
  exchangeName,
  feeKinds,
  inputsOnPage,
  listed,
  listedPlan,
  planInfo,
  purchasePhrase,
  securityName,
  setManagement,
  setTrade,
  simpleFees,
  trackOnPage,
} from './core'
import type { Exchange, FeeKind, PlanData, Security } from './core'
import { choice, expect, rows, test, type Page } from './fixtures'
import { t } from '../src/lib/text'

// Your plans: copying a listed plan to change, plans of your own, and the
// editor's two views. Expected values come from the core's editor functions.

const card = (page: Page) => page.locator('section.card', { hasText: t.yourPlans })
const editor = (page: Page) => page.getByRole('dialog')

/** A fee's name, as its fields are labelled: "קנייה או מכירה" in
 * "קנייה או מכירה: price". */
const fee = (kind: FeeKind) => feeKinds().find(({ value }) => value === kind)!.name
const TRADE = fee('Trade')
const NEGATIVE = 'עמלות לא יכולות להיות שליליות'

/** A copy of the plan called `englishLabel`, as the app makes one for the
 * page's inputs: on the track the comparison picked. */
async function copied(page: Page, englishLabel: string) {
  const { key, plan, broker } = listed(englishLabel)
  const inputs = await inputsOnPage(page)
  const track = trackOnPage(inputs, key)
  const copy = copyOf(key.broker, key.plan, track)
  const original = listedPlan(key.broker, key.plan, track)
  const simple = (data: PlanData = copy) => simpleFees(data, inputs.security, inputs.exchange, original)
  return { plan, broker, copy, original, inputs, simple }
}

/** Opens a draft copy of a listed plan with its ✎: "Leumi · Pepper". */
async function copy(page: Page, englishLabel: string) {
  await page.getByRole('button', { name: t.changeACopy(listed(englishLabel).label), exact: true }).click()
  await expect(editor(page)).toBeVisible()
}

/** Copies Pepper, charges $2 an order instead of its price, and adds it. */
async function addDeal(page: Page) {
  await copy(page, 'Leumi · Pepper')
  await editor(page).getByLabel(`${TRADE}: price`).fill('2')
  await editor(page).getByRole('button', { name: t.addPlan }).click()
  await expect(editor(page)).toBeHidden()
}

const price = (fields: { amount?: number | null }) => formatNumber(fields.amount ?? null)

test("✎ copies a plan to change, showing the original's fees", { tag: '@phone' }, async ({ page }) => {
  await expect(card(page)).toContainText(t.thinkLowerFees)
  const { plan, broker, copy: data, inputs, simple } = await copied(page, 'Leumi · Pepper')
  await copy(page, 'Leumi · Pepper')
  const dialog = page.getByRole('dialog', { name: planInfo(data).name })
  await expect(dialog).toContainText(t.copyOf(plan.name, broker.name))
  const trade = simple().trade!
  await expect(dialog.getByLabel(`${TRADE}: price`)).toHaveValue(price(trade.fields))

  await dialog.getByLabel(`${TRADE}: price`).fill('2')
  const changed = setTrade(data, inputs.security, inputs.exchange, { ...trade.fields, amount: 2 })
  const was = simple(changed).trade!.was!
  await expect(dialog).toContainText(`${plan.name}: ${was.price.text}`)

  await dialog.getByRole('button', { name: t.addPlan }).click()
  await expect(dialog).toBeHidden()
  await expect(card(page)).toContainText(t.copyOf(plan.name, broker.name))
  const row = rows(page).filter({ hasText: planInfo(data).name })
  await expect(row).toContainText(t.yourDeal(broker.name))
  await expect(row.locator('.mark')).toHaveClass(/yours/)
  await expect(rows(page)).toHaveCount((await inputsOnPage(page)).plans.length + 1)
})

test(
  "a fund's copy has its manager's fee alone, and a lower one ranks above the average",
  { tag: '@phone' },
  async ({ page }) => {
    const label = 'Provident fund · Average fee'
    const { plan, broker, copy: data, simple } = await copied(page, label)
    const yours = planInfo(data).name
    await copy(page, label)
    const dialog = page.getByRole('dialog', { name: yours })
    await expect(dialog).toContainText(t.copyOf(plan.name, broker.name))
    // No price list to open, and no trades or conversions to price.
    await expect(dialog.getByRole('radiogroup', { name: t.view })).toBeHidden()
    await expect(dialog.getByLabel(`${TRADE}: price`)).toBeHidden()
    const balance = dialog.getByLabel('Management fee: of the balance, a year')
    const fee = simple().management!
    await expect(balance).toHaveValue(formatNumber(fee.fields.ofBalance ?? null))
    await expect(dialog.getByLabel('Management fee: of each deposit')).toHaveValue('')

    await balance.fill('0.4')
    const changed = setManagement(data, { ...fee.fields, ofBalance: 0.4 })
    await expect(dialog).toContainText(`${plan.name}: ${simple(changed).management!.was!.price.text}`)
    await balance.fill('-1')
    await expect(dialog).toContainText(NEGATIVE)
    await balance.fill('0.4')
    await expect(dialog).not.toContainText(NEGATIVE)

    await dialog.getByRole('button', { name: t.addPlan }).click()
    await expect(dialog).toBeHidden()
    const row = rows(page).filter({ hasText: yours })
    await expect(row).toContainText(t.yourDeal(broker.name))
    const names = await rows(page).locator('.names > span:first-child').allTextContents()
    expect(names.indexOf(yours)).toBeGreaterThanOrEqual(0)
    expect(names.indexOf(yours)).toBeLessThan(names.indexOf(plan.name))
  },
)

test('↺ goes back to the original fee; a bad fee says why and keeps the last good one', async ({ page }) => {
  const { plan, simple } = await copied(page, 'Leumi · Pepper')
  await copy(page, 'Leumi · Pepper')
  const field = editor(page).getByLabel(`${TRADE}: price`)
  await field.fill('2')
  await editor(page)
    .getByRole('button', { name: t.backTo(plan.name) })
    .click()
  await expect(field).toHaveValue(price(simple().trade!.fields))
  await expect(editor(page)).not.toContainText(`${plan.name}:`)
  await field.fill('-3')
  await expect(editor(page)).toContainText(NEGATIVE)
  // Still the original's fee, so nothing to go back to.
  await expect(editor(page)).not.toContainText(`${plan.name}:`)
})

test(
  'Cancel adds nothing; deleting asks first, then unticks and unpins',
  { tag: '@phone' },
  async ({ page }) => {
    const before = await rows(page).count()
    await copy(page, 'Leumi · Pepper')
    await editor(page).getByRole('button', { name: t.cancel }).click()
    await expect(editor(page)).toBeHidden()
    await expect(card(page)).toContainText(t.thinkLowerFees)
    await expect(rows(page)).toHaveCount(before)

    await addDeal(page)
    const { copy: data } = await copied(page, 'Leumi · Pepper')
    const name = planInfo(data).name
    await rows(page).filter({ hasText: name }).click()
    await expect(page.locator('tbody tr.pinned')).toHaveCount(1)
    await page.getByRole('button', { name: t.change(name) }).click()
    await editor(page).getByRole('button', { name: t.deletePlan }).click()
    await expect(editor(page)).toContainText(t.deleteQuestion(name))
    await editor(page).getByRole('button', { name: t.keepPlan, exact: true }).click()
    await editor(page).getByRole('button', { name: t.deletePlan }).click()
    await editor(page).getByRole('button', { name: t.delete, exact: true }).click()
    await expect(editor(page)).toBeHidden()
    await expect(rows(page)).toHaveCount(before)
    await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
    await expect(card(page)).toContainText(t.thinkLowerFees)
  },
)

test('your plans and the editor view stay after a reload', async ({ page }) => {
  const { plan, broker, copy: data, inputs, simple } = await copied(page, 'Leumi · Pepper')
  const name = planInfo(data).name
  await addDeal(page)
  await page.getByRole('button', { name: t.change(name) }).click()
  await choice(editor(page), t.view, t.fullPriceList).click()
  await editor(page).getByRole('button', { name: t.done }).click()

  await page.reload()
  await expect(rows(page).filter({ hasText: name })).toContainText(t.yourDeal(broker.name))
  await page.getByRole('button', { name: t.change(name) }).click()
  await expect(editor(page).getByRole('radio', { name: t.fullPriceList })).toBeChecked()
  // Still $2, not the original's price.
  const trade = simple().trade!
  const changed = setTrade(data, inputs.security, inputs.exchange, { ...trade.fields, amount: 2 })
  await expect(editor(page)).toContainText(`${plan.name}: ${simple(changed).trade!.was!.price.text}`)
})

test('a new plan, with a broker name, then the next free name', async ({ page }) => {
  await card(page).getByRole('button', { name: t.newPlan }).click()
  const dialog = page.getByRole('dialog', { name: t.yourPlan })
  await dialog.getByPlaceholder(t.optional).fill('IBI')
  await dialog.getByLabel(`${TRADE}: price`).fill('0.1')
  await dialog.getByRole('button', { name: t.addPlan }).click()
  await expect(rows(page).filter({ hasText: t.yourPlan })).toContainText(t.brokerYourOwn('IBI'))
  await card(page).getByRole('button', { name: t.newPlan }).click()
  await expect(page.getByRole('dialog', { name: t.yourPlanNumbered(2) })).toBeVisible()
})

test('a fee can be added where the original offers none', async ({ page }) => {
  await choice(page, t.exchange, exchangeName('Europe')).click()
  const { simple } = await copied(page, 'Altshuler · Full tariff')
  expect(simple().trade).toBeUndefined()
  await copy(page, 'Altshuler · Full tariff')
  await expect(editor(page)).toContainText(t.notOfferedHere)
  await editor(page).getByRole('button', { name: t.addAFee }).click()
  await expect(editor(page).getByLabel(`${TRADE}: price`)).toHaveValue('0')
})

test('the full price list: rows, what they cover, and overlaps', async ({ page }) => {
  await card(page).getByRole('button', { name: t.newPlan }).click()
  const dialog = editor(page)
  await choice(dialog, t.view, t.fullPriceList).click()

  // A row for what the user buys, which takes it from the broader row.
  const etfInUsa = `${securityName('Etf')} ב${exchangeName('Usa')}`
  const anythingAbroad = `הכול ב${exchangeName('Usa')}, ${exchangeName('Europe')}`
  await dialog.getByRole('button', { name: t.addRow }).first().click()
  await expect(dialog.getByRole('button', { name: `${etfInUsa} ▾` })).toBeVisible()
  await expect(dialog).toContainText(`מלבד ${etfInUsa} (שורה משלה למעלה)`)

  // Covering all of the USA and Europe makes the other row useless. The
  // ticks apply when the checklist closes.
  await dialog.getByRole('button', { name: `${etfInUsa} ▾` }).click()
  const checklist = page.getByRole('group', { name: t.whatItCovers })
  await checklist.getByLabel(securityName('Etf')).uncheck()
  await checklist.getByLabel(exchangeName('Europe')).check()
  await expect(dialog).not.toContainText(t.neverUsed)
  await dialog.getByRole('button', { name: `${etfInUsa} ▾` }).click()
  await expect(checklist).toBeHidden()
  await expect(dialog).toContainText(t.neverUsed)

  await dialog
    .getByRole('button', { name: t.removeRow(anythingAbroad) })
    .first()
    .click()
  await expect(dialog).not.toContainText(t.neverUsed)
  // Esc closes an open checklist, not the whole editor.
  await dialog.getByRole('button', { name: `${anythingAbroad} ▾` }).click()
  await page.keyboard.press('Escape')
  await expect(checklist).toBeHidden()
  await expect(dialog).toBeVisible()
})

test('the simple view shows the fees for every security and exchange', async ({ page }) => {
  // A loop over everything: near the timeout under load.
  test.slow()
  for (const exchange of ['Tlv', 'Usa', 'Europe'] as Exchange[]) {
    await choice(page, t.exchange, exchangeName(exchange)).click()
    for (const security of ['Etf', 'IndexFund', 'Bond', 'Stock'] as Security[]) {
      await choice(page, t.security, securityName(security)).click()
      const { simple } = await copied(page, 'Leumi · Online')
      await copy(page, 'Leumi · Online')
      const dialog = editor(page)
      await expect(dialog, `${security} on ${exchange}`).toContainText(
        t.forPurchase(purchasePhrase(security, exchange)),
      )
      await expect(dialog.getByLabel(`${TRADE}: price`)).toBeVisible()
      await expect(dialog.getByLabel('Share of holdings: rate')).toBeVisible()
      // Nothing is converted on Tel Aviv.
      await expect(dialog.getByLabel('Conversion: fee')).toHaveCount(simple().conversion ? 1 : 0)
      await dialog.getByRole('button', { name: t.cancel }).click()
      await expect(dialog).toBeHidden()
    }
  }
})

test('✎ shows on hover', async ({ page }) => {
  const { label, plan } = listed('Leumi · Pepper')
  const pencil = page.getByRole('button', { name: t.changeACopy(label) })
  await expect(pencil).toHaveCSS('opacity', '0')
  await page.locator('aside li', { hasText: plan.name }).hover()
  await expect(pencil).toHaveCSS('opacity', '1')
})

test('✎ always shows on touch screens', { tag: '@touch' }, async ({ page }) => {
  await expect(page.getByRole('button', { name: t.changeACopy(listed('Leumi · Pepper').label) })).toHaveCSS(
    'opacity',
    '1',
  )
})

test("a copy's standing order price can be changed, in both views", async ({ page }) => {
  await choice(page, t.security, securityName('IndexFund')).click()
  await choice(page, t.exchange, exchangeName('Tlv')).click()
  const { plan, simple } = await copied(page, 'Leumi · Online, monthly standing order')
  const { trade, standingOrder } = simple()
  await copy(page, 'Leumi · Online, monthly standing order')
  const dialog = editor(page)
  const byStandingOrder = fee('StandingOrder')
  // Buying otherwise stays Online's.
  await expect(dialog.getByLabel(`${TRADE}: price`)).toHaveValue(price(trade!.fields))
  await expect(dialog.getByLabel(`${byStandingOrder}: price`)).toHaveValue(price(standingOrder!.fields))
  await dialog.getByLabel(`${byStandingOrder}: price`).fill('0.1')
  await expect(dialog).toContainText(`${plan.name}: ${standingOrder!.price.text}`)
  await choice(dialog, t.view, t.fullPriceList).click()
  const covers = `${securityName('IndexFund')} ב${exchangeName('Tlv')}`
  await expect(dialog.getByLabel(`${byStandingOrder}, ${covers}: price`)).toHaveValue('0.1')
})

test("a copy's second conversion fee can be changed, and the copy then ends with more", async ({ page }) => {
  const label = "Leumi · Online, 'Leumi 18+'"
  await page.getByRole('checkbox', { name: listed(label).plan.name, exact: true }).check()
  const { plan, copy: data, simple } = await copied(page, label)
  const { conversion, secondConversion } = simple()
  await copy(page, label)
  const dialog = editor(page)
  const second = dialog.getByLabel(`${fee('SecondConversion')}: fee`)
  await expect(dialog.getByLabel('Conversion: fee')).toHaveValue(
    price(conversion!.fields.percent !== undefined ? { amount: conversion!.fields.percent } : {}),
  )
  await expect(second).toHaveValue(price({ amount: secondConversion!.fields.percent }))
  await second.fill('0.12')
  await expect(dialog).toContainText(`${plan.name}: ${secondConversion!.price.text}`)
  await dialog.getByRole('button', { name: t.addPlan }).click()
  // The copy converts for less than the original, so it's left with more
  // (the third amount: yearly cost, lost to fees, value if sold).
  const valueIfSold = async (name: string) => {
    const row = rows(page).filter({ has: page.getByText(name, { exact: true }) })
    return Number((await row.locator('td.amount').nth(2).textContent())!.replace(/\D/g, ''))
  }
  expect(await valueIfSold(planInfo(data).name)).toBeGreaterThan(await valueIfSold(plan.name))
})

test('a copy is made on the track the comparison picked', async ({ page }) => {
  await page.getByRole('checkbox', { name: listed('IBI · Full tariff').broker.name, exact: true }).check()
  const { plan, simple } = await copied(page, 'IBI · Full tariff')
  const trade = simple().trade!
  await copy(page, 'IBI · Full tariff')
  const dialog = editor(page)
  await expect(dialog.getByLabel(`${TRADE}: unit`)).toHaveValue(trade.fields.kind)
  await expect(dialog.getByLabel(`${TRADE}: price`)).toHaveValue(price(trade.fields))
  await expect(dialog.getByLabel(`${TRADE}: per share`)).toHaveValue(price({ amount: trade.fields.perShare }))
  await dialog.getByLabel(`${TRADE}: per share`).fill('0.005')
  await expect(dialog).toContainText(`${plan.name}: ${trade.price.text}`)
})

test("a copy's handling fee, markup and fractions can be changed", async ({ page }) => {
  const { plan, simple } = await copied(page, 'Excellence · Typical offer')
  const { handling, markup, sellsFractions } = simple()
  await copy(page, 'Excellence · Typical offer')
  const dialog = editor(page)
  await expect(dialog.getByLabel('Fixed amount: a month')).toHaveValue(
    price({ amount: handling.fields.perMonth }),
  )
  await expect(dialog.getByLabel('Fixed amount: free months')).toHaveValue(String(handling.fields.freeMonths))
  await dialog.getByLabel('Fixed amount: free months').fill('36')
  await expect(dialog).toContainText(`${plan.name}: ${handling.price.text}`)
  await expect(dialog.getByLabel('Conversion: markup unit')).toHaveValue(
    markup!.fields.perDollar !== undefined ? 'perDollar' : 'percent',
  )
  await expect(dialog.getByLabel('Conversion: markup', { exact: true })).toHaveValue(
    price({ amount: markup!.fields.perDollar ?? markup!.fields.percent }),
  )
  await dialog.getByLabel('Conversion: markup unit').selectOption('percent')
  await expect(dialog.getByLabel('Conversion: markup', { exact: true })).toHaveValue('')
  await expect(dialog).toContainText(t.countedAs0)

  const fractions = dialog.getByRole('checkbox', { name: t.sellsFractions })
  await expect(fractions).toBeChecked({ checked: sellsFractions! })
  await fractions.uncheck()
  await choice(dialog, t.view, t.fullPriceList).click()
  const inFull = dialog.getByRole('group', { name: t.fractionsOfAShare })
  await expect(inFull.getByLabel(exchangeName('Usa'))).not.toBeChecked()
  await inFull.getByLabel(exchangeName('Usa')).check()
  await expect(dialog.getByLabel('Fixed amount: free months')).toHaveValue('36')
})

test("a copy's conversion by standing order can be changed", async ({ page }) => {
  const { plan, simple } = await copied(page, 'Interactive · Standard')
  expect(simple().standingOrderConversion).toBeDefined()
  await copy(page, 'Interactive · Standard')
  const dialog = editor(page)
  // Its automatic investment plan converts for free.
  const byStandingOrder = dialog.getByLabel(
    `${t.conversionByStandingOrder(feeKinds().find(({ value }) => value === 'StandingOrder')!.label)}: min`,
  )
  await expect(byStandingOrder).toHaveValue('')
  await byStandingOrder.fill('2')
  await expect(dialog).toContainText(`${plan.name}: ${simple().standingOrderConversion!.price.text}`)
})
