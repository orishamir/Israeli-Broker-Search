import { feesFor, listed, purchaseOnPage, trackOnPage } from './core'
import { aboutPlan, away, details, expect, test, type Page } from './fixtures'
import { t } from '../src/lib/text'

// The "?" tips: how they open and close.

const yearlyReturn = (page: Page) => ({
  button: page.getByRole('button', { name: t.whatMeans(t.yearlyReturn) }),
  tip: page.getByRole('tooltip').filter({ hasText: t.yearlyReturnTip }),
})

test('a ? explains its label on hover, until the mouse leaves', async ({ page }) => {
  const { button, tip } = yearlyReturn(page)
  await expect(tip).toBeHidden()
  await button.hover()
  await expect(tip).toBeVisible()
  await away(page)
  await expect(tip).toBeHidden()
})

test('a tapped ? explains its label until a tap elsewhere', { tag: '@touch' }, async ({ page }) => {
  const { button, tip } = yearlyReturn(page)
  await button.tap()
  await expect(tip).toBeVisible()
  await page.getByRole('heading', { name: t.deposits }).tap()
  await expect(tip).toBeHidden()
})

test('a clicked ? stays open until Esc', async ({ page }) => {
  const button = page.getByRole('button', { name: t.whatMeans(t.leftAfterTax) })
  const tip = page.getByRole('tooltip').filter({ hasText: t.leftAfterTaxTip })
  await button.click()
  await away(page)
  await expect(tip).toBeVisible()
  await page.keyboard.press('Escape')
  await expect(tip).toBeHidden()
})

/** Opens Pepper's details, and finds the account fee's "?" and its tip. */
async function custodyTip(page: Page) {
  const { key } = listed('Leumi · Pepper')
  const { inputs, purchase } = await purchaseOnPage(page)
  const custody = feesFor(key.broker, key.plan, purchase, trackOnPage(inputs, key)).fees.find(
    ({ kind }) => kind === 'Account',
  )!
  await aboutPlan(page, 'Leumi · Pepper')
  return {
    button: details(page).getByRole('button', { name: t.whatMeans(custody.name) }),
    tip: page.getByRole('tooltip').filter({ hasText: custody.explanation }),
  }
}

test("a fee's ? opens on tap in a plan's details", { tag: '@touch' }, async ({ page }) => {
  const { button, tip } = await custodyTip(page)
  await button.tap()
  await expect(tip).toBeVisible()
})
