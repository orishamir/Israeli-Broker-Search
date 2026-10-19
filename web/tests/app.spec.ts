import { test as baseTest } from '@playwright/test'
import {
  banks,
  expectedRows,
  inputsOnPage,
  listed,
  RATES,
  rowsOnPage,
  securityName,
  situationAnswers,
  usualPlans,
} from './core'
import { dateText } from '../src/lib/format'
import { t } from '../src/lib/text'
import {
  choice,
  decide,
  expect,
  expectAnswerFor,
  expertSwitch,
  planInList,
  rows,
  setExpert,
  test,
  tickBrokers,
  workbench,
} from './fixtures'

// Starting up, and links: what the page shows once the core has loaded, and
// that a link opens the page as it was.

test(
  "starts with each broker's usual plan and the provident fund's average ticked and compared",
  { tag: '@phone' },
  async ({ page }) => {
    await workbench(page)
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
    await workbench(page)
    await page.locator('section.workbench').getByRole('button', { name: t.changeAssumptions }).click()
    await expect(
      page.getByText(`$1 = ₪${RATES.ilsPerUsd} · €1 = ₪${RATES.ilsPerEur} · ${dateText(RATES.date)}`),
    ).toBeVisible()
    // The page's shape from index.html, shown while loading, has been removed.
    await expect(page.locator('#skeleton')).toHaveCount(0)
  },
)

test('Share copies a link that opens the same answer: its choices and the bank', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  const answers = situationAnswers('goal')
  await expectAnswerFor(page, answers)
  await decide(page, answers, {}, 'SafeWay', { SafeWay: 'Bond' })
  // The bank, asked where to buy, where it changes what is shown.
  const bank = banks()[0]
  const where = page.locator('section.how .where')
  await where.getByRole('button', { name: bank.name, exact: true }).click()
  const chosen = { ...answers, bank: bank.bank }
  await expectAnswerFor(page, chosen, { safeWay: 'Bond' })

  await page.getByRole('button', { name: t.share, exact: true }).click()
  await expect(page.getByText(t.linkCopied)).toBeVisible()
  const link = await page.evaluate(() => navigator.clipboard.readText())
  expect(link).toContain('v=2')

  await page.goto(link)
  await expectAnswerFor(page, chosen, { safeWay: 'Bond' })
  await expect(where.getByRole('button', { name: bank.name, exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  )
  // And a link edited into nonsense still opens the page.
  await page.goto('/#v=2&n=abc&h=zz')
  await expect(page.locator('#now')).toBeVisible()
})

test('Share copies a link that opens the same workbench', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await workbench(page)
  await choice(page, t.security, securityName('Bond')).click()
  await page.locator('#monthly').fill('3500')
  await tickBrokers(page, [listed('Leumi · Online').broker.name])
  const before = expectedRows(await inputsOnPage(page))
  await expect.poll(() => rowsOnPage(page)).toEqual(before)

  await page.getByRole('button', { name: t.share, exact: true }).click()
  await expect(page.getByText(t.linkCopied)).toBeVisible()
  const link = await page.evaluate(() => navigator.clipboard.readText())
  expect(link).toContain('v=2')

  await page.goto(link)
  await expect(expertSwitch(page)).toBeChecked()
  await expect(page.locator('#monthly')).toHaveValue('3,500')
  expect(await inputsOnPage(page)).toMatchObject({ security: 'Bond', monthlyDeposit: 3500 })
  await expect.poll(() => rowsOnPage(page)).toEqual(before)
  await setExpert(page, false)
})

// Without the fixture, which waits for the app.
baseTest("if the core can't load, the page says so instead of pulsing on", async ({ page }) => {
  await page.route(/\.wasm(\?|$)/, (route) => route.abort())
  await page.goto('/')
  await expect(page.getByRole('alert')).toHaveText(t.couldntLoad)
  // The shape that pulsed is replaced by the message: nothing pulses on.
  await expect(page.locator('#skeleton')).toHaveClass(/failed/)
  await expect(page.locator('#skeleton .block')).toHaveCount(0)
})
