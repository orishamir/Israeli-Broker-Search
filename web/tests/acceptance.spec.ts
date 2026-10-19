import {
  alternativesFor,
  compareWays,
  followUps,
  inputsOnPage,
  ladderFor,
  recommendation,
  situationAnswers,
  ways,
} from './core'
import type { AnswersInput, Kind } from './core'
import {
  answerCard,
  chip,
  choice,
  enter,
  expect,
  expectAnswerFor,
  expertSwitch,
  fillField,
  planInList,
  setExpert,
  situationName,
  test,
  workbench,
  closePlans,
  type Page,
} from './fixtures'
import { arenaOf } from '../src/lib/kinds'
import { shekels } from '../src/lib/format'
import { t } from '../src/lib/text'

// The user's five questions and the audit's five tasks (REDESIGN.md §8),
// end to end: what a person types, and that the page then does what the
// core says for it. Each checks behaviour (the arena, the kind named, the
// band, the unlock line, the two-part plan, the ladder) and never wording:
// the words are the core's, tested in Rust. Where the real data differs
// from what the plan expected, the test follows the data (the core).

const isIndexKind = (kind: Kind) => typeof kind === 'object' && 'Way' in kind
const kindsDrawn = (answers: AnswersInput) => alternativesFor(answers).series.map(({ kind }) => kind)
const rungs = (page: Page) => page.locator('section.ladder button.rung')
const rows = (page: Page) => page.locator('section.within tbody tr')

test(
  'a1: a fixed date in five years is a safe place, with the stock fund drawn beside it',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 50_000, monthly: 0, horizon: 5, flexibility: 'Fixed' })
    const guide = await expectAnswerFor(page, answers)
    expect(guide.arena).toBe('Safe')
    expect(isIndexKind(guide.kind.kind)).toBe(false)
    // The stock index fund is drawn, with the history behind it.
    expect(kindsDrawn(answers).some(isIndexKind)).toBe(true)
    expect(alternativesFor(answers).band).toBeDefined()
    await expect(page.locator('.alternatives .caption')).toBeVisible()
    // The places that hold it, ranked; the steps; the donut.
    await expect(rows(page)).toHaveCount(Math.min(3, guide.plans.length))
    await expect(page.locator('section.steps li')).toHaveCount(guide.steps.length)
    await expect(page.locator('section.donut .chart canvas')).toBeVisible()
  },
)

test(
  'a2: the same money that can wait goes to the cheapest way to hold an index',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 50_000, monthly: 0, horizon: 5, flexibility: 'CanWait' })
    const guide = await expectAnswerFor(page, answers)
    expect(guide.arena).toBe('Index')
    expect(isIndexKind(guide.kind.kind)).toBe(true)
    // The safe kinds are drawn beside it.
    expect(kindsDrawn(answers).some((kind) => !isIndexKind(kind))).toBe(true)
    await expect(page.locator('.alternatives li.row')).toHaveCount(alternativesFor(answers).series.length)
    expect(guide.band).toBeDefined()
  },
)

test('b: "I know what to buy": the S&P 500 in all its ways, with the fund\'s own cost and every chart', async ({
  page,
}) => {
  await page.getByRole('button', { name: t.iKnowWhatToBuy }).click()
  await expect(expertSwitch(page)).toBeChecked()
  await enter(page, { now: 10_000, monthly: 2000, horizon: 20 })
  await workbench(page, t.thingSp500)
  const pressed = page.getByRole('list', { name: t.waysTitle }).getByRole('button', { pressed: true })
  const selected = ways().filter(({ name }) => name)
  expect(await pressed.count()).toBeGreaterThan(1)
  expect(selected.length).toBeGreaterThanOrEqual(await pressed.count())
  const expected = compareWays({
    inputs: await inputsOnPage(page),
    ways: ways()
      .map(({ way }) => way)
      .slice(0, await pressed.count()),
  })
  expect(expected.ways.length).toBeGreaterThan(0)
  await expect(rows(page)).not.toHaveCount(0)
  // The fund's cost is a part of the breakdown; every view is offered.
  await choice(page, t.chart, t.breakdownView).click()
  await expect(
    page.getByRole('group', { name: t.compareBy }).getByRole('button', { name: t.productItself }),
  ).toBeVisible()
  for (const view of [t.lostView, t.valueView, t.breakdownView])
    await expect(
      page.getByRole('radiogroup', { name: t.chart }).getByText(view, { exact: true }),
    ).toBeVisible()
})

test(
  'c: no cushion, no date: a plan in two parts, and the ladder of five rungs',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 0, monthly: 3000, horizon: 'unknown', cushion: 17_000 })
    const guide = await expectAnswerFor(page, answers)
    const plan = guide.twoParts!
    expect(plan, 'two parts').toBeDefined()
    // Three months of the expenses, filled from the monthly amount.
    expect(plan.target).toBe(3 * 17_000)
    expect(plan.monthsToFill).toBe(Math.ceil(plan.target / 3000))
    expect(guide.followUps).toEqual(followUps(answers))
    const ladder = ladderFor(answers)
    expect(ladder).toHaveLength(5)
    await expect(rungs(page)).toHaveCount(ladder.length)
  },
)

test('d: retirement: a pension in the provident fund, and the other ways drawn', async ({ page }) => {
  const answers = await enter(page, { now: 0, monthly: 1500, horizon: { age: 40, retireAt: 60 } })
  const guide = await expectAnswerFor(page, answers)
  expect(guide.arena).toBe('Index')
  expect(guide.kind.kind).toEqual({ Way: { product: 'InvestmentGemelTrack', vehicle: 'InvestmentGemel' } })
  // The brokerage account after tax, the study fund (with who can open
  // one), the savings policy and the current account are drawn beside it,
  // whichever plans the workbench ticks.
  const drawn = guide.alternatives.map(({ kind }) => JSON.stringify(kind))
  expect(drawn).toContain(JSON.stringify('CurrentAccount'))
  for (const vehicle of ['Brokerage', 'StudyFund', 'SavingsPolicy'])
    expect(
      drawn.some((kind) => kind.includes(vehicle)),
      vehicle,
    ).toBe(true)
  expect(guide.ranked.some(({ kind }) => JSON.stringify(kind).includes('Brokerage'))).toBe(true)
  await expect(page.locator('.alternatives li.row')).toHaveCount(alternativesFor(answers).series.length)
})

test('e: seventeen years is an index, with the history and what safety costs', async ({ page }) => {
  const answers = await enter(page, { now: 10_000, monthly: 2000, horizon: 17 })
  const guide = await expectAnswerFor(page, answers)
  expect(guide.arena).toBe('Index')
  expect(guide.band).toBeDefined()
  expect(kindsDrawn(answers).some((kind) => !isIndexKind(kind))).toBe(true)
  await expect(page.locator('.alternatives .caption')).toBeVisible()
})

test(
  'T1: a bonus with no date: the ladder, and tapping 20 years asks the guide about 20 years',
  { tag: '@phone' },
  async ({ page }) => {
    await chip(page, situationName('bonus')).click()
    const bonus = situationAnswers('bonus')
    const ladder = ladderFor(bonus)
    await expect(rungs(page)).toHaveCount(ladder.length)
    // The last rung is the longest.
    const longest = ladder.at(-1)!
    await rungs(page).last().click()
    await expect(page.locator('#horizon')).toHaveValue(String(longest.months / 12))
    const twenty = { ...bonus, horizon: { Months: longest.months } }
    const guide = await expectAnswerFor(page, twenty)
    expect(guide.arena).toBe(longest.arena)
    // The plans that hold it, as the core ranks them (ties go to the plan
    // with no "may cost more" flag); the best one carries the flag if it has it.
    await setExpert(page)
    await expect(rows(page).first()).toContainText(guide.plans[0].name)
    await expect(rows(page).first().getByText('⚠')).toHaveCount(Number(Boolean(guide.plans[0].mayCostMore)))
  },
)

test('T2: thirty years of 500 a month: an index, and the line about a bigger first deposit', async ({
  page,
}) => {
  const answers = await enter(page, { now: 0, monthly: 500, horizon: 30 })
  const guide = await expectAnswerFor(page, answers)
  expect(guide.arena).toBe('Index')
  expect(guide.unlock, 'a first deposit would open a plan').toBeDefined()
  await expect(answerCard(page).locator('.unlock')).toHaveText(guide.unlockText!)
  // The provident fund for investment is among the kinds the guide ranks.
  expect(guide.ranked.some(({ kind }) => JSON.stringify(kind).includes('InvestmentGemelTrack'))).toBe(true)
})

test(
  'T3: a fixed date in three years: a safe place, the stock fund with its band, and "continue anyway"',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 100_000, monthly: 2000, horizon: 3 })
    const guide = await expectAnswerFor(page, answers)
    expect(guide.arena).toBe('Safe')
    const drawn = alternativesFor(answers)
    expect(drawn.band).toBeDefined()
    const index = drawn.series.find(({ kind }) => isIndexKind(kind))!
    await page.getByRole('button', { name: t.continueWith(index.name) }).click()
    const chosen = await expectAnswerFor(page, answers, { arena: arenaOf(index.kind), kind: index.kind })
    expect(chosen.arena).toBe('Index')
  },
)

test('T4: a portfolio at a bank: its branch plan is ticked and ranked, and a plan of your own opens simple', async ({
  page,
}) => {
  await chip(page, situationName('existing')).click()
  const dialog = page.getByRole('dialog', { name: t.whichBank })
  await expect(dialog).toBeVisible()
  await dialog.getByRole('group', { name: t.whichBank }).getByRole('button').first().click()
  const answers: AnswersInput = {
    ...situationAnswers('existing'),
    situation: { ExistingPortfolio: { bank: 'Leumi' } },
  }
  const guide = recommendation(answers)
  const yours = guide.plans.find((plan) => plan.yours)!
  expect(yours, "the bank's branch plan is among the places").toBeDefined()
  await expect(dialog.locator('ol.steps li')).toHaveCount(guide.steps.length)
  // A plan the list doesn't have opens the editor in its simple view.
  await dialog.getByRole('button', { name: t.newOwnPlan }).click()
  // Over the bank dialog, which stays open under it.
  const editor = page.locator('dialog[open]').filter({ has: page.getByRole('radiogroup', { name: t.view }) })
  await expect(editor).toBeVisible()
  await expect(editor.getByRole('radiogroup', { name: t.view }).locator('input:checked')).toHaveValue(
    'simple',
  )
  await page.keyboard.press('Escape')
  await page.keyboard.press('Escape')
  await expect(dialog).toBeHidden()
  // Their bank's plan is in the table wherever it ranks, not only among
  // the first three; and it is ticked in what's compared.
  await expect(rows(page).filter({ hasText: yours.name })).toHaveCount(1)
  const box = (await planInList(page, 'Leumi · Branch or through a banker')).getByRole('checkbox')
  await expect(box).toBeChecked()
  await closePlans(page)
})

test('T5: the defaults: amounts in today\'s money, and the current account answers "do I need to care"', async ({
  page,
}) => {
  const answers = situationAnswers('goal')
  const guide = await expectAnswerFor(page, answers)
  await expect(answerCard(page).locator('.big')).toHaveText(shekels(guide.left.today))
  await expect(answerCard(page).locator('.sub')).toContainText(guide.leftNote.today)
  await expect(answerCard(page).locator('.base')).toHaveText(guide.currentAccountLine)
  // The same deposits left in a current account are never more than what fits.
  expect(guide.currentAccount.today).toBeLessThanOrEqual(guide.left.today)
  await fillField(page, 'now', '0')
  await expect(answerCard(page).locator('.big')).toHaveText(
    shekels(recommendation({ ...answers, now: 0 }).left.today),
  )
})
