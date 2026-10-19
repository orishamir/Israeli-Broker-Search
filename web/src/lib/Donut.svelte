<script lang="ts">
  // "לאן הולך כל שקל בסוף": the donut (`donutOption`, from
  // `recommendation.breakdown`, held while a number is typed), and under it
  // the parts too small to label. Screens s2, s3.
  import type { AppState } from './app.svelte'
  import { donutOption, donutParts, tinyList } from './donut'
  import { chart } from './echarts.svelte'
  import { glideHeight } from './motion'
  import { HeldWhileTyping } from './held.svelte'
  import { touchScreen } from './pointer'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.recommendation?.breakdown)
  const tiny = $derived(held.current ? tinyList(donutParts(held.current)) : '')

  const option = () =>
    donutOption({
      breakdown: held.current ?? { putIn: 0, fees: 0, keptByThePlace: 0, tax: 0, left: 0 },
      touch: touchScreen,
    })
</script>

{#if held.current}
  <section class="card donut" {@attach glideHeight()}>
    <div>
      <h2 class="titled">{t.donutTitle}<Tip about={t.donutTitle}>{t.donutTip}</Tip></h2>
      <div class="chart" {@attach chart(option)}></div>
      {#if tiny}
        <p class="tiny">{t.donutTiny(tiny)}</p>
      {/if}
    </div>
  </section>
{/if}

<style>
  .chart {
    width: 100%;
    height: 260px;
  }

  .tiny {
    margin: 4px 0 0;
    color: var(--weak);
    font-size: 0.8rem;
  }
</style>
