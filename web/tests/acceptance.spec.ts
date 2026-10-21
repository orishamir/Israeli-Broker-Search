import { answersWith, compareWays, decisionsOf, inputsOnPage, situationAnswers, ways } from './core'
import type { AnswerData, AnswersInput, Choices, DecisionId, Holdings, LineKind, Pick } from './core'
import {
  answerCard,
  answerLine,
  pickExample,
  choice,
  decide,
  decisionGroup,
  enter,
  expect,
  expectAnswerFor,
  expertSwitch,
  situationName,
  test,
  workbench,
  type Page,
} from './fixtures'
import { t } from '../src/lib/text'

// REDESIGN.md §2.5's three worked examples, end to end, by their structure:
// which decisions appear and on which line, which don't, their options, the
// order of the decisions that depend on others, and the confidence state.
// Never by their amounts or a winner: those change with every month's data,
// and are the core's (crates/core/tests). Each example says the structure
// §2.5 describes as a rule, checks the core keeps it, and checks the page
// shows exactly what the core says. Where the data differs from §2.5, the
// test follows the data and says so.

type Shape = Record<LineKind, 'stated' | 'waiting' | DecisionId | undefined>

/** An answer's structure: each line's state, a decision by its id. */
function shapeOf(data: AnswerData): Shape {
  const shape: Shape = { Before: undefined, PlaceType: undefined, Way: undefined, Where: undefined }
  for (const line of data.lines)
    shape[line.kind] = line.state.type === 'decision' ? line.state.decision.id : line.state.type
  return shape
}

/** The options a decision offers, as picks, in the core's order. */
const optionsOf = (data: AnswerData, id: DecisionId): Pick[] =>
  decisionsOf(data)
    .find(({ decision }) => decision.id === id)!
    .decision.options.map(({ pick }) => pick)

/** The plan's path shows a step for each of the core's, in its order. */
async function expectPathOnPage(page: Page, expected: AnswerData) {
  const steps = answerCard(page).locator('.path .step')
  await expect(steps).toHaveCount(expected.path.length)
  for (const [at, step] of expected.path.entries())
    await expect(steps.nth(at).locator('.choice')).toHaveText(step.choice)
  return expected
}

/** The page shows the core's answer line by line: a decision as its group,
 * a waiting line as waiting, a stated line as the core's sentence. */
async function expectShapeOnPage(page: Page, answers: AnswersInput, choices: Choices = {}) {
  const expected = await expectAnswerFor(page, answers, choices)
  // Every decision made, the card is the plan, and the lines are the
  // path's steps, one for each in its order (§4.5).
  if (expected.plan !== undefined) return expectPathOnPage(page, expected)
  await expect(answerCard(page).locator('.line')).toHaveCount(expected.lines.length)
  for (const line of expected.lines) {
    const shown = answerLine(page, line.kind)
    if (line.state.type === 'decision') {
      await expect(
        shown.getByRole('group', { name: line.state.decision.question, exact: true }),
      ).toBeVisible()
      await expect(decisionGroup(page, line.state.decision).locator('.option')).toHaveCount(
        line.state.decision.options.length,
      )
    } else if (line.state.type === 'waiting') {
      await expect(shown.locator('.said')).toHaveText(t.lineWaiting)
    } else {
      await expect(shown.locator('.said')).toHaveText(line.state.text)
    }
  }
  return expected
}

test(
  'the apartment in 3 years: the way is the visitor’s call, the place is said, and nothing asks about the date or an account',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 100_000, monthly: 2000, horizon: 3 })
    const first = await expectShapeOnPage(page, answers)
    // §2.5: state B; the place stated; the way a decision; where it waits.
    expect(first.confidence.state).toBe('B')
    expect(shapeOf(first)).toEqual({
      Before: undefined,
      PlaceType: 'stated',
      Way: 'SafeWay',
      Where: 'waiting',
    })
    // Two options, the fund and the bond; no question about the date
    // (principle 2) or about having an account (principle 5).
    expect(optionsOf(first, 'SafeWay')).toEqual(
      expect.arrayContaining([{ SafeWay: 'MoneyMarketFund' }, { SafeWay: 'Bond' }]),
    )
    expect(optionsOf(first, 'SafeWay')).toHaveLength(2)
    // Where follows the choice, either way, and can be changed back.
    for (const safeWay of ['Bond', 'MoneyMarketFund'] as const) {
      await decide(page, answers, {}, 'SafeWay', { SafeWay: safeWay })
      const chosen = await expectShapeOnPage(page, answers, { safeWay })
      expect(shapeOf(chosen)).toEqual({ ...shapeOf(first), Where: 'stated' })
    }
  },
)

test(
  'the bonus in 5 years: safe or growth comes first, and the safe ways only after it',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 15_000, monthly: 0, horizon: 5 })
    const first = await expectShapeOnPage(page, answers)
    // §2.5: state C; the place is the deciding question; the rest waits on it.
    expect(first.confidence.state).toBe('C')
    expect(shapeOf(first)).toEqual({
      Before: undefined,
      PlaceType: 'PlaceType',
      Way: 'waiting',
      Where: 'waiting',
    })
    expect(optionsOf(first, 'PlaceType')).toEqual([{ Flexibility: 'Fixed' }, { Flexibility: 'CanWait' }])

    // Safe: which safe way is a decision of three.
    await decide(page, answers, {}, 'PlaceType', { Flexibility: 'Fixed' })
    const fixed = { ...answers, flexibility: 'Fixed' as const }
    const safe = await expectShapeOnPage(page, fixed)
    expect(shapeOf(safe)).toEqual({ ...shapeOf(first), Way: 'SafeWay' })
    expect(optionsOf(safe, 'SafeWay')).toHaveLength(3)

    // Growth: the way is said. §2.5 states where too; the data makes it a
    // decision between a new account and the visitor's bank (the 30-year
    // example's), so the test follows the core: whatever it is, it comes
    // only after the place is chosen.
    await decide(page, fixed, {}, 'PlaceType', { Flexibility: 'CanWait' })
    const wait = { ...answers, flexibility: 'CanWait' as const }
    const growth = await expectShapeOnPage(page, wait)
    expect(shapeOf(growth).Way).toBe('stated')
    expect(shapeOf(growth).PlaceType).toBe('PlaceType')
    // No decision about an account before the place was chosen.
    expect(decisionsOf(first).some(({ decision }) => decision.id === 'Account')).toBe(false)
  },
)

test(
  '500 a month for 30 years: a cushion first, the index said, where the money builds up asked, then the account',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, { now: 0, monthly: 500, horizon: 30 })
    const first = await expectShapeOnPage(page, answers)
    // §2.5: state A; the cushion first; the index stated; where it builds up asked.
    expect(first.confidence.state).toBe('A')
    expect(shapeOf(first)).toEqual({
      Before: 'Cushion',
      PlaceType: 'stated',
      Way: 'Wrapper',
      Where: 'waiting',
    })
    expect(optionsOf(first, 'Wrapper')).toEqual([{ Wrapper: 'Anyone' }, { Wrapper: 'StudyFund' }])
    // Only state A wears the accent.
    await expect(answerCard(page)).toHaveClass(/state-A/)

    // An ETF: then a new account or the visitor's bank.
    await decide(page, answers, {}, 'Wrapper', { Wrapper: 'Anyone' })
    const etf = await expectShapeOnPage(page, answers, { wrapper: 'Anyone' })
    expect(shapeOf(etf)).toEqual({ ...shapeOf(first), Where: 'Account' })
    expect(optionsOf(etf, 'Account')).toEqual([{ Account: 'NewAccount' }, { Account: 'YourBank' }])
    // The study fund: where is said, and the account question goes.
    await decide(page, answers, { wrapper: 'Anyone' }, 'Wrapper', { Wrapper: 'StudyFund' })
    const study = await expectShapeOnPage(page, answers, { wrapper: 'StudyFund' })
    expect(shapeOf(study)).toEqual({ ...shapeOf(first), Where: 'stated' })
  },
)

test('a portfolio at a bank: nothing opens, the place is said, and what it holds is asked on the way', async ({
  page,
}) => {
  await pickExample(page, situationName('existing'))
  await expect(page.getByRole('dialog')).toHaveCount(0)
  const answers = situationAnswers('existing')
  const first = await expectShapeOnPage(page, answers)
  // The money is already invested: the place's type isn't asked again, and
  // what the portfolio holds is the visitor's to say, never assumed (BUT-6).
  expect(shapeOf(first).PlaceType).toBe('stated')
  expect(decisionsOf(first).find(({ decision }) => decision.id === 'Holdings')?.line).toBe('Way')
  // Where waits for it; once said, the move is stated, and the choice stays.
  expect(shapeOf(first).Where).toBe('waiting')
  for (const pick of optionsOf(first, 'Holdings')) {
    await decide(page, answers, {}, 'Holdings', pick)
    const told = await expectShapeOnPage(page, answers, {
      holdings: (pick as { Holdings: Holdings }).Holdings,
    })
    expect(shapeOf(told)).toEqual({ ...shapeOf(first), Where: 'stated' })
  }
})

test('retirement: the answer says the monthly pension the core estimates, where the core puts it', async ({
  page,
}) => {
  await pickExample(page, situationName('retirement'))
  const answers = situationAnswers('retirement')
  const first = await expectAnswerFor(page, answers)
  // Where the core says it: the growth option's second amount while the
  // place type is the visitor's (chosen or not), on the card otherwise; and
  // the page shows it once (the review's N7).
  const placeType = decisionsOf(first).find(({ decision }) => decision.id === 'PlaceType')
  const growing = placeType?.decision.options.find(
    ({ pick }) => JSON.stringify(pick) === '{"Flexibility":"CanWait"}',
  )
  const pension = growing?.also ?? first.annuity
  expect(pension, 'the core estimates a pension').toBeDefined()
  await expect(answerCard(page).getByText(pension!, { exact: true })).toHaveCount(1)
  await expect(answerCard(page).getByText(pension!, { exact: true })).toBeVisible()
  await expect(answerCard(page).locator('.annuity')).toHaveCount(Number(first.annuity !== undefined))
})

test('"I know what to buy": the S&P 500 in all its ways, with the fund\'s own cost and every chart', async ({
  page,
}) => {
  await page.getByRole('button', { name: t.iKnowWhatToBuy }).click()
  await expect(expertSwitch(page)).toBeChecked()
  await enter(page, { now: 10_000, monthly: 2000, horizon: 20 })
  await workbench(page, t.thingSp500)
  const pressed = page.getByRole('list', { name: t.waysTitle }).getByRole('button', { pressed: true })
  expect(await pressed.count()).toBeGreaterThan(1)
  const expected = compareWays({
    inputs: await inputsOnPage(page),
    ways: ways()
      .map(({ way }) => way)
      .slice(0, await pressed.count()),
  })
  expect(expected.ways.length).toBeGreaterThan(0)
  await expect(page.locator('section.within tbody tr')).not.toHaveCount(0)
  await choice(page, t.chart, t.breakdownView).click()
  await expect(
    page.getByRole('group', { name: t.compareBy }).getByRole('button', { name: t.productItself }),
  ).toBeVisible()
  for (const view of [t.lostView, t.valueView, t.breakdownView])
    await expect(
      page.getByRole('radiogroup', { name: t.chart }).getByText(view, { exact: true }),
    ).toBeVisible()
})

test('nothing in the guide is a row of assumptions, and every decision is the one component', async ({
  page,
}) => {
  for (const id of ['goal', 'bonus', 'salary', 'existing']) {
    await pickExample(page, situationName(id))
    const expected = await expectAnswerFor(page, answersWith(situationAnswers(id)))
    // Every group on the answer card is a decision the core asks.
    await expect(answerCard(page).locator('[role=group]')).toHaveCount(decisionsOf(expected).length)
    await expect(answerCard(page).locator('[role=group].decision')).toHaveCount(decisionsOf(expected).length)
  }
})
