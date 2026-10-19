// The donut's options: where every shekel goes at the end (deposit, gain
// kept, fees, tax, and the product's own cost as its own slice). Pure, so it
// is tested without a browser.
//
// STUB (package J fills it): the signature and a minimal valid option.
// Rules (REDESIGN.md 6.3): labels short (הפקדה / רווח / עמלות / מס / הקרן
// עצמה) and `overflow: 'none'`; a slice under 0.5% is listed under the donut
// instead of labelled on it; each part's colour is the same everywhere
// (`colors.ts`); every data item has an id.

import type { PieSeriesOption } from 'echarts/charts'
import type { BreakdownData } from './core/core'
import type { ChartOption } from './echarts.svelte'

export interface DonutView {
  breakdown: BreakdownData
  touch: boolean
}

export function donutOption({ breakdown }: DonutView): ChartOption {
  const series: PieSeriesOption = {
    id: 'donut',
    type: 'pie',
    radius: ['55%', '80%'],
    data: [
      { id: 'putIn', name: 'putIn', value: breakdown.putIn },
      { id: 'fees', name: 'fees', value: breakdown.fees },
      { id: 'keptByThePlace', name: 'keptByThePlace', value: breakdown.keptByThePlace },
      { id: 'tax', name: 'tax', value: breakdown.tax },
    ],
  }
  return { series: [series] }
}
