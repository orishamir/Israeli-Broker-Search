import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  BrokerInfo,
  Choice,
  ComparisonData,
  Exchange,
  FeesFor,
  OutcomeData,
  PlanInfo,
  PlanKey,
  Security,
} from './core/core'
import { DEFAULT_RATES, fetchRates } from './rates'

/** The Okabe-Ito palette, distinguishable with color blindness, with its dark
 * blue last since it's hard to see on the dark background. Each plan keeps
 * its color everywhere. */
const PLAN_COLORS = ['#56b4e9', '#e69f00', '#009e73', '#cc79a7', '#f0e442', '#d55e00', '#aa78ff', '#0072b2']

/** A plan, with its broker and its color. */
export interface Plan {
  key: PlanKey
  /** "broker:plan", for sets and keyed lists. */
  id: string
  broker: BrokerInfo
  info: PlanInfo
  color: string
}

/** One plan's line in the results, best first. */
export interface Result {
  plan: Plan
  /** Missing if the plan doesn't offer the security on that exchange. */
  outcome: OutcomeData | undefined
  /** Why the plan can't be used as the inputs are: "Needs a first deposit of at least ₪5,000". */
  warning: string | undefined
}

export type ChartView = 'value' | 'lost'

/** The chosen plans, best first, or why they couldn't be compared. */
export type Comparison = { results: Result[]; noFees: OutcomeData; deposited: number } | { error: string }

/** What the details dialog shows. */
export type Details = { kind: 'broker'; broker: BrokerInfo } | { kind: 'plan'; plan: Plan }

type RatesStatus =
  { kind: 'downloading' } | { kind: 'downloaded'; date: string } | { kind: 'failed'; error: string }

export const planId = ({ broker, plan }: PlanKey): string => `${broker}:${plan}`

/** Everything the page shows, and what the user chose. The comparison is
 * derived: it's recalculated by the Rust core whenever an input changes. */
export class AppState {
  readonly brokers: BrokerInfo[] = core.brokers()
  readonly securities: Choice<Security>[] = core.securities()
  readonly exchanges: Choice<Exchange>[] = core.exchanges()
  readonly plans: Plan[] = this.brokers
    .flatMap((broker, brokerIndex) =>
      broker.plans.map((info, planIndex) => ({
        broker,
        info,
        key: { broker: brokerIndex, plan: planIndex },
      })),
    )
    .map((plan, index) => ({
      ...plan,
      id: planId(plan.key),
      color: PLAN_COLORS[index % PLAN_COLORS.length],
    }))

  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- never changed
  private readonly plansById: ReadonlyMap<string, Plan> = new Map(this.plans.map((plan) => [plan.id, plan]))

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
  selected = new SvelteSet<string>(this.plans.map((plan) => plan.id))

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

  /** A plan's fees for what the user buys, in words. */
  feesFor(plan: Plan): FeesFor {
    return core.feesFor(plan.key.broker, plan.key.plan, this.security, this.exchange)
  }

  /** A broker's caveats about all its plans, for what the user buys. */
  brokerCaveats(broker: BrokerInfo): string[] {
    return core.brokerCaveats(this.brokers.indexOf(broker), this.security, this.exchange)
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

  /** The chosen plans, best first, or why they couldn't be compared. */
  comparison = $derived.by((): Comparison => {
    let data: ComparisonData
    try {
      data = core.compare({
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
      })
    } catch (error) {
      return {
        error: error instanceof Error ? error.message : String(error),
      }
    }
    return {
      deposited: data.deposited,
      noFees: data.noFees,
      results: data.plans.map(({ key, outcome, warning }) => ({
        plan: this.plansById.get(planId(key))!,
        outcome,
        warning,
      })),
    }
  })

  constructor() {
    this.loadRates()
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
