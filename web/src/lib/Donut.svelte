<script lang="ts">
  // "לאן הולך כל שקל בסוף": the donut (`donutOption`, from the answer's
  // breakdown, held while a number is typed), tabs over it for the kinds
  // drawn beside the answer (screen s3: "גמל להשקעה / חשבון מסחר"), and under
  // it the parts too small to label. Screens s2, s3.
  import type { AppState } from './app.svelte'
  import { donutOption, donutParts, donutTabs, tinyList } from './donut'
  import { chart } from './echarts.svelte'
  import { glideHeight } from './motion'
  import { HeldWhileTyping } from './held.svelte'
  import { touchScreen } from './pointer'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => {
    const recommendation = app.recommendation
    return recommendation ? donutTabs(recommendation, recommendation.alternatives) : undefined
  })
  let chosen = $state('answer')
  // A tab that went with the inputs falls back to the answer's.
  const tab = $derived(held.current?.find(({ id }) => id === chosen) ?? held.current?.[0])
  const tiny = $derived(tab ? tinyList(donutParts(tab.breakdown)) : '')

  const option = () =>
    donutOption({
      breakdown: tab?.breakdown ?? { putIn: 0, fees: 0, product: 0, keptByThePlace: 0, tax: 0, left: 0 },
      touch: touchScreen,
    })
</script>

{#if tab && held.current}
  <section class="card donut" {@attach glideHeight()}>
    <div>
      <h2 class="titled">{t.donutTitle}<Tip about={t.donutTitle}>{t.donutTip}</Tip></h2>
      {#if held.current.length > 1}
        <div class="tabs" role="group" aria-label={t.donutTitle}>
          {#each held.current as each (each.id)}
            <button
              class="tab"
              class:chosen={each.id === tab.id}
              aria-pressed={each.id === tab.id}
              onclick={() => (chosen = each.id)}>{each.name}</button
            >
          {/each}
        </div>
      {/if}
      <div class="chart" {@attach chart(option)}></div>
      {#if tiny}
        <p class="tiny">{t.donutTiny(tiny)}</p>
      {/if}
    </div>
  </section>
{/if}

<style>
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 8px 0 4px;
  }

  .tab {
    min-height: 33px;
    padding: 4px 14px;
    border-radius: 999px;
  }

  .tab.chosen {
    border-color: var(--accent);
    background: var(--accent-tint);
  }

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
