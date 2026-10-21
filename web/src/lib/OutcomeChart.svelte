<script lang="ts">
  // "מה יהיה לכם ב‑…" (REDESIGN.md §2.6, amended by §4.6): every option the
  // visitor could choose as a row on one ₪ axis, in today's money and after
  // tax, through which run what was put in (solid) and the current account
  // (dashed), each labelled with its amount. A row has its name with its
  // amount beside it, its facts in plain words, and its marks; a stock
  // row's band labels every mark with its amount, and the first stock row
  // carries the return its figures assume, as a field: "אם המדד יעלה [10]%
  // בשנה". The rows are the list of options: a tap shows the row's "איך
  // עושים את זה" (`app.focusRow`); it never changes the answer. Rows the
  // visitor's choices leave out dim. Where everything goes is `outcome.ts`'s.
  import type { AppState } from './app.svelte'
  import { kindColor } from './colors'
  import * as core from './core/core'
  import { HeldWhileTyping } from './held.svelte'
  import { kindKey } from './kinds'
  import NumberField from './NumberField.svelte'
  import {
    assumedText,
    at,
    canvasMeasure,
    fieldRow,
    isFocused,
    LANE,
    layoutRow,
    LINE_LANES,
    lineLabels,
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
  const measure = canvasMeasure(LABEL_FONT_PX)

  const percent = $derived(app.yearlyReturnPercent ?? 0)
  const span: Span = $derived({ min: outcome?.axis[0] ?? 0, max: outcome?.axis[1] ?? 1 })
  const track = $derived(width || 400)
  const field = $derived(outcome ? fieldRow(outcome.rows) : -1)
  const plain = $derived(assumedText(percent, words))
  const layouts = $derived(
    outcome
      ? outcome.rows.map((row) => layoutRow(row, span, track, { text: measure, assumedText: plain }))
      : [],
  )
  const rowNotes = $derived(outcome ? notes(outcome.rows) : [])
  const axisTicks = $derived(ticks(span, track))
  const lines = $derived(outcome ? lineLabels(outcome.lines, span, track, measure) : [])
  const share = (fraction: number) => `${(Math.min(1, Math.max(0, fraction)) * 100).toFixed(3)}%`
  const lineAt = (which: core.RefLineKind) =>
    share(at(span, outcome?.lines.find((line) => line.which === which)?.value.today ?? span.min))
</script>

{#snippet refs()}
  <span class="ref deposited" style:inset-inline-start={lineAt('Deposited')}></span>
  <span class="ref cash" style:inset-inline-start={lineAt('CurrentAccount')}></span>
{/snippet}

{#if outcome && outcome.rows.length > 0}
  <section class="card outcome" class:stale={app.stale} aria-labelledby="outcome-title">
    <div class="heading">
      <h2 id="outcome-title">{outcome.title}</h2>
      <p class="caption">{outcome.caption}</p>
    </div>

    <!-- The lines' amounts, for a screen reader; drawn beside the lines. -->
    <ul class="visually-hidden">
      {#each outcome.lines as line (line.which)}<li>{line.label} {line.amount}</li>{/each}
    </ul>

    <div class="rows" role="list">
      <div class="row lines" aria-hidden="true">
        <span class="track" dir="ltr" style:--track-height="{LINE_LANES * LANE + 6}px">
          {@render refs()}
          {#each lines as label (label.key)}
            <span
              class="line-label {label.key}"
              style:top="{label.top}px"
              style:inset-inline-start="{label.x}px"
              style:width="{label.width}px"><bdi dir="rtl">{label.text}</bdi></span
            >
          {/each}
        </span>
      </div>
      {#each outcome.rows as row, index (kindKey(row.id))}
        {@const layout = layouts[index]}
        {@const focused = isFocused(row, app.focusedRow)}
        <div
          class="row"
          role="listitem"
          class:dim={!row.active && !focused}
          class:focused
          style:--kind={kindColor(row.id)}
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
          <span class="head">
            <span class="titled" aria-hidden="true"
              ><span class="swatch"></span><span class="name">{row.name}</span></span
            >
            <!-- An excluded option shows no end figure, only its reason (a note). -->
            <span class="value num" aria-hidden="true">{row.excluded ? '' : row.amount}</span>
            <span class="sub">
              {#if row.facts}<span class="facts" aria-hidden="true">{row.facts}</span>{/if}
              {#if index === field}
                <span class="return">
                  <span aria-hidden="true">{row.returnLead}</span>
                  <span class="box">
                    <NumberField
                      id="outcome-return"
                      suffix="%"
                      step={0.5}
                      label={t.outcomeReturnLabel}
                      bind:value={app.yearlyReturnPercent}
                    />
                  </span>
                  <span aria-hidden="true">{words.perYear}</span>
                </span>
              {/if}
            </span>
          </span>
          <!-- The lines run the row's whole height, behind its marks. -->
          <span class="refs" dir="ltr" aria-hidden="true">{@render refs()}</span>
          <span class="track" dir="ltr" style:--track-height="{layout.height}px">
            <span class="marks" aria-hidden="true">
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
                <!-- Placed in the track's left-to-right terms; the words inside read right to left. -->
                <span
                  class="label {label.key}"
                  style:top="{label.top}px"
                  style:inset-inline-start="{label.x}px"
                  style:width="{label.width}px"><bdi dir="rtl">{label.text}</bdi></span
                >
              {/each}
            </span>
          </span>
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
              style:inset-inline-start="{tickLabelX(tick, track, labelWidth)}px"
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
  .caption {
    margin: 0;
    color: var(--weak);
  }
  /* ── The rows: the name with its amount and facts, then the track; on a
     narrow card the track goes under them ── */
  .rows {
    --head-column: 21rem;
    --value-column: 8.5rem;

    display: flex;
    flex-direction: column;
  }
  .row {
    --row-background: var(--surface);

    position: relative;
    display: grid;
    grid-template-columns: var(--head-column) minmax(0, 1fr);
    align-items: start;
    padding-block: 10px;
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
  .row.lines {
    padding-block: 0;
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
  .head,
  .track,
  .note {
    position: relative;
    pointer-events: none;
  }
  /* Name and amount on the first line, the amount in a column of its own
     width so a new amount never rewraps the name (principle 21); the facts
     and the return's field on the second. */
  .head {
    grid-row: 1;
    grid-column: 1;
    display: grid;
    grid-template-columns: minmax(0, 1fr) var(--value-column);
    gap: 4px 10px;
    align-items: baseline;
    padding-inline: 12px;
  }
  .titled {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-weight: 700;
  }
  .swatch {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--kind);
  }
  .value {
    font-weight: 700;
    text-align: end;
    white-space: nowrap;
  }
  .num {
    font-variant-numeric: tabular-nums;
  }
  .sub {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 4px 12px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  .sub:empty {
    display: none;
  }
  .return {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
    white-space: nowrap;
    pointer-events: auto;
  }
  .return .box {
    flex: none;
    width: 4.75rem;
  }
  .return .box :global(input) {
    padding-block: 4px;
  }
  .note {
    grid-column: 1 / -1;
    margin: 0;
    padding: 6px 12px 0;
    color: var(--weak);
    font-size: 0.875rem;
  }

  /* ── The track: everything on it is placed by outcome.ts ── */
  .track {
    position: relative;
    grid-row: 1;
    grid-column: 2;
    display: block;
    align-self: center;
    height: var(--track-height);
    margin-inline: 8px;
  }
  .marks {
    position: absolute;
    inset: 0;
  }
  .refs {
    position: relative;
    grid-row: 1;
    grid-column: 2;
    align-self: stretch;
    margin-block: -10px;
    margin-inline: 8px;
    pointer-events: none;
  }
  .ref {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    margin-inline-start: -1px;
    transition: inset-inline-start var(--settle);
  }
  .ref.deposited {
    border-inline-start: 2px solid var(--reference);
  }
  .ref.cash {
    border-inline-start: 2px dashed var(--weak);
  }
  .line-label {
    position: absolute;
    box-sizing: border-box;
    padding-inline: 3px;
    border-radius: 3px;
    background: var(--surface);
    color: var(--weak);
    font-size: 12px;
    line-height: 16px;
    white-space: nowrap;
    text-align: center;
    transition:
      inset-inline-start var(--settle),
      top var(--settle);
  }
  .line-label.Deposited {
    color: var(--text);
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
    background: var(--text);
    box-shadow: 0 0 0 2px var(--row-background);
  }
  .tick.median {
    width: 2px;
    height: 18px;
    margin-inline-start: -1px;
    background: color-mix(in srgb, var(--text) 60%, transparent);
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

  /* ── The axis ── */
  .axis {
    padding-block: 0;
    border-bottom: 0;
  }
  .axis .track {
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
      grid-template-columns: minmax(0, 1fr);
    }
    .rows {
      --value-column: auto;
    }
    .track {
      grid-row: 2;
      grid-column: 1;
    }
    .row.lines .track,
    .row.axis .track {
      grid-row: 1;
    }
    .row:not(.lines, .axis) .track {
      margin-block-start: 10px;
    }
    .refs {
      grid-row: 2;
      grid-column: 1;
      margin-block: 10px -10px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .row,
    .ref,
    .bar,
    .tick,
    .dot,
    .label,
    .line-label,
    .axis-mark,
    .axis-tick {
      transition: none;
    }
  }
</style>
