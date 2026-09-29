import { formatNumber } from '../src/lib/numbers'
import { copyOf, inputsOnPage, listed, listedPlan, planInfo, setTrade, simpleFees, trackOnPage } from './core'
import type { PlanData } from './core'
import { choice, expect, rows, test, type Page } from './fixtures'

// Your plans: copying a listed plan to change, plans of your own, and the
// editor's two views. Expected values come from the core's editor functions.

const card = (page: Page) => page.locator('section.card', { hasText: 'Your plans' })
const editor = (page: Page) => page.getByRole('dialog')

/** A copy of the plan called `label`, as the app makes one for the page's
 * inputs: on the track the comparison picked. */
async function copied(page: Page, label: string) {
  const { key, plan } = listed(label)
  const inputs = await inputsOnPage(page)
  const track = trackOnPage(inputs, key)
  const copy = copyOf(key.broker, key.plan, track)
  const original = listedPlan(key.broker, key.plan, track)
  const simple = (data: PlanData = copy) => simpleFees(data, inputs.security, inputs.exchange, original)
  return { plan, copy, original, inputs, simple }
}

/** Opens a draft copy of a listed plan with its ✎: "Leumi · Pepper". */
async function copy(page: Page, label: string) {
  await page.getByRole('button', { name: `Change a copy of ${label}'s fees` }).click()
  await expect(editor(page)).toBeVisible()
}

/** Copies Pepper, charges $2 an order instead of its price, and adds it. */
async function addDeal(page: Page) {
  await copy(page, 'Leumi · Pepper')
  await editor(page).getByLabel('Buy or sell: price').fill('2')
  await editor(page).getByRole('button', { name: 'Add plan' }).click()
  await expect(editor(page)).toBeHidden()
}

const price = (fields: { amount?: number | null }) => formatNumber(fields.amount ?? null)

test("✎ copies a plan to change, showing the original's fees", { tag: '@phone' }, async ({ page }) => {
  await expect(card(page)).toContainText("Think you can get lower fees, or use a broker that isn't listed?")
  const { plan, copy: data, inputs, simple } = await copied(page, 'Leumi · Pepper')
  await copy(page, 'Leumi · Pepper')
  const dialog = page.getByRole('dialog', { name: planInfo(data).name })
  await expect(dialog).toContainText(`Copy of ${plan.name} · Bank Leumi`)
  const trade = simple().trade!
  await expect(dialog.getByLabel('Buy or sell: price')).toHaveValue(price(trade.fields))

  await dialog.getByLabel('Buy or sell: price').fill('2')
  const changed = setTrade(data, inputs.security, inputs.exchange, { ...trade.fields, amount: 2 })
  const was = simple(changed).trade!.was!
  await expect(dialog).toContainText(`${plan.name}: ${was.price.text}`)

  await dialog.getByRole('button', { name: 'Add plan' }).click()
  await expect(dialog).toBeHidden()
  await expect(card(page)).toContainText(`Copy of ${plan.name} · Bank Leumi`)
  const row = rows(page).filter({ hasText: planInfo(data).name })
  await expect(row).toContainText('Your deal · Bank Leumi')
  await expect(row.locator('.mark')).toHaveClass(/yours/)
  await expect(rows(page)).toHaveCount((await inputsOnPage(page)).plans.length + 1)
})

test('↺ goes back to the original fee; a bad fee says why and keeps the last good one', async ({ page }) => {
  const { plan, simple } = await copied(page, 'Leumi · Pepper')
  await copy(page, 'Leumi · Pepper')
  const field = editor(page).getByLabel('Buy or sell: price')
  await field.fill('2')
  await editor(page)
    .getByRole('button', { name: `Back to ${plan.name}'s fee` })
    .click()
  await expect(field).toHaveValue(price(simple().trade!.fields))
  await expect(editor(page)).not.toContainText(`${plan.name}:`)
  await field.fill('-3')
  await expect(editor(page)).toContainText("Fees can't be negative")
  // Still the original's fee, so nothing to go back to.
  await expect(editor(page)).not.toContainText(`${plan.name}:`)
})

test(
  'Cancel adds nothing; deleting asks first, then unticks and unpins',
  { tag: '@phone' },
  async ({ page }) => {
    const before = await rows(page).count()
    await copy(page, 'Leumi · Pepper')
    await editor(page).getByRole('button', { name: 'Cancel' }).click()
    await expect(editor(page)).toBeHidden()
    await expect(card(page)).toContainText('Think you can get lower fees')
    await expect(rows(page)).toHaveCount(before)

    await addDeal(page)
    const { copy: data } = await copied(page, 'Leumi · Pepper')
    const name = planInfo(data).name
    await rows(page).filter({ hasText: name }).click()
    await expect(page.locator('tbody tr.pinned')).toHaveCount(1)
    await page.getByRole('button', { name: `Change ${name}` }).click()
    await editor(page).getByRole('button', { name: 'Delete plan' }).click()
    await expect(editor(page)).toContainText(`Delete “${name}”?`)
    await editor(page).getByRole('button', { name: 'Keep', exact: true }).click()
    await editor(page).getByRole('button', { name: 'Delete plan' }).click()
    await editor(page).getByRole('button', { name: 'Delete', exact: true }).click()
    await expect(editor(page)).toBeHidden()
    await expect(rows(page)).toHaveCount(before)
    await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
    await expect(card(page)).toContainText('Think you can get lower fees')
  },
)

test('your plans and the editor view stay after a reload', async ({ page }) => {
  const { plan, copy: data, inputs, simple } = await copied(page, 'Leumi · Pepper')
  const name = planInfo(data).name
  await addDeal(page)
  await page.getByRole('button', { name: `Change ${name}` }).click()
  await choice(editor(page), 'View', 'Full price list').click()
  await editor(page).getByRole('button', { name: 'Done' }).click()

  await page.reload()
  await expect(rows(page).filter({ hasText: name })).toContainText('Your deal · Bank Leumi')
  await page.getByRole('button', { name: `Change ${name}` }).click()
  await expect(editor(page).getByRole('radio', { name: 'Full price list' })).toBeChecked()
  // Still $2, not the original's price.
  const trade = simple().trade!
  const changed = setTrade(data, inputs.security, inputs.exchange, { ...trade.fields, amount: 2 })
  await expect(editor(page)).toContainText(`${plan.name}: ${simple(changed).trade!.was!.price.text}`)
})

test('a new plan, with a broker name, then the next free name', async ({ page }) => {
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  const dialog = page.getByRole('dialog', { name: 'Your plan' })
  await dialog.getByPlaceholder('optional').fill('IBI')
  await dialog.getByLabel('Buy or sell: price').fill('0.1')
  await dialog.getByRole('button', { name: 'Add plan' }).click()
  await expect(rows(page).filter({ hasText: 'Your plan' })).toContainText('IBI · your own')
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  await expect(page.getByRole('dialog', { name: 'Your plan 2' })).toBeVisible()
})

test('a fee can be added where the original offers none', async ({ page }) => {
  await choice(page, 'Exchange', 'Europe').click()
  const { simple } = await copied(page, 'Altshuler · Full tariff')
  expect(simple().trade).toBeUndefined()
  await copy(page, 'Altshuler · Full tariff')
  await expect(editor(page)).toContainText('not offered')
  await editor(page).getByRole('button', { name: '+ Add a fee' }).click()
  await expect(editor(page).getByLabel('Buy or sell: price')).toHaveValue('0')
})

test('the full price list: rows, what they cover, and overlaps', async ({ page }) => {
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  const dialog = editor(page)
  await choice(dialog, 'View', 'Full price list').click()

  // A row for what the user buys, which takes it from the broader row.
  await dialog.getByRole('button', { name: '+ Row' }).first().click()
  await expect(dialog.getByRole('button', { name: 'ETF on USA ▾' })).toBeVisible()
  await expect(dialog).toContainText('except ETF on USA (its own row above)')

  // Covering all of the USA and Europe makes the other row useless. The
  // ticks apply when the checklist closes.
  await dialog.getByRole('button', { name: 'ETF on USA ▾' }).click()
  const checklist = page.getByRole('group', { name: 'What it covers' })
  await checklist.getByLabel('ETF').uncheck()
  await checklist.getByLabel('Europe').check()
  await expect(dialog).not.toContainText('Never used')
  await dialog.getByRole('button', { name: 'ETF on USA ▾' }).click()
  await expect(checklist).toBeHidden()
  await expect(dialog).toContainText('Never used: more specific rows cover all of it.')

  await dialog.getByRole('button', { name: 'Remove the row for Anything on USA, Europe' }).first().click()
  await expect(dialog).not.toContainText('Never used')
  // Esc closes an open checklist, not the whole editor.
  await dialog.getByRole('button', { name: 'Anything on USA, Europe ▾' }).click()
  await page.keyboard.press('Escape')
  await expect(checklist).toBeHidden()
  await expect(dialog).toBeVisible()
})

test('the simple view shows the fees for every security and exchange', async ({ page }) => {
  // A loop over everything: near the timeout under load.
  test.slow()
  for (const exchange of ['Tel Aviv', 'USA', 'Europe']) {
    await choice(page, 'Exchange', exchange).click()
    for (const security of ['ETF', 'Index fund', 'Bond', 'Stock']) {
      await choice(page, 'Security', security).click()
      const { simple } = await copied(page, 'Leumi · Online')
      await copy(page, 'Leumi · Online')
      const dialog = editor(page)
      await expect(dialog, `${security} on ${exchange}`).toContainText('bought in')
      await expect(dialog.getByLabel('Buy or sell: price')).toBeVisible()
      await expect(dialog.getByLabel('Share of holdings: rate')).toBeVisible()
      // Nothing is converted on Tel Aviv.
      await expect(dialog.getByLabel('Conversion: fee')).toHaveCount(simple().conversion ? 1 : 0)
      await dialog.getByRole('button', { name: 'Cancel' }).click()
      await expect(dialog).toBeHidden()
    }
  }
})

test('✎ shows on hover', async ({ page }) => {
  const pencil = page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" })
  await expect(pencil).toHaveCSS('opacity', '0')
  await page.locator('aside li', { hasText: 'Pepper' }).hover()
  await expect(pencil).toHaveCSS('opacity', '1')
})

test('✎ always shows on touch screens', { tag: '@touch' }, async ({ page }) => {
  await expect(page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" })).toHaveCSS(
    'opacity',
    '1',
  )
})

test("a copy's standing order price can be changed, in both views", async ({ page }) => {
  await choice(page, 'Security', 'Index fund').click()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  const { plan, simple } = await copied(page, 'Leumi · Online, monthly standing order')
  const { trade, standingOrder } = simple()
  await copy(page, 'Leumi · Online, monthly standing order')
  const dialog = editor(page)
  // Buying otherwise stays Online's.
  await expect(dialog.getByLabel('Buy or sell: price')).toHaveValue(price(trade!.fields))
  await expect(dialog.getByLabel('Standing order: price')).toHaveValue(price(standingOrder!.fields))
  await dialog.getByLabel('Standing order: price').fill('0.1')
  await expect(dialog).toContainText(`${plan.name}: ${standingOrder!.price.text}`)
  await choice(dialog, 'View', 'Full price list').click()
  await expect(dialog.getByLabel('Standing order, Index fund on Tel Aviv: price')).toHaveValue('0.1')
})

test("a copy's second conversion fee can be changed, and the copy then ends with more", async ({ page }) => {
  const label = "Leumi · Online, 'Leumi 18+'"
  await page.getByRole('checkbox', { name: "Online, 'Leumi 18+'", exact: true }).check()
  const { plan, copy: data, simple } = await copied(page, label)
  const { conversion, secondConversion } = simple()
  await copy(page, label)
  const dialog = editor(page)
  await expect(dialog.getByLabel('Conversion: fee')).toHaveValue(
    price(conversion!.fields.percent !== undefined ? { amount: conversion!.fields.percent } : {}),
  )
  await expect(dialog.getByLabel('Second conversion fee: fee')).toHaveValue(
    price({ amount: secondConversion!.fields.percent }),
  )
  await dialog.getByLabel('Second conversion fee: fee').fill('0.12')
  await expect(dialog).toContainText(`${plan.name}: ${secondConversion!.price.text}`)
  await dialog.getByRole('button', { name: 'Add plan' }).click()
  // The copy converts for less than the original, so it's left with more
  // (the third amount: yearly cost, lost to fees, value if sold).
  const valueIfSold = async (name: string) => {
    const row = rows(page).filter({ has: page.getByText(name, { exact: true }) })
    return Number((await row.locator('td.amount').nth(2).textContent())!.replace(/\D/g, ''))
  }
  expect(await valueIfSold(planInfo(data).name)).toBeGreaterThan(await valueIfSold(plan.name))
})

test('a copy is made on the track the comparison picked', async ({ page }) => {
  await page.getByRole('checkbox', { name: 'IBI', exact: true }).check()
  const { plan, simple } = await copied(page, 'IBI · Full tariff')
  const trade = simple().trade!
  await copy(page, 'IBI · Full tariff')
  const dialog = editor(page)
  await expect(dialog.getByLabel('Buy or sell: unit')).toHaveValue(trade.fields.kind)
  await expect(dialog.getByLabel('Buy or sell: price')).toHaveValue(price(trade.fields))
  await expect(dialog.getByLabel('Buy or sell: per share')).toHaveValue(
    price({ amount: trade.fields.perShare }),
  )
  await dialog.getByLabel('Buy or sell: per share').fill('0.005')
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
  await expect(dialog).toContainText('counted as 0')

  const fractions = dialog.getByRole('checkbox', { name: 'Sells fractions of a share' })
  await expect(fractions).toBeChecked({ checked: sellsFractions! })
  await fractions.uncheck()
  await choice(dialog, 'View', 'Full price list').click()
  const inFull = dialog.getByRole('group', { name: 'Fractions of a share' })
  await expect(inFull.getByLabel('USA')).not.toBeChecked()
  await inFull.getByLabel('USA').check()
  await expect(dialog.getByLabel('Fixed amount: free months')).toHaveValue('36')
})

test("a copy's conversion by standing order can be changed", async ({ page }) => {
  const { plan, simple } = await copied(page, 'Interactive · Standard')
  expect(simple().standingOrderConversion).toBeDefined()
  await copy(page, 'Interactive · Standard')
  const dialog = editor(page)
  // Its automatic investment plan converts for free.
  const byStandingOrder = dialog.getByLabel('Conversion by standing order: min')
  await expect(byStandingOrder).toHaveValue('')
  await byStandingOrder.fill('2')
  await expect(dialog).toContainText(`${plan.name}: ${simple().standingOrderConversion!.price.text}`)
})
