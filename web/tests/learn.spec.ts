import { glossary, learnCard, ways } from './core'
import { away, expect, test, workbench, type Page } from './fixtures'
import { t } from '../src/lib/text'

// The learn layer: a term's card from a tip's "more about this", the
// glossary that lists them all, and the way back through the cards opened
// one from another. The cards' sentences are the core's.

const card = (page: Page, name: string) => page.getByRole('dialog', { name, exact: true })

/** The first way the workbench lists that has a card, and its card. */
function wayWithCard() {
  const way = ways().find(({ card }) => card)
  if (!way) throw new Error('no way has a card')
  return { way, learn: learnCard(way.card!) }
}

/** Opens the card of the first way with one, from its tip in the workbench. */
async function openWayCard(page: Page) {
  const { way, learn } = wayWithCard()
  await workbench(page, t.thingSp500)
  const item = page
    .getByRole('list', { name: t.waysTitle })
    .getByRole('listitem')
    .filter({ hasText: way.name })
  await item.getByRole('button', { name: t.whatMeans(way.name) }).click()
  await page.getByRole('tooltip').getByRole('button', { name: t.learnMore }).click()
  await expect(card(page, learn.name)).toBeVisible()
  return learn
}

test('a tip with a card has "more about this", which opens the card', { tag: '@phone' }, async ({ page }) => {
  const learn = await openWayCard(page)
  await expect(card(page, learn.name)).toContainText(learn.what)
})

test('cards open from one another, and the way back leads through them', async ({ page }) => {
  const first = await openWayCard(page)
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
