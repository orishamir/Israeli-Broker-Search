import { alternativesFor, glossary, learnCard } from './core'
import { away, enter, expect, expectAnswerFor, test, type Page } from './fixtures'
import { t } from '../src/lib/text'

// The learn layer: a term's card from a tip's "more about this" or the
// answer's "what is it", the glossary that lists them all, and the way back
// through the cards opened one from another. The cards' sentences are the
// core's.

const fiveYearsFixed = { now: 50_000, monthly: 0, horizon: 5, flexibility: 'Fixed' } as const
const card = (page: Page, name: string) => page.getByRole('dialog', { name, exact: true })

test('a tip with a card has "more about this", which opens the card', { tag: '@phone' }, async ({ page }) => {
  const answers = await enter(page, fiveYearsFixed)
  await expectAnswerFor(page, answers)
  // The first kind drawn whose safety tag has a card of its own.
  const series = alternativesFor(answers).series
  const index = series.findIndex(({ card }) => card)
  expect(index, 'some kind drawn has a card').toBeGreaterThanOrEqual(0)
  const row = page.locator('.alternatives li.row').nth(index)
  const wanted = learnCard(series[index].card!)
  await row.getByRole('button', { name: t.whatMeans(series[index].safetyTag) }).click()
  await page.getByRole('tooltip').getByRole('button', { name: t.learnMore }).click()
  await expect(card(page, wanted.name)).toBeVisible()
  await expect(card(page, wanted.name)).toContainText(wanted.what)
})

test('cards open from one another, and the way back leads through them', async ({ page }) => {
  const answers = await enter(page, fiveYearsFixed)
  const { kind } = await expectAnswerFor(page, answers)
  const first = learnCard(kind.card!)
  await page.getByRole('button', { name: t.whatIsIt }).click()
  await expect(card(page, first.name)).toBeVisible()
  // No way back from the first card.
  await expect(page.getByRole('button', { name: t.cardBack })).toHaveCount(0)
  const next = learnCard(first.compareWith[0])
  await card(page, first.name)
    .getByRole('button', { name: t.cardVersus(next.name) })
    .click()
  await expect(card(page, next.name)).toBeVisible()
  await page.getByRole('button', { name: t.cardBack }).click()
  await expect(card(page, first.name)).toBeVisible()
  await away(page)
  await page.keyboard.press('Escape')
  await expect(card(page, first.name)).toBeHidden()
})

test(
  'the glossary lists every card, opens one, and "all terms" comes back',
  { tag: '@phone' },
  async ({ page }) => {
    await page.getByRole('button', { name: t.glossary }).click()
    const list = page.getByRole('dialog', { name: t.glossary })
    await expect(list).toBeVisible()
    const cards = glossary()
    await expect(list.locator('li')).toHaveCount(cards.length)
    const some = cards[Math.floor(cards.length / 2)]
    await list.locator('li').filter({ hasText: some.name }).first().getByRole('button').click()
    await expect(card(page, learnCard(some.term).name)).toBeVisible()
    await expect(list).toBeHidden()
    await card(page, learnCard(some.term).name).getByRole('button', { name: t.allTerms }).click()
    await expect(list).toBeVisible()
  },
)
