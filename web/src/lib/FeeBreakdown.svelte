<script lang="ts">
  import type { BarSeriesOption } from 'echarts/charts'
  import type { ECharts } from 'echarts/core'
  import type { AppState, Result } from './app.svelte'
  import type { FeeAmounts, OutcomeData } from './core/core'
  import { chart, type ChartOption } from './echarts.svelte'
  import { compactShekels, readableOn, shekels } from './format'
  import { touchScreen } from './pointer'
  import { tip } from './tip'

  let { app, results }: { app: AppState; results: Result[] } = $props()

  type FeeType = Exclude<keyof FeeAmounts, 'total'>

  /** The kinds of fee, in the order they're stacked. The colors are checked
   * for color blindness in this order, where each differs from its
   * neighbors; not every pair differs, so a focused fee fades the others. */
  const FEE_TYPES: { key: FeeType; name: string; color: string; explanation: string }[] = [
    {
      key: 'purchases',
      name: 'Purchases',
      color: '#3987e5',
      explanation: 'The trade fee on every purchase.',
    },
    {
      key: 'conversions',
      name: 'Conversions',
      color: '#d95926',
      explanation: "Converting shekels to the security's currency: the fee and the markup.",
    },
    { key: 'custody', name: 'Custody', color: '#199e70', explanation: 'Charged for holding the securities.' },
    {
      key: 'selling',
      name: 'Selling',
      color: '#c98500',
      explanation: 'Selling everything at the end, and converting back to shekels.',
    },
  ]
  // The page's colors (see app.css).
  const WEAK = '#8e95a5'
  const TEXT = '#e6e8ee'
  const GRID = 'rgba(255, 255, 255, 0.07)'
  const SURFACE = '#151820'
  const FADED = 'rgba(142, 149, 165, 0.3)'

  type Focus = 'all' | FeeType
  let focus = $state<Focus>('all')
  /** Picks a fee to compare by, or all of them again if it's picked. */
  const toggle = (key: FeeType) => (focus = focus === key ? 'all' : key)
  /** Each fee button's explanation, shown on hover. */
  const popovers = $state<HTMLElement[]>([])

  const notOffered = $derived(results.filter(({ outcome }) => !outcome).map(({ plan }) => plan.info.name))
  const offered = $derived(
    results.filter((result): result is Result & { outcome: OutcomeData } => result.outcome !== undefined),
  )
  /** Least fees first: in total, or in the focused fee. */
  const rows = $derived(
    offered.toSorted(
      (a, b) =>
        (focus === 'all' ? a.outcome.fees.total : a.outcome.fees[focus]) -
        (focus === 'all' ? b.outcome.fees.total : b.outcome.fees[focus]),
    ),
  )
  /** The focused fee first, so it starts where the bar does. */
  const stacking = $derived(
    focus === 'all'
      ? FEE_TYPES
      : [FEE_TYPES.find(({ key }) => key === focus)!, ...FEE_TYPES.filter(({ key }) => key !== focus)],
  )
  const colorOf = (type: (typeof FEE_TYPES)[number]) =>
    focus === 'all' || focus === type.key ? type.color : FADED

  /** The plan shown over time: the one under the mouse, else the last pinned,
   * else the best. */
  const shown = $derived.by(() => {
    const byId = (id: string | null | undefined) => offered.find(({ plan }) => plan.id === id)
    return byId(app.hovered) ?? byId([...app.pinned].at(-1)) ?? offered[0]
  })

  /** The bars' box, for fitting amounts into the parts of a bar. */
  let barsWidth = $state(0)
  const nameWidth = $derived(barsWidth < 500 ? 84 : 120)

  function barsOption(): ChartOption {
    const largest = Math.max(...rows.map(({ outcome }) => outcome.fees.total))
    // Roughly: the box without the names and the totals.
    const pixelsPerShekel = Math.max(barsWidth - nameWidth - 60, 0) / largest
    const fits = (amount: number) => amount * pixelsPerShekel > 44
    return {
      grid: { left: 4, right: 4, top: 4, bottom: 24 },
      xAxis: {
        type: 'value',
        max: largest,
        axisLabel: { formatter: compactShekels, color: WEAK, showMaxLabel: false, hideOverlap: true },
        splitLine: { lineStyle: { color: GRID } },
      },
      yAxis: [
        {
          type: 'category',
          inverse: true,
          // Ids, not names, which can repeat: the formatter shows the names.
          data: rows.map(({ plan }) => plan.id),
          axisTick: { show: false },
          axisLine: { show: false },
          triggerEvent: true,
          axisLabel: {
            color: TEXT,
            width: nameWidth,
            overflow: 'break',
            // A dot in the plan's color (hollow for your own plans), and the
            // name, bold on a tint of that color when pinned.
            formatter: (_id: string, index: number) => {
              const { plan } = rows[index]
              const style = app.pinned.has(plan.id) ? `pinned${index}` : 'name'
              return `{dot${index}|${plan.yours ? '◯' : '●'}} {${style}|${plan.info.name}}`
            },
            rich: {
              ...Object.fromEntries(
                rows.flatMap(({ plan }, index) => [
                  [`dot${index}`, { color: plan.color }],
                  [
                    `pinned${index}`,
                    {
                      color: TEXT,
                      fontWeight: 'bold',
                      backgroundColor: `${plan.color}40`,
                      borderRadius: 4,
                      padding: [2, 5],
                    },
                  ],
                ]),
              ),
              name: { color: TEXT },
            },
          },
        },
        {
          // Each plan's total, at the end of its bar.
          type: 'category',
          inverse: true,
          position: 'right',
          data: rows.map(({ outcome }) => compactShekels(outcome.fees.total)),
          axisTick: { show: false },
          axisLine: { show: false },
          axisLabel: { color: TEXT, fontWeight: 'bold' },
        },
      ],
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        valueFormatter: (value) => shekels(value as number),
      },
      // No ids: ECharts would then keep each series' old position, and a
      // focused fee wouldn't move to the start of the bars.
      series: [
        {
          // Around each pinned or hovered plan, from its name to its total, an
          // outline in its color. Only an outline: filled colors in the bars
          // are always fees, and the plans' colors look much like the fees'.
          // Pinned ones are drawn here; a hovered one is highlighted (see
          // `setup`), since redrawing would lose a tap that's also a hover.
          type: 'custom',
          silent: true,
          tooltip: { show: false },
          data: rows.map((_, index) => [0, index]),
          renderItem: (_params, api) => {
            const index = api.value(1) as number
            const { color, id } = rows[index].plan
            const [, middle] = api.coord([0, index])
            const [, height] = api.size!([0, 1]) as number[]
            return {
              type: 'rect',
              shape: {
                x: 1,
                y: middle - height / 2 + 2,
                width: api.getWidth() - 2,
                height: height - 4,
                r: 6,
              },
              style: {
                fill: 'transparent',
                stroke: app.pinned.has(id) ? color : 'transparent',
                lineWidth: 1.5,
              },
              emphasis: { style: { stroke: color } },
            }
          },
        },
        ...stacking.map((type): BarSeriesOption => ({
          name: type.name,
          type: 'bar',
          stack: 'fees',
          barWidth: 18,
          // Once plans are pinned, the others fade a little.
          data: rows.map(({ plan, outcome }) => ({
            value: outcome.fees[type.key],
            itemStyle: { opacity: app.pinned.size === 0 || app.pinned.has(plan.id) ? 1 : 0.6 },
          })),
          color: colorOf(type),
          // A thin gap between the parts of a bar.
          itemStyle: { borderColor: SURFACE, borderWidth: 1 },
          // Amounts only where they fit.
          label: {
            show: true,
            position: 'inside',
            color: readableOn(type.color),
            fontSize: 11,
            formatter: ({ value }) =>
              fits(value as number) && colorOf(type) !== FADED ? compactShekels(value as number) : '',
          },
        })),
      ],
    }
  }

  function overTimeOption(): ChartOption {
    const byYear = shown.outcome.feesUpToYear
    return {
      grid: { left: 4, right: 12, top: 12, bottom: 28 },
      xAxis: {
        type: 'value',
        name: 'Years',
        nameLocation: 'middle',
        nameGap: 24,
        nameTextStyle: { color: WEAK },
        min: 0,
        max: byYear.length,
        minInterval: 1,
        axisLabel: { color: WEAK },
        axisLine: { lineStyle: { color: GRID } },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        axisLabel: { formatter: compactShekels, color: WEAK },
        splitLine: { lineStyle: { color: GRID } },
      },
      tooltip: {
        trigger: 'axis',
        valueFormatter: (value) => shekels(value as number),
        axisPointer: { label: { formatter: ({ value }) => `After ${value} years` } },
      },
      series: stacking.map((type) => ({
        name: type.name,
        type: 'line',
        stack: 'over time',
        symbol: 'none',
        color: colorOf(type),
        lineStyle: { width: 1.5 },
        areaStyle: { opacity: 0.85 },
        // Nothing paid at the start, then the total by the end of each year.
        data: [[0, 0], ...byYear.map((fees, year) => [year + 1, fees[type.key]])],
      })),
    }
  }

  /** Hovering a bar or a plan's name shows it over time; clicking pins it. */
  function setup(instance: ECharts) {
    const planAt = (event: { componentType?: string; dataIndex?: number; value?: unknown }) =>
      event.componentType === 'yAxis'
        ? rows.find(({ plan }) => plan.id === event.value)?.plan
        : rows[event.dataIndex ?? -1]?.plan
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
      const index = rows.findIndex(({ plan }) => plan.id === app.hovered)
      instance.dispatchAction({ type: 'downplay', seriesIndex: 0 })
      if (index >= 0) instance.dispatchAction({ type: 'highlight', seriesIndex: 0, dataIndex: index })
    })
  }
</script>

<!-- The legend is also the control: a fee's button compares the plans by
     it. One row where it fits, else two by two. -->
<p class="label">
  <span class="mouse">Click</span><span class="touch">Tap</span> a fee to compare the plans by it
</p>
<div class="fees" role="group" aria-label="Compare the plans by">
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
      <span class="swatch" style:background={colorOf(type)}></span>{type.name}
    </button>
    <div class="popover" popover="manual" role="tooltip" bind:this={popovers[index]}>
      <p>{type.explanation}</p>
    </div>
  {/each}
</div>

<div
  class="bars"
  bind:clientWidth={barsWidth}
  style:height="{rows.length * 34 + 32}px"
  {@attach chart(barsOption, setup)}
  role="img"
  aria-label="Fees paid by each plan over the whole period, by kind"
></div>

{#if notOffered.length > 0}
  <p class="hint">Not offered for {app.purchase}: {notOffered.join(', ')}</p>
{/if}

<h4>
  Year by year: <span class="dot" style:background={shown.plan.color}></span>{shown.plan.info.name}
  <span class="hint mouse">· hover or pin another plan to see it</span>
  <span class="hint touch">· tap another plan's bar to see it</span>
</h4>
<div
  class="over-time"
  {@attach chart(overTimeOption)}
  role="img"
  aria-label="{shown.plan.info.name}'s fees piling up year by year, by kind"
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
  /* Two by two where one row doesn't fit. */
  @container (width < 440px) {
    .fees {
      display: grid;
      grid-template-columns: 1fr 1fr;
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
  .fee[aria-pressed='true'] {
    border-color: var(--fee-color);
    background: var(--fee-color);
    color: var(--on-fee-color);
    font-weight: 600;
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
