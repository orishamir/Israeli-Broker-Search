// What the workbench's charts draw: a line or a bar for each plan (when
// comparing plans on one exchange) or for each way (the S&P 500's ways,
// each drawn as its best plan). Pure, so the lines can be tested without a
// browser.

import { wayColor } from './colors'
import type { OutcomeData, PlanOutcomeData, Way, WayChoice, WayData, WhyNot } from './core/core'
import { wayKey } from './kinds'

/** One line or bar of a chart. */
export interface ChartEntry {
  /** What the table row, the line and the bar share (a plan's id, or a way's key). */
  id: string
  label: string
  color: string
  /** Its row's number in the table, 1 for the best. */
  rank: number | undefined
  /** Your own plans are dotted and hollow. */
  yours: boolean
  outcome: OutcomeData
}

/** The plans without numbers, grouped by why: a line under the breakdown for each. */
export interface Missing {
  whyNot: WhyNot | undefined
  plans: { id: string; label: string; reason: string | undefined }[]
}

/** The plan of a way that stands for it: the first the first deposit opens
 * (the core lists them best first), else the first with numbers. */
export function bestPlanOf(way: { plans: PlanOutcomeData[] }): PlanOutcomeData | undefined {
  const usable = way.plans.filter(({ outcome }) => outcome)
  return usable.find(({ fits }) => fits) ?? usable[0]
}

/** Each way with numbers, as its best plan's outcome, numbered as the ways table is. */
export function wayEntries(ways: WayData[], colorOf: (way: Way) => string): ChartEntry[] {
  let rank = 0
  return ways.flatMap((way) => {
    const outcome = bestPlanOf(way)?.outcome
    if (!outcome) return []
    rank += 1
    return [{ id: wayKey(way.way), label: way.name, color: colorOf(way.way), rank, yours: false, outcome }]
  })
}

/** What a plan's row has, as the charts need it. */
export interface PlanRow {
  plan: { id: string; label: string; color: string; yours?: unknown }
  outcome: OutcomeData | undefined
  rank: number | undefined
  whyNot: WhyNot | undefined
  notOffered: string | undefined
}

export const planEntries = (rows: PlanRow[]): ChartEntry[] =>
  rows.flatMap(({ plan, outcome, rank }) =>
    outcome
      ? [
          {
            id: plan.id,
            label: plan.label,
            color: plan.color,
            rank,
            yours: plan.yours !== undefined,
            outcome,
          },
        ]
      : [],
  )

/** The plans left out, in the table's order, a group for each reason. */
export function missingPlans(rows: PlanRow[]): Missing[] {
  const groups: Missing[] = []
  for (const { plan, outcome, whyNot, notOffered } of rows) {
    if (outcome) continue
    const entry = { id: plan.id, label: plan.label, reason: notOffered }
    const group = groups.find((each) => each.whyNot === whyNot)
    if (group) group.plans.push(entry)
    else groups.push({ whyNot, plans: [entry] })
  }
  return groups
}

/** A way's color, by its place in the core's list of ways (see `wayColor`). */
export const wayColorOf =
  (choices: WayChoice[]) =>
  (way: Way): string =>
    wayColor(
      Math.max(
        0,
        choices.findIndex((choice) => wayKey(choice.way) === wayKey(way)),
      ),
    )
