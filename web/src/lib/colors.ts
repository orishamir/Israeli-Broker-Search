// The colors the ticked plans and places are drawn in, the same in the
// table, the chips and the charts.

/** Chosen to differ as much as they can on the dark background, with color
 * blindness too: the first 8 clearly, the rest less so, as 14 colors can't
 * all be far apart. Checked with the dataviz skill's validator. */
export const PLAN_COLORS = [
  '#56b4e9',
  '#e69f00',
  '#8d73f0',
  '#c1e319',
  '#e141aa',
  '#3ceedd',
  '#fd2e1c',
  '#fc899d',
  '#27ef8e',
  '#dc855d',
  '#39bda0',
  '#c86b82',
  '#adc367',
  '#4493d0',
]

/** An unticked plan isn't drawn, so it only needs a color where it's listed. */
export const UNTICKED_COLOR = '#8e95a5'

/** The ticked ids' colors. Each keeps the color `handedOut` gives it while
 * it's ticked, and a newly ticked one takes the palette's first free color,
 * so they differ as much as they can; past the palette's end, the least
 * used. `handedOut` is changed to match, for the next call. */
export function handOut(
  handedOut: Map<string, string>,
  ticked: ReadonlySet<string>,
): ReadonlyMap<string, string> {
  for (const id of handedOut.keys()) if (!ticked.has(id)) handedOut.delete(id)
  for (const id of ticked) {
    if (handedOut.has(id)) continue
    const used = [...handedOut.values()]
    const uses = (color: string) => used.filter((each) => each === color).length
    const color = PLAN_COLORS.reduce((least, each) => (uses(each) < uses(least) ? each : least))
    handedOut.set(id, color)
  }
  return new Map(handedOut)
}

/** `item` with a color looked up whenever it's read, so it stays the same
 * object as its color changes. */
export function withColor<T extends object>(item: T, color: () => string): T & { color: string } {
  return Object.defineProperty(item, 'color', { get: color, enumerable: true }) as T & { color: string }
}

// ── Package K: the workbench's ways and the fund's own cost ──
/** "הקרן עצמה" in the fee breakdown: a part of a bar like the others, so one
 * color in every bar. A teal, apart from the fees' blue, orange, violet and
 * amber. */
export const PRODUCT_COLOR = '#1f9e8f'

/** A way's own color, beside its name only: by its place in the core's list
 * of ways, so it doesn't change as other ways are ticked. */
export const wayColor = (index: number): string => PLAN_COLORS[index % PLAN_COLORS.length]
