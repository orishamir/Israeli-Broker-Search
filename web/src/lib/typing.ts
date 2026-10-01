// When a number was last typed into a field. What follows the inputs waits
// for the typing to pause (see `drawAt`), rather than showing 3, 35 and 350
// on the way to 3,500. A slider or a choice isn't typing: it's followed.

let last = -Infinity
addEventListener(
  'input',
  ({ target }) => {
    if (target instanceof HTMLInputElement && target.type === 'text') last = performance.now()
  },
  { capture: true },
)

/** When a number was last typed, as `performance.now()` tells time. */
export const lastTyped = (): number => last
