import { answerFor, answersWith, banks, chipSituations, decisionsOf, situations } from './core'
import type { AnswersInput, Choices } from './core'
import { chip, decide, expect, expectAnswerFor, fillField, test, type Page } from './fixtures'
import { shown } from '../src/lib/horizon'
import { kindKey } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// The questions: the chips, the fields, the horizon and its unit. They ask
// only the visitor's facts (REDESIGN.md §2.1): what only the visitor can
// decide is asked in the answer, where it decides, never here as a card.
// What the answer is for each is the core's (`answer`); here, that the page
// asks the core what the fields say.

/** The questions ask nothing but the facts: the chips are their only group,
 * and no follow-up card is left. */
async function expectOnlyFacts(page: Page) {
  const questions = page.locator('.questions-column')
  await expect(questions.getByRole('group')).toHaveCount(1)
  await expect(page.locator('section.follow')).toHaveCount(0)
}

const moneyOnPage = async (page: Page) => ({
  now: await page.locator('#now').inputValue(),
  monthly: await page.locator('#monthly').inputValue(),
})

test(
  'every chip fills the fields with its situation and the answer follows',
  { tag: '@phone' },
  async ({ page }) => {
    // The workbench's and the portfolio's open other things, tested apart.
    for (const situation of chipSituations()) {
      await chip(page, situation.name).click()
      await expect(chip(page, situation.name)).toHaveAttribute('aria-pressed', 'true')
      const { now, monthly } = situation.answers
      await expect(page.locator('#now')).toHaveValue(now === null ? '' : now.toLocaleString('en-US'))
      await expect(page.locator('#monthly')).toHaveValue(
        monthly === null ? '' : monthly.toLocaleString('en-US'),
      )
      await expectOnlyFacts(page)
      await expectAnswerFor(page, situation.answers)
    }
  },
)

test(
  'typing the money changes the answer, and clearing a field is an answer too',
  { tag: '@phone' },
  async ({ page }) => {
    const goal = situations().find(({ id }) => id === 'goal')!.answers
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
  const goal = situations().find(({ id }) => id === 'goal')!.answers
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
  // A chip keeps the bank: it is the visitor's, not the example's.
  await field.selectOption(banks()[1].bank)
  await chip(page, situations().find(({ id }) => id === 'bonus')!.name).click()
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

test('the horizon is a number and a unit, and the unit can be changed', async ({ page }) => {
  const goal = situations().find(({ id }) => id === 'goal')!
  await chip(page, goal.name).click()
  const unit = page.getByLabel(t.timeUnit)
  await expect(unit).toHaveValue('years')
  await expect(page.locator('#horizon')).toHaveValue('3')
  // The same three years, said in months.
  await page.locator('#horizon').blur()
  await unit.selectOption('months')
  await expect(page.locator('#horizon')).toHaveValue(String(shown(36, 'months')))
  await fillField(page, 'horizon', '18')
  await expectAnswerFor(page, { ...goal.answers, horizon: { Months: 18 } })
  // And back: a year and a half.
  // A click on the menu takes the focus off the field, as it would a person's.
  await page.locator('#horizon').blur()
  await unit.selectOption('years')
  await expect(page.locator('#horizon')).toHaveValue('1.5')
  // A new chip starts over with the unit that reads best.
  await chip(page, situations().find(({ id }) => id === 'salary')!.name).click()
  await expect(unit).toHaveValue('years')
})

test('an emptied date field is a date not known yet, and typing one answers for it', async ({ page }) => {
  const salary = situations().find(({ id }) => id === 'salary')!.answers
  await chip(page, situations().find(({ id }) => id === 'salary')!.name).click()
  await page.locator('#horizon').fill('')
  await expect(page.locator('#horizon')).toHaveValue('')
  const unknown = { ...salary, horizon: 'Unknown' as const }
  await expectAnswerFor(page, unknown)
  await expectOnlyFacts(page)
  // A horizon: the answer is the core's for it, whatever it asks.
  await fillField(page, 'horizon', '10')
  await expectAnswerFor(page, { ...unknown, horizon: { Months: 120 } })
})

test('for retirement the questions ask the age and when to retire', async ({ page }) => {
  const retirement = situations().find(({ id }) => id === 'retirement')!
  await chip(page, retirement.name).click()
  await expectOnlyFacts(page)
  await expect(page.locator('#age')).toHaveValue('45')
  await expect(page.locator('#retire-at')).toHaveValue('65')
  await page.locator('#age').fill('40')
  await page.locator('#retire-at').fill('60')
  const answers = { ...retirement.answers, horizon: { Retirement: { age: 40, retire_at: 60 } } }
  await expectAnswerFor(page, answers)
})

test('typing years for a retirement keeps the ages and the retirement', async ({ page }) => {
  const retirement = situations().find(({ id }) => id === 'retirement')!
  await chip(page, retirement.name).click()
  await fillField(page, 'horizon', '18')
  await expect(page.locator('#age')).toHaveValue('45')
  await expect(page.locator('#retire-at')).toHaveValue('63')
  const answers = { ...retirement.answers, horizon: { Retirement: { age: 45, retire_at: 63 } } }
  await expectAnswerFor(page, answers)
})

test('the horizon label is not squeezed to a word a line', { tag: '@phone' }, async ({ page }) => {
  const label = page.locator('label[for="horizon"]')
  const box = (await label.boundingBox())!
  const lineHeight = Number.parseFloat(await label.evaluate((el) => getComputedStyle(el).lineHeight))
  // A label of a few words fits two lines, in the room the field leaves.
  expect(box.height).toBeLessThanOrEqual(lineHeight * 2.1)
})

test('text that is not a number is said to be, not silently ignored', async ({ page }) => {
  await fillField(page, 'now', '-5000abc')
  await expect(page.locator('.card.error')).toContainText(t.notANumber)
  await fillField(page, 'now', '5000')
  await expect(page.locator('.card.error')).toBeHidden()
})
