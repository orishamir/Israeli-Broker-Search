import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  BrokerInfo,
  Choice,
  ComparisonData,
  Exchange,
  FeesFor,
  OutcomeData,
  PlanData,
  PlanInfo,
  PlanKey,
  Security,
} from './core/core'
import { DEFAULT_RATES, fetchRates } from './rates'
import { load, save } from './saved'

/** The Okabe-Ito palette, distinguishable with color blindness, with its dark
 * blue last since it's hard to see on the dark background. Each plan keeps
 * its color everywhere. */
const PLAN_COLORS = ['#56b4e9', '#e69f00', '#009e73', '#cc79a7', '#f0e442', '#d55e00', '#aa78ff', '#0072b2']

/** One of the user's own plans, as kept in this browser. */
export interface YourPlan {
  id: string
  /** Made and changed only by the core. */
  plan: PlanData
  /** A copy's broker, or the one the user typed for their own plan. */
  brokerName: string | null
  /** The listed plan it's a copy of, by name: positions shift when brokers
   * are added. */
  basedOn: { broker: string; plan: string } | null
}

/** A plan, listed or the user's own, with its color. */
export interface Plan {
  key: PlanKey
  /** For sets and keyed lists. */
  id: string
  info: PlanInfo
  color: string
  /** Under its name: "Bank Leumi", "Your deal · Bank Leumi", "Your own". */
  subtitle: string
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
  /** Why the plan can't be used as the inputs are: "Needs a one-time deposit of at least ₪5,000". */
  warning: string | undefined
}

export type ChartView = 'value' | 'lost' | 'breakdown'

/** The chosen plans, best first, or why they couldn't be compared. */
export type Comparison = { results: Result[]; noFees: OutcomeData; deposited: number } | { error: string }

/** What the details dialog shows. A plan of the user's own opens in the
 * editor, and so does a draft, before "Add plan". */
export type Details =
  { kind: 'broker'; broker: BrokerInfo } | { kind: 'plan'; plan: Plan } | { kind: 'draft'; draft: YourPlan }

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

/** Everything the page shows, and what the user chose. The comparison is
 * derived: it's recalculated by the Rust core whenever an input changes. */
export class AppState {
  readonly brokers: BrokerInfo[] = core.brokers()
  readonly securities: Choice<Security>[] = core.securities()
  readonly exchanges: Choice<Exchange>[] = core.exchanges()
  readonly listedPlans: Plan[] = this.brokers
    .flatMap((broker, brokerIndex) =>
      broker.plans.map((info, planIndex) => ({
        broker,
        info,
        key: { kind: 'listed' as const, broker: brokerIndex, plan: planIndex },
        subtitle: broker.name,
      })),
    )
    .map((plan, index) => ({
      ...plan,
      id: planId(plan.key),
      color: PLAN_COLORS[index % PLAN_COLORS.length],
    }))

  /** The user's own plans, oldest first. Raw: they're replaced, not changed. */
  yourPlans = $state.raw<YourPlan[]>(loadYourPlans())

  /** Listed plans, then the user's own. */
  plans: Plan[] = $derived.by(() => {
    let own = 0
    const yours = this.yourPlans.map((yours): Plan => {
      const original = this.originalOf(yours)
      const brokerName = yours.brokerName?.trim()
      return {
        key: { kind: 'yours', id: yours.id },
        id: planId({ kind: 'yours', id: yours.id }),
        info: core.planInfo(yours.plan),
        // A copy is drawn like its original, dotted; others take the
        // palette's colors in turn.
        color: original?.color ?? PLAN_COLORS[own++ % PLAN_COLORS.length],
        subtitle: original
          ? `Your deal · ${original.subtitle}`
          : brokerName
            ? `${brokerName} · your own`
            : 'Your own',
        yours,
        original,
      }
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
  selected = new SvelteSet<string>(this.plans.map((plan) => plan.id))
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

  /** A plan's fees for what the user buys, in words. */
  feesFor(plan: Plan): FeesFor {
    return plan.key.kind === 'listed'
      ? core.feesFor(plan.key.broker, plan.key.plan, this.security, this.exchange)
      : core.feesForPlan(plan.yours!.plan, this.security, this.exchange)
  }

  /** The listed plan `yours` is a copy of, if it's still listed. */
  originalOf(yours: YourPlan): Plan | undefined {
    const { basedOn } = yours
    return basedOn
      ? this.listedPlans.find((plan) => plan.subtitle === basedOn.broker && plan.info.name === basedOn.plan)
      : undefined
  }

  /** Opens a copy of a listed plan in the editor, not yet added. */
  draftCopyOf(plan: Plan) {
    if (plan.key.kind !== 'listed') return
    const broker = plan.subtitle
    this.details = {
      kind: 'draft',
      draft: {
        id: crypto.randomUUID(),
        plan: core.copyOf(plan.key.broker, plan.key.plan),
        brokerName: broker,
        basedOn: { broker, plan: plan.info.name },
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

  /** Shows what a plan's fees went to: the breakdown, with the plan pinned
   * last, so it's the one shown year by year. */
  showFees(id: string) {
    this.chartView = 'breakdown'
    this.pinned.delete(id)
    this.pinned.add(id)
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
        yourPlans: this.yourPlans.map(({ id, plan }) => ({ id, plan })),
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
