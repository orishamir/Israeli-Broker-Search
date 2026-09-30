import { expect, test } from './fixtures'

// The page is in Hebrew, right to left, whatever the browser's language (the
// tests' is English): every text the page frames the numbers with, and the
// core's words too.

test('the page is in Hebrew, right to left, in any browser', { tag: '@phone' }, async ({ page }) => {
  await expect(page.locator('html')).toHaveAttribute('dir', 'rtl')
  await expect(page.locator('html')).toHaveAttribute('lang', 'he')
  await expect(page).toHaveTitle('עמלות המסחר, בריבית דריבית')
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('עמלות המסחר, בריבית דריבית')
  // The core's words: a security's name, a plan's, a column's tip.
  await expect(
    page.getByRole('radiogroup', { name: 'נייר ערך' }).getByText('קרן סל', { exact: true }),
  ).toBeVisible()
  await expect(page.getByRole('checkbox', { name: 'בנק לאומי' })).toBeVisible()
  await expect(page.locator('.stat.best .label')).toContainText('המשתלם ביותר: ')
  await expect(page.locator('th', { hasText: 'נשאר אחרי מס' })).toBeVisible()
  // A kind of fund, and how its money is taken out.
  await expect(
    page.getByRole('list', { name: 'קופת גמל להשקעה' }).getByRole('checkbox', { name: 'דמי ניהול ממוצעים' }),
  ).toBeChecked()
  await expect(page.getByRole('radiogroup', { name: 'משיכת הכסף' }).getByText('כקצבה')).toBeVisible()
  // Its tax rule, under its name and in its row.
  await expect(
    page.locator('aside').getByText('אין מס בקצבה מגיל 60; אחרת ממוסה כמו חשבון מסחר'),
  ).toBeVisible()
  await expect(page.locator('tbody .tax-note')).toHaveText('ממוסה כמו חשבון מסחר; אין מס בקצבה מגיל 60')
  // Nothing sticks out sideways in the mirrored layout.
  const pageWidth = await page.evaluate(() => document.documentElement.scrollWidth)
  expect(pageWidth).toBeLessThanOrEqual(await page.evaluate(() => innerWidth))
})

test("a plan's details are in Hebrew, tariff and caveats included", async ({ page }) => {
  await page.getByRole('button', { name: 'על לאומי · אונליין', exact: true }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog.getByRole('heading', { level: 2 })).toHaveText('אונליין')
  await expect(dialog).toContainText('תעריפון מ-29/06/2026')
  await expect(dialog).toContainText('קנייה או מכירה')
  await expect(dialog).toContainText('הפרשנות שלנו')
  await expect(dialog.getByRole('link', { name: /תעריפון \(PDF\)/ }).first()).toBeVisible()
})
