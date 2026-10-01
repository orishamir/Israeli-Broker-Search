// Where the interest goes, as the short-term calculator draws it: for each
// place, the interest the Bank of Israel's rate would pay on the money, as
// one bar split three ways: what the saver keeps, what the place keeps, and
// the tax. Pure, so it can be tested without a browser; the component wires
// it to ECharts and to the state. The rows are laid out as the fee
// breakdown's are: each place's name above its bar.

import type { BarSeriesOption } from 'echarts/charts'
import type { ChartOption } from './echarts.svelte'
import { BAR, BAR_GAP, fitName, MARGIN, NAME_LINE, NAME_SIZE, TOTAL_GAP } from './fee-breakdown'
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
  /** A text's width in pixels, in the names' font (see `fitName`), or in
   * bold, the totals'. */
  measure: (text: string, bold?: boolean) => number
}

/** A bar's length: past the line where a bank pays more than the rate. */
export const lengthOf = ({ yours, kept, tax }: Split) => yours + tax + Math.max(kept, 0)

export function splitOption({ splits, atTheRate, pinned, width, measure }: SplitView): ChartOption {
  const largest = Math.max(atTheRate, ...splits.map(lengthOf))
  const totals = splits.map(({ yours }) => shekels(yours))
  // The totals go right of the bars.
  const right =
    MARGIN.right + Math.ceil(Math.max(0, ...totals.map((total) => measure(total, true)))) + TOTAL_GAP
  const pixelsPerShekel = Math.max(width - MARGIN.left - right, 0) / largest
  const fits = (amount: number) => amount * pixelsPerShekel > 44
  // The box, less its margins and a pinned name's padding.
  const room = width - MARGIN.left - MARGIN.right - 2 * 5
  const opacity = (id: string) => (pinned.size === 0 || pinned.has(id) ? 1 : 0.6)
  const part = (
    id: string,
    name: string,
    value: (split: Split) => number,
    color: (split: Split) => string,
    labelled = true,
  ): BarSeriesOption => ({
    id,
    name,
    type: 'bar',
    stack: 'interest',
    barWidth: BAR,
    barGap: BAR_GAP,
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
    grid: { ...MARGIN, right },
    xAxis: {
      type: 'value',
      max: largest,
      axisLabel: { formatter: compactShekels, color: WEAK, showMaxLabel: false, hideOverlap: true },
      splitLine: { lineStyle: { color: GRID } },
    },
    yAxis: {
      type: 'category',
      inverse: true,
      // Ids, not names, which can repeat.
      data: splits.map(({ id }) => id),
      axisTick: { show: false },
      axisLine: { show: false },
      // Each place's name and total ride on its row (see the series), so
      // they move with it when the order changes, as in the fee breakdown.
      axisLabel: { show: false },
    },
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
    // Every series has an id, as in the fee breakdown: one without is
    // drawn again from nothing at every change.
    series: [
      {
        // Around each pinned or hovered place, from its name to its total,
        // an outline in its color, as in the fee breakdown: pinned ones are
        // drawn here, a hovered one highlighted (see the component). Clear
        // inside, but not empty, so a click anywhere on the row picks it.
        id: 'outline',
        type: 'custom',
        tooltip: { show: false },
        // Named by place, so that each moves with its row.
        data: splits.map(({ id }, index) => ({ name: id, value: [0, index] })),
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
        // The empty slot above each bar, which carries the place's name.
        id: 'names',
        type: 'bar',
        barWidth: NAME_LINE,
        data: splits.map(() => 0),
        tooltip: { show: false },
        emphasis: { disabled: true },
        label: {
          show: true,
          // From where the bar starts.
          position: 'right',
          distance: 0,
          color: TEXT,
          fontSize: NAME_SIZE,
          // A dot in the place's color (hollow for your own deposits), and
          // the name, on a tint of that color when pinned.
          formatter: ({ dataIndex }) => {
            const split = splits[dataIndex]
            const dot = split.hollow ? '◯' : '●'
            const style = pinned.has(split.id) ? `pinned${dataIndex}` : 'name'
            const name = fitName(split.label, room - measure(`${dot} `), measure)
            return `{dot${dataIndex}|${dot}} {${style}|${name}}`
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
        ...part(
          'yours',
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
        'kept',
        t.keptPart,
        ({ kept }) => Math.max(kept, 0),
        () => KEPT_COLOR,
      ),
      part(
        'tax',
        t.taxPart,
        ({ tax }) => tax,
        () => TAX_COLOR,
      ),
      {
        // The rest of each row, clear, with what the saver keeps past its end.
        id: 'totals',
        type: 'bar',
        stack: 'interest',
        barWidth: BAR,
        barGap: BAR_GAP,
        data: splits.map((split) => largest - lengthOf(split)),
        itemStyle: { color: 'transparent' },
        tooltip: { show: false },
        emphasis: { disabled: true },
        label: {
          show: true,
          position: 'right',
          distance: TOTAL_GAP,
          formatter: ({ dataIndex }) => `{total|${totals[dataIndex]}}`,
          rich: { total: { color: TEXT, fontWeight: 'bold' } },
        },
      },
    ],
  }
}
