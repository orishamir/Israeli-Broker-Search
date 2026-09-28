import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  BrokerInfo,
  CaveatGroup,
  Choice,
  ComparisonData,
  Exchange,
  FeesFor,
  Inputs,
  OutcomeData,
  PlanData,
  PlanInfo,
  PlanKey,
  Purchase,
  Security,
} from './core/core'
import { DEFAULT_RATES, fetchRates } from './rates'
import { load, save } from './saved'

/** Chosen to differ as much as they can on the dark background, with color
 * blindness too: the first 8 clearly, the rest less so, as 14 colors can't
 * all be far apart. Checked with the dataviz skill's validator. */
const PLAN_COLORS = [
  '#56b4e9',
  '#e69f00',
  '#8d73f0',
  '#c1e319',
  '#e141aa',
  '#3ceedd',
  '#fd2e1c',
  '#fc899d',
  '#27ef8e',
  '#dc855d',
  '#39bda0',
  '#c86b82',
  '#adc367',
  '#4493d0',
]
/** An unticked plan isn't drawn, so it only needs a color where it's listed. */
const UNTICKED_COLOR = '#8e95a5'

/** One of the user's own plans, as kept in this browser. */
export interface YourPlan {
  id: string
  /** Made and changed only by the core. */
  plan: PlanData
  /** A copy's broker, or the one the user typed for their own plan. */
  brokerName: string | null
  /** The listed plan it's a copy of, and the track it was copied on, by
   * name: positions shift when brokers are added. */
  basedOn: { broker: string; plan: string; track?: string } | null
}

/** A plan, listed or the user's own, with its color: handed out when it's
 * ticked, so the ticked plans' colors differ as much as they can. */
export interface Plan {
  key: PlanKey
  /** For sets and keyed lists. */
  id: string
  info: PlanInfo
  color: string
  /** Under its name: "Bank Leumi", "Your deal · Bank Leumi", "Your own". */
  subtitle: string
  /** Its name with its broker's, where the subtitle isn't beside it: plan
   * names repeat across brokers. "Meitav · Typical offer". */
  label: string
  /** A listed plan's broker. */
  broker?: BrokerInfo
  /** One of the user's own. */
  yours?: YourPlan
  /** The listed plan it's a copy of, if it's still listed. */
  original?: Plan
}

/** One plan's line in the results, best first. */
export interface Result {
  plan: Plan
  /** Missing if the plan doesn't offer the security on that exchange. */
  outcome: OutcomeData | undefined
  /** The plan's track that was used, the cheapest, if its tracks price the security. */
  track: number | undefined
  /** Why the plan can't be used as the inputs are: "Needs a one-time deposit of at least ₪5,000". */
  warning: string | undefined
  /** Why its numbers may be too low: "May cost more: conversion markup not published". */
  mayCostMore: string | undefined
  /** What makes sense of its numbers: "A standing order buys every month, so it isn't used here". */
  note: string | undefined
  /** Why it has no numbers, if it doesn't offer the security there: "Nothing in Europe is offered". */
  notOffered: string | undefined
}

export type ChartView = 'value' | 'lost' | 'breakdown'

/** The chosen plans, best first, or why they couldn't be compared.
 * `largestTrade`: the biggest single order, in the exchange's currency. */
export type Comparison =
  { results: Result[]; noFees: OutcomeData; deposited: number; largestTrade: number } | { error: string }

/** What the details dialog shows. A plan of the user's own opens in the
 * editor, and so does a draft, before "Add plan". `about` is the page on how
 * the numbers are made, opened at one of its sections. */
export type Details =
  | { kind: 'broker'; broker: BrokerInfo }
  | { kind: 'plan'; plan: Plan }
  | { kind: 'draft'; draft: YourPlan }
  | { kind: 'about'; section?: number }

export type EditorView = 'simple' | 'full'

type RatesStatus =
  { kind: 'downloading' } | { kind: 'downloaded'; date: string } | { kind: 'failed'; error: string }

export const planId = (key: PlanKey): string =>
  key.kind === 'listed' ? `${key.broker}:${key.plan}` : `yours:${key.id}`

/** Your plans saved by an earlier visit, leaving out any the core can't read
 * (saved by another version of the app). */
function loadYourPlans(): YourPlan[] {
  const saved = load<YourPlan[]>('your-plans-v1', [])
  if (!Array.isArray(saved)) return []
  return saved.filter((yours) => {
    try {
      core.planInfo(yours.plan)
      return true
    } catch (error) {
      console.warn('Dropped a saved plan the app can no longer read', yours, error)
      return false
    }
  })
}

/** `plan` with a color looked up whenever it's read, so a plan stays the
 * same object as its color changes. */
function withColor<T extends object>(plan: T, color: () => string): T & { color: string } {
  return Object.defineProperty(plan, 'color', { get: color, enumerable: true }) as T & { color: string }
}

/** Everything the page shows, and what the user chose. The comparison is
 * derived: it's recalculated by the Rust core whenever an input changes. */
export class AppState {
  readonly brokers: BrokerInfo[] = core.brokers()
  /** How the numbers are made, what isn't counted, and the sources. */
  readonly about = core.about()
  readonly securities: Choice<Security>[] = core.securities()
  readonly exchanges: Choice<Exchange>[] = core.exchanges()
  readonly listedPlans: Plan[] = (() => {
    const plans = this.brokers.flatMap((broker, brokerIndex) =>
      broker.plans.map((info, planIndex) => ({
        broker,
        info,
        key: { kind: 'listed' as const, broker: brokerIndex, plan: planIndex },
        id: planId({ kind: 'listed', broker: brokerIndex, plan: planIndex }),
        subtitle: broker.name,
        label: `${broker.shortName} · ${info.name}`,
      })),
    )
    return plans.map((plan) => withColor(plan, () => this.colorOf(plan.id)))
  })()

  /** The user's own plans, oldest first. Raw: they're replaced, not changed. */
  yourPlans = $state.raw<YourPlan[]>(loadYourPlans())

  /** Listed plans, then the user's own. */
  plans: Plan[] = $derived.by(() => {
    const yours = this.yourPlans.map((yours): Plan => {
      const original = this.originalOf(yours)
      const brokerName = yours.brokerName?.trim()
      const info = core.planInfo(yours.plan)
      const id = planId({ kind: 'yours', id: yours.id })
      // A copy is drawn like its original, dotted.
      return withColor(
        {
          key: { kind: 'yours', id: yours.id },
          id,
          info,
          subtitle: original
            ? `Your deal · ${original.subtitle}`
            : brokerName
              ? `${brokerName} · your own`
              : 'Your own',
          label: original
            ? `${original.broker!.shortName} · ${info.name}`
            : brokerName
              ? `${brokerName} · ${info.name}`
              : info.name,
          yours,
          original,
        },
        () => this.colorOf(original?.id ?? id),
      )
    })
    return [...this.listedPlans, ...yours]
  })

  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
  plansById: ReadonlyMap<string, Plan> = $derived(new Map(this.plans.map((plan) => [plan.id, plan])))

  security = $state<Security>('Etf')
  exchange = $state<Exchange>('Usa')
  // Numbers are null while their field is empty.
  firstDeposit = $state<number | null>(10_000)
  monthlyDeposit = $state<number | null>(2_000)
  yearlyReturnPercent = $state<number | null>(10)
  years = $state(20)
  buyEveryMonths = $state(1)
  sharePrice = $state<number | null>(500)
  ilsPerUsd = $state<number | null>(DEFAULT_RATES.ilsPerUsd)
  ilsPerEur = $state<number | null>(DEFAULT_RATES.ilsPerEur)
  ratesStatus = $state<RatesStatus>({ kind: 'downloading' })
  /** Ticked: at first, the plan a new customer usually gets from each
   * broker, and all of the user's own. */
  selected = new SvelteSet<string>(
    this.plans
      .filter((plan) => plan.key.kind === 'yours' || plan.key.plan === plan.broker!.newCustomerPlan)
      .map((plan) => plan.id),
  )
  /** The colors handed out, by plan id; a copy goes by its original's.
   * Plain, not state: only `colors` reads and changes it. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- remembered between runs of `colors`
  private readonly handedOut = new Map<string, string>()

  /** The ticked plans' colors. Each keeps its color while it's ticked, and a
   * newly ticked plan takes the palette's first free color, so they differ
   * as much as they can. A copy shares its original's, as it's drawn dotted. */
  private colors: ReadonlyMap<string, string> = $derived.by(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    const ticked = new Set(
      this.plans.filter((plan) => this.selected.has(plan.id)).map((plan) => plan.original?.id ?? plan.id),
    )
    for (const id of this.handedOut.keys()) if (!ticked.has(id)) this.handedOut.delete(id)
    for (const id of ticked) {
      if (this.handedOut.has(id)) continue
      // Past the palette's end, the least used color.
      const used = [...this.handedOut.values()]
      const uses = (color: string) => used.filter((each) => each === color).length
      const color = PLAN_COLORS.reduce((least, each) => (uses(each) < uses(least) ? each : least))
      this.handedOut.set(id, color)
    }
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a copy, never changed
    return new Map(this.handedOut)
  })

  private colorOf(id: string): string {
    return this.colors.get(id) ?? UNTICKED_COLOR
  }

  /** The editor's view, as last chosen. */
  editorView = $state<EditorView>(load<EditorView>('editor-view', 'simple'))

  chartView = $state<ChartView>('value')
  /** Plans clicked in the table or the chart; they stay highlighted. */
  pinned = new SvelteSet<string>()
  /** The plan under the mouse, in the table or the chart. */
  hovered = $state<string | null>(null)
  /** Plans to make stand out: the pinned ones and the hovered one. */
  highlighted: ReadonlySet<string> = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    new Set([...this.pinned, ...(this.hovered ? [this.hovered] : [])]),
  )

  /** Open in the details dialog. Raw, so its broker and plan stay the same
   * objects as in `brokers` and `plans` (not proxies), to find them there. */
  details = $state.raw<Details | null>(null)

  /** "an ETF bought in the USA" */
  purchase = $derived(core.purchasePhrase(this.security, this.exchange))

  /** A plan's fees for what the user buys, in words: on the track the
   * comparison picked, if it picked one. */
  feesFor(plan: Plan): FeesFor {
    return plan.key.kind === 'listed'
      ? core.feesFor(plan.key.broker, plan.key.plan, this.buying, this.trackOf(plan))
      : core.feesForPlan(plan.yours!.plan, this.buying)
  }

  /** The track the comparison picked for a plan, if it's compared and its
   * tracks price what the user buys. */
  trackOf(plan: Plan): number | undefined {
    const { comparison } = this
    if ('error' in comparison) return undefined
    return comparison.results.find((result) => result.plan.id === plan.id)?.track
  }

  /** The listed plan `yours` is a copy of, if it's still listed. */
  originalOf(yours: YourPlan): Plan | undefined {
    const { basedOn } = yours
    return basedOn
      ? this.listedPlans.find((plan) => plan.subtitle === basedOn.broker && plan.info.name === basedOn.plan)
      : undefined
  }

  /** The position of the track `yours` was copied on, in its original's
   * tracks; none if it wasn't on one, or the track is gone. */
  originalTrack(yours: YourPlan, original: Plan): number | undefined {
    const track = original.info.tariff.tracks.findIndex(({ name }) => name === yours.basedOn?.track)
    return track === -1 ? undefined : track
  }

  /** Opens a copy of a listed plan in the editor, not yet added: on the
   * track the comparison picked, so it charges what the table shows. */
  draftCopyOf(plan: Plan) {
    if (plan.key.kind !== 'listed') return
    const broker = plan.subtitle
    const track = this.trackOf(plan)
    this.details = {
      kind: 'draft',
      draft: {
        id: crypto.randomUUID(),
        plan: core.copyOf(plan.key.broker, plan.key.plan, track),
        brokerName: broker,
        basedOn: { broker, plan: plan.info.name, track: plan.info.tariff.tracks[track ?? 0]?.name },
      },
    }
  }

  /** Opens a new plan of the user's own in the editor, not yet added:
   * "Your plan", or "Your plan 2" if that's taken, and so on. */
  draftNewPlan() {
    const names = this.plans.map((plan) => plan.info.name)
    let name = 'Your plan'
    for (let count = 2; names.includes(name); count++) name = `Your plan ${count}`
    this.details = {
      kind: 'draft',
      draft: { id: crypto.randomUUID(), plan: core.newPlan(name), brokerName: null, basedOn: null },
    }
  }

  /** Adds a draft to your plans, ticked. */
  addYourPlan(yours: YourPlan) {
    this.yourPlans = [...this.yourPlans, yours]
    this.selected.add(planId({ kind: 'yours', id: yours.id }))
  }

  updateYourPlan(yours: YourPlan) {
    this.yourPlans = this.yourPlans.map((each) => (each.id === yours.id ? yours : each))
  }

  deleteYourPlan(id: string) {
    const planKey = planId({ kind: 'yours', id })
    this.setSelected([planKey], false)
    if (this.hovered === planKey) this.hovered = null
    this.yourPlans = this.yourPlans.filter((yours) => yours.id !== id)
  }

  /** A broker's caveats about all its plans, for what the user buys, grouped
   * by how sure they are. */
  brokerCaveats(broker: BrokerInfo): CaveatGroup[] {
    return core.brokerCaveats(this.brokers.indexOf(broker), this.buying)
  }

  /** Ticks or unticks plans. Unticked plans are unpinned too. */
  setSelected(ids: string[], selected: boolean) {
    for (const id of ids) {
      if (selected) {
        this.selected.add(id)
      } else {
        this.selected.delete(id)
        this.pinned.delete(id)
      }
    }
  }

  /** Clicking a pinned plan unpins it. */
  togglePin(id: string) {
    if (!this.pinned.delete(id)) this.pinned.add(id)
  }

  /** Shows what a plan's fees went to: the breakdown, with the plan pinned
   * last, so it's the one shown year by year. */
  showFees(id: string) {
    this.chartView = 'breakdown'
    this.pinned.delete(id)
    this.pinned.add(id)
  }

  /** Everything the core compares the plans by. */
  inputs: Inputs = $derived({
    security: this.security,
    exchange: this.exchange,
    firstDeposit: this.firstDeposit,
    monthlyDeposit: this.monthlyDeposit,
    yearlyReturnPercent: this.yearlyReturnPercent,
    years: this.years,
    buyEveryMonths: this.buyEveryMonths,
    sharePrice: this.sharePrice,
    ilsPerUsd: this.ilsPerUsd,
    ilsPerEur: this.ilsPerEur,
    plans: this.plans.filter((plan) => this.selected.has(plan.id)).map((plan) => plan.key),
    yourPlans: this.yourPlans.map(({ id, plan }) => ({ id, plan })),
  })

  /** The share price's currency, "$", if the share price matters to what's
   * compared; its field is hidden otherwise. */
  sharePriceSymbol = $derived.by(() => {
    try {
      return core.sharePriceSymbol(this.inputs)
    } catch {
      // The comparison says what's wrong.
      return undefined
    }
  })

  /** The chosen plans, best first, or why they couldn't be compared. */
  comparison = $derived.by((): Comparison => {
    let data: ComparisonData
    try {
      data = core.compare(this.inputs)
    } catch (error) {
      return {
        error: error instanceof Error ? error.message : String(error),
      }
    }
    return {
      deposited: data.deposited,
      noFees: data.noFees,
      largestTrade: data.largestTrade,
      results: data.plans.map(({ key, outcome, track, warning, mayCostMore, note, notOffered }) => ({
        plan: this.plansById.get(planId(key))!,
        outcome,
        track,
        warning,
        mayCostMore,
        note,
        notOffered,
      })),
    }
  })

  /** What the user buys, for the fees and caveats that matter to them, with
   * the comparison's biggest order: a caveat about large orders shows only
   * when the user's orders reach it. */
  buying: Purchase = $derived({
    security: this.security,
    exchange: this.exchange,
    largestTrade: 'error' in this.comparison ? null : this.comparison.largestTrade,
    ilsPerUsd: this.ilsPerUsd,
    ilsPerEur: this.ilsPerEur,
  })

  constructor() {
    this.loadRates()
    $effect(() => save('your-plans-v1', this.yourPlans))
    $effect(() => save('editor-view', this.editorView))
  }

  private async loadRates() {
    try {
      const { ilsPerUsd, ilsPerEur, date } = await fetchRates()
      this.ilsPerUsd = ilsPerUsd
      this.ilsPerEur = ilsPerEur
      this.ratesStatus = { kind: 'downloaded', date }
    } catch (error) {
      this.ratesStatus = { kind: 'failed', error: String(error) }
    }
  }
}
