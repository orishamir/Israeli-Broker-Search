import { answer, answerFor, guideSettings, situationAnswers } from './core'
import type { AnswerData, Kind } from './core'
import { pickExample, decide, expect, expectAnswerFor, situationName, test, type Page } from './fixtures'
import { kindKey } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// "מה יהיה לכם ב‑…" (REDESIGN.md §2.6) and "איך עושים את זה" (§2.1 item 4):
// the chart's rows are the options; tapping one shows its how-to, rows the
// visitor's choices leave out dim, and the stock row's yearly return is set
// in place. Where each mark goes is a unit test (`outcome.test.svelte.ts`).

const chart = (page: Page) => page.locator('section.outcome')
const rows = (page: Page) => chart(page).locator('.row:not(.axis, .lines)')
const rowPress = (page: Page, at: number) => rows(page).nth(at).locator('button.press')
const howHeading = (page: Page) => page.locator('#how-heading')

/** The How the core gives for a row, by its kind. */
const howOf = (data: AnswerData, kind: Kind) => data.how.find((how) => kindKey(how.row) === kindKey(kind))

test(
  'a tapped row shows its how-to, and the row is marked; a choice brings the answer’s back',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = situationAnswers('goal')
    const expected = await expectAnswerFor(page, answers)
    await expect(rows(page)).toHaveCount(expected.outcome.rows.length)
    // Nothing chosen yet: the empty state.
    expect(expected.focus).toBeUndefined()
    await expect(howHeading(page)).toHaveText(t.howNothingYet)
    const withHow = expected.outcome.rows.flatMap((row, at) => {
      const how = howOf(expected, row.id)
      return how ? [{ at, how }] : []
    })
    expect(withHow.length).toBeGreaterThan(1)
    for (const { at, how } of withHow) {
      await rowPress(page, at).click()
      await expect(rowPress(page, at)).toHaveAttribute('aria-pressed', 'true')
      await expect(howHeading(page)).toHaveText(how.name)
      await expect(rows(page).locator('button.press[aria-pressed=true]')).toHaveCount(1)
    }
    // A choice in the answer: how-to follows the choice, not the last tap.
    const decision = expected.lines.flatMap((line) =>
      line.state.type === 'decision' ? [line.state.decision] : [],
    )[0]
    const option = decision.options[0]
    await decide(page, answers, {}, decision.id, option.pick)
    const chosen = answerFor(answers, 'SafeWay' in option.pick ? { safeWay: option.pick.SafeWay } : {})
    expect(chosen.focus, 'a chosen way is the answer’s focus').toBeDefined()
    await expect(howHeading(page)).toHaveText(howOf(chosen, chosen.focus!)!.name)
  },
)

test('rows are reached and chosen by keyboard', async ({ page }) => {
  const expected = await expectAnswerFor(page, situationAnswers('goal'))
  const at = expected.outcome.rows.findIndex((row) => howOf(expected, row.id))
  await rowPress(page, at).focus()
  await page.keyboard.press('Enter')
  await expect(rowPress(page, at)).toHaveAttribute('aria-pressed', 'true')
  await expect(howHeading(page)).toHaveText(howOf(expected, expected.outcome.rows[at].id)!.name)
})

test('the rows a choice leaves out dim, as the core says, and come back when it changes', async ({
  page,
}) => {
  const answers = situationAnswers('goal')
  const expected = await expectAnswerFor(page, answers)
  const dimmed = (data: AnswerData) =>
    data.outcome.rows.filter(
      (row) => !row.active && kindKey(row.id) !== kindKey(data.focus ?? 'CurrentAccount'),
    ).length
  await expect(rows(page).and(page.locator('.dim'))).toHaveCount(dimmed(expected))
  for (const safeWay of ['Bond', 'MoneyMarketFund'] as const) {
    await decide(page, answers, {}, 'SafeWay', { SafeWay: safeWay })
    const chosen = await expectAnswerFor(page, answers, { safeWay })
    expect(dimmed(chosen), 'a choice leaves some rows out').toBeGreaterThan(0)
    await expect(rows(page).and(page.locator('.dim'))).toHaveCount(dimmed(chosen))
  }
})

test(
  'the stock row’s yearly return is set in place, and the row follows it',
  { tag: '@phone' },
  async ({ page }) => {
    await pickExample(page, situationName('salary'))
    const answers = situationAnswers('salary')
    await expectAnswerFor(page, answers)
    const field = page.locator('#outcome-return')
    await expect(field).toBeVisible()
    await expect(field).toHaveValue('10')
    await field.fill('6')
    const six = answerFor(answers, {}, { indexReturnPercent: 6 })
    const stock = six.outcome.rows.findIndex((row) => typeof row.id === 'object' && 'Way' in row.id)
    await expect(rows(page).nth(stock).locator('.value')).toHaveText(six.outcome.rows[stock].amount)
    // It travels in the address, so a refresh keeps it.
    await expect.poll(() => page.evaluate(() => location.hash)).toContain('r=6')
  },
)

const isStock = (kind: Kind) => typeof kind === 'object' && 'Way' in kind

for (const id of ['goal', 'bonus', 'salary', 'cushion']) {
  test(
    `the chart of "${id}": each row's amount by its name, its facts in words, the band's marks and the lines labelled with their amounts`,
    { tag: '@phone' },
    async ({ page }) => {
      await pickExample(page, situationName(id))
      const expected = await expectAnswerFor(page, situationAnswers(id))
      const outcome = expected.outcome
      // The words are the core's, checked in Rust: here, that each has its
      // place on the page.
      await expect(chart(page).locator('.caption')).toBeVisible()
      await expect(rows(page)).toHaveCount(outcome.rows.length)
      for (const [at, row] of outcome.rows.entries()) {
        const shown = rows(page).nth(at)
        await expect(shown.locator('.name')).toBeVisible()
        await expect(shown.locator('.value')).not.toBeEmpty()
        if (row.facts) await expect(shown.locator('.facts')).toBeVisible()
        // Facts are words, not pills.
        await expect(shown.locator('.tag')).toHaveCount(0)
        // Every mark of the band has its label.
        await expect(shown.locator('.marks .label')).toHaveCount(row.marks.length)
      }
      for (const line of outcome.lines) await expect(chart(page).locator(`.line-label.${line.which}`)).toBeVisible()
    },
  )
}

// Only what the visitor could choose (§4.6, §4.9): no stock row, so no
// return to set.
test('money needed any day: only its own options, with no stock row and no return to set', async ({
  page,
}) => {
  await pickExample(page, situationName('cushion'))
  const expected = await expectAnswerFor(page, situationAnswers('cushion'))
  expect(expected.outcome.rows.some((row) => isStock(row.id))).toBe(false)
  await expect(rows(page)).toHaveCount(expected.outcome.rows.length)
  await expect(page.locator('#outcome-return')).toHaveCount(0)
  await expect(chart(page).locator('.line-label')).toHaveCount(2)
})

test('how to do it: the facts as labelled rows, the same questions in the same order for every product', async ({
  page,
}) => {
  const expected = await expectAnswerFor(page, situationAnswers('goal'))
  const questions = expected.how[0].facts.map(({ question }) => question)
  expect(questions).toHaveLength(4)
  const withHow = expected.outcome.rows.flatMap((row, at) => {
    const how = howOf(expected, row.id)
    return how ? [{ at, how }] : []
  })
  expect(withHow.length).toBeGreaterThan(1)
  for (const { at, how } of withHow) {
    expect(how.facts.map(({ question }) => question)).toEqual(questions)
    await rowPress(page, at).click()
    // A question beside each answer, as many as the core gives.
    const facts = page.locator('section.how dl.facts')
    await expect(facts.locator('dt')).toHaveCount(questions.length)
    await expect(facts.locator('dd')).toHaveCount(questions.length)
  }
})

test(
  'where to buy: one row through a bank, as a range, and the difference in words; the gain named part by part',
  { tag: '@phone' },
  async ({ page }) => {
    await pickExample(page, situationName('salary'))
    const expected = await expectAnswerFor(page, situationAnswers('salary'))
    const at = expected.outcome.rows.findIndex((row) => isStock(row.id))
    await rowPress(page, at).click()
    const where = howOf(expected, expected.outcome.rows[at].id)!.whereToBuy!
    expect(where.reference, 'no bank named: the banks as one reference row').toBeDefined()
    const shown = page.locator('section.how .where')
    await expect(shown.locator('.row.reference')).toHaveCount(1)
    await expect(shown.locator('.row.reference .amount')).not.toBeEmpty()
    await expect(shown.locator('.difference')).toBeVisible()
    // The gain: every part's words, on its bar or beside it.
    const gain = expected.gain!
    const bars = page.locator('section.gain .bars li')
    await expect(bars).toHaveCount(gain.rows.length)
    for (const [index, row] of gain.rows.entries())
      for (const part of row.parts)
        await expect(bars.nth(index).locator('.words, .beside').filter({ hasText: part.label })).toHaveCount(1)
    await expect(page.locator('section.gain .takeaway')).toBeVisible()
  },
)

for (const id of ['goal', 'bonus', 'salary']) {
  test(
    `the horizon of "${id}": dates around the visitor's, theirs marked, what fits, how much, what was deposited`,
    { tag: '@phone' },
    async ({ page }) => {
      await pickExample(page, situationName(id))
      await expectAnswerFor(page, situationAnswers(id))
      const horizons = answer({
        answers: situationAnswers(id),
        choices: {},
        settings: guideSettings(),
        horizons: true,
      }).horizons!
      const section = page.locator('section.horizons')
      await expect(section).not.toHaveAttribute('aria-busy', 'true')
      await expect(section.locator('tbody tr')).toHaveCount(horizons.flatMap((row) => row.fits).length)
      await expect(section.locator('tbody th .when')).toHaveCount(horizons.length)
      expect(horizons.filter((row) => row.current)).toHaveLength(1)
      // The visitor's date is marked, on its own row.
      const at = horizons.findIndex((row) => row.current)
      await expect(section.locator('tbody th .yours')).toHaveCount(1)
      await expect(section.locator('tbody th').nth(at).locator('.yours')).toHaveCount(1)
      await expect(section.locator('td.amount .range')).toHaveCount(horizons.flatMap((row) => row.fits).length)
      // What was deposited, by its date: a column, or under the date on a phone.
      await expect(section.locator('td.deposited:visible, .deposits-here:visible')).toHaveCount(
        horizons.length,
      )
      await expect(section.getByText(/על כל ₪1/)).toHaveCount(0)
    },
  )
}
