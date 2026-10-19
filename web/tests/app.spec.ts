import { test as baseTest } from '@playwright/test'
import { expectedRows, inputsOnPage, listed, RATES, rowsOnPage, securityName, usualPlans } from './core'
import { dateText } from '../src/lib/format'
import { arenaOf } from '../src/lib/kinds'
import { t } from '../src/lib/text'
import {
  answerCard,
  choice,
  enter,
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

test("Share copies a link that opens the same answer, over the guide's own choice", async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  const answers = await enter(page, { now: 50_000, monthly: 0, horizon: 5, flexibility: 'CanWait' })
  const guide = await expectAnswerFor(page, answers)
  // Continuing with another kind is part of the link too.
  const other = guide.ranked.find(({ kind }) => JSON.stringify(kind) !== JSON.stringify(guide.kind.kind))!
  await answerCard(page).getByRole('button', { name: t.changeKind }).click()
  await answerCard(page).locator('.picker li').nth(guide.ranked.indexOf(other)).getByRole('button').click()

  await page.getByRole('button', { name: t.share, exact: true }).click()
  await expect(page.getByText(t.linkCopied)).toBeVisible()
  const link = await page.evaluate(() => navigator.clipboard.readText())
  expect(link).toContain('v=2')

  await page.goto(link)
  await expect(page.locator('#now')).toHaveValue('50,000')
  await expect(page.locator('#horizon')).toHaveValue('5')
  await expect(page.getByRole('button', { name: t.backToRecommendation })).toBeVisible()
  await expectAnswerFor(page, answers, { arena: arenaOf(other.kind), kind: other.kind })
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
