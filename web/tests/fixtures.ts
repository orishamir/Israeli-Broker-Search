import { expect, test as base, type Locator, type Page } from '@playwright/test'
import { dateText, shekels } from '../src/lib/format'
import { t } from '../src/lib/text'
import { answersWith, followCard, listed, RATES, recommendation, situations } from './core'
import type { AnswersInput, Overrides } from './core'

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
    await expect(page.locator('.alternatives .chart canvas')).toBeVisible()
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
 * Waits for the exchange rates, which the table's footnote dates once downloaded. */
export async function workbench(page: Page, thing: string = t.thingOther) {
  await setExpert(page)
  await choice(page, t.whatToBuy, thing).click()
  if (thing === t.thingOther)
    await expect(page.locator('section.within .dates')).toContainText(dateText(RATES.date))
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

/** Drags a real finger (touch events through the debugging protocol, which
 * `tap()` can't do) from a point on `target` by `dx`, `dy` pixels, in ten
 * steps. Chromium only. */
export async function fingerDrag(page: Page, target: Locator, at: number, dx: number, dy: number) {
  const box = (await target.boundingBox())!
  const [x, y] = [box.x + box.width * at, box.y + box.height / 2]
  const cdp = await page.context().newCDPSession(page)
  const touch = (type: string, touchPoints: { x: number; y: number }[]) =>
    cdp.send('Input.dispatchTouchEvent', { type: type as 'touchStart', touchPoints })
  await touch('touchStart', [{ x, y }])
  for (let step = 1; step <= 10; step++) {
    await touch('touchMove', [{ x: x + (dx * step) / 10, y: y + (dy * step) / 10 }])
  }
  await touch('touchEnd', [])
}

// ─────────────────────────── The guide ───────────────────────────

/** The answer card. */
export const answerCard = (page: Page) => page.locator('section.answer')

/** The page shows what the core recommends for `answers`: the kind named in
 * the sentence and, when it is a single sum, the amount that is left. Waits
 * for the worker's answer on long horizons. */
export async function expectAnswerFor(page: Page, answers: AnswersInput, overrides?: Overrides) {
  const expected = recommendation(answers, overrides)
  const card = answerCard(page)
  // The sentence names the kind in the bank's words, bold where the core says.
  const sentence = [...expected.what, ...expected.whatThen].map(({ text }) => text).join('')
  await expect(card.locator('.what p').first()).toContainText(sentence)
  if (!expected.twoParts) await expect(card.locator('.big')).toHaveText(shekels(expected.left.today))
  return expected
}

/** What a person types into the questions. */
export interface Entered {
  now: number | null
  monthly: number | null
  /** Whole years; or `'unknown'`, or the ages for retirement. */
  horizon: number | 'unknown' | { age: number; retireAt: number }
  flexibility?: 'Fixed' | 'CanWait'
  /** `'has'`, or the month's expenses of someone who has no cushion. */
  cushion?: 'has' | number
}

const fieldText = (value: number | null) => (value === null ? '' : String(value))

/** Types `entered` into the questions, as a person would (from the first
 * chip's page), and returns the answers it stands for, as the core takes them. */
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
    await page.locator('.money').getByRole('button', { name: t.dontKnowYet }).click()
  } else {
    await fillField(page, 'horizon', String(horizon))
    answers = { ...answers, horizon: { Months: horizon * 12 } }
  }
  if (entered.flexibility) {
    const card = followCard('Flexibility')
    await page
      .locator('section.follow')
      .filter({ hasText: card.title })
      .getByRole('button', { name: card.answers[entered.flexibility === 'Fixed' ? 0 : 1] })
      .click()
    answers = { ...answers, flexibility: entered.flexibility }
  }
  if (entered.cushion !== undefined) {
    const card = followCard('Cushion')
    const has = entered.cushion === 'has'
    await page
      .locator('section.follow')
      .filter({ hasText: card.title })
      .getByRole('button', { name: card.answers[has ? 0 : 1] })
      .click()
    if (!has) await page.locator('#expenses').fill(String(entered.cushion))
    answers = {
      ...answers,
      cushion: has ? { kind: 'has' } : { kind: 'hasNot', monthlyExpenses: entered.cushion as number },
    }
  }
  return answers
}

/** A situation's name by its id, for its chip. */
export const situationName = (id: string) => situations().find((each) => each.id === id)!.name
