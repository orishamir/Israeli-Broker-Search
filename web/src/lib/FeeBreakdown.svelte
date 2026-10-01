<script lang="ts">
  import type { ECharts } from 'echarts/core'
  import { MediaQuery } from 'svelte/reactivity'
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { chart } from './echarts.svelte'
  import {
    barsHeight,
    barsOption,
    byFee,
    colorOf,
    FEE_TYPES,
    NAME_SIZE,
    NARROW_SCREEN,
    overTimeOption,
    type Bar,
    type FeeType,
    type Focus,
  } from './fee-breakdown'
  import { readableOn } from './format'
  import { touchScreen } from './pointer'
  import { tip } from './tip'
  import Tip from './Tip.svelte'
  import { t } from './text'

  let { app, results }: { app: AppState; results: Result[] } = $props()

  let focus = $state<Focus>('all')
  /** Picks a fee to compare by, or all of them again if it's picked. */
  const toggle = (key: FeeType) => (focus = focus === key ? 'all' : key)
  /** Each fee button's explanation, shown on hover. */
  const popovers = $state<HTMLElement[]>([])

  const notOffered = $derived(results.filter(({ outcome }) => !outcome))
  const offered = $derived(
    results.filter((result): result is Result & { outcome: OutcomeData } => result.outcome !== undefined),
  )
  /** The bars, least fees first: in total, or in the focused fee. */
  const rows = $derived(
    byFee(
      offered.map(({ plan, outcome }): Bar => ({
        id: plan.id,
        label: plan.label,
        color: plan.color,
        hollow: plan.yours !== undefined,
        fees: outcome.fees,
      })),
      focus,
    ),
  )

  /** The plan shown over time: the one under the mouse, else the last pinned,
   * else the best. */
  const shown = $derived.by(() => {
    const byId = (id: string | null | undefined) => offered.find(({ plan }) => plan.id === id)
    return byId(app.hovered) ?? byId([...app.pinned].at(-1)) ?? offered[0]
  })

  /** The bars' box, for fitting names and amounts. */
  let barsWidth = $state(0)
  /** Hidden, the box is 0 px wide: the width it was shown at stands, or the
   * chart would be drawn again, for nothing, each time the view is left. */
  const setBarsWidth = (width: number) => {
    if (width > 0) barsWidth = width
  }
  const narrow = new MediaQuery(NARROW_SCREEN)

  /** A text's width as the chart draws it: the page's font, at the names' size. */
  const context = document.createElement('canvas').getContext('2d')!
  const font = getComputedStyle(document.documentElement).fontFamily
  const measure = (text: string, bold = false) => {
    context.font = `${bold ? 'bold ' : ''}${NAME_SIZE}px ${font}`
    return context.measureText(text).width
  }

  const bars = () => barsOption({ bars: rows, focus, pinned: app.pinned, width: barsWidth, measure })
  const overTime = () => overTimeOption(shown.outcome.feesUpToYear, focus)

  /** Hovering a bar or a plan's name shows it over time; clicking pins it. */
  function setup(instance: ECharts) {
    // The names and totals ride on the bars: every part of a row has its index.
    const planAt = (event: { dataIndex?: number }) => rows[event.dataIndex ?? -1]
    instance.on('mouseover', (event) => {
      if (!touchScreen) app.hovered = planAt(event)?.id ?? null
    })
    instance.on('mouseout', () => {
      app.hovered = null
    })
    instance.on('click', (event) => {
      const plan = planAt(event)
      if (plan) app.togglePin(plan.id)
    })

    // The hovered plan's band, wherever it's hovered (here or in the table).
    $effect(() => {
      const index = rows.findIndex(({ id }) => id === app.hovered)
      instance.dispatchAction({ type: 'downplay', seriesIndex: 0 })
      if (index >= 0) instance.dispatchAction({ type: 'highlight', seriesIndex: 0, dataIndex: index })
    })
  }
</script>

<!-- The legend is also the control: a fee's button compares the plans by
     it. One row where it fits, else two by two. -->
<p class="label">
  <span class="mouse">{t.clickAFee}</span><span class="touch">{t.tapAFee}</span>
  {t.aFeeToCompare}
</p>
<div class="fees" role="group" aria-label={t.compareBy}>
  {#each FEE_TYPES as type, index (type.key)}
    {@const popover = popovers[index]}
    <button
      type="button"
      class="fee"
      aria-pressed={focus === type.key}
      style:--fee-color={type.color}
      style:--on-fee-color={readableOn(type.color)}
      onclick={() => toggle(type.key)}
      {@attach popover && tip(popover, { onClick: false })}
    >
      <span class="swatch" style:background={colorOf(type, focus)}></span>{type.name}
    </button>
    <div class="popover" popover="manual" role="tooltip" bind:this={popovers[index]}>
      <p>{type.explanation}</p>
    </div>
  {/each}
</div>

<!-- The chart takes its new height at once as plans come and go, and glides
     into it; the frame around it follows on the same curve, so what's below
     moves smoothly too. -->
<div class="frame" style:height="{barsHeight(rows.length, narrow.current)}px">
  <div
    class="bars"
    bind:clientWidth={null, setBarsWidth}
    style:height="{barsHeight(rows.length, narrow.current)}px"
    {@attach chart(bars, setup)}
    role="img"
    aria-label={t.barsAria}
  ></div>
</div>

{#if notOffered.length > 0}
  <p class="hint">
    {t.notOfferedFor(app.purchase)}:
    {#each notOffered as { plan, notOffered: reason }, index (plan.id)}
      {index > 0 ? ', ' : ''}{plan.label}<Tip about={t.notOffered}>{reason}</Tip>
    {/each}
  </p>
{/if}

<h4>
  {t.yearByYear} <span class="dot" style:background={shown.plan.color}></span>{shown.plan.label}
  <span class="hint mouse">{t.hoverOrPin}</span>
  <span class="hint touch">{t.tapAnotherBar}</span>
</h4>
<div
  class="over-time"
  {@attach chart(overTime)}
  role="img"
  aria-label={t.overTimeAria(shown.plan.label)}
></div>

<style>
  .label {
    margin: 12px 0 6px;
  }
  .fees {
    container-type: inline-size;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 6px;
  }
  /* Two by two where one row doesn't fit, each column as wide as its widest
     name, which then doesn't wrap: halves were narrower than "דמי ניהול
     ומשמרת" on a Galaxy S24. One column where even that doesn't fit. */
  @container (width < 440px) {
    .fees {
      display: grid;
      grid-template-columns: repeat(2, minmax(max-content, 1fr));
    }
  }
  @container (width < 300px) {
    .fees {
      grid-template-columns: 1fr;
    }
  }
  .fee {
    display: flex;
    gap: 7px;
    align-items: center;
    padding: 4px 10px;
    border: 1px solid var(--strong-border);
    background: transparent;
    color: var(--text);
  }
  /* Not bold, as the choices aren't: a bolder name is wider, and would widen
     its column. */
  .fee[aria-pressed='true'] {
    border-color: var(--fee-color);
    background: var(--fee-color);
    color: var(--on-fee-color);
  }
  .label,
  .hint {
    color: var(--weak);
    font-size: 0.85rem;
  }
  .swatch {
    width: 12px;
    height: 12px;
    border-radius: 3px;
  }
  .frame {
    overflow: clip;
    transition: height var(--settle);
  }
  .bars {
    width: 100%;
  }
  h4 {
    margin: 16px 0 0;
    font-size: 0.95rem;
    font-weight: 600;
  }
  .hint {
    font-weight: normal;
  }
  .dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    margin: 0 6px 0 4px;
    border-radius: 50%;
  }
  .over-time {
    width: 100%;
    height: 260px;
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
</style>
