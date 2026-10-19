// The rows of "where it's cheapest to buy it": one list for both arenas.
// A row is a plan, a way (with the plans that can hold it), or a place of
// the safe arena, and its amounts go under the columns the core names
// (`columnsFor`). Pure, so the rows can be tested without a browser; the
// component draws them.

import type { Result } from './app.svelte'
import { bestPlanOf } from './chart-entries'
import type { Arena, ColumnId, OutcomeData, PlanKey, RankedData, Way, WayData, WhyNot } from './core/core'
import { placeCost, shekels, yearlyCost } from './format'
import { wayKey } from './kinds'
import { t } from './text'

/** A line under a row's name. `warning`: a number that may be too low, or a
 * plan the inputs don't fit; `tax` and `pension` are as readable as the name. */
export interface Note {
  kind: 'warning' | 'note' | 'tax' | 'pension'
  text: string
}

export interface Row {
  /** What the table row, the charts' lines and the pins share. */
  id: string
  /** Unique in the list, which a plan listed under several ways isn't. */
  key: string
  name: string
  /** Who it's bought through: the plan's broker. */
  subtitle: string | undefined
  /** Its own color, beside its name only; none where no chart draws it. */
  color: string | undefined
  /** Your own plans get a hollow dot, like their dotted lines. */
  yours: boolean
  rank: number | undefined
  /** Missing where it has no numbers. */
  cells: Partial<Record<ColumnId, string>> | undefined
  /** Where it has none: why, in the few words the row says, and for its "?". */
  absent: { cell: string; about: string; reason: string | undefined } | undefined
  notes: Note[]
  /** Hovering and pinning it is shared with the charts. */
  linked: boolean
  /** Its fees cell links to the fee breakdown. */
  feesLink: boolean
  /** The first plan the first deposit can't open: a line above it says why. */
  firstUnfit: boolean
  /** A way's plans, under it when it's opened. */
  more: Row[] | undefined
}

export type WhyNotWords = (whyNot: WhyNot | undefined) => { cell: string; about: string }

const cellsOf = (outcome: OutcomeData): Partial<Record<ColumnId, string>> => ({
  LeftAfterTax: shekels(outcome.afterTax),
  CostAYear: yearlyCost(outcome.yearlyCostPercent),
  Fees: shekels(outcome.fees.total),
  Tax: outcome.tax === 0 ? t.noTax : shekels(outcome.tax),
})

/** The columns some row has an amount for, in the core's order: the safe
 * arena's rows have no fees or tax to show, whatever the arena's columns. */
export function usedColumns<C extends { id: ColumnId }>(columns: C[], rows: Row[]): C[] {
  const all = rows.flatMap((row) => [row, ...(row.more ?? [])])
  return columns.filter(({ id }) => all.some((row) => row.cells?.[id] !== undefined))
}

/** The plans the guide recommends: what's left and what it costs a year. */
export function rankedRows(plans: RankedData[], arena: Arena, todaysMoney: boolean): Row[] {
  const cost = arena === 'Index' ? yearlyCost : placeCost
  const anyFits = plans.some(({ fits }) => fits)
  const firstUnfit = plans.findIndex(({ fits }) => !fits)
  const money = (amount: { today: number; nominal: number }) => (todaysMoney ? amount.today : amount.nominal)
  return plans.map((plan, index) => ({
    id: `${index}:${plan.name}`,
    key: `${index}:${plan.name}`,
    name: plan.name,
    subtitle: undefined,
    color: undefined,
    yours: plan.yours,
    rank: plan.fits || !anyFits ? index + 1 : undefined,
    cells: {
      LeftAfterTax: shekels(todaysMoney ? plan.left.today : plan.left.nominal),
      ...(plan.costAYear === undefined ? {} : { CostAYear: cost(plan.costAYear) }),
      ...(plan.fees === undefined ? {} : { Fees: shekels(money(plan.fees)) }),
      ...(plan.tax === undefined ? {} : { Tax: money(plan.tax) === 0 ? t.noTax : shekels(money(plan.tax)) }),
    },
    absent: undefined,
    notes: plan.mayCostMoreText ? [{ kind: 'warning', text: `⚠ ${plan.mayCostMoreText}` }] : [],
    linked: false,
    feesLink: false,
    firstUnfit: anyFits && index === firstUnfit,
    more: undefined,
  }))
}

/** The notes of a plan's row: its warnings, then what makes sense of its numbers. */
function notesOf(result: Result): Note[] {
  const { warning, mayCostMore, note, whoCanJoin, spread, taxNote, pensionNote } = result
  const notes: Note[] = []
  if (warning) notes.push({ kind: 'warning', text: `⚠ ${warning}` })
  if (mayCostMore) notes.push({ kind: 'warning', text: `⚠ ${mayCostMore}` })
  if (note) notes.push({ kind: 'note', text: note })
  if (whoCanJoin) notes.push({ kind: 'note', text: whoCanJoin })
  if (spread?.cheapest && spread.dearest) {
    notes.push({
      kind: 'note',
      text: t.spreadBetween(
        shekels(spread.dearest.left),
        spread.dearest.name,
        shekels(spread.cheapest.left),
        spread.cheapest.name,
      ),
    })
  }
  if (spread?.mostAllowed) {
    notes.push({
      kind: 'note',
      text: t.spreadAtMost(spread.mostAllowed.name, shekels(spread.mostAllowed.left)),
    })
  }
  if (taxNote) notes.push({ kind: 'tax', text: taxNote })
  if (pensionNote) notes.push({ kind: 'pension', text: pensionNote })
  return notes
}

/** The plans compared, best first (the core ranks the ones the first
 * deposit can't open after the rest). `child`: listed under a way. */
export function planRows(results: Result[], whyNot: WhyNotWords, child?: string): Row[] {
  const firstUnfit = results.find(({ outcome, fits }) => outcome && !fits)?.plan.id
  return results.map((result) => {
    const { plan, outcome, rank } = result
    const words = whyNot(result.whyNot)
    return {
      id: plan.id,
      key: child === undefined ? plan.id : `${child}>${plan.id}`,
      name: plan.info.name,
      subtitle: plan.subtitle,
      color: plan.color,
      yours: plan.yours !== undefined,
      rank,
      cells: outcome && cellsOf(outcome),
      absent: outcome ? undefined : { ...words, reason: result.notOffered },
      notes: notesOf(result),
      linked: child === undefined && outcome !== undefined,
      feesLink: child === undefined,
      firstUnfit: plan.id === firstUnfit,
      more: undefined,
    }
  })
}

/** The ways compared, best first, each as the plan that holds it best, with
 * the plans that could hold it under it. `labelOf` names a plan by its key. */
export function wayRows(
  ways: WayData[],
  colorOf: (way: Way) => string,
  labelOf: (key: PlanKey) => string,
  plansOf: (way: WayData) => Result[],
  whyNot: WhyNotWords,
): Row[] {
  let rank = 0
  return ways.map((way) => {
    const best = bestPlanOf(way)
    const id = wayKey(way.way)
    const outcome = best?.outcome
    if (outcome) rank += 1
    const notes: Note[] = []
    if (best?.mayCostMore) notes.push({ kind: 'warning', text: `⚠ ${best.mayCostMore}` })
    if (best?.warning) notes.push({ kind: 'warning', text: `⚠ ${best.warning}` })
    if (way.productCost !== undefined && way.productCost > 0) {
      notes.push({ kind: 'note', text: t.productCostNote(shekels(way.productCost)) })
    }
    return {
      id,
      key: id,
      name: way.name,
      subtitle: best && labelOf(best.key),
      color: colorOf(way.way),
      yours: false,
      rank: outcome ? rank : undefined,
      cells: outcome && {
        ...cellsOf(outcome),
        // The way's own figures, which count the fund's cost too.
        ...(way.afterTax === undefined ? {} : { LeftAfterTax: shekels(way.afterTax) }),
        ...(way.costAYear === undefined ? {} : { CostAYear: yearlyCost(way.costAYear) }),
      },
      absent: outcome
        ? undefined
        : { ...whyNot(way.plans[0]?.whyNot ?? 'NotSold'), reason: way.plans[0]?.notOffered },
      notes,
      linked: outcome !== undefined,
      feesLink: true,
      firstUnfit: false,
      more: way.plans.length > 1 ? planRows(plansOf(way), whyNot, id) : undefined,
    }
  })
}
