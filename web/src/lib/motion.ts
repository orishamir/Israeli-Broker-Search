/** Whether the user asked their system for less motion. Animations are then
 * skipped: CSS ones by the rule in app.css, the rest by checking this. */
export const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches

/** An animation's length: `ms`, or none when the user asked for less motion. */
export const duration = (ms: number): number => (reducedMotion ? 0 : ms)
