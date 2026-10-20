import { answerFor, decisionsOf, situationAnswers } from './core'
import type { AnswerData, AnswersInput, Choices, DecisionData, LineData, Pick } from './core'
import {
  answerCard,
  answerLine,
  chip,
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

const options = (page: Page, decision: DecisionData) => decisionGroup(page, decision).getByRole('button')

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
    const answers = situationAnswers('goal')
    await expectAnswerFor(page, answers)
    const decision = firstDecision(answers)
    const buttons = options(page, decision)
    await expect(buttons).toHaveCount(decision.options.length)
    for (const [at, option] of decision.options.entries()) {
      await decide(page, answers, {}, decision.id, option.pick)
      await expectAnswerFor(page, answers, choicesWith({}, option.pick))
      await expect(buttons.nth(at)).toHaveAttribute('aria-pressed', 'true')
      // The others are still there, and still choices.
      await expect(buttons).toHaveCount(decision.options.length)
      await expect(decisionGroup(page, decision).getByRole('button', { pressed: false })).toHaveCount(
        decision.options.length - 1,
      )
      await expect(answerCard(page).locator('.option.other')).toHaveCount(decision.options.length - 1)
    }
  },
)

test('a decision is chosen by keyboard, and its targets are big enough', async ({ page }) => {
  const answers = situationAnswers('goal')
  await expectAnswerFor(page, answers)
  const decision = firstDecision(answers)
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
  await expect(answerCard(page).locator('.plan p')).toHaveText(after.plan!)
  await expect(answerCard(page).locator('.left')).toHaveCount(0)
})

test(
  'the monthly expenses are asked only after "not yet", and open without moving the card more than their height',
  { tag: '@phone' },
  async ({ page }) => {
    await chip(page, situationName('salary')).click()
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
    const box = page.locator('.option .expenses')
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

/** The chosen button stays on the screen with the new question, wherever
 * the two fit on one (on a phone, what lies between them can be taller). */
async function expectChosenKeptWhereItFits(page: Page, pressed: Locator, question: Locator) {
  const [button, asked] = [(await pressed.boundingBox())!, (await question.boundingBox())!]
  if (asked.y + asked.height - button.y <= page.viewportSize()!.height - 24)
    await expect(pressed).toBeInViewport()
}

// A choice can open a decision that depends on it (the review of 2026-10-20,
// X7): the chosen option stays on the screen while the card changes around
// it, and the new question comes into view, on a phone too.
test(
  'a choice that opens a decision keeps the chosen option on the screen, and shows the new question',
  { tag: '@phone' },
  async ({ page }) => {
    let opened = 0
    for (const id of ['bonus', 'child', 'unknown']) {
      const answers = situationAnswers(id)
      await chip(page, situationName(id)).click()
      const before = await expectAnswerFor(page, answers)
      const first = decisionsOf(before)[0]
      for (const { option, fresh } of opening(answers, before)) {
        opened++
        await decide(page, answers, {}, first.decision.id, option.pick)
        const pressed = decisionGroup(page, first.decision).getByRole('button', { pressed: true })
        await expect(pressed).toBeVisible()
        await expect(decisionGroup(page, fresh[0].decision)).toBeVisible()
        await expect(pressed).toBeInViewport()
        const question = answerLine(page, fresh[0].line).locator('.question')
        await expect(question).toBeInViewport({ ratio: 1 })
        await expectChosenKeptWhereItFits(page, pressed, question)
        // Back to the first, for the next option.
        await chip(page, situationName('goal')).click()
        await chip(page, situationName(id)).click()
        await expectAnswerFor(page, answers)
      }
    }
    expect(opened, 'some choice opens another decision').toBeGreaterThan(0)
  },
)
