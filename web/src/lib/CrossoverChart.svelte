<script lang="ts">
  import type { ECharts } from 'echarts/core'
  import { planId, type AppState } from './app.svelte'
  import { crossoverOption, YOURS, type CostLine } from './crossover-chart'
  import { chart } from './echarts.svelte'
  import { touchScreen } from './pointer'

  let { app }: { app: AppState } = $props()

  const option = () => {
    const sweep = app.sweep
    const lines: CostLine[] = (sweep?.plans ?? []).flatMap(({ key, costs }) => {
      const plan = app.plansById.get(planId(key))
      // Plans that don't offer the security have no line.
      const rank = app.rankOf(planId(key))
      if (!plan || !costs || rank === undefined) return []
      return [
        { id: plan.id, label: plan.label, color: plan.color, rank, dotted: plan.yours !== undefined, costs },
      ]
    })
    return crossoverOption({
      amounts: sweep?.amounts ?? [],
      lines,
      swept: app.swept,
      yours: app.swept === 'Monthly' ? app.monthlyDeposit : app.firstDeposit,
      pinned: app.pinned,
    })
  }

  /** Hovering and clicking lines, linked with the table. */
  function setup(instance: ECharts) {
    // Events say which series by position; this finds the plan's id.
    const planAt = (seriesIndex: number | undefined): string | undefined => {
      const series = instance.getOption().series as { id: string }[]
      const id = seriesIndex === undefined ? undefined : series[seriesIndex]?.id
      return id === YOURS ? undefined : id
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
  }
</script>

<div class="by-deposit" {@attach chart(option, setup)}></div>

<style>
  .by-deposit {
    width: 100%;
    height: 480px;
  }
</style>
