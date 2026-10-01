// The growth chart's options: what each plan's line looks like, from what's
// shown and pinned. Pure, so it can be tested without a browser; the
// component wires it to ECharts and to the app's state.

import type { LineSeriesOption } from 'echarts/charts'
import type { ChartOption } from './echarts.svelte'
import { compactShekels, elapsed, readableOn, shekels } from './format'
import { endLabel, labelLayout } from './end-label'
import { t } from './text'

// The page's colors (see app.css).
const WEAK = '#8e95a5'
const GRID = 'rgba(255, 255, 255, 0.07)'

/** The series id of the no-fee line, which can't be pinned or hovered. */
export const NO_FEES = 'no fees'

/** One plan's line: its values at the start and at the end of each month. */
export interface Line {
  id: string
  label: string
  color: string
  /** Its row's number in the table. */
  rank: number
  /** Your own plans are dotted, in their original's color. */
  dotted: boolean
  values: number[]
}

export interface GrowthView {
  lines: Line[]
  /** The no-fee line, or none in the "lost to fees" view, where it would be flat. */
  noFees: number[] | null
  /** Least first: the "lost to fees" view. */
  lost: boolean
  pinned: ReadonlySet<string>
  /** Phones and tablets: pinching there zooms the page, not the chart, so
   * they get a slider to zoom and move with instead of the wheel. */
  touch: boolean
  /** Time in months rather than years: the short term's. */
  months?: boolean
  /** The no-fee line's name, if not "No fees". */
  noFeesName?: string
}

/** [time, value] points, a month apart, from index 0 (the start): time in
 * years, or in months. */
const pointsIn = (months: boolean) => (values: number[]) =>
  values.map((value, month) => [months ? month : month / 12, value])

/** Which months a pinned line shows its value at: every year, or with time
 * in months, about eight times along it. */
const labelEvery = (view: GrowthView, count: number) =>
  view.months ? Math.max(1, Math.ceil((count - 1) / 8)) : 12

const compactLabel = ({ value }: { value: unknown }) => compactShekels((value as number[])[1])

// Pinned plans are styled through the options. A hovered plan isn't: it's
// emphasized with ECharts' highlight action (see the component's `setup`),
// because changing the options redraws the lines, which ends the hover.
function lineSeries(line: Line, view: GrowthView): LineSeriesOption {
  const pinned = view.pinned.has(line.id)
  const faded = view.pinned.size > 0 && !pinned
  // Pinned plans, in table order: every other one gets its yearly values
  // below its line rather than above, so they collide less.
  const pinnedOrder = view.lines.map(({ id }) => id).filter((id) => view.pinned.has(id))
  const points = pointsIn(view.months ?? false)
  const every = labelEvery(view, line.values.length)
  return {
    id: line.id,
    name: line.label,
    type: 'line',
    // Pinned plans show a dot and their value at each whole year; ones that
    // would overlap are hidden, so fewer show when zoomed out. Other plans
    // get plain points, which ECharts copies and compares much faster.
    data: pinned
      ? points(line.values).map((point, index, all) => {
          const yearly = index % every === 0
          // Not at the last point, where the end label shows the same value:
          // the two would collide, and the end label could be hidden.
          const last = index === all.length - 1
          return { value: point, symbol: yearly ? 'emptyCircle' : 'none', label: { show: yearly && !last } }
        })
      : points(line.values),
    color: line.color,
    lineStyle: { width: pinned ? 3.5 : 2, opacity: faded ? 0.4 : 1, type: line.dotted ? 'dotted' : 'solid' },
    itemStyle: { opacity: faded ? 0.4 : 1 },
    showSymbol: pinned,
    // Otherwise ECharts skips some of them on lines with many points.
    showAllSymbol: true,
    label: {
      position: pinnedOrder.indexOf(line.id) % 2 === 0 ? 'top' : 'bottom',
      formatter: compactLabel,
      backgroundColor: line.color,
      color: readableOn(line.color),
      padding: [1, 4],
      borderRadius: 3,
    },
    // Its number and the value where the line leaves the view.
    endLabel: endLabel(line, faded, compactLabel),
    // On a phone, where twenty labels can't fit along a line, ones that
    // would overlap are hidden, and more show as the years are zoomed into.
    labelLayout: labelLayout(!view.touch),
    emphasis: { focus: 'series' },
    // Hovering or clicking the line itself, not only its points.
    triggerEvent: 'line',
  }
}

export function growthOption(view: GrowthView): ChartOption {
  const series: LineSeriesOption[] = view.lines.map((line) => lineSeries(line, view))
  const months = view.months ?? false
  if (view.noFees) {
    series.unshift({
      id: NO_FEES,
      name: view.noFeesName ?? t.noFees,
      type: 'line',
      data: pointsIn(months)(view.noFees),
      color: WEAK,
      lineStyle: { type: 'dashed', width: 1.5 },
      showSymbol: false,
      endLabel: { show: true, valueAnimation: true, formatter: compactLabel },
      labelLayout: { moveOverlap: 'shiftY' },
      silent: true,
    })
  }
  return {
    grid: { left: 16, right: 90, top: 24, bottom: view.touch ? 88 : 32 },
    xAxis: {
      type: 'value',
      name: months ? t.monthsAxis : t.years,
      nameLocation: 'middle',
      nameGap: 26,
      minInterval: 1,
      // Zoomed in, the axis ends at fractions of a year; label whole years
      // (or months) only.
      axisLabel: { formatter: (time: number) => (Number.isInteger(time) ? String(time) : ''), color: WEAK },
      nameTextStyle: { color: WEAK },
      axisLine: { lineStyle: { color: GRID } },
      splitLine: { show: false },
      // Also the tooltip's title.
      axisPointer: {
        label: {
          formatter: ({ value }) =>
            months ? t.afterMonths(Math.round(Number(value))) : elapsed(Number(value)),
        },
      },
    },
    // Fits the lines in view exactly (not rounded to a nice number), so it
    // changes smoothly while zooming; a little room above for the labels.
    yAxis: {
      type: 'value',
      min: ({ min }) => min,
      max: ({ min, max }) => max + (max - min) * 0.04,
      // Only the round values; the ends are wherever the lines are.
      axisLabel: { formatter: compactShekels, color: WEAK, showMinLabel: false, showMaxLabel: false },
      splitLine: { lineStyle: { color: GRID } },
    },
    // The wheel zooms the years and dragging moves along them; on touch
    // screens, the slider's handles and middle do. Points out of view are
    // dropped, so the value axis fits what's in view. Redrawn about every
    // frame (the default is every 100 ms).
    dataZoom: [
      view.touch
        ? {
            type: 'slider',
            xAxisIndex: 0,
            filterMode: 'filter',
            minValueSpan: 1,
            throttle: 16,
            bottom: 8,
            height: 32,
            left: 24,
            right: 90,
            // The years without the plans' lines in it: they're all alike.
            showDataShadow: false,
            brushSelect: false,
            labelFormatter: (time: number) => `${Math.round(time)}`,
            textStyle: { color: WEAK },
            borderColor: GRID,
            fillerColor: 'rgba(123, 155, 255, 0.2)',
            handleSize: '120%',
            handleStyle: { color: '#7b9bff', borderColor: '#7b9bff' },
            moveHandleSize: 0,
          }
        : { type: 'inside', xAxisIndex: 0, filterMode: 'filter', minValueSpan: 1, throttle: 16 },
    ],
    tooltip: {
      trigger: 'axis',
      // Best first: most value, or least lost.
      order: view.lost ? 'valueAsc' : 'valueDesc',
      valueFormatter: (value) => shekels(value as number),
    },
    series,
  }
}
