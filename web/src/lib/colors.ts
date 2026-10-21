// The colors the ticked plans and places are drawn in, the same in the
// table, the chips and the charts.

import type { Kind } from './core/core'

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
 * color in every bar. A yellow-green, apart from the gain's green (teal was too near it) and the
 * fees' blue, orange, violet and amber: every pair against them passes the
 * dataviz validator (normal vision ΔE ≥ 16.7, color-blind ≥ 10.9). */
export const PRODUCT_COLOR = '#bcc83c'

/** A way's own color, beside its name only: by its place in the core's list
 * of ways, so it doesn't change as other ways are ticked. */
export const wayColor = (index: number): string => PLAN_COLORS[index % PLAN_COLORS.length]

// ── The guide's kinds ──
// One color for each type of product, the same in every view, whatever else
// is drawn: a kind never changes color when another is added or dropped
// (REDESIGN.md §2.8). The outcome chart shows up to five rows: any of the
// five safe kinds, the study fund's track, and one other holding of the
// S&P 500 (the guide's lead way). Those seven are told apart in every pair
// (the dataviz skill's validator, `--pairs all`, on the page's surface
// #151820: normal vision ΔE ≥ 15.1, contrast ≥ 3:1; color-blind ΔE ≥ 8.4).
// The money-market fund was a lilac beside the index's magenta, too near
// in hue (the review of 2026-10-20, X17): it is a light green now. The
// any-day deposit (a pale yellow) and makam (a violet) came with §4.9;
// with them the nine are still told apart in every pair (normal vision
// ΔE ≥ 15.1, color-blind ≥ 8.0, contrast ≥ 3:1).
// Thirteen kinds can't all be told apart, and the other holdings of the
// S&P 500 are never drawn beside each other in the guide (the workbench
// colors its ways by `wayColor`), so they share the index's magenta. The
// lightness band fails as it does for `PLAN_COLORS` (bright marks on a dark
// page). The current account is the page's grey, drawn dashed.

const SP500 = '#c547a3'

export const KIND_COLORS: Record<string, string> = {
  MoneyMarketFund: '#9fe07a',
  FixedDeposit: '#4682cc',
  LinkedDeposit: '#39b7cb',
  AnyDayDeposit: '#fdfda5',
  TreasuryBill: '#b583fc',
  'GovernmentBond:linked': '#ca9d33',
  GovernmentBond: '#419363',
  StudyFundTrack: '#e54e3c',
  IsraeliIndexFund: SP500,
  IsraeliEtf: SP500,
  UsFund: SP500,
  IrishFund: SP500,
  InvestmentGemelTrack: SP500,
  SavingsPolicyTrack: SP500,
  ForeignEtfInTelAviv: SP500,
  CurrentAccount: UNTICKED_COLOR,
}

export function kindColor(kind: Kind): string {
  if (typeof kind === 'string') return KIND_COLORS[kind]
  if ('GovernmentBond' in kind)
    return KIND_COLORS[kind.GovernmentBond.linked ? 'GovernmentBond:linked' : 'GovernmentBond']
  return KIND_COLORS[kind.Way.product]
}
