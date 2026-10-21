import { answerFor, answersWith, banks, decisionsOf, exampleSituations, situationAnswers } from './core'
import type { AnswersInput, Choices } from './core'
import {
  choice,
  decide,
  examplePill,
  expect,
  expectAnswerFor,
  fillField,
  pickExample,
  situationName,
  test,
  when,
  type Page,
} from './fixtures'
import { fieldsOf, numbersLine } from '../src/lib/examples'
import { shown } from '../src/lib/horizon'
import { kindKey } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// The questions card (REDESIGN.md §4.2): the money, "מתי" as four answers
// over one box of fixed height, the bank, and the examples behind
// "דוגמאות" with the status line. They ask only the visitor's facts: what
// only the visitor can decide is asked in the answer, where it decides.
// What the answer is for each is the core's (`answer`); here, that the page
// asks the core what the fields say, and that nothing moves while it's used
// (principles 19, 21 and 22).

/** The questions ask nothing but the facts: "מתי" and the date's unit
 * (while a date is asked) are their only groups, and no follow-up card is left. */
async function expectOnlyFacts(page: Page) {
  const questions = page.locator('.questions-column')
  await expect(
    questions
      .getByRole('radiogroup')
      .and(
        questions
          .getByRole('radiogroup', { name: t.whenTitle })
          .or(questions.getByRole('radiogroup', { name: t.timeUnit })),
      ),
  ).toHaveCount(await questions.getByRole('radiogroup').count())
  await expect(questions.getByRole('radiogroup', { name: t.whenTitle })).toHaveCount(1)
  await expect(page.locator('section.follow')).toHaveCount(0)
}

const moneyOnPage = async (page: Page) => ({
  now: await page.locator('#now').inputValue(),
  monthly: await page.locator('#monthly').inputValue(),
})

const examplesButton = (page: Page) =>
  page.locator('section.questions').getByRole('button', { name: t.examplesButton, exact: true })
const list = (page: Page) => page.locator('dialog.examples')
const status = (page: Page) => page.locator('section.questions .status')
const backButton = (page: Page) => status(page).getByRole('button', { name: t.backToYours })

test(
  'every example fills the fields, is ticked in the list and named in the status line, and the answer follows',
  { tag: '@phone' },
  async ({ page }) => {
    for (const situation of exampleSituations()) {
      await pickExample(page, situation.name)
      await expect(examplePill(page)).toHaveText(t.exampleOf(situation.name))
      const { now, monthly } = situationAnswers(situation.id)
      await expect(page.locator('#now')).toHaveValue(now === null ? '' : now.toLocaleString('en-US'))
      await expect(page.locator('#monthly')).toHaveValue(
        monthly === null ? '' : monthly.toLocaleString('en-US'),
      )
      await expectOnlyFacts(page)
      await expectAnswerFor(page, situationAnswers(situation.id))
      // In the list: its numbers, ticked as the one on screen, and no other.
      await examplesButton(page).click()
      const item = list(page).getByRole('button', { name: new RegExp(`^${situation.name}`) })
      await expect(item).toContainText(numbersLine(fieldsOf(situation.answers)))
      await expect(item).toHaveAttribute('aria-current', 'true')
      await expect(list(page).locator('[aria-current="true"]')).toHaveCount(1)
      await page.keyboard.press('Escape')
      await expect(list(page)).toBeHidden()
    }
  },
)

test('the list floats over the fields, takes no room, and closes beside it', async ({ page }) => {
  const card = page.locator('section.questions')
  const height = (await card.boundingBox())!.height
  await examplesButton(page).click()
  await expect(list(page)).toBeVisible()
  await expect(examplesButton(page)).toHaveAttribute('aria-expanded', 'true')
  expect((await card.boundingBox())!.height).toBe(height)
  await page.locator('section.answer #answer-title').click()
  await expect(list(page)).toBeHidden()
  await expect(examplesButton(page)).toHaveAttribute('aria-expanded', 'false')
})

test('on a phone the list is a sheet from the bottom', { tag: '@touch' }, async ({ page }) => {
  await examplesButton(page).click()
  await expect(list(page)).toBeVisible()
  await expect(list(page)).toContainText(t.examplesTitle)
  await expect(list(page)).toContainText(t.examplesWait)
  const box = (await list(page).boundingBox())!
  const screen = page.viewportSize()!
  expect(Math.round(box.y + box.height)).toBe(screen.height)
  await list(page).getByRole('button', { name: t.close }).click()
  await expect(list(page)).toBeHidden()
})

test(
  'trying examples keeps the visitor’s numbers, and ↺ puts them back exactly',
  { tag: '@phone' },
  async ({ page }) => {
    // The first visit: an example, to be changed.
    await expect(examplePill(page)).toHaveText(t.exampleOf(situationName('goal')))
    await expect(status(page)).toContainText(t.makeThemYours)
    await fillField(page, 'now', '12345')
    await fillField(page, 'monthly', '678')
    await when(page, t.whenRetirement).click()
    await page.locator('#age').fill('38')
    await page.locator('#retire-at').fill('62')
    await expect(status(page)).toContainText(t.yourNumbers)
    await expect(examplePill(page)).toHaveCount(0)
    const yours = answersWith({
      now: 12_345,
      monthly: 678,
      horizon: { Retirement: { age: 38, retire_at: 62 } },
    })
    await expectAnswerFor(page, yours)
    await page.locator('#bank').selectOption(banks()[2].bank)
    for (const id of ['salary', 'cushion', 'inheritance']) {
      await pickExample(page, situationName(id))
      await expect(backButton(page)).toBeVisible()
      await expect(page.locator('#bank')).toHaveValue(banks()[2].bank)
    }
    await backButton(page).click()
    await expect(page.locator('#now')).toHaveValue('12,345')
    await expect(page.locator('#monthly')).toHaveValue('678')
    await expect(page.locator('#age')).toHaveValue('38')
    await expect(page.locator('#retire-at')).toHaveValue('62')
    await expect(status(page)).toContainText(t.yourNumbers)
    await expectAnswerFor(page, { ...yours, bank: banks()[2].bank })
  },
)

test('a field changed over an example unmarks it, and changed back marks it again', async ({ page }) => {
  await pickExample(page, situationName('bonus'))
  await expect(examplePill(page)).toHaveText(t.exampleOf(situationName('bonus')))
  await fillField(page, 'now', '15001')
  await expect(examplePill(page)).toHaveCount(0)
  await fillField(page, 'now', '15000')
  await expect(examplePill(page)).toHaveText(t.exampleOf(situationName('bonus')))
})

/** Where the status line, the fields, the box under "מתי" and the bank are. */
const places = (page: Page) =>
  page.evaluate(() =>
    ['.questions .status', '#now', '#monthly', '.when .choices', '.when .box', '#bank'].map((selector) => {
      const box = document.querySelector(selector)!.getBoundingClientRect()
      return `${selector} ${box.top + scrollY} ${box.left} ${box.width}×${box.height}`
    }),
  )

/** Tries every answer of "מתי", every example and the way back: nothing on
 * the card moves, the status line and the box keep their heights. */
async function expectNothingMoves(page: Page) {
  const first = await places(page)
  for (const name of [t.whenRetirement, t.whenAnyDay, t.whenUnknown, t.whenDate]) {
    await when(page, name).click()
    expect(await places(page), name).toEqual(first)
  }
  await fillField(page, 'monthly', '1500')
  expect(await places(page), 'typed').toEqual(first)
  for (const situation of exampleSituations()) {
    await pickExample(page, situation.name)
    expect(await places(page), situation.name).toEqual(first)
  }
  await backButton(page).click()
  expect(await places(page), 'back').toEqual(first)
}

test(
  'nothing on the card moves between the answers of "מתי" and the examples',
  { tag: '@phone' },
  async ({ page }) => {
    await expectNothingMoves(page)
  },
)

test('nothing moves on the narrowest phone either', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 667 })
  await expectNothingMoves(page)
})

test('only "לשעת חירום" says the money may be needed any day, and the date waits for its answer', async ({
  page,
}) => {
  const base = answersWith({ now: 30_000, monthly: 0 })
  await fillField(page, 'now', '30000')
  await fillField(page, 'monthly', '0')
  await fillField(page, 'horizon', '2')
  await when(page, t.whenAnyDay).click()
  await expect(page.locator('#horizon')).toBeHidden()
  await expectAnswerFor(page, { ...base, needsAnyDay: true })
  await when(page, t.whenUnknown).click()
  await expectAnswerFor(page, base)
  await when(page, t.whenDate).click()
  await expect(page.locator('#horizon')).toHaveValue('2')
  await expectAnswerFor(page, { ...base, horizon: { Months: 24 } })
})

test(
  'typing the money changes the answer, and clearing a field is an answer too',
  { tag: '@phone' },
  async ({ page }) => {
    const goal = situationAnswers('goal')
    await fillField(page, 'now', '20000')
    await expectAnswerFor(page, { ...goal, now: 20_000 })
    await fillField(page, 'monthly', '')
    expect((await moneyOnPage(page)).monthly).toBe('')
    await expectAnswerFor(page, { ...goal, now: 20_000, monthly: null })
    // Nothing to compare: the page says so instead of an answer for no money.
    await fillField(page, 'now', '')
    await expect(page.locator('.card.error')).toBeVisible()
    await fillField(page, 'now', '30000')
    await expect(page.locator('.card.error')).toBeHidden()
    await expectAnswerFor(page, { ...goal, now: 30_000, monthly: null })
  },
)

/** The reference rows' names where to buy, as the core makes them. */
function referenceRows(answers: AnswersInput, choices: Choices) {
  const expected = answerFor(answers, choices)
  const where = expected.how.find((how) => kindKey(how.row) === kindKey(expected.focus!))!.whereToBuy!
  return [
    ...where.rows.filter((row) => row.yours).map((row) => t.yourBankRow(row.name)),
    ...(where.bankRange ? [t.throughABank] : []),
  ]
}

// The bank is one of the visitor's facts (REDESIGN.md §2.1; the review of
// 2026-10-20, X8): optional, asked once, with the others, and where to buy
// is compared from it. The reference row is whatever the core makes of it.
test('the bank is asked with the facts, optional, and where to buy follows it', async ({ page }) => {
  const goal = situationAnswers('goal')
  const choices: Choices = { safeWay: 'Bond' }
  const field = page.locator('#bank')
  await expect(field).toHaveValue('')
  await decide(page, goal, {}, 'SafeWay', { SafeWay: 'Bond' })
  const where = page.locator('section.how .where')
  await expect(where).toBeVisible()
  // Not asked again where to buy (principle 7).
  await expect(where.getByRole('button')).toHaveCount(0)
  for (const bank of banks()) {
    await field.selectOption(bank.bank)
    await expectAnswerFor(page, { ...goal, bank: bank.bank }, choices)
    await expect(where.locator('.row.yours .name')).toHaveText(
      referenceRows({ ...goal, bank: bank.bank }, choices),
    )
  }
  // Another bank names none, as no answer does, and the menu says what was picked.
  await field.selectOption('other')
  await expect(field).toHaveValue('other')
  await expect(where.locator('.row.yours .name')).toHaveText(
    referenceRows({ ...goal, bank: undefined }, choices),
  )
  // An example keeps the bank: it is the visitor's, not the example's.
  await field.selectOption(banks()[1].bank)
  await pickExample(page, situationName('bonus'))
  await expect(field).toHaveValue(banks()[1].bank)
})

test('whether the date can move is asked in the answer, only where the core asks it', async ({ page }) => {
  const base = answersWith({ now: 50_000, monthly: 0 })
  await fillField(page, 'monthly', '0')
  await fillField(page, 'now', '50000')
  for (const years of [2, 3, 5, 6, 8, 3]) {
    await fillField(page, 'horizon', String(years))
    await expect(page.locator('#horizon')).toHaveValue(String(years))
    // As many decisions as the core asks for this date, none in the questions.
    await expectAnswerFor(page, { ...base, horizon: { Months: years * 12 } })
    await expectOnlyFacts(page)
  }
  // Where it is asked, answering it changes the answer.
  await fillField(page, 'horizon', '5')
  const five = { ...base, horizon: { Months: 60 } }
  const asks = decisionsOf(answerFor(five)).some(({ decision }) => decision.id === 'PlaceType')
  expect(asks, 'five years with nothing monthly asks whether the date can move').toBe(true)
  for (const flexibility of ['Fixed', 'CanWait'] as const) {
    await decide(page, five, {}, 'PlaceType', { Flexibility: flexibility })
    await expectAnswerFor(page, { ...five, flexibility })
  }
})

test('the date is a number and a unit, and the unit can be changed', async ({ page }) => {
  const goal = situationAnswers('goal')
  const unit = (name: string) => choice(page, t.timeUnit, name)
  await expect(page.getByRole('radiogroup', { name: t.timeUnit }).locator('input:checked')).toHaveValue(
    'years',
  )
  await expect(page.locator('#horizon')).toHaveValue('3')
  // The same three years, said in months.
  await unit(t.monthsUnit).click()
  await expect(page.locator('#horizon')).toHaveValue(String(shown(36, 'months')))
  await fillField(page, 'horizon', '18')
  await expectAnswerFor(page, { ...goal, horizon: { Months: 18 } })
  // And back: a year and a half.
  await page.locator('#horizon').blur()
  await unit(t.yearsUnit).click()
  await expect(page.locator('#horizon')).toHaveValue('1.5')
})

test('an emptied date is said, and "עוד אין תאריך" is the date not known', async ({ page }) => {
  const salary = situationAnswers('salary')
  await pickExample(page, situationName('salary'))
  await page.locator('#horizon').fill('')
  await expect(page.locator('.card.error')).toBeVisible()
  await when(page, t.whenUnknown).click()
  await expect(page.locator('.card.error')).toBeHidden()
  await expectAnswerFor(page, { ...salary, horizon: 'Unknown' })
  await expectOnlyFacts(page)
  await when(page, t.whenDate).click()
  await fillField(page, 'horizon', '10')
  await expectAnswerFor(page, { ...salary, horizon: { Months: 120 } })
})

test('for retirement the questions ask the age today and the age of retiring', async ({ page }) => {
  await pickExample(page, situationName('retirement'))
  await expectOnlyFacts(page)
  await expect(page.locator('#horizon')).toBeHidden()
  await expect(page.locator('#age')).toHaveValue('45')
  await expect(page.locator('#retire-at')).toHaveValue('65')
  await page.locator('#age').fill('40')
  await page.locator('#retire-at').fill('60')
  const answers = { ...situationAnswers('retirement'), horizon: { Retirement: { age: 40, retire_at: 60 } } }
  await expectAnswerFor(page, answers)
})

test('text that is not a number is said to be, not silently ignored', async ({ page }) => {
  await fillField(page, 'now', '-5000abc')
  await expect(page.locator('.card.error')).toContainText(t.notANumber)
  await fillField(page, 'now', '5000')
  await expect(page.locator('.card.error')).toBeHidden()
})
