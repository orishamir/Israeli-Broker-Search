import { compare, examples, expectedRows, inputsOnPage, rowsOnPage, sharePriceSymbol } from './core'
import { choice, expect, rows, test } from './fixtures'

// The inputs panel: fields, choices and what they show or hide.

test('Tel Aviv needs no exchange rates or share price', { tag: '@phone' }, async ({ page }) => {
  await choice(page, 'Exchange', 'Tel Aviv').click()
  await expect(page.getByText('$1 = ₪')).toBeHidden()
  await expect(page.getByLabel('Share price')).toBeHidden()
  await expect(rows(page).first()).toBeVisible()
})

test('the share price is asked for only where the core says it matters', async ({ page }) => {
  const sharePrice = page.getByLabel('Share price', { exact: true })
  const unit = page.locator('.field').filter({ has: sharePrice }).locator('.unit')
  for (const security of ['ETF', 'Index fund', 'Bond']) {
    for (const exchange of ['USA', 'Europe', 'Tel Aviv']) {
      await choice(page, 'Security', security).click()
      await choice(page, 'Exchange', exchange).click()
      const symbol = sharePriceSymbol(await inputsOnPage(page)) ?? null
      const shown = async () => ({
        visible: await sharePrice.isVisible(),
        unit: (await unit.allTextContents())[0] ?? null,
      })
      await expect
        .poll(shown, { message: `${security} on ${exchange}` })
        .toEqual({ visible: symbol !== null, unit: symbol })
    }
  }
})

test('bad inputs show why, in the words of the core, instead of breaking', async ({ page }) => {
  const reason = async () => {
    try {
      compare(await inputsOnPage(page))
      return null
    } catch (error) {
      return (error as Error).message
    }
  }
  await page.getByLabel('Every month').fill('-5')
  await expect(page.locator('.error')).toHaveText(`Check your inputs: ${await reason()}.`)
  await page.getByLabel('Every month').fill('2000')
  await page.getByLabel('Yearly return', { exact: true }).fill('')
  await expect(page.locator('.error')).toHaveText(`Check your inputs: ${await reason()}.`)
  await page.getByLabel('Yearly return', { exact: true }).fill('10')
  await expect(page.locator('.error')).toBeHidden()
  await expect(rows(page).first()).toBeVisible()
})

test('amounts show thousands separators, and the arrow keys step them', async ({ page }) => {
  const first = page.getByLabel('One-time deposit')
  await expect(first).toHaveValue('10,000')
  await first.focus()
  await page.keyboard.press('ArrowUp')
  await expect(first).toHaveValue('11,000')
  await first.fill('25000')
  await first.blur()
  await expect(first).toHaveValue('25,000')
  await expect(page.getByText('₪505,000')).toBeVisible() // deposited: 25,000 + 2,000 × 240
})

test(
  'exchange rates are folded away until opened, and can be changed',
  { tag: '@phone' },
  async ({ page }) => {
    const summary = page.getByText('$1 = ₪3.0338 · €1 = ₪3.4594')
    await expect(summary).toBeVisible()
    await expect(page.getByLabel('$1 =')).toBeHidden()
    await summary.click()
    await page.getByLabel('$1 =').fill('4')
    await expect(page.getByText('$1 = ₪4 · €1 = ₪3.4594')).toBeVisible()
  },
)

test('the chosen security is explained, with its Hebrew names', { tag: '@phone' }, async ({ page }) => {
  // The popovers say it too, hidden.
  const shown = (text: string) => page.locator('.explained').getByText(text)
  await expect(shown('קרן סל מחקה מדד')).toBeVisible()
  await choice(page, 'Security', 'Index fund').click()
  await expect(shown('קרן מחקה')).toBeVisible()
  await expect(shown('at one price a day')).toBeVisible()
})

// Every label, choice and number, as text: a change to any shows as a diff.
test('the inputs, as text', async ({ page }) => {
  await expect(page.locator('aside')).toMatchAriaSnapshot({ name: 'inputs.aria.yml' })
})

test(
  'an example fills the fields as the core says; More options reveals the expert fields, which count',
  { tag: '@phone' },
  async ({ page }) => {
    const example = examples().find(({ name }) => name === 'Bonds, 5 years')!
    await page.getByRole('button', { name: example.name, exact: true }).click()
    expect(await inputsOnPage(page)).toMatchObject({
      security: example.security,
      exchange: example.exchange,
      firstDeposit: example.firstDeposit,
      monthlyDeposit: example.monthlyDeposit,
      yearlyReturnPercent: example.yearlyReturnPercent,
      years: example.years,
      buyEveryMonths: example.buyEveryMonths,
    })
    const growth = page.getByLabel('Growing by', { exact: true })
    await expect(growth).toBeHidden()
    await page.getByLabel('More options', { exact: true }).check()
    await expect(growth).toBeVisible()
    await expect(page.getByLabel('Inflation', { exact: true })).toBeVisible()
    await growth.fill('3')
    const grown = await inputsOnPage(page)
    expect(grown.depositGrowthPercent).toBe(3)
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(grown))
    // Switched off, the fields go, and so does their effect.
    await page.getByLabel('More options', { exact: true }).uncheck()
    await expect(growth).toBeHidden()
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  },
)
