import { expect, test, type Page } from './fixtures'

test.beforeEach(async ({ page }) => {
  await expect(page.locator('.chart canvas')).toBeVisible()
})

/** Where the chart is, after scrolling it into view (so that later
 * screenshots don't scroll the page under the mouse). */
const chartBox = async (page: Page) => {
  await page.locator('.chart').scrollIntoViewIfNeeded()
  return (await page.locator('.chart').boundingBox())!
}

test('rows highlight on hover and pin on click', async ({ page }) => {
  const meitav = page.locator('tbody tr', { hasText: 'Meitav Trade' })
  await meitav.hover()
  await expect(meitav).toHaveClass(/highlighted/)
  await meitav.click()
  await page.locator('tbody tr', { hasText: 'Bank Leumi' }).click()
  await page.mouse.move(0, 0)
  await expect(meitav).toHaveClass(/highlighted/)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
  await expect(page.locator('.chart')).toHaveScreenshot('pinned.png')

  await page.getByRole('radio', { name: 'Lost to fees' }).click({ force: true })
  await expect(page.locator('.chart')).toHaveScreenshot('pinned-lost.png')

  await page.getByRole('button', { name: 'Unpin all' }).click()
  await expect(meitav).not.toHaveClass(/highlighted/)
})

test('the wheel zooms the years and R resets', async ({ page, isMobile }) => {
  test.skip(isMobile, 'touch screens zoom with the slider instead')
  const box = await chartBox(page)
  await page.mouse.move(box.x + box.width * 0.7, box.y + box.height / 2)
  for (let i = 0; i < 6; i++) await page.mouse.wheel(0, -200)
  await expect(page.locator('.chart')).toHaveScreenshot('zoomed.png')

  await page.mouse.down()
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height / 2, { steps: 10 })
  await page.mouse.up()
  await expect(page.locator('.chart')).toHaveScreenshot('panned.png')

  await page.keyboard.press('r')
  await expect(page.locator('.chart')).toHaveScreenshot('reset.png')
})

test('hovering a line highlights its row, clicking pins it', async ({ page, isMobile }) => {
  test.skip(isMobile, 'no hovering on touch screens; see the tapping test')
  const box = await chartBox(page)
  const highlightedRow = page.locator('tbody tr.highlighted')
  // Sweep up the chart at year ~15 until the mouse is on a line.
  for (let y = box.y + box.height - 40; y > box.y + 20; y -= 3) {
    await page.mouse.move(box.x + box.width * 0.72, y)
    if ((await highlightedRow.count()) > 0) break
  }
  await expect(highlightedRow).toHaveCount(1)
  await expect(page.locator('.chart')).toHaveScreenshot('line-hover.png')
  await page.mouse.down()
  await page.mouse.up()
  await page.mouse.move(0, 0)
  await expect(highlightedRow).toHaveCount(1)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
})

test('tapping a line pins it', async ({ page, isMobile }) => {
  test.skip(!isMobile, 'touch screens only')
  const box = await chartBox(page)
  // ECharts counts a tap within about 44 px of a line as touching it (its
  // "coarse pointer" mode), so the top of the plot, near the best lines.
  await page.touchscreen.tap(box.x + box.width * 0.72, box.y + 90)
  await expect(page.locator('tbody tr.pinned')).toHaveCount(1)
  await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
})

test('on touch screens, the slider zooms the years', async ({ page, isMobile }) => {
  test.skip(!isMobile, 'the wheel zooms instead')
  const box = await chartBox(page)
  // The slider runs along the bottom, from 24 px to 90 px short of the right.
  const y = box.y + box.height - 8 - 16
  await page.mouse.move(box.x + 24, y)
  await page.mouse.down()
  await page.mouse.move(box.x + 24 + (box.width - 114) * 0.5, y, { steps: 10 })
  await page.mouse.up()
  await expect(page.locator('.chart')).toHaveScreenshot('slider.png')
})
