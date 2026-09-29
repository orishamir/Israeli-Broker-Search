// The fee breakdown's options: each plan's fees as a stacked bar, and one
// plan's fees piling up year by year. Pure, so it can be tested without a
// browser; the component wires it to ECharts and to the app's state.

import type { BarSeriesOption } from 'echarts/charts'
import type { FeeAmounts } from './core/core'
import type { ChartOption } from './echarts.svelte'
import { compactShekels, readableOn, shekels } from './format'
import { t } from './text'

export type FeeType = Exclude<keyof FeeAmounts, 'total'>

/** A fee to compare the plans by, or all of them. */
export type Focus = 'all' | FeeType

/** The kinds of fee, in the order they're stacked. The colors are checked
 * for color blindness in this order, where each differs from its
 * neighbors; not every pair differs, so a focused fee fades the others. */
export const FEE_TYPES: { key: FeeType; name: string; color: string; explanation: string }[] = [
  { key: 'purchases', name: t.purchases, color: '#3987e5', explanation: t.purchasesTip },
  { key: 'conversions', name: t.conversions, color: '#d95926', explanation: t.conversionsTip },
  { key: 'account', name: t.account, color: '#199e70', explanation: t.accountTip },
  { key: 'selling', name: t.selling, color: '#c98500', explanation: t.sellingTip },
]

// The page's colors (see app.css).
const WEAK = '#8e95a5'
const TEXT = '#e6e8ee'
const GRID = 'rgba(255, 255, 255, 0.07)'
const SURFACE = '#151820'
export const FADED = 'rgba(142, 149, 165, 0.3)'

/** One plan's bar. */
export interface Bar {
  id: string
  label: string
  color: string
  /** Your own plans get a hollow dot, like their dotted lines. */
  hollow: boolean
  fees: FeeAmounts
}

/** Least fees first: in total, or in the focused fee. */
export const byFee = (bars: Bar[], focus: Focus): Bar[] =>
  bars.toSorted(
    (a, b) =>
      (focus === 'all' ? a.fees.total : a.fees[focus]) - (focus === 'all' ? b.fees.total : b.fees[focus]),
  )

/** The fees in stacking order: the focused fee first, so it starts where the bar does. */
export const stacking = (focus: Focus) =>
  focus === 'all'
    ? FEE_TYPES
    : [FEE_TYPES.find(({ key }) => key === focus)!, ...FEE_TYPES.filter(({ key }) => key !== focus)]

/** A fee's color: its own, unless another fee is focused. */
export const colorOf = (type: (typeof FEE_TYPES)[number], focus: Focus) =>
  focus === 'all' || focus === type.key ? type.color : FADED

export interface BarsView {
  /** In the order to draw them (see `byFee`). */
  bars: Bar[]
  focus: Focus
  pinned: ReadonlySet<string>
  /** The box's width in pixels, for fitting names and amounts. */
  width: number
  /** A text's width in pixels, in the names' font (see `fitName`). */
  measure: (text: string) => number
}

/** Screens where the rows get a little more height, for a finger. Decided
 * by the screen rather than the box's own width: a height set from an
 * observed width would change the box again while the browser is still
 * reporting its size. */
export const NARROW_SCREEN = '(width < 560px)'
export const rowHeight = (narrow: boolean) => (narrow ? 46 : 44)
/** The names' font size: ECharts' default. */
export const NAME_SIZE = 12

// Each row, top to bottom: the plan's name on a line of its own, a gap, and
// its bar. ECharts centers both a row's bar and its name in the row, so an
// empty slot the height of the name's line goes above each bar, and the
// name is moved up into it.
const NAME_LINE = 15
const GAP = 3
const BAR = 16
const NAME_ABOVE = (GAP + BAR) / 2
const BAR_BELOW = (NAME_LINE + GAP) / 2

/** Around the rows: room for the axis below them. */
const MARGIN = { left: 4, right: 4, top: 4, bottom: 24 }
/** The bars' box: a row for each plan, and the axis. */
export const barsHeight = (rows: number, narrow: boolean) =>
  rows * rowHeight(narrow) + MARGIN.top + MARGIN.bottom
/** Where the name and the bar of row `index` are, down from the box's top. */
export const nameY = (index: number, narrow: boolean) =>
  MARGIN.top + (index + 0.5) * rowHeight(narrow) - NAME_ABOVE
export const barY = (index: number, narrow: boolean) =>
  MARGIN.top + (index + 0.5) * rowHeight(narrow) + BAR_BELOW

/** `name`, or as much of it as fits in `room`, then "…". Measured here:
 * ECharts guesses every letter outside ASCII to be as wide as a Chinese one
 * (zrender 6's `measureCharWidth`) when it cuts or wraps a text, so it cut
 * Hebrew names at half the room. Only a name of your own can be that long. */
export function fitName(name: string, room: number, measure: (text: string) => number): string {
  if (measure(name) <= room) return name
  // By code point, so no emoji is cut in half.
  const kept = [...name]
  while (kept.length > 0 && measure(`${kept.join('')}…`) > room) kept.pop()
  return `${kept.join('').trimEnd()}…`
}

export function barsOption({ bars, focus, pinned, width, measure }: BarsView): ChartOption {
  const largest = Math.max(...bars.map(({ fees }) => fees.total))
  // Roughly: the box without the totals.
  const pixelsPerShekel = Math.max(width - 60, 0) / largest
  const fits = (amount: number) => amount * pixelsPerShekel > 44
  // The box, less its margins and a pinned name's padding.
  const room = width - MARGIN.left - MARGIN.right - 2 * 5
  return {
    grid: { ...MARGIN },
    xAxis: {
      type: 'value',
      max: largest,
      axisLabel: { formatter: compactShekels, color: WEAK, showMaxLabel: false, hideOverlap: true },
      splitLine: { lineStyle: { color: GRID } },
    },
    yAxis: [
      {
        type: 'category',
        inverse: true,
        // Ids, not names, which can repeat: the formatter shows the names.
        data: bars.map(({ id }) => id),
        axisTick: { show: false },
        axisLine: { show: false },
        triggerEvent: true,
        axisLabel: {
          // Above the bar, from where it starts: padding below the name
          // moves it up into the slot over the bar.
          inside: true,
          margin: 0,
          padding: [0, 0, 2 * NAME_ABOVE, 0],
          color: TEXT,
          fontSize: NAME_SIZE,
          // Every name: ECharts skips labels it thinks would crowd.
          interval: 0,
          // A dot in the plan's color (hollow for your own plans), and the
          // name, on a tint of that color when pinned.
          formatter: (_id: string, index: number) => {
            const bar = bars[index]
            const dot = bar.hollow ? '◯' : '●'
            const style = pinned.has(bar.id) ? `pinned${index}` : 'name'
            const name = fitName(bar.label, room - measure(`${dot} `), measure)
            return `{dot${index}|${dot}} {${style}|${name}}`
          },
          rich: {
            ...Object.fromEntries(
              bars.flatMap((bar, index) => [
                [`dot${index}`, { color: bar.color }],
                [
                  `pinned${index}`,
                  {
                    color: TEXT,
                    backgroundColor: `${bar.color}40`,
                    borderRadius: 4,
                    padding: [2, 5],
                  },
                ],
              ]),
            ),
            name: { color: TEXT },
          },
        },
      },
      {
        // Each plan's total, level with its bar.
        type: 'category',
        inverse: true,
        position: 'right',
        data: bars.map(({ fees }) => compactShekels(fees.total)),
        axisTick: { show: false },
        axisLine: { show: false },
        axisLabel: {
          // Padding moves only rich text, as the names are: plain text stays
          // in the middle of the row.
          padding: [2 * BAR_BELOW, 0, 0, 0],
          formatter: (total: string) => `{total|${total}}`,
          rich: { total: { color: TEXT, fontWeight: 'bold' } },
        },
      },
    ],
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow',
        // Headed by the plan's name: the axis holds ids.
        label: { formatter: ({ value }) => bars.find(({ id }) => id === value)?.label ?? '' },
      },
      valueFormatter: (value) => shekels(value as number),
    },
    // No ids: ECharts would then keep each series' old position, and a
    // focused fee wouldn't move to the start of the bars.
    series: [
      {
        // Around each pinned or hovered plan, from its name to its total, an
        // outline in its color. Only an outline: filled colors in the bars
        // are always fees, and the plans' colors look much like the fees'.
        // Pinned ones are drawn here; a hovered one is highlighted (see the
        // component's `setup`), since redrawing would lose a tap that's also
        // a hover. Clear inside, but not empty: a click or a tap anywhere on
        // the row picks its plan, not only on its name or its bar.
        type: 'custom',
        tooltip: { show: false },
        data: bars.map((_, index) => [0, index]),
        renderItem: (_params, api) => {
          const index = api.value(1) as number
          const { color, id } = bars[index]
          const [, middle] = api.coord([0, index])
          const [, height] = api.size!([0, 1]) as number[]
          return {
            type: 'rect',
            shape: {
              x: 1,
              y: middle - height / 2 + 2,
              width: api.getWidth() - 2,
              height: height - 4,
              r: 6,
            },
            style: {
              fill: 'transparent',
              stroke: pinned.has(id) ? color : 'transparent',
              lineWidth: 1.5,
            },
            emphasis: { style: { stroke: color } },
          }
        },
      },
      {
        // The empty slot above each bar, where its name goes. Being the
        // first bars, they're put first in each row, at the top.
        type: 'bar',
        barWidth: NAME_LINE,
        data: bars.map(() => 0),
        silent: true,
        tooltip: { show: false },
      },
      ...stacking(focus).map((type): BarSeriesOption => ({
        name: type.name,
        type: 'bar',
        stack: 'fees',
        barWidth: BAR,
        // ECharts puts a gap after each slot, as a share of its height.
        barGap: `${(GAP / NAME_LINE) * 100}%`,
        // Once plans are pinned, the others fade a little.
        data: bars.map(({ id, fees }) => ({
          value: fees[type.key],
          itemStyle: { opacity: pinned.size === 0 || pinned.has(id) ? 1 : 0.6 },
        })),
        color: colorOf(type, focus),
        // A thin gap between the parts of a bar.
        itemStyle: { borderColor: SURFACE, borderWidth: 1 },
        // Amounts only where they fit.
        label: {
          show: true,
          position: 'inside',
          color: readableOn(type.color),
          fontSize: 11,
          formatter: ({ value }) =>
            fits(value as number) && colorOf(type, focus) !== FADED ? compactShekels(value as number) : '',
        },
      })),
    ],
  }
}

/** One plan's fees piling up: nothing paid at the start, then the total by
 * the end of each year. */
export function overTimeOption(feesUpToYear: FeeAmounts[], focus: Focus): ChartOption {
  return {
    grid: { left: 4, right: 12, top: 12, bottom: 28 },
    xAxis: {
      type: 'value',
      name: t.years,
      nameLocation: 'middle',
      nameGap: 24,
      nameTextStyle: { color: WEAK },
      min: 0,
      max: feesUpToYear.length,
      minInterval: 1,
      axisLabel: { color: WEAK },
      axisLine: { lineStyle: { color: GRID } },
      splitLine: { show: false },
    },
    yAxis: {
      type: 'value',
      axisLabel: { formatter: compactShekels, color: WEAK },
      splitLine: { lineStyle: { color: GRID } },
    },
    tooltip: {
      trigger: 'axis',
      valueFormatter: (value) => shekels(value as number),
      axisPointer: { label: { formatter: ({ value }) => t.after(Number(value), 0) } },
    },
    series: stacking(focus).map((type) => ({
      name: type.name,
      type: 'line',
      stack: 'over time',
      symbol: 'none',
      color: colorOf(type, focus),
      lineStyle: { width: 1.5 },
      areaStyle: { opacity: 0.85 },
      data: [[0, 0], ...feesUpToYear.map((fees, year) => [year + 1, fees[type.key]])],
    })),
  }
}
