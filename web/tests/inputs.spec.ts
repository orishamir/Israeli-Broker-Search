import {
  brokers,
  compare,
  examples,
  exchangeName,
  expectedRows,
  inputsOnPage,
  listed,
  listedPlans,
  rowsOnPage,
  securityName,
  sharePriceSymbol,
  usualPlans,
  type Exchange,
  type Security,
} from './core'
import { t } from '../src/lib/text'
import { choice, expect, openPlans, planInList, plansList, rows, test } from './fixtures'

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
  // The chart's canvas, marked, to tell whether it's made again.
  await page
    .locator('.chart canvas')
    .first()
    .evaluate((canvas) => canvas.setAttribute('data-kept', ''))
  await page.getByLabel(t.everyMonth).fill('-5')
  await expect(page.locator('.error')).toHaveText(t.checkInputs((await reason())!))
  // A field being retyped: the last results stay, faded and out of reach,
  // so that the page keeps its shape.
  await expect(rows(page).first()).toBeVisible()
  await expect(page.locator('.results')).toHaveAttribute('inert')
  await page.getByLabel(t.everyMonth).fill('2000')
  await page.getByLabel(t.yearlyReturn, { exact: true }).fill('')
  await expect(page.locator('.error')).toHaveText(t.checkInputs((await reason())!))
  await page.getByLabel(t.yearlyReturn, { exact: true }).fill('10')
  await expect(page.locator('.error')).toBeHidden()
  await expect(page.locator('.results')).not.toHaveAttribute('inert')
  await expect(rows(page).first()).toBeVisible()
  // Nor were the charts taken off the page, to be drawn again from nothing.
  await expect(page.locator('.chart canvas[data-kept]')).toHaveCount(1)
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
  // And the list of plans, every broker unfolded.
  await openPlans(page)
  for (const { name } of brokers()) await plansList(page).getByRole('button', { name, exact: true }).click()
  await expect(plansList(page)).toMatchAriaSnapshot({ name: 'plans.aria.yml' })
})

test(
  'an example fills the fields as the core says; More options reveals the expert fields, which count',
  { tag: '@phone' },
  async ({ page }) => {
    // Saving for retirement, which also chooses a pension and the age.
    const example = examples().find(({ asPension }) => asPension)!
    await page.getByRole('button', { name: example.name, exact: true }).click()
    expect(await inputsOnPage(page)).toMatchObject({
      security: example.security,
      exchange: example.exchange,
      firstDeposit: example.firstDeposit,
      monthlyDeposit: example.monthlyDeposit,
      yearlyReturnPercent: example.yearlyReturnPercent,
      years: example.years,
      buyEveryMonths: example.buyEveryMonths,
      asPension: true,
      age: example.age,
    })
    const growth = page.getByLabel(t.growingBy, { exact: true })
    await expect(growth).toBeHidden()
    await page.getByLabel(t.moreOptions, { exact: true }).check()
    // In the switch's own card, the last, where they open.
    const card = page.locator('aside section.card').last()
    await expect(card.getByLabel(t.moreOptions, { exact: true })).toBeChecked()
    await expect(card.getByLabel(t.growingBy, { exact: true })).toBeVisible()
    await expect(card.getByLabel(t.inflation, { exact: true })).toBeVisible()
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

// What's compared: a card naming the plans, and the list they're ticked in.

test(
  'the list of plans opens folded; ticking there changes the list, the table and the card',
  { tag: '@phone' },
  async ({ page }) => {
    const [online, pepper] = [listed('Leumi · Online'), listed('Leumi · Pepper')]
    const leumi = online.broker
    await openPlans(page)
    // Every broker is one folded line, saying which of its plans are ticked.
    for (const { name } of brokers()) {
      const folded = plansList(page).getByRole('button', { name, exact: true })
      await expect(folded).toHaveAttribute('aria-expanded', 'false')
    }
    const line = plansList(page).getByRole('button', { name: leumi.name, exact: true })
    await expect(line).toHaveAccessibleDescription(`${online.plan.name} ${t.countOf(1, leumi.plans.length)}`)

    await (await planInList(page, 'Leumi · Pepper')).getByRole('checkbox').check()
    await expect(line).toHaveAttribute('aria-expanded', 'true')
    await expect(line).toHaveAccessibleDescription(
      `${online.plan.name}, ${pepper.plan.name} ${t.countOf(2, leumi.plans.length)}`,
    )
    // The table changes while the list is open.
    await expect(rows(page)).toHaveCount(usualPlans.length + 1)
    await plansList(page)
      .getByRole('button', { name: t.doneComparing(usualPlans.length + 1) })
      .click()
    await expect(plansList(page)).toBeHidden()
    await expect(page.getByRole('button', { name: t.addOrRemove })).toBeFocused()

    // Two of Leumi's plans: the card names each by its label.
    const card = page.getByRole('list', { name: t.whatsCompared }).getByRole('listitem')
    await expect(card).toHaveCount(usualPlans.length + 1)
    await expect(card.filter({ hasText: online.label })).toHaveCount(1)
    await expect(card.filter({ hasText: pepper.label })).toHaveCount(1)
    await expect(card.getByText(leumi.shortName, { exact: true })).toHaveCount(0)
    await expect(page.getByText(t.comparedOf(usualPlans.length + 1, listedPlans.length))).toBeVisible()
    // ✕ closes it too.
    await openPlans(page)
    await plansList(page).getByRole('button', { name: t.close }).click()
    await expect(plansList(page)).toBeHidden()
  },
)

// Beside the results, it leaves them in sight, changing as plans are ticked.
test('on a wide screen the list covers only the inputs, and a click beside it closes it', async ({
  page,
}) => {
  await openPlans(page)
  const list = (await plansList(page).boundingBox())!
  const inputs = (await page.locator('aside').boundingBox())!
  const table = (await page.locator('section.table').boundingBox())!
  // At the start of the line, the right in Hebrew, over the inputs and clear of the table.
  expect(Math.round(list.x + list.width)).toBe(page.viewportSize()!.width)
  expect(list.x).toBeLessThanOrEqual(inputs.x)
  expect(table.x + table.width).toBeLessThanOrEqual(list.x)
  await page.mouse.click(list.x - 40, list.y + list.height / 2)
  await expect(plansList(page)).toBeHidden()
  await expect(page.locator('tbody tr.pinned')).toHaveCount(0)
})

test('on a phone the list covers the whole screen', { tag: '@touch' }, async ({ page }) => {
  await openPlans(page)
  const list = (await plansList(page).boundingBox())!
  const { width, height } = page.viewportSize()!
  expect([list.x, list.y, Math.round(list.width), Math.round(list.height)]).toEqual([0, 0, width, height])
})
