import type { PlanData } from './core/core'

/** Changes the edited plan with `update`, a core function; `field` is where
 * to show why, if the core refuses. */
export type Change = (field: string, update: (plan: PlanData) => PlanData) => void

/** The plan a copy is compared to: its name, and the plan itself. */
export type Original = { name: string; data: PlanData }
