import { expect, test } from './fixtures'

test('a ? explains its label, by hover or tap, until tapping elsewhere', async ({ page, isMobile }) => {
  const button = page.getByRole('button', { name: 'What “Yearly return” means' })
  const tip = page.getByRole('tooltip').filter({ hasText: 'The S&P 500 has averaged about 10%' })
  await expect(tip).toBeHidden()
  if (isMobile) await button.tap()
  else await button.hover()
  await expect(tip).toBeVisible()
  await expect(page).toHaveScreenshot('tip.png')
  if (isMobile) await page.getByRole('heading', { name: 'Deposits' }).tap()
  else await page.mouse.move(0, 0)
  await expect(tip).toBeHidden()
})

test('a clicked ? stays open until Esc', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no mouse to move away')
  const button = page.getByRole('button', { name: 'What “Value if sold” means' })
  const tip = page.getByRole('tooltip').filter({ hasText: 'after the sell fee' })
  await button.click()
  await page.mouse.move(0, 0)
  await expect(tip).toBeVisible()
  await page.keyboard.press('Escape')
  await expect(tip).toBeHidden()
})

test('the chosen security is explained, with its Hebrew names', async ({ page }) => {
  // The popovers say it too, hidden.
  const shown = (text: string) => page.locator('.explained').getByText(text)
  await expect(shown('קרן סל מחקה מדד')).toBeVisible()
  await page.getByRole('radiogroup', { name: 'Security' }).getByText('Mutual fund').click()
  await expect(shown('קרן נאמנות')).toBeVisible()
  await expect(shown('at one price a day')).toBeVisible()
})

test('hovering a security explains it', async ({ page, isMobile }) => {
  test.skip(isMobile, 'tapping picks it instead')
  await page.getByRole('radiogroup', { name: 'Security' }).getByText('Bond').hover()
  await expect(page.getByRole('tooltip').filter({ hasText: 'איגרת חוב' })).toBeVisible()
})

test('exchange rates are folded away until opened', async ({ page }) => {
  const summary = page.getByText('$1 = ₪3.0338 · €1 = ₪3.4594')
  await expect(summary).toBeVisible()
  await expect(page.getByLabel('$1 =')).toBeHidden()
  await summary.click()
  await page.getByLabel('$1 =').fill('4')
  await expect(page.getByText('$1 = ₪4 · €1 = ₪3.4594')).toBeVisible()
})

test('amounts show thousands separators, and the arrow keys step them', async ({ page }) => {
  const first = page.getByLabel('One-time deposit')
  await expect(first).toHaveValue('10,000')
  await first.focus()
  await page.keyboard.press('ArrowUp')
  await expect(first).toHaveValue('11,000')
  await first.fill('25000')
  await first.blur()
  await expect(first).toHaveValue('25,000')
  await expect(page.getByText('₪505,000')).toBeVisible() // deposited: 25,000 + 2,000 × 240
})

test("a plan's caveats and fees follow what's bought", async ({ page }) => {
  const about = page.getByRole('button', { name: 'About Pepper' })
  const dialog = page.getByRole('dialog', { name: 'Pepper' })

  await about.click()
  await expect(dialog.locator('dt', { hasText: 'Conversion markup' })).toBeVisible()
  await expect(dialog.getByText("doesn't publish its conversion markup")).toBeVisible()
  await expect(dialog.getByText('₪4 is only stated')).toBeHidden()
  await page.keyboard.press('Escape')

  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Tel Aviv').click()
  await about.click()
  await expect(dialog.locator('dt', { hasText: 'Conversion markup' })).toBeHidden()
  await expect(dialog.getByText('₪4 is only stated')).toBeVisible()
  await expect(dialog.getByText("doesn't publish its conversion markup")).toBeHidden()
  // The rest are in the full tariff.
  await dialog.getByText('Full tariff').click()
  await expect(dialog.getByText("doesn't publish its conversion markup")).toBeVisible()
})

test('a fee explains itself, with its Hebrew name', async ({ page, isMobile }) => {
  await page.getByRole('button', { name: 'About Pepper' }).click()
  const button = page.getByRole('dialog').getByRole('button', { name: 'What “Custody” means' })
  if (isMobile) await button.tap()
  else await button.hover()
  await expect(page.getByRole('tooltip').filter({ hasText: 'דמי משמרת' })).toBeVisible()
})
