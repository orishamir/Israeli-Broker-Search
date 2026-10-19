<script lang="ts">
  // STUB (package J fills it): "לעומת האפשרויות האחרות": the chart
  // (`alternativesOption`, from `app.alternatives`, held while typing with
  // `HeldWhileTyping`) and the rows under it (`recommendation.alternatives`:
  // kind, safety tag, assumption, amount, why not; tapping a row calls
  // `app.chooseKind(kind)`), linked to the chart (`linked.svelte.ts`).
  // Screens s2, s3.
  import type { AppState } from './app.svelte'
  import { alternativesOption } from './alternatives-chart'
  import { chart } from './echarts.svelte'
  import { kindKey } from './kinds'
  import { touchScreen } from './pointer'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const option = () =>
    alternativesOption({
      data: app.alternatives ?? { months: 1, series: [], band: undefined },
      highlighted: new Set(),
      colors: new Map(),
      touch: touchScreen,
    })
</script>

<section class="card alternatives" data-stub="Alternatives">
  <h2>{t.alternativesTitle}</h2>
  <div class="chart" {@attach chart(option)}></div>
  <ul>
    {#each app.recommendation?.alternatives ?? [] as alternative (kindKey(alternative.kind))}
      <li>
        <button onclick={() => app.chooseKind(alternative.kind)}>{alternative.name}</button>
        {alternative.whyNot.text}
      </li>
    {/each}
  </ul>
</section>

<style>
  .chart {
    width: 100%;
    height: 280px;
  }
</style>
