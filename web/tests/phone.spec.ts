import { answerFor, decisionsOf, factsLine, situationAnswers } from './core'
import type { DecisionData } from './core'
import {
  answerCard,
  decisionGroup,
  expect,
  expectAnswerFor,
  pickExample,
  situationName,
  test,
  type Locator,
  type Page,
} from './fixtures'
import { t } from '../src/lib/text'

// The phone's flow (REDESIGN.md §4.3): the facts as one line with "לשנות",
// the answer on the first screen, the bar while the fields are open, no
// scrolling by itself, the pill for a decision that opens out of sight, and
// a decision as a table. Only on phones (@touch: the Galaxy, the iPhone; the
// narrowest iPhone runs this whole file). The words are the core's and
// `t`'s; here, what only a browser shows: what moves, and what doesn't.

const questions = (page: Page) => page.locator('section.questions')
const toggle = (page: Page) => questions(page).locator('.facts-toggle')
const bar = (page: Page) => page.locator('.dock aside.bar')
const pill = (page: Page) => page.locator('.dock .pill')
/** Where an element is on the screen. */
const box = async (locator: Locator) => (await locator.boundingBox())!
const scrolled = (page: Page) => page.evaluate(() => scrollY)

test(
  'the facts are one line, and the answer starts on the first screen',
  { tag: '@touch' },
  async ({ page }) => {
    const answers = situationAnswers('goal')
    await expect(questions(page).locator('.facts-line')).toHaveText(factsLine(answers))
    await expect(page.locator('#now')).toBeHidden()
    await expect(toggle(page)).toHaveAccessibleName(t.factsChange)
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'false')
    await expect(answerCard(page).locator('.chip')).toBeInViewport({ ratio: 1 })
    await expect(answerCard(page).locator('#answer-title')).toBeInViewport({ ratio: 1 })
  },
)

test(
  '"לשנות" opens the fields in place; typing shows the bar with before and after, and nothing scrolls',
  { tag: '@touch' },
  async ({ page }) => {
    const at = await box(toggle(page))
    await toggle(page).click()
    await expect(page.locator('#monthly')).toBeVisible()
    await expect(toggle(page)).toHaveAccessibleName(t.factsDone)
    expect(await box(toggle(page))).toEqual(at)
    expect(await scrolled(page)).toBe(0)
    // The bar shows the answer as it is; nothing to compare with yet.
    await expect(bar(page)).toBeVisible()
    await expect(bar(page).locator('.before')).toHaveCount(0)
    const before = await bar(page).locator('.amount').innerText()

    const field = page.locator('#monthly')
    const fieldAt = await box(field)
    await field.fill('2500')
    const answers = { ...situationAnswers('goal'), monthly: 2500 }
    await expectAnswerFor(page, answers)
    await expect(bar(page).locator('.before')).toHaveText(t.beforeChange(before))
    await expect(bar(page).locator('.amount')).not.toHaveText(before)
    const after = await bar(page).locator('.amount').innerText()
    // A screen reader hears one sentence.
    await expect(page.locator('[aria-live]', { hasText: t.answerNow(after) })).toHaveCount(1)
    expect(await box(field)).toEqual(fieldAt)
    expect(await scrolled(page)).toBe(0)
    // Another change: "before" is still what it was when "לשנות" was tapped.
    await field.fill('3000')
    await expectAnswerFor(page, { ...answers, monthly: 3000 })
    await expect(bar(page).locator('.before')).toHaveText(t.beforeChange(before))
    expect(await scrolled(page)).toBe(0)
  },
)

test('"סיום" closes the fields where they are, and nothing scrolls', { tag: '@touch' }, async ({ page }) => {
  await toggle(page).click()
  await page.locator('#now').fill('80000')
  const answers = { ...situationAnswers('goal'), now: 80000 }
  await expectAnswerFor(page, answers)
  const at = await box(toggle(page))
  const y = await scrolled(page)
  await toggle(page).click()
  await expect(page.locator('#now')).toBeHidden()
  await expect(bar(page)).toHaveCount(0)
  await expect(questions(page).locator('.facts-line')).toHaveText(factsLine(answers))
  expect(await box(toggle(page))).toEqual(at)
  expect(await scrolled(page)).toBe(y)
  // Opened again, nothing is "before" yet.
  await toggle(page).click()
  await expect(bar(page).locator('.before')).toHaveCount(0)
})

test(
  '"לתשובה" closes the fields and takes the visitor to the answer',
  { tag: '@touch' },
  async ({ page }) => {
    await toggle(page).click()
    await page.locator('#monthly').fill('1000')
    await expectAnswerFor(page, { ...situationAnswers('goal'), monthly: 1000 })
    await bar(page).getByRole('button', { name: t.toAnswer }).click()
    await expect(page.locator('#monthly')).toBeHidden()
    await expect(bar(page)).toHaveCount(0)
    await expect(answerCard(page).locator('.chip')).toBeInViewport()
    const top = await answerCard(page).evaluate((card) => card.getBoundingClientRect().top)
    expect(Math.abs(top)).toBeLessThan(40)
  },
)

test(
  'a decision that opens below the screen has the pill; the pressed button stays, and the pill takes the visitor there',
  { tag: '@touch' },
  async ({ page }) => {
    await pickExample(page, situationName('bonus'))
    const answers = situationAnswers('bonus')
    const before = await expectAnswerFor(page, answers)
    const first = decisionsOf(before)[0]
    const fixed = first.decision.options.find(
      (option) => 'Flexibility' in option.pick && option.pick.Flexibility === 'Fixed',
    )!
    const opened = decisionsOf(answerFor({ ...answers, flexibility: 'Fixed' })).find(
      ({ decision }) => decision.id !== first.decision.id,
    )!
    const button = decisionGroup(page, first.decision).getByRole('button', {
      name: fixed.button,
      exact: true,
    })
    // The button near the screen's bottom, so what it opens is below.
    await button.scrollIntoViewIfNeeded()
    await page.evaluate(
      (by) => scrollBy({ top: by, behavior: 'instant' }),
      (await box(button)).y + 60 - page.viewportSize()!.height / 2,
    )
    const at = await box(button)
    await button.click()
    await expectAnswerFor(page, { ...answers, flexibility: 'Fixed' })
    await expect(pill(page)).toHaveText(t.newDecision(opened.decision.question))
    expect((await box(button)).y).toBeCloseTo(at.y, 0)
    await pill(page).click()
    await expect(decisionGroup(page, opened.decision)).toBeInViewport()
    await expect(pill(page)).toHaveCount(0)
  },
)

test('the pill goes at the visitor’s next change', { tag: '@touch' }, async ({ page }) => {
  await pickExample(page, situationName('bonus'))
  const answers = situationAnswers('bonus')
  const first = decisionsOf(await expectAnswerFor(page, answers))[0]
  const fixed = first.decision.options.find(
    (option) => 'Flexibility' in option.pick && option.pick.Flexibility === 'Fixed',
  )!
  const button = decisionGroup(page, first.decision).getByRole('button', { name: fixed.button, exact: true })
  await button.scrollIntoViewIfNeeded()
  await page.evaluate(
    (by) => scrollBy({ top: by, behavior: 'instant' }),
    (await box(button)).y + 60 - page.viewportSize()!.height / 2,
  )
  await button.click()
  await expect(pill(page)).toBeVisible()
  const growth = first.decision.options.find((option) => option !== fixed)!
  await decisionGroup(page, first.decision).getByRole('button', { name: growth.button, exact: true }).click()
  await expect(pill(page)).toHaveCount(0)
})

/** Two options: a table whose columns are the options, the buttons side by side. */
async function expectColumns(group: Locator, decision: DecisionData) {
  const table = group.locator('table.table')
  await expect(table.locator('th[scope=col]')).toHaveText(decision.options.map((option) => option.title))
  await expect(table.locator('.label th').first()).toHaveText(t.tableEnd)
  const [a, b] = await Promise.all(
    decision.options.map((option) => box(table.getByRole('button', { name: option.button, exact: true }))),
  )
  expect(a.y).toBeCloseTo(b.y, 0)
}

/** Three or more: a row each, the amounts in one column at the end. */
async function expectRows(group: Locator, decision: DecisionData) {
  const rows = group.locator('.options.rows .option')
  await expect(rows).toHaveCount(decision.options.length)
  const ends = await rows
    .locator('.end')
    .evaluateAll((all) => all.map((end) => end.getBoundingClientRect().left))
  expect(new Set(ends.map(Math.round)).size).toBe(1)
}

/** The decision's phone layout, and the rest behind "עוד", opening in place. */
async function expectPhoneDecision(page: Page, decision: DecisionData) {
  const group = decisionGroup(page, decision)
  if (decision.options.length === 2) await expectColumns(group, decision)
  else await expectRows(group, decision)
  const more = group.getByRole('button', { name: t.moreAboutOptions(decision.options.length) })
  if ((await more.count()) === 0) return
  await expect(group.locator('.rest-option')).toHaveCount(0)
  await more.click()
  await expect(group.locator('.rest-option').first()).toBeVisible()
}

test('two options are a table of two columns; three are a row each', { tag: '@touch' }, async ({ page }) => {
  let shown = 0
  for (const id of ['goal', 'bonus', 'cushion', 'existing']) {
    await pickExample(page, situationName(id))
    const answer = await expectAnswerFor(page, situationAnswers(id))
    for (const { decision } of decisionsOf(answer)) {
      await expectPhoneDecision(page, decision)
      shown += decision.options.length
    }
  }
  expect(shown, 'two-option and three-option decisions were seen').toBeGreaterThan(4)
})
