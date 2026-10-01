import { slide, type TransitionConfig } from 'svelte/transition'
import { createCubicEasingFunc } from 'zrender/lib/animation/cubicEasing.js'

/** Whether the user asked their system for less motion. Animations are then
 * skipped: CSS ones by the rule in app.css, the rest by checking this. */
export const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches

/** An animation's length: `ms`, or none when the user asked for less motion. */
export const duration = (ms: number): number => (reducedMotion ? 0 : ms)

/** How long the results take to move to new numbers: the charts glide, the
 * summary's numbers roll and the table's rows slide for this long, together.
 * `--settle` in app.css. */
export const SETTLE = 400

/** The page's curves, as app.css has them (`--ease-out`, `--ease-in`): what
 * arrives starts fast and settles; what leaves gets out of the way. ECharts
 * reads them as they are. */
export const EASE_OUT = 'cubic-bezier(0.2, 0, 0, 1)'
export const EASE_IN = 'cubic-bezier(0.4, 0, 1, 1)'

/** The same curves as functions, for Svelte's tweens and transitions. They
 * are ECharts' own, so the numbers and the charts move alike. */
export const easeOut = createCubicEasingFunc(EASE_OUT)!
export const easeIn = createCubicEasingFunc(EASE_IN)!

/** For a block that comes and goes (More options' fields, a card): it opens
 * downwards as it fades in, and closes faster. `transition:reveal`. */
export function reveal(
  node: Element,
  _params: undefined,
  { direction }: { direction: 'in' | 'out' | 'both' },
): TransitionConfig {
  const leaving = direction === 'out'
  const opening = slide(node, { duration: duration(leaving ? 160 : 240), easing: leaving ? easeIn : easeOut })
  // Slide's own fade is over in the first twentieth of the way. Its CSS ends
  // without a semicolon.
  return { ...opening, css: (t, u) => `${opening.css!(t, u)}; opacity: ${t}` }
}
