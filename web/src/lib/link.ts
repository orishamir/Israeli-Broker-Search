// A page as a link: the answers, the switch and, when they matter, the
// workbench's inputs and the ticked plans, in the URL's hash ("#v=2&..."), so
// what one person sees can be sent to another. Your own ticked plans travel
// with it, as data, and become the recipient's. Nothing is sent anywhere:
// the link is the state. A link without `v=2` (the first page's, or an
// edited one) is not read: the page opens as usual.

import type { YourPlan } from './app.svelte'
import * as core from './core/core'
import type { Arena, Bank, CushionInput, Exchange, Flexibility, Horizon, Security } from './core/core'
import { firstOfEachId } from './saved'
import { kindByKey, wayByKey } from './kinds'

export const LINK_VERSION = '2'

/** What the workbench is looking at. */
export type Thing = 'sp500' | 'other' | 'bonds' | 'short'
const THINGS: Thing[] = ['sp500', 'other', 'bonds', 'short']

/** What a link carries. Every field is optional: a link edited by hand may
 * lack some, and the rest still apply. */
export interface Shared {
  /** The situation chip's id, and the bank the existing-portfolio one asked. */
  situation?: string
  bank?: Bank
  now?: number
  monthly?: number
  horizon?: Horizon
  flexibility?: Flexibility
  cushion?: CushionInput
  needsAnyDay?: boolean
  /** The switch. The inputs below are in a link only when it is on, or an
   * override is. */
  expert?: boolean
  thing?: Thing
  /** The ticked ways, as `wayKey`s. */
  ways?: string[]
  security?: Security
  exchange?: Exchange
  yearlyReturnPercent?: number
  inflationPercent?: number
  boiRatePercent?: number
  depositGrowthPercent?: number
  buyEveryMonths?: number
  sharePrice?: number
  /** False when amounts are shown as they will be rather than in today's money. */
  todaysMoney?: boolean
  sellAtEnd?: boolean
  asPension?: boolean
  age?: number
  /** What the user chose over the guide: an arena, and a kind as `kindKey`. */
  overrideArena?: Arena
  overrideKind?: string
  /** The ticked listed plans, by label: "Leumi · Pepper". Names, not
   * positions, which shift when brokers are added. */
  plans?: string[]
  /** Your own ticked plans, whole. */
  yours?: YourPlan[]
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
const NUMBERS = {
  now: 'n',
  monthly: 'm',
  yearlyReturnPercent: 'r',
  inflationPercent: 'f',
  boiRatePercent: 'br',
  depositGrowthPercent: 'g',
  buyEveryMonths: 'b',
  sharePrice: 'p',
  age: 'a',
} as const
const VERSION = 'v'
const HORIZON = 'h'
const SITUATION = 'si'
const BANK = 'bk'
const FLEXIBILITY = 'fx'
const CUSHION = 'cu'
const ANY_DAY = 'ad'
const EXPERT = 'x'
const THING = 'th'
const WAY = 'way'
const SECURITY = 's'
const EXCHANGE = 'e'
const NOT_TODAYS_MONEY = 'nom'
const HOLD = 'keep'
const PENSION = 'k'
const OVERRIDE_ARENA = 'oa'
const OVERRIDE_KIND = 'ok'
const PLAN = 'plan'
const YOURS = 'yours'

/** The hash (without its #) that `decode` reads back as `shared`. */
export function encode(shared: Shared): string {
  const params = new URLSearchParams({ [VERSION]: LINK_VERSION })
  const set = (key: string, value: string | number | undefined) => {
    if (value !== undefined) params.set(key, String(value))
  }
  set(SITUATION, shared.situation)
  set(BANK, shared.bank)
  for (const [field, key] of Object.entries(NUMBERS)) {
    set(key, shared[field as keyof typeof NUMBERS])
  }
  const { horizon } = shared
  if (horizon === 'Unknown') set(HORIZON, 'u')
  else if (horizon && 'Months' in horizon) set(HORIZON, horizon.Months)
  else if (horizon) set(HORIZON, `r:${horizon.Retirement.age}:${horizon.Retirement.retire_at}`)
  if (shared.flexibility) set(FLEXIBILITY, shared.flexibility === 'Fixed' ? 'fixed' : 'wait')
  const { cushion } = shared
  if (cushion?.kind === 'has') set(CUSHION, 'has')
  else if (cushion) {
    set(CUSHION, cushion.monthlyExpenses === null ? 'n' : `n:${cushion.monthlyExpenses}`)
  }
  if (shared.needsAnyDay) set(ANY_DAY, 1)
  if (shared.expert) set(EXPERT, 1)
  set(THING, shared.thing)
  for (const way of shared.ways ?? []) params.append(WAY, way)
  set(SECURITY, shared.security)
  set(EXCHANGE, shared.exchange)
  if (shared.todaysMoney === false) set(NOT_TODAYS_MONEY, 1)
  if (shared.sellAtEnd === false) set(HOLD, 1)
  if (shared.asPension) set(PENSION, 1)
  set(OVERRIDE_ARENA, shared.overrideArena)
  set(OVERRIDE_KIND, shared.overrideKind)
  for (const plan of shared.plans ?? []) params.append(PLAN, plan)
  if (shared.yours?.length) params.set(YOURS, toBase64Url(JSON.stringify(shared.yours)))
  return params.toString()
}

/** A link's number, if it's one and `check` accepts it. */
const numberIn =
  (params: URLSearchParams) =>
  (key: string, check: (value: number) => boolean = () => true): number | undefined => {
    const text = params.get(key)
    if (text === null || text.trim() === '') return undefined
    const value = Number(text)
    return Number.isFinite(value) && check(value) ? value : undefined
  }

const defined = <T extends object>(shared: T): T =>
  Object.fromEntries(Object.entries(shared).filter(([, value]) => value !== undefined)) as T

const nonNegative = (value: number) => value >= 0

/** What the page's address says: the hash, or the query string where the link
 * was typed or built with `?v=2&...`. */
export function decodeLocation(where: { hash: string; search: string }): Shared {
  const fromHash = decode(where.hash)
  return Object.keys(fromHash).length > 0 ? fromHash : decode(where.search)
}

/** What a link's hash says, keeping only what makes sense: a number that
 * isn't one, or a security the core doesn't know, is left out. The core
 * checks the plans' data when they're loaded. Nothing is read without `v=2`. */
export function decode(hash: string): Shared {
  const params = new URLSearchParams(hash.replace(/^[#?]/, ''))
  if (params.get(VERSION) !== LINK_VERSION) return {}
  const number = numberIn(params)
  const among = <T extends string>(key: string, values: readonly T[]) => {
    const text = params.get(key)
    return values.find((value) => value === text)
  }
  const shared: Shared = {}
  const situationId = params.get(SITUATION)
  if (situationId !== null && core.situations().some(({ id }) => id === situationId)) {
    shared.situation = situationId
  }
  shared.bank = among(
    BANK,
    core.banks().map(({ bank }) => bank),
  )
  shared.now = number(NUMBERS.now, nonNegative)
  shared.monthly = number(NUMBERS.monthly, nonNegative)
  shared.horizon = horizonIn(params.get(HORIZON))
  const flexibility = among(FLEXIBILITY, ['fixed', 'wait'])
  if (flexibility) shared.flexibility = flexibility === 'fixed' ? 'Fixed' : 'CanWait'
  shared.cushion = cushionIn(params.get(CUSHION))
  if (params.get(ANY_DAY) === '1') shared.needsAnyDay = true
  if (params.get(EXPERT) === '1') shared.expert = true
  shared.thing = among(THING, THINGS)
  const ways = params.getAll(WAY).filter((key) => wayByKey(key))
  if (ways.length > 0) shared.ways = ways
  shared.security = among(
    SECURITY,
    core.securities().map(({ value }) => value),
  )
  shared.exchange = among(
    EXCHANGE,
    core.exchanges().map(({ value }) => value),
  )
  shared.yearlyReturnPercent = number(NUMBERS.yearlyReturnPercent)
  shared.inflationPercent = number(NUMBERS.inflationPercent)
  shared.boiRatePercent = number(NUMBERS.boiRatePercent)
  shared.depositGrowthPercent = number(NUMBERS.depositGrowthPercent)
  shared.buyEveryMonths = number(NUMBERS.buyEveryMonths, (value) =>
    BUYING_INTERVALS.some(({ months }) => months === value),
  )
  shared.sharePrice = number(NUMBERS.sharePrice, (value) => value > 0)
  if (params.get(NOT_TODAYS_MONEY) === '1') shared.todaysMoney = false
  if (params.get(HOLD) === '1') shared.sellAtEnd = false
  if (params.get(PENSION) === '1') shared.asPension = true
  shared.age = number(NUMBERS.age, (value) => value >= 0 && value <= 120)
  shared.overrideArena = among(OVERRIDE_ARENA, ['Safe', 'Index'])
  const kind = params.get(OVERRIDE_KIND)
  if (kind !== null && kindByKey(kind)) shared.overrideKind = kind
  const plans = params.getAll(PLAN)
  if (plans.length > 0) shared.plans = plans
  shared.yours = listIn(params, YOURS, looksLikeYourPlan)
  return defined(shared)
}

/** `h=60` months, `h=u` not known, `h=r:45:65` retiring at 65 at 45. */
function horizonIn(text: string | null): Horizon | undefined {
  if (text === null) return undefined
  if (text === 'u') return 'Unknown'
  const retirement = /^r:(\d{1,3}):(\d{1,3})$/.exec(text)
  if (retirement) return { Retirement: { age: Number(retirement[1]), retire_at: Number(retirement[2]) } }
  const months = Number(text)
  return Number.isInteger(months) && months >= 1 && months <= 1200 ? { Months: months } : undefined
}

/** `cu=has`, `cu=n` (none, expenses unsaid), `cu=n:5000`. */
function cushionIn(text: string | null): CushionInput | undefined {
  if (text === 'has') return { kind: 'has' }
  if (text === 'n') return { kind: 'hasNot', monthlyExpenses: null }
  const expenses = /^n:(\d+(?:\.\d+)?)$/.exec(text ?? '')
  return expenses ? { kind: 'hasNot', monthlyExpenses: Number(expenses[1]) } : undefined
}

/** A link's list, from base64 JSON, keeping only the items `looksRight`
 * accepts, and the first of each id. */
function listIn<T extends { id: string }>(
  params: URLSearchParams,
  key: string,
  looksRight: (value: unknown) => value is T,
): T[] | undefined {
  const text = params.get(key)
  if (text === null) return undefined
  try {
    const parsed: unknown = JSON.parse(fromBase64Url(text))
    return Array.isArray(parsed) ? firstOfEachId(parsed.filter(looksRight)) : undefined
  } catch {
    // Not a list: left out.
    return undefined
  }
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
