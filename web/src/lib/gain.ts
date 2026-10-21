// "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7, amended by §4.7): each
// option's gain as one bar on one ₪ scale, split into what stays, the tax,
// the fees and the fund's own cost, every part named with its amount on it
// or, where it's too narrow to hold its words, beside it, above or below
// the bar, never covering another. Pure, so it is tested without a browser.
//
// Colours: four hues, one per part, the same in every bar, checked with the
// dataviz skill's validator on the card's surface (#151820, dark, `--pairs
// all`): normal vision ΔE ≥ 20.9, colour-blind ≥ 9.4, contrast ≥ 3:1. "הקרן
// עצמה" has its own, the workbench's (`PRODUCT_COLOR`); it is brighter than
// the validator's lightness band, as the kinds' bright marks are. The words
// on a part are dark, which reads on all four. Never colour alone: every
// part is named.

import { PRODUCT_COLOR } from './colors'
import type { GainPartKind, GainRowData } from './core/core'
import { place, type Placed } from './outcome'

export const GAIN_COLORS: Record<GainPartKind, string> = {
  Kept: '#199e70',
  Tax: '#d95926',
  Fees: '#3987e5',
  Fund: PRODUCT_COLOR,
}

/** A part's words: "עמלות ₪2,090". */
export const partText = (part: { label: string; amount: string }): string =>
  `${part.label}\u00a0${part.amount}`

/** Padding inside a part around its words, px. */
const INSIDE = 12
/** A lane of words beside the bar, px. */
export const GAIN_LANE = 18

export interface GainSegment {
  which: GainPartKind
  /** In today's money. */
  value: number
  /** Where it starts and how wide it is, as shares of the widest bar. */
  from: number
  width: number
  text: string
  /** Whether its words fit on it. */
  inside: boolean
}

export interface GainBar {
  name: string
  note: string | undefined
  segments: GainSegment[]
  /** Parts too narrow for their words, placed beside the bar: lane 0 above
   * it, 1 below it. `x` is from the bar's start edge (its right, on a
   * right-to-left page), px. */
  beside: (Placed & { which: GainPartKind })[]
  /** Parts the bar can't draw (none, or below zero: a gain that didn't
   * cover the deposits), named in words under it. */
  undrawn: string[]
  /** Lanes above and below the bar its words take. */
  above: number
  below: number
}

/** The bars, all on the scale of the biggest gain, on a track `width` px
 * wide, with each part's words measured by `measure`. */
export function gainBars(
  rows: readonly GainRowData[],
  width: number,
  measure: (text: string) => number,
): GainBar[] {
  const drawn = (row: GainRowData) => row.parts.filter((part) => part.value.today > 0)
  const total = (row: GainRowData) => drawn(row).reduce((sum, part) => sum + part.value.today, 0)
  const widest = Math.max(0, ...rows.map(total))
  return rows.map((row) => {
    let before = 0
    const segments = drawn(row).map((part): GainSegment => {
      const share = widest > 0 ? part.value.today / widest : 0
      const text = partText(part)
      const segment = {
        which: part.which,
        value: part.value.today,
        from: before,
        width: share,
        text,
        inside: measure(text) + INSIDE <= share * width,
      }
      before += share
      return segment
    })
    const outside = segments.filter(({ inside }) => !inside)
    const beside = place(
      outside.map((segment, index) => ({
        key: segment.which,
        text: segment.text,
        width: measure(segment.text),
        anchor: (segment.from + segment.width / 2) * width,
        align: 'center' as const,
        // Alternating, so neighbours start apart: above, then below.
        lanes: index % 2 === 0 ? [0, 1] : [1, 0],
      })),
      width,
    ).map((label, index) => ({ ...label, which: outside[index].which }))
    return {
      name: row.name,
      note: row.note,
      segments,
      beside,
      undrawn: row.parts.filter((part) => part.value.today <= 0).map(partText),
      above: beside.some(({ lane }) => lane === 0) ? 1 : 0,
      below: Math.max(0, ...beside.map(({ lane }) => lane)),
    }
  })
}
