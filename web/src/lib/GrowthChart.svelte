<script lang="ts">
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { chart } from './echarts.svelte'
  import { growthOption, NO_FEES, type Line } from './growth-chart'
  import { linkLines } from './linked.svelte'
  import { touchScreen } from './pointer'

  let { app, results, noFees }: { app: AppState; results: Result[]; noFees: OutcomeData } = $props()

  /** Lost to fees rather than the value: as the view shown, or while another
   * chart is, as the last one shown, so that coming back to it (the fee
   * breakdown and back) doesn't redraw the chart. It's first drawn while one
   * of its own views is shown, so the first value is never used. */
  let lastLost = false
  const lost = $derived.by(() => {
    if (app.chartView === 'lost' || app.chartView === 'value') lastLost = app.chartView === 'lost'
    return lastLost
  })

  const option = () =>
    growthOption({
      lines: results
        .filter(
          (result): result is Result & { outcome: OutcomeData; rank: number } => result.outcome !== undefined,
        )
        .map(({ plan, outcome, rank }): Line => ({
          id: plan.id,
          label: plan.label,
          color: plan.color,
          rank,
          dotted: plan.yours !== undefined,
          values: lost ? outcome.lostByMonth : outcome.valueByMonth,
        })),
      noFees: lost ? null : noFees.valueByMonth,
      lost,
      pinned: app.pinned,
      touch: touchScreen,
    })
</script>

<div class="chart" {@attach chart(option, linkLines(app, NO_FEES))}></div>

<style>
  .chart {
    width: 100%;
    height: 480px;
  }
</style>
