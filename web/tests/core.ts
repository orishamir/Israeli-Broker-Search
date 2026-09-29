// The Rust core, in Node, for working out what the page should show. Tests
// compare the page with what the core says for the inputs on it, rather
// than with numbers written into the test, so a tariff change doesn't break
// a test of the page: the numbers themselves are checked in Rust.

import { readFileSync } from 'node:fs'
import type { Page } from '@playwright/test'
import { percent, shekels } from '../src/lib/format'
import { parseNumber } from '../src/lib/numbers'
import { brokers, compare, initSync } from '../src/lib/core/core'
import type { Exchange, Inputs, PlanKey, Security } from '../src/lib/core/core'

/** The key of a listed plan: where its broker is, and where it is in the broker's plans. */
export type ListedKey = Extract<PlanKey, { kind: 'listed' }>

initSync({ module: readFileSync(new URL('../src/lib/core/core_bg.wasm', import.meta.url)) })
export * from '../src/lib/core/core'

/** The exchange rates tests/fixtures.ts answers the app with. */
export const RATES = { date: '2026-09-25', ilsPerEur: 3.4594, ilsPerUsd: 3.0338 }

/** Every listed plan, with its broker and its key. */
export const listedPlans = brokers().flatMap((broker, brokerIndex) =>
  broker.plans.map((plan, planIndex) => ({
    broker,
    plan,
    key: { kind: 'listed', broker: brokerIndex, plan: planIndex } as ListedKey,
    /** As the table and buttons name it: "Leumi · Pepper". */
    label: `${broker.shortName} · ${plan.name}`,
    usual: planIndex === broker.newCustomerPlan,
  })),
)

export const usualPlans = listedPlans.filter(({ usual }) => usual)

/** The plan called `label` ("Leumi · Pepper"). */
export function listed(label: string) {
  const found = listedPlans.find((plan) => plan.label === label)
  if (!found) throw new Error(`no listed plan "${label}"`)
  return found
}

/** What the page's inputs say, read back from its fields and ticks, as the
 * core takes them. */
export async function inputsOnPage(page: Page): Promise<Inputs> {
  const chosen = async (group: string) =>
    page.getByRole('radiogroup', { name: group }).locator('input:checked').getAttribute('value')
  const number = async (id: string) => {
    const field = page.locator(`#${id}`)
    return (await field.count()) ? parseNumber(await field.inputValue()) : null
  }
  const ticked: PlanKey[] = []
  for (const { broker, plan, key } of listedPlans) {
    const box = page
      .getByRole('list', { name: broker.name })
      .getByRole('checkbox', { name: plan.name, exact: true })
    if (await box.isChecked()) ticked.push(key)
  }
  const exchange = (await chosen('Exchange')) as Exchange
  // The expert fields are on the page only under "More options"; off, the
  // app takes the defaults.
  const moreOptions = await page.getByLabel('More options', { exact: true }).isChecked()
  return {
    security: (await chosen('Security')) as Security,
    exchange,
    firstDeposit: await number('first-deposit'),
    monthlyDeposit: await number('monthly-deposit'),
    yearlyReturnPercent: await number('yearly-return'),
    years: Number(await page.locator('#years').inputValue()),
    buyEveryMonths: Number(await page.locator('#buy-every').inputValue()),
    sharePrice: await number('share-price'),
    depositGrowthPercent: moreOptions ? await number('deposit-growth') : 0,
    inflationPercent: moreOptions ? await number('inflation') : 0,
    sellAtEnd: moreOptions ? (await chosen('At the end')) === 'sell' : true,
    // The rate fields are only on the page abroad; the app has the same rates.
    ilsPerUsd: exchange === 'Tlv' ? RATES.ilsPerUsd : await number('usd'),
    ilsPerEur: exchange === 'Tlv' ? RATES.ilsPerEur : await number('eur'),
    plans: ticked,
    yourPlans: [],
  }
}

/** The results table the core would give for `inputs`: each row's plan
 * name and its amounts as the page formats them, best first. Nothing "if
 * sold" when nothing is sold. */
export function expectedRows(inputs: Inputs) {
  return compare(inputs).plans.map(({ key, outcome }) => {
    const plan = listedPlans.find((listed) => JSON.stringify(listed.key) === JSON.stringify(key))!
    const amounts = outcome
      ? [
          shekels(outcome.held),
          ...(inputs.sellAtEnd ? [shekels(outcome.afterSelling)] : []),
          shekels(outcome.fees.total),
          shekels(outcome.lostToFees),
          percent(outcome.yearlyCostPercent),
        ]
      : []
    return { name: plan.plan.name, broker: plan.broker.name, amounts }
  })
}

/** The results table as the page shows it, in the same shape. */
export async function rowsOnPage(page: Page) {
  const rows = await page.locator('tbody tr').all()
  return Promise.all(
    rows.map(async (row) => ({
      name: await row.locator('.names > span').first().textContent(),
      broker: await row.locator('.broker').textContent(),
      amounts: (await row.locator('td.amount').allTextContents())
        .map((text) => text.replace(/ ›$/, ''))
        .filter((text) => !text.startsWith('Not offered')),
    })),
  )
}

/** The row of the plan called `label` ("Meitav · Typical offer"): plan names
 * repeat across brokers, so the row is found by both names. */
export function rowOf(page: Page, label: string) {
  const { plan, broker } = listed(label)
  return page
    .locator('tbody tr')
    .filter({ has: page.getByText(plan.name, { exact: true }) })
    .filter({ has: page.getByText(broker.name, { exact: true }) })
}

/** The row of the plan with `key`. */
export const rowOfKey = (page: Page, key: PlanKey) =>
  rowOf(page, listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.label)

/** What the page buys, as the core takes it for a plan's fees and caveats:
 * with the biggest order of the comparison, if the inputs allow one. */
export async function purchaseOnPage(page: Page) {
  const inputs = await inputsOnPage(page)
  let largestTrade: number | null = null
  try {
    largestTrade = compare(inputs).largestTrade
  } catch {
    // Invalid inputs: no comparison, so the biggest order is unknown.
  }
  const { security, exchange, ilsPerUsd, ilsPerEur } = inputs
  return { inputs, purchase: { security, exchange, largestTrade, ilsPerUsd, ilsPerEur } }
}

/** The track the comparison on the page picked for a plan, if it's compared
 * and its tracks price what's bought. */
export function trackOnPage(inputs: Inputs, key: PlanKey): number | undefined {
  try {
    return compare(inputs).plans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))?.track
  } catch {
    return undefined
  }
}
