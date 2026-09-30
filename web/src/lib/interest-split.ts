// Where the interest goes, as the short-term calculator draws it: for each
// place, the interest the Bank of Israel's rate would pay on the money, as
// one bar split three ways: what the saver keeps, what the place keeps, and
// the tax. Pure, so it can be tested without a browser; the component wires
// it to ECharts and to the state. The rows are laid out as the fee
// breakdown's are: each place's name above its bar.

import type { BarSeriesOption } from 'echarts/charts'
import type { ChartOption } from './echarts.svelte'
import { BAR, BAR_BELOW, fitName, GAP, MARGIN, NAME_ABOVE, NAME_LINE, NAME_SIZE } from './fee-breakdown'
import { compactShekels, readableOn, shekels } from './format'
import { t } from './text'

// The page's colors (see app.css).
const WEAK = '#8e95a5'
const TEXT = '#e6e8ee'
const GRID = 'rgba(255, 255, 255, 0.07)'
const SURFACE = '#151820'
/** What the place keeps, and the tax: grays, so the saver's part, in the
 * place's own color, is what stands out. */
export const KEPT_COLOR = '#6b7285'
export const TAX_COLOR = '#3a4050'

/** One place's bar. */
export interface Split {
  id: string
  label: string
  color: string
  /** Your own deposits get a hollow dot, like their dotted lines. */
  hollow: boolean
  /** ₪ of the interest the saver keeps, after tax. */
  yours: number
  /** ₪ the place keeps: a fund's fee, or how much less than the rate a bank
   * pays; below zero where it pays more. */
  kept: number
  tax: number
}

export interface SplitView {
  /** In the table's order. */
  splits: Split[]
  /** The interest the Bank of Israel's rate would pay on the money: the
   * line every bar reaches where the place keeps something. */
  atTheRate: number
  pinned: ReadonlySet<string>
  /** The box's width in pixels, for fitting names and amounts. */
  width: number
  /** A text's width in pixels, in the names' font (see `fitName`). */
  measure: (text: string) => number
}

/** A bar's length: past the line where a bank pays more than the rate. */
export const lengthOf = ({ yours, kept, tax }: Split) => yours + tax + Math.max(kept, 0)

export function splitOption({ splits, atTheRate, pinned, width, measure }: SplitView): ChartOption {
  const largest = Math.max(atTheRate, ...splits.map(lengthOf))
  // Roughly: the box without the totals.
  const pixelsPerShekel = Math.max(width - 60, 0) / largest
  const fits = (amount: number) => amount * pixelsPerShekel > 44
  // The box, less its margins and a pinned name's padding.
  const room = width - MARGIN.left - MARGIN.right - 2 * 5
  const opacity = (id: string) => (pinned.size === 0 || pinned.has(id) ? 1 : 0.6)
  const part = (
    name: string,
    value: (split: Split) => number,
    color: (split: Split) => string,
    labelled = true,
  ): BarSeriesOption => ({
    name,
    type: 'bar',
    stack: 'interest',
    barWidth: BAR,
    // ECharts puts a gap after each slot, as a share of its height.
    barGap: `${(GAP / NAME_LINE) * 100}%`,
    data: splits.map((split) => ({
      value: value(split),
      itemStyle: { color: color(split), opacity: opacity(split.id) },
      label: { color: readableOn(color(split)) },
    })),
    // A thin gap between the parts of a bar.
    itemStyle: { borderColor: SURFACE, borderWidth: 1 },
    // Amounts only where they fit; the saver's part is the total beside the bar.
    label: {
      show: labelled,
      position: 'inside',
      fontSize: 11,
      formatter: ({ value }) => (fits(value as number) ? shekels(value as number) : ''),
    },
  })
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
        data: splits.map(({ id }) => id),
        axisTick: { show: false },
        axisLine: { show: false },
        triggerEvent: true,
        axisLabel: {
          // Above the bar, from where it starts (see the fee breakdown).
          inside: true,
          margin: 0,
          padding: [0, 0, 2 * NAME_ABOVE, 0],
          color: TEXT,
          fontSize: NAME_SIZE,
          interval: 0,
          formatter: (_id: string, index: number) => {
            const split = splits[index]
            const dot = split.hollow ? '◯' : '●'
            const style = pinned.has(split.id) ? `pinned${index}` : 'name'
            const name = fitName(split.label, room - measure(`${dot} `), measure)
            return `{dot${index}|${dot}} {${style}|${name}}`
          },
          rich: {
            ...Object.fromEntries(
              splits.flatMap((split, index) => [
                [`dot${index}`, { color: split.color }],
                [
                  `pinned${index}`,
                  { color: TEXT, backgroundColor: `${split.color}40`, borderRadius: 4, padding: [2, 5] },
                ],
              ]),
            ),
            name: { color: TEXT },
          },
        },
      },
      {
        // What the saver keeps, level with its bar.
        type: 'category',
        inverse: true,
        position: 'right',
        data: splits.map(({ yours }) => shekels(yours)),
        axisTick: { show: false },
        axisLine: { show: false },
        axisLabel: {
          padding: [2 * BAR_BELOW, 0, 0, 0],
          formatter: (total: string) => `{total|${total}}`,
          rich: { total: { color: TEXT, fontWeight: 'bold' } },
        },
      },
    ],
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      // Each part as it is: a bank that pays more than the rate keeps less
      // than nothing, which its bar can't show.
      formatter: (params) => {
        const index = (Array.isArray(params) ? params[0] : params).dataIndex
        const split = splits[index]
        if (!split) return ''
        const line = (color: string, name: string, amount: number) =>
          `<div><span style="display:inline-block;width:9px;height:9px;border-radius:2px;background:${color};margin-inline-end:6px"></span>${name}: <b>${shekels(amount)}</b></div>`
        return [
          `<div style="margin-bottom:4px">${split.label}</div>`,
          line(split.color, t.yoursPart, split.yours),
          line(KEPT_COLOR, t.keptPart, split.kept),
          line(TAX_COLOR, t.taxPart, split.tax),
        ].join('')
      },
    },
    series: [
      {
        // Around each pinned or hovered place, from its name to its total,
        // an outline in its color, as in the fee breakdown: pinned ones are
        // drawn here, a hovered one highlighted (see the component). Clear
        // inside, but not empty, so a click anywhere on the row picks it.
        type: 'custom',
        tooltip: { show: false },
        data: splits.map((_, index) => [0, index]),
        renderItem: (_params, api) => {
          const index = api.value(1) as number
          const { color, id } = splits[index]
          const [, middle] = api.coord([0, index])
          const [, height] = api.size!([0, 1]) as number[]
          return {
            type: 'rect',
            shape: { x: 1, y: middle - height / 2 + 2, width: api.getWidth() - 2, height: height - 4, r: 6 },
            style: { fill: 'transparent', stroke: pinned.has(id) ? color : 'transparent', lineWidth: 1.5 },
            emphasis: { style: { stroke: color } },
          }
        },
      },
      {
        // The empty slot above each bar, where its name goes.
        type: 'bar',
        barWidth: NAME_LINE,
        data: splits.map(() => 0),
        silent: true,
        tooltip: { show: false },
      },
      {
        ...part(
          t.yoursPart,
          ({ yours }) => yours,
          ({ color }) => color,
          false,
        ),
        // The Bank of Israel's rate on the money.
        markLine: {
          silent: true,
          symbol: 'none',
          data: [{ xAxis: atTheRate }],
          lineStyle: { color: TEXT, type: 'dashed', width: 1 },
          label: { show: false },
        },
      },
      part(
        t.keptPart,
        ({ kept }) => Math.max(kept, 0),
        () => KEPT_COLOR,
      ),
      part(
        t.taxPart,
        ({ tax }) => tax,
        () => TAX_COLOR,
      ),
    ],
  }
}
