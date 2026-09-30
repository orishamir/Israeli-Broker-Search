<script lang="ts">
  import { AppState, type ChartView } from './lib/app.svelte'
  import Choices from './lib/Choices.svelte'
  import type { Choice } from './lib/core/core'
  import CrossoverChart from './lib/CrossoverChart.svelte'
  import DetailsDialog from './lib/DetailsDialog.svelte'
  import FeeBreakdown from './lib/FeeBreakdown.svelte'
  import GrowthChart from './lib/GrowthChart.svelte'
  import InputsPanel from './lib/InputsPanel.svelte'
  import { decode } from './lib/link'
  import ResultsTable from './lib/ResultsTable.svelte'
  import Share from './lib/Share.svelte'
  import { askSweep } from './lib/sweeper'
  import { percent, shekels } from './lib/format'
  import { duration, reducedMotion } from './lib/motion'
  import { lang, switchLang, t } from './lib/text'
  import { en } from './lib/text/en'
  import { cubicOut } from 'svelte/easing'
  import { Tween } from 'svelte/motion'

  const allViews: Choice<ChartView>[] = [
    {
      value: 'lost',
      name: t.lostView,
      englishName: en.lostView,
      explanation: t.lostViewTip,
      hebrewNames: [],
    },
    {
      value: 'value',
      name: t.valueView,
      englishName: en.valueView,
      explanation: t.valueViewTip,
      hebrewNames: [],
    },
    {
      value: 'crossover',
      name: t.byDepositView,
      englishName: en.byDepositView,
      explanation: t.byDepositViewTip,
      hebrewNames: [],
    },
    {
      value: 'breakdown',
      name: t.breakdownView,
      englishName: en.breakdownView,
      explanation: t.breakdownViewTip,
      hebrewNames: [],
    },
  ]

  // Opened from a link, perhaps: its state comes first.
  const app = new AppState(decode(location.hash))
  /** The chart by deposit is for experts: offered with More options only. */
  const views = $derived(allViews.filter((view) => view.value !== 'crossover' || app.moreOptions))
  /** The growth chart's views: over the years, with a zoom. */
  const overYears = $derived(app.chartView === 'value' || app.chartView === 'lost')
  /** The line charts, rather than the fee breakdown. */
  const lines = $derived(app.chartView !== 'breakdown')
  const best = $derived(app.best)

  // The sweep runs off the page's thread; its answer comes a moment later.
  $effect(() => {
    const request = app.sweepRequest
    askSweep(request, (sweep) => (app.sweepAnswer = { request, sweep }))
  })

  /** On phones the results are screens below the inputs, so the best plan
   * follows along the bottom of the screen until they're reached: while the
   * summary is below the screen, and no field is being typed in (the
   * keyboard would push the bar up over the field). */
  let stats = $state<HTMLElement>()
  let resultsBelow = $state(false)
  let typing = $state(false)
  const placeBar = () => {
    // Null, not undefined, while the summary is unmounted (bad inputs).
    resultsBelow = !!stats && stats.getBoundingClientRect().top > innerHeight
  }
  $effect(() => {
    const element = stats
    if (!element) return
    // Watched two ways: the observer sees the summary move as the inputs
    // above it change size, and scrolling is watched itself, since a jump
    // straight past the summary (tapping the bar, a test) crosses none of
    // the observer's thresholds.
    const observer = new IntersectionObserver(placeBar)
    observer.observe(element)
    placeBar()
    return () => observer.disconnect()
  })
  const typingIn = (target: EventTarget | null) =>
    target instanceof HTMLElement &&
    target.matches('input:not([type=checkbox], [type=radio], [type=range]), select')

  /** The summary's numbers roll to their new values rather than jump. Each
   * keeps its last value while the inputs are invalid, when the cards are
   * hidden: otherwise they'd roll up from 0 when the cards return. */
  const rolling = (value: () => number | undefined) => {
    let last = 0
    return Tween.of(() => (last = value() ?? last), { duration: duration(240), easing: cubicOut })
  }
  const comparison = () => ('results' in app.comparison ? app.comparison : undefined)
  const deposited = rolling(() => comparison()?.deposited)
  const noFees = rolling(() => comparison()?.noFees.afterSelling)
  const bestValue = rolling(() => best?.outcome?.afterTax)
  const bestTax = rolling(() => best?.outcome?.tax)
  const bestLost = rolling(() => best?.outcome?.lostToFees)
</script>

<!-- index.html's skeleton repeats the title and the line under it. -->
<header class="top">
  <div class="title">
    <h1>{t.title}</h1>
    <div class="actions">
      <!-- The other language, named in itself, so anyone can find their own. -->
      <button
        class="language"
        lang={lang === 'he' ? 'en' : 'he'}
        aria-label={t.switchLanguage}
        onclick={() => switchLang(lang === 'he' ? 'en' : 'he', app.shareLink().split('#')[1] ?? '')}
        >{t.otherLanguage}</button
      >
      <Share {app} />
    </div>
  </div>
  <p>{t.subtitle}</p>
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
      <p class="card error">{t.checkInputs(app.comparison.error)}</p>
    {:else}
      <div class="stats" bind:this={stats}>
        <div class="card stat">
          <span class="label">{t.youDeposit}</span>
          <span class="value">{shekels(deposited.current)}</span>
          <span class="note">{t.overYears(app.years, app.inTodaysMoney)}</span>
        </div>
        <div class="card stat">
          <span class="label">{t.withNoFees}</span>
          <span class="value">{shekels(noFees.current)}</span>
          <span class="note">{app.sellAtEnd ? t.ifSoldBeforeTax : t.heldAtEnd}</span>
        </div>
        {#if best?.outcome}
          <div class="card stat best" style:--plan-color={best.plan.color}>
            <span class="label">{t.best(best.plan.label)}</span>
            <span class="value">{shekels(bestValue.current)}</span>
            {#if app.sellAtEnd}
              <span class="note"
                >{t.leftAfter(best.outcome.tax === 0 ? undefined : shekels(bestTax.current))}</span
              >
            {/if}
            <span class="note"
              >{t.lostAndYearly(shekels(bestLost.current), percent(best.outcome.yearlyCostPercent))}</span
            >
            {#if best.warning}<span class="note warning">⚠ {best.warning}</span>{/if}
            <span class="note around" class:gone={!app.aroundLine.shown}>{app.aroundLine.text}</span>
          </div>
        {/if}
      </div>

      {#if app.comparison.results.length === 0}
        <p class="card empty">{t.tickABroker}</p>
      {:else}
        <section class="card table">
          <div class="scrolls">
            <ResultsTable {app} results={app.comparison.results} />
          </div>
        </section>

        <section class="card" id="chart">
          <div class="chart-bar">
            <Choices label={t.chart} options={views} bind:value={app.chartView} wraps />
            <!-- The hint for lines, the one for bars and the button take turns
                 in one place that fits any of them, so pinning the first plan
                 or switching views doesn't move the chart. -->
            <span class="pinning">
              {#each [false, true] as bars (bars)}
                {@const shown = app.pinned.size === 0 && bars === !lines}
                <span class="hint mouse" class:gone={!shown}>{t.pinHintMouse(bars)}</span>
                <span class="hint touch" class:gone={!shown}>{t.pinHintTouch(bars)}</span>
              {/each}
              <button class:gone={app.pinned.size === 0} onclick={() => app.pinned.clear()}
                >{t.unpinAll}</button
              >
            </span>
          </div>
          <!-- Every view stays, so switching back is instant: a hidden one
               isn't drawn (see echarts.svelte.ts), and the shown one fades in. -->
          <div class="view" hidden={!overYears} inert={!overYears}>
            <GrowthChart {app} results={app.comparison.results} noFees={app.comparison.noFees} />
            <!-- Under the chart it zooms, not in the bar: the bar is then the
                 same in every view, and switching views doesn't move the chart. -->
            <p class="hint zoom mouse">{t.zoomHintMouse}</p>
            <p class="hint zoom touch">{t.zoomHintTouch}</p>
          </div>
          <div class="view" hidden={app.chartView !== 'crossover'} inert={app.chartView !== 'crossover'}>
            <CrossoverChart {app} />
          </div>
          <div class="view" hidden={lines} inert={lines}>
            <FeeBreakdown {app} results={app.comparison.results} />
          </div>
        </section>
      {/if}
    {/if}
  </main>
</div>

<svelte:window
  onscroll={placeBar}
  onfocusin={(event) => (typing = typingIn(event.target))}
  onfocusout={() => (typing = false)}
/>

<!-- The best plan, along the bottom of a phone's screen while the results
     are below it; tapping it goes to them. -->
{#if best?.outcome && resultsBelow && !typing}
  <button
    class="best-bar"
    style:--plan-color={best.plan.color}
    onclick={() => stats?.scrollIntoView({ block: 'start', behavior: reducedMotion ? 'auto' : 'smooth' })}
  >
    <span class="bar-label">{t.best(best.plan.label)}</span>
    <span class="bar-note"
      >{t.yearlyAndLost(percent(best.outcome.yearlyCostPercent), shekels(best.outcome.lostToFees))}</span
    >
    <span class="bar-arrow" aria-hidden="true">↓</span>
  </button>
{/if}

<DetailsDialog {app} />

<style>
  .top {
    padding: 20px 24px 0;
  }
  .title {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
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
  .stat .warning {
    color: var(--warning);
  }

  /* The bar exists on phones only (the desktop shows the results beside the
     inputs). It slides up when it appears and is just gone when it hides. */
  .best-bar {
    display: none;
  }
  @media (width < 800px) {
    .best-bar {
      position: fixed;
      inset: auto 12px calc(10px + env(safe-area-inset-bottom));
      z-index: 2;
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        'label arrow'
        'note arrow';
      gap: 0 12px;
      align-items: center;
      padding: 8px 14px;
      border: 1px solid var(--strong-border);
      border-top: 2px solid var(--plan-color);
      background: var(--raised);
      box-shadow: var(--shadow);
      text-align: start;
      transition:
        translate 200ms var(--ease-out),
        opacity 200ms var(--ease-out);
    }
    @starting-style {
      .best-bar {
        opacity: 0;
        translate: 0 12px;
      }
    }
  }
  .bar-label {
    grid-area: label;
    font-weight: 600;
  }
  .bar-note {
    grid-area: note;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .bar-arrow {
    grid-area: arrow;
    color: var(--accent);
    font-size: 1.2rem;
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
  :global(html[dir='rtl']) .scrolls {
    background:
      linear-gradient(to right, var(--surface) 40%, transparent) left / 40px 100% no-repeat local,
      radial-gradient(farthest-side at 0% 50%, rgb(0 0 0 / 0.6), transparent) left / 16px 100% no-repeat
        scroll;
  }
  /* ECharts shows a tooltip where the last one was before moving it beside
     the pointer, and the first one in the middle of the chart: on a phone,
     that reached past the screen's edge for a moment, and the browser zoomed
     the page out to fit it, which jumped the page. Cut off here instead. */
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
  /* The language switch and Share, at the end of the title's row. */
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-inline-start: auto;
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
