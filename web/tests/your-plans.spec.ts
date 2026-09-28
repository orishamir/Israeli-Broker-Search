import { expect, test, type Page } from './fixtures'

const card = (page: Page) => page.locator('section.card', { hasText: 'Your plans' })
const editor = (page: Page) => page.getByRole('dialog')
const rows = (page: Page) => page.locator('tbody tr')

/** Opens a draft copy of a listed plan with its ✎. */
async function copy(page: Page, plan: string) {
  await page.getByRole('button', { name: `Change a copy of ${plan}'s fees` }).click()
  await expect(editor(page)).toBeVisible()
}

/** Copies Pepper, charges $2 an order instead of $4, and adds it. */
async function addDeal(page: Page) {
  await copy(page, 'Pepper')
  await editor(page).getByLabel('Buy or sell: price').fill('2')
  await editor(page).getByRole('button', { name: 'Add plan' }).click()
  await expect(editor(page)).toBeHidden()
}

test("✎ copies a plan to change, showing the original's fees", async ({ page }) => {
  await expect(card(page)).toContainText("Think you can get lower fees, or use a broker that isn't listed?")
  await copy(page, 'Pepper')
  const dialog = page.getByRole('dialog', { name: 'Pepper, your deal' })
  await expect(dialog).toContainText('Copy of Pepper · Bank Leumi')
  await expect(dialog.getByLabel('Buy or sell: price')).toHaveValue('4')

  await dialog.getByLabel('Buy or sell: price').fill('2')
  await expect(dialog).toContainText('Pepper: $4 per order')
  await page.mouse.move(0, 0)
  await expect(dialog).toHaveScreenshot('draft.png')

  await dialog.getByRole('button', { name: 'Add plan' }).click()
  await expect(dialog).toBeHidden()
  await expect(card(page)).toContainText('Copy of Pepper · Bank Leumi')
  const row = rows(page).filter({ hasText: 'Pepper, your deal' })
  await expect(row).toContainText('Your deal · Bank Leumi')
  await expect(row.locator('.mark')).toHaveClass(/yours/)
  await expect(rows(page)).toHaveCount(9)
})

test('↺ goes back to the original fee', async ({ page }) => {
  await copy(page, 'Pepper')
  await editor(page).getByLabel('Buy or sell: price').fill('2')
  await editor(page).getByRole('button', { name: "Back to Pepper's fee" }).click()
  await expect(editor(page).getByLabel('Buy or sell: price')).toHaveValue('4')
  await expect(editor(page)).not.toContainText('Pepper: $4 per order')
})

test('a bad fee says why and keeps the last good one', async ({ page }) => {
  await copy(page, 'Pepper')
  await editor(page).getByLabel('Buy or sell: price').fill('-3')
  await expect(editor(page)).toContainText("Fees can't be negative")
  // Still Pepper's fee, so nothing to go back to.
  await expect(editor(page)).not.toContainText('Pepper: $4 per order')
})

test('Cancel adds nothing', async ({ page }) => {
  await copy(page, 'Pepper')
  await editor(page).getByRole('button', { name: 'Cancel' }).click()
  await expect(editor(page)).toBeHidden()
  await expect(card(page)).toContainText('Think you can get lower fees')
  await expect(rows(page)).toHaveCount(8)
})

test('deleting asks first, then unticks and unpins', async ({ page }) => {
  await addDeal(page)
  await rows(page).filter({ hasText: 'Pepper, your deal' }).click()
  await expect(page.locator('tbody tr.pinned')).toHaveCount(1)

  await page.getByRole('button', { name: 'Change Pepper, your deal' }).click()
  await editor(page).getByRole('button', { name: 'Delete plan' }).click()
  await expect(editor(page)).toContainText('Delete “Pepper, your deal”?')
  await editor(page).getByRole('button', { name: 'Keep' }).click()
  await editor(page).getByRole('button', { name: 'Delete plan' }).click()
  await editor(page).getByRole('button', { name: 'Delete', exact: true }).click()
  await expect(editor(page)).toBeHidden()
  await expect(rows(page)).toHaveCount(8)
  await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
  await expect(card(page)).toContainText('Think you can get lower fees')
})

test('your plans and the editor view stay after a reload', async ({ page }) => {
  await addDeal(page)
  await page.getByRole('button', { name: 'Change Pepper, your deal' }).click()
  await editor(page).getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await editor(page).getByRole('button', { name: 'Done' }).click()

  await page.reload()
  await expect(rows(page).filter({ hasText: 'Pepper, your deal' })).toContainText('Your deal · Bank Leumi')
  await page.getByRole('button', { name: 'Change Pepper, your deal' }).click()
  await expect(editor(page).getByRole('radio', { name: 'Full price list' })).toBeChecked()
  // Still $2, not Pepper's $4.
  await expect(editor(page)).toContainText('Pepper: $4 per order')
})

test('a new plan, with a broker name', async ({ page }) => {
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  const dialog = page.getByRole('dialog', { name: 'Your plan' })
  await dialog.getByPlaceholder('optional').fill('IBI')
  await dialog.getByLabel('Buy or sell: price').fill('0.1')
  await dialog.getByRole('button', { name: 'Add plan' }).click()
  await expect(rows(page).filter({ hasText: 'Your plan' })).toContainText('IBI · your own')

  // The next one gets the next free name.
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  await expect(page.getByRole('dialog', { name: 'Your plan 2' })).toBeVisible()
})

test('a fee can be added where the original offers none', async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Europe').click()
  await copy(page, 'US 1¢/share')
  await expect(editor(page)).toContainText('not offered')
  await editor(page).getByRole('button', { name: '+ Add a fee' }).click()
  await expect(editor(page).getByLabel('Buy or sell: price')).toHaveValue('0')
})

test('the full price list: rows, what they cover, and overlaps', async ({ page }) => {
  await card(page).getByRole('button', { name: '+ New plan' }).click()
  const dialog = editor(page)
  await dialog.getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()

  // A row for what the user buys, which takes it from the broader row.
  await dialog.getByRole('button', { name: '+ Row' }).first().click()
  await expect(dialog.getByRole('button', { name: 'ETF on USA ▾' })).toBeVisible()
  await expect(dialog).toContainText('except ETF on USA (its own row above)')

  // Covering all of the USA and Europe makes the other row useless. The
  // ticks apply when the checklist closes.
  await dialog.getByRole('button', { name: 'ETF on USA ▾' }).click()
  const checklist = page.getByRole('group', { name: 'What it covers' })
  await checklist.getByLabel('ETF').uncheck()
  await checklist.getByLabel('Europe').check()
  await expect(dialog).not.toContainText('Never used')
  await page.mouse.move(0, 0)
  await expect(dialog).toHaveScreenshot('checklist.png')
  await dialog.getByRole('button', { name: 'ETF on USA ▾' }).click()
  await expect(checklist).toBeHidden()
  await expect(dialog).toContainText('Never used: more specific rows cover all of it.')

  await dialog.getByRole('button', { name: 'Remove the row for Anything on USA, Europe' }).first().click()
  await expect(dialog).not.toContainText('Never used')
  // Esc closes an open checklist, not the whole editor.
  await dialog.getByRole('button', { name: 'Anything on USA, Europe ▾' }).click()
  await page.keyboard.press('Escape')
  await expect(checklist).toBeHidden()
  await expect(dialog).toBeVisible()
})

test('the simple view shows the fees for every security and exchange', async ({ page }) => {
  for (const exchange of ['Tel Aviv', 'USA', 'Europe']) {
    await page.getByRole('radiogroup', { name: 'Exchange' }).getByText(exchange).click()
    for (const security of ['ETF', 'Mutual fund', 'Bond', 'Stock']) {
      await page.getByRole('radiogroup', { name: 'Security' }).getByText(security, { exact: true }).click()
      await copy(page, 'Online')
      const dialog = editor(page)
      await expect(dialog, `${security} on ${exchange}`).toContainText(`bought in`)
      await expect(dialog.getByLabel('Buy or sell: price')).toBeVisible()
      await expect(dialog.getByLabel('Custody: rate')).toBeVisible()
      // Nothing is converted on Tel Aviv.
      await expect(dialog.getByLabel('Conversion: fee')).toHaveCount(exchange === 'Tel Aviv' ? 0 : 1)
      await dialog.getByRole('button', { name: 'Cancel' }).click()
      await expect(dialog).toBeHidden()
    }
  }
})

test('your plans are dotted in the chart, in their original’s color', async ({ page }) => {
  await addDeal(page)
  await rows(page).filter({ hasText: 'Pepper, your deal' }).click()
  await page.mouse.move(0, 0)
  await expect(page.locator('.chart')).toHaveScreenshot('dotted.png')
})

test('✎ shows on hover, and always on touch screens', async ({ page, isMobile }) => {
  const pencil = page.getByRole('button', { name: "Change a copy of Pepper's fees" })
  if (isMobile) {
    await expect(pencil).toHaveCSS('opacity', '1')
  } else {
    await expect(pencil).toHaveCSS('opacity', '0')
    await page.locator('li', { hasText: 'Pepper' }).hover()
    await expect(pencil).toHaveCSS('opacity', '1')
  }
})
