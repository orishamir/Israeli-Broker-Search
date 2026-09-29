import { feesFor, listed, purchaseOnPage, securities, trackOnPage } from './core'
import { away, choice, expect, test, type Page } from './fixtures'

// The "?" tips: how they open and close, and that they explain in Hebrew too.

const yearlyReturn = (page: Page) => ({
  button: page.getByRole('button', { name: 'What “Yearly return” means' }),
  tip: page.getByRole('tooltip').filter({ hasText: 'The S&P 500 has averaged about 10%' }),
})

test('a ? explains its label on hover, until the mouse leaves', async ({ page }) => {
  const { button, tip } = yearlyReturn(page)
  await expect(tip).toBeHidden()
  await button.hover()
  await expect(tip).toBeVisible()
  await expect(tip).toHaveScreenshot('tip.png')
  await away(page)
  await expect(tip).toBeHidden()
})

test('a tapped ? explains its label until a tap elsewhere', { tag: '@touch' }, async ({ page }) => {
  const { button, tip } = yearlyReturn(page)
  await button.tap()
  await expect(tip).toBeVisible()
  await page.getByRole('heading', { name: 'Deposits' }).tap()
  await expect(tip).toBeHidden()
})

test('a clicked ? stays open until Esc', async ({ page }) => {
  const button = page.getByRole('button', { name: 'What “Value if sold” means' })
  const tip = page.getByRole('tooltip').filter({ hasText: 'after the sell fee' })
  await button.click()
  await away(page)
  await expect(tip).toBeVisible()
  await page.keyboard.press('Escape')
  await expect(tip).toBeHidden()
})

test('hovering a security explains it, with its Hebrew names', async ({ page }) => {
  const bond = securities().find(({ value }) => value === 'Bond')!
  await choice(page, 'Security', bond.name).hover()
  const tip = page.getByRole('tooltip').filter({ hasText: bond.explanation })
  await expect(tip).toBeVisible()
  for (const name of bond.hebrewNames) await expect(tip).toContainText(name)
})

/** Opens Pepper's details, and finds the account fee's "?" and what the tip should say. */
async function custodyTip(page: Page) {
  const { key, label } = listed('Leumi · Pepper')
  const { inputs, purchase } = await purchaseOnPage(page)
  const custody = feesFor(key.broker, key.plan, purchase, trackOnPage(inputs, key)).fees.find(
    ({ kind }) => kind === 'Account',
  )!
  await page.getByRole('button', { name: `About ${label}` }).click()
  return {
    button: page.getByRole('dialog').getByRole('button', { name: `What “${custody.name}” means` }),
    tip: page.getByRole('tooltip').filter({ hasText: custody.explanation }),
    hebrew: custody.hebrewNames[0],
  }
}

test("a fee explains itself, with its Hebrew name, in a plan's details", async ({ page }) => {
  const { button, tip, hebrew } = await custodyTip(page)
  await button.hover()
  await expect(tip).toBeVisible()
  await expect(tip).toContainText(hebrew)
})

test("a fee's ? opens on tap in a plan's details", { tag: '@touch' }, async ({ page }) => {
  const { button, tip, hebrew } = await custodyTip(page)
  await button.tap()
  await expect(tip).toBeVisible()
  await expect(tip).toContainText(hebrew)
})
