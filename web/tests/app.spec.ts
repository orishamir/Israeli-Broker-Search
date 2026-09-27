import { expect, fitScreenToPage, test } from './fixtures'

const rows = (page: import('@playwright/test').Page) => page.locator('tbody tr')

const choice = (page: import('@playwright/test').Page, group: string, name: string) =>
  page.getByRole('radiogroup', { name: group }).getByText(name, { exact: true })

test('ranks every plan, best first', async ({ page }) => {
  await expect(rows(page)).toHaveCount(8)
  await expect(rows(page).first()).toContainText('New customers')
  await expect(page.getByText('$1 = ₪3.0338 · €1 = ₪3.4594 · 2026-09-25')).toBeVisible()
  await fitScreenToPage(page)
  await expect(page).toHaveScreenshot('ranked.png', { fullPage: true })
})

const checkbox = (page: import('@playwright/test').Page, name: string) =>
  page.getByRole('checkbox', { name, exact: true })

test('unticking plans removes them, and their pins', async ({ page }) => {
  await rows(page).filter({ hasText: 'Pepper' }).click()
  await checkbox(page, 'Bank Leumi').uncheck()
  await expect(rows(page)).toHaveCount(4)
  await checkbox(page, 'Pepper').check()
  await expect(rows(page)).toHaveCount(5)
  await expect(checkbox(page, 'Bank Leumi')).toHaveJSProperty('indeterminate', true)
  await expect(rows(page).filter({ hasText: 'Pepper' })).not.toHaveClass(/pinned/)
})

test('Tel Aviv needs no exchange rates or share price', async ({ page }) => {
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await expect(page.getByText('$1 = ₪3.0338')).toBeHidden()
  await expect(page.getByLabel('Share price')).toBeHidden()
  await expect(rows(page)).toHaveCount(8)
})

test('Europe: plans without a price go last', async ({ page }) => {
  await choice(page, 'Exchange', 'Europe').click()
  await expect(rows(page).last()).toHaveClass(/not-offered/)
  await expect(rows(page).first()).toContainText('Pepper')
})

test('bad inputs show why instead of breaking', async ({ page }) => {
  await page.getByLabel('Every month').fill('-5')
  await expect(page.getByText("the monthly deposit can't be negative")).toBeVisible()
  await expect(page).toHaveScreenshot('error.png')
  await page.getByLabel('Every month').fill('2000')
  await page.getByLabel('Yearly return', { exact: true }).fill('')
  await expect(page.getByText('fill in the yearly return')).toBeVisible()
  await page.getByLabel('Yearly return', { exact: true }).fill('10')
  await expect(rows(page)).toHaveCount(8)
})

test("a plan's ℹ explains it, and links to its broker", async ({ page }) => {
  await page.getByRole('button', { name: 'About Pepper' }).click()
  const dialog = page.getByRole('dialog', { name: 'Pepper' })
  await expect(dialog).toBeVisible()
  await expect(dialog).toContainText('For an ETF bought in the USA:')
  await expect(dialog).toContainText('Caveats')
  await dialog.getByText('Full tariff').click()
  await expect(page).toHaveScreenshot('plan-details.png')

  await dialog.getByRole('button', { name: 'Bank Leumi' }).click()
  const broker = page.getByRole('dialog', { name: 'Bank Leumi' })
  await expect(broker).toContainText('Online, monthly standing order')
  await expect(page).toHaveScreenshot('broker-details.png')

  await page.keyboard.press('Escape')
  await expect(broker).toBeHidden()
})

test('hovering a plan previews it', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no hovering on touch screens')
  await page.getByText('US $11 flat', { exact: true }).first().hover()
  const preview = page.locator('.preview')
  await expect(preview).toContainText('For an ETF bought in the USA:')
  await expect(preview).toHaveCSS('opacity', '1') // after fading in
  await expect(page).toHaveScreenshot('preview.png')
  await page.mouse.move(700, 10)
  await expect(preview).toBeHidden()
})

test("a plan's minimum one-time deposit warns, and it isn't called best", async ({ page }) => {
  const newCustomers = rows(page).filter({ hasText: 'New customers' })
  await expect(page.getByText('Best: New customers')).toBeVisible()
  await page.getByLabel('One-time deposit').fill('1000')
  await expect(newCustomers).toContainText('Needs a one-time deposit of at least ₪5,000')
  await expect(page.getByText('Best: New customers')).toBeHidden()
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
  await page.getByRole('button', { name: /^Pepper fees: .*see what they went to$/ }).click()
  await expect(page.getByRole('radio', { name: 'Breakdown' })).toBeChecked()
  await expect(page.locator('h4', { hasText: 'Year by year' })).toContainText('Pepper')
  await expect(rows(page).filter({ hasText: 'Pepper' })).toHaveClass(/pinned/)
  await expect(page.locator('#chart')).toBeInViewport()
})

test('hovering a plan in the sidebar highlights it in the table', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no hovering on touch screens')
  await page.locator('aside li', { hasText: 'Pepper' }).hover()
  await expect(rows(page).filter({ hasText: 'Pepper' })).toHaveClass(/highlighted/)
  await page.mouse.move(0, 0)
  await expect(rows(page).filter({ hasText: 'Pepper' })).not.toHaveClass(/highlighted/)
})
