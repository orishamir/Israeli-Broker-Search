import { answerFor, situationAnswers } from './core'
import type { AnswerData, Kind } from './core'
import { pickExample, decide, expect, expectAnswerFor, situationName, test, type Page } from './fixtures'
import { kindKey } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// "מה יהיה לכם ב‑…" (REDESIGN.md §2.6) and "איך עושים את זה" (§2.1 item 4):
// the chart's rows are the options; tapping one shows its how-to, rows the
// visitor's choices leave out dim, and the stock row's yearly return is set
// in place. Where each mark goes is a unit test (`outcome.test.svelte.ts`).

const chart = (page: Page) => page.locator('section.outcome')
const rows = (page: Page) => chart(page).locator('.row:not(.axis)')
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
