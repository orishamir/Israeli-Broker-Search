// A comparison as a link: the inputs and the ticked plans in the URL's hash,
// so that a comparison can be sent to someone, who then sees the same
// table. Your own ticked plans travel with it, as data, and become the
// recipient's. Nothing is sent anywhere: the link is the state.

import type { Family, YourPlan } from './app.svelte'
import * as core from './core/core'
import type { Exchange, Product, Security } from './core/core'
import { looksLikeYourDeposit, type YourDeposit } from './short-term.svelte'

/** What a link carries. Every field is optional: a link from another
 * version of the app, or one edited by hand, may lack some, and the rest
 * still apply. */
export interface Shared {
  /** Which calculator: the long term's, unless it says otherwise. */
  family?: Family
  /** The short-term calculator's inputs, when it's the one. */
  short?: SharedShort
  security?: Security
  exchange?: Exchange
  /** The fund chosen, where the purchase has a choice (an ETF in Tel Aviv). */
  product?: Product
  firstDeposit?: number
  monthlyDeposit?: number
  yearlyReturnPercent?: number
  years?: number
  buyEveryMonths?: number
  sharePrice?: number
  depositGrowthPercent?: number
  inflationPercent?: number
  /** Whether amounts are shown in today's money, at that inflation. */
  inTodaysMoney?: boolean
  sellAtEnd?: boolean
  /** Whether the money is taken as a pension, and the saver's age today. */
  asPension?: boolean
  age?: number
  /** The ticked listed plans, by label: "Leumi · Pepper". Names, not
   * positions, which shift when brokers are added. */
  plans?: string[]
  /** Your own ticked plans, whole. */
  yours?: YourPlan[]
}

/** What a link to the short-term calculator carries. */
export interface SharedShort {
  firstDeposit?: number
  monthlyDeposit?: number
  months?: number
  ratePercent?: number
  inflationPercent?: number
  /** The ticked listed places, by label: "Fixed-rate deposit · Bank Leumi". */
  places?: string[]
  /** Your own ticked deposits, whole. */
  yours?: YourDeposit[]
}

/** How often one can buy, in months: the "Buy every" dropdown. */
export const BUYING_INTERVALS = [
  { months: 1, name: 'month' },
  { months: 2, name: '2 months' },
  { months: 3, name: '3 months' },
  { months: 6, name: '6 months' },
  { months: 12, name: 'year' },
]

// Short keys: a link is read by people too, in a chat.
const KEYS = {
  security: 's',
  exchange: 'x',
  firstDeposit: 'd',
  monthlyDeposit: 'm',
  yearlyReturnPercent: 'r',
  years: 'y',
  buyEveryMonths: 'b',
  sharePrice: 'p',
  depositGrowthPercent: 'g',
  age: 'a',
} as const
// Inflation with the amounts in today's money, as every link with an
// inflation meant before the two were told apart; and inflation alone.
const TODAYS_MONEY_AT = 'i'
const INFLATION = 'f'
const HOLD = 'h'
const PENSION = 'k'
const FUND = 'fund'
const PLAN = 'plan'
const YOURS = 'yours'
// The short-term calculator's link says so, and its months; the amounts and
// the rate reuse the long term's keys, as a link holds one calculator.
const CALCULATOR = 'c'
const SHORT = 's'
const MONTHS = 'n'
const PLACE = 'place'
const DEPOSITS = 'deposits'

/** The hash (without its #) that `decode` reads back as `shared`. */
export function encode(shared: Shared): string {
  if (shared.family === 'short') return encodeShort(shared.short ?? {})
  const params = new URLSearchParams()
  for (const [field, key] of Object.entries(KEYS)) {
    const value = shared[field as keyof typeof KEYS]
    if (value !== undefined) params.set(key, String(value))
  }
  if (shared.inflationPercent !== undefined) {
    params.set(shared.inTodaysMoney ? TODAYS_MONEY_AT : INFLATION, String(shared.inflationPercent))
  }
  if (shared.sellAtEnd === false) params.set(HOLD, '1')
  if (shared.asPension) params.set(PENSION, '1')
  if (shared.product) params.set(FUND, shared.product)
  for (const plan of shared.plans ?? []) params.append(PLAN, plan)
  if (shared.yours?.length) params.set(YOURS, toBase64Url(JSON.stringify(shared.yours)))
  return params.toString()
}

function encodeShort(shared: SharedShort): string {
  const params = new URLSearchParams({ [CALCULATOR]: SHORT })
  const keys = {
    firstDeposit: KEYS.firstDeposit,
    monthlyDeposit: KEYS.monthlyDeposit,
    months: MONTHS,
    ratePercent: KEYS.yearlyReturnPercent,
    inflationPercent: INFLATION,
  } as const
  for (const [field, key] of Object.entries(keys)) {
    const value = shared[field as keyof typeof keys]
    if (value !== undefined) params.set(key, String(value))
  }
  for (const place of shared.places ?? []) params.append(PLACE, place)
  if (shared.yours?.length) params.set(DEPOSITS, toBase64Url(JSON.stringify(shared.yours)))
  return params.toString()
}

/** A link's number, if it's one and `check` accepts it. */
const numberIn =
  (params: URLSearchParams) =>
  (key: string, check: (value: number) => boolean = () => true): number | undefined => {
    const text = params.get(key)
    if (text === null) return undefined
    const value = Number(text)
    return Number.isFinite(value) && check(value) ? value : undefined
  }

/** A link's list, from base64 JSON, keeping only the items `looksRight` accepts. */
function listIn<T>(
  params: URLSearchParams,
  key: string,
  looksRight: (value: unknown) => value is T,
): T[] | undefined {
  const text = params.get(key)
  if (text === null) return undefined
  try {
    const parsed: unknown = JSON.parse(fromBase64Url(text))
    return Array.isArray(parsed) ? parsed.filter(looksRight) : undefined
  } catch {
    // Not a list: left out.
    return undefined
  }
}

/** Only what was there. */
const defined = <T extends object>(shared: T): T =>
  Object.fromEntries(Object.entries(shared).filter(([, value]) => value !== undefined)) as T

/** What a link's hash says, keeping only what makes sense: a number that
 * isn't one, or a security the core doesn't know, is left out. The core
 * checks the plans' data when they're loaded. */
export function decode(hash: string): Shared {
  const params = new URLSearchParams(hash.replace(/^#/, ''))
  if (params.get(CALCULATOR) === SHORT) return { family: 'short', short: decodeShort(params) }
  const shared: Shared = {}
  const number = numberIn(params)
  const among = <T extends string>(key: string, values: T[]) => {
    const text = params.get(key)
    return values.find((value) => value === text)
  }
  shared.security = among(
    KEYS.security,
    core.securities().map(({ value }) => value),
  )
  shared.exchange = among(
    KEYS.exchange,
    core.exchanges().map(({ value }) => value),
  )
  // A fund the core knows; one the purchase doesn't offer is passed over.
  shared.product = among(
    FUND,
    core
      .securities()
      .flatMap(({ value: security }) =>
        core
          .exchanges()
          .flatMap(({ value: exchange }) =>
            core.productsFor({ security, exchange, largestTrade: null, ilsPerUsd: null, ilsPerEur: null }),
          ),
      )
      .map(({ product }) => product),
  )
  shared.firstDeposit = number(KEYS.firstDeposit, (value) => value >= 0)
  shared.monthlyDeposit = number(KEYS.monthlyDeposit, (value) => value >= 0)
  shared.yearlyReturnPercent = number(KEYS.yearlyReturnPercent)
  shared.years = number(KEYS.years, (value) => Number.isInteger(value) && value >= 1 && value <= 50)
  shared.buyEveryMonths = number(KEYS.buyEveryMonths, (value) =>
    BUYING_INTERVALS.some(({ months }) => months === value),
  )
  shared.sharePrice = number(KEYS.sharePrice, (value) => value > 0)
  shared.depositGrowthPercent = number(KEYS.depositGrowthPercent)
  shared.inflationPercent = number(TODAYS_MONEY_AT) ?? number(INFLATION)
  if (number(TODAYS_MONEY_AT) !== undefined) shared.inTodaysMoney = true
  if (params.get(HOLD) === '1') shared.sellAtEnd = false
  if (params.get(PENSION) === '1') shared.asPension = true
  shared.age = number(KEYS.age, (value) => value >= 0 && value <= 120)
  const plans = params.getAll(PLAN)
  if (plans.length > 0) shared.plans = plans
  shared.yours = listIn(params, YOURS, looksLikeYourPlan)
  return defined(shared)
}

function decodeShort(params: URLSearchParams): SharedShort {
  const number = numberIn(params)
  const places = params.getAll(PLACE)
  return defined({
    firstDeposit: number(KEYS.firstDeposit, (value) => value >= 0),
    monthlyDeposit: number(KEYS.monthlyDeposit, (value) => value >= 0),
    months: number(
      MONTHS,
      (value) => Number.isInteger(value) && value >= 1 && value <= core.shortTermLongest(),
    ),
    ratePercent: number(KEYS.yearlyReturnPercent),
    inflationPercent: number(INFLATION),
    places: places.length > 0 ? places : undefined,
    yours: listIn(params, DEPOSITS, looksLikeYourDeposit),
  })
}

/** The shape of one of your plans, as far as the web side knows it; the
 * core checks the plan's data itself. */
function looksLikeYourPlan(value: unknown): value is YourPlan {
  if (typeof value !== 'object' || value === null) return false
  const { id, plan, brokerName, basedOn } = value as Record<string, unknown>
  return (
    typeof id === 'string' &&
    typeof plan === 'object' &&
    plan !== null &&
    (brokerName === null || typeof brokerName === 'string') &&
    (basedOn === null || typeof basedOn === 'object')
  )
}

// Base64 for URLs (no + / or =), of the text's UTF-8 bytes: btoa takes
// only single-byte characters.
function toBase64Url(text: string): string {
  const bytes = new TextEncoder().encode(text)
  const binary = Array.from(bytes, (byte) => String.fromCharCode(byte)).join('')
  return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')
}

function fromBase64Url(text: string): string {
  const binary = atob(text.replaceAll('-', '+').replaceAll('_', '/'))
  return new TextDecoder().decode(Uint8Array.from(binary, (char) => char.charCodeAt(0)))
}
