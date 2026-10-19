// The page's state: one class, made once by App.svelte (opened from a link,
// perhaps). Everything the page shows is derived here from what the user
// chose, and the Rust core does the work. Components read these fields and
// call these methods, and nothing else:
//
// ANSWERS (the guide's questions; `$state`, edit freely, e.g. bind:value)
//   app.answers: AnswersInput        situation, now, monthly, horizon, flexibility,
//                                    cushion, needsAnyDay (null amounts = empty field)
//   app.situationId: string?         the chip chosen (SituationData.id)
//   app.situations: SituationData[]  the chips, in the core's order
//   app.setSituation(id)             fills the answers from a chip, clears overrides;
//                                    a 'workbench' chip turns the switch on, an
//                                    `opensDialog` one opens the portfolio dialog
//   app.knowsWhatToBuy()             the "אני יודע / יודעת מה אני רוצה לקנות" link: switch on
//   app.followUps: FollowUp[]        the cards the answers still need
//   app.setHorizonMonths(m) / setHorizonRetirement(age, retireAt)
//   app.horizonMonths: number | undefined
//   app.banks: BankChoice[]; app.setBank(bank | undefined)   the visitor's bank (optional,
//                                    asked in context; the existing-portfolio situation's too)
//
// THE REDESIGNED ANSWER (REDESIGN.md Part 2; replaces `recommendation` once W6 is done)
//   app.choices: Choices             the visitor's choices on the answer (not facts)
//   app.answer: AnswerData | undefined   chip, lines, decisions, plan, outcome chart,
//        how, gain, horizons. Up to 5 years worked out on the page's thread first
//        (without the horizons), then the worker's full answer; the last one stays
//        while the next is worked out (`answerFresh` false).
//   app.answerFresh: boolean
//   app.choose(decision, pick)       picks an option: a fact goes into the answers
//                                    (cushion, flexibility), the rest into `choices`
//   app.focusedRow: Kind | undefined the row "איך עושים את זה" shows: the one tapped
//                                    in the chart, else the answer's focus
//   app.focusRow(kind | undefined)   tap a row (undefined: back to the answer's)
//
// THE ANSWER
//   app.recommendation: RecommendationData | undefined   what fits, why, steps, plans
//        Computed on the page's thread when it is cheap (horizon <= 5 years or
//        unknown; <10 ms), in the worker otherwise; until the new one arrives the old
//        stays (`recommendationFresh` false). undefined only before the first valid one.
//   app.recommendationFresh: boolean
//   app.inputError: string | undefined   why the answers/inputs can't be compared; the
//        last results stay under it, faded (`stale` true)
//   app.stale: boolean
//   app.ladder: RungData[] | undefined, app.ladderFresh
//   app.alternatives: AlternativesData | undefined, app.alternativesFresh
//        (worker; keep the last answer while the next is being worked out. Wrap in
//        `HeldWhileTyping` where a chart should wait for the typing to pause.)
//   app.chooseKind(kind) / app.chooseArena(arena) / app.clearOverrides()
//        "לשנות" and "continue anyway": the kind or arena the user chose over the guide
//   app.overrides: Overrides; app.overridden: boolean
//
// THE SWITCH AND THE WORKBENCH (`expert`; turning it on changes only what is shown)
//   app.expert: boolean              persisted ('expert-v2'), in links (x=1)
//   app.thing: Thing                 'sp500' | 'other' | 'bonds' | 'short'; the
//        recommendation's unless `app.chooseThing(thing)` chose another
//   app.wayChoices: WayChoice[]; app.selectedWays: Way[]; app.toggleWay(way)
//   app.security / exchange / product, yearlyReturnPercent, inflationPercent,
//   boiRatePercent, depositGrowthPercent, buyEveryMonths, sharePrice, atEnd, wayOut,
//   age, todaysMoney (default true), ilsPerUsd, ilsPerEur, ratesStatus
//   app.waysComparison: WaysView | undefined      thing 'sp500': compareWays
//   app.comparison: Comparison | undefined        thing 'other': compare (plans)
//   app.best, app.rowOf(id), app.around, app.aroundLine, app.sweep   (as before)
//   app.chartView ('lost'|'value'|'crossover'|'breakdown'; guide mode offers value only)
//   app.resultsOf(plans: PlanOutcomeData[]): Result[]   a way's plans as table rows
//
// PLANS (as before): brokers, listedPlans, plans, plansById, selected (ticked ids),
//   compared, yourPlans, pinned/hovered/highlighted, details, editorView, feesFor,
//   draftCopyOf, draftNewPlan, addYourPlan, updateYourPlan, deleteYourPlan, setSelected,
//   togglePin, showFees, brokerCaveats.
//
// SHEETS AND DIALOGS (open and close by setting these; `Sheet` binds them)
//   app.cardStack: Term[]; app.cardTerm; app.openCard(term) pushes (the sheet opens
//        on the first); app.cardBack(); app.closeCard(); app.cardFor(term): LearnCard
//   app.glossaryOpen; app.glossary: LearnCard[] (lazy)
//   app.planSheetOpen: boolean       "מה משווים": the plans' list
//   app.portfolioDialogOpen: boolean the existing-portfolio situation's dialog
//   app.details / openAbout(section?)   brokers, plans, the editor, and the about page
//   app.about: About; app.footer: string[]; app.dataDates: string
//
// LINKS: app.shareLink(): string (the Share button); the constructor reads a `Shared`.

import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  AlternativesData,
  AnswerData,
  AnswerRequest,
  AnswersInput,
  Arena,
  AroundData,
  Bank,
  BankChoice,
  BrokerInfo,
  CaveatGroup,
  Choice,
  Choices,
  DecisionId,
  ComparisonData,
  Exchange,
  FeesFor,
  FollowCardData,
  FollowUp,
  GuideSettings,
  Horizon,
  Inputs,
  Kind,
  LearnCard,
  OutcomeData,
  Overrides,
  PlanData,
  PlanInfo,
  PlanKey,
  PlanOutcomeData,
  Pick,
  Product,
  Purchase,
  RecommendationData,
  RungData,
  Security,
  SituationData,
  SpreadData,
  SweepData,
  Swept,
  Term,
  WaysComparisonData,
  Way,
  WayChoice,
  WhyNot,
} from './core/core'
import { aroundWords } from './around'
import { branchPlans, yoursBranch } from './bank-plan'
import { handOut, UNTICKED_COLOR, withColor } from './colors'
import { shekels } from './format'
import { arenaOf, kindByKey, kindKey, wayByKey, wayKey } from './kinds'
import { encode, type Shared, type Thing } from './link'
import { DEFAULT_RATES, todaysRates } from './rates'
import { firstOfEachId, load, save } from './saved'
import { ask, askSweep, type GuideRequest, type SweepRequest } from './sweeper'
import { run, type Worked } from './work'
import { t } from './text'
import { unreadable } from './unreadable.svelte'

/** The calendar year, for the core's "in 2041's shekels" note. */
const THIS_YEAR = new Date().getFullYear()
/** The month, 1 to 12, for the answer's "in October 2029". */
const THIS_MONTH = new Date().getMonth() + 1

export { PLAN_COLORS } from './colors'
export type { Thing } from './link'

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
  broker?: BrokerInfo
  yours?: YourPlan
  /** The listed plan it's a copy of, if it's still listed. */
  original?: Plan
}

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
  /** Whether the first deposit is enough to open it: one that isn't comes
   * after every plan that is. */
  fits: boolean
  /** Why its numbers may be too low: "May cost more: conversion markup not published". */
  mayCostMore: string | undefined
  /** What makes sense of its numbers: "A standing order buys every month, so it isn't used here". */
  note: string | undefined
  /** Who can open it, if not anyone: "Self-employed". */
  whoCanJoin: string | undefined
  /** As a pension, the monthly amount the balance buys: "Expected pension:
   * about ₪10,000 a month for life, at a conversion factor of about 200". */
  pensionNote: string | undefined
  /** For a kind of fund's average, what's left at its cheapest and dearest
   * company and at the most allowed. */
  spread: SpreadData | undefined
  /** What became of a fund's tax: "No tax: your deposits are within ₪20,566 a year". */
  taxNote: string | undefined
  /** Why it has no numbers: "Nothing in Europe is offered", or how far the
   * deposits are over its yearly ceiling. */
  notOffered: string | undefined
  /** Which of those it is, for the few words the table itself says. */
  whyNot: WhyNot | undefined
}

export type ChartView = 'value' | 'lost' | 'crossover' | 'breakdown'
const FIRST_VIEW: ChartView = 'lost'

export type AtEnd = 'sell' | 'hold'

/** How the money is taken out at the end: all at once, or as a monthly
 * pension where a plan pays one. */
export type WayOut = 'atOnce' | 'pension'

/** The chosen plans, best first, or why they couldn't be compared.
 * `largestTrade`: the biggest single order, in the exchange's currency. */
export type Comparison =
  | {
      results: Result[]
      noFees: OutcomeData
      deposited: number
      largestTrade: number
      /** The US estate tax, when a broker's plan holds enough of a US
       * security: said once, under the table. */
      usEstateTax: string | undefined
    }
  | { error: string }

/** What the details dialog shows. A plan of the user's own opens in the
 * editor, and so does a draft, before "Add plan". `about` is the page on how
 * the numbers are made, opened at one of its sections. */
export type Details =
  | { kind: 'broker'; broker: BrokerInfo }
  | { kind: 'plan'; plan: Plan }
  | { kind: 'draft'; draft: YourPlan }
  | { kind: 'about'; section?: number }

/** The ways compared side by side, or why they couldn't be. */
export type WaysView = WaysComparisonData | { error: string }

/** What a worker answered, and the request it answers (matched by identity,
 * so an answer to an older request is known for what it is). */
type Answered<T> = Worked<T> & { request: GuideRequest }

/** The months a horizon is, if it says: "I don't know yet" doesn't. */
export function monthsOf(horizon: Horizon): number | undefined {
  if (horizon === 'Unknown') return undefined
  if ('Months' in horizon) return horizon.Months
  return Math.max(0, horizon.Retirement.retire_at - horizon.Retirement.age) * 12
}

/** Up to this horizon the recommendation is worked out on the page's thread
 * (measured in Node on the real core: 0.4 to 8 ms for 6 months to 5 years or
 * an unknown horizon, where the safe kinds win; 10 to 50 ms for 15 to 30
 * years, where the index arena is run). Beyond it the worker does it, and
 * the answer on screen is the last one until the new one arrives. */
export const RECOMMEND_ON_PAGE_UP_TO_MONTHS = 60

/** Where the guide opens: the situation of a dated goal in about five years. */
const FIRST_SITUATION = 'goal'
const NO_OVERRIDES: Overrides = { arena: undefined, kind: undefined }
const USUAL_RETURN_PERCENT = 10

/** The ways ticked at first: every way a broker's account or a fund's track
 * holds, except the ones the guide offers only when asked (a study fund,
 * a savings policy). */
const NOT_AT_FIRST: Product[] = ['StudyFundTrack', 'SavingsPolicyTrack']

export type EditorView = 'simple' | 'full'

type RatesStatus =
  { kind: 'downloading' } | { kind: 'downloaded'; date: string } | { kind: 'failed'; error: string }

export const planId = (key: PlanKey): string =>
  key.kind === 'listed' ? `${key.broker}:${key.plan}` : `yours:${key.id}`

/** Your plans saved by an earlier visit, leaving out any the core can't read
 * (saved by another version of the app). */
function loadYourPlans(): YourPlan[] {
  const saved = load<YourPlan[]>('your-plans-v1', [])
  return Array.isArray(saved) ? firstOfEachId(readable(saved, 'a saved plan')) : []
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

/** Everything the page shows, and what the user chose. Everything shown is
 * derived: the Rust core works it out again whenever an answer or an input
 * changes. */
export class AppState {
  readonly brokers: BrokerInfo[] = core.brokers()
  /** How much prices are taken to rise a year, in percent, unless the user
   * says otherwise: the core's, since the tax is worked out with it. */
  readonly usualInflation: number = core.usualInflationPercent()
  /** The surtax's rate and threshold, for the tax's tip. */
  readonly surtaxTerms = core.surtaxTerms() as [string, string]
  readonly dataDates: string = core.dataDates()
  readonly securities: Choice<Security>[] = core.securities()
  readonly exchanges: Choice<Exchange>[] = core.exchanges()
  /** The exchanges as offered for the chosen security: their explanations
   * say what buying it there means. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
  private readonly exchangesFor: ReadonlyMap<Security, Choice<Exchange>[]> = new Map(
    this.securities.map(({ value }) => [value, core.exchangesFor(value)]),
  )
  get exchangeChoices(): Choice<Exchange>[] {
    return this.exchangesFor.get(this.security)!
  }
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a cache of the core's answers
  private readonly productsCache = new Map<string, Choice<Product>[]>()
  /** The funds that can hold the chosen security on the chosen exchange, the
   * most held first; none for a share or a bond. */
  get productChoices(): Choice<Product>[] {
    const key = `${this.security}/${this.exchange}`
    let choices = this.productsCache.get(key)
    if (!choices) {
      choices = core.productsFor(this.security, this.exchange)
      this.productsCache.set(key, choices)
    }
    return choices
  }
  /** The fund the user picked, if it is still one of the choices. */
  get chosenProduct(): Product | undefined {
    return this.productChoices.find(({ value }) => value === this.product)?.value
  }

  readonly situations: SituationData[] = core.situations()
  readonly banks: BankChoice[] = core.banks()
  /** What "another bank" is priced as, in the core's words. */
  /** The label of the month's-expenses field, which the cushion's card holds. */
  readonly expensesLabel: string = core.followCard('MonthlyExpenses').field ?? ''
  readonly bankStandIn: string = core.bankStandInNote()
  readonly wayChoices: WayChoice[] = core.ways()
  readonly footer: string[] = core.footer()
  readonly about = core.aboutAll()
  /** The Bank of Israel's rate the saver is taken to expect, unless they say. */
  readonly usualBoiRate: number = core.todaysRatePercent()

  // ── The guide's answers ──
  /** Raw numbers are `null` while their field is empty. */
  answers = $state<AnswersInput>({
    situation: undefined,
    now: null,
    monthly: null,
    horizon: { Months: 60 },
    flexibility: undefined,
    cushion: undefined,
    needsAnyDay: false,
    bank: undefined,
  })
  situationId = $state<string | undefined>()
  /** What the visitor chose on the answer: the decisions that aren't facts
   * about the money (those stay in `answers`). */
  choices = $state<Choices>({})
  /** The row tapped in the outcome chart, over the answer's own focus. */
  private tappedRow = $state.raw<Kind | undefined>()
  /** The first visit's apartment scenario is on screen as an example, until
   * the saver changes anything (or opened a link, which is not an example). */
  exampleShown = $state(false)
  private exampleKey = ''
  /** What the user chose over the guide ("לשנות", "continue anyway"). */
  overrides = $state.raw<Overrides>(NO_OVERRIDES)
  overridden: boolean = $derived(this.overrides.arena !== undefined || this.overrides.kind !== undefined)

  /** The switch: the workbench is shown, and every lever. Remembered. Turning
   * it on changes only what is shown. */
  expert = $state(load<boolean>('expert-v2', false))

  // ── The workbench's inputs ──
  private chosenThing = $state<Thing | undefined>()
  private chosenWays = $state.raw<string[] | undefined>()
  security = $state<Security>('Etf')
  exchange = $state<Exchange>('Usa')
  /** The fund that holds the index, if the user picked one. */
  product = $state<Product | undefined>()
  sharePrice = $state<number | null>(500)
  yearlyReturnPercent = $state<number | null>(USUAL_RETURN_PERCENT)
  depositGrowthPercent = $state<number | null>(0)
  buyEveryMonths = $state(1)
  /** How much prices rise a year: the tax is on the gain beyond it. */
  inflationPercent = $state<number | null>(this.usualInflation)
  /** The Bank of Israel's rate the saver expects, for the safe kinds. */
  boiRatePercent = $state<number | null>(this.usualBoiRate)
  /** Amounts in today's shekels, with the nominal figure as a note. */
  todaysMoney = $state(true)
  atEnd = $state<AtEnd>('sell')
  /** How the money is taken out, as chosen; asked only while a ticked plan
   * pays a pension. */
  wayOut = $state<WayOut>('atOnce')
  /** The saver's age today: a pension opens from an age. */
  age = $state<number | null>(45)
  ilsPerUsd = $state<number | null>(DEFAULT_RATES.ilsPerUsd)
  ilsPerEur = $state<number | null>(DEFAULT_RATES.ilsPerEur)
  ratesStatus = $state<RatesStatus>({ kind: 'downloading' })

  /** The months of the horizon, if it says. */
  horizonMonths: number | undefined = $derived(monthsOf(this.answers.horizon))

  /** The ways ticked. Never none: none means every way to the core. */
  selectedWays: Way[] = $derived(
    this.chosenWays
      ? this.chosenWays.flatMap((key) => wayByKey(key) ?? [])
      : this.wayChoices.filter(({ product }) => !NOT_AT_FIRST.includes(product)).map(({ way }) => way),
  )

  /** Every plan the list offers. A kind of fund's cheapest and dearest
   * company and its maximum aren't among them: they're shown beside its
   * average (`shownBeside`). */
  readonly listedPlans: Plan[] = (() => {
    const plans = this.brokers.flatMap((broker, brokerIndex) =>
      broker.plans.flatMap((info, planIndex) =>
        broker.shownBeside.includes(planIndex)
          ? []
          : [
              {
                broker,
                info,
                key: { kind: 'listed' as const, broker: brokerIndex, plan: planIndex },
                id: planId({ kind: 'listed', broker: brokerIndex, plan: planIndex }),
                subtitle: broker.name,
                // A non-breaking space: a line mustn't start with the separator.
                label: `${broker.shortName}\u00a0· ${info.name}`,
                englishLabel: `${broker.englishShortName} · ${info.englishName}`,
              },
            ],
      ),
    )
    return plans.map((plan) => withColor(plan, () => this.colorOf(plan.id)))
  })()

  /** By English label, the average of the kind of fund each plan shown
   * beside an average belongs to: links made before they were, which tick
   * one, tick its average. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
  private readonly averageFor: ReadonlyMap<string, string> = new Map(
    this.brokers.flatMap((broker, brokerIndex) =>
      broker.shownBeside.map((planIndex) => [
        `${broker.englishShortName} · ${broker.plans[planIndex].englishName}`,
        planId({ kind: 'listed', broker: brokerIndex, plan: broker.newCustomerPlan }),
      ]),
    ),
  )

  /** The user's own plans, oldest first. Raw: they're replaced, not changed. */
  yourPlans = $state.raw<YourPlan[]>(loadYourPlans())

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
            ? `${original.broker!.shortName}\u00a0· ${info.name}`
            : brokerName
              ? `${brokerName}\u00a0· ${info.name}`
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

  selected = new SvelteSet<string>(
    this.plans
      .filter(
        (plan) =>
          plan.key.kind === 'yours' ||
          (plan.broker!.comparedAtFirst && plan.key.plan === plan.broker!.newCustomerPlan),
      )
      .map((plan) => plan.id),
  )
  /** The colors handed out, by plan id; a copy goes by its original's.
   * Plain, not state: only `colors` reads and changes it. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- remembered between runs of `colors`
  private readonly handedOut = new Map<string, string>()

  /** The ticked plans' colors. Each keeps its color while it's ticked, and a
   * newly ticked plan takes the palette's first free color, so they differ
   * as much as they can. A copy shares its original's, as it's drawn dotted. */
  private colors: ReadonlyMap<string, string> = $derived(
    handOut(
      this.handedOut,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
      new Set(
        this.plans.filter((plan) => this.selected.has(plan.id)).map((plan) => plan.original?.id ?? plan.id),
      ),
    ),
  )

  private colorOf(id: string): string {
    return this.colors.get(id) ?? UNTICKED_COLOR
  }

  /** The ticked plans, in the list's order, as the "What's compared" card
   * names them: each as briefly as it can be told apart. A broker goes by its
   * short name while the plan a new customer gets is its only plan ticked
   * ("Leumi"); otherwise a plan goes by its label ("Leumi · Pepper"), as do
   * your own plans. */
  compared: { plan: Plan; name: string }[] = $derived.by(() => {
    const ticked = this.plans.filter((plan) => this.selected.has(plan.id))
    return ticked.map((plan) => {
      const { broker, key } = plan
      const alone = ticked.filter((other) => other.broker === broker).length === 1
      const usual = key.kind === 'listed' && key.plan === broker?.newCustomerPlan
      return { plan, name: broker && alone && usual ? broker.shortName : plan.label }
    })
  })

  editorView = $state<EditorView>(load<EditorView>('editor-view-v2', 'simple'))

  private chosenView = $state<ChartView>(FIRST_VIEW)
  /** The chart shown. The guide offers the growth chart alone; the other
   * views are the workbench's, and the choice comes back with the switch. */
  get chartView(): ChartView {
    return this.expert ? this.chosenView : 'value'
  }
  set chartView(view: ChartView) {
    this.chosenView = view
  }

  /** Plans clicked in the table or the chart; they stay highlighted. */
  pinned = new SvelteSet<string>()
  hovered = $state<string | null>(null)
  highlighted: ReadonlySet<string> = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    new Set([...this.pinned, ...(this.hovered ? [this.hovered] : [])]),
  )

  /** Open in the details dialog. Raw, so its broker and plan stay the same
   * objects as in `brokers` and `plans` (not proxies), to find them there. */
  details = $state.raw<Details | null>(null)

  purchase = $derived(core.purchasePhrase(this.security, this.exchange))

  /** Why a plan has no numbers, in the few words its row says, and what its
   * ? is about; the core's sentence is in the tip. The fee chart's note
   * says it the same way. */
  whyNotWords(whyNot: WhyNot | undefined): { cell: string; about: string } {
    if (whyNot === 'OverTheCeiling') return { cell: t.overTheCeiling, about: t.theCeiling }
    if (whyNot === 'Locked') return { cell: t.stillLocked, about: t.theLock }
    if (whyNot === 'NotCounted') return { cell: t.priceUnclear, about: t.notWorkedOut }
    return { cell: t.notOfferedFor(this.purchase), about: t.notOffered }
  }

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
    const { comparison, waysComparison } = this
    if (comparison && 'results' in comparison) {
      return comparison.results.find((result) => result.plan.id === plan.id)?.track
    }
    if (waysComparison && 'ways' in waysComparison) {
      return waysComparison.ways.flatMap((way) => way.plans).find((each) => planId(each.key) === plan.id)
        ?.track
    }
    return undefined
  }

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

  // ── What the guide says ──
  /** The plans ticked, as the core's keys: the guide's list of places. */
  private tickedKeys: PlanKey[] = $derived(
    this.plans.filter((plan) => this.selected.has(plan.id)).map((plan) => plan.key),
  )

  settings: GuideSettings = $derived({
    indexReturnPercent: this.yearlyReturnPercent,
    inflationPercent: this.inflationPercent,
    boiRatePercent: this.boiRatePercent,
    ilsPerUsd: this.ilsPerUsd,
    ilsPerEur: this.ilsPerEur,
    plans: this.tickedKeys,
    yourPlans: this.yourPlans.map(({ id, plan }) => ({ id, plan })),
    overrides: this.overrides,
    thisYear: THIS_YEAR,
    thisMonth: THIS_MONTH,
  })

  /** What the guide is asked: a new object whenever the answers or the
   * settings change, to match the workers' answers to. */
  guideRequest: GuideRequest = $derived({
    answers: $state.snapshot(this.answers) as AnswersInput,
    settings: this.settings,
  })

  /** Whether the recommendation is worked out on the page's thread. */
  private get onThePage(): boolean {
    const months = this.horizonMonths
    return months === undefined || months <= RECOMMEND_ON_PAGE_UP_TO_MONTHS
  }

  private lastRecommendation: RecommendationData | undefined = undefined

  /** The recommendation as worked out here: always for a short horizon, and
   * for a long one the first time, when there's nothing to show meanwhile. */
  private onPage = $derived.by((): Worked<RecommendationData> | undefined =>
    this.onThePage || !this.lastRecommendation ? run('recommend', this.guideRequest) : undefined,
  )
  /** The worker's answer for a long horizon, and the request it answers. */
  private recommendAnswer = $state.raw<Answered<RecommendationData>>()
  private recommended: Worked<RecommendationData> | undefined = $derived.by(() => {
    const worked = this.onPage ?? this.recommendAnswer
    if (worked?.value) this.lastRecommendation = worked.value
    return worked
  })

  /** What fits money like this, and why. While the answers can't be
   * compared, or the worker is still at it, the last one stays. Missing only
   * before the first one that could be made. */
  recommendation: RecommendationData | undefined = $derived(
    this.recommended?.value ?? this.lastRecommendation,
  )
  /** Whether `recommendation` answers the answers as they are now. */
  recommendationFresh: boolean = $derived(
    this.onPage
      ? this.onPage.value !== undefined
      : this.recommendAnswer?.request === this.guideRequest && this.recommendAnswer.value !== undefined,
  )

  // ── The redesigned answer ──
  /** What the answer is asked: a new object whenever the answers, the
   * choices or the settings change. */
  answerRequest: AnswerRequest = $derived({
    answers: $state.snapshot(this.answers) as AnswersInput,
    choices: $state.snapshot(this.choices) as Choices,
    settings: this.settings,
    horizons: true,
  })
  /** For a short horizon, the answer worked out here at once, without the
   * horizon section (most of its time); the worker's full one follows. */
  private answerOnPage = $derived.by((): Worked<AnswerData> | undefined =>
    this.onThePage ? run('answer', { ...this.answerRequest, horizons: false }) : undefined,
  )
  private answerFromWorker = $state.raw<Worked<AnswerData> & { request: AnswerRequest }>()
  private lastAnswer: AnswerData | undefined = undefined
  /** The answer (REDESIGN.md Part 2). While the answers can't be worked on,
   * or the worker is still at it, the last one stays. */
  answer: AnswerData | undefined = $derived.by(() => {
    const fromWorker =
      this.answerFromWorker?.request === this.answerRequest ? this.answerFromWorker.value : undefined
    const answer = fromWorker ?? this.answerOnPage?.value
    // The page's quick answer has no horizons: keep the last ones meanwhile.
    if (answer && !fromWorker && this.lastAnswer) {
      this.lastAnswer = { ...answer, horizons: this.lastAnswer.horizons }
    } else if (answer) this.lastAnswer = answer
    return this.lastAnswer
  })
  /** Whether `answer` (its horizons included) answers the answers as they are now. */
  answerFresh: boolean = $derived(
    this.answerFromWorker?.request === this.answerRequest && this.answerFromWorker.value !== undefined,
  )
  /** The row "איך עושים את זה" shows. */
  focusedRow: Kind | undefined = $derived(this.tappedRow ?? this.answer?.focus ?? undefined)

  /** Tapping a row of the outcome chart; `undefined` goes back to the answer's. */
  focusRow(kind: Kind | undefined) {
    this.tappedRow = kind
  }

  /** Picks an option of a decision. The facts (the cushion, whether the
   * date can move) go into the answers; the rest into `choices`. The core
   * re-evaluates which dependent decisions appear. */
  choose(decision: DecisionId, pick: Pick) {
    this.tappedRow = undefined
    if ('Cushion' in pick) {
      this.answers.cushion =
        pick.Cushion === 'Has'
          ? { kind: 'has' }
          : {
              kind: 'hasNot',
              monthlyExpenses:
                this.answers.cushion?.kind === 'hasNot' ? this.answers.cushion.monthlyExpenses : null,
            }
    } else if ('Flexibility' in pick) this.answers.flexibility = pick.Flexibility
    else if ('SafeWay' in pick) this.choices.safeWay = pick.SafeWay
    else if ('Wrapper' in pick) this.choices.wrapper = pick.Wrapper
    else if ('Account' in pick) this.choices.account = pick.Account
    else if ('Holdings' in pick) this.choices.holdings = pick.Holdings
    void decision
  }

  /** What the same money does at other horizons, and each kind's money
   * month by month: from the worker, so a little behind the answers. */
  private ladderAnswer = $state.raw<Answered<RungData[]>>()
  private alternativesAnswer = $state.raw<Answered<AlternativesData>>()
  private lastLadder: RungData[] | undefined
  private lastAlternatives: AlternativesData | undefined
  ladder: RungData[] | undefined = $derived.by(() => {
    if (this.ladderAnswer?.value) this.lastLadder = this.ladderAnswer.value
    return this.lastLadder
  })
  ladderFresh: boolean = $derived(
    this.ladderAnswer?.request === this.guideRequest && this.ladderAnswer.value !== undefined,
  )
  alternatives: AlternativesData | undefined = $derived.by(() => {
    if (this.alternativesAnswer?.value) this.lastAlternatives = this.alternativesAnswer.value
    return this.lastAlternatives
  })
  alternativesFresh: boolean = $derived(
    this.alternativesAnswer?.request === this.guideRequest && this.alternativesAnswer.value !== undefined,
  )

  /** The years the comparison runs, from the horizon, or from what the
   * guide worked out when the horizon isn't known. */
  years: number = $derived(
    Math.min(50, Math.max(1, Math.ceil((this.horizonMonths ?? this.recommendation?.months ?? 60) / 12))),
  )

  /** The questions the answers still need. */
  followUps: FollowUp[] = $derived.by(() => {
    if (this.recommendation) return this.recommendation.followUps
    try {
      return core.followUps($state.snapshot(this.answers) as AnswersInput)
    } catch {
      return []
    }
  })

  /** The follow-up questions as cards, in the core's words. */
  followCards: FollowCardData[] = $derived(this.followUps.map((each) => core.followCard(each)))

  /** What the workbench looks at: what the user chose there, or what the
   * recommendation is about. */
  thing: Thing = $derived.by(() => {
    if (this.chosenThing) return this.chosenThing
    const kind = this.recommendation?.kind.kind
    if (kind === undefined || (typeof kind === 'object' && 'Way' in kind)) return 'sp500'
    return typeof kind === 'object' ? 'bonds' : 'short'
  })

  chooseThing(thing: Thing) {
    this.chosenThing = thing
    if (thing === 'sp500') this.overrides = { arena: 'Index', kind: undefined }
    else if (thing === 'bonds')
      this.overrides = { arena: 'Safe', kind: { GovernmentBond: { linked: false } } }
    else if (thing === 'short') this.overrides = { arena: 'Safe', kind: 'MoneyMarketFund' }
  }

  /** "לשנות" / "continue anyway": the guide goes on with this kind. */
  chooseKind(kind: Kind) {
    this.overrides = { arena: arenaOf(kind), kind }
    this.chosenThing = undefined
  }
  chooseArena(arena: Arena) {
    this.overrides = { arena, kind: undefined }
    this.chosenThing = undefined
  }
  /** Back to what the guide recommends. */
  clearOverrides() {
    this.overrides = NO_OVERRIDES
    this.chosenThing = undefined
  }

  toggleWay(way: Way) {
    const key = wayKey(way)
    const keys = this.selectedWays.map(wayKey)
    const next = keys.includes(key) ? keys.filter((each) => each !== key) : [...keys, key]
    // None ticked would mean every way to the core: keep the last one.
    if (next.length > 0) this.chosenWays = next
  }

  /** A chip: fills the answers with the situation's. */
  setSituation(id: string) {
    const situation = this.situations.find((each) => each.id === id)
    if (!situation) return
    this.situationId = id
    // A chip is the money's facts: the visitor's own bank stays.
    const bank = this.answers.bank
    this.answers = { ...($state.snapshot(situation.answers) as AnswersInput), bank }
    this.choices = {}
    this.tappedRow = undefined
    this.clearOverrides()
    if (situation.workbench) this.expert = true
    if (situation.opensDialog) this.portfolioDialogOpen = true
  }

  /** "אני יודע / יודעת מה אני רוצה לקנות ולהשוות": the switch on, the workbench's. */
  knowsWhatToBuy() {
    this.expert = true
  }

  /** The visitor's bank: what the places are compared from. For the
   * existing-portfolio situation, it is also the bank the portfolio is at. */
  setBank(bank: Bank | undefined) {
    this.answers.bank = bank
    if (this.situationId === 'existing' || typeof this.answers.situation === 'object')
      this.answers.situation = { ExistingPortfolio: { bank } }
  }

  // ── Package I: the bank's branch plan, ticked ──
  /** Ticks the plan the recommendation marks as the saver's own bank's
   * branch plan, and unticks the other banks' branch plans that an earlier
   * choice of bank ticked. Called by the answer while the situation is the
   * existing portfolio; changes nothing when the recommendation hasn't a
   * plan of theirs yet. */
  tickYourBranch() {
    const yours = yoursBranch(this.plans, this.recommendation)
    if (!yours) return
    for (const plan of branchPlans(this.plans)) {
      if (plan.id !== yours.id && this.selected.has(plan.id)) this.selected.delete(plan.id)
    }
    if (!this.selected.has(yours.id)) this.selected.add(yours.id)
  }

  /** The years field. For a retirement the ages stay and the years are the
   * gap between them, so typing years moves the age of retiring; an empty
   * field changes nothing. Otherwise an empty field is a date not known yet. */
  setHorizonMonths(months: number | null) {
    const { horizon } = this.answers
    if (typeof horizon === 'object' && 'Retirement' in horizon) {
      if (months === null) return
      const { age } = horizon.Retirement
      this.answers.horizon = { Retirement: { age, retire_at: age + Math.round(months / 12) } }
      return
    }
    this.answers.horizon = months === null ? 'Unknown' : { Months: months }
  }
  setHorizonRetirement(age: number, retireAt: number) {
    this.answers.horizon = { Retirement: { age, retire_at: retireAt } }
  }

  // ── Sheets and dialogs ──
  /** The learn cards open in the sheet, the first at the bottom: "back" goes
   * to the one before. */
  cardStack = $state.raw<Term[]>([])
  get cardTerm(): Term | undefined {
    return this.cardStack.at(-1)
  }
  openCard(term: Term) {
    this.cardStack = [...this.cardStack, term]
  }
  cardBack() {
    this.cardStack = this.cardStack.slice(0, -1)
  }
  closeCard() {
    this.cardStack = []
  }
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a cache, never watched
  private readonly cards = new Map<Term, LearnCard>()
  cardFor(term: Term): LearnCard {
    let card = this.cards.get(term)
    if (!card) this.cards.set(term, (card = core.learnCard(term)))
    return card
  }
  glossaryOpen = $state(false)
  private glossaryCards: LearnCard[] | undefined
  /** Every card, for the index in the header. */
  get glossary(): LearnCard[] {
    return (this.glossaryCards ??= core.glossary())
  }
  /** The list of plans to compare ("מה משווים"). */
  planSheetOpen = $state(false)
  /** The dialog the existing-portfolio situation opens. */
  portfolioDialogOpen = $state(false)

  openAbout(section?: number) {
    this.details = { kind: 'about', section }
  }

  // ── The workbench's comparisons ──
  sellAtEnd: boolean = $derived(this.atEnd === 'sell')
  inTodaysMoney: boolean = $derived(this.todaysMoney && (this.inflationPercent ?? 0) !== 0)
  /** The age from which a ticked plan can be taken as a pension, the
   * youngest if they differ; none if no ticked plan pays one. How the money
   * is taken out is asked only while there is one, and while it's sold. */
  pensionFromAge: number | undefined = $derived.by(() => {
    if (!this.sellAtEnd) return undefined
    const ages = this.plans
      .filter((plan) => this.selected.has(plan.id))
      .flatMap((plan) => plan.info.pensionFromAge ?? [])
    return ages.length > 0 ? Math.min(...ages) : undefined
  })
  asPension: boolean = $derived(this.pensionFromAge !== undefined && this.wayOut === 'pension')

  inputs: Inputs = $derived(this.inputsWith(this.answers.now, this.answers.monthly))

  /** The inputs with the deposits given: the comparison's own, or the
   * sweep's, which leaves out the deposit it varies. */
  private inputsWith(firstDeposit: number | null, monthlyDeposit: number | null): Inputs {
    return {
      security: this.security,
      exchange: this.exchange,
      product: this.chosenProduct,
      firstDeposit,
      monthlyDeposit,
      yearlyReturnPercent: this.yearlyReturnPercent,
      years: this.years,
      buyEveryMonths: this.buyEveryMonths,
      sharePrice: this.sharePrice,
      depositGrowthPercent: this.depositGrowthPercent,
      inflationPercent: this.inflationPercent,
      inTodaysMoney: this.inTodaysMoney,
      asPension: this.asPension,
      age: this.age,
      sellAtEnd: this.sellAtEnd,
      ilsPerUsd: this.ilsPerUsd,
      ilsPerEur: this.ilsPerEur,
      plans: this.tickedKeys,
      yourPlans: this.yourPlans.map(({ id, plan }) => ({ id, plan })),
    }
  }

  /** Which deposit the chart by deposit varies: the monthly one if there is
   * one, else the one-time deposit. */
  swept: Swept = $derived((this.answers.monthly ?? 0) > 0 ? 'Monthly' : 'OneTime')

  /** What the sweep is asked for: the inputs without the deposit it varies,
   * so typing that deposit asks for nothing new (only the chart's marker
   * moves). A new object whenever the inputs change, to match answers to.
   * Missing when the workbench isn't comparing plans on one exchange. */
  sweepRequest: SweepRequest | undefined = $derived.by(() => {
    if (this.thing !== 'other') return undefined
    const swept = this.swept
    return {
      swept,
      inputs: this.inputsWith(
        swept === 'OneTime' ? 0 : this.answers.now,
        swept === 'Monthly' ? 0 : this.answers.monthly,
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
    if (
      !answer?.sweep ||
      answer.request !== this.sweepRequest ||
      !this.comparison ||
      'error' in this.comparison
    )
      return undefined
    try {
      return core.around({
        sweep: answer.sweep,
        deposit: (this.swept === 'Monthly' ? this.answers.monthly : this.answers.now) ?? 0,
        // A plan the first deposit can't open isn't in the running, as in
        // the sweep.
        costs: this.comparison.results.map(({ plan, outcome, fits }) => ({
          key: plan.key,
          cost: fits ? (outcome?.yearlyCostPercent ?? null) : null,
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

  /** The plans' outcomes as table rows, best first, with the places the core
   * gives them (equal results share one) for the charts to show. */
  resultsOf(plans: PlanOutcomeData[]): Result[] {
    return plans.map(
      ({
        key,
        outcome,
        track,
        warning,
        fits,
        rank,
        mayCostMore,
        note,
        whoCanJoin,
        pensionNote,
        spread,
        taxNote,
        notOffered,
        whyNot,
      }) => ({
        plan: this.plansById.get(planId(key))!,
        outcome,
        rank,
        track,
        warning,
        fits,
        mayCostMore,
        note,
        whoCanJoin,
        pensionNote,
        spread,
        taxNote,
        notOffered,
        whyNot,
      }),
    )
  }

  /** The plans compared on one exchange (the workbench's "another index or
   * stock"), or why they couldn't be. Missing for the other things. */
  private latest = $derived.by((): Comparison | undefined => {
    if (this.thing !== 'other') return undefined
    let data: ComparisonData
    try {
      data = core.compare(this.inputs)
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) }
    }
    return {
      deposited: data.deposited,
      noFees: data.noFees,
      largestTrade: data.largestTrade,
      usEstateTax: data.usEstateTax,
      results: this.resultsOf(data.plans),
    }
  })

  private lastWorked: Comparison | undefined

  /** What the page shows: the comparison, or while the inputs can't be
   * compared (a field being retyped), the last one that could, faded under
   * why not, so the page keeps its shape. */
  comparison: Comparison | undefined = $derived.by(() => {
    if (this.latest && 'results' in this.latest) this.lastWorked = this.latest
    return this.lastWorked ?? this.latest
  })

  /** The ways side by side, for the S&P 500 (switch on), or why not. */
  private latestWays = $derived.by((): WaysView | undefined => {
    if (!this.expert || this.thing !== 'sp500') return undefined
    try {
      return core.compareWays({ inputs: this.inputs, ways: this.selectedWays })
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) }
    }
  })
  private lastWays: WaysView | undefined
  waysComparison: WaysView | undefined = $derived.by(() => {
    if (this.latestWays && 'ways' in this.latestWays) this.lastWays = this.latestWays
    return this.lastWays ?? this.latestWays
  })

  /** Why the answers or the inputs can't be compared, in the core's words;
   * the last results stay, faded, under it. */
  inputError: string | undefined = $derived.by(() => {
    if (unreadable.size > 0) return t.notANumber
    if (this.recommended?.error) return this.recommended.error
    if (this.latest && 'error' in this.latest) return this.latest.error
    if (this.latestWays && 'error' in this.latestWays) return this.latestWays.error
    return undefined
  })
  stale: boolean = $derived(this.inputError !== undefined)

  /** The table's first plan, which the first deposit can open: the core
   * ranks the ones it can't after the rest. Only if none can is one of them
   * the best, with its warning. */
  best: Result | undefined = $derived(
    this.comparison && 'results' in this.comparison
      ? this.comparison.results.find((result) => result.outcome)
      : undefined,
  )

  /** The plans with numbers in the table, by id, each with its row number
   * (none for a plan the first deposit can't open), for the chart by
   * deposit: its sweep answers a moment after the table, which it may not
   * match. */
  private ranks: ReadonlyMap<string, { rank: number | undefined }> = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    new Map(
      this.comparison && 'results' in this.comparison
        ? this.comparison.results.flatMap(({ plan, outcome, rank }) =>
            outcome ? [[plan.id, { rank }] as const] : [],
          )
        : [],
    ),
  )

  rowOf(id: string): { rank: number | undefined } | undefined {
    return this.ranks.get(id)
  }

  /** What the user buys, for the fees and caveats that matter to them, with
   * the comparison's biggest order: a caveat about large orders shows only
   * when the user's orders reach it. */
  buying: Purchase = $derived({
    security: this.security,
    exchange: this.exchange,
    largestTrade: this.comparison && 'results' in this.comparison ? this.comparison.largestTrade : null,
    ilsPerUsd: this.ilsPerUsd,
    ilsPerEur: this.ilsPerEur,
  })

  /** A link to this page: the answers, the switch, the workbench's inputs
   * when they matter, and the ticked plans, your own included, in the page's
   * address. */
  shareLink(): string {
    return `${location.origin}${location.pathname}#${this.address()}`
  }

  /** The hash (without its #) the page's address keeps while the saver
   * goes: the same as the Share button's link. */
  address(): string {
    return encode(this.sharedState())
  }

  private sharedState(): Shared {
    const { answers } = this
    const ticked = this.plans.filter((plan) => this.selected.has(plan.id))
    const shared: Shared = {
      situation: this.situationId,
      now: answers.now ?? undefined,
      monthly: answers.monthly ?? undefined,
      horizon: $state.snapshot(answers.horizon) as Horizon,
      flexibility: answers.flexibility,
      cushion: $state.snapshot(answers.cushion),
      needsAnyDay: answers.needsAnyDay || undefined,
      expert: this.expert || undefined,
      plans: ticked.filter((plan) => plan.key.kind === 'listed').map((plan) => plan.englishLabel),
      yours: ticked.flatMap((plan) => (plan.yours ? [plan.yours] : [])),
    }
    shared.bank =
      answers.bank ??
      (typeof answers.situation === 'object' ? answers.situation.ExistingPortfolio.bank : undefined) ??
      undefined
    const choices = $state.snapshot(this.choices) as Choices
    if (Object.values(choices).some((value) => value !== undefined)) shared.choices = choices
    // The three assumptions travel whenever they aren't the usual, in the
    // guide too, where the answer's line lets them be changed.
    if (this.yearlyReturnPercent !== USUAL_RETURN_PERCENT)
      shared.yearlyReturnPercent = this.yearlyReturnPercent ?? undefined
    if (this.boiRatePercent !== this.usualBoiRate) shared.boiRatePercent = this.boiRatePercent ?? undefined
    if (this.inflationPercent !== this.usualInflation)
      shared.inflationPercent = this.inflationPercent ?? undefined
    const { overrides } = this
    if (overrides.arena) shared.overrideArena = overrides.arena
    if (overrides.kind) shared.overrideKind = kindKey(overrides.kind)
    // The workbench's inputs, when the user can see them or chose in it.
    if (this.expert || this.overridden) {
      shared.thing = this.chosenThing
      if (this.chosenWays) shared.ways = this.chosenWays
      shared.security = this.security
      shared.exchange = this.exchange
      shared.yearlyReturnPercent = this.yearlyReturnPercent ?? undefined
      shared.sharePrice = this.sharePrice ?? undefined
      if (this.boiRatePercent !== this.usualBoiRate) shared.boiRatePercent = this.boiRatePercent ?? undefined
      if (this.inflationPercent !== this.usualInflation)
        shared.inflationPercent = this.inflationPercent ?? undefined
      if (this.depositGrowthPercent) shared.depositGrowthPercent = this.depositGrowthPercent
      if (this.buyEveryMonths !== 1) shared.buyEveryMonths = this.buyEveryMonths
      if (!this.todaysMoney) shared.todaysMoney = false
      if (!this.sellAtEnd) shared.sellAtEnd = false
    }
    if (this.asPension) {
      shared.asPension = true
      shared.age = this.age ?? undefined
    }
    return shared
  }

  /** Puts the page back as an address says (Back, Forward): the defaults,
   * then what the address names. An address that names nothing is the
   * first visit's, with its example. */
  restore(shared: Shared) {
    this.setSituation(shared.situation ?? FIRST_SITUATION)
    this.portfolioDialogOpen = false
    this.expert = shared.expert === true
    this.chosenThing = undefined
    this.chosenWays = undefined
    this.security = 'Etf'
    this.exchange = 'Usa'
    this.sharePrice = 500
    this.yearlyReturnPercent = USUAL_RETURN_PERCENT
    this.inflationPercent = this.usualInflation
    this.boiRatePercent = this.usualBoiRate
    this.depositGrowthPercent = 0
    this.buyEveryMonths = 1
    this.todaysMoney = true
    this.atEnd = 'sell'
    this.wayOut = 'atOnce'
    this.applyShared(shared)
    this.exampleShown = Object.keys(shared).length === 0
    this.exampleKey = this.address()
  }

  /** Takes what a link says, over the defaults. Its plans, listed and your
   * own, replace what's ticked; your plans from it join yours, a plan with
   * the same id (from your own other device) taking the newer version. */
  private applyShared(shared: Shared) {
    if (shared.bank !== undefined) this.setBank(shared.bank)
    if (shared.now !== undefined) this.answers.now = shared.now
    if (shared.monthly !== undefined) this.answers.monthly = shared.monthly
    if (shared.horizon !== undefined) this.answers.horizon = shared.horizon
    if (shared.flexibility !== undefined) this.answers.flexibility = shared.flexibility
    if (shared.cushion !== undefined) this.answers.cushion = shared.cushion
    if (shared.needsAnyDay) this.answers.needsAnyDay = true
    if (shared.choices) this.choices = { ...shared.choices }
    if (shared.expert) this.expert = true
    this.chosenThing = shared.thing
    if (shared.ways) this.chosenWays = shared.ways
    if (shared.security !== undefined) this.security = shared.security
    if (shared.exchange !== undefined) this.exchange = shared.exchange
    if (shared.yearlyReturnPercent !== undefined) this.yearlyReturnPercent = shared.yearlyReturnPercent
    if (shared.inflationPercent !== undefined) this.inflationPercent = shared.inflationPercent
    if (shared.boiRatePercent !== undefined) this.boiRatePercent = shared.boiRatePercent
    if (shared.depositGrowthPercent !== undefined) this.depositGrowthPercent = shared.depositGrowthPercent
    if (shared.buyEveryMonths !== undefined) this.buyEveryMonths = shared.buyEveryMonths
    if (shared.sharePrice !== undefined) this.sharePrice = shared.sharePrice
    if (shared.todaysMoney === false) this.todaysMoney = false
    if (shared.sellAtEnd === false) this.atEnd = 'hold'
    if (shared.asPension) this.wayOut = 'pension'
    if (shared.age !== undefined) this.age = shared.age
    if (shared.overrideArena !== undefined || shared.overrideKind !== undefined) {
      this.overrides = {
        arena: shared.overrideArena,
        kind: shared.overrideKind === undefined ? undefined : kindByKey(shared.overrideKind),
      }
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
      for (const label of labels) {
        const average = this.averageFor.get(label)
        if (average) this.selected.add(average)
      }
    }
  }

  /** `shared`: what the page's address says, if it was opened from a link. */
  constructor(shared: Shared = {}) {
    // The guide opens on its first situation (or the link's); a link's own
    // answers come over it.
    this.setSituation(shared.situation ?? FIRST_SITUATION)
    this.portfolioDialogOpen = false
    this.applyShared(shared)
    this.exampleShown = Object.keys(shared).length === 0
    this.exampleKey = this.address()
    this.loadRates()
    $effect(() => {
      if (this.exampleShown && this.address() !== this.exampleKey) this.exampleShown = false
    })
    $effect(() => save('your-plans-v1', this.yourPlans))
    $effect(() => save('editor-view-v2', this.editorView))
    $effect(() => save('expert-v2', this.expert))
    // The heavy work runs off the page's thread; its answers come a moment later.
    $effect(() => {
      const request = this.answerRequest
      ask('answer', request, (worked) => (this.answerFromWorker = { request, ...worked }))
    })
    $effect(() => {
      if (this.onPage) return
      const request = this.guideRequest
      ask('recommend', request, (worked) => (this.recommendAnswer = { request, ...worked }))
    })
    $effect(() => {
      const request = this.guideRequest
      ask('ladder', request, (worked) => (this.ladderAnswer = { request, ...worked }))
    })
    $effect(() => {
      const request = this.guideRequest
      ask('alternatives', request, (worked) => (this.alternativesAnswer = { request, ...worked }))
    })
    $effect(() => {
      const request = this.sweepRequest
      if (request) askSweep(request, (sweep) => (this.sweepAnswer = { request, sweep }))
    })
  }

  private async loadRates() {
    try {
      const { ilsPerUsd, ilsPerEur, date } = await todaysRates()
      this.ilsPerUsd = ilsPerUsd
      this.ilsPerEur = ilsPerEur
      this.ratesStatus = { kind: 'downloaded', date }
    } catch (error) {
      this.ratesStatus = { kind: 'failed', error: error instanceof Error ? error.message : String(error) }
    }
  }
}
