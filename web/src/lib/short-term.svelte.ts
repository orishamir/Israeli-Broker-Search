// The short-term calculator's state: what the saver puts in, for how long,
// the Bank of Israel's rate they expect, and where to compare. The
// comparison is derived: the Rust core works it out again whenever an input
// changes.

import { SvelteSet } from 'svelte/reactivity'
import * as core from './core/core'
import type {
  About,
  KindInfo,
  PlaceInfo,
  PlaceKey,
  ShortTermComparisonData,
  ShortTermExampleData,
  ShortTermInputs,
  ShortTermOutcomeData,
} from './core/core'
import { handOut, UNTICKED_COLOR, withColor } from './colors'
import { percent } from './format'
import type { SharedShort } from './link'
import { load, save } from './saved'
import { t } from './text'

/** One of the saver's own deposits: a rate a bank offered them. */
export interface YourDeposit {
  id: string
  name: string
  /** Percent a year; null while its field is empty. */
  ratePercent: number | null
}

/** A place to keep the money, listed or the saver's own, with its color:
 * handed out when it's ticked, as a plan's is. */
export interface Place {
  key: PlaceKey
  /** For sets and keyed lists. */
  id: string
  info: PlaceInfo
  /** Its kind's name, under its own in the table: "Money market fund". */
  kindName: string
  /** The flag its kind raises for all its places, said once: "August's
   * rates, before a cut". Its own flag is `info.mayCostMore`. */
  kindFlag: string | undefined
  /** Where it's named alone, in the chips and the charts: "Cheapest fund", "Leumi". */
  label: string
  /** How links name a listed place: "Fixed-rate deposit · Bank Leumi". */
  englishLabel: string
  color: string
  /** One of the saver's own. */
  yours?: YourDeposit
}

/** One place's line in the results, best first. */
export interface PlaceResult {
  place: Place
  /** Missing where the money can't be kept, as the inputs are. */
  outcome: ShortTermOutcomeData | undefined
  /** Its row's number in the table, 1 for the best; missing with the outcome. */
  rank: number | undefined
  /** Why there's no outcome, in a few words ("Takes one sum"), and in full. */
  notOffered: string | undefined
  notOfferedReason: string | undefined
}

/** The places compared, best first, or why they couldn't be. */
export type ShortTermComparison =
  { deposited: number; atTheRate: number[]; term: string; results: PlaceResult[] } | { error: string }

export type ShortTermView = 'split' | 'value'

export const placeId = (key: PlaceKey): string =>
  key.kind === 'listed' ? `${key.group}:${key.place}` : `yours:${key.id}`

/** The shape of one of your deposits, from storage or a link. */
export function looksLikeYourDeposit(value: unknown): value is YourDeposit {
  if (typeof value !== 'object' || value === null) return false
  const { id, name, ratePercent } = value as Record<string, unknown>
  return (
    typeof id === 'string' &&
    typeof name === 'string' &&
    (ratePercent === null || (typeof ratePercent === 'number' && Number.isFinite(ratePercent)))
  )
}

function loadYourDeposits(): YourDeposit[] {
  const saved = load<unknown>('your-deposits-v1', [])
  return Array.isArray(saved) ? saved.filter(looksLikeYourDeposit) : []
}

export class ShortTermState {
  /** Whether More options is on, which the two calculators share: set by
   * the constructor. */
  private readonly expert: () => boolean = () => false
  readonly kinds: KindInfo[] = core.shortTermKinds()
  /** Deposits: the kind whose places pay a rate by term. */
  private readonly depositsKind = this.kinds.find((kind) => kind.places.some(({ rates }) => rates.length > 0))
  /** Your own deposit's description, tax and lock. */
  private readonly yoursInfo: PlaceInfo = core.yourDepositInfo()
  /** How the numbers are made, what isn't counted, and the sources. */
  readonly about: About = core.aboutShortTerm()
  /** Ready-made patterns, each setting the amounts and the months. */
  readonly examples: ShortTermExampleData[] = core.shortTermExamples()
  /** The Bank of Israel's rate when the numbers were checked, in percent. */
  readonly todaysRate: number = core.todaysRatePercent()
  /** The longest the money can be kept, in months. */
  readonly longest: number = core.shortTermLongest()
  readonly usualInflation: number = core.usualInflationPercent()

  readonly listedPlaces: Place[] = this.kinds.flatMap((kind, group) =>
    kind.places.map((info, place) => {
      const key = { kind: 'listed' as const, group, place }
      const id = placeId(key)
      return withColor(
        {
          key,
          id,
          info,
          kindName: kind.name,
          kindFlag: kind.mayCostMore,
          label: info.shortName,
          englishLabel: `${kind.englishName} · ${info.englishName}`,
        },
        () => this.colorOf(id),
      )
    }),
  )

  /** Your own deposits, oldest first. Raw: they're replaced, not changed. */
  yourDeposits = $state.raw<YourDeposit[]>(loadYourDeposits())

  /** Listed places, then your own. */
  places: Place[] = $derived([
    ...this.listedPlaces,
    ...this.yourDeposits.map((yours) => {
      const key = { kind: 'yours' as const, id: yours.id }
      const id = placeId(key)
      return withColor(
        {
          key,
          id,
          info: { ...this.yoursInfo, name: yours.name, shortName: yours.name },
          kindName: this.depositsKind?.name ?? '',
          kindFlag: undefined,
          label: yours.name,
          englishLabel: yours.name,
          yours,
        },
        () => this.colorOf(id),
      )
    }),
  ])

  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
  placesById: ReadonlyMap<string, Place> = $derived(new Map(this.places.map((place) => [place.id, place])))

  // Numbers are null while their field is empty.
  firstDeposit = $state<number | null>(100_000)
  monthlyDeposit = $state<number | null>(0)
  months = $state(12)
  ratePercent = $state<number | null>(this.todaysRate)
  inflationPercent = $state<number | null>(this.usualInflation)

  /** Ticked: at first, the places the core ticks, and all of your own. */
  selected = new SvelteSet<string>(
    this.places.filter((place) => place.yours || place.info.comparedAtFirst).map((place) => place.id),
  )
  /** The colors handed out, by place id. Plain: only `colors` reads and changes it. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- remembered between runs of `colors`
  private readonly handedOut = new Map<string, string>()
  private colors: ReadonlyMap<string, string> = $derived(
    handOut(
      this.handedOut,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
      new Set(this.places.filter((place) => this.selected.has(place.id)).map((place) => place.id)),
    ),
  )
  private colorOf(id: string): string {
    return this.colors.get(id) ?? UNTICKED_COLOR
  }

  /** The ticked places, in the list's order. */
  compared: Place[] = $derived(this.places.filter((place) => this.selected.has(place.id)))

  chartView = $state<ShortTermView>('split')
  /** Places clicked in the table or the chart; they stay highlighted. */
  pinned = new SvelteSet<string>()
  /** The place under the mouse, in the table or the chart. */
  hovered = $state<string | null>(null)
  /** Places to make stand out: the pinned ones and the hovered one. */
  highlighted: ReadonlySet<string> = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- rebuilt, never changed
    new Set([...this.pinned, ...(this.hovered ? [this.hovered] : [])]),
  )

  /** Which of a deposit's rates the months fall in. */
  term: number | undefined = $derived(core.termFor(this.months))

  /** What a place pays for these months, in a few words: "3.87% interest",
   * "0.17% fee". */
  paysFor(place: Place): string {
    if (place.yours)
      return t.rateOf(place.yours.ratePercent === null ? '?' : percent(place.yours.ratePercent))
    const { feePercent, rates } = place.info
    if (feePercent !== undefined && feePercent !== null) return t.feeOf(percent(feePercent))
    const rate = this.term === undefined ? undefined : rates[this.term]?.rate
    return rate === undefined || rate === null ? t.noRate : t.rateOf(percent(rate))
  }

  /** Everything the core compares the places by. More options off, prices
   * rise as usual, whatever the inflation field says. */
  inputs: ShortTermInputs = $derived({
    firstDeposit: this.firstDeposit,
    monthlyDeposit: this.monthlyDeposit,
    months: this.months,
    ratePercent: this.ratePercent,
    inflationPercent: this.expert() ? this.inflationPercent : this.usualInflation,
    places: this.compared.map((place) => place.key),
    yourDeposits: this.yourDeposits.map(({ id, ratePercent }) => ({ id, ratePercent })),
  })

  /** The ticked places, best first, or why they couldn't be compared. */
  private latest: ShortTermComparison = $derived.by(() => {
    let data: ShortTermComparisonData
    try {
      data = core.compareShortTerm(this.inputs)
    } catch (error) {
      return { error: error instanceof Error ? error.message : String(error) }
    }
    return {
      deposited: data.deposited,
      atTheRate: data.atTheRate,
      term: data.term,
      results: data.places.map(({ key, outcome, notOffered, notOfferedReason }, index) => ({
        place: this.placesById.get(placeId(key))!,
        outcome,
        rank: outcome ? index + 1 : undefined,
        notOffered,
        notOfferedReason,
      })),
    }
  })

  /** The last comparison that worked. */
  private lastWorked: ShortTermComparison | undefined

  /** What the page shows: the comparison, or while the inputs can't be
   * compared, the last one that could, faded under why not (see AppState). */
  comparison: ShortTermComparison = $derived.by(() => {
    if ('results' in this.latest) this.lastWorked = this.latest
    return this.lastWorked ?? this.latest
  })

  /** Why the inputs can't be compared, while they can't. */
  inputError: string | undefined = $derived('error' in this.latest ? this.latest.error : undefined)

  /** The table's first place. */
  best: PlaceResult | undefined = $derived(
    'error' in this.comparison ? undefined : this.comparison.results.find((result) => result.outcome),
  )

  /** Ticks or unticks places. Unticked places are unpinned too. */
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

  /** Clicking a pinned place unpins it. */
  togglePin(id: string) {
    if (!this.pinned.delete(id)) this.pinned.add(id)
  }

  /** Fills the amounts and the months from an example. The rate expected stays. */
  applyExample(example: ShortTermExampleData) {
    this.firstDeposit = example.firstDeposit
    this.monthlyDeposit = example.monthlyDeposit
    this.months = example.months
  }

  /** Adds a deposit of your own, ticked, at today's rate until you type
   * yours: "Your deposit", or "Your deposit 2" if that's taken. Returns it. */
  addYourDeposit(): YourDeposit {
    const names = this.yourDeposits.map(({ name }) => name)
    let name = t.yourDepositName
    for (let count = 2; names.includes(name); count++) name = t.yourDepositNumbered(count)
    const yours = { id: crypto.randomUUID(), name, ratePercent: this.todaysRate }
    this.yourDeposits = [...this.yourDeposits, yours]
    this.selected.add(placeId({ kind: 'yours', id: yours.id }))
    return yours
  }

  updateYourDeposit(yours: YourDeposit) {
    this.yourDeposits = this.yourDeposits.map((each) => (each.id === yours.id ? yours : each))
  }

  deleteYourDeposit(id: string) {
    const key = placeId({ kind: 'yours', id })
    this.setSelected([key], false)
    if (this.hovered === key) this.hovered = null
    this.yourDeposits = this.yourDeposits.filter((yours) => yours.id !== id)
  }

  /** What a link to this comparison carries: the inputs and the ticked
   * places, your own ticked deposits whole. */
  shared(): SharedShort {
    const shared: SharedShort = {
      firstDeposit: this.firstDeposit ?? undefined,
      monthlyDeposit: this.monthlyDeposit ?? undefined,
      months: this.months,
      ratePercent: this.ratePercent ?? undefined,
      places: this.compared.filter((place) => !place.yours).map((place) => place.englishLabel),
      yours: this.compared.flatMap((place) => (place.yours ? [place.yours] : [])),
    }
    // Inflation only when it'd change something.
    if (this.expert() && this.inflationPercent !== null && this.inflationPercent !== this.usualInflation) {
      shared.inflationPercent = this.inflationPercent
    }
    return shared
  }

  /** Takes what a link says, over the defaults. Its places replace what's
   * ticked; your deposits from it join yours, one with the same id taking
   * the newer version. */
  private applyShared(shared: SharedShort) {
    if (shared.firstDeposit !== undefined) this.firstDeposit = shared.firstDeposit
    if (shared.monthlyDeposit !== undefined) this.monthlyDeposit = shared.monthlyDeposit
    if (shared.months !== undefined) this.months = shared.months
    if (shared.ratePercent !== undefined) this.ratePercent = shared.ratePercent
    if (shared.inflationPercent !== undefined) this.inflationPercent = shared.inflationPercent
    const yours = shared.yours ?? []
    if (yours.length > 0) {
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const fromLink = new Map(yours.map((deposit) => [deposit.id, deposit]))
      this.yourDeposits = [
        ...this.yourDeposits.map((mine) => fromLink.get(mine.id) ?? mine),
        ...yours.filter((deposit) => !this.yourDeposits.some((mine) => mine.id === deposit.id)),
      ]
    }
    if (shared.places !== undefined || yours.length > 0) {
      this.selected.clear()
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const labels = new Set(shared.places ?? [])
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- built once, never changed
      const ids = new Set(yours.map(({ id }) => placeId({ kind: 'yours', id })))
      for (const place of this.places) {
        if ((!place.yours && labels.has(place.englishLabel)) || ids.has(place.id)) this.selected.add(place.id)
      }
    }
  }

  /** `shared`: what a link says, if the page was opened from one. */
  constructor(expert: () => boolean, shared: SharedShort = {}) {
    this.expert = expert
    this.applyShared(shared)
    $effect(() => save('your-deposits-v1', this.yourDeposits))
  }
}
