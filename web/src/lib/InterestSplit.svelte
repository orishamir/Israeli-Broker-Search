<script lang="ts">
  import type { ECharts } from 'echarts/core'
  import { MediaQuery } from 'svelte/reactivity'
  import type { ShortTermOutcomeData } from './core/core'
  import { chart } from './echarts.svelte'
  import { barsHeight, NAME_SIZE, NARROW_SCREEN } from './fee-breakdown'
  import { shekels } from './format'
  import { PART_COLORS, splitOption, type Split } from './interest-split'
  import { touchScreen } from './pointer'
  import type { PlaceResult, ShortTermState } from './short-term.svelte'
  import { t } from './text'

  /** Where each place's interest goes: to the saver, to the place, and to tax. */
  let {
    short,
    results,
    deposited,
    atTheRate,
  }: { short: ShortTermState; results: PlaceResult[]; deposited: number; atTheRate: number[] } = $props()

  const splits = $derived(
    results
      .filter(
        (result): result is PlaceResult & { outcome: ShortTermOutcomeData } => result.outcome !== undefined,
      )
      .map(({ place, outcome }): Split => ({
        id: place.id,
        label: place.label,
        color: place.color,
        hollow: place.yours !== undefined,
        yours: outcome.afterTax - deposited,
        kept: outcome.kept,
        tax: outcome.tax,
      })),
  )
  /** The interest at the Bank of Israel's rate, with nothing kept and no tax. */
  const interest = $derived((atTheRate.at(-1) ?? deposited) - deposited)

  /** The bars' box, for fitting names and amounts; hidden, the width it was
   * shown at stands (see the fee breakdown). */
  let width = $state(0)
  const setWidth = (next: number) => {
    if (next > 0) width = next
  }
  const narrow = new MediaQuery(NARROW_SCREEN)

  /** A text's width as the chart draws it: the page's font, at the names' size. */
  const context = document.createElement('canvas').getContext('2d')!
  const font = getComputedStyle(document.documentElement).fontFamily
  const measure = (text: string, bold = false) => {
    context.font = `${bold ? 'bold ' : ''}${NAME_SIZE}px ${font}`
    return context.measureText(text).width
  }

  const option = () => splitOption({ splits, atTheRate: interest, pinned: short.pinned, width, measure })

  /** Hovering a bar or a name highlights its row; clicking pins it. */
  function setup(instance: ECharts) {
    // The names and totals ride on the bars: every part of a row has its index.
    const placeAt = (event: { dataIndex?: number }) => splits[event.dataIndex ?? -1]
    instance.on('mouseover', (event) => {
      if (!touchScreen) short.hovered = placeAt(event)?.id ?? null
    })
    instance.on('mouseout', () => {
      short.hovered = null
    })
    instance.on('click', (event) => {
      const place = placeAt(event)
      if (place) short.togglePin(place.id)
    })
    // The hovered place's outline, wherever it's hovered (here or in the table).
    $effect(() => {
      const index = splits.findIndex(({ id }) => id === short.hovered)
      instance.dispatchAction({ type: 'downplay', seriesIndex: 0 })
      if (index >= 0) instance.dispatchAction({ type: 'highlight', seriesIndex: 0, dataIndex: index })
    })
  }
</script>

<p class="hint">{t.rateLine(shekels(interest))}</p>
<!-- Above the bars, as the fee breakdown's fees are: read before them. -->
<ul class="legend">
  <li><span class="swatch" style:background={PART_COLORS.yours}></span>{t.yoursPart}</li>
  <li><span class="swatch" style:background={PART_COLORS.kept}></span>{t.keptPart}</li>
  <li><span class="swatch" style:background={PART_COLORS.tax}></span>{t.taxPart}</li>
</ul>
<!-- As in the fee breakdown: the frame's height follows the chart's. -->
<div class="frame" style:height="{barsHeight(splits.length, narrow.current)}px">
  <div
    class="bars"
    bind:clientWidth={null, setWidth}
    style:height="{barsHeight(splits.length, narrow.current)}px"
    {@attach chart(option, setup)}
    role="img"
    aria-label={t.splitAria}
  ></div>
</div>

<style>
  .hint {
    margin: 8px 0 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  .frame {
    overflow: clip;
    transition: height var(--settle);
  }
  .bars {
    width: 100%;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    margin: 0 0 6px;
    padding: 0;
    list-style: none;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }
</style>
