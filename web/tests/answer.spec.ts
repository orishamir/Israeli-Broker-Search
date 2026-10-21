import { answerFor, decisionsOf, situationAnswers } from './core'
import type { AnswerData, AnswersInput, Choices, DecisionData, DecisionId, LineData, Pick } from './core'
import {
  answerCard,
  answerLine,
  pickExample,
  decide,
  decisionGroup,
  expect,
  expectAnswerFor,
  situationName,
  test,
  type Locator,
  type Page,
} from './fixtures'
import { t } from '../src/lib/text'

// The answer card (REDESIGN.md §2.2–§2.4): the chip, the lines, the one
// decision component by pointer and by keyboard, and the plan once nothing
// is left to decide. What it says is the core's; here, that the page shows
// as many lines and decisions as the core gives, and that choosing does
// what the core says it does.

/** What a pick that isn't a fact about the money puts in the choices. */
function choicesWith(choices: Choices, pick: Pick): Choices {
  if ('SafeWay' in pick) return { ...choices, safeWay: pick.SafeWay }
  if ('Wrapper' in pick) return { ...choices, wrapper: pick.Wrapper }
  if ('Account' in pick) return { ...choices, account: pick.Account }
  if ('Holdings' in pick) return { ...choices, holdings: pick.Holdings }
  return choices
}

/** The first decision the core asks for these answers. */
function firstDecision(answers: AnswersInput): DecisionData {
  const first = decisionsOf(answerFor(answers))[0]
  if (!first) throw new Error('the situation asks nothing')
  return first.decision
}

/** The pill a line wears before anything is chosen. */
const pillOf = ({ state }: LineData) =>
  state.type === 'stated' ? t.linePill[state.pill] : state.type === 'waiting' ? t.pillWaiting : t.pillYourCall

/** A decision's choose buttons (on a phone, "עוד על …" is a button there too). */
const options = (page: Page, decision: DecisionData) => decisionGroup(page, decision).locator('button.choose')

test('the card shows the core’s lines, each with its pill', { tag: '@phone' }, async ({ page }) => {
  const answers = situationAnswers('goal')
  const expected = await expectAnswerFor(page, answers)
  await expect(answerCard(page).locator('.line')).toHaveCount(expected.lines.length)
  // In the core's order: a decision that depends on another comes after it
  // (principle 8), and the core decides which that is.
  expect(
    await answerCard(page)
      .locator('.line')
      .evaluateAll((lines) => lines.map((line) => (line as HTMLElement).dataset.line)),
  ).toEqual(expected.lines.map((line) => line.kind))
  for (const line of expected.lines)
    await expect(answerLine(page, line.kind).locator('.pill')).toHaveText(pillOf(line))
})

test(
  'a decision is chosen by pointer: the chosen option is pressed, the others stay, and it can be changed',
  { tag: '@phone' },
  async ({ page }) => {
    // The salary: two decisions, so one made leaves the lines (the other
    // still open), and every option of the first can be tried.
    await pickExample(page, situationName('salary'))
    const answers = situationAnswers('salary')
    await expectAnswerFor(page, answers)
    const decision = firstDecision(answers)
    const buttons = options(page, decision)
    await expect(buttons).toHaveCount(decision.options.length)
    for (const [at, option] of decision.options.entries()) {
      await decide(page, answers, {}, decision.id, option.pick)
      await expect(buttons.nth(at)).toHaveAttribute('aria-pressed', 'true')
      // The others are still there, and still choices.
      await expect(buttons).toHaveCount(decision.options.length)
      await expect(options(page, decision).and(page.getByRole('button', { pressed: false }))).toHaveCount(
        decision.options.length - 1,
      )
      await expect(answerCard(page).locator('.option.other')).toHaveCount(decision.options.length - 1)
    }
  },
)

test('a decision is chosen by keyboard, and its targets are big enough', async ({ page }) => {
  await pickExample(page, situationName('salary'))
  const answers = situationAnswers('salary')
  await expectAnswerFor(page, answers)
  // Where the money builds up: options with nothing inside them but words
  // (the cushion's "not yet" holds a field, which Tab would stop at).
  const decision = decisionsOf(answerFor(answers)).find(({ decision }) => decision.id === 'Wrapper')!.decision
  const second = options(page, decision).nth(1)
  await second.focus()
  await page.keyboard.press('Enter')
  await expect(second).toHaveAttribute('aria-pressed', 'true')
  await expect(second).toBeFocused()
  await page.keyboard.press('Shift+Tab')
  await page.keyboard.press('Space')
  await expect(options(page, decision).first()).toHaveAttribute('aria-pressed', 'true')
  for (const button of await options(page, decision).all())
    expect((await button.boundingBox())!.height).toBeGreaterThanOrEqual(44)
})

test('"one decision left" until it is made, then the plan', async ({ page }) => {
  const answers = situationAnswers('goal')
  const before = await expectAnswerFor(page, answers)
  expect(before.openDecisions).toBeGreaterThan(0)
  await expect(answerCard(page).locator('.left')).toHaveText(t.decisionsLeft(before.openDecisions))
  await expect(answerCard(page).locator('.plan')).toHaveCount(0)
  const decision = firstDecision(answers)
  const pick = decision.options[0].pick
  await decide(page, answers, {}, decision.id, pick)
  // The apartment has one decision (REDESIGN.md §2.5): made, the plan shows.
  const after = await expectAnswerFor(page, answers, choicesWith({}, pick))
  expect(after.openDecisions).toBe(0)
  await expect(answerCard(page).locator('.plan .plan-sentence')).toHaveText(after.plan!)
  await expect(answerCard(page).locator('.left')).toHaveCount(0)
})

test(
  'the monthly expenses are asked only after "not yet", and open without moving the card more than their height',
  { tag: '@phone' },
  async ({ page }) => {
    await pickExample(page, situationName('salary'))
    const answers = situationAnswers('salary')
    await expectAnswerFor(page, answers)
    const cushion = decisionsOf(answerFor(answers)).find(({ decision }) => decision.id === 'Cushion')
    expect(cushion, 'the salary asks about a cushion').toBeDefined()
    await expect(page.locator('#expenses')).toHaveCount(0)
    // Nothing above the options moves (in the page, whatever the scroll),
    // and they grow by the field's height, no more.
    const top = (locator: Locator) =>
      locator.evaluate((element) => element.getBoundingClientRect().top + scrollY)
    const height = (locator: Locator) => locator.evaluate((element) => element.getBoundingClientRect().height)
    const why = decisionGroup(page, cushion!.decision).locator('.why')
    const options = decisionGroup(page, cushion!.decision).locator('.options')
    const [whyBefore, before] = [await top(why), await height(options)]
    await decide(page, answers, {}, 'Cushion', { Cushion: 'HasNot' })
    const field = page.locator('#expenses')
    await expect(field).toBeVisible()
    const box = decisionGroup(page, cushion!.decision).locator('.expenses')
    expect(await top(why)).toBeCloseTo(whyBefore, 0)
    expect((await height(options)) - before).toBeLessThanOrEqual((await height(box)) + 16)
    await field.fill('9000')
    await expectAnswerFor(page, { ...answers, cushion: { kind: 'hasNot', monthlyExpenses: 9000 } })
    await decide(page, answers, {}, 'Cushion', { Cushion: 'Has' })
    await expect(page.locator('#expenses')).toHaveCount(0)
  },
)

/** The first decision's options that open a decision still to answer, each
 * with the decisions it opens, as the core says. */
function opening(answers: AnswersInput, before: AnswerData) {
  const first = decisionsOf(before)[0]
  const asked = new Set(decisionsOf(before).map(({ decision }) => decision.id))
  return first.decision.options.flatMap((option) => {
    const pick = option.pick
    const facts: AnswersInput =
      'Flexibility' in pick
        ? { ...answers, flexibility: pick.Flexibility }
        : 'Cushion' in pick
          ? {
              ...answers,
              cushion: pick.Cushion === 'Has' ? { kind: 'has' } : { kind: 'hasNot', monthlyExpenses: null },
            }
          : answers
    // New, and still to answer (an example may have answered one already).
    const fresh = decisionsOf(answerFor(facts, choicesWith({}, pick))).filter(
      ({ decision }) => !asked.has(decision.id) && decision.chosen === undefined,
    )
    return fresh.length > 0 ? [{ option, fresh }] : []
  })
}

/** The new decision is on the screen whole wherever it fits on one (the
 * review's N8: its question alone isn't the decision); taller, its question
 * and its first option's button are. */
async function expectDecisionShown(page: Page, group: Locator) {
  const box = (await group.boundingBox())!
  if (box.height <= page.viewportSize()!.height - 24) await expect(group).toBeInViewport({ ratio: 1 })
  else await expect(group.locator('.choose').first()).toBeInViewport({ ratio: 1 })
}

// A choice can open a decision that depends on it (the review of 2026-10-20,
// X7, N8): the chosen option stays exactly where it was while the card
// changes around it (principle 21), and the page never scrolls by itself:
// a new decision out of sight is announced by the pill, which takes the
// visitor there (principle 22; REDESIGN.md §4.3, at every width since the
// review of 2026-10-21). The phone's own taps are in `phone.spec.ts`.
test('a choice that opens a decision keeps the button still, and shows or announces the new one', async ({
  page,
}) => {
  const pill = page.locator('.dock .pill')
  let opened = 0
  for (const id of ['bonus', 'child', 'unknown']) {
    const answers = situationAnswers(id)
    await pickExample(page, situationName(id))
    const before = await expectAnswerFor(page, answers)
    const first = decisionsOf(before)[0]
    for (const { option, fresh } of opening(answers, before)) {
      opened++
      const button = decisionGroup(page, first.decision).getByRole('button', { name: option.button, exact: true })
      const at = (await button.boundingBox())!.y
      await decide(page, answers, {}, first.decision.id, option.pick)
      const pressed = decisionGroup(page, first.decision).getByRole('button', { pressed: true })
      await expect(pressed).toBeVisible()
      await expect(decisionGroup(page, fresh[0].decision)).toBeAttached()
      // Settled: the pressed button is where it was.
      await page.waitForTimeout(700)
      expect(Math.abs((await pressed.boundingBox())!.y - at)).toBeLessThan(1)
      const group = answerLine(page, fresh[0].line)
      if (!(await group.locator('.question').isVisible()) || !(await isInSight(group))) {
        await expect(pill).toHaveText(t.newDecision(fresh[0].decision.question))
        await pill.click()
        await expectDecisionShown(page, group)
        await expect(pill).toHaveCount(0)
      }
      // Back to the first, for the next option.
      await pickExample(page, situationName('goal'))
      await pickExample(page, situationName(id))
      await expectAnswerFor(page, answers)
    }
  }
  expect(opened, 'some choice opens another decision').toBeGreaterThan(0)
})

/** Some of it is on the screen, above the dock's room (scroll.ts `inSight`). */
const isInSight = (group: Locator) =>
  group.evaluate((element) => {
    const box = element.getBoundingClientRect()
    return box.top < innerHeight - 96 && box.bottom > 0
  })

// ── The plan and the path to it (REDESIGN.md §4.5) ──
// The steps' words are the core's, checked in Rust and in the unit tests;
// here, what only a browser shows: what opens and closes in place, and that
// what the visitor is using doesn't move (principle 21).

/** The worked examples (§2.5), each with every decision made, in order. */
const planned: Record<string, [DecisionId, Pick][]> = {
  goal: [['SafeWay', { SafeWay: 'Bond' }]],
  bonus: [['PlaceType', { Flexibility: 'CanWait' }]],
  salary: [
    ['Cushion', { Cushion: 'Has' }],
    ['Wrapper', { Wrapper: 'Anyone' }],
    ['Account', { Account: 'NewAccount' }],
  ],
}

/** Makes every decision of an example; returns the answers and choices. */
async function plan(page: Page, id: string) {
  await pickExample(page, situationName(id))
  let answers = situationAnswers(id)
  let choices: Choices = {}
  await expectAnswerFor(page, answers)
  for (const [decision, pick] of planned[id]) {
    await decide(page, answers, choices, decision, pick)
    if ('Flexibility' in pick) answers = { ...answers, flexibility: pick.Flexibility }
    else if ('Cushion' in pick) answers = { ...answers, cushion: { kind: 'has' } }
    else choices = choicesWith(choices, pick)
    await expectAnswerFor(page, answers, choices)
  }
  // The last choice's step is held where its option was until the card has
  // settled (`holdChosen`). A person scrolling ends that at once; a test's
  // scrolling into view doesn't, so scroll as a person would.
  await page.evaluate(() => dispatchEvent(new WheelEvent('wheel')))
  return { answers, choices, expected: answerFor(answers, choices) }
}

const step = (page: Page, key: string) => answerCard(page).locator(`.path [data-step="${key}"]`)
/** Where an element is on the screen. */
const screenTop = (locator: Locator) => locator.evaluate((element) => element.getBoundingClientRect().top)

/** A step of the path, as the page draws it: what it can do. */
const controls = (shown: Locator) =>
  shown.evaluate((element) => ({
    change: element.querySelectorAll('button.change').length,
    why: element.querySelectorAll('button.why').length,
    any: element.querySelectorAll('button, a, [aria-disabled]').length,
    filled: element.querySelector('.node')!.classList.contains('filled'),
  }))

test(
  'every decision made, the card is the plan, with a step for each of the core’s',
  { tag: '@phone' },
  async ({ page }) => {
    for (const id of Object.keys(planned)) {
      const { expected } = await plan(page, id)
      expect(expected.plan, id).toBeDefined()
      await expect(answerCard(page).locator('.line')).toHaveCount(0)
      const steps = answerCard(page).locator('.path .step')
      await expect(steps).toHaveCount(expected.path.length)
      for (const [at, each] of expected.path.entries()) {
        const shown = await controls(steps.nth(at))
        // The visitor's: "שינוי"; the site's: "למה זה ברור?" where it has one;
        // which index: an empty circle, and nothing to press, not even greyed.
        expect(shown, `${id} ${each.label}`).toEqual({
          change: each.kind === 'Visitor' ? 1 : 0,
          why: each.whyClear ? 1 : 0,
          any: each.kind === 'Visitor' || each.whyClear ? 1 : 0,
          filled: each.kind !== 'Outside',
        })
      }
      // Stocks: which index is the visitor's call (principle 17).
      expect(
        expected.path.some((each) => each.kind === 'Outside'),
        id,
      ).toBe(id !== 'goal')
      // "איך עושים את זה, צעד אחר צעד" takes the visitor there.
      await answerCard(page).locator('a.to-how').click()
      await expect(page.locator('section.how')).toBeInViewport()
    }
  },
)

test(
  '"שינוי" opens the decision in place; choosing closes it, the plan changes, and the step stays',
  { tag: '@phone' },
  async ({ page }) => {
    const { answers, choices, expected } = await plan(page, 'salary')
    const head = step(page, 'Account').locator('.step-head')
    const change = step(page, 'Account').locator('button.change')
    await change.scrollIntoViewIfNeeded()
    const before = await screenTop(head)
    await change.click()
    await expect(change).toHaveAttribute('aria-expanded', 'true')
    expect(await screenTop(head)).toBeCloseTo(before, 0)
    const decision = decisionsOf(expected).find((each) => each.decision.id === 'Account')!.decision
    await expect(step(page, 'Account').getByRole('group')).toBeVisible()
    await expect(decisionGroup(page, decision).getByRole('button', { pressed: true })).toHaveText(
      decision.options[0].button,
    )
    // The bank: the decision closes back into the step, and the plan above
    // changes while the step's head stays where it is on the screen.
    const bank = decisionGroup(page, decision).getByRole('button', {
      name: decision.options[1].button,
      exact: true,
    })
    await bank.scrollIntoViewIfNeeded()
    const pressedAt = await screenTop(head)
    await bank.click()
    await expectAnswerFor(page, answers, { ...choices, account: 'YourBank' })
    await expect(answerCard(page).getByRole('group')).toHaveCount(0)
    await expect(change).toHaveAttribute('aria-expanded', 'false')
    await expect.poll(async () => Math.abs((await screenTop(head)) - pressedAt)).toBeLessThan(2)
    await expect(answerCard(page).locator('.plan .before')).toHaveText(
      t.beforeChange(expected.planAmount!.amount),
    )
  },
)

test(
  '"למה זה ברור?" opens its explanation in place, and nothing above it moves',
  { tag: '@phone' },
  async ({ page }) => {
    const { expected } = await plan(page, 'salary')
    const clear = expected.path.find((each) => each.whyClear)!
    const button = step(page, clear.line!).locator('button.why')
    await button.scrollIntoViewIfNeeded()
    const before = await screenTop(button)
    await button.click()
    await expect(button).toHaveAttribute('aria-expanded', 'true')
    const why = step(page, clear.line!).locator('.why-clear')
    await expect(why).toBeVisible()
    expect(await screenTop(button)).toBeCloseTo(before, 0)
    await button.click()
    await expect(why).toHaveCount(0)
  },
)

test(
  'a change on the path that opens a decision keeps the path, the step still',
  { tag: '@phone' },
  async ({ page }) => {
    const { answers } = await plan(page, 'bonus')
    const head = step(page, 'PlaceType').locator('.step-head')
    await step(page, 'PlaceType').locator('button.change').click()
    const fixed = step(page, 'PlaceType').locator('[role=group] .choose').first()
    await fixed.scrollIntoViewIfNeeded()
    const before = await screenTop(head)
    await fixed.click()
    const safe = await expectAnswerFor(page, { ...answers, flexibility: 'Fixed' })
    expect(safe.plan).toBeUndefined()
    await expect(answerCard(page).locator('.line')).toHaveCount(0)
    const open = safe.path.find((each) => each.kind === 'Open')!
    await expect(step(page, open.reopens!).getByRole('group')).toBeVisible()
    await expect.poll(async () => Math.abs((await screenTop(head)) - before)).toBeLessThan(2)
  },
)
