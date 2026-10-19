import { feesFor, listed, purchaseOnPage, trackOnPage } from './core'
import { aboutPlan, away, details, expect, test, workbench, type Page } from './fixtures'
import { t } from '../src/lib/text'

// The "?" tips: how they open and close.

/** The assumptions are folded behind a line in the workbench. */
async function unfoldAssumptions(page: Page) {
  await workbench(page)
  await page.locator('section.workbench').getByRole('button', { name: t.changeAssumptions }).click()
}

const yearlyReturn = (page: Page) => ({
  button: page.getByRole('button', { name: t.whatMeans(t.yearlyReturn) }),
  tip: page.getByRole('tooltip').filter({ hasText: t.yearlyReturnTip }),
})

test('a ? explains its label on hover, until the mouse leaves', async ({ page }) => {
  await unfoldAssumptions(page)
  const { button, tip } = yearlyReturn(page)
  await expect(tip).toBeHidden()
  await button.hover()
  await expect(tip).toBeVisible()
  await away(page)
  await expect(tip).toBeHidden()
})

test('a tapped ? explains its label until a tap elsewhere', { tag: '@touch' }, async ({ page }) => {
  await unfoldAssumptions(page)
  const { button, tip } = yearlyReturn(page)
  await button.tap()
  await expect(tip).toBeVisible()
  // The title: on a phone the tip is tall enough to cover the cards beside it.
  await page.getByRole('heading', { name: t.title }).tap()
  await expect(tip).toBeHidden()
})

test('a clicked ? stays open until Esc', async ({ page }) => {
  // The table's column heads, in the workbench.
  await workbench(page)
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
  await workbench(page)
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

type Box = { x: number; y: number; width: number; height: number }
const overlap = (a: Box, b: Box) =>
  a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height

/** The situation chips, by position, that the open tip of chip `index` (its
 * "?") covers. */
async function coveredBy(page: Page, index: number) {
  const strip = page.getByRole('group', { name: t.whatForTitle })
  const chips = strip.locator('.chip')
  await away(page)
  await strip.locator('.tip-button').nth(index).hover()
  const tip = page.locator('.questions .popover.open')
  await expect(tip).toBeVisible()
  const shown = (await tip.boundingBox())!
  const boxes = await chips.evaluateAll((all) => all.map((each) => each.getBoundingClientRect().toJSON()))
  return boxes.flatMap((box, other) => (other !== index && overlap(shown, box) ? [other] : []))
}

// A situation's description stood over the chips beside it, so the one to
// press next was hidden behind the one just touched.
test('a situation’s tip covers none of the other chips', { tag: '@phone' }, async ({ page }) => {
  const count = await page.getByRole('group', { name: t.whatForTitle }).locator('.chip').count()
  expect(count).toBeGreaterThan(3)
  for (let index = 0; index < count; index++) expect(await coveredBy(page, index)).toEqual([])
})

// A chip's tap chose it and opened its tip, which stayed over the header
// (the review of 2026-10-20, X11): only its own "?" opens it.
test('choosing a situation opens no tip', { tag: '@phone' }, async ({ page }) => {
  const chips = page.getByRole('group', { name: t.whatForTitle }).locator('.chip')
  for (const at of [1, 2]) {
    await chips.nth(at).click()
    await expect(chips.nth(at)).toHaveAttribute('aria-pressed', 'true')
    await expect(page.locator('.popover.open')).toHaveCount(0)
  }
})
