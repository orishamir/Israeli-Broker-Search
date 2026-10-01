// When a chart draws a change, and how long it glides there. Pure, so it can
// be tested without a browser; the chart attachment (echarts.svelte.ts)
// keeps the times.

import { SETTLE } from './motion'

/** A change that comes while the chart is still gliding to the last one
 * glides for this long instead: more are coming (a slider being dragged, an
 * arrow key held), and the chart follows them in shorter steps. */
export const CHAINED = 250

/** While a number is being typed, a chart waits until the typing has paused
 * this long and glides once to the number typed, rather than drawing 3, 35
 * and 350 on the way to 3,500, */
export const TYPING_PAUSE = 300
/** but no longer than this after the first change it hasn't drawn. */
export const MAX_WAIT = 900

/** When a glide has all but settled: the page's curve has covered 99% of
 * the way when 85% of the time has passed. The next glide may start then:
 * the jump to where the last one was going is too small to see. Waiting out
 * the rest made the chart all but stop between glides while a slider was
 * being dragged. */
export const settlesIn = (glide: number): number => glide * 0.85

export interface Timing {
  now: number
  /** When the glide in flight has all but settled (`settlesIn`): in the
   * past if none is in flight. */
  settled: number
  /** When a number was last typed into a field. */
  lastTyped: number
  /** When the first change not drawn yet came. */
  firstWaiting: number
}

/** When a chart may draw the latest change: once the glide in flight has
 * settled, since cut short it would jump (ECharts starts a line's next glide
 * from where the last one was going, not from where the line is), and while
 * a number is being typed, once the typing has paused. */
export const drawAt = ({ now, settled, lastTyped, firstWaiting }: Timing): number =>
  Math.max(now, settled, Math.min(lastTyped + TYPING_PAUSE, firstWaiting + MAX_WAIT))

/** How long a draw glides: less while changes keep coming. */
export const glideFor = (chained: boolean): number => (chained ? CHAINED : SETTLE)
