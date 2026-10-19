import { alternativesFor, NO_OVERRIDES, recommendation } from './core'
import {
  away,
  canvasPicture,
  enter,
  expect,
  expectAnswerFor,
  settledPicture,
  test,
  type Page,
} from './fixtures'
import { arenaOf } from '../src/lib/kinds'
import { t } from '../src/lib/text'

// The alternatives: the chart of what the same money would be in the other
// kinds, and the rows under it, linked by hovering and pinning. What the
// lines are is checked in src/lib/alternatives-chart.test.ts.

const fiveYearsFixed = { now: 50_000, monthly: 0, horizon: 5, flexibility: 'Fixed' } as const

const chart = (page: Page) => page.locator('.alternatives .chart')
const rowsOf = (page: Page) => page.locator('.alternatives li.row')

/** Enters the five fixed years and waits until the chart shows them: it holds
 * the last answer while a number is typed, and for the worker's. */
async function enterAndWaitForChart(page: Page, flexibility?: 'CanWait') {
  const answers = await enter(page, { ...fiveYearsFixed, ...(flexibility && { flexibility }) })
  const { band } = alternativesFor(answers)
  await expect(page.locator('.alternatives .caption')).toContainText(band!.sentence)
  return answers
}

/** Moves the mouse up the chart at its right side until it is on a line
 * (some row is then hovered); false if none was found. */
async function moveOntoALine(page: Page) {
  await chart(page).scrollIntoViewIfNeeded()
  const box = (await chart(page).boundingBox())!
  const hovered = page.locator('.alternatives li.row.hovered')
  for (let y = box.y + box.height - 40; y > box.y + 20 && (await hovered.count()) === 0; y -= 3) {
    await page.mouse.move(box.x + box.width * 0.72, y)
  }
  return hovered
}

test(
  'a row for every kind drawn, what fits first, and the band explained',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, fiveYearsFixed)
    await expectAnswerFor(page, answers)
    const drawn = alternativesFor(answers)
    await expect(rowsOf(page)).toHaveCount(drawn.series.length)
    // What fits has no button to choose it: it is already chosen.
    await expect(page.locator('.alternatives .row .choose:disabled')).toHaveCount(1)
    await expect(rowsOf(page).first().locator('.choose')).toBeDisabled()
    // A stock index fund's band comes with the core's sentence.
    expect(drawn.band, 'the core draws a band for a fixed date in 5 years').toBeDefined()
    await expect(page.locator('.alternatives .caption')).toContainText(drawn.band!.sentence)
  },
)

test('hovering a row redraws the chart, and leaving it restores the chart', async ({ page }) => {
  await enterAndWaitForChart(page)
  await expect(rowsOf(page).nth(1)).toBeVisible()
  const plain = await settledPicture(chart(page))
  const row = rowsOf(page).nth(1)
  await row.hover()
  await expect(row).toHaveClass(/hovered/)
  await expect.poll(() => canvasPicture(chart(page))).not.toBe(plain)
  await away(page)
  await expect(row).not.toHaveClass(/hovered/)
  await expect.poll(() => canvasPicture(chart(page))).toBe(plain)
})

test('hovering a line lights its row, and clicking it pins the row', async ({ page }) => {
  await enterAndWaitForChart(page)
  await expect(chart(page).locator('canvas')).toBeVisible()
  const plain = await settledPicture(chart(page))
  const hovered = await moveOntoALine(page)
  await expect(hovered).toHaveCount(1)
  await page.mouse.down()
  await page.mouse.up()
  await away(page)
  await expect(page.locator('.alternatives li.row.pinned')).toHaveCount(1)
  await expect.poll(() => canvasPicture(chart(page))).not.toBe(plain)
  // Clicking the same line again lets go.
  await moveOntoALine(page)
  await page.mouse.down()
  await page.mouse.up()
  await away(page)
  await expect(page.locator('.alternatives li.row.pinned')).toHaveCount(0)
})

test(
  '"continue anyway" goes on with that kind, and the guide can be asked again',
  { tag: '@phone' },
  async ({ page }) => {
    const answers = await enter(page, fiveYearsFixed)
    const guide = await expectAnswerFor(page, answers)
    const other = alternativesFor(answers).series.find(({ recommended }) => !recommended)!
    await page.getByRole('button', { name: t.continueWith(other.name) }).click()
    const overrides = { arena: arenaOf(other.kind), kind: other.kind }
    const chosen = await expectAnswerFor(page, answers, overrides)
    expect(chosen.kind.kind).toEqual(other.kind)
    // The chosen kind is what fits now: its row has no button.
    await expect(page.getByRole('button', { name: t.continueWith(other.name) })).toHaveCount(0)
    await page.getByRole('button', { name: t.backToRecommendation }).click()
    await expectAnswerFor(page, answers, NO_OVERRIDES)
    expect(recommendation(answers).kind.kind).toEqual(guide.kind.kind)
  },
)

test('the donut has a tab for the answer and each kind drawn beside it', async ({ page }) => {
  const answers = await enter(page, { ...fiveYearsFixed, flexibility: 'CanWait' })
  const guide = await expectAnswerFor(page, answers)
  const tabs = page.locator('section.donut .tabs button')
  const withBreakdown = guide.alternatives.filter(({ breakdown }) => breakdown)
  await expect(tabs).toHaveCount(1 + withBreakdown.length)
  const donut = page.locator('section.donut .chart')
  await expect.poll(() => canvasPicture(donut)).toBeTruthy()
  const first = await canvasPicture(donut)
  await tabs.nth(1).click()
  await expect(tabs.nth(1)).toHaveAttribute('aria-pressed', 'true')
  await expect.poll(() => canvasPicture(donut)).not.toBe(first)
})
