import {
  compare,
  examples,
  exchangeName,
  expectedRows,
  inputsOnPage,
  rowsOnPage,
  securityName,
  sharePriceSymbol,
  type Exchange,
  type Security,
} from './core'
import { t } from '../src/lib/text'
import { choice, expect, rows, test } from './fixtures'

// The inputs panel: fields, choices and what they show or hide.

test('Tel Aviv needs no exchange rates or share price', { tag: '@phone' }, async ({ page }) => {
  await choice(page, t.exchange, exchangeName('Tlv')).click()
  await expect(page.getByText('$1 = ₪')).toBeHidden()
  await expect(page.getByLabel(t.sharePrice)).toBeHidden()
  await expect(rows(page).first()).toBeVisible()
})

test('the share price is asked for only where the core says it matters', async ({ page }) => {
  const sharePrice = page.getByLabel(t.sharePrice, { exact: true })
  const unit = page.locator('.field').filter({ has: sharePrice }).locator('.unit')
  for (const security of ['Etf', 'IndexFund', 'Bond'] as Security[]) {
    for (const exchange of ['Usa', 'Europe', 'Tlv'] as Exchange[]) {
      await choice(page, t.security, securityName(security)).click()
      await choice(page, t.exchange, exchangeName(exchange)).click()
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
  await page.getByLabel(t.everyMonth).fill('-5')
  await expect(page.locator('.error')).toHaveText(t.checkInputs((await reason())!))
  await page.getByLabel(t.everyMonth).fill('2000')
  await page.getByLabel(t.yearlyReturn, { exact: true }).fill('')
  await expect(page.locator('.error')).toHaveText(t.checkInputs((await reason())!))
  await page.getByLabel(t.yearlyReturn, { exact: true }).fill('10')
  await expect(page.locator('.error')).toBeHidden()
  await expect(rows(page).first()).toBeVisible()
})

test('amounts show thousands separators, and the arrow keys step them', async ({ page }) => {
  const first = page.getByLabel(t.oneTimeDeposit)
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

test(
  'the chosen security is explained, with its English name and its other Hebrew names',
  { tag: '@phone' },
  async ({ page }) => {
    // The popovers say it too, hidden.
    const shown = (text: string) => page.locator('.explained').getByText(text, { exact: true })
    await expect(shown('ETF')).toBeVisible()
    await expect(shown('קרן סל מחקה מדד')).toBeVisible()
    await choice(page, t.security, securityName('IndexFund')).click()
    await expect(shown('Index fund')).toBeVisible()
    await expect(shown('קרן נאמנות מחקה')).toBeVisible()
    await expect(page.locator('.explained').getByText('במחיר אחד ביום')).toBeVisible()
  },
)

// Every label, choice and number, as text: a change to any shows as a diff.
test('the inputs, as text', async ({ page }) => {
  await expect(page.locator('aside')).toMatchAriaSnapshot({ name: 'inputs.aria.yml' })
})

test(
  'an example fills the fields as the core says; More options reveals the expert fields, which count',
  { tag: '@phone' },
  async ({ page }) => {
    const example = examples().find(({ security, years }) => security === 'Bond' && years === 5)!
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
    const growth = page.getByLabel(t.growingBy, { exact: true })
    await expect(growth).toBeHidden()
    await page.getByLabel(t.moreOptions, { exact: true }).check()
    await expect(growth).toBeVisible()
    await expect(page.getByLabel(t.inflation, { exact: true })).toBeVisible()
    await growth.fill('3')
    const grown = await inputsOnPage(page)
    expect(grown.depositGrowthPercent).toBe(3)
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(grown))
    // Switched off, the fields go, and so does their effect.
    await page.getByLabel(t.moreOptions, { exact: true }).uncheck()
    await expect(growth).toBeHidden()
    await expect.poll(() => rowsOnPage(page)).toEqual(expectedRows(await inputsOnPage(page)))
  },
)
