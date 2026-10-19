<script lang="ts">
  // "ואם התאריך יזוז?" (REDESIGN.md §2.9): the same deposits at 1, 3, 5, 10
  // and 20 years, each row what the answer would be with the date there
  // (`app.answer.horizons`, from the core). Where the visitor hasn't said
  // whether the date can move and that decides the place, a row for each
  // answer, with the answer in words. A row's click moves the date there,
  // and gives that answer about the date if the row is for one, so the
  // answer above becomes exactly the row.
  import type { AppState } from './app.svelte'
  import type { HorizonRowData } from './core/core'
  import { HeldWhileTyping } from './held.svelte'
  import { glideHeight } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  let section: HTMLElement | null = $state(null)
  const held = new HeldWhileTyping(() => app.answer?.horizons)
  const rows = $derived(held.current ?? [])
  /** One sum put in once is the same at every date: said once, above the rows. */
  const sharedDeposits = $derived(
    rows.length > 0 && rows.every((row) => row.deposits === rows[0].deposits) ? rows[0].deposits : undefined,
  )

  /** Already the answer above: the visitor's date, with nothing to answer. */
  const isShown = (row: HorizonRowData) => row.current && row.flexibility === undefined

  function open(row: HorizonRowData) {
    app.setHorizonMonths(row.years * 12)
    if (row.flexibility !== undefined) app.choose('PlaceType', { Flexibility: row.flexibility })
    // The answer is above the section: bring it into view.
    const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches
    section
      ?.closest('.answer-column')
      ?.scrollIntoView({ block: 'start', behavior: reduced ? 'auto' : 'smooth' })
  }
</script>

{#if rows.length > 0}
  <section
    class="card horizons"
    class:waiting={!app.answerFresh}
    aria-labelledby="horizons-title"
    bind:this={section}
    {@attach glideHeight()}
  >
    <div>
      <h2 id="horizons-title">{t.horizonsTitle}</h2>
      <p class="lead">{sharedDeposits ? `${t.horizonsLead} ${sharedDeposits}.` : t.horizonsLead}</p>
      <ul class="rows">
        {#each rows as row, at (`${row.years}-${row.flexibility ?? ''}`)}
          {@const again = at > 0 && rows[at - 1].years === row.years}
          <li class:again>
            <button
              type="button"
              class="row"
              class:current={row.current}
              disabled={isShown(row)}
              aria-current={isShown(row) ? 'true' : undefined}
              aria-label={t.horizonsOpen(
                [t.horizonsIn(row.years), row.condition, row.name, row.range, row.perShekelText]
                  .filter(Boolean)
                  .join(', '),
              )}
              onclick={() => open(row)}
            >
              <span class="when" class:hidden={again}>
                {t.horizonsIn(row.years)}
                {#if row.current}<span class="yours">{t.horizonsYours}</span>{/if}
              </span>
              <span class="what">
                {#if row.condition}<span class="condition">{row.condition}</span>{/if}
                <span class="name">{row.name}</span>
                {#if !sharedDeposits}<span class="deposits">{row.deposits}</span>{/if}
              </span>
              <span class="outcome">
                <span class="range">{row.range}</span>
                <span class="note">{row.rangeNote}</span>
                <span class="note">{row.perShekelText}</span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
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

  .rows {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    border-block-start: 1px solid var(--border);
  }

  /* The second answer of the same date reads as part of the first. */
  li.again {
    border-block-start: 1px dashed var(--border);
  }

  /* A whole row is a button, drawn as a row. */
  .row {
    display: grid;
    grid-template-columns: 7rem minmax(0, 1fr) minmax(0, 1fr);
    gap: 4px 16px;
    align-items: start;
    inline-size: 100%;
    block-size: auto;
    padding: 10px 8px;
    border: none;
    border-radius: 0;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: start;
  }

  .row:disabled {
    opacity: 1;
    cursor: default;
  }

  .row:hover:not(:disabled) {
    background: var(--accent-tint);
  }

  .row.current {
    /* The start edge, right in a right-to-left page. */
    box-shadow: inset -3px 0 0 var(--accent);
  }

  .when {
    display: grid;
    gap: 2px;
    color: var(--weak);
  }

  .when.hidden {
    visibility: hidden;
  }

  .yours {
    color: var(--accent);
    font-size: 0.75rem;
  }

  .what,
  .outcome {
    display: grid;
    gap: 2px;
  }

  .condition {
    color: var(--weak);
    font-size: 0.85rem;
  }

  .name,
  .range {
    font-weight: 600;
  }

  .deposits,
  .note {
    color: var(--weak);
    font-size: 0.85rem;
  }

  @container (width < 34rem) {
    .row {
      grid-template-columns: minmax(0, 1fr);
    }

    .when.hidden {
      display: none;
    }
  }
</style>
