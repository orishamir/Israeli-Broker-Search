// The Rust core, in Node, for working out what the page should show. Tests
// compare the page with what the core says for the inputs on it, rather
// than with numbers written into the test, so a tariff change doesn't break
// a test of the page: the numbers themselves are checked in Rust. It answers
// in Hebrew, as on the page; tests name plans in English, as links do.

import { readFileSync } from 'node:fs'
import type { Page } from '@playwright/test'
import { shekels, yearlyCost } from '../src/lib/format'
import { parseNumber } from '../src/lib/numbers'
import { t } from '../src/lib/text'
import {
  aboutAll,
  answer,
  brokers,
  columnsFor,
  compare,
  exchanges,
  initSync,
  recommend,
  securities,
  setLang,
  situations,
  todaysRatePercent,
  usualInflationPercent,
} from '../src/lib/core/core'
import type {
  AnswerData,
  AnswersInput,
  Choices,
  Exchange,
  GuideSettings,
  Inputs,
  Overrides,
  PlanKey,
  Security,
} from '../src/lib/core/core'

/** The key of a listed plan: where its broker is, and where it is in the broker's plans. */
export type ListedKey = Extract<PlanKey, { kind: 'listed' }>

initSync({ module: readFileSync(new URL('../src/lib/core/core_bg.wasm', import.meta.url)) })
setLang('He')
export * from '../src/lib/core/core'

/** The exchange rates tests/fixtures.ts answers the app with. */
export const RATES = { date: '2026-09-25', ilsPerEur: 3.4594, ilsPerUsd: 3.0338 }

/** Every plan the list offers, with its broker and its key: as the page,
 * without a kind of fund's plans shown beside its average. */
export const listedPlans = brokers().flatMap((broker, brokerIndex) =>
  broker.plans.flatMap((plan, planIndex) =>
    broker.shownBeside.includes(planIndex)
      ? []
      : [
          {
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
          },
        ],
  ),
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

// ─────────────────────────── The guide ───────────────────────────

/** The page about the numbers. */
export const about = () => aboutAll()

/** The guide's own choice: nothing chosen over it. */
export const NO_OVERRIDES: Overrides = { arena: undefined, kind: undefined }

/** What the guide is asked for at the page's start: the usual rates, the
 * plans ticked at first, the mocked exchange rates, no override. The page's
 * own settings are these until the workbench changes them. */
export const guideSettings = (overrides: Overrides = NO_OVERRIDES): GuideSettings => ({
  indexReturnPercent: 10,
  inflationPercent: usualInflationPercent(),
  boiRatePercent: todaysRatePercent(),
  ilsPerUsd: RATES.ilsPerUsd,
  ilsPerEur: RATES.ilsPerEur,
  plans: usualPlans.map(({ key }) => key),
  yourPlans: [],
  overrides,
  thisYear: new Date().getFullYear(),
  thisMonth: new Date().getMonth() + 1,
})

/** Answers with nothing but what the test says: a horizon not known, a sum
 * not given, no flexibility. */
export const answersWith = (some: Partial<AnswersInput>): AnswersInput => ({
  situation: undefined,
  now: null,
  monthly: null,
  horizon: 'Unknown',
  flexibility: undefined,
  cushion: undefined,
  needsAnyDay: false,
  bank: undefined,
  ...some,
})

/** A situation chip's answers, by its id ("goal", "bonus", ...). */
export function situationAnswers(id: string): AnswersInput {
  const found = situations().find((situation) => situation.id === id)
  if (!found) throw new Error(`no situation "${id}"`)
  return found.answers
}

/** The answer the page should show for `answers` and the visitor's choices
 * on it, under the page's settings at its start: without the horizons, as
 * the card's own job. */
export const answerFor = (
  answers: AnswersInput,
  choices: Choices = {},
  settings: Partial<GuideSettings> = {},
) => answer({ answers, choices, settings: { ...guideSettings(), ...settings }, horizons: false })

/** The decisions an answer asks, in the order of its lines, with the line each sits on. */
export const decisionsOf = (data: AnswerData) =>
  data.lines.flatMap((line) =>
    line.state.type === 'decision' ? [{ line: line.kind, decision: line.state.decision }] : [],
  )

/** The workbench's own guide (the switch on), held to `overrides`. */
export const recommendation = (answers: AnswersInput, overrides: Overrides = NO_OVERRIDES) =>
  recommend(answers, guideSettings(overrides))

// ─────────────────────────── The workbench ───────────────────────────

/** What the workbench's inputs say, read back from its fields and ticks, as
 * the core takes them. Only with the switch on and "another index or a
 * stock" chosen: the levers are on the page while the assumptions are
 * unfolded, and the app's own values otherwise. The horizon is a number
 * and a unit, and not unknown. */
export async function inputsOnPage(page: Page): Promise<Inputs> {
  const chosen = async (group: string) => {
    const checked = page.getByRole('radiogroup', { name: group }).locator('input:checked')
    return (await checked.count()) ? checked.getAttribute('value') : null
  }
  const number = async (id: string, otherwise: number | null = null) => {
    const field = page.locator(`#${id}`)
    return (await field.count()) ? parseNumber(await field.inputValue()) : otherwise
  }
  // The ticks are in the list of plans, closed or folded most of the time.
  const ticked: PlanKey[] = []
  for (const { broker, plan, key } of listedPlans) {
    const box = page
      .getByRole('list', { name: broker.name, includeHidden: true })
      .getByRole('checkbox', { name: plan.name, exact: true, includeHidden: true })
    if (await box.isChecked()) ticked.push(key)
  }
  const exchange = ((await chosen(t.exchange)) ?? 'Usa') as Exchange
  const typed = parseNumber(await page.locator('#horizon').inputValue())
  const unit = await page.getByLabel(t.timeUnit).inputValue()
  const months = unit === 'years' ? (typed ?? 0) * 12 : (typed ?? 0)
  const inflation = await number('inflation', usualInflationPercent())
  // How the money is taken out is asked only while a ticked plan pays a pension.
  const wayOut = page.getByRole('radiogroup', { name: t.takingTheMoneyOut })
  const asPension = (await wayOut.count()) > 0 && (await chosen(t.takingTheMoneyOut)) === 'pension'
  const atEnd = page.getByRole('radiogroup', { name: t.atTheEnd })
  const todays = page.getByRole('checkbox', { name: t.todaysMoney })
  const inTodaysMoney = (await todays.count()) ? await todays.isChecked() : true
  return {
    security: ((await chosen(t.security)) ?? 'Etf') as Security,
    exchange,
    product: undefined,
    firstDeposit: await number('now'),
    monthlyDeposit: await number('monthly'),
    yearlyReturnPercent: await number('yearly-return', 10),
    years: Math.min(50, Math.max(1, Math.ceil(months / 12))),
    buyEveryMonths: (await page.locator('#buy-every').count())
      ? Number(await page.locator('#buy-every').inputValue())
      : 1,
    sharePrice: await number('share-price', 500),
    depositGrowthPercent: await number('deposit-growth', 0),
    inflationPercent: inflation,
    inTodaysMoney: !!inflation && inTodaysMoney,
    asPension,
    age: asPension ? await number('age') : null,
    sellAtEnd: (await atEnd.count()) ? (await chosen(t.atTheEnd)) === 'sell' : true,
    // The rate fields are only on the page abroad; the app has the same rates.
    ilsPerUsd: exchange === 'Tlv' ? RATES.ilsPerUsd : await number('usd', RATES.ilsPerUsd),
    ilsPerEur: exchange === 'Tlv' ? RATES.ilsPerEur : await number('eur', RATES.ilsPerEur),
    plans: ticked,
    yourPlans: [],
  }
}

/** The workbench's table the core would give for `inputs`: each row's plan
 * name and its amounts as the page formats them, best first, in the columns
 * the core names for the index arena. */
export function expectedRows(inputs: Inputs) {
  const columns = columnsFor('Index').map(({ id }) => id)
  return compare(inputs).plans.map(({ key, outcome }) => {
    const plan = listedPlans.find((listed) => JSON.stringify(listed.key) === JSON.stringify(key))!
    const amount = (column: string) => {
      if (!outcome) return undefined
      if (column === 'LeftAfterTax') return shekels(inputs.sellAtEnd ? outcome.afterTax : outcome.held)
      if (column === 'CostAYear') return yearlyCost(outcome.yearlyCostPercent)
      if (column === 'Fees') return shekels(outcome.fees.total)
      return outcome.tax === 0 ? t.noTax : shekels(outcome.tax)
    }
    return { name: plan.plan.name, broker: plan.broker.name, amounts: columns.map(amount).filter(Boolean) }
  })
}

/** The workbench's table as the page shows it, in the same shape. */
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

/** The situations that are chips in the strip: the workbench's and the
 * existing portfolio's open other things, and are tested apart. */
export const chipSituations = () =>
  situations().filter(({ workbench, opensDialog }) => !workbench && !opensDialog)
