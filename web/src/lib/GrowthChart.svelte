<script lang="ts">
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { chart, type ChartOption } from './echarts.svelte'
  import { touchScreen } from './pointer'
  import { compactShekels, readableOn, shekels } from './format'
  import type { ECharts } from 'echarts/core'
  import type { LineSeriesOption } from 'echarts/charts'

  let { app, results, noFees }: { app: AppState; results: Result[]; noFees: OutcomeData } = $props()

  // The page's colors (see app.css).
  const WEAK = '#8e95a5'
  const GRID = 'rgba(255, 255, 255, 0.07)'

  const lost = $derived(app.chartView === 'lost')

  /** Phones and tablets: pinching there zooms the page, not the chart, so
   * they get a slider to zoom and move with instead. */
  const touch = touchScreen

  /** [years, value] points, a month apart, from index 0 (the start). */
  const points = (values: number[]) => values.map((value, month) => [month / 12, value])

  /** "After 15 years, 5 months" */
  function elapsed(years: number): string {
    const months = Math.round(years * 12)
    const plural = (count: number, unit: string) => `${count} ${unit}${count === 1 ? '' : 's'}`
    const whole = plural(Math.floor(months / 12), 'year')
    return months % 12 === 0 ? `After ${whole}` : `After ${whole}, ${plural(months % 12, 'month')}`
  }

  /** Pinned plans, in table order: every other one gets its yearly values
   * below its line rather than above, so they collide less. */
  const pinnedOrder = $derived(results.map(({ plan }) => plan.id).filter((id) => app.pinned.has(id)))

  // Pinned plans are styled through the options. A hovered plan isn't: it's
  // emphasized with ECharts' highlight action (see `setup`), because changing
  // the options redraws the lines, which ends the hover.
  function planSeries({ plan, outcome }: Result & { outcome: OutcomeData }): LineSeriesOption {
    const pinned = app.pinned.has(plan.id)
    const faded = app.pinned.size > 0 && !pinned
    return {
      id: plan.id,
      name: `${plan.broker.name} – ${plan.info.name}`,
      type: 'line',
      // Pinned plans show a dot and their value at each whole year; ones that
      // would overlap are hidden, so fewer show when zoomed out. Other plans
      // get plain points, which ECharts copies and compares much faster.
      data: pinned
        ? points(lost ? outcome.lostByMonth : outcome.valueByMonth).map((point) => {
            const yearly = Number.isInteger(point[0])
            return { value: point, symbol: yearly ? 'emptyCircle' : 'none', label: { show: yearly } }
          })
        : points(lost ? outcome.lostByMonth : outcome.valueByMonth),
      color: plan.color,
      lineStyle: { width: pinned ? 3.5 : 2, opacity: faded ? 0.4 : 1 },
      itemStyle: { opacity: faded ? 0.4 : 1 },
      showSymbol: pinned,
      // Otherwise ECharts skips some of them on lines with many points.
      showAllSymbol: true,
      label: {
        position: pinnedOrder.indexOf(plan.id) % 2 === 0 ? 'top' : 'bottom',
        formatter: ({ value }) => compactShekels((value as number[])[1]),
        backgroundColor: plan.color,
        color: readableOn(plan.color),
        padding: [1, 4],
        borderRadius: 3,
      },
      // The value where the line leaves the view, in the plan's color.
      endLabel: {
        show: true,
        formatter: ({ value }) => compactShekels((value as number[])[1]),
        color: faded ? undefined : plan.color,
        opacity: faded ? 0.6 : 1,
      },
      labelLayout: { moveOverlap: 'shiftY', hideOverlap: true },
      emphasis: { focus: 'series' },
      // Hovering or clicking the line itself, not only its points.
      triggerEvent: 'line',
    }
  }

  const option = (): ChartOption => {
    const offered = results.filter(
      (result): result is Result & { outcome: OutcomeData } => result.outcome !== undefined,
    )
    const series: LineSeriesOption[] = offered.map(planSeries)
    if (!lost) {
      series.unshift({
        id: 'no fees',
        name: 'No fees',
        type: 'line',
        data: points(noFees.valueByMonth),
        color: WEAK,
        lineStyle: { type: 'dashed', width: 1.5 },
        showSymbol: false,
        endLabel: { show: true, formatter: ({ value }) => compactShekels((value as number[])[1]) },
        labelLayout: { moveOverlap: 'shiftY' },
        silent: true,
      })
    }
    return {
      grid: { left: 16, right: 90, top: 24, bottom: touch ? 88 : 32 },
      xAxis: {
        type: 'value',
        name: 'Years',
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
        touch
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
        order: lost ? 'valueAsc' : 'valueDesc',
        valueFormatter: (value) => shekels(value as number),
      },
      series,
    }
  }

  /** Hovering and clicking lines; R or Home resets the zoom. */
  function setup(instance: ECharts) {
    // Events say which series by position; this finds the plan's id.
    const planAt = (seriesIndex: number | undefined): string | undefined => {
      const series = instance.getOption().series as { id: string }[]
      const id = seriesIndex === undefined ? undefined : series[seriesIndex]?.id
      return id === 'no fees' ? undefined : id
    }
    instance.on('mouseover', 'series', ({ seriesIndex }) => {
      if (touchScreen) return
      app.hovered = planAt(seriesIndex) ?? null
    })
    instance.on('mouseout', 'series', () => {
      app.hovered = null
    })
    instance.on('click', 'series', ({ seriesIndex }) => {
      const id = planAt(seriesIndex)
      if (id) app.togglePin(id)
    })

    // A plan hovered anywhere (here or in the table) stands out, the rest fade.
    $effect(() => {
      instance.dispatchAction({ type: 'downplay' })
      if (app.hovered) instance.dispatchAction({ type: 'highlight', seriesId: app.hovered })
    })

    const resetZoom = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
      if (!typing && (event.key === 'r' || event.key === 'Home')) {
        instance.dispatchAction({ type: 'dataZoom', start: 0, end: 100 })
      }
    }
    window.addEventListener('keydown', resetZoom)
    return () => window.removeEventListener('keydown', resetZoom)
  }
</script>

<div class="chart" {@attach chart(option, setup)}></div>

<style>
  .chart {
    width: 100%;
    height: 480px;
  }
</style>
