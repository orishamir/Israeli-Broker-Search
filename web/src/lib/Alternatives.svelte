<script lang="ts">
  // "לעומת האפשרויות האחרות": the chart (`alternativesOption`, from
  // `app.alternatives`, held while a number is typed) and the rows under it
  // (the kinds drawn, with the reason each isn't what fits from
  // `recommendation.alternatives`). Tapping a row goes on with that kind
  // (`app.chooseKind`); hovering a row or a line, or pinning a line, links
  // them (`linked.svelte.ts`). Screens s2, s3.
  import { SvelteSet } from 'svelte/reactivity'
  import type { AppState } from './app.svelte'
  import { alternativesOption } from './alternatives-chart'
  import { kindColor } from './colors'
  import { chart } from './echarts.svelte'
  import { shekels } from './format'
  import { HeldWhileTyping } from './held.svelte'
  import { kindKey } from './kinds'
  import { linkLines, type Linked } from './linked.svelte'
  import { glideHeight } from './motion'
  import { touchScreen } from './pointer'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.alternatives)
  const reasons = new HeldWhileTyping(() => app.recommendation?.alternatives)

  // The rows and the lines are linked by the kind's key.
  const linked: Linked = $state({
    hovered: null,
    pinned: new SvelteSet<string>(),
    togglePin(id: string) {
      if (!this.pinned.delete(id)) this.pinned.add(id)
    },
  })

  const rows = $derived(
    (held.current?.series ?? [])
      .map((line) => ({
        line,
        id: kindKey(line.kind),
        color: kindColor(line.kind),
        why: reasons.current?.find((alternative) => kindKey(alternative.kind) === kindKey(line.kind))?.whyNot
          .text,
      }))
      // What fits first; the rest as the core ranks them.
      .toSorted((a, b) => Number(b.line.recommended) - Number(a.line.recommended)),
  )

  const context = document.createElement('canvas').getContext('2d')!
  const font = getComputedStyle(document.documentElement).fontFamily
  const measure = (text: string) => {
    context.font = `12px ${font}`
    return context.measureText(text).width
  }

  const option = () =>
    alternativesOption({
      data: held.current ?? { months: 1, series: [], band: undefined },
      highlighted: linked.pinned,
      colors: new Map(rows.map(({ id, color }) => [id, color])),
      touch: touchScreen,
      measure,
    })

  const band = $derived(held.current?.band)
</script>

{#if held.current}
  <section class="card alternatives" {@attach glideHeight()}>
    <div>
      <h2 class="titled">{t.alternativesTitle}<Tip about={t.alternativesTitle}>{t.alternativesTip}</Tip></h2>
      <p class="lead">{t.alternativesLead}</p>
      <div class="chart" {@attach chart(option, linkLines(linked, ''))}></div>
      {#if band}
        <p class="caption">
          <strong>{t.bandCaption}:</strong>
          {band.sentence}
          <span class="caveat">{band.caveat}</span>
        </p>
      {/if}
      <ul class="rows">
        {#each rows as row (row.id)}
          <li
            class="row"
            class:hovered={linked.hovered === row.id}
            class:pinned={linked.pinned.has(row.id)}
            onpointerenter={() => {
              if (!touchScreen) linked.hovered = row.id
            }}
            onpointerleave={() => (linked.hovered = null)}
          >
            <span class="name">
              <span class="dot" style:background={row.color}></span>
              {#if row.line.recommended}
                <span class="fits">{t.rowRecommended}</span>
              {/if}
              <button
                type="button"
                class="choose"
                disabled={row.line.recommended}
                aria-label={row.line.recommended ? undefined : t.continueWith(row.line.name)}
                onclick={() => app.chooseKind(row.line.kind)}>{row.line.name}</button
              >
            </span>
            <bdi class="amount" dir="ltr">{shekels(row.line.left.today)}</bdi>
            <span class="tag-line">
              <span class="tag">{row.line.safetyTag}</span><Tip about={row.line.safetyTag}
                >{row.line.safetyNote}</Tip
              >
            </span>
            {#if row.why}
              <p class="why">{row.why}</p>
            {/if}
          </li>
        {/each}
      </ul>
      <p class="hint">{t.continueHint}</p>
    </div>
  </section>
{/if}

<style>
  .lead,
  .hint {
    margin: 0 0 8px;
    color: var(--weak);
    font-size: 0.85rem;
  }

  .chart {
    width: 100%;
    height: 280px;
  }

  .caption {
    margin: 4px 0 12px;
    font-size: 0.8rem;
    color: var(--weak);
  }

  .caption strong {
    color: var(--text);
    font-weight: 600;
  }

  .caveat {
    display: block;
    margin-block-start: 2px;
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: baseline;
    gap: 2px 12px;
    padding: 10px 4px;
    border-block-start: 1px solid var(--border);
    transition: background var(--quick);
  }

  .row.hovered,
  .row.pinned {
    background: var(--accent-tint);
  }

  .name {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .dot {
    flex: none;
    width: 9px;
    height: 9px;
    border-radius: 50%;
  }

  .fits {
    padding: 1px 6px;
    border-radius: 6px;
    background: var(--good-tint);
    color: var(--good);
    font-size: 0.72rem;
  }

  /* The whole row is the button: its box stretches over the row, and the
     tip's "?" sits above it. */
  .choose {
    padding: 0;
    border: none;
    background: none;
    color: var(--text);
    font: inherit;
    font-weight: 600;
    text-align: start;
  }

  .choose:enabled {
    cursor: pointer;
  }

  .choose::after {
    content: '';
    position: absolute;
    inset: 0;
  }

  .choose:disabled {
    opacity: 1;
    cursor: default;
    color: var(--text);
  }

  .choose:hover:not(:disabled) {
    border: none;
    background: none;
  }

  .amount {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .tag-line {
    position: relative;
    z-index: 1;
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    width: fit-content;
  }

  .tag {
    padding: 1px 8px;
    border: 1px solid var(--strong-border);
    border-radius: 999px;
    color: var(--weak);
    font-size: 0.72rem;
  }

  .why {
    grid-column: 1 / -1;
    margin: 2px 0 0;
    color: var(--weak);
    font-size: 0.8rem;
  }
</style>
