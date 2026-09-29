// The growth chart's options: what each plan's line looks like, from what's
// shown and pinned. Pure, so it can be tested without a browser; the
// component wires it to ECharts and to the app's state.

import type { LineSeriesOption } from 'echarts/charts'
import type { ChartOption } from './echarts.svelte'
import { compactShekels, elapsed, readableOn, shekels } from './format'
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
}

/** [years, value] points, a month apart, from index 0 (the start). */
const points = (values: number[]) => values.map((value, month) => [month / 12, value])

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
  return {
    id: line.id,
    name: line.label,
    type: 'line',
    // Pinned plans show a dot and their value at each whole year; ones that
    // would overlap are hidden, so fewer show when zoomed out. Other plans
    // get plain points, which ECharts copies and compares much faster.
    data: pinned
      ? points(line.values).map((point) => {
          const yearly = Number.isInteger(point[0])
          return { value: point, symbol: yearly ? 'emptyCircle' : 'none', label: { show: yearly } }
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
    // The value where the line leaves the view, in the plan's color.
    endLabel: {
      show: true,
      formatter: compactLabel,
      color: faded ? undefined : line.color,
      opacity: faded ? 0.6 : 1,
    },
    // Labels that would overlap are moved apart; on a phone, where twenty
    // of them can't fit along a line, they're hidden instead, and more show
    // as the years are zoomed into.
    labelLayout: view.touch ? { hideOverlap: true } : { moveOverlap: 'shiftY', hideOverlap: true },
    emphasis: { focus: 'series' },
    // Hovering or clicking the line itself, not only its points.
    triggerEvent: 'line',
  }
}

export function growthOption(view: GrowthView): ChartOption {
  const series: LineSeriesOption[] = view.lines.map((line) => lineSeries(line, view))
  if (view.noFees) {
    series.unshift({
      id: NO_FEES,
      name: t.noFees,
      type: 'line',
      data: points(view.noFees),
      color: WEAK,
      lineStyle: { type: 'dashed', width: 1.5 },
      showSymbol: false,
      endLabel: { show: true, formatter: compactLabel },
      labelLayout: { moveOverlap: 'shiftY' },
      silent: true,
    })
  }
  return {
    grid: { left: 16, right: 90, top: 24, bottom: view.touch ? 88 : 32 },
    xAxis: {
      type: 'value',
      name: t.years,
      nameLocation: 'middle',
      nameGap: 26,
      minInterval: 1,
      // Zoomed in, the axis ends at fractions of a year; label whole years only.
      axisLabel: { formatter: (year: number) => (Number.isInteger(year) ? String(year) : ''), color: WEAK },
      nameTextStyle: { color: WEAK },
      axisLine: { lineStyle: { color: GRID } },
      splitLine: { show: false },
      // Also the tooltip's title.
      axisPointer: { label: { formatter: ({ value }) => elapsed(Number(value)) } },
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
            labelFormatter: (year: number) => `${Math.round(year)}`,
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
