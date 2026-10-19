// The alternatives chart's options: one line per kind (what the same money
// does in each), the history's band behind the stock fund's line, and end
// labels for three lines only. Pure, so it is tested without a browser; the
// component wires it to ECharts and to the state.
//
// Rules that bind (REDESIGN.md 6.3, CLAUDE.md "Known traps"): every series
// has an `id` (the kind's `kindKey`); the band is two `stack`ed series, the
// lower one's line transparent and the upper one's `areaStyle`; labels never
// wrap or are cut by ECharts (`fitName`, measured on a canvas); only the
// recommended kind, the stock fund and the current account get end labels;
// hover goes through `dispatchAction` (see `linkLines`), never an option.

import type { LineSeriesOption } from 'echarts/charts'
import type { AlternativesData, KindSeriesData } from './core/core'
import type { ChartOption } from './echarts.svelte'
import { BLUR, FADED_LABEL, FADED_LINE } from './end-label'
import { lastMonthMarker } from './end-label'
import { fitName } from './fee-breakdown'
import { compactShekels, elapsed, shekels } from './format'
import { kindKey } from './kinds'
import { t } from './text'

// The page's colors (see app.css).
const WEAK = '#8e95a5'
const GRID = 'rgba(255, 255, 255, 0.07)'

export const BAND_LOW = 'band:low'
export const BAND_HIGH = 'band:high'
/** The two series that draw the band: not rows, never hovered or pinned. */
export const BAND_IDS: readonly string[] = [BAND_LOW, BAND_HIGH]

/** How far above the highest line the chart reaches. Over a long horizon the
 * best stretch of the history ends several times higher than every line,
 * which would squeeze them all into a strip at the bottom; the band is cut
 * off at the top instead, and the tip says so. */
export const HEADROOM = 1.5

/** The room the end labels have, to the right of the plot. */
export const LABEL_ROOM = 76

export interface AlternativesView {
  data: AlternativesData
  /** The kinds (as `kindKey`) pinned in the chart. */
  highlighted: ReadonlySet<string>
  /** A colour for each kind, by `kindKey`. */
  colors: ReadonlyMap<string, string>
  touch: boolean
  /** A text's width in pixels, in the label's font (see `fitName`). */
  measure?: (text: string) => number
}

/** The kind the band is drawn behind: the recommended one if it is a way of
 * holding the index, else the first such way, as drawn in the stock market. */
export function stockLine(series: readonly KindSeriesData[]): KindSeriesData | undefined {
  const isWay = (line: KindSeriesData) => typeof line.kind === 'object' && 'Way' in line.kind
  return series.find((line) => line.recommended && isWay(line)) ?? series.find(isWay)
}

/** The word at the end of a line, for the lines that have one: what fits,
 * then the stock fund, then the current account. */
function endWord(line: KindSeriesData, stock: KindSeriesData | undefined): string | undefined {
  if (line.recommended) return t.endRecommended
  if (line === stock) return t.endStock
  if (line.kind === 'CurrentAccount') return t.endCash
  return undefined
}

/** Under this many months the time is drawn in months: a year's axis would
 * have no tick to read a four-month cushion by. */
export const MONTHS_BELOW = 24

const points = (values: readonly number[], inMonths: boolean) =>
  values.map((value, month) => [inMonths ? month : month / 12, value])

/** A time given in months: "אחרי 4 חודשים", "אחרי שנה". */
const timeText = (months: number) => elapsed(Math.round(months) / 12)

export function alternativesOption(view: AlternativesView): ChartOption {
  const { data, colors, highlighted } = view
  const measure = view.measure ?? ((text: string) => text.length * 7)
  const stock = stockLine(data.series)
  const inMonths = data.months < MONTHS_BELOW
  const anyPinned = highlighted.size > 0
  const stockColor = stock ? (colors.get(kindKey(stock.kind)) ?? WEAK) : WEAK

  const lines: LineSeriesOption[] = data.series.map((line) => {
    const id = kindKey(line.kind)
    const color = colors.get(id) ?? WEAK
    const pinned = highlighted.has(id)
    const faded = anyPinned && !pinned
    const word = endWord(line, stock)
    return {
      id,
      name: line.name,
      type: 'line',
      data: points(line.valueByMonth, inMonths),
      color,
      showSymbol: false,
      lineStyle: {
        width: pinned ? 3.5 : line.recommended ? 3 : 2,
        opacity: faded ? FADED_LINE : 1,
        type: line.kind === 'CurrentAccount' ? 'dashed' : 'solid',
      },
      itemStyle: { opacity: faded ? FADED_LINE : 1 },
      // The recommended line is drawn over the rest.
      z: line.recommended ? 5 : 3,
      endLabel: word
        ? {
            show: true,
            formatter: fitName(word, LABEL_ROOM - 8, measure),
            color,
            opacity: faded ? FADED_LABEL : 1,
            minMargin: 3,
          }
        : { show: false },
      // Only a few labels, but they are near each other: move them apart.
      labelLayout: { moveOverlap: 'shiftY' },
      emphasis: { focus: 'series' },
      ...BLUR,
      triggerEvent: 'line',
    }
  })

  const band: LineSeriesOption[] = data.band
    ? [
        {
          id: BAND_LOW,
          name: t.bandName,
          type: 'line',
          stack: 'band',
          data: points(data.band.worst, inMonths),
          showSymbol: false,
          // The lower edge is only the floor the upper series stacks on.
          lineStyle: { opacity: 0 },
          silent: true,
          z: 1,
        },
        {
          id: BAND_HIGH,
          name: t.bandName,
          type: 'line',
          stack: 'band',
          // What the best stretch adds above the worst, so the two stack to the best.
          data: points(
            data.band.best.map((best, month) => best - data.band!.worst[month]),
            inMonths,
          ),
          showSymbol: false,
          lineStyle: { opacity: 0 },
          areaStyle: { color: stockColor, opacity: 0.14 },
          silent: true,
          z: 1,
        },
      ]
    : []

  const ends = data.series.map((line) => line.valueByMonth.at(-1) ?? 0)
  const top = Math.max(...ends, ...data.series.map((line) => line.valueByMonth[0] ?? 0)) * HEADROOM

  return {
    grid: { left: 16, right: LABEL_ROOM, top: 16, bottom: 32 },
    xAxis: {
      type: 'value',
      name: inMonths ? t.chartMonths : t.chartYears,
      nameLocation: 'middle',
      nameGap: 24,
      min: 0,
      max: inMonths ? data.months : data.months / 12,
      minInterval: 1,
      axisLabel: { formatter: (time: number) => (Number.isInteger(time) ? String(time) : ''), color: WEAK },
      nameTextStyle: { color: WEAK },
      axisLine: { lineStyle: { color: GRID } },
      splitLine: { show: false },
      axisPointer: { label: { formatter: ({ value }) => timeText(Number(value) * (inMonths ? 1 : 12)) } },
    },
    yAxis: {
      type: 'value',
      min: 0,
      max: top,
      axisLabel: { formatter: compactShekels, color: WEAK, showMaxLabel: false },
      splitLine: { lineStyle: { color: GRID } },
    },
    tooltip: {
      trigger: 'axis',
      // Best first; the band's two series are one row of their own.
      formatter: (params) => {
        const rows = (Array.isArray(params) ? params : [params]) as unknown as {
          seriesId: string
          seriesName: string
          color: string
          value: [number, number]
        }[]
        const month = Math.round(((rows[0]?.value?.[0] ?? 0) as number) * (inMonths ? 1 : 12))
        const at = (values: readonly number[]) => values[Math.min(month, values.length - 1)]
        const dot = (color: string) =>
          `<span style="display:inline-block;width:9px;height:9px;border-radius:50%;background:${color};margin-inline-end:6px"></span>`
        const body = data.series
          .toSorted((a, b) => at(b.valueByMonth) - at(a.valueByMonth))
          .map((line) => {
            const color = colors.get(kindKey(line.kind)) ?? WEAK
            return `<div style="white-space:normal;max-width:260px">${dot(color)}${line.name}: <b>${shekels(at(line.valueByMonth))}</b></div>`
          })
        if (data.band)
          body.push(
            `<div style="color:${WEAK}">${t.rangeInTooltip}: ${shekels(at(data.band.worst))} – ${shekels(at(data.band.best))}</div>`,
          )
        // The last month is after selling and tax: a drop that is not the market's.
        const last = month >= data.months
        if (last) body.push(`<div style="color:${WEAK}">${t.lastPointAfterTax}</div>`)
        return `<div>${timeText(month)}</div>${body.join('')}`
      },
    },
    series: [...band, ...lines, lastMonthMarker(inMonths ? data.months : data.months / 12, measure)],
  }
}
