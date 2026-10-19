import { alternativesFor, learnCard, NO_OVERRIDES, recommendation } from './core'
import { answerCard, enter, expect, expectAnswerFor, test } from './fixtures'
import { arenaOf } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// The answer: what the guide chose, in the words people meet, and what
// follows from it. The sentences are the core's; here, that the page shows
// them and that its buttons do what they say.

const fiveYearsFixed = { now: 50_000, monthly: 0, horizon: 5, flexibility: 'Fixed' } as const

test('the sentence names what fits, its amount and its reasons', { tag: '@phone' }, async ({ page }) => {
  const answers = await enter(page, fiveYearsFixed)
  const expected = await expectAnswerFor(page, answers)
  const reasons = answerCard(page).locator('ul.reasons li')
  await expect(reasons).toHaveCount(expected.reasons.length)
  await expect(page.locator('section.steps li')).toHaveCount(expected.steps.length)
  // The same money left in a current account, for comparison.
  await expect(answerCard(page)).toContainText(expected.currentAccountLine)
})

test('"what is it" opens the kind\'s card, and it closes', { tag: '@phone' }, async ({ page }) => {
  const answers = await enter(page, fiveYearsFixed)
  const { kind } = await expectAnswerFor(page, answers)
  await answerCard(page).getByRole('button', { name: t.whatIsIt }).click()
  const card = page.getByRole('dialog', { name: learnCard(kind.card!).name })
  await expect(card).toBeVisible()
  await expect(card).toContainText(learnCard(kind.card!).what)
  await page.keyboard.press('Escape')
  await expect(card).toBeHidden()
})

test('"change" offers the other kinds, and choosing one goes on with it until sent back', async ({
  page,
}) => {
  const answers = await enter(page, { ...fiveYearsFixed, flexibility: 'CanWait' })
  const guide = await expectAnswerFor(page, answers)
  await answerCard(page).getByRole('button', { name: t.changeKind }).click()
  const picker = answerCard(page).locator('.picker')
  await expect(picker.locator('li')).toHaveCount(guide.ranked.length)
  // The first other kind the guide ranks.
  const other = guide.ranked.find(({ kind }) => JSON.stringify(kind) !== JSON.stringify(guide.kind.kind))!
  await picker.locator('li').nth(guide.ranked.indexOf(other)).getByRole('button').click()
  const chosen = { arena: arenaOf(other.kind), kind: other.kind }
  const changed = await expectAnswerFor(page, answers, chosen)
  expect(changed.kind.kind).toEqual(other.kind)
  await expect(answerCard(page).getByRole('button', { name: t.backToRecommendation })).toBeVisible()

  await answerCard(page).getByRole('button', { name: t.backToRecommendation }).click()
  await expectAnswerFor(page, answers, NO_OVERRIDES)
  await expect(answerCard(page).getByRole('button', { name: t.backToRecommendation })).toBeHidden()
})

test('"change" also offers the other arena as a whole', async ({ page }) => {
  const answers = await enter(page, { ...fiveYearsFixed, flexibility: 'CanWait' })
  const guide = await expectAnswerFor(page, answers)
  await answerCard(page).getByRole('button', { name: t.changeKind }).click()
  const arenas = guide.arenaChoices.filter(({ arena }) => arena !== guide.arena)
  await answerCard(page).locator('.picker .options').getByRole('button', { name: arenas[0].name }).click()
  await expectAnswerFor(page, answers, { arena: arenas[0].arena, kind: undefined })
  await expect(answerCard(page).getByRole('button', { name: t.backToRecommendation })).toBeVisible()
})

// A first deposit that opens a cheaper plan: the line says which and how much more.
test('the unlock line says what a bigger first deposit would open', async ({ page }) => {
  const answers = await enter(page, { now: 0, monthly: 500, horizon: 30 })
  const expected = await expectAnswerFor(page, answers)
  expect(expected.unlockText, 'the core has an unlock for this money').toBeDefined()
  await expect(answerCard(page).locator('.unlock')).toHaveText(expected.unlockText!)
  // With the deposit, there is nothing left to unlock.
  const richer = await enter(page, { now: 50_000, monthly: 500, horizon: 30 })
  await expectAnswerFor(page, richer)
  expect(recommendation(richer).unlockText).toBeUndefined()
  await expect(answerCard(page).locator('.unlock')).toHaveCount(0)
})

test('with no cushion the answer is a plan in two parts', async ({ page }) => {
  const answers = await enter(page, { now: 0, monthly: 3000, horizon: 'unknown', cushion: 17_000 })
  const expected = await expectAnswerFor(page, answers)
  expect(expected.twoParts).toBeDefined()
  // One plan, not one sum: no single amount in the box.
  await expect(answerCard(page).locator('.amount-box')).toHaveCount(0)
  await expect(page.locator('section.steps li')).toHaveCount(expected.steps.length)
  // With a cushion the plan is one part again.
  const has = await enter(page, { now: 0, monthly: 3000, horizon: 'unknown', cushion: 'has' })
  const one = await expectAnswerFor(page, has)
  expect(one.twoParts).toBeUndefined()
  await expect(answerCard(page).locator('.amount-box')).toHaveCount(1)
})

test("the other kinds drawn are the core's, one row each", async ({ page }) => {
  const answers = await enter(page, fiveYearsFixed)
  await expectAnswerFor(page, answers)
  const drawn = alternativesFor(answers).series
  await expect(page.locator('.alternatives li.row')).toHaveCount(drawn.length)
})
