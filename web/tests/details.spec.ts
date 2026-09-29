import {
  about,
  brokerCaveats,
  brokers,
  feesFor,
  listed,
  listedPlans,
  purchaseOnPage,
  purchasePhrase,
  trackOnPage,
} from './core'
import { away, choice, closeDialog, expect, test, type Page } from './fixtures'

// The details dialog: a plan's fees for what's bought, its caveats and its
// prices, its broker, and the page about the numbers. Each is compared with
// what the core describes for the inputs on the page.

/** Opens the details of the plan called `label`, and describes them as the
 * core does for the page's inputs. */
async function open(page: Page, label: string) {
  const { key, plan } = listed(label)
  const { inputs, purchase } = await purchaseOnPage(page)
  const fees = feesFor(key.broker, key.plan, purchase, trackOnPage(inputs, key))
  await page.getByRole('button', { name: `About ${label}` }).click()
  const dialog = page.getByRole('dialog', { name: plan.name })
  await expect(dialog).toBeVisible()
  return { dialog, fees, purchase }
}

test(
  "a plan's ℹ shows its fees and caveats for what's bought, as the core describes them",
  { tag: '@phone' },
  async ({ page }) => {
    const { broker, plan } = listed('Leumi · Pepper')
    const { dialog, fees, purchase } = await open(page, 'Leumi · Pepper')
    await expect(dialog).toContainText(`For ${purchasePhrase(purchase.security, purchase.exchange)}:`)
    await expect(dialog).toContainText(`${broker.tariffDate} · ${broker.checked}`)
    await expect(dialog).toContainText(plan.description)

    // Each fee, its price and its parts, in the core's order and words.
    const names = dialog.locator('.fees-for > dl > dt')
    await expect(names).toHaveCount(fees.fees.length)
    for (const [index, fee] of fees.fees.entries()) {
      await expect(names.nth(index)).toContainText(fee.name)
      await expect(dialog.locator('.fees-for > dl > dd:not(.part)').nth(index)).toContainText(fee.price.text)
      const line = (label: string) => dialog.locator('dd.part', { hasText: label })
      for (const part of fee.parts) await expect(line(part.label)).toContainText(part.price.text)
      for (const part of fee.parts.filter(({ mark }) => mark)) {
        await expect(line(part.label).locator('.mark')).toHaveText(part.mark!.text)
      }
    }
    // The caveats that matter, grouped by how sure they are, most serious first.
    const groups = dialog.locator('.group')
    await expect(groups).toHaveCount(fees.caveats.length)
    for (const [index, group] of fees.caveats.entries()) {
      await expect(groups.nth(index).locator('h4')).toHaveText(group.label)
      await expect(groups.nth(index).getByRole('listitem')).toHaveCount(group.caveats.length)
      for (const caveat of group.caveats) {
        const item = groups.nth(index).getByRole('listitem').filter({ hasText: caveat.text })
        for (const source of caveat.sources) {
          await expect(item.getByRole('link', { name: `${source.name} ↗` })).toHaveAttribute(
            'href',
            source.url,
          )
        }
      }
    }
    // The rest are with all its prices.
    const summary = dialog.getByText('All prices')
    await expect(summary).toContainText(`${fees.others.length} caveat`)
    await summary.click()
    await expect(dialog.locator('.tariff')).toBeVisible()
    await expect(dialog.locator('.tariff table').first().locator('tr')).toHaveCount(
      plan.tariff.trading.length + (plan.tariff.fractionsOn.length > 0 ? 1 : 0),
    )
    await expect(dialog.locator('.tariff li')).toHaveCount(fees.others.length)
    for (const source of plan.sources) {
      await expect(
        dialog.locator('.sources-line').getByRole('link', { name: `${source.name} ↗` }),
      ).toBeVisible()
    }
  },
)

test("a plan's fees follow what's bought: nothing about conversion on Tel Aviv", async ({ page }) => {
  const abroad = await open(page, 'Leumi · Pepper')
  expect(abroad.fees.fees.map(({ kind }) => kind)).toContain('Conversion')
  await closeDialog(page)
  await choice(page, 'Exchange', 'Tel Aviv').click()
  const { dialog, fees } = await open(page, 'Leumi · Pepper')
  expect(fees.fees.map(({ kind }) => kind)).not.toContain('Conversion')
  await expect(dialog.locator('.fees-for > dl > dt')).toHaveCount(fees.fees.length)
  await expect(dialog.locator('.fees-for')).not.toContainText('Conversion')
  // Selling everything at the end is a big order, so a caveat about large
  // orders in Tel Aviv now matters.
  const texts = fees.caveats.flatMap((group) => group.caveats.map(({ text }) => text))
  expect(texts.some((text) => text.includes('₪'))).toBe(true)
  for (const text of texts)
    await expect(dialog.getByRole('listitem').filter({ hasText: text }).first()).toBeVisible()
})

test('a plan with tracks names the one picked and the others', async ({ page }) => {
  const { key, label } = listed('IBI · Full tariff')
  await page.getByRole('checkbox', { name: 'IBI', exact: true }).check()
  const { dialog, fees } = await open(page, label)
  const track = fees.fees[0].parts.find(({ kind }) => kind === 'Track')!
  await expect(dialog.locator('dd.part', { hasText: track.label })).toContainText(track.price.text)
  expect(track.price.text).toContain('(others: ')
  expect(key.kind).toBe('listed')
})

test(
  "a plan's broker is a link to its details: its plans and its own caveats",
  { tag: '@phone' },
  async ({ page }) => {
    const { broker, plan, key } = listed('Leumi · Pepper')
    const { dialog, purchase } = await open(page, 'Leumi · Pepper')
    await dialog.getByRole('button', { name: broker.name }).click()
    const brokerDialog = page.getByRole('dialog', { name: broker.name })
    await expect(brokerDialog).toContainText(broker.description)
    await expect(brokerDialog.locator('.plans .plan-name')).toHaveText(broker.plans.map(({ name }) => name))
    const groups = brokerCaveats(key.broker, purchase)
    await expect(brokerDialog.locator('.group')).toHaveCount(groups.length)
    // Back to the plan from the broker.
    await brokerDialog
      .locator('.plans li', { has: page.getByText(plan.name, { exact: true }) })
      .getByRole('button')
      .click()
    await expect(page.getByRole('dialog', { name: plan.name })).toBeVisible()
    await closeDialog(page)
  },
)

// Opening each plan's details, with all its prices, breaks nothing: the
// fixture fails the test on any error thrown in the page.
test("every plan's details open", async ({ page }) => {
  // A loop over everything: near the timeout under load.
  test.slow()
  const buttons = page.getByRole('button', { name: /^About .+ · / })
  await expect(buttons).toHaveCount(listedPlans.length)
  for (const button of await buttons.all()) {
    await button.click()
    await away(page)
    const dialog = page.getByRole('dialog')
    await dialog.getByText('All prices').click()
    await expect(dialog.locator('.tariff')).toBeVisible()
    await closeDialog(page)
  }
})

test('the numbers are explained, from the title and from a plan', { tag: '@phone' }, async ({ page }) => {
  const { sections } = about()
  await page.getByRole('button', { name: sections[2].title }).click()
  const dialog = page.getByRole('dialog', { name: 'About the numbers' })
  await expect(dialog).toBeVisible()
  for (const section of sections)
    await expect(dialog.getByRole('heading', { name: section.title })).toBeVisible()
  // Every page the numbers rest on, by broker, the tariff first.
  const sources = sections.find(({ sources }) => sources.length > 0)!
  for (const group of sources.sources) {
    const item = dialog.locator('.source-groups > li', { hasText: group.title })
    for (const source of group.sources)
      await expect(item.getByRole('link', { name: `${source.name} ↗` })).toBeVisible()
  }
  await expect(dialog.getByRole('link', { name: 'Tariff (PDF) ↗' })).toHaveCount(
    brokers().filter(({ sourceUrl }) => sourceUrl).length,
  )
  await closeDialog(page)

  await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
  await page
    .getByRole('dialog', { name: 'Pepper' })
    .getByRole('button', { name: /How the numbers are made/ })
    .click()
  await expect(dialog).toBeVisible()
  await expect(dialog).toContainText(sections[1].title)
})
