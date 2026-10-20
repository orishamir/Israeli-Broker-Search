<script lang="ts">
  // "מה יהיה לכם ב‑…" (REDESIGN.md §2.6): every option as a row on one ₪
  // axis, in today's money and after tax, beside what was put in (solid) and
  // the current account (dashed). The rows are the list of options: a tap
  // shows the row's "איך עושים את זה" (`app.focusRow`); it never changes
  // the answer. Rows the visitor's choices leave out dim. The index's row
  // carries the return its figure assumes, which can be set right there.
  // Where everything goes is `outcome.ts`'s.
  import type { AppState } from './app.svelte'
  import { kindColor } from './colors'
  import * as core from './core/core'
  import { shekels } from './format'
  import { HeldWhileTyping } from './held.svelte'
  import { kindKey } from './kinds'
  import NumberField from './NumberField.svelte'
  import {
    assumedText,
    at,
    fieldRow,
    isFocused,
    layoutRow,
    legendKeys,
    notes,
    rowLabel,
    ticks,
    tickLabelX,
    type Span,
  } from './outcome'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const words = core.outcomeWords()
  const held = new HeldWhileTyping(() => app.answer?.outcome)
  const outcome = $derived(held.current)

  /** The tracks' width, px: every row's is the axis row's. */
  let width = $state(0)
  let axisTrack: HTMLElement | null = $state(null)
  $effect(() => {
    if (!axisTrack) return
    const observer = new ResizeObserver(([entry]) => {
      const next = Math.round(entry.contentRect.width)
      if (next > 0 && next !== width) width = next
    })
    observer.observe(axisTrack)
    return () => observer.disconnect()
  })

  // Labels are measured on a canvas in the labels' own font.
  const LABEL_FONT_PX = 12
  let context: CanvasRenderingContext2D | null = null
  const measure = (text: string): number => {
    context ??= document.createElement('canvas').getContext('2d')
    if (!context) return text.length * 7
    context.font = `${LABEL_FONT_PX}px ${getComputedStyle(document.documentElement).fontFamily}`
    return Math.ceil(context.measureText(text).width) + 6
  }
  /** The field's box, px, and the gap before "בשנה". */
  const FIELD_WIDTH = 66
  const FIELD_GAP = 6

  const percent = $derived(app.yearlyReturnPercent ?? 0)
  const span: Span = $derived({ min: outcome?.axis[0] ?? 0, max: outcome?.axis[1] ?? 1 })
  const field = $derived(outcome ? fieldRow(outcome.rows) : -1)
  const plain = $derived(assumedText(percent, words))
  const layouts = $derived.by(() => {
    if (!outcome) return []
    const track = width || 400
    return outcome.rows.map((row, index) =>
      layoutRow(
        row,
        span,
        track,
        words,
        {
          text: measure,
          assumed: index === field ? FIELD_WIDTH + FIELD_GAP + measure(words.perYear) : measure(plain),
          assumedText: plain,
        },
        index === field,
      ),
    )
  })
  const rowNotes = $derived(outcome ? notes(outcome.rows) : [])
  const keys = $derived(outcome ? legendKeys(outcome) : { known: false, market: false })
  const axisTicks = $derived(ticks(span, width || 400))
  const deposited = $derived(outcome ? at(span, outcome.deposited.today) : 0)
  const cash = $derived(outcome ? at(span, outcome.currentAccount.today) : 0)
  const share = (fraction: number) => `${(Math.min(1, Math.max(0, fraction)) * 100).toFixed(3)}%`
</script>

{#if outcome && outcome.rows.length > 0}
  <section class="card outcome" class:stale={app.stale} aria-labelledby="outcome-title">
    <div class="heading">
      <h2 id="outcome-title">{outcome.title}</h2>
      <p class="subtitle">{words.subtitle}</p>
    </div>

    <ul class="legend">
      <li>
        <span class="sample line deposited" aria-hidden="true"></span>
        {words.deposited}:&nbsp;<strong class="num">{shekels(outcome.deposited.today)}</strong>
      </li>
      <li>
        <span class="sample line cash" aria-hidden="true"></span>
        {words.currentAccount}:&nbsp;<strong class="num">{shekels(outcome.currentAccount.today)}</strong>
      </li>
      {#if keys.known}
        <li><span class="sample known" aria-hidden="true"></span>{words.known}</li>
      {/if}
      {#if keys.market}
        <li><span class="sample range" aria-hidden="true"></span>{words.market}</li>
      {/if}
    </ul>

    <div class="rows" role="list">
      {#each outcome.rows as row, index (kindKey(row.id))}
        {@const layout = layouts[index]}
        {@const focused = isFocused(row, app.focusedRow)}
        {@const color = kindColor(row.id)}
        <div
          class="row"
          role="listitem"
          class:dim={!row.active && !focused}
          class:focused
          style:--kind={color}
          style:--track-height="{layout.height}px"
        >
          <!-- The row's button lies under its content, which lets the pointer
               through: a field can't sit inside a button, and the return's does. -->
          <button
            type="button"
            class="press"
            aria-pressed={focused}
            aria-label={rowLabel(row, words, percent)}
            onclick={() => app.focusRow(row.id)}
          ></button>
          <span class="name" aria-hidden="true">
            <span class="titled"><span class="swatch"></span><span>{row.name}</span></span>
            {#if row.tags.length > 0}
              <span class="tags">
                {#each row.tags as tag, at (at)}<span class="tag">{tag}</span>{/each}
              </span>
            {/if}
          </span>
          <span class="track" dir="ltr">
            <span class="marks" aria-hidden="true">
              <span class="ref deposited" style:inset-inline-start={share(deposited)}></span>
              <span class="ref cash" style:inset-inline-start={share(cash)}></span>
              {#if layout.bar}
                <span
                  class="bar {layout.bar.kind}"
                  class:cut-low={layout.bar.cutLow}
                  class:cut-high={layout.bar.cutHigh}
                  style:top="{layout.middle}px"
                  style:inset-inline-start={share(layout.bar.from)}
                  style:width={share(layout.bar.to - layout.bar.from)}
                ></span>
              {/if}
              {#each layout.ticks as tick (tick.role)}
                <span
                  class="tick {tick.role}"
                  style:top="{layout.middle}px"
                  style:inset-inline-start={share(tick.at)}
                ></span>
              {/each}
              {#if layout.dot !== undefined}
                <span class="dot" style:top="{layout.middle}px" style:inset-inline-start={share(layout.dot)}
                ></span>
              {/if}
              {#each layout.labels as label (label.key)}
                {#if !(label.key === 'assumed' && index === field)}
                  <!-- Placed in the track's left-to-right terms; the words inside read right to left. -->
                  <span
                    class="label {label.key}"
                    style:top="{label.top}px"
                    style:inset-inline-start="{label.x}px"
                    style:width="{label.width}px"><bdi dir="rtl">{label.text}</bdi></span
                  >
                {/if}
              {/each}
            </span>
            {#if index === field}
              {@const label = layout.labels.find(({ key }) => key === 'assumed')}
              {#if label}
                <!-- Left to right on the track: "בשנה" first, so the field reads first. -->
                <span
                  class="return"
                  style:top="{label.top}px"
                  style:inset-inline-start="{label.x}px"
                  style:width="{label.width}px"
                >
                  <span aria-hidden="true">{words.perYear}</span>
                  <span class="box" dir="rtl" style:width="{FIELD_WIDTH}px">
                    <NumberField
                      id="outcome-return"
                      suffix="%"
                      step={0.5}
                      label={t.outcomeReturnLabel}
                      bind:value={app.yearlyReturnPercent}
                    />
                  </span>
                </span>
              {/if}
            {/if}
          </span>
          <!-- An excluded option shows no end figure, only its reason (a note). -->
          <span class="value num" aria-hidden="true">{row.excluded ? '' : row.amount}</span>
          {#each rowNotes[index] as note, at (at)}
            <p class="note">{note}</p>
          {/each}
        </div>
      {/each}
      <div class="row axis" aria-hidden="true">
        <span class="track" dir="ltr" bind:this={axisTrack}>
          {#each axisTicks as tick (tick.value)}
            {@const labelWidth = measure(tick.label)}
            <span class="axis-mark" style:inset-inline-start={share(tick.at)}></span>
            <span
              class="axis-tick"
              style:inset-inline-start="{tickLabelX(tick, width || 400, labelWidth)}px"
              style:width="{labelWidth}px">{tick.label}</span
            >
          {/each}
        </span>
      </div>
    </div>
  </section>
{/if}

<style>
  .outcome {
    /* On the card, not the rows: the legend's sample uses it too (unset
       there, its line wasn't drawn at all). */
    --reference: color-mix(in srgb, var(--text) 75%, transparent);

    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: opacity var(--quick);
  }
  .stale {
    opacity: 0.55;
  }
  .heading {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  h2 {
    margin: 0;
  }
  .subtitle {
    margin: 0;
    color: var(--weak);
  }

  /* ── The legend ── */
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 20px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.875rem;
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .sample {
    flex: none;
  }
  .sample.line {
    width: 0;
    height: 18px;
  }
  .sample.deposited,
  .ref.deposited {
    border-inline-start: 2px solid var(--reference);
  }
  .sample.cash,
  .ref.cash {
    border-inline-start: 2px dashed var(--weak);
  }
  .sample.known {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text);
  }
  .sample.range {
    box-sizing: border-box;
    width: 30px;
    height: 10px;
    border: 1.5px solid var(--text);
    border-radius: 5px;
  }

  /* ── The rows: the name, the track, the amount; on a narrow card the
     track goes under the name and the amount ── */
  .rows {
    --name-column: 15rem;
    --value-column: 7.5rem;

    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--border);
  }
  .row {
    --row-background: var(--surface);

    position: relative;
    display: grid;
    grid-template-columns: var(--name-column) minmax(0, 1fr) var(--value-column);
    align-items: center;
    min-height: 56px;
    padding-block: 8px;
    border-bottom: 1px solid var(--border);
    background: var(--row-background);
    transition:
      opacity var(--settle),
      background-color var(--quick);
  }
  .row.dim {
    opacity: 0.5;
  }
  .row.focused {
    --row-background: var(--raised);
  }
  /* The whole row is the button; what's drawn over it lets the pointer
     through, except the return's field. */
  .press {
    position: absolute;
    inset: 0;
    width: 100%;
    margin: 0;
    padding: 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    cursor: pointer;
  }
  .press:hover {
    background: color-mix(in srgb, var(--text) 4%, transparent);
  }
  .press:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .press[aria-pressed='true'] {
    box-shadow: inset -3px 0 0 var(--strong-border);
  }
  .name,
  .track,
  .value,
  .note {
    position: relative;
    pointer-events: none;
  }
  .name {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-inline: 12px;
    font-weight: 600;
  }
  .titled {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .swatch {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--kind);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag {
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--raised-strong);
    color: var(--weak);
    font-size: 0.8125rem;
    font-weight: 400;
  }
  .value {
    padding-inline: 12px;
    font-weight: 700;
    text-align: end;
  }
  .num {
    font-variant-numeric: tabular-nums;
  }
  .note {
    grid-column: 1 / -1;
    margin: 0;
    padding: 4px 12px 0;
    color: var(--weak);
    font-size: 0.875rem;
  }

  /* ── The track: everything on it is placed by outcome.ts ── */
  .track {
    position: relative;
    display: block;
    height: var(--track-height);
    margin-inline: 8px;
  }
  .marks {
    position: absolute;
    inset: 0;
  }
  .ref {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    margin-inline-start: -1px;
    transition: inset-inline-start var(--settle);
  }
  .bar,
  .tick,
  .dot {
    position: absolute;
    translate: 0 -50%;
    transition:
      inset-inline-start var(--settle),
      width var(--settle),
      top var(--settle);
  }
  .bar.rate {
    height: 8px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--kind) 45%, transparent);
  }
  .bar.market {
    box-sizing: border-box;
    height: 14px;
    border: 1.5px solid var(--kind);
    border-radius: 7px;
    background: color-mix(in srgb, var(--kind) 22%, transparent);
  }
  /* A tail past the axis: the bar runs to the edge, open, with an arrow. */
  .bar.cut-low {
    border-start-start-radius: 0;
    border-end-start-radius: 0;
    border-inline-start-style: dotted;
  }
  .bar.cut-high {
    border-start-end-radius: 0;
    border-end-end-radius: 0;
    border-inline-end-style: dotted;
  }
  .bar.cut-low::before,
  .bar.cut-high::after {
    content: '';
    position: absolute;
    top: 50%;
    translate: 0 -50%;
    border-block: 6px solid transparent;
  }
  .bar.cut-low::before {
    inset-inline-start: -2px;
    border-inline-end: 8px solid var(--kind);
  }
  .bar.cut-high::after {
    inset-inline-end: -2px;
    border-inline-start: 8px solid var(--kind);
  }
  .tick {
    margin-inline-start: -2px;
    border-radius: 2px;
  }
  .tick.assumed {
    width: 4px;
    height: 24px;
    background: var(--kind);
    box-shadow: 0 0 0 2px var(--row-background);
  }
  .tick.median {
    width: 2px;
    height: 18px;
    margin-inline-start: -1px;
    background: var(--text);
  }
  .dot {
    box-sizing: border-box;
    width: 14px;
    height: 14px;
    margin-inline-start: -7px;
    border-radius: 50%;
    background: var(--kind);
    box-shadow: 0 0 0 2px var(--row-background);
  }
  .label {
    position: absolute;
    box-sizing: border-box;
    padding-inline: 3px;
    border-radius: 3px;
    background: var(--row-background);
    color: var(--weak);
    font-size: 12px;
    line-height: 16px;
    white-space: nowrap;
    text-align: center;
    transition:
      inset-inline-start var(--settle),
      top var(--settle);
  }
  .label.assumed {
    color: var(--text);
    font-weight: 600;
  }

  /* The return's field sits on the track, over the row's button: only it
     takes the pointer. */
  .return {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    font-size: 13px;
    font-weight: 600;
    pointer-events: auto;
    transition: inset-inline-start var(--settle);
  }
  .return .box {
    flex: none;
  }
  .return .box :global(input) {
    padding-block: 4px;
  }

  /* ── The axis ── */
  .axis {
    min-height: 0;
    padding-block: 0;
    border-bottom: 0;
  }
  .axis .track {
    grid-column: 2;
    height: 26px;
  }
  .axis-mark {
    position: absolute;
    top: 0;
    width: 0;
    height: 4px;
    border-inline-start: 1px solid var(--weak);
    transition: inset-inline-start var(--settle);
  }
  .axis-tick {
    position: absolute;
    top: 6px;
    text-align: center;
    color: var(--weak);
    font-size: 12px;
    white-space: nowrap;
    transition: inset-inline-start var(--settle);
  }

  @container (width < 40rem) {
    .row {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .name {
      grid-row: 1;
      grid-column: 1;
    }
    .value {
      grid-row: 1;
      grid-column: 2;
      align-self: start;
    }
    .track {
      grid-row: 2;
      grid-column: 1 / -1;
      margin-block-start: 8px;
    }
    .axis .track {
      grid-row: 1;
      grid-column: 1 / -1;
      margin-block-start: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .row,
    .ref,
    .bar,
    .tick,
    .dot,
    .label,
    .return,
    .axis-mark,
    .axis-tick {
      transition: none;
    }
  }
</style>
