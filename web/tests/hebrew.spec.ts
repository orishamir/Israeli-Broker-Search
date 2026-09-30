import { expect, fitScreenToPage, test } from './fixtures'

// The page in Hebrew: a Hebrew browser gets it right to left, every text the
// page frames the numbers with in Hebrew, and the core's words in Hebrew too.
// The switch in the header goes back to English, keeping the comparison.
test.use({ locale: 'he-IL' })

test('a Hebrew browser gets the page in Hebrew, right to left', { tag: '@phone' }, async ({ page }) => {
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

// Beside the plan, on the side of the results, which is the left here.
test("a plan's preview opens left of it, on the screen", async ({ page }) => {
  const plan = page.getByRole('list', { name: 'בנק לאומי' }).getByRole('listitem').first()
  await plan.hover()
  const preview = page.locator('.preview')
  await expect(preview).toBeInViewport({ ratio: 1 })
  const [planBox, previewBox] = [(await plan.boundingBox())!, (await preview.boundingBox())!]
  expect(previewBox.x + previewBox.width).toBeLessThan(planBox.x)
})

test('the switch goes back to English and keeps the comparison', async ({ page }) => {
  await page.getByLabel('כל חודש').fill('3,500')
  await page.getByRole('button', { name: 'החלפה לאנגלית' }).click()
  await expect(page.locator('html')).toHaveAttribute('dir', 'ltr')
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Broker fees, compounded')
  await expect(page.getByLabel('Every month')).toHaveValue('3,500')
  // Remembered: the next visit is in English too, whatever the browser says.
  await page.reload()
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Broker fees, compounded')
})

test('the start page in Hebrew', { tag: ['@phone', '@pictures'] }, async ({ page }) => {
  await fitScreenToPage(page)
  await expect(page).toHaveScreenshot('start-he.png', {
    fullPage: true,
    mask: [
      page.locator('td.amount'),
      page.locator('.stat .value'),
      page.locator('.stat.best .label'),
      page.locator('.stat.best .around'),
      page.locator('.best-bar'),
      page.locator('.chart'),
      page.locator('.date'),
    ],
  })
})
