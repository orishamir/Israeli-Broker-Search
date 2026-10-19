// The donut's options: where every shekel goes at the end (deposit, gain
// kept, fees, tax, and what the place keeps out of the rate). Pure, so it is
// tested without a browser. REDESIGN.md (§2.7, §2.13) replaces it with
// bars of how much of the gain is kept.
//
// Rules: labels short (הפקדה / רווח / עמלות / מס) and
// `overflow: 'none'`, so ECharts never wraps or cuts them (zrender guesses
// every Hebrew letter as wide as a Chinese one); a slice under 0.5% is
// listed under the donut instead of labelled on it, where its label would
// sit on its neighbours'; each part has one color, the same for every
// place; every data item has an id.

import type { PieSeriesOption } from 'echarts/charts'
import type { BreakdownData } from './core/core'
import type { ChartOption } from './echarts.svelte'
import { shekels } from './format'
import { PART_COLORS, PRODUCT_COLOR } from './colors'
import { t } from './text'

export type PartId = 'putIn' | 'gain' | 'fees' | 'product' | 'tax' | 'kept'

/** The parts' colors. The gain, the tax and what the place keeps are the
 * interest split's (`PART_COLORS`); the deposit is a neutral, as it isn't
 * a result; fees are blue. Checked with the dataviz skill's validator on the
 * page's surface for the neighbours in the donut's order (every slice is also
 * labelled directly): the lightness band fails for the blue, as for the
 * page's other bright colors; the rest passes. */
export const DONUT_COLORS: Record<PartId, string> = {
  putIn: '#4a5266',
  gain: PART_COLORS.yours,
  fees: '#3aa6e0',
  // The fund itself, in the breakdown's color for it.
  product: PRODUCT_COLOR,
  tax: PART_COLORS.tax,
  kept: PART_COLORS.kept,
}

/** A part under this share of the whole isn't labelled on the donut. */
export const TINY = 0.005

export interface Part {
  id: PartId
  name: string
  value: number
  /** Of the whole, 0 to 1. */
  share: number
}

export interface DonutTab {
  id: string
  name: string
  breakdown: BreakdownData
}

/** The tabs over the donut: the answer's kind first, then each kind drawn
 * beside it that has a breakdown of its own (the current account has none). */
export function donutTabs(
  recommendation: { kind: { kind: unknown; name: string }; breakdown: BreakdownData },
  alternatives: readonly { kind: unknown; name: string; breakdown: BreakdownData | undefined }[],
): DonutTab[] {
  const tabs: DonutTab[] = [
    { id: 'answer', name: recommendation.kind.name, breakdown: recommendation.breakdown },
  ]
  alternatives.forEach(({ name, breakdown }, index) => {
    if (breakdown) tabs.push({ id: `${index}:${name}`, name, breakdown })
  })
  return tabs
}

export interface DonutView {
  breakdown: BreakdownData
  touch: boolean
}

/** The parts, in the order round the donut: the shekels the money came to,
 * split by who gets them. The gain is what is left after the deposits (so
 * after fees and tax); what a place keeps is shown only when it keeps
 * something: a bank paying above the rate has no such slice. Nothing under a
 * shekel is a part. */
export function donutParts({ putIn, fees, product, keptByThePlace, tax, left }: BreakdownData): Part[] {
  const named: [PartId, string, number][] = [
    ['putIn', t.donutDeposit, putIn],
    ['gain', t.donutGain, Math.max(0, left - putIn)],
    ['fees', t.donutFees, fees],
    ['product', t.donutProduct, product],
    ['tax', t.donutTax, tax],
    ['kept', t.donutKept, keptByThePlace],
  ]
  const parts = named.filter(([, , value]) => value >= 1)
  const total = parts.reduce((sum, [, , value]) => sum + value, 0)
  return parts.map(([id, name, value]) => ({ id, name, value, share: value / total }))
}

/** The percent as the labels say it: a whole number, one decimal under 1%. */
export const sharePercent = (share: number): string =>
  `${share < 0.01 ? (share * 100).toFixed(1) : Math.round(share * 100)}%`

/** The parts too small to label, as "fees 0.1%, tax 0.3%". */
export const tinyList = (parts: readonly Part[]): string =>
  parts
    .filter(({ share }) => share < TINY)
    .map(({ name, share }) => `${name} ${sharePercent(share)}`)
    .join(', ')

export function donutOption({ breakdown }: DonutView): ChartOption {
  const parts = donutParts(breakdown)
  const series: PieSeriesOption = {
    id: 'donut',
    type: 'pie',
    radius: ['52%', '74%'],
    center: ['50%', '50%'],
    // The parts start at the top, clockwise; the tiny ones aren't drawn.
    startAngle: 90,
    avoidLabelOverlap: true,
    itemStyle: { borderColor: '#151820', borderWidth: 2 },
    label: {
      color: '#e6e8ee',
      fontSize: 12,
      // Never let ECharts wrap or cut a label.
      overflow: 'none',
      formatter: ({ name, percent }) => `${name}\n${sharePercent((percent ?? 0) / 100)}`,
    },
    labelLine: { lineStyle: { color: '#4a5060' } },
    // Hover is the highlight action's: a pie's own emphasis only grows the slice.
    emphasis: { scaleSize: 4 },
    data: parts
      .filter(({ share }) => share >= TINY)
      .map(({ id, name, value }) => ({ id, name, value, itemStyle: { color: DONUT_COLORS[id] } })),
  }
  return {
    tooltip: {
      trigger: 'item',
      formatter: (params) => {
        const { name, value, percent } = params as unknown as {
          name: string
          value: number
          percent?: number
        }
        return `${name}: <b>${shekels(value)}</b> (${sharePercent((percent ?? 0) / 100)})`
      },
    },
    series: [series],
  }
}
