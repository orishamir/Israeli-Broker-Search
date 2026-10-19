import { answersWith, chipSituations, followCard, followUps, situations } from './core'
import type { AnswersInput, FollowUp } from './core'
import { chip, expect, expectAnswerFor, fillField, test, type Page } from './fixtures'
import { shown } from '../src/lib/horizon'
import { t } from '../src/lib/text'

// The questions: the chips, the fields, the horizon and its unit, and the
// follow-up cards that come and go with the answers. What the answer is for
// each is the core's (`recommend`); here, that the page asks the core what
// the fields say.

/** The follow-up cards the page shows, against those the core asks for. */
async function expectFollowUps(page: Page, answers: AnswersInput) {
  const asked = followUps(answers)
  for (const each of ['Flexibility', 'AgeAndRetirement', 'Cushion'] as FollowUp[]) {
    const title = page.getByRole('heading', { name: followCard(each).title, exact: true })
    await expect(title, each).toHaveCount(asked.includes(each) ? 1 : 0)
  }
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
      await expectFollowUps(page, situation.answers)
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

test('the follow-up cards come and go as the horizon changes', async ({ page }) => {
  const base = answersWith({ now: 50_000, monthly: 0, flexibility: undefined })
  await chip(page, situations().find(({ id }) => id === 'goal')!.name).click()
  await fillField(page, 'monthly', '0')
  await fillField(page, 'now', '50000')
  // Under 3 years and past 7: the date doesn't matter; between: it does.
  for (const years of [2, 3, 5, 6, 8, 3]) {
    await fillField(page, 'horizon', String(years))
    await expect(page.locator('#horizon')).toHaveValue(String(years))
    const answers = { ...base, horizon: { Months: years * 12 } }
    await expectFollowUps(page, answers)
  }
  // Answering the card changes the answer.
  await fillField(page, 'horizon', '5')
  for (const flexibility of ['Fixed', 'CanWait'] as const) {
    const card = page.locator('section.follow').filter({ hasText: followCard('Flexibility').title })
    await card
      .getByRole('button', { name: followCard('Flexibility').answers[flexibility === 'Fixed' ? 0 : 1] })
      .click()
    await expectAnswerFor(page, { ...base, horizon: { Months: 60 }, flexibility })
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

test('"don\'t know yet" empties the horizon and asks about a cushion', async ({ page }) => {
  const salary = situations().find(({ id }) => id === 'salary')!.answers
  await chip(page, situations().find(({ id }) => id === 'salary')!.name).click()
  await page.locator('.money').getByRole('button', { name: t.dontKnowYet }).click()
  await expect(page.locator('#horizon')).toHaveValue('')
  const unknown = { ...salary, horizon: 'Unknown' as const }
  await expectFollowUps(page, unknown)
  // The cushion's card asks for the month's expenses when there is none.
  const card = page.locator('section.follow').filter({ hasText: followCard('Cushion').title })
  await card.getByRole('button', { name: followCard('Cushion').answers[1] }).click()
  await page.locator('#expenses').fill('17000')
  const answers = { ...unknown, cushion: { kind: 'hasNot' as const, monthlyExpenses: 17_000 } }
  await expectFollowUps(page, answers)
  await expectAnswerFor(page, answers)
  // Typing a horizon takes the question back.
  await fillField(page, 'horizon', '10')
  await expectFollowUps(page, { ...salary, horizon: { Months: 120 } })
})

test('for retirement the card asks the age and when to retire', async ({ page }) => {
  const retirement = situations().find(({ id }) => id === 'retirement')!
  await chip(page, retirement.name).click()
  await expectFollowUps(page, retirement.answers)
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
  await expectFollowUps(page, answers)
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
