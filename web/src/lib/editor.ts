import * as core from './core/core'
import type { FeeKind, FeeKindChoice, PlanData } from './core/core'

/** Changes the edited plan with `update`, a core function; `field` is where
 * to show why, if the core refuses. */
export type Change = (field: string, update: (plan: PlanData) => PlanData) => void

/** The plan a copy is compared to: its name, and the plan itself. */
export type Original = { name: string; data: PlanData }

/** A kind of fee, as the core names and explains it. */
export const feeKind = (value: FeeKind): FeeKindChoice =>
  core.feeKinds().find((kind) => kind.value === value)!
