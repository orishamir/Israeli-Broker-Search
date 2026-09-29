import { compare, inputsOnPage, listed, rowOf, rowOfKey } from './core'
import { away, canvasPicture, choice, expect, recordDrawnText, test, type Page } from './fixtures'

// The growth chart: pinning and hovering, linked with the table, and zooming.
// What the lines look like is checked in src/lib/growth-chart.test.ts; here,
// a picture of the canvas is compared before and after, with no stored
// baseline.

/** Where the chart is, after scrolling it into view (so that later actions
 * don't scroll the page under the mouse). */
const chartBox = async (page: Page) => {
  await page.locator('.chart').scrollIntoViewIfNeeded()
  // The phone's bar with the best plan leaves the bottom of the screen,
  // where the zoom slider is, a frame or two after the scroll.
  await expect(page.locator('.best-bar')).toHaveCount(0)
  return (await page.locator('.chart').boundingBox())!
}

test('rows highlight on hover and pin on click, and the chart redraws them', async ({ page }) => {
  const chart = page.locator('.chart')
  const plain = await canvasPicture(chart)
  const meitav = rowOf(page, 'Meitav · Typical offer')
  await meitav.hover()
  await expect(meitav).toHaveClass(/highlighted/)
  await meitav.click()
  await rowOf(page, 'Leumi · Online').click()
  await away(page)
  await expect(meitav).toHaveClass(/highlighted/)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
  await expect.poll(() => canvasPicture(chart)).not.toBe(plain)

  const pinned = await canvasPicture(chart)
  await choice(page, 'Chart', 'Value').click()
  await expect.poll(() => canvasPicture(chart)).not.toBe(pinned)

  await page.getByRole('button', { name: 'Unpin all' }).click()
  await expect(meitav).not.toHaveClass(/highlighted/)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeHidden()
})

// A chart behind another view keeps its canvas, so coming back to it has
// nothing to draw. Switching back from the fee breakdown to Lost to fees
// used to draw the whole chart again, which took twice as long on a phone.
test('coming back to the chart over the years draws nothing again', async ({ page }) => {
  /** Resolves once a change has been drawn: charts draw just after the paint
   * that follows it, and this waits a frame longer. */
  const drawingDone = () =>
    page.evaluate(async () => {
      for (let frame = 0; frame < 3; frame++) await new Promise((resolve) => requestAnimationFrame(resolve))
      await new Promise((resolve) => setTimeout(resolve))
    })
  await choice(page, 'Chart', 'Fee breakdown').click()
  await drawingDone()
  const drawn = await recordDrawnText(page)
  await choice(page, 'Chart', 'Lost to fees').click()
  await drawingDone()
  expect(await drawn()).toEqual([])
})

test('the wheel zooms the years, dragging moves, and R resets', async ({ page }) => {
  const chart = page.locator('.chart')
  const box = await chartBox(page)
  const start = await canvasPicture(chart)
  await page.mouse.move(box.x + box.width * 0.7, box.y + box.height / 2)
  for (let i = 0; i < 6; i++) await page.mouse.wheel(0, -200)
  await expect.poll(() => canvasPicture(chart)).not.toBe(start)

  const zoomed = await canvasPicture(chart)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height / 2, { steps: 10 })
  await page.mouse.up()
  await expect.poll(() => canvasPicture(chart)).not.toBe(zoomed)

  await page.keyboard.press('r')
  await away(page)
  await expect.poll(() => canvasPicture(chart)).toBe(start)
})

/** Moves the mouse up the chart at year ~15 until it's on a line. */
async function moveOntoALine(page: Page) {
  const box = await chartBox(page)
  const highlightedRow = page.locator('tbody tr.highlighted')
  for (let y = box.y + box.height - 40; y > box.y + 20 && (await highlightedRow.count()) === 0; y -= 3) {
    await page.mouse.move(box.x + box.width * 0.72, y)
  }
  return highlightedRow
}

test('hovering a line highlights its row, clicking pins it', async ({ page }) => {
  const highlightedRow = await moveOntoALine(page)
  await expect(highlightedRow).toHaveCount(1)
  await page.mouse.down()
  await page.mouse.up()
  await away(page)
  await expect(highlightedRow).toHaveCount(1)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
})

test('tapping a line pins it', { tag: '@touch' }, async ({ page }) => {
  const box = await chartBox(page)
  // ECharts counts a tap within about 44 px of a line as touching it (its
  // "coarse pointer" mode), so the top of the plot, near the best lines.
  await page.touchscreen.tap(box.x + box.width * 0.72, box.y + 90)
  await expect(page.locator('tbody tr.pinned')).toHaveCount(1)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
})

// ECharts shows its first tooltip in the middle of the chart before moving
// it beside the finger: on a phone that reached past the screen, and the
// browser zoomed the page out to fit it, so the page jumped. Where it ends
// up must be inside the chart too: past its left edge, a right-to-left page
// could be scrolled sideways to it.
test('a first tap on the chart moves neither the page nor its zoom', { tag: '@touch' }, async ({ page }) => {
  const box = await chartBox(page)
  // How wide the page is laid out (wider when zoomed out) and where it's
  // scrolled to, every frame for a second.
  const views = page.evaluate(
    () =>
      new Promise<string[]>((resolve) => {
        const seen = new Set<string>()
        const start = performance.now()
        const record = () => {
          seen.add(`${innerWidth}×${innerHeight} at ${scrollX}, ${scrollY}`)
          if (performance.now() - start < 1000) requestAnimationFrame(record)
          else resolve([...seen])
        }
        record()
      }),
  )
  await page.touchscreen.tap(box.x + box.width * 0.2, box.y + 150)
  expect(await views).toHaveLength(1)
  // Headed by the time, in either view over the years.
  const tooltip = (await page.locator('.chart > div', { hasText: 'After' }).boundingBox())!
  expect(tooltip.x).toBeGreaterThanOrEqual(box.x)
  expect(tooltip.x + tooltip.width).toBeLessThanOrEqual(box.x + box.width)
})

test('on touch screens, the slider zooms the years', { tag: '@touch' }, async ({ page }) => {
  const chart = page.locator('.chart')
  const box = await chartBox(page)
  const start = await canvasPicture(chart)
  // The slider runs along the bottom, from 24 px to 90 px short of the right.
  const y = box.y + box.height - 8 - 16
  await page.mouse.move(box.x + 24, y)
  await page.mouse.down()
  await page.mouse.move(box.x + 24 + (box.width - 114) * 0.5, y, { steps: 10 })
  await page.mouse.up()
  await expect.poll(() => canvasPicture(chart)).not.toBe(start)
})

test("a plan's fees in the table open its breakdown, pinned", { tag: '@phone' }, async ({ page }) => {
  const { key, label } = listed('Leumi · Online')
  const { plans } = compare(await inputsOnPage(page))
  expect(plans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))?.outcome).toBeDefined()
  await rowOfKey(page, key)
    .getByRole('button', { name: /see what they went to$/ })
    .click()
  await expect(page.getByRole('radio', { name: 'Fee breakdown' })).toBeChecked()
  await expect(page.locator('h4', { hasText: 'Year by year' })).toContainText(label)
  await expect(rowOfKey(page, key)).toHaveClass(/pinned/)
  await expect(page.locator('#chart')).toBeInViewport()
})

test('the chart by deposit draws the plans, follows the pins, and its marker follows the deposit', async ({
  page,
}) => {
  // An expert's chart: offered with More options only.
  await expect(choice(page, 'Chart', 'By deposit')).toHaveCount(0)
  await page.getByLabel('More options', { exact: true }).check()
  await choice(page, 'Chart', 'By deposit').click()
  const chart = page.locator('.by-deposit')
  await expect(chart.locator('canvas')).toBeVisible()
  await expect(page.getByText('Click a row or a line to pin it')).toBeVisible()
  await expect(page.getByText('Wheel: zoom years')).toBeHidden()
  const plain = await canvasPicture(chart)

  const meitav = rowOf(page, 'Meitav · Typical offer')
  await meitav.click()
  await away(page)
  await expect(meitav).toHaveClass(/pinned/)
  await expect.poll(() => canvasPicture(chart)).not.toBe(plain)

  const pinned = await canvasPicture(chart)
  await page.getByLabel('Every month').fill('5000')
  await expect.poll(() => canvasPicture(chart)).not.toBe(pinned)
})
