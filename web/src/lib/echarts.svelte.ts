// ECharts, with only the parts the app uses (so the rest isn't bundled), and
// a Svelte attachment to put a chart on an element.

import type { Attachment } from 'svelte/attachments'
import { BarChart, CustomChart, LineChart } from 'echarts/charts'
import {
  DataZoomInsideComponent,
  DataZoomSliderComponent,
  GridComponent,
  MarkLineComponent,
  TooltipComponent,
} from 'echarts/components'
import * as echarts from 'echarts/core'
import type { ComposeOption, ECharts } from 'echarts/core'
import type { BarSeriesOption, CustomSeriesOption, LineSeriesOption } from 'echarts/charts'
import type {
  DataZoomComponentOption,
  GridComponentOption,
  MarkLineComponentOption,
  TooltipComponentOption,
} from 'echarts/components'
import { LabelLayout } from 'echarts/features'
import { CanvasRenderer } from 'echarts/renderers'
import { drawAt, glideFor, settlesIn } from './glide'
import { lastTyped } from './typing'
import { duration, EASE_OUT, easeOut, reducedMotion, SETTLE } from './motion'

echarts.use([
  LineChart,
  BarChart,
  CustomChart,
  GridComponent,
  TooltipComponent,
  DataZoomInsideComponent,
  DataZoomSliderComponent,
  MarkLineComponent,
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
  | MarkLineComponentOption
>

/** What every chart shares, on top of ECharts' dark theme: the page's font
 * and colors, its curve for changes (no motion if the user asked for less),
 * and a tooltip that keeps up with the pointer and stays inside the chart. */
const BASE: ChartOption = {
  backgroundColor: 'transparent',
  textStyle: { fontFamily: getComputedStyle(document.documentElement).fontFamily },
  animation: !reducedMotion,
  animationDuration: 500,
  animationEasingUpdate: EASE_OUT,
  tooltip: {
    // On a phone the chart is narrower than twice the tooltip, so ECharts
    // put it past the chart's left edge: cut off by the screen, and on a
    // right-to-left page, where the page scrolls to anything on its left,
    // the whole page could then be scrolled sideways.
    confine: true,
    // In seconds; the default 0.4 trails behind the pointer.
    transitionDuration: 0.1,
    backgroundColor: 'rgba(29, 33, 44, 0.92)',
    borderColor: 'rgba(255, 255, 255, 0.16)',
    textStyle: { color: '#e6e8ee' },
    extraCssText:
      'backdrop-filter: blur(8px); border-radius: 10px; box-shadow: 0 12px 40px rgb(0 0 0 / 0.5);',
  },
}

/** ECharts never animates a line's end label (it turns that off as it makes
 * the label), so the label jumped to where its line was going while the line
 * glided there. The label's own update animation keeps it on the line's end,
 * once ECharts has its place to glide from. */
function releaseEndLabels(instance: ECharts, firstDraw: boolean) {
  let made = false
  for (const root of instance.getZr().storage.getRoots()) {
    root.traverse((element) => {
      if (element.type !== 'ec-polyline' || !element.getTextContent()) return
      made ||= (element as { disableLabelAnimation?: boolean }).disableLabelAnimation === true
      Object.assign(element, { disableLabelAnimation: false })
    })
  }
  // On the chart's first draw, the labels are given their places now, or the
  // first change would move none of them. Not later: it would cut short the
  // glides of the labels already there. A line ticked later has its label
  // glide from its second change on.
  if (firstDraw && made) instance.updateLabelLayout()
}

/**
 * Puts a chart on the element: `<div {@attach chart(() => option, setup)}>`.
 * The chart updates whenever state read by `option` changes, never before
 * the next paint, so that what the user typed and the table show first, and
 * glides to the change on the page's curve. A change that comes mid-glide
 * waits for it to end, and while a number is typed the chart waits for a
 * pause (`drawAt`). It follows the element's size. While the element has no
 * size (its view is hidden) nothing is computed or drawn, and the chart is
 * only made once it first has one.
 * `setup` can add event handlers; what it returns is called on removal.
 */
export function chart(
  option: () => ChartOption,
  setup?: (chart: ECharts) => (() => void) | void,
): Attachment<HTMLElement> {
  return (element) => {
    let instance: ECharts | undefined
    let stopSetup: (() => void) | undefined
    let lastDrawn: ChartOption | undefined
    /** The change waiting to be drawn, when the first change it replaced
     * came, and whether one came while the chart was still gliding. */
    let waiting: ChartOption | undefined
    let firstWaiting = 0
    let chained = false
    /** When the glide in flight has all but settled. */
    let settled = -Infinity
    let size = { width: 0, height: 0 }
    let frame = 0
    let timer: ReturnType<typeof setTimeout> | undefined
    /** Whether the element has a size. */
    let shown = $state(false)
    /** Computed only when read: not while hidden, and not again when shown
     * again with nothing changed. */
    const current = $derived.by(option)

    const draw = () => {
      const glide = duration(glideFor(chained))
      // Replace, not merge, the series: unticked plans must disappear.
      instance!.setOption({ ...waiting, animationDurationUpdate: glide }, { replaceMerge: ['series'] })
      releaseEndLabels(instance!, lastDrawn === undefined)
      lastDrawn = waiting
      waiting = undefined
      chained = false
      settled = performance.now() + settlesIn(glide)
    }
    const cancelDraw = () => {
      cancelAnimationFrame(frame)
      clearTimeout(timer)
    }

    $effect(() => {
      if (!shown) return
      const next = current
      if (next === lastDrawn || next === waiting) return
      const now = performance.now()
      if (waiting === undefined) firstWaiting = now
      if (now < settled) chained = true
      waiting = next
      const at = drawAt({ now, settled, lastTyped: lastTyped(), firstWaiting })
      // A timer set from an animation frame runs once that frame is
      // painted. A change before then replaces the one waiting.
      cancelDraw()
      frame = requestAnimationFrame(() => {
        timer = setTimeout(draw, at - performance.now())
      })
    })

    const resizing = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      if (width === 0 || height === 0) {
        shown = false
        // A draw still waiting would draw what the chart was told as it was
        // hidden (the fee breakdown once drew itself 0 px wide).
        cancelDraw()
        waiting = undefined
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
      } else if (width !== size.width) {
        instance.resize()
      } else if (height !== size.height) {
        // Only taller or shorter: a bar chart's rows came or went. Its bars
        // glide into the new height, and on to the new rows when they're
        // drawn, a frame later or once the typing pauses.
        instance.resize({ animation: { duration: duration(SETTLE), easing: easeOut } })
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
