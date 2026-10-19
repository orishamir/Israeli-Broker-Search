<script lang="ts">
  // The workbench's charts, shown with the switch on: how the money grows,
  // what was lost to fees, the fees by kind (with the fund's own cost), and
  // the chart by deposit. Each draws a line or a bar for every plan compared,
  // or, for the S&P 500, for every way (as its cheapest plan). The guide has
  // its own chart (`Alternatives`) and donut. Screen s4.
  import type { AppState, ChartView } from './app.svelte'
  import { missingPlans, planEntries, wayColorOf, wayEntries, type ChartEntry } from './chart-entries'
  import Choices from './Choices.svelte'
  import type { Choice, OutcomeData } from './core/core'
  import CrossoverChart from './CrossoverChart.svelte'
  import FeeBreakdown from './FeeBreakdown.svelte'
  import GrowthChart from './GrowthChart.svelte'
  import { t } from './text'
  import { en } from './text/en'

  let { app }: { app: AppState } = $props()

  const choice = (
    value: ChartView,
    name: string,
    englishName: string,
    explanation: string,
  ): Choice<ChartView> => ({
    value,
    name,
    englishName,
    explanation,
    card: undefined,
    hebrewNames: [],
  })
  const lostView = choice('lost', t.lostView, en.lostView, t.lostViewTip)
  const valueView = choice('value', t.valueView, en.valueView, t.valueViewTip)
  const breakdownView = choice('breakdown', t.breakdownView, en.breakdownView, t.breakdownViewTip)
  const byDepositView = choice('crossover', t.byDepositView, en.byDepositView, t.byDepositViewTip)

  const waysView = $derived(
    app.thing === 'sp500' && app.waysComparison && 'ways' in app.waysComparison
      ? app.waysComparison
      : undefined,
  )
  const plansView = $derived(
    app.thing === 'other' && app.comparison && 'results' in app.comparison ? app.comparison : undefined,
  )

  const entries: ChartEntry[] = $derived(
    waysView
      ? wayEntries(waysView.ways, wayColorOf(app.wayChoices))
      : plansView
        ? planEntries(plansView.results)
        : [],
  )
  const missing = $derived(plansView ? missingPlans(plansView.results) : [])
  const noFees: OutcomeData | undefined = $derived(plansView?.noFees ?? waysView?.ways[0]?.noFees)

  /** The chart by deposit compares plans on one exchange: the ways' plans
   * are on different ones. */
  const byDeposit = $derived(plansView !== undefined)
  const views = $derived(
    byDeposit ? [lostView, valueView, breakdownView, byDepositView] : [lostView, valueView, breakdownView],
  )
  // A view the thing doesn't have isn't left chosen.
  $effect(() => {
    if (!byDeposit && app.chartView === 'crossover') app.chartView = 'lost'
  })
  const overYears = $derived(app.chartView === 'value' || app.chartView === 'lost')
  const lines = $derived(app.chartView !== 'breakdown')
</script>

{#if entries.length > 0 && noFees}
  <!-- ECharts shows a tooltip where the last one was before moving it beside
       the pointer: the card clips it (see CLAUDE.md, "Known traps"). -->
  <section class="card" id="chart">
    <div class="chart-bar">
      <Choices label={t.chart} options={views} bind:value={app.chartView} wraps />
      <!-- The hint for lines, the one for bars and the button take turns in
           one place that fits any of them, so pinning the first plan or
           switching views doesn't move the chart. -->
      <span class="pinning">
        {#each [false, true] as bars (bars)}
          {@const shown = app.pinned.size === 0 && bars === !lines}
          <span class="hint mouse" class:gone={!shown}>{t.pinHintMouse(bars)}</span>
          <span class="hint touch" class:gone={!shown}>{t.pinHintTouch(bars)}</span>
        {/each}
        <button class:gone={app.pinned.size === 0} onclick={() => app.pinned.clear()}>{t.unpinAll}</button>
      </span>
    </div>
    {#if waysView}<p class="hint ways-note">{t.waysChartsNote}</p>{/if}
    <!-- Every view stays, so switching back is instant: a hidden one isn't
         drawn (see echarts.svelte.ts), and the shown one fades in. -->
    <div class="view" hidden={!overYears} inert={!overYears}>
      <GrowthChart {app} {entries} {noFees} />
      <!-- Under the chart it zooms, not in the bar: the bar is then the same
           in every view, and switching views doesn't move the chart. -->
      <p class="hint zoom mouse">{t.zoomHintMouse}</p>
      <p class="hint zoom touch">{t.zoomHintTouch}</p>
    </div>
    {#if byDeposit}
      <div class="view" hidden={app.chartView !== 'crossover'} inert={app.chartView !== 'crossover'}>
        <CrossoverChart {app} />
      </div>
    {/if}
    <div class="view" hidden={lines} inert={lines}>
      <FeeBreakdown {app} {entries} {missing} />
    </div>
  </section>
{/if}

<style>
  #chart {
    overflow: clip;
  }
  .chart-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
    margin-bottom: 4px;
  }
  /* The view switched to fades in; the other is just gone. */
  .view {
    transition: opacity 150ms var(--ease-out);
  }
  @starting-style {
    .view:not([hidden]) {
      opacity: 0;
    }
  }
  .hint {
    font-size: 0.85rem;
    color: var(--weak);
  }
  .ways-note {
    margin: 0 0 4px;
  }
  .pinning {
    display: grid;
    align-items: center;
    justify-items: start;
  }
  .pinning > * {
    grid-area: 1 / 1;
  }
  /* Hidden, but still holding its place. */
  .gone {
    visibility: hidden;
  }
  .zoom {
    margin: 0;
    text-align: end;
  }
  @media (pointer: coarse) {
    .mouse {
      display: none;
    }
  }
  @media not (pointer: coarse) {
    .touch {
      display: none;
    }
  }
</style>
