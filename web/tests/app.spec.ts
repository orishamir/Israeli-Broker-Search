import { expectedRows, inputsOnPage, listed, RATES, rowsOnPage, securityName, usualPlans } from './core'
import { dateText } from '../src/lib/format'
import { t } from '../src/lib/text'
import { choice, expect, planInList, rows, test, tickBrokers } from './fixtures'

// Starting up: what the page shows once the core has loaded.

test(
  "starts with each broker's usual plan and the provident fund's average ticked and compared",
  { tag: '@phone' },
  async ({ page }) => {
    await expect(rows(page)).toHaveCount(usualPlans.length)
    await expect(page.getByText(t.tickedAtFirst)).toBeVisible()
    // Each is the only plan of its broker ticked, and the usual one: named by the broker alone.
    await expect(page.getByRole('list', { name: t.whatsCompared }).getByRole('listitem')).toHaveText(
      usualPlans.map(({ broker }) => broker.shortName),
    )
    // In the list, a broker's usual plan is marked so; a fund has none.
    for (const { englishLabel, broker } of usualPlans) {
      const plan = await planInList(page, englishLabel)
      await expect(plan.getByRole('checkbox')).toBeChecked()
      await expect(plan.locator('.usual')).toHaveCount(broker.kind === 'Funds' ? 0 : 1)
    }
    // Of the funds, listed apart, only the provident fund's average.
    await expect(
      (await planInList(page, 'Savings policy · Average fee')).getByRole('checkbox'),
    ).not.toBeChecked()
  },
)

test(
  "shows today's rates once downloaded, and the loading shape is gone",
  { tag: '@phone' },
  async ({ page }) => {
    await expect(
      page.getByText(`$1 = ₪${RATES.ilsPerUsd} · €1 = ₪${RATES.ilsPerEur} · ${dateText(RATES.date)}`),
    ).toBeVisible()
    // The page's shape from index.html, shown while loading, has been removed.
    await expect(page.locator('#skeleton')).toHaveCount(0)
  },
)

test('Share copies a link that opens the same comparison', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await choice(page, t.security, securityName('Bond')).click()
  await page.getByLabel(t.everyMonth).fill('3500')
  await tickBrokers(page, [listed('Leumi · Online').broker.name])
  const before = expectedRows(await inputsOnPage(page))
  await expect.poll(() => rowsOnPage(page)).toEqual(before)

  await page.getByRole('button', { name: t.share, exact: true }).click()
  await expect(page.getByText(t.linkCopied)).toBeVisible()
  const link = await page.evaluate(() => navigator.clipboard.readText())
  expect(link).toContain('#s=Bond&x=Usa&d=10000&m=3500')

  await page.goto(link)
  await expect(page.locator('.chart canvas')).toBeVisible()
  expect(await inputsOnPage(page)).toMatchObject({ security: 'Bond', monthlyDeposit: 3500 })
  await expect.poll(() => rowsOnPage(page)).toEqual(before)
})
