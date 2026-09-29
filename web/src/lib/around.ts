// The line under the best plan in the summary: where, at other values of
// the deposit the chart by deposit varies, another plan becomes cheaper.
// Pure, so it can be tested without a browser.

import type { AroundData, PlanKey, Swept } from './core/core'
import { shekels } from './format'
import { t } from './text'

/** "The cheapest at any monthly deposit from ₪100 to ₪32,000", or where
 * another plan is cheaper, each way. `labelOf` names a plan. */
export function aroundWords(around: AroundData, swept: Swept, labelOf: (key: PlanKey) => string): string {
  const { below, above } = around
  if (below && above) {
    const [low, high] = [shekels(below.amount), shekels(above.amount)]
    return t.cheaperBothWays(low, labelOf(below.key), high, labelOf(above.key), swept)
  }
  if (below) return t.cheaperBelow(shekels(below.amount), labelOf(below.key), swept)
  if (above) return t.cheaperAbove(shekels(above.amount), labelOf(above.key), swept)
  return t.cheapestThroughout(shekels(around.from), shekels(around.to), swept)
}
