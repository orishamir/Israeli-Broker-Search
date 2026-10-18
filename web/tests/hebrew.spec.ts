import { expect, test } from './fixtures'

// The page is in Hebrew, right to left, whatever the browser's language (the
// tests' is English), and so are the core's words.

test('the page is in Hebrew, right to left, in any browser', { tag: '@phone' }, async ({ page }) => {
  await expect(page.locator('html')).toHaveAttribute('dir', 'rtl')
  await expect(page.locator('html')).toHaveAttribute('lang', 'he')
  await expect(page).toHaveTitle('כמה יישאר לכם בסוף')
  // A word from the core, which answers in Hebrew only once told to.
  await expect(
    page.getByRole('radiogroup', { name: 'נייר ערך' }).getByText('קרן סל', { exact: true }),
  ).toBeVisible()
  // Nothing sticks out sideways in the mirrored layout.
  const pageWidth = await page.evaluate(() => document.documentElement.scrollWidth)
  expect(pageWidth).toBeLessThanOrEqual(await page.evaluate(() => innerWidth))
})
