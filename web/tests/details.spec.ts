import { about, listed } from './core'
import { aboutPlan, closeDialog, expect, test } from './fixtures'
import { t } from '../src/lib/text'

// The details dialog: moving between a plan, its broker and the page about
// the numbers. What they say is the core's, tested in Rust.

test("a plan's prices unfold, and its broker leads back to it", { tag: '@phone' }, async ({ page }) => {
  const { broker, plan } = listed('Leumi · Pepper')
  await aboutPlan(page, 'Leumi · Pepper')
  const dialog = page.getByRole('dialog', { name: plan.name })
  await dialog.getByText(t.allPrices).click()
  await expect(dialog.locator('.tariff')).toBeVisible()

  await dialog.getByRole('button', { name: broker.name }).click()
  const brokerDialog = page.getByRole('dialog', { name: broker.name })
  await brokerDialog
    .locator('.plans li', { has: page.getByText(plan.name, { exact: true }) })
    .getByRole('button')
    .click()
  await expect(page.getByRole('dialog', { name: plan.name })).toBeVisible()
  await closeDialog(page)
})

test("the footer's accessibility statement opens at its heading", { tag: '@phone' }, async ({ page }) => {
  const footer = page.locator('footer.page-footer')
  await footer.getByRole('button', { name: t.accessibilityStatement }).click()
  const dialog = page.getByRole('dialog', { name: t.aboutTheNumbers })
  await expect(dialog.getByRole('heading', { name: about().accessibility.title })).toBeInViewport()
  await closeDialog(page)
})

test('the numbers are explained, from the title and from a plan', { tag: '@phone' }, async ({ page }) => {
  const { sections } = about()
  await page.getByRole('button', { name: t.aboutLink }).click()
  const dialog = page.getByRole('dialog', { name: t.aboutTheNumbers })
  await expect(dialog).toBeVisible()
  await closeDialog(page)

  const pepper = listed('Leumi · Pepper')
  await aboutPlan(page, 'Leumi · Pepper')
  await page
    .getByRole('dialog', { name: pepper.plan.name })
    .getByRole('button', { name: t.howTheNumbersAreMade })
    .click()
  await expect(dialog).toContainText(sections[1].title)
})
