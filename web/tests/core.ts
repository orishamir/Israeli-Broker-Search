// The Rust core, in Node, for working out what the page should show. Tests
// compare the page with what the core says for the inputs on it, rather
// than with numbers written into the test, so a tariff change doesn't break
// a test of the page: the numbers themselves are checked in Rust. It answers
// in Hebrew, as on the page; tests name plans in English, as links do.

import { readFileSync } from 'node:fs'
import type { Page } from '@playwright/test'
import { aroundWords } from '../src/lib/around'
import { percent, shekels } from '../src/lib/format'
import { parseNumber } from '../src/lib/numbers'
import { t } from '../src/lib/text'
import {
  around,
  brokers,
  compare,
  compareShortTerm,
  exchanges,
  initSync,
  productsFor,
  securities,
  setLang,
  shortTermKinds,
  sweep,
  usualInflationPercent,
} from '../src/lib/core/core'
import type {
  Exchange,
  Inputs,
  Product,
  PlaceKey,
  PlanKey,
  Security,
  ShortTermInputs,
  Swept,
} from '../src/lib/core/core'

/** The key of a listed plan: where its broker is, and where it is in the broker's plans. */
export type ListedKey = Extract<PlanKey, { kind: 'listed' }>

initSync({ module: readFileSync(new URL('../src/lib/core/core_bg.wasm', import.meta.url)) })
setLang('He')
export * from '../src/lib/core/core'

/** The exchange rates tests/fixtures.ts answers the app with. */
export const RATES = { date: '2026-09-25', ilsPerEur: 3.4594, ilsPerUsd: 3.0338 }

/** Every listed plan, with its broker and its key. */
export const listedPlans = brokers().flatMap((broker, brokerIndex) =>
  broker.plans.map((plan, planIndex) => ({
    broker,
    plan,
    key: { kind: 'listed', broker: brokerIndex, plan: planIndex } as ListedKey,
    /** As the table and buttons name it: "לאומי · פפר". */
    label: `${broker.shortName}\u00a0· ${plan.name}`,
    /** As tests name it, and links: "Leumi · Pepper". */
    englishLabel: `${broker.englishShortName} · ${plan.englishName}`,
    /** Ticked when the app opens: each broker's usual plan, and what
     * savers in a provident fund for investment pay on average. */
    usual: broker.comparedAtFirst && planIndex === broker.newCustomerPlan,
  })),
)

export const usualPlans = listedPlans.filter(({ usual }) => usual)

/** The plan called `englishLabel` ("Leumi · Pepper"). */
export function listed(englishLabel: string) {
  const found = listedPlans.find((plan) => plan.englishLabel === englishLabel)
  if (!found) throw new Error(`no listed plan "${englishLabel}"`)
  return found
}

/** A broker's name on the page, by its English one: "בנק לאומי" for "Bank Leumi". */
export const brokerName = (englishName: string) =>
  brokers().find((broker) => broker.englishName === englishName)!.name

/** A security's name on the page: "קרן סל" for `Etf`. */
export const securityName = (value: Security) => securities().find((choice) => choice.value === value)!.name

/** An exchange's name on the page: "תל אביב" for `Tlv`. */
export const exchangeName = (value: Exchange) => exchanges().find((choice) => choice.value === value)!.name

/** What the page's inputs say, read back from its fields and ticks, as the
 * core takes them. */
export async function inputsOnPage(page: Page): Promise<Inputs> {
  const chosen = async (group: string) =>
    page.getByRole('radiogroup', { name: group }).locator('input:checked').getAttribute('value')
  const number = async (id: string) => {
    const field = page.locator(`#${id}`)
    return (await field.count()) ? parseNumber(await field.inputValue()) : null
  }
  // The ticks are in the list of plans, closed or folded most of the time.
  const ticked: PlanKey[] = []
  for (const { broker, plan, key } of listedPlans) {
    const box = page
      .getByRole('list', { name: broker.name, includeHidden: true })
      .getByRole('checkbox', { name: plan.name, exact: true, includeHidden: true })
    if (await box.isChecked()) ticked.push(key)
  }
  const exchange = (await chosen(t.exchange)) as Exchange
  // The fund is a choice only where there are two; the core takes the first
  // where there's one.
  const products = page.getByRole('radiogroup', { name: t.theFundYouBuy })
  const product = (await products.count()) ? ((await chosen(t.theFundYouBuy)) as Product) : undefined
  // The expert fields are on the page only under "More options"; off, the
  // app takes the defaults.
  const moreOptions = await page.getByLabel(t.moreOptions, { exact: true }).isChecked()
  const inflation = moreOptions ? await number('inflation') : usualInflationPercent()
  // How the money is taken out is asked only while a ticked plan pays a pension.
  const wayOut = page.getByRole('radiogroup', { name: t.takingTheMoneyOut })
  const asPension = (await wayOut.count()) > 0 && (await chosen(t.takingTheMoneyOut)) === 'pension'
  return {
    security: (await chosen(t.security)) as Security,
    exchange,
    product,
    firstDeposit: await number('first-deposit'),
    monthlyDeposit: await number('monthly-deposit'),
    yearlyReturnPercent: await number('yearly-return'),
    years: Number(await page.locator('#years').inputValue()),
    buyEveryMonths: moreOptions ? Number(await page.locator('#buy-every').inputValue()) : 1,
    sharePrice: await number('share-price'),
    depositGrowthPercent: moreOptions ? await number('deposit-growth') : 0,
    inflationPercent: inflation,
    inTodaysMoney:
      moreOptions && !!inflation && (await page.getByRole('checkbox', { name: t.todaysMoney }).isChecked()),
    asPension,
    age: asPension ? await number('age') : null,
    sellAtEnd: moreOptions ? (await chosen(t.atTheEnd)) === 'sell' : true,
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
            outcome.tax === 0 ? t.none : shekels(outcome.tax),
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
        .filter(
          (text) => ![t.notOffered, t.overTheCeiling, t.stillLocked].some((why) => text.startsWith(why)),
        ),
    })),
  )
}

/** The row of the plan called `englishLabel` ("Meitav · Typical offer"):
 * plan names repeat across brokers, so the row is found by both names. */
export function rowOf(page: Page, englishLabel: string) {
  const { plan, broker } = listed(englishLabel)
  return page
    .locator('tbody tr')
    .filter({ has: page.getByText(plan.name, { exact: true }) })
    .filter({ has: page.getByText(broker.name, { exact: true }) })
}

/** The row of the plan with `key`. */
export const rowOfKey = (page: Page, key: PlanKey) =>
  rowOf(page, listedPlans.find((plan) => JSON.stringify(plan.key) === JSON.stringify(key))!.englishLabel)

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
  const { security, exchange, product, ilsPerUsd, ilsPerEur } = inputs
  return { inputs, purchase: { security, exchange, product, largestTrade, ilsPerUsd, ilsPerEur } }
}

/** What the purchase on the page can be held through, as the core lists
 * it: the fund line's names, prices and warning. */
export async function productsOnPage(page: Page) {
  return productsFor((await purchaseOnPage(page)).purchase)
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

// ─────────────────────────── The short term ───────────────────────────

/** The key of a listed place: where its kind is, and where it is in the kind's places. */
export type ListedPlaceKey = Extract<PlaceKey, { kind: 'listed' }>

/** Every listed place, with its kind and its key. */
export const shortTermPlaces = shortTermKinds().flatMap((kind, group) =>
  kind.places.map((place, index) => ({
    kind,
    place,
    key: { kind: 'listed', group, place: index } as ListedPlaceKey,
    /** As tests name it, and links: "Fixed-rate deposit · Bank Leumi". */
    englishLabel: `${kind.englishName} · ${place.englishName}`,
  })),
)

/** The place called `englishLabel` ("Fixed-rate deposit · Bank Leumi"). */
export function place(englishLabel: string) {
  const found = shortTermPlaces.find((listed) => listed.englishLabel === englishLabel)
  if (!found) throw new Error(`no listed place "${englishLabel}"`)
  return found
}

/** What the short-term calculator's inputs say, read back from its fields
 * and from the ticks in its list of places (closed most of the time). */
export async function shortTermInputsOnPage(page: Page): Promise<ShortTermInputs> {
  const number = async (id: string) => {
    const field = page.locator(`#${id}`)
    return (await field.count()) ? parseNumber(await field.inputValue()) : null
  }
  const places: PlaceKey[] = []
  for (const { kind, place, key } of shortTermPlaces) {
    const box = page
      .getByRole('list', { name: kind.name, includeHidden: true })
      .getByRole('checkbox', { name: place.name, exact: true, includeHidden: true })
    if (await box.isChecked()) places.push(key)
  }
  // Your deposits: each rate field's id holds the deposit's.
  const yourDeposits = []
  const rates = page.locator('[id^="deposit-rate-"]')
  for (const field of await rates.all()) {
    const id = (await field.getAttribute('id'))!.replace('deposit-rate-', '')
    yourDeposits.push({ id, ratePercent: parseNumber(await field.inputValue()) })
    const box = page.locator('li.yours', { has: field }).getByRole('checkbox', { includeHidden: true })
    if (await box.isChecked()) places.push({ kind: 'yours', id })
  }
  const moreOptions = await page.getByLabel(t.moreOptions, { exact: true }).isChecked()
  return {
    firstDeposit: await number('first-deposit'),
    monthlyDeposit: await number('monthly-deposit'),
    months: Number(await page.locator('#months').inputValue()),
    ratePercent: await number('rate'),
    inflationPercent: moreOptions ? await number('inflation') : usualInflationPercent(),
    places,
    yourDeposits,
  }
}

/** The short-term table the core would give for `inputs`: each row's name
 * and its amounts as the page formats them, best first; a place the money
 * can't be kept in has the core's few words instead. `yourNames`: your
 * deposits' names, by id. */
export function expectedPlaceRows(inputs: ShortTermInputs, yourNames: Record<string, string> = {}) {
  return compareShortTerm(inputs).places.map(({ key, outcome, notOffered }) => {
    const name =
      key.kind === 'yours'
        ? yourNames[key.id]
        : shortTermPlaces.find((listed) => JSON.stringify(listed.key) === JSON.stringify(key))!.place.name
    const amounts = outcome
      ? [
          shekels(outcome.afterTax),
          percent(outcome.yearlyAfterTaxPercent),
          percent(outcome.yearlyCostPercent),
          outcome.tax === 0 ? t.noTax : shekels(outcome.tax),
        ]
      : [notOffered]
    return { name, amounts }
  })
}

/** The short-term table as the page shows it, in the same shape: a place
 * not offered by the words its cell starts with, before its ?. */
export async function placeRowsOnPage(page: Page) {
  const rows = await page.locator('tbody tr').all()
  return Promise.all(
    rows.map(async (row) => {
      const cells = row.locator('td.amount')
      const offered = (await cells.count()) > 1
      return {
        name: await row.locator('.names > span').first().textContent(),
        amounts: offered
          ? await cells.allTextContents()
          : [(await cells.first().evaluate((cell) => cell.firstChild?.textContent ?? '')).trim()],
      }
    }),
  )
}
