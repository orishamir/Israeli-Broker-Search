// Which plan the "I already have a portfolio at a bank" situation ticks: the
// bank's branch plan (the plan for an order given to a banker or at a
// branch). The core already adds that plan to the guide's comparison on its
// own and marks it `yours`; ticking it as well puts it in "what's compared"
// and the workbench's list. For "another bank" the core picks a stand-in, so
// the plan to tick is the one the recommendation says is `yours`.

import type { Plan } from './app.svelte'
import type { RecommendationData } from './core/core'

/** The English name of the branch plan, in the tariffs. */
export const BRANCH_PLAN = 'Branch or through a banker'

/** The listed branch plans among `plans`. */
export const branchPlans = (plans: Plan[]): Plan[] =>
  plans.filter((plan) => plan.key.kind === 'listed' && plan.info.englishName === BRANCH_PLAN)

const plain = (text: string) => text.replaceAll(' ', ' ')

/** The branch plan the recommendation marks as the saver's own, found by the
 * name the core gave it (the broker's short name, then the plan's). */
export function yoursBranch(plans: Plan[], recommendation: RecommendationData | undefined): Plan | undefined {
  // For "another bank" the core shows the plan as "Another bank", and says
  // here which plan it really is.
  const own = recommendation?.plans.find((plan) => plan.yours)
  const name = own?.ownPlan ?? own?.name
  if (name === undefined) return undefined
  return branchPlans(plans).find((plan) => plain(plan.label) === plain(name))
}
