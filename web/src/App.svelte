<script lang="ts">
  import { AppState, type ChartView } from './lib/app.svelte'
  import Choices from './lib/Choices.svelte'
  import type { Choice } from './lib/core/core'
  import DetailsDialog from './lib/DetailsDialog.svelte'
  import FeeBreakdown from './lib/FeeBreakdown.svelte'
  import GrowthChart from './lib/GrowthChart.svelte'
  import InputsPanel from './lib/InputsPanel.svelte'
  import ResultsTable from './lib/ResultsTable.svelte'
  import { shekels } from './lib/format'

  const views: Choice<ChartView>[] = [
    {
      value: 'value',
      name: 'Value',
      explanation: 'What each plan is worth, year by year, before selling.',
      hebrewNames: [],
    },
    {
      value: 'lost',
      name: 'Lost to fees',
      explanation:
        'How much less each plan has than with no fees and buying every month, year by year. Shows where plans overtake each other. The end is after selling, as in the table.',
      hebrewNames: [],
    },
    {
      value: 'breakdown',
      name: 'Breakdown',
      explanation: 'What each plan pays in fees, by kind, and how they pile up over the years.',
      hebrewNames: [],
    },
  ]

  const app = new AppState()
  /** The line charts, rather than the fee breakdown. */
  const lines = $derived(app.chartView !== 'breakdown')
  const best = $derived(
    // One that can be opened with these deposits.
    'results' in app.comparison
      ? app.comparison.results.find((result) => result.outcome && !result.warning)
      : undefined,
  )
</script>

<header class="top">
  <h1>Broker fees, compounded</h1>
  <p>What Israeli brokers' fees cost you over the years, for ETFs, index funds and bonds.</p>
  <!-- The page on how the numbers are made, opened at each of its sections. -->
  <p class="about">
    {#each app.about.sections as section, index (section.title)}
      {#if index > 0}<span class="separator" aria-hidden="true">·</span>{/if}
      <button class="link" onclick={() => (app.details = { kind: 'about', section: index })}>
        {section.title}
      </button>
    {/each}
  </p>
</header>

<div class="layout">
  <aside>
    <InputsPanel {app} />
  </aside>

  <main>
    {#if 'error' in app.comparison}
      <p class="card error">Check your inputs: {app.comparison.error}.</p>
    {:else}
      <div class="stats">
        <div class="card stat">
          <span class="label">You deposit</span>
          <span class="value">{shekels(app.comparison.deposited)}</span>
          <span class="note">over {app.years} years</span>
        </div>
        <div class="card stat">
          <span class="label">With no fees</span>
          <span class="value">{shekels(app.comparison.noFees.afterSelling)}</span>
          <span class="note">buying every month</span>
        </div>
        {#if best?.outcome}
          <div class="card stat best" style:--plan-color={best.plan.color}>
            <span class="label">Best: {best.plan.label}</span>
            <span class="value">{shekels(best.outcome.afterSelling)}</span>
            <span class="note">{shekels(best.outcome.lostToFees)} lost to fees</span>
          </div>
        {/if}
      </div>

      {#if app.comparison.results.length === 0}
        <p class="card empty">Tick a broker on the left to compare.</p>
      {:else}
        <section class="card table">
          <div class="scrolls">
            <ResultsTable {app} results={app.comparison.results} />
          </div>
        </section>

        <section class="card" id="chart">
          <div class="chart-bar">
            <Choices label="Chart" options={views} bind:value={app.chartView} />
            {#if app.pinned.size === 0}
              <span class="hint mouse">Click a row or a {lines ? 'line' : 'bar'} to pin it</span>
              <span class="hint touch">Tap a row or a {lines ? 'line' : 'bar'} to pin it</span>
            {:else}
              <button onclick={() => app.pinned.clear()}>Unpin all</button>
            {/if}
            {#if lines}
              <span class="hint right mouse">Wheel: zoom years · Drag: move · R: reset</span>
              <span class="hint right touch">Drag the slider's ends to zoom</span>
            {/if}
          </div>
          {#if lines}
            <GrowthChart {app} results={app.comparison.results} noFees={app.comparison.noFees} />
          {:else}
            <FeeBreakdown {app} results={app.comparison.results} />
          {/if}
        </section>
      {/if}
    {/if}
  </main>
</div>

<DetailsDialog {app} />

<style>
  .top {
    padding: 20px 24px 0;
  }
  .top h1 {
    margin: 0;
    font-size: 1.4rem;
    letter-spacing: -0.01em;
  }
  .top p {
    margin: 2px 0 0;
    color: var(--weak);
  }
  .about {
    font-size: 0.9rem;
  }
  .separator {
    margin: 0 8px;
  }
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: inherit;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
  .layout {
    display: grid;
    grid-template-columns: 360px minmax(0, 1fr);
    gap: 20px;
    padding: 16px 24px 24px;
    align-items: start;
  }
  main {
    container: results / inline-size;
    display: grid;
    /* No wider than the page: the table scrolls instead. */
    grid-template-columns: minmax(0, 1fr);
    gap: 16px;
  }
  /* Until the phone layout: one column, inputs first. */
  @media (width < 800px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
      padding: 12px;
    }
    .top {
      padding: 16px 12px 0;
    }
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }
  /* On phones the best plan comes first, across, with the others under it. */
  /* A narrow results column: the best plan first, across, with the others
     under it. */
  @container results (width < 520px) {
    .stats {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 8px;
    }
    .best {
      grid-column: 1 / -1;
      order: -1;
    }
    .stat:not(.best) .value {
      font-size: 1.2rem;
    }
  }
  .stat {
    display: grid;
    gap: 2px;
    border-top: 2px solid var(--plan-color, transparent);
  }
  .stat .label {
    color: var(--weak);
    font-size: 0.8rem;
  }
  .stat .value {
    font-size: 1.5rem;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
  }
  .stat .note {
    color: var(--weak);
    font-size: 0.8rem;
  }

  .table {
    padding: 4px;
  }
  .scrolls {
    overflow-x: auto;
    /* A shadow at the right edge while there's more to scroll to: the cover
       moves with the content and hides it at the end. */
    background:
      linear-gradient(to left, var(--surface) 40%, transparent) right / 40px 100% no-repeat local,
      radial-gradient(farthest-side at 100% 50%, rgb(0 0 0 / 0.6), transparent) right / 16px 100% no-repeat
        scroll;
  }
  .chart-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
    margin-bottom: 4px;
  }
  .hint {
    font-size: 0.85rem;
    color: var(--weak);
  }
  .right {
    margin-left: auto;
  }
  /* Mouse or finger, whichever this screen mostly uses. */
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
  .error {
    margin: 0;
    color: var(--error);
  }
  .empty {
    margin: 0;
    color: var(--weak);
  }
</style>
