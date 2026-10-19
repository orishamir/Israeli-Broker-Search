// "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7): each option's gain as one
// bar on one ₪ scale, split into what stays, the tax, the fees and the
// fund's own cost. Pure, so it is tested without a browser.
//
// Colours: three hues, checked with the dataviz skill's validator on the
// card's surface (#151820, dark, `--pairs all`, every check passes); the
// fund's own cost is a fee too, so it wears the fees' hue, hatched. Every
// part is also named with its amount under its bar: never colour alone.

import type { GainRowData } from './core/core'

export type GainPartId = 'kept' | 'tax' | 'fees' | 'product'

export const GAIN_PARTS: readonly GainPartId[] = ['kept', 'tax', 'fees', 'product']

export const GAIN_COLORS: Record<GainPartId, string> = {
  kept: '#199e70',
  tax: '#d95926',
  fees: '#3987e5',
  product: '#3987e5',
}

export interface GainSegment {
  id: GainPartId
  /** In today's money. */
  value: number
  /** Of the widest bar, 0 to 1. */
  width: number
}

export interface GainBar {
  name: string
  /** The parts in order, every one, for the words under the bar. */
  parts: { id: GainPartId; value: number }[]
  /** The parts the bar draws: none that is empty or negative. */
  segments: GainSegment[]
}

/** The bars, all on the scale of the biggest gain. A part below zero (a
 * gain that didn't cover the deposits) isn't drawn, and still named. */
export function gainBars(rows: readonly GainRowData[]): GainBar[] {
  const parts = (row: GainRowData) => GAIN_PARTS.map((id) => ({ id, value: row[id].today }))
  const total = (row: GainRowData) => parts(row).reduce((sum, part) => sum + Math.max(0, part.value), 0)
  const widest = Math.max(0, ...rows.map(total))
  return rows.map((row) => ({
    name: row.name,
    parts: parts(row),
    segments: parts(row)
      .filter((part) => part.value > 0)
      .map((part) => ({ ...part, width: widest > 0 ? part.value / widest : 0 })),
  }))
}
