import { aboutPlan, closePlans, expect, openPlans, plansList, test } from './fixtures'

// The page is in Hebrew, right to left, whatever the browser's language (the
// tests' is English): every text the page frames the numbers with, and the
// core's words too.

test('the page is in Hebrew, right to left, in any browser', { tag: '@phone' }, async ({ page }) => {
  await expect(page.locator('html')).toHaveAttribute('dir', 'rtl')
  await expect(page.locator('html')).toHaveAttribute('lang', 'he')
  await expect(page).toHaveTitle('כמה יישאר לכם בסוף')
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('כמה יישאר לכם בסוף')
  // The two calculators, named by how long the money is kept.
  await expect(page.getByRole('radio', { name: 'השקעה לטווח ארוך' })).toBeChecked()
  await expect(page.getByRole('radio', { name: 'חיסכון לטווח קצר' })).toHaveAccessibleDescription(
    'פיקדון בבנק, קרן כספית',
  )
  // The core's words: a security's name, a plan's, a column's tip.
  await expect(
    page.getByRole('radiogroup', { name: 'נייר ערך' }).getByText('קרן סל', { exact: true }),
  ).toBeVisible()
  await expect(
    page.getByRole('list', { name: 'מה משווים' }).getByText('לאומי', { exact: true }),
  ).toBeVisible()
  await expect(page.locator('.stat.best .label')).toContainText('המשתלם ביותר: ')
  await expect(page.locator('th', { hasText: 'נשאר אחרי מס' })).toBeVisible()
  // A kind of fund, how its money is taken out, and its tax rule in its row.
  await expect(page.getByRole('radiogroup', { name: 'משיכת הכסף' }).getByText('כקצבה')).toBeVisible()
  await expect(page.locator('tbody .tax-note')).toHaveText('ממוסה כמו חשבון מסחר; אין מס בקצבה מגיל 60')
  // In the list of plans: a bank, and the fund with its tax rule under its name.
  await openPlans(page)
  await expect(plansList(page).getByRole('checkbox', { name: 'בנק לאומי' })).toBeVisible()
  await expect(plansList(page).getByText('אין מס בקצבה מגיל 60; אחרת ממוסה כמו חשבון מסחר')).toBeVisible()
  await plansList(page).getByRole('button', { name: 'קופת גמל להשקעה', exact: true }).click()
  await expect(
    plansList(page)
      .getByRole('list', { name: 'קופת גמל להשקעה' })
      .getByRole('checkbox', { name: 'דמי ניהול ממוצעים' }),
  ).toBeChecked()
  await closePlans(page)
  // Nothing sticks out sideways in the mirrored layout.
  const pageWidth = await page.evaluate(() => document.documentElement.scrollWidth)
  expect(pageWidth).toBeLessThanOrEqual(await page.evaluate(() => innerWidth))
})

test("a plan's details are in Hebrew, tariff and caveats included", async ({ page }) => {
  await aboutPlan(page, 'Leumi · Online')
  const dialog = page.getByRole('dialog', { name: 'אונליין' })
  await expect(dialog.getByRole('heading', { level: 2 })).toHaveText('אונליין')
  await expect(dialog).toContainText('תעריפון מ-29/06/2026')
  await expect(dialog).toContainText('קנייה או מכירה')
  await expect(dialog).toContainText('הפרשנות שלנו')
  await expect(dialog.getByRole('link', { name: /תעריפון \(PDF\)/ }).first()).toBeVisible()
})
