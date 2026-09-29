// The chart by deposit: each plan's yearly cost over a range of deposits,
// both axes logarithmic, with a marker at the user's own deposit. Where two
// lines cross, the plans' ranking flips. Pure, so it can be tested without
// a browser; the component wires it to ECharts and to the app's state.

import type { LineSeriesOption } from 'echarts/charts'
import type { Swept } from './core/core'
import type { ChartOption } from './echarts.svelte'
import { compactPercent, compactShekels, percent, readableOn, shekels } from './format'

// The page's colors (see app.css).
const WEAK = '#8e95a5'
const GRID = 'rgba(255, 255, 255, 0.07)'

/** The series id of the marker at the user's deposit, which can't be pinned
 * or hovered. */
export const YOURS = 'yours'

/** The least cost drawn: a logarithmic axis has no zero, and a cost below
 * this a year is as good as none. */
export const FLOOR = 0.01

/** One plan's line: its yearly cost, in percent, at each of the amounts. */
export interface CostLine {
  id: string
  label: string
  color: string
  /** Your own plans are dotted, in their original's color. */
  dotted: boolean
  costs: number[]
}

export interface CrossoverView {
  /** The deposits tried, ₪, smallest first. */
  amounts: number[]
  lines: CostLine[]
  swept: Swept
  /** The user's own deposit, to mark; none while its field is empty or 0. */
  yours: number | null
  pinned: ReadonlySet<string>
}

/** "₪2,000 a month", "₪200,000 at once" */
export const depositText = (amount: number, swept: Swept): string =>
  `${shekels(amount)} ${swept === 'Monthly' ? 'a month' : 'at once'}`

const compactLabel = ({ value }: { value: unknown }) => percent((value as number[])[1])

// Pinned plans are styled through the options; a hovered one is emphasized
// with ECharts' highlight action, as in the growth chart.
function lineSeries(line: CostLine, view: CrossoverView): LineSeriesOption {
  const pinned = view.pinned.has(line.id)
  const faded = view.pinned.size > 0 && !pinned
  return {
    id: line.id,
    name: line.label,
    type: 'line',
    data: view.amounts.map((amount, index) => [amount, Math.max(line.costs[index], FLOOR)]),
    color: line.color,
    lineStyle: { width: pinned ? 3.5 : 2, opacity: faded ? 0.4 : 1, type: line.dotted ? 'dotted' : 'solid' },
    itemStyle: { opacity: faded ? 0.4 : 1 },
    symbol: 'emptyCircle',
    symbolSize: 6,
    showSymbol: pinned,
    showAllSymbol: true,
    label: {
      show: pinned,
      formatter: compactLabel,
      backgroundColor: line.color,
      color: readableOn(line.color),
      padding: [1, 4],
      borderRadius: 3,
    },
    // The cost at the largest deposit, in the plan's color.
    endLabel: {
      show: true,
      formatter: compactLabel,
      color: faded ? undefined : line.color,
      opacity: faded ? 0.6 : 1,
    },
    labelLayout: { moveOverlap: 'shiftY', hideOverlap: true },
    emphasis: { focus: 'series' },
    // Hovering or clicking the line itself, not only its points.
    triggerEvent: 'line',
  }
}

export function crossoverOption(view: CrossoverView): ChartOption {
  const series: LineSeriesOption[] = view.lines.map((line) => lineSeries(line, view))
  const first = view.amounts[0] ?? 1
  const last = view.amounts.at(-1) ?? first
  // The axis reaches the user's deposit even beyond the range tried.
  const yours = view.yours !== null && view.yours > 0 ? view.yours : null
  if (yours !== null) {
    series.push({
      id: YOURS,
      name: 'Your deposit',
      type: 'line',
      data: [],
      silent: true,
      markLine: {
        silent: true,
        symbol: 'none',
        animation: false,
        data: [{ xAxis: yours }],
        lineStyle: { type: 'dashed', color: WEAK, width: 1.5 },
        label: { formatter: `You: ${depositText(yours, view.swept)}`, position: 'insideEndTop', color: WEAK },
      },
    })
  }
  return {
    grid: { left: 16, right: 80, top: 24, bottom: 36, containLabel: true },
    xAxis: {
      type: 'log',
      logBase: 10,
      min: Math.min(first, yours ?? first),
      max: Math.max(last, yours ?? last),
      name: view.swept === 'Monthly' ? 'Deposit a month' : 'One-time deposit',
      nameLocation: 'middle',
      nameGap: 28,
      nameTextStyle: { color: WEAK },
      axisLabel: { formatter: compactShekels, color: WEAK },
      axisLine: { lineStyle: { color: GRID } },
      splitLine: { show: false },
      // Also the tooltip's title.
      axisPointer: { label: { formatter: ({ value }) => depositText(Number(value), view.swept) } },
    },
    yAxis: {
      type: 'log',
      logBase: 10,
      name: 'Yearly cost',
      nameTextStyle: { color: WEAK, align: 'left' },
      axisLabel: { formatter: compactPercent, color: WEAK },
      splitLine: { lineStyle: { color: GRID } },
    },
    tooltip: {
      trigger: 'axis',
      // Cheapest first.
      order: 'valueAsc',
      valueFormatter: (value) => percent(value as number),
    },
    series,
  }
}
