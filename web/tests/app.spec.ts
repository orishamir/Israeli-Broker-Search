import { expect, fitScreenToPage, test } from './fixtures'

const rows = (page: import('@playwright/test').Page) => page.locator('tbody tr')

const choice = (page: import('@playwright/test').Page, group: string, name: string) =>
  page.getByRole('radiogroup', { name: group }).getByText(name, { exact: true })

// At first, the plan a new customer usually gets from each broker.
test('ranks every plan, best first', async ({ page }) => {
  await expect(rows(page)).toHaveCount(6)
  await expect(rows(page).first()).toContainText('Standard')
  await expect(page.getByText('$1 = ₪3.0338 · €1 = ₪3.4594 · 2026-09-25')).toBeVisible()
  // The page's shape from index.html, shown while loading, is gone.
  await expect(page.locator('#skeleton')).toHaveCount(0)
  await fitScreenToPage(page)
  await expect(page).toHaveScreenshot('ranked.png', { fullPage: true })
})

const checkbox = (page: import('@playwright/test').Page, name: string) =>
  page.getByRole('checkbox', { name, exact: true })

test("each broker's usual plan is labelled, and ticked at first", async ({ page }) => {
  await expect(page.getByText("Ticked at first: each broker's usual plan")).toBeVisible()
  const usual = page.locator('aside li', { has: page.locator('.usual') })
  await expect(usual).toHaveCount(6)
  for (const plan of await usual.all()) await expect(plan.getByRole('checkbox')).toBeChecked()
  // The label isn't part of the plan's name.
  await expect(checkbox(page, 'Online')).toBeChecked()
  await expect(usual.filter({ has: checkbox(page, 'Online') })).toHaveCount(1)
})

test('unticking plans removes them, and their pins', async ({ page }) => {
  const leumi = rows(page).filter({ hasText: 'Bank Leumi' })
  await leumi.click()
  await expect(checkbox(page, 'Bank Leumi')).toHaveJSProperty('indeterminate', true)
  await checkbox(page, 'Bank Leumi').check()
  await expect(rows(page)).toHaveCount(9)
  await checkbox(page, 'Bank Leumi').uncheck()
  await expect(rows(page)).toHaveCount(5)
  await checkbox(page, 'Online').check()
  await expect(rows(page)).toHaveCount(6)
  await expect(leumi).not.toHaveClass(/pinned/)
})

test('Tel Aviv needs no exchange rates or share price', async ({ page }) => {
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await expect(page.getByText('$1 = ₪3.0338')).toBeHidden()
  await expect(page.getByLabel('Share price')).toBeHidden()
  await expect(rows(page)).toHaveCount(6)
})

test('Europe: plans without a price go last', async ({ page }) => {
  await choice(page, 'Exchange', 'Europe').click()
  await expect(rows(page).last()).toHaveClass(/not-offered/)
  await expect(rows(page).first()).not.toHaveClass(/not-offered/)
  // ETFs abroad are bought in whole shares, priced in the exchange's currency.
  const sharePrice = page.locator('.field').filter({ has: page.getByLabel('Share price') })
  await expect(sharePrice.locator('.unit')).toHaveText('€')
})

test('bad inputs show why instead of breaking', async ({ page }) => {
  await page.getByLabel('Every month').fill('-5')
  await expect(page.getByText("the monthly deposit can't be negative")).toBeVisible()
  await expect(page).toHaveScreenshot('error.png')
  await page.getByLabel('Every month').fill('2000')
  await page.getByLabel('Yearly return', { exact: true }).fill('')
  await expect(page.getByText('fill in the yearly return')).toBeVisible()
  await page.getByLabel('Yearly return', { exact: true }).fill('10')
  await expect(rows(page)).toHaveCount(6)
})

test("a plan's ℹ explains it, and links to its broker", async ({ page }) => {
  await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
  const dialog = page.getByRole('dialog', { name: 'Pepper' })
  await expect(dialog).toBeVisible()
  await expect(dialog).toContainText('For an ETF bought in the USA:')
  // Its caveats say how sure each number is, and when the numbers were checked.
  await expect(dialog).toContainText('Assumed, may cost more')
  await expect(dialog).toContainText('Checked 29/09/2026')
  await dialog.getByText('All prices').click()
  await expect(page).toHaveScreenshot('plan-details.png')

  await dialog.getByRole('button', { name: 'Bank Leumi' }).click()
  const broker = page.getByRole('dialog', { name: 'Bank Leumi' })
  await expect(broker).toContainText('Online, monthly standing order')
  await expect(page).toHaveScreenshot('broker-details.png')

  await page.mouse.move(0, 0)
  await page.keyboard.press('Escape')
  await expect(broker).toBeHidden()
})

test('hovering a plan previews it', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no hovering on touch screens')
  await page.getByText('Pepper', { exact: true }).first().hover()
  const preview = page.locator('.preview')
  await expect(preview).toContainText('For an ETF bought in the USA:')
  await expect(preview).toHaveCSS('opacity', '1') // after fading in
  await expect(page).toHaveScreenshot('preview.png')
  await page.mouse.move(700, 10)
  await expect(preview).toBeHidden()
})

test("a plan's minimum one-time deposit warns, and it isn't called best", async ({ page }) => {
  // Leaving Altshuler the best plan that isn't warned (IBI needs ₪15,000).
  // A broker's box is "mixed", which uncheck() takes for unchecked: untick its plan.
  await checkbox(page, 'Interactive Israel').uncheck()
  await page
    .getByRole('list', { name: 'Excellence Trade' })
    .getByRole('checkbox', { name: 'Typical offer' })
    .uncheck()
  const newCustomers = rows(page).filter({ hasText: 'New customers' })
  await expect(page.getByText('Best: Altshuler · New customers')).toBeVisible()
  await page.getByLabel('One-time deposit').fill('1000')
  await expect(newCustomers).toContainText('Needs a one-time deposit of at least ₪5,000')
  await expect(page.getByText('Best: Altshuler · New customers')).toBeHidden()
  await page.getByLabel('One-time deposit').fill('5000')
  await expect(newCustomers).not.toContainText('Needs a one-time deposit')
})

// Every label, choice and number, as text: a change to any shows as a diff.
test('the inputs and results, as text', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'the same on every device')
  await expect(page.locator('aside')).toMatchAriaSnapshot({ name: 'inputs.aria.yml' })
  await expect(page.locator('table')).toMatchAriaSnapshot({ name: 'results.aria.yml' })
})

test("a plan's fees paid open its breakdown", async ({ page }) => {
  await page.getByRole('button', { name: /^Leumi · Online fees: .*see what they went to$/ }).click()
  await expect(page.getByRole('radio', { name: 'Breakdown' })).toBeChecked()
  await expect(page.locator('h4', { hasText: 'Year by year' })).toContainText('Leumi · Online')
  await expect(rows(page).filter({ hasText: 'Bank Leumi' })).toHaveClass(/pinned/)
  await expect(page.locator('#chart')).toBeInViewport()
})

test('hovering a plan in the sidebar highlights it in the table', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no hovering on touch screens')
  await page.locator('aside li', { hasText: 'New customers' }).hover()
  await expect(rows(page).filter({ hasText: 'New customers' })).toHaveClass(/highlighted/)
  await page.mouse.move(0, 0)
  await expect(rows(page).filter({ hasText: 'New customers' })).not.toHaveClass(/highlighted/)
})

test('a standing order is used only when buying every month', async ({ page }) => {
  await checkbox(page, 'Online, monthly standing order').check()
  await choice(page, 'Security', 'Index fund').click()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  const row = (name: string) => rows(page).filter({ has: page.getByText(name, { exact: true }) })
  const standingOrder = row('Online, monthly standing order')
  await expect(standingOrder).not.toContainText("isn't used here")

  await page.getByLabel('Buy every', { exact: true }).selectOption({ label: '3 months' })
  await expect(standingOrder).toContainText("A standing order buys every month, so it isn't used here")
  // Then it costs what Online does.
  const amounts = (row: import('@playwright/test').Locator) => row.locator('td.amount').allTextContents()
  expect(await amounts(standingOrder)).toEqual(await amounts(row('Online')))
})

test('fees more than the deposits are flagged', async ({ page }) => {
  await checkbox(page, 'Altshuler Shaham Trade').check()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await page.getByLabel('One-time deposit').fill('0')
  await page.getByLabel('Every month').fill('50')
  // Altshuler's full tariff charges custody of at least ₪75 a month.
  const regular = rows(page).filter({ hasText: 'Altshuler' }).filter({ hasText: 'Full tariff' })
  await expect(regular).toContainText('⚠ Its fees are more than you deposit')
  await expect(rows(page).filter({ hasText: 'New customers' })).not.toContainText('more than you deposit')
})

test('the share price shows only where it matters', async ({ page }) => {
  const sharePrice = page.getByLabel('Share price', { exact: true })
  await expect(sharePrice).toBeVisible() // US ETFs are bought in whole shares
  await choice(page, 'Security', 'Index fund').click()
  // Funds are bought by amount, but one of Excellence's tracks for US
  // securities charges per share.
  await expect(sharePrice).toBeVisible()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await expect(sharePrice).toBeHidden()
  await expect(rows(page)).toHaveCount(6)
})

test('second prices show under the fee they belong to', async ({ page }) => {
  await page.getByRole('button', { name: "About Leumi · Online, 'Leumi 18+'" }).click()
  const plus18 = page.getByRole('dialog', { name: "Online, 'Leumi 18+'" })
  await expect(plus18.locator('.part', { hasText: 'or, if less' })).toContainText(
    '0.16%, min $5.76, max $2,400',
  )
  await page.mouse.move(0, 0)
  await page.keyboard.press('Escape')

  await choice(page, 'Security', 'Index fund').click()
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await page.getByRole('button', { name: 'About Leumi · Online, monthly standing order' }).click()
  const standingOrder = page.getByRole('dialog', { name: 'Online, monthly standing order' })
  await expect(standingOrder.locator('.part', { hasText: 'by standing order' })).toContainText(
    '0.225%, min ₪5, max ₪6,300',
  )
  await expect(standingOrder).toContainText('Buying by standing order')
})

test('a plan with tracks is compared on the cheapest, which is named', async ({ page }) => {
  await checkbox(page, 'IBI').check()
  const full = rows(page).filter({ hasText: 'IBI' }).filter({ hasText: 'Full tariff' })
  await expect(full).toContainText('US track: 0.15% + 1¢ a share, the cheapest for you')

  // Its fees name the track, and the others it has.
  await page.getByRole('button', { name: 'About IBI · Full tariff' }).click()
  const track = page.getByRole('dialog').locator('dd.part', { hasText: 'track picked for you' })
  await expect(track).toContainText(
    '0.15% + 1¢ a share (others: $0.01 per share, min $10; $14 per order; 0.15%, min $10)',
  )
  // On Tel Aviv the tracks don't matter.
  await page.mouse.move(0, 0)
  await page.keyboard.press('Escape')
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await expect(full).not.toContainText('track')
})

test('every plan can be ticked, and names its broker where the table does not', async ({ page }) => {
  for (const broker of ['Altshuler Shaham Trade', 'Bank Leumi', 'Excellence Trade', 'IBI', 'Meitav Trade']) {
    await checkbox(page, broker).check()
  }
  await expect(rows(page)).toHaveCount(13)
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  // Away from the plans, or the one under the pointer shows year by year.
  await page.mouse.move(0, 0)
  await expect(page.locator('h4', { hasText: 'Year by year' })).toContainText('Interactive · Standard')
})

const colorOf = (row: import('@playwright/test').Locator) =>
  row.locator('.mark').evaluate((mark) => getComputedStyle(mark).getPropertyValue('--plan-color'))

test('ticked plans all differ in color, and an unticked one frees its color', async ({ page }) => {
  const newCustomers = await colorOf(rows(page).filter({ hasText: 'New customers' }))
  await checkbox(page, 'New customers').uncheck()
  await checkbox(page, 'Pepper').check()
  expect(await colorOf(rows(page).filter({ hasText: 'Pepper' }))).toBe(newCustomers)

  for (const broker of ['Altshuler Shaham Trade', 'Bank Leumi', 'Excellence Trade', 'IBI', 'Meitav Trade']) {
    await checkbox(page, broker).check()
  }
  await expect(rows(page)).toHaveCount(13)
  const colors = await Promise.all((await rows(page).all()).map(colorOf))
  expect(new Set(colors).size).toBe(13)
})

// Opening each plan's details, with all its prices, breaks nothing.
test("every plan's details open", async ({ page }) => {
  // A loop over everything: near the timeout on the iPhone project under load.
  test.slow()
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  const buttons = page.getByRole('button', { name: /^About .+ · / })
  await expect(buttons).toHaveCount(13)
  for (const button of await buttons.all()) {
    await button.click()
    // Away, or a ? under the pointer opens its tip, which Esc would close.
    await page.mouse.move(0, 0)
    const dialog = page.getByRole('dialog')
    await dialog.getByText('All prices').click()
    await expect(dialog.locator('.tariff')).toBeVisible()
    await page.mouse.move(0, 0)
    await page.keyboard.press('Escape')
    await expect(dialog).toBeHidden()
  }
  expect(errors).toEqual([])
})

test('a number that may be too low is flagged, and the plan stays ranked', async ({ page }) => {
  // Pepper's $4 is stated only for orders up to $8,000, and selling
  // everything at the end is far more; Leumi's markup is measured, not flagged.
  await checkbox(page, 'Pepper').check()
  const pepper = rows(page).filter({ hasText: 'Pepper' })
  await expect(pepper).toContainText('May cost more: $4 is stated only for orders up to $8,000')
  await expect(pepper.locator('.rank')).not.toHaveText('–')
  await expect(rows(page).filter({ hasText: 'Online' })).not.toContainText('May cost more')
  // Interactive converts at the market rate, as its site says: nothing to flag.
  await expect(rows(page).filter({ hasText: 'Interactive' })).not.toContainText('May cost more')
  await expect(rows(page).filter({ hasText: 'Altshuler' })).not.toContainText('May cost more')
  // The ⚠ in the sidebar means the same thing, for a broker's every plan;
  // no broker has one left.
  await expect(page.getByRole('button', { name: /May cost more at/ })).toBeHidden()
})

test('the numbers are explained, from the title and from a plan', async ({ page }) => {
  await page.getByRole('button', { name: 'Sources' }).click()
  const about = page.getByRole('dialog', { name: 'About the numbers' })
  await expect(about).toBeVisible()
  await expect(about).toContainText('Checked 29/09/2026')
  // Every page the numbers rest on, by broker.
  await expect(about.getByRole('link', { name: "Leumi's exchange rates ↗" })).toBeVisible()
  await expect(about.getByRole('link', { name: 'Tariff (PDF) ↗' })).toHaveCount(6)
  await expect(about).toContainText('Bank Leumi · Tariff of 29/06/2026')
  await page.mouse.move(0, 0)
  await page.keyboard.press('Escape')
  await expect(about).toBeHidden()

  await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
  const pepper = page.getByRole('dialog', { name: 'Pepper' })
  await pepper.getByRole('button', { name: /How the numbers are made/ }).click()
  await expect(about).toBeVisible()
  await expect(about).toContainText("What isn't counted")
})
