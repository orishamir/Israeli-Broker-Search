/** Whether the screen is mostly used with a finger. Charts then don't react
 * to "hovering": a finger touching a chart to scroll the page would count as
 * one. A tap still pins. */
export const touchScreen = matchMedia('(pointer: coarse)').matches
