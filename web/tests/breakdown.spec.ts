import { expect, fitScreenToPage, test, type Page } from './fixtures'

test.beforeEach(async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  await page.mouse.move(0, 0)
})

const card = (page: Page) => page.locator('section.card', { has: page.locator('.bars') })
const overTime = (page: Page) => page.locator('h4', { hasText: 'Year by year' })

/** Compares the plans by one fee: its button in the legend. */
async function compare(page: Page, fee: string) {
  await page.getByRole('group', { name: 'Compare the plans by' }).getByRole('button', { name: fee }).click()
  await page.mouse.move(0, 0)
}

test("each plan's fees, by kind, and the best plan's over time", async ({ page }) => {
  await expect(overTime(page)).toContainText('New customers')
  await fitScreenToPage(page)
  await expect(card(page)).toHaveScreenshot('all.png')
})

test('comparing one fee lines it up at the start of the bars', async ({ page }) => {
  await compare(page, 'Custody')
  await fitScreenToPage(page)
  await expect(card(page)).toHaveScreenshot('custody.png')
})

test('clicking a bar pins its plan and shows it over time', async ({ page, isMobile }) => {
  // In view, or the click lands outside the screen.
  await page.locator('.bars').scrollIntoViewIfNeeded()
  const box = (await page.locator('.bars').boundingBox())!
  // The second bar (fewest fees first): US 1¢/share. Just right of the names.
  const x = box.x + (box.width < 500 ? 84 : 120) + 24
  const y = box.y + 4 + 34.5 * 1.5
  if (isMobile) await page.touchscreen.tap(x, y)
  else await page.mouse.click(x, y)
  await page.mouse.move(0, 0)
  await expect(page.locator('tbody tr.pinned')).toContainText('US 1¢/share')
  await expect(overTime(page)).toContainText('US 1¢/share')
})

test("scrolling the page with a finger on the chart doesn't pick a plan", async ({
  page,
  browserName,
  isMobile,
}) => {
  test.skip(!isMobile || browserName !== 'chromium', 'real touch events need a Chromium phone')
  await page.locator('.bars').scrollIntoViewIfNeeded()
  const box = (await page.locator('.bars').boundingBox())!
  const [x, y] = [box.x + box.width / 2, box.y + box.height / 2]
  const cdp = await page.context().newCDPSession(page)
  const touch = (type: string, touchPoints: { x: number; y: number }[]) =>
    cdp.send('Input.dispatchTouchEvent', { type: type as 'touchStart', touchPoints })
  await touch('touchStart', [{ x, y }])
  for (let step = 1; step <= 8; step++) await touch('touchMove', [{ x, y: y - step * 25 }])
  await touch('touchEnd', [])
  await expect(overTime(page)).toContainText('New customers')
  await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
})

test('plans without a price are named, not just left out', async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Europe').click()
  await expect(page.getByText('Not offered for an ETF bought in Europe:')).toContainText('US 1¢/share')
})
