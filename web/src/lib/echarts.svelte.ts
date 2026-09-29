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

/** How long a change glides, in ms. */
const GLIDE = 180
/** Changes closer together than this (typing, dragging a slider) snap into
 * place instead: each glide would restart on the next change, and a phone
 * spent most of every keystroke redrawing mid-glide. */
const QUICK_SUCCESSION = 250

/** What every chart shares, on top of ECharts' dark theme: the page's font
 * and colors, quick drawing (none if the user asked for less motion), and a
 * tooltip that keeps up with the pointer. */
const BASE: ChartOption = {
  backgroundColor: 'transparent',
  textStyle: { fontFamily: getComputedStyle(document.documentElement).fontFamily },
  animation: !reducedMotion,
  animationDuration: 500,
  animationEasingUpdate: 'cubicOut',
  tooltip: {
    // In seconds; the default 0.4 trails behind the pointer.
    transitionDuration: 0.1,
    backgroundColor: 'rgba(29, 33, 44, 0.92)',
    borderColor: 'rgba(255, 255, 255, 0.16)',
    textStyle: { color: '#e6e8ee' },
    extraCssText:
      'backdrop-filter: blur(8px); border-radius: 10px; box-shadow: 0 12px 40px rgb(0 0 0 / 0.5);',
  },
}

/**
 * Puts a chart on the element: `<div {@attach chart(() => option, setup)}>`.
 * The chart updates whenever state read by `option` changes, just after the
 * next paint, so that what the user typed and the table show first and the
 * chart follows a frame later. It follows the element's size. While the
 * element has no size (its view is hidden) nothing is computed or drawn, and
 * the chart is only made once it first has one.
 * `setup` can add event handlers; what it returns is called on removal.
 */
export function chart(
  option: () => ChartOption,
  setup?: (chart: ECharts) => (() => void) | void,
): Attachment<HTMLElement> {
  return (element) => {
    let instance: ECharts | undefined
    let stopSetup: (() => void) | undefined
    let lastChange = -Infinity
    let lastDrawn: ChartOption | undefined
    let size = { width: 0, height: 0 }
    let frame = 0
    let timer: ReturnType<typeof setTimeout> | undefined
    /** Whether the element has a size. */
    let shown = $state(false)
    /** Computed only when read: not while hidden, and not again when shown
     * again with nothing changed. */
    const current = $derived.by(option)

    const draw = (next: ChartOption) => {
      const now = performance.now()
      const glide = now - lastChange < QUICK_SUCCESSION ? 0 : GLIDE
      lastChange = now
      lastDrawn = next
      // Replace, not merge, the series: unticked plans must disappear.
      instance!.setOption({ ...next, animationDurationUpdate: glide }, { replaceMerge: ['series'] })
    }
    const cancelDraw = () => {
      cancelAnimationFrame(frame)
      clearTimeout(timer)
    }

    $effect(() => {
      if (!shown) return
      const next = current
      if (next === lastDrawn) return
      // A timer set from an animation frame runs once that frame is
      // painted. A change before then replaces the one waiting.
      cancelDraw()
      frame = requestAnimationFrame(() => {
        timer = setTimeout(() => draw(next))
      })
    })

    const resizing = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      if (width === 0 || height === 0) {
        shown = false
        return
      }
      if (!instance) {
        instance = echarts.init(element, 'dark')
        // Options are merged, so later ones keep these unless they override them.
        instance.setOption(BASE)
        // `setup` may make effects, which need a scope that outlives this callback.
        stopSetup = $effect.root(() => {
          const cleanup = setup?.(instance!)
          return () => cleanup?.()
        })
      } else if (width !== size.width || height !== size.height) {
        instance.resize()
      }
      size = { width, height }
      shown = true
    })
    resizing.observe(element)

    return () => {
      cancelDraw()
      stopSetup?.()
      resizing.disconnect()
      instance?.dispose()
    }
  }
}
