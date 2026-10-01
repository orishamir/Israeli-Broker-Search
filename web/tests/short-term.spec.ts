import { t } from '../src/lib/text'
import {
  aboutShortTerm,
  expectedPlaceRows,
  place,
  placeRowsOnPage,
  shortTermExamples,
  shortTermInputsOnPage,
  shortTermKinds,
  shortTermPlaces,
  termFor,
} from './core'
import { away, details, expect, test, type Page } from './fixtures'

// The short-term calculator: the switch to it, its inputs, the places to
// compare, and what the core says for them.

/** Opens the short-term calculator. */
async function toShortTerm(page: Page) {
  await page.getByText(t.shortTerm, { exact: true }).click()
  await expect(page.getByRole('heading', { name: t.forHowLong })).toBeVisible()
}

/** The list of places, a dialog over the inputs. */
const placesList = (page: Page) => page.getByRole('dialog', { name: t.whatsCompared })

async function openPlaces(page: Page) {
  await page.getByRole('button', { name: t.addOrRemove }).click()
  await expect(placesList(page)).toBeVisible()
}

async function closePlaces(page: Page) {
  await away(page)
  await page.keyboard.press('Escape')
  await expect(placesList(page)).toBeHidden()
}

/** The table matches what the core says for the inputs on the page. */
async function expectCoresTable(page: Page, yourNames: Record<string, string> = {}) {
  const inputs = await shortTermInputsOnPage(page)
  await expect.poll(() => placeRowsOnPage(page)).toEqual(expectedPlaceRows(inputs, yourNames))
}

test(
  'the switch opens the short-term calculator, ranked as the core ranks',
  { tag: '@phone' },
  async ({ page }) => {
    await toShortTerm(page)
    await expect(page.getByRole('radio', { name: t.shortTerm })).toBeChecked()
    // The places the core ticks at first, named in the card.
    const ticked = shortTermPlaces.filter(({ place }) => place.comparedAtFirst)
    await expect(page.getByRole('list', { name: t.whatsCompared }).getByRole('listitem')).toHaveText(
      ticked.map(({ place }) => place.shortName),
    )
    await expectCoresTable(page)
    // The long term's inputs aren't on the page.
    await expect(page.getByRole('radiogroup', { name: t.security })).toHaveCount(0)
  },
)

test('each calculator keeps its inputs while the other is shown', async ({ page }) => {
  await page.getByLabel(t.oneTimeDeposit).fill('12345')
  await toShortTerm(page)
  await expect(page.getByLabel(t.oneTimeDeposit)).toHaveValue('100,000')
  await page.getByLabel(t.oneTimeDeposit).fill('50000')
  await page.getByText(t.longTerm, { exact: true }).click()
  await expect(page.getByLabel(t.oneTimeDeposit)).toHaveValue('12,345')
  await toShortTerm(page)
  await expect(page.getByLabel(t.oneTimeDeposit)).toHaveValue('50,000')
})

test('an example sets the amounts and months; with money every month, deposits step aside', async ({
  page,
}) => {
  await toShortTerm(page)
  const flat = shortTermExamples().find(({ monthlyDeposit }) => monthlyDeposit > 0)!
  await page.getByRole('button', { name: flat.name }).click()
  await expect(page.locator('#months')).toHaveValue(String(flat.months))
  await expect(page.getByLabel(t.everyMonth)).toHaveValue(flat.monthlyDeposit.toLocaleString('en-US'))
  await expectCoresTable(page)
  // Every deposit is last, saying why, and the list says so where they're ticked.
  const leumi = place('Fixed-rate deposit · Bank Leumi').place
  const row = page.locator('tbody tr', { has: page.getByText(leumi.name, { exact: true }) })
  await row.getByRole('button', { name: t.whatMeans('מקבל סכום אחד') }).hover()
  await expect(page.getByRole('tooltip').filter({ hasText: 'תוכנית חיסכון' })).toBeVisible()
  await openPlaces(page)
  await expect(placesList(page).getByText(t.monthlyNotForDeposits)).toBeVisible()
})

test('a deposit of your own is compared at the rate you type, and kept for the next visit', async ({
  page,
}) => {
  await toShortTerm(page)
  await openPlaces(page)
  await placesList(page).getByRole('button', { name: t.addYourDeposit }).click()
  // The caret is in its rate, to type the one offered.
  const rate = placesList(page).getByRole('textbox', { name: t.depositRate, exact: true })
  await expect(rate).toBeFocused()
  await rate.fill('4.6')
  const yours = { [(await rate.getAttribute('id'))!.replace('deposit-rate-', '')]: t.yourDepositName }
  await closePlaces(page)
  await expectCoresTable(page, yours)
  await expect(page.locator('tbody tr').first()).toContainText(t.yourDepositName)

  await page.reload()
  await toShortTerm(page)
  await expectCoresTable(page, yours)

  await openPlaces(page)
  await placesList(page)
    .getByRole('button', { name: t.deleteDeposit(t.yourDepositName) })
    .click()
  await closePlaces(page)
  await expect(page.locator('tbody')).not.toContainText(t.yourDepositName)
})

test("a place's details show its rates by term, the months' term marked", async ({ page }) => {
  await toShortTerm(page)
  await openPlaces(page)
  const { place: leumi } = place('Fixed-rate deposit · Bank Leumi')
  await placesList(page)
    .getByRole('button', { name: t.about(leumi.name) })
    .click()
  const dialog = details(page)
  await expect(dialog.getByRole('heading', { name: leumi.name })).toBeVisible()
  await expect(dialog.getByRole('row')).toHaveCount(leumi.rates.length)
  const chosen = leumi.rates[termFor(12)!]
  await expect(dialog.locator('tr.chosen')).toHaveText(new RegExp(chosen.term))
  await expect(dialog.getByText(leumi.tax)).toBeVisible()
})

test("each kind is explained by the table's place column and beside its heading in the list", async ({
  page,
}) => {
  await toShortTerm(page)
  const kinds = shortTermKinds()
  await page.getByRole('button', { name: t.whatMeans(t.place) }).hover()
  const tip = page.getByRole('tooltip').filter({ hasText: t.placeTip })
  for (const { name, description } of kinds) {
    await expect(tip).toContainText(name)
    await expect(tip).toContainText(description)
  }
  await away(page)

  await openPlaces(page)
  for (const { name, description } of kinds) {
    // An open tip covers the next heading's ? until the mouse leaves it.
    await away(page)
    await placesList(page)
      .getByRole('button', { name: t.whatMeans(name) })
      .hover()
    await expect(page.getByRole('tooltip').filter({ hasText: description })).toBeVisible()
  }
})

test("the title's links open the short term's page on its numbers", async ({ page }) => {
  await toShortTerm(page)
  const about = aboutShortTerm()
  await page.getByRole('button', { name: about.sections[0].title }).click()
  await expect(details(page).getByRole('heading', { name: about.sections[0].title })).toBeVisible()
  await expect(details(page).getByText(about.sections[0].paragraphs[0])).toBeVisible()
})

test('a row pins its place in the chart, and the button unpins it', async ({ page }) => {
  await toShortTerm(page)
  const first = page.locator('tbody tr').first()
  await first.click()
  await expect(first).toHaveClass(/pinned/)
  await page.getByRole('button', { name: t.unpinAll }).click()
  await expect(first).not.toHaveClass(/pinned/)
})
