import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  AroundData,
  BrokerInfo,
  CaveatGroup,
  Choice,
  ComparisonData,
  Exchange,
  ExampleData,
  FeesFor,
  Inputs,
  OutcomeData,
  PlanData,
  PlanInfo,
  PlanKey,
  Purchase,
  Security,
  SweepData,
  Swept,
} from './core/core'
import { aroundWords } from './around'
import { shekels } from './format'
import { encode, type Shared } from './link'
import { DEFAULT_RATES, todaysRates } from './rates'
import { load, save } from './saved'
import type { SweepRequest } from './sweeper'
import { t } from './text'

/** Chosen to differ as much as they can on the dark background, with color
 * blindness too: the first 8 clearly, the rest less so, as 14 colors can't
 * all be far apart. Checked with the dataviz skill's validator. */
export const PLAN_COLORS = [
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
  /** The label in English, the same in every language: how links name
   * listed plans. */
  englishLabel: string
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
  /** Its row's number in the table, 1 for the best; the charts show it at
   * each line's end, so a color used twice can't mix two plans up. Missing
   * with the outcome. */
  rank: number | undefined
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

export type ChartView = 'value' | 'lost' | 'crossover' | 'breakdown'
/** The chart shown at first: what the fees cost, over the years. */
const FIRST_VIEW: ChartView = 'lost'

/** What happens at the end of the years: everything is sold, or kept. */
export type AtEnd = 'sell' | 'hold'

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
  return Array.isArray(saved) ? readable(saved, 'a saved plan') : []
}

/** `plans` without any the core can't read: saved, or in a link, by
 * another version of the app. */
function readable(plans: YourPlan[], what: string): YourPlan[] {
  return plans.filter((yours) => {
    try {
      core.planInfo(yours.plan)
      return true
    } catch (error) {
      console.warn(`Dropped ${what} the app can't read`, yours, error)
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
  /** Ready-made investing patterns, each setting every basic input. */
  readonly examples: ExampleData[] = core.examples()
  readonly listedPlans: Plan[] = (() => {
    const plans = this.brokers.flatMap((broker, brokerIndex) =>
      broker.plans.map((info, planIndex) => ({
        broker,
        info,
        key: { kind: 'listed' as const, broker: brokerIndex, plan: planIndex },
        id: planId({ kind: 'listed', broker: brokerIndex, plan: planIndex }),
        subtitle: broker.name,
        label: `${broker.shortName} · ${info.name}`,
        englishLabel: `${broker.englishShortName} · ${info.englishName}`,
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
            ? t.yourDeal(original.subtitle)
            : brokerName
              ? t.brokerYourOwn(brokerName)
              : t.yourOwn,
          label: original
            ? `${original.broker!.shortName} · ${info.name}`
            : brokerName
              ? `${brokerName} · ${info.name}`
              : info.name,
          englishLabel: info.name,
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
  /** The expert inputs below are shown, and used. Off, the deposits stay the
   * same, nothing is taken off for inflation and everything is sold at the
   * end, whatever their fields say. Remembered. */
  moreOptions = $state(load<boolean>('more-options', false))
  depositGrowthPercent = $state<number | null>(0)
  inflationPercent = $state<number | null>(0)
  atEnd = $state<AtEnd>('sell')
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

  private chosenView = $state<ChartView>(FIRST_VIEW)
  /** The chart shown. The chart by deposit is for experts: without More
   * options it's the first one instead, and the choice comes back with them. */
  get chartView(): ChartView {
    return this.chosenView === 'crossover' && !this.moreOptions ? FIRST_VIEW : this.chosenView
  }
  set chartView(view: ChartView) {
    this.chosenView = view
  }
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
    // Named in English, the same in every language.
    return basedOn
      ? this.listedPlans.find(
          (plan) => plan.broker!.englishName === basedOn.broker && plan.info.englishName === basedOn.plan,
        )
      : undefined
  }

  /** The position of the track `yours` was copied on, in its original's
   * tracks; none if it wasn't on one, or the track is gone. */
  originalTrack(yours: YourPlan, original: Plan): number | undefined {
    const track = original.info.tariff.tracks.findIndex(
      ({ englishName }) => englishName === yours.basedOn?.track,
    )
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
        basedOn: {
          broker: plan.broker!.englishName,
          plan: plan.info.englishName,
          track: plan.info.tariff.tracks[track ?? 0]?.englishName,
        },
      },
    }
  }

  /** Opens a new plan of the user's own in the editor, not yet added:
   * "Your plan", or "Your plan 2" if that's taken, and so on. */
  draftNewPlan() {
    const names = this.plans.map((plan) => plan.info.name)
    let name = t.yourPlan
    for (let count = 2; names.includes(name); count++) name = t.yourPlanNumbered(count)
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

  /** Whether everything is sold at the end, as the core is told. */
  sellAtEnd: boolean = $derived(!this.moreOptions || this.atEnd === 'sell')
  /** Whether the amounts shown are in today's money. */
  inTodaysMoney: boolean = $derived(this.moreOptions && (this.inflationPercent ?? 0) !== 0)

  /** Everything the core compares the plans by. */
  inputs: Inputs = $derived(this.inputsWith(this.firstDeposit, this.monthlyDeposit))

  /** The inputs with the deposits given: the comparison's own, or the
   * sweep's, which leaves out the deposit it varies. */
  private inputsWith(firstDeposit: number | null, monthlyDeposit: number | null): Inputs {
    return {
      security: this.security,
      exchange: this.exchange,
      firstDeposit,
      monthlyDeposit,
      yearlyReturnPercent: this.yearlyReturnPercent,
      years: this.years,
      buyEveryMonths: this.buyEveryMonths,
      sharePrice: this.sharePrice,
      depositGrowthPercent: this.moreOptions ? this.depositGrowthPercent : 0,
      inflationPercent: this.moreOptions ? this.inflationPercent : 0,
      sellAtEnd: this.sellAtEnd,
      ilsPerUsd: this.ilsPerUsd,
      ilsPerEur: this.ilsPerEur,
      plans: this.plans.filter((plan) => this.selected.has(plan.id)).map((plan) => plan.key),
      yourPlans: this.yourPlans.map(({ id, plan }) => ({ id, plan })),
    }
  }

  /** Which deposit the chart by deposit varies: the monthly one if there is
   * one, else the one-time deposit. */
  swept: Swept = $derived((this.monthlyDeposit ?? 0) > 0 ? 'Monthly' : 'OneTime')

  /** What the sweep is asked for: the inputs without the deposit it varies,
   * so typing that deposit asks for nothing new (only the chart's marker
   * moves). A new object whenever the inputs change, to match answers to. */
  sweepRequest: SweepRequest = $derived.by(() => {
    const swept = this.swept
    return {
      swept,
      inputs: this.inputsWith(
        swept === 'OneTime' ? 0 : this.firstDeposit,
        swept === 'Monthly' ? 0 : this.monthlyDeposit,
      ),
    }
  })

  /** The latest sweep worked out, and the request it answers: worked out
   * off the page's thread (see sweeper.ts), so it trails the inputs by a
   * moment. Set by App.svelte. Raw, so the request is matched by identity. */
  sweepAnswer = $state.raw<{ request: SweepRequest; sweep: SweepData | undefined }>()

  /** The chart by deposit's lines: every ticked plan's yearly cost over a
   * range of the swept deposit, as last worked out. Missing while the
   * inputs are invalid. */
  sweep: SweepData | undefined = $derived(this.sweepAnswer?.sweep)

  /** Where the best plan stops being the cheapest at other values of the
   * swept deposit, if the sweep answers the inputs as they are now. */
  around: AroundData | undefined = $derived.by(() => {
    const answer = this.sweepAnswer
    if (!answer?.sweep || answer.request !== this.sweepRequest || 'error' in this.comparison) return undefined
    try {
      return core.around({
        sweep: answer.sweep,
        deposit: (this.swept === 'Monthly' ? this.monthlyDeposit : this.firstDeposit) ?? 0,
        costs: this.comparison.results.map(({ plan, outcome }) => ({
          key: plan.key,
          cost: outcome?.yearlyCostPercent ?? null,
        })),
      })
    } catch {
      return undefined
    }
  })

  /** The line under the best plan about other deposits. Between a change
   * and the sweep's answer it keeps its last words, shown only while they're
   * about the plan that's still the best; hidden, it keeps its place, so the
   * table below doesn't move when the words come. Before the first answer
   * it holds words of about the right length, hidden. */
  aroundLine: { text: string; shown: boolean } = $derived.by(() => {
    const best = this.best
    if (this.around && best) {
      const labelOf = (key: PlanKey) => this.plansById.get(planId(key))?.label ?? ''
      this.lastAround = { planId: best.plan.id, text: aroundWords(this.around, this.swept, labelOf) }
      return { text: this.lastAround.text, shown: true }
    }
    const last = this.lastAround
    if (!last) return { text: t.cheapestThroughout(shekels(100), shekels(32_000), this.swept), shown: false }
    const answered = this.sweepAnswer?.request === this.sweepRequest
    return { text: last.text, shown: !answered && last.planId === best?.plan.id }
  })
  /** The line's last words, and the plan they're about. Plain: only
   * `aroundLine` reads and writes it. */
  private lastAround: { planId: string; text: string } | undefined

  /** Fills every basic input from an example. The expert inputs stay. */
  applyExample(example: ExampleData) {
    this.security = example.security
    this.exchange = example.exchange
    this.firstDeposit = example.firstDeposit
    this.monthlyDeposit = example.monthlyDeposit
    this.yearlyReturnPercent = example.yearlyReturnPercent
    this.years = example.years
    this.buyEveryMonths = example.buyEveryMonths
  }

  /** A link to this comparison: the inputs and the ticked plans, your own
   * ticked plans included, in the page's address. */
  shareLink(): string {
    const ticked = this.plans.filter((plan) => this.selected.has(plan.id))
    const shared: Shared = {
      security: this.security,
      exchange: this.exchange,
      firstDeposit: this.firstDeposit ?? undefined,
      monthlyDeposit: this.monthlyDeposit ?? undefined,
      yearlyReturnPercent: this.yearlyReturnPercent ?? undefined,
      years: this.years,
      buyEveryMonths: this.buyEveryMonths,
      sharePrice: this.sharePrice ?? undefined,
      plans: ticked.filter((plan) => plan.key.kind === 'listed').map((plan) => plan.englishLabel),
      yours: ticked.flatMap((plan) => (plan.yours ? [plan.yours] : [])),
    }
    // The expert inputs only when they'd change something.
    if (this.moreOptions) {
      if (this.depositGrowthPercent) shared.depositGrowthPercent = this.depositGrowthPercent
      if (this.inflationPercent) shared.inflationPercent = this.inflationPercent
      if (!this.sellAtEnd) shared.sellAtEnd = false
    }
    return `${location.origin}${location.pathname}#${encode(shared)}`
  }

  /** Takes what a link says, over the defaults. Its plans, listed and your
   * own, replace what's ticked; your plans from it join yours, a plan with
   * the same id (from your own other device) taking the newer version. */
  private applyShared(shared: Shared) {
    if (shared.security !== undefined) this.security = shared.security
    if (shared.exchange !== undefined) this.exchange = shared.exchange
    if (shared.firstDeposit !== undefined) this.firstDeposit = shared.firstDeposit
    if (shared.monthlyDeposit !== undefined) this.monthlyDeposit = shared.monthlyDeposit
    if (shared.yearlyReturnPercent !== undefined) this.yearlyReturnPercent = shared.yearlyReturnPercent
    if (shared.years !== undefined) this.years = shared.years
    if (shared.buyEveryMonths !== undefined) this.buyEveryMonths = shared.buyEveryMonths
    if (shared.sharePrice !== undefined) this.sharePrice = shared.sharePrice
    if (shared.depositGrowthPercent !== undefined) this.depositGrowthPercent = shared.depositGrowthPercent
    if (shared.inflationPercent !== undefined) this.inflationPercent = shared.inflationPercent
    if (shared.sellAtEnd === false) this.atEnd = 'hold'
    if (shared.depositGrowthPercent || shared.inflationPercent || shared.sellAtEnd === false) {
      this.moreOptions = true
    }
    const yours = readable(shared.yours ?? [], 'a plan from the link')
    if (yours.length > 0) {
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const fromLink = new Map(yours.map((plan) => [plan.id, plan]))
      this.yourPlans = [
        ...this.yourPlans.map((mine) => fromLink.get(mine.id) ?? mine),
        ...yours.filter((plan) => !this.yourPlans.some((mine) => mine.id === plan.id)),
      ]
    }
    if (shared.plans !== undefined || yours.length > 0) {
      this.selected.clear()
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const labels = new Set(shared.plans ?? [])
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const ids = new Set(yours.map(({ id }) => planId({ kind: 'yours', id })))
      for (const plan of this.plans) {
        if ((plan.key.kind === 'listed' && labels.has(plan.englishLabel)) || ids.has(plan.id))
          this.selected.add(plan.id)
      }
    }
  }

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
      // Best first, and the plans without an outcome last.
      results: data.plans.map(({ key, outcome, track, warning, mayCostMore, note, notOffered }, index) => ({
        plan: this.plansById.get(planId(key))!,
        outcome,
        rank: outcome ? index + 1 : undefined,
        track,
        warning,
        mayCostMore,
        note,
        notOffered,
      })),
    }
  })

  /** The table's first plan. One that can't be opened with these deposits
   * (a minimum first deposit) is still the best: its warning is shown with
   * it, rather than a dearer plan named as best. */
  best: Result | undefined = $derived(
    'error' in this.comparison ? undefined : this.comparison.results.find((result) => result.outcome),
  )

  /** Each plan's row number in the table, by id, for the chart by deposit:
   * its sweep answers a moment after the table, which it may not match. */
  private ranks: ReadonlyMap<string, number> = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    new Map(
      'error' in this.comparison
        ? []
        : this.comparison.results.flatMap(({ plan, rank }) =>
            rank === undefined ? [] : [[plan.id, rank] as const],
          ),
    ),
  )

  rankOf(id: string): number | undefined {
    return this.ranks.get(id)
  }

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

  /** `shared`: what the page's address says, if it was opened from a link. */
  constructor(shared: Shared = {}) {
    this.applyShared(shared)
    this.loadRates()
    $effect(() => save('your-plans-v1', this.yourPlans))
    $effect(() => save('editor-view', this.editorView))
    $effect(() => save('more-options', this.moreOptions))
  }

  private async loadRates() {
    try {
      const { ilsPerUsd, ilsPerEur, date } = await todaysRates()
      this.ilsPerUsd = ilsPerUsd
      this.ilsPerEur = ilsPerEur
      this.ratesStatus = { kind: 'downloaded', date }
    } catch (error) {
      this.ratesStatus = { kind: 'failed', error: String(error) }
    }
  }
}
