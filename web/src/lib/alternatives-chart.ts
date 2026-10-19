// The alternatives chart's options: one line per kind (what the same money
// does in each), the history's band behind the stock fund's line, and end
// labels for three lines only. Pure, so it is tested without a browser; the
// component wires it to ECharts and to the state.
//
// STUB (package J fills it): the signature and a minimal valid option.
// Rules that bind (REDESIGN.md 6.3, CLAUDE.md "Known traps"): every series has
// an `id` (the kind's `kindKey`); the band is two `stack`ed series, the lower
// one's line transparent and the upper one's `areaStyle`; labels never wrap
// or are cut (`fitName`); only the recommended kind, the stock fund and the
// current account get end labels; hover goes through `dispatchAction`.

import type { LineSeriesOption } from 'echarts/charts'
import type { AlternativesData } from './core/core'
import type { ChartOption } from './echarts.svelte'
import { kindKey } from './kinds'

export interface AlternativesView {
  data: AlternativesData
  /** The kinds (as `kindKey`) pinned or hovered in the rows under the chart. */
  highlighted: ReadonlySet<string>
  /** A colour for each kind, by `kindKey`. */
  colors: ReadonlyMap<string, string>
  touch: boolean
}

export function alternativesOption(view: AlternativesView): ChartOption {
  const { data, colors } = view
  const lines: LineSeriesOption[] = data.series.map((line) => ({
    id: kindKey(line.kind),
    name: line.name,
    type: 'line',
    showSymbol: false,
    lineStyle: { color: colors.get(kindKey(line.kind)) },
    data: line.valueByMonth.map((value, month) => [month, value]),
  }))
  return {
    xAxis: { type: 'value', min: 0, max: data.months },
    yAxis: { type: 'value' },
    series: lines,
  }
}
