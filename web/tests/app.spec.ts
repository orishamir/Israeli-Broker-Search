import { expectedRows, inputsOnPage, RATES, rowsOnPage, usualPlans } from './core'
import { checkbox, choice, expect, rows, test } from './fixtures'

// Starting up: what the page shows once the core has loaded.

test("starts with each broker's usual plan ticked and compared", { tag: '@phone' }, async ({ page }) => {
  await expect(rows(page)).toHaveCount(usualPlans.length)
  await expect(page.getByText("Ticked at first: each broker's usual plan")).toBeVisible()
  const usual = page.locator('aside li', { has: page.locator('.usual') })
  await expect(usual).toHaveCount(usualPlans.length)
  for (const plan of await usual.all()) await expect(plan.getByRole('checkbox')).toBeChecked()
  // "usual" isn't part of the plan's name.
  const { plan } = usualPlans[0]
  await expect(page.getByRole('checkbox', { name: plan.name, exact: true }).first()).toBeChecked()
})

test(
  "shows today's rates once downloaded, and the loading shape is gone",
  { tag: '@phone' },
  async ({ page }) => {
    await expect(
      page.getByText(`$1 = ₪${RATES.ilsPerUsd} · €1 = ₪${RATES.ilsPerEur} · ${RATES.date}`),
    ).toBeVisible()
    // The page's shape from index.html, shown while loading, has been removed.
    await expect(page.locator('#skeleton')).toHaveCount(0)
  },
)

test('Share copies a link that opens the same comparison', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await choice(page, 'Security', 'Bond').click()
  await page.getByLabel('Every month').fill('3500')
  await checkbox(page, 'Bank Leumi').check()
  const before = expectedRows(await inputsOnPage(page))
  await expect.poll(() => rowsOnPage(page)).toEqual(before)

  await page.getByRole('button', { name: 'Share', exact: true }).click()
  await expect(page.getByText('Link copied')).toBeVisible()
  const link = await page.evaluate(() => navigator.clipboard.readText())
  expect(link).toContain('#s=Bond&x=Usa&d=10000&m=3500')

  await page.goto(link)
  await expect(page.locator('.chart canvas')).toBeVisible()
  expect(await inputsOnPage(page)).toMatchObject({ security: 'Bond', monthlyDeposit: 3500 })
  await expect.poll(() => rowsOnPage(page)).toEqual(before)
})
