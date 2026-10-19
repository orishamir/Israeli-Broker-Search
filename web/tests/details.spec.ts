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

test(
  'the About sheet opens with a table of contents that leads to every section',
  { tag: '@phone' },
  async ({ page }) => {
    await page.getByRole('button', { name: t.aboutLink }).click()
    const dialog = page.getByRole('dialog', { name: t.aboutTheNumbers })
    const { sections, accessibility } = about()
    const all = [...sections, accessibility]
    const contents = dialog.getByRole('navigation', { name: t.aboutContents })
    const links = contents.getByRole('link')
    await expect(links).toHaveCount(all.length)
    // Every entry points at a section that is on the page.
    const targets = await links.evaluateAll((anchors) => anchors.map((a) => a.getAttribute('href')!.slice(1)))
    for (const target of targets) await expect(dialog.locator(`section[id="${target}"]`)).toHaveCount(1)
    // The last entry scrolls its heading into view and takes the focus.
    await links.last().click()
    const heading = dialog.getByRole('heading', { name: accessibility.title })
    await expect(heading).toBeInViewport()
    await expect(heading).toBeFocused()
    // And the way back.
    await dialog.getByRole('link', { name: t.backToTop }).last().click()
    await expect(dialog.getByRole('heading', { name: t.aboutTheNumbers })).toBeInViewport()
    await closeDialog(page)
  },
)
