// The Rust core, in Node, for working out what the page should show. Tests
// compare the page with what the core says for the inputs on it, rather
// than with numbers written into the test, so a tariff change doesn't break
// a test of the page: the numbers themselves are checked in Rust.

import { readFileSync } from 'node:fs'
import type { Page } from '@playwright/test'
import { aroundWords } from '../src/lib/around'
import { percent, shekels } from '../src/lib/format'
import { parseNumber } from '../src/lib/numbers'
import { around, brokers, compare, initSync, sweep, usualInflationPercent } from '../src/lib/core/core'
import type { Exchange, Inputs, PlanKey, Security, Swept } from '../src/lib/core/core'

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
    /** Ticked when the app opens: each broker's usual plan, and what
     * savers in a provident fund for investment pay on average. */
    usual: broker.comparedAtFirst && planIndex === broker.newCustomerPlan,
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
  const inflation = moreOptions ? await number('inflation') : usualInflationPercent()
  // How the money is taken out is asked only while a ticked plan pays a pension.
  const wayOut = page.getByRole('radiogroup', { name: 'Taking the money out' })
  const asPension = (await wayOut.count()) > 0 && (await chosen('Taking the money out')) === 'pension'
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
    inflationPercent: inflation,
    inTodaysMoney:
      moreOptions &&
      !!inflation &&
      (await page.getByRole('checkbox', { name: "Show amounts in today's money" }).isChecked()),
    asPension,
    age: asPension ? await number('age') : null,
    sellAtEnd: moreOptions ? (await chosen('At the end')) === 'sell' : true,
    // The rate fields are only on the page abroad; the app has the same rates.
    ilsPerUsd: exchange === 'Tlv' ? RATES.ilsPerUsd : await number('usd'),
    ilsPerEur: exchange === 'Tlv' ? RATES.ilsPerEur : await number('eur'),
    plans: ticked,
    yourPlans: [],
  }
}

/** The results table the core would give for `inputs`: each row's plan
 * name and its amounts as the page formats them, best first, in the table's
 * column order: what's left after tax first when everything is sold, and
 * what's held, with no tax yet, when nothing is. */
export function expectedRows(inputs: Inputs) {
  return compare(inputs).plans.map(({ key, outcome }) => {
    const plan = listedPlans.find((listed) => JSON.stringify(listed.key) === JSON.stringify(key))!
    const [lost, yearly, fees] = outcome
      ? [shekels(outcome.lostToFees), percent(outcome.yearlyCostPercent), shekels(outcome.fees.total)]
      : []
    const amounts = !outcome
      ? []
      : inputs.sellAtEnd
        ? [
            shekels(outcome.afterTax),
            lost,
            outcome.tax === 0 ? 'none' : shekels(outcome.tax),
            yearly,
            fees,
            shekels(outcome.held),
          ]
        : [shekels(outcome.held), lost, yearly, fees]
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
        .filter((text) => !/^(Not offered|Your deposits are over|Its money is still locked)/.test(text)),
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

/** The best plan's line about other deposits for `inputs`: the core's
 * answer, in the page's words. The sweep varies the monthly deposit if
 * there is one, else the one-time deposit, and is asked without it. */
export function expectedAroundLine(inputs: Inputs): string | undefined {
  const swept: Swept = (inputs.monthlyDeposit ?? 0) > 0 ? 'Monthly' : 'OneTime'
  const without = swept === 'Monthly' ? { ...inputs, monthlyDeposit: 0 } : { ...inputs, firstDeposit: 0 }
  const found = around({
    sweep: sweep(without, swept),
    deposit: (swept === 'Monthly' ? inputs.monthlyDeposit : inputs.firstDeposit) ?? 0,
    costs: compare(inputs).plans.map(({ key, outcome }) => ({
      key,
      cost: outcome?.yearlyCostPercent ?? null,
    })),
  })
  const labelOf = (key: PlanKey) =>
    listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.label
  return found && aroundWords(found, swept, labelOf)
}
