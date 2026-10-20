import { expect, test as base, type Locator, type Page } from '@playwright/test'
import { dateText } from '../src/lib/format'
import { t } from '../src/lib/text'
import { answerFor, answersWith, decisionsOf, listed, RATES, situations } from './core'
import type { AnswersInput, Choices, DecisionData, DecisionId, LineKind, Pick } from './core'

export { expect, type Locator, type Page }

/** Every test starts on the app, with fixed exchange rates instead of
 * today's, so results don't change daily. It starts once the app is ready:
 * the page loads before the WebAssembly does, and a half-drawn page is still
 * enough for a screenshot to look settled. Any error thrown in the page
 * fails the test. */
export const test = base.extend({
  page: async ({ page }, use) => {
    const errors: string[] = []
    page.on('pageerror', (error) => errors.push(error.message))
    // What the app makes of this is RATES.
    await page.route('https://api.frankfurter.dev/**', (route) =>
      route.fulfill({ json: { date: RATES.date, rates: { ILS: 3.4594, USD: 1.1403 } } }),
    )
    await page.goto('/')
    await expect(page.locator('section.answer #answer-title')).toBeVisible()
    await use(page)
    expect(errors, 'errors thrown in the page').toEqual([])
  },
})

/** The switch under the title, כל האפשרויות. */
export const expertSwitch = (page: Page) => page.getByRole('switch', { name: t.allOptions })

/** Turns the switch on or off by its label, as a finger does: the input
 * itself is hidden behind the track. */
export async function setExpert(page: Page, on = true) {
  if ((await expertSwitch(page).isChecked()) !== on) await page.locator('label.expert-switch').click()
  await expect(expertSwitch(page)).toBeChecked({ checked: on })
}

/** The workbench, with the switch on and `thing` chosen: by default another
 * index or a stock, whose plans on one exchange are what most specs compare
 * (the S&P 500 compares ways, bonds and short-term money the guide's kinds).
 * Waits for the exchange rates, which the table's footnote dates once
 * downloaded, and for the horizons above the table: they come from the
 * worker after the answer, and their rows push the table down when they do
 * (a hover there would find another row under the mouse). */
export async function workbench(page: Page, thing: string = t.thingOther) {
  await setExpert(page)
  await choice(page, t.whatToBuy, thing).click()
  if (thing === t.thingOther)
    await expect(page.locator('section.within .dates')).toContainText(dateText(RATES.date))
  await expect(page.locator('section.horizons[aria-busy="true"]')).toHaveCount(0)
}

/** Types into one of the money fields (`now`, `monthly`, `horizon`). */
export async function fillField(page: Page, id: 'now' | 'monthly' | 'horizon', value: string) {
  await page.locator(`#${id}`).fill(value)
}

/** A situation chip, by the name the core gives it. */
export const chip = (page: Page, name: string) =>
  page.getByRole('group', { name: t.whatForTitle }).getByRole('button', { name, exact: true })

/** The results table's rows, best first. */
export const rows = (page: Page) => page.locator('tbody tr')

/** The row of the plan called `name` (its plan name, not its broker's). */
export const row = (page: Page, name: string) =>
  rows(page).filter({ has: page.getByText(name, { exact: true }) })

/** A security, exchange, chart or editor view button, by its group and name. */
export const choice = (within: Page | Locator, group: string, name: string) =>
  within.getByRole('radiogroup', { name: group }).getByText(name, { exact: true })

/** The list of plans to compare, a dialog over the inputs, opened by "Add
 * or remove". While it's open the rest of the page can't be used. */
export const plansList = (page: Page) => page.getByRole('dialog', { name: t.whatsCompared })

/** The details dialog (`DetailsDialog`): a plan's or broker's details, the
 * editor, the page about the numbers. It may open over the list of plans. */
export const details = (page: Page) =>
  page.getByRole('dialog').filter({ hasNot: page.getByRole('heading', { name: t.whatsCompared }) })

/** Opens the list of plans, unless it's open. */
export async function openPlans(page: Page) {
  const open = page.getByRole('button', { name: t.addOrRemove })
  if (!(await plansList(page).isVisible())) {
    if (!(await open.isVisible())) await workbench(page)
    await open.click()
  }
  await expect(plansList(page)).toBeVisible()
}

/** Closes the list of plans with Esc, and waits until it's gone. */
export async function closePlans(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(plansList(page)).toBeHidden()
}

/** A broker's tick, for all its plans, in the list of plans, which it opens. */
export async function brokerTick(page: Page, name: string): Promise<Locator> {
  await openPlans(page)
  return plansList(page).getByRole('checkbox', { name, exact: true })
}

/** The plan called `englishLabel` in the list of plans, which it opens,
 * with the plan's broker unfolded: its tick, ✎ and ℹ are in it. */
export async function planInList(page: Page, englishLabel: string): Promise<Locator> {
  const { broker, plan } = listed(englishLabel)
  await openPlans(page)
  const line = plansList(page).getByRole('button', { name: broker.name, exact: true })
  if ((await line.getAttribute('aria-expanded')) === 'false') await line.click()
  return plansList(page)
    .getByRole('list', { name: broker.name })
    .getByRole('listitem')
    .filter({ has: page.getByRole('checkbox', { name: plan.name, exact: true }) })
}

/** Ticks, or unticks, all the plans of each broker named, in the list of
 * plans, and closes it. */
export async function tickBrokers(page: Page, names: string[], ticked = true) {
  await openPlans(page)
  for (const name of names)
    await plansList(page).getByRole('checkbox', { name, exact: true }).setChecked(ticked)
  await closePlans(page)
}

/** Ticks, or unticks, the plan called `englishLabel` in the list of plans,
 * and closes it. */
export async function tickPlan(page: Page, englishLabel: string, ticked = true) {
  await (await planInList(page, englishLabel)).getByRole('checkbox').setChecked(ticked)
  await closePlans(page)
}

/** Opens the plan called `englishLabel`'s details with its ℹ, in the list
 * of plans: over it, which stays open under them. */
export async function aboutPlan(page: Page, englishLabel: string) {
  const plan = await planInList(page, englishLabel)
  await plan.getByRole('button', { name: t.about(listed(englishLabel).label), exact: true }).click()
  await expect(details(page)).toBeVisible()
}

/** Opens a copy of the plan called `englishLabel` in the editor with its
 * ✎, in the list of plans: over it, which stays open under the editor. */
export async function copyPlan(page: Page, englishLabel: string) {
  const plan = await planInList(page, englishLabel)
  await plan.getByRole('button', { name: t.changeACopy(listed(englishLabel).label), exact: true }).click()
  await expect(details(page)).toBeVisible()
}

/** Moves the mouse to the corner, so no hover tip is open: before Esc in a
 * dialog, or Esc closes the tip instead. */
export const away = (page: Page) => page.mouse.move(0, 0)

/** Closes the open dialog (not the list of plans under it) with Esc, and
 * waits until it's gone. */
export async function closeDialog(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(details(page)).toBeHidden()
}

/** A picture of the chart's canvas, to compare before and after a change
 * without a stored baseline: zooming must change it, resetting restore it. */
export const canvasPicture = (chart: Locator) =>
  chart
    .locator('canvas')
    .first()
    .evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL())

/** Records every text drawn on a canvas from now on (what a chart draws,
 * which a picture can't read); the function returned lists them. */
export async function recordDrawnText(page: Page) {
  await page.evaluate(() => {
    const drawn: string[] = []
    Object.assign(window, { drawn })
    const fill = CanvasRenderingContext2D.prototype.fillText
    CanvasRenderingContext2D.prototype.fillText = function (text, ...rest) {
      drawn.push(text)
      fill.call(this, text, ...rest)
    }
  })
  return () => page.evaluate(() => (window as unknown as { drawn: string[] }).drawn)
}

// ─────────────────────────── The guide ───────────────────────────

/** The answer card. */
export const answerCard = (page: Page) => page.locator('section.answer')

/** The answer card's line of `kind` ("Before", "PlaceType", "Way", "Where"). */
export const answerLine = (page: Page, kind: LineKind) =>
  answerCard(page).locator(`.line[data-line="${kind}"]`)

/** The decision group of `decision`, labelled by its question. */
export const decisionGroup = (page: Page, decision: DecisionData) =>
  answerCard(page).getByRole('group', { name: decision.question, exact: true })

/** The page shows the core's answer for `answers` and `choices`: its chip,
 * its title, and as many decisions as the core asks. Waits for the worker's
 * answer on long horizons. */
export async function expectAnswerFor(page: Page, answers: AnswersInput, choices: Choices = {}) {
  const expected = answerFor(answers, choices)
  const card = answerCard(page)
  await expect(card.locator('#answer-title')).toHaveText(expected.title)
  await expect(card.locator('.confidence .chip')).toHaveText(t.confidenceChip[expected.confidence.state])
  await expect(card.getByRole('group')).toHaveCount(decisionsOf(expected).length)
  return expected
}

/** Chooses `pick` in the answer's decision `id`, by its button, as the core
 * words it for `answers` and `choices`; returns the decision. */
export async function decide(
  page: Page,
  answers: AnswersInput,
  choices: Choices,
  id: DecisionId,
  pick: Pick,
): Promise<DecisionData> {
  const found = decisionsOf(answerFor(answers, choices)).find(({ decision }) => decision.id === id)
  if (!found) throw new Error(`the answer asks no ${id}`)
  const option = found.decision.options.find((each) => JSON.stringify(each.pick) === JSON.stringify(pick))
  if (!option) throw new Error(`${id} has no option ${JSON.stringify(pick)}`)
  await decisionGroup(page, found.decision).getByRole('button', { name: option.button, exact: true }).click()
  return found.decision
}

/** What a person types into the questions. */
export interface Entered {
  now: number | null
  monthly: number | null
  /** Whole years; or `'unknown'`, or the ages for retirement. */
  horizon: number | 'unknown' | { age: number; retireAt: number }
  /** Answered in the answer's decision about the place, where it asks it. */
  flexibility?: 'Fixed' | 'CanWait'
  /** `'has'`, or the month's expenses of someone who has no cushion:
   * answered in the answer's cushion decision. */
  cushion?: 'has' | number
}

const fieldText = (value: number | null) => (value === null ? '' : String(value))

/** Types `entered` into the questions, as a person would (from the first
 * chip's page), and answers the decisions it names in the answer; returns
 * the answers it stands for, as the core takes them. */
export async function enter(page: Page, entered: Entered): Promise<AnswersInput> {
  const { horizon } = entered
  const retirement = typeof horizon === 'object'
  if (retirement) await chip(page, situationName('retirement')).click()
  await fillField(page, 'now', fieldText(entered.now))
  await fillField(page, 'monthly', fieldText(entered.monthly))
  let answers = answersWith({ now: entered.now, monthly: entered.monthly })
  if (retirement) {
    await page.locator('#age').fill(String(horizon.age))
    await page.locator('#retire-at').fill(String(horizon.retireAt))
    answers = { ...answers, horizon: { Retirement: { age: horizon.age, retire_at: horizon.retireAt } } }
  } else if (horizon === 'unknown') {
    await page.locator('#horizon').fill('')
  } else {
    await fillField(page, 'horizon', String(horizon))
    answers = { ...answers, horizon: { Months: horizon * 12 } }
  }
  if (entered.cushion !== undefined) {
    const has = entered.cushion === 'has'
    await expectAnswerFor(page, answers)
    await decide(page, answers, {}, 'Cushion', { Cushion: has ? 'Has' : 'HasNot' })
    if (!has) await page.locator('#expenses').fill(String(entered.cushion))
    answers = {
      ...answers,
      cushion: has ? { kind: 'has' } : { kind: 'hasNot', monthlyExpenses: entered.cushion as number },
    }
  }
  if (entered.flexibility) {
    await expectAnswerFor(page, answers)
    await decide(page, answers, {}, 'PlaceType', { Flexibility: entered.flexibility })
    answers = { ...answers, flexibility: entered.flexibility }
  }
  return answers
}

/** A situation's name by its id, for its chip. */
export const situationName = (id: string) => situations().find((each) => each.id === id)!.name
