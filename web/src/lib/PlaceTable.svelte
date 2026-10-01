<script lang="ts">
  import { flip } from 'svelte/animate'
  import type { ShortTermOutcomeData } from './core/core'
  import { compactPercent, percent, shekels } from './format'
  import { duration, easeOut, SETTLE } from './motion'
  import type { PlaceResult, ShortTermState } from './short-term.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** The places compared, best first: what's left after tax, and what makes
   * the difference. Linked to the charts, as the long term's table is. */
  let { short, results }: { short: ShortTermState; results: PlaceResult[] } = $props()

  interface Column {
    title: string
    explanation: string
    text: (outcome: ShortTermOutcomeData) => string
  }

  /** The amounts, what the table is ranked by first: on a phone only the
   * first two fit beside the place's name, and the rest scroll into view. */
  const columns: Column[] = $derived([
    { title: t.leftAfterTax, explanation: t.leftAfterTaxTip, text: (outcome) => shekels(outcome.afterTax) },
    {
      title: t.netYearlyColumn,
      explanation: t.netYearlyTip,
      text: (outcome) => percent(outcome.yearlyAfterTaxPercent),
    },
    {
      title: t.costYearly,
      explanation: t.costYearlyTip,
      text: (outcome) => percent(outcome.yearlyCostPercent),
    },
    {
      title: t.tax,
      explanation: t.shortTaxTip(compactPercent(short.inputs.inflationPercent ?? 0)),
      text: (outcome) => (outcome.tax === 0 ? t.noTax : shekels(outcome.tax)),
    },
  ])

  /** As in the long term's table: the place's column takes a set share and
   * the rest split what's left, so a note under a place makes its row
   * taller rather than moving the numbers. The date is one of the rest. */
  const width = $derived({ place: '30%', other: `${70 / (columns.length + 1)}%` })
</script>

<table>
  <thead>
    <tr>
      <th class="rank"><span class="visually-hidden">{t.rank}</span></th>
      <th class="place" style:width={width.place}>
        {t.place}<Tip about={t.place}>
          <p>{t.placeTip}</p>
          <!-- Each kind as the list of places explains it, for whoever starts
               from the table. -->
          {#each short.kinds as kind (kind.name)}
            <div class="term">
              <strong>{kind.name}</strong>
              <p class="weak">{kind.description}</p>
            </div>
          {/each}
        </Tip>
      </th>
      {#each columns as { title, explanation } (title)}
        <th class="amount" style:width={width.other}>{title}<Tip about={title}>{explanation}</Tip></th>
      {/each}
      <th class="when" style:width={width.other}>{t.whenOut}<Tip about={t.whenOut}>{t.whenOutTip}</Tip></th>
    </tr>
  </thead>
  <tbody>
    {#each results as { place, outcome, rank, notOffered, notOfferedReason } (place.id)}
      <tr
        animate:flip={{ duration: duration(SETTLE), easing: easeOut }}
        class:not-offered={!outcome}
        class:highlighted={short.highlighted.has(place.id)}
        class:pinned={short.pinned.has(place.id)}
        style:--plan-color={place.color}
        onmouseenter={() => (short.hovered = outcome ? place.id : null)}
        onmouseleave={() => (short.hovered = null)}
        onclick={() => outcome && short.togglePin(place.id)}
      >
        <td class="rank">{rank ?? '–'}</td>
        <td class="place">
          <div>
            <span class="mark" class:yours={place.yours} style:--plan-color={place.color}></span>
            <span class="names">
              <span class:best={rank === 1}>{place.info.name}</span>
              <span class="kind">{place.kindName} · {short.paysFor(place)}</span>
              <!-- A number that may be too good: the place stays ranked, flagged.
                   A flag its whole kind shares is said once, under the table. -->
              {#if place.info.mayCostMore}<span class="warning">⚠ {place.info.mayCostMore}</span>{/if}
            </span>
          </div>
        </td>
        {#if outcome}
          {#each columns as column (column.title)}
            <td class="amount number">{column.text(outcome)}</td>
          {/each}
        {:else}
          <td class="amount" colspan={columns.length}
            >{notOffered}<Tip about={notOffered ?? t.notOffered}>{notOfferedReason}</Tip></td
          >
        {/if}
        <td class="when">
          <span class="liquidity" class:any-day={place.info.liquidity === 'AnyDay'}
            >{place.info.liquidityName}</span
          >
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  table {
    border-collapse: collapse;
    width: 100%;
    font-size: 0.9rem;
  }
  th,
  td {
    padding: 9px 10px;
    text-align: start;
    white-space: nowrap;
  }
  th {
    border-bottom: 1px solid var(--border);
    color: var(--weak);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    vertical-align: bottom;
  }
  /* A heading may take two lines, so a column is as wide as its numbers. */
  th.amount,
  th.when {
    max-width: 6.5rem;
    white-space: normal;
  }
  td {
    border-bottom: 1px solid var(--border);
  }
  tbody tr:last-child td {
    border-bottom: none;
  }
  tbody tr {
    position: relative;
    cursor: pointer;
    transition: background var(--quick);
  }
  tbody tr:hover {
    background: var(--raised);
  }
  tbody tr:active {
    background: var(--raised-strong);
  }
  /* Tinted in the place's color, like its line in the chart. */
  tbody tr.highlighted {
    --row-background: color-mix(in srgb, var(--plan-color) 16%, var(--surface));
    background: var(--row-background);
  }
  /* At the row's start, on the right: an inset shadow has no logical form. */
  tbody tr.pinned td:first-child {
    box-shadow: inset -3px 0 var(--plan-color);
  }
  .rank {
    width: 1px;
    padding-inline-end: 0;
    color: var(--weak);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    text-align: end;
  }
  /* A row's mark beside its names. Rows only: the header's tip is in a
     .place cell too. */
  td.place > div {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .names {
    display: grid;
    line-height: 1.25;
  }
  .kind,
  .warning {
    font-size: 0.75rem;
  }
  .kind {
    color: var(--weak);
  }
  /* One kind in the place column's tip, as in the long term's "What you buy". */
  .term {
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
  .term p {
    margin: 4px 0 0;
  }
  .weak {
    color: var(--weak);
  }
  .warning {
    color: var(--warning);
    white-space: normal;
  }
  /* When the table is wider than its card, the amounts scroll sideways
     under the place. */
  @container (width < 760px) {
    .place {
      position: sticky;
      inset-inline-start: 0;
      z-index: 1;
      background: var(--row-background, var(--surface));
    }
    tbody tr:hover:not(.highlighted) .place {
      background: var(--raised);
    }
    .names {
      min-width: 8rem;
      white-space: normal;
    }
  }
  @container (width < 440px) {
    th,
    td {
      padding: 8px 6px;
    }
  }
  tbody tr.not-offered {
    cursor: default;
  }
  .amount {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  /* Left to right, or a cost below zero reads "0.49%-". */
  .number {
    direction: ltr;
  }
  .not-offered {
    color: var(--weak);
  }
  .best {
    font-weight: 700;
  }
  .mark {
    display: inline-block;
    flex: none;
    box-sizing: border-box;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--plan-color);
  }
  /* Your own deposits are hollow, like their dotted lines in the chart. */
  .mark.yours {
    border: 2px solid var(--plan-color);
    background: none;
  }
  .not-offered .mark {
    opacity: 0.3;
  }
  /* When the money can come out: any day in green, locked in amber. */
  .liquidity {
    display: inline-block;
    padding: 1px 8px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--warning) 13%, transparent);
    color: var(--warning);
    font-size: 0.75rem;
  }
  .liquidity.any-day {
    background: color-mix(in srgb, var(--good) 14%, transparent);
    color: var(--good);
  }
</style>
