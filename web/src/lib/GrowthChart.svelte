<script lang="ts">
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { chart } from './echarts.svelte'
  import { growthOption, NO_FEES, type Line } from './growth-chart'
  import { touchScreen } from './pointer'
  import type { ECharts } from 'echarts/core'

  let { app, results, noFees }: { app: AppState; results: Result[]; noFees: OutcomeData } = $props()

  const lost = $derived(app.chartView === 'lost')

  const option = () =>
    growthOption({
      lines: results
        .filter((result): result is Result & { outcome: OutcomeData } => result.outcome !== undefined)
        .map(({ plan, outcome }): Line => ({
          id: plan.id,
          label: plan.label,
          color: plan.color,
          dotted: plan.yours !== undefined,
          values: lost ? outcome.lostByMonth : outcome.valueByMonth,
        })),
      noFees: lost ? null : noFees.valueByMonth,
      lost,
      pinned: app.pinned,
      touch: touchScreen,
    })

  /** Hovering and clicking lines; R or Home resets the zoom. */
  function setup(instance: ECharts) {
    // Events say which series by position; this finds the plan's id.
    const planAt = (seriesIndex: number | undefined): string | undefined => {
      const series = instance.getOption().series as { id: string }[]
      const id = seriesIndex === undefined ? undefined : series[seriesIndex]?.id
      return id === NO_FEES ? undefined : id
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
