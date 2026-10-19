<script lang="ts">
  // STUB (package J fills it): "לאן הולך כל שקל בסוף": the donut
  // (`donutOption`, from `recommendation.breakdown`) with tabs to see an
  // alternative's. Screens s2, s3.
  import type { AppState } from './app.svelte'
  import { donutOption } from './donut'
  import { chart } from './echarts.svelte'
  import { touchScreen } from './pointer'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const option = () =>
    donutOption({
      breakdown: app.recommendation?.breakdown ?? { putIn: 0, fees: 0, keptByThePlace: 0, tax: 0, left: 0 },
      touch: touchScreen,
    })
</script>

{#if app.recommendation}
  <section class="card donut" data-stub="Donut">
    <h2>{t.donutTitle}</h2>
    <div class="chart" {@attach chart(option)}></div>
  </section>
{/if}

<style>
  .chart {
    width: 100%;
    height: 260px;
  }
</style>
