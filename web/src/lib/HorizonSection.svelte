<script lang="ts">
  // "ואם התאריך יזוז?" (REDESIGN.md §2.9, amended by §4.10): the same
  // deposits at dates around the visitor's (`app.horizons`, from the core:
  // the worker's job after the answer's, so the card above never waits for
  // it; until the first rows come, the section holds their place, empty and
  // marked busy). A table: מתי, with the visitor's own row marked "התאריך
  // שלכם"; מה מתאים אז, one line per answer about the date where that
  // decides it ("אם תצטרכו בדיוק אז:", "אם תוכלו לחכות:"); כמה יהיה; and
  // הפקדתם. The stocks' range is said once, under the table. A line's
  // button moves the date there, and gives that answer about the date if
  // the line is for one, so the answer above becomes exactly the line.
  import type { AppState } from './app.svelte'
  import type { HorizonFitData, HorizonRowData } from './core/core'
  import { HeldWhileTyping } from './held.svelte'
  import { glideHeight } from './motion'
  import { showAnswer } from './scroll'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.horizons)
  const rows = $derived(held.current ?? [])
  /** Before the first rows: their places, empty. */
  const pending = $derived(held.current === undefined)

  const isStocks = (fit: HorizonFitData) => typeof fit.kind === 'object' && 'Way' in fit.kind
  /** The note every stock range shares (the past's stretches, by the S&P
   * 500 as the example): said once, under the table (principle 13). A
   * stock note of its own (a date past the history) stays by its range. */
  const stockNote = $derived.by(() => {
    const notes = rows.flatMap((row) => row.fits.filter(isStocks).map((fit) => fit.rangeNote))
    const counts = new Map<string, number>()
    for (const note of notes) counts.set(note, (counts.get(note) ?? 0) + 1)
    return [...counts].sort((a, b) => b[1] - a[1])[0]?.[0]
  })
  const noteFor = (fit: HorizonFitData) => (isStocks(fit) && fit.rangeNote === stockNote ? '' : fit.rangeNote)

  /** Already the answer above: the visitor's date, with nothing to answer. */
  const isShown = (row: HorizonRowData, fit: HorizonFitData) => row.current && fit.flexibility === undefined

  function open(row: HorizonRowData, fit: HorizonFitData) {
    app.setHorizonMonths(row.months)
    if (fit.flexibility !== undefined) app.choose({ Flexibility: fit.flexibility })
    // The answer is above the section: bring it into view. (Not its column:
    // on a phone that has no box of its own, `display: contents`.)
    showAnswer()
  }
</script>

<!-- Money that may be needed any day has no date to move: no section. -->
{#if app.answer?.moves && (pending || rows.length > 0)}
  <section
    class="card horizons"
    class:waiting={!app.horizonsFresh}
    aria-labelledby="horizons-title"
    aria-busy={!app.horizonsFresh}
    {@attach glideHeight()}
  >
    <div>
      <h2 id="horizons-title">{t.horizonsTitle}</h2>
      <p class="lead">{t.horizonsLead}</p>
      {#if pending}
        <div class="placeholder" aria-hidden="true"></div>
      {:else}
        <table>
          <thead>
            <tr>
              <th scope="col">{t.horizonsWhen}</th>
              <th scope="col">{t.horizonsWhat}</th>
              <th scope="col">{t.horizonsAmount}</th>
              <th scope="col" class="deposited">{t.horizonsDeposited}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as row (row.months)}
              {#each row.fits as fit, index (`${row.months}-${fit.flexibility ?? ''}`)}
                {@const shown = isShown(row, fit)}
                {@const note = noteFor(fit)}
                <tr class:first={index === 0} class:current={row.current}>
                  {#if index === 0}
                    <th scope="row" rowspan={row.fits.length}>
                      <span class="when">{row.when}</span>
                      {#if row.label}<span class="yours">{row.label}</span>{/if}
                      <!-- On a phone the deposits go under the date: four columns don't fit. -->
                      <span class="deposits-here">{t.horizonsDeposited} {row.deposits}</span>
                    </th>
                  {/if}
                  <td class="what">
                    <button
                      type="button"
                      class="row"
                      disabled={shown}
                      aria-current={shown ? 'true' : undefined}
                      aria-label={t.horizonsOpen(
                        [row.when, fit.condition, fit.name, fit.range].filter(Boolean).join(', '),
                      )}
                      onclick={() => open(row, fit)}
                    >
                      {#if fit.condition}<span class="condition">{fit.condition}</span>{/if}
                      <span class="name">{fit.name}</span>
                    </button>
                  </td>
                  <td class="amount">
                    <span class="range">{fit.range}</span>
                    {#if note}<span class="note">{note}</span>{/if}
                  </td>
                  {#if index === 0}
                    <td class="deposited" rowspan={row.fits.length}>{row.deposits}</td>
                  {/if}
                </tr>
              {/each}
            {/each}
          </tbody>
        </table>
        {#if stockNote}<p class="footnote">{t.horizonsStockRange} {stockNote}.</p>{/if}
      {/if}
    </div>
  </section>
{/if}

<style>
  .horizons {
    transition: opacity var(--quick);
  }

  .waiting {
    opacity: 0.6;
  }

  h2 {
    margin-block-end: 4px;
  }

  .lead {
    margin: 0 0 8px;
    color: var(--weak);
  }

  /* Four dates' height, while the rows are being worked out. */
  .placeholder {
    block-size: 16rem;
  }

  table {
    inline-size: 100%;
    border-collapse: collapse;
    font-size: 0.92rem;
  }

  thead th {
    padding: 6px 8px;
    color: var(--weak);
    font-size: 0.8rem;
    font-weight: 600;
    text-align: start;
  }

  tr.first {
    border-block-start: 1px solid var(--border);
  }

  tbody th,
  td {
    padding: 8px;
    vertical-align: top;
    text-align: start;
  }

  tbody th {
    font-weight: 600;
  }

  tr.current th {
    /* The start edge, right in a right-to-left page. */
    box-shadow: inset -3px 0 0 var(--accent);
  }

  .deposits-here {
    display: none;
    margin-block-start: 4px;
    color: var(--weak);
    font-size: 0.78rem;
    font-weight: 400;
  }

  .when,
  .yours,
  .condition,
  .name,
  .range,
  .note {
    display: block;
  }

  .yours {
    color: var(--accent);
    font-size: 0.78rem;
  }

  /* A line's words are its button, drawn as text. */
  .row {
    display: block;
    inline-size: 100%;
    block-size: auto;
    min-block-size: 0;
    margin: -4px;
    padding: 4px;
    border: none;
    border-radius: 4px;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: start;
  }

  .row:hover:not(:disabled) {
    background: var(--accent-tint);
  }

  .row:disabled {
    opacity: 1;
    cursor: default;
  }

  .condition,
  .note {
    color: var(--weak);
    font-size: 0.82rem;
  }

  .name,
  .range {
    font-weight: 600;
  }

  .range,
  .deposited {
    font-variant-numeric: tabular-nums;
  }

  .deposited {
    color: var(--weak);
    white-space: nowrap;
  }

  .footnote {
    margin: 10px 0 0;
    color: var(--weak);
    font-size: 0.82rem;
  }

  @container (width < 34rem) {
    table {
      font-size: 0.85rem;
    }

    tbody th,
    td,
    thead th {
      padding-inline: 4px;
    }

    .deposited {
      display: none;
    }

    .deposits-here {
      display: block;
    }
  }
</style>
