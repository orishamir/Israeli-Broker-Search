// ECharts, with only the parts the app uses (so the rest isn't bundled), and
// a Svelte attachment to put a chart on an element.

import type { Attachment } from 'svelte/attachments'
import { BarChart, CustomChart, LineChart } from 'echarts/charts'
import {
  DataZoomInsideComponent,
  DataZoomSliderComponent,
  GridComponent,
  TooltipComponent,
} from 'echarts/components'
import * as echarts from 'echarts/core'
import type { ComposeOption, ECharts } from 'echarts/core'
import type { BarSeriesOption, CustomSeriesOption, LineSeriesOption } from 'echarts/charts'
import type { DataZoomComponentOption, GridComponentOption, TooltipComponentOption } from 'echarts/components'
import { LabelLayout } from 'echarts/features'
import { CanvasRenderer } from 'echarts/renderers'
import { reducedMotion } from './motion'

echarts.use([
  LineChart,
  BarChart,
  CustomChart,
  GridComponent,
  TooltipComponent,
  DataZoomInsideComponent,
  DataZoomSliderComponent,
  LabelLayout,
  CanvasRenderer,
])

/** The options of the parts registered above, so TypeScript checks them. */
export type ChartOption = ComposeOption<
  | LineSeriesOption
  | BarSeriesOption
  | CustomSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | DataZoomComponentOption
>

/** What every chart shares, on top of ECharts' dark theme: the page's font
 * and colors, and quick, smooth updates (none if the user asked for less
 * motion). */
const BASE: ChartOption = {
  backgroundColor: 'transparent',
  textStyle: { fontFamily: getComputedStyle(document.documentElement).fontFamily },
  animation: !reducedMotion,
  animationDurationUpdate: 400,
  animationEasingUpdate: 'cubicOut',
  tooltip: {
    backgroundColor: 'rgba(29, 33, 44, 0.92)',
    borderColor: 'rgba(255, 255, 255, 0.16)',
    textStyle: { color: '#e6e8ee' },
    extraCssText:
      'backdrop-filter: blur(8px); border-radius: 10px; box-shadow: 0 12px 40px rgb(0 0 0 / 0.5);',
  },
}

/**
 * Puts a chart on the element: `<div {@attach chart(() => option, setup)}>`.
 * The chart updates whenever state read by `option` changes, and follows the
 * element's size.
 * `setup` can add event handlers; what it returns is called on removal.
 */
export function chart(
  option: () => ChartOption,
  setup?: (chart: ECharts) => (() => void) | void,
): Attachment<HTMLElement> {
  return (element) => {
    const instance = echarts.init(element, 'dark')
    // Options are merged, so later ones keep these unless they override them.
    instance.setOption(BASE)
    const cleanup = setup?.(instance)

    $effect(() => {
      // Replace, not merge, the series: unticked plans must disappear.
      instance.setOption(option(), { replaceMerge: ['series'] })
    })

    const resizing = new ResizeObserver(() => instance.resize())
    resizing.observe(element)

    return () => {
      cleanup?.()
      resizing.disconnect()
      instance.dispose()
    }
  }
}
