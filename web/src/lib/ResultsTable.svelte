<script lang="ts">
  import { tick } from 'svelte'
  import { flip } from 'svelte/animate'
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { percent, shekels } from './format'
  import { duration, reducedMotion } from './motion'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app, results }: { app: AppState; results: Result[] } = $props()

  interface Column {
    /** The same in every language, for the fees column's link. */
    key: 'yearly' | 'lost' | 'sold' | 'fees' | 'held'
    title: string
    explanation: string
    /** The cell's text for an outcome. */
    text: (outcome: OutcomeData) => string
  }

  /** The amounts, the ones that decide first: on a phone only the first two
   * fit beside the plan's name, and the rest scroll into view. As the inputs
   * have them: no "if sold" when nothing is sold, and a note when they're in
   * today's money. */
  const columns: Column[] = $derived.by(() => {
    const selling = app.sellAtEnd
    const money = app.inTodaysMoney ? t.inTodaysMoneyNote : ''
    const columns: Column[] = [
      {
        key: 'yearly',
        title: t.yearlyCost,
        explanation: t.yearlyCostTip,
        text: (outcome) => percent(outcome.yearlyCostPercent),
      },
      {
        key: 'lost',
        title: t.lostToFees,
        explanation: t.lostToFeesTip(selling) + money,
        text: (outcome) => shekels(outcome.lostToFees),
      },
    ]
    if (selling) {
      columns.push({
        key: 'sold',
        title: t.valueIfSold,
        explanation: t.valueIfSoldTip + money,
        text: (outcome) => shekels(outcome.afterSelling),
      })
    }
    columns.push(
      {
        key: 'fees',
        title: t.feesPaid,
        explanation: t.feesPaidTip(selling) + money,
        text: (outcome) => shekels(outcome.fees.total),
      },
      {
        key: 'held',
        title: t.valueHeld,
        explanation: t.valueHeldTip + money,
        text: (outcome) => shekels(outcome.held),
      },
    )
    return columns
  })
</script>

<table>
  <thead>
    <tr>
      <th class="rank"><span class="visually-hidden">{t.rank}</span></th>
      <th class="plan">{t.plan}</th>
      {#each columns as { title, explanation } (title)}
        <th class="amount">{title}<Tip about={title}>{explanation}</Tip></th>
      {/each}
    </tr>
  </thead>
  <tbody>
    <!-- Rows glide to their new place when the ranking changes. Plans that
         don't offer the security have no line to highlight. -->
    {#each results as { plan, outcome, warning, mayCostMore, note, notOffered }, index (plan.id)}
      <tr
        animate:flip={{ duration: duration(300) }}
        class:not-offered={!outcome}
        class:highlighted={app.highlighted.has(plan.id)}
        class:pinned={app.pinned.has(plan.id)}
        style:--plan-color={plan.color}
        onmouseenter={() => (app.hovered = outcome ? plan.id : null)}
        onmouseleave={() => (app.hovered = null)}
        onclick={() => outcome && app.togglePin(plan.id)}
      >
        <td class="rank">{outcome ? index + 1 : '–'}</td>
        <td class="plan">
          <div>
            <span class="mark" class:yours={plan.yours} style:--plan-color={plan.color}></span>
            <span class="names">
              <span class:best={index === 0 && outcome}>{plan.info.name}</span>
              <span class="broker">{plan.subtitle}</span>
              {#if warning}<span class="warning">⚠ {warning}</span>{/if}
              <!-- A number that may be too low: the plan stays ranked, flagged. -->
              {#if mayCostMore}<span class="warning">⚠ {mayCostMore}</span>{/if}
              {#if note}<span class="note">{note}</span>{/if}
            </span>
          </div>
        </td>
        {#if outcome}
          {#each columns as column (column.title)}
            <td class="amount">
              {#if column.key === 'fees'}
                <!-- Not also a click on the row, which would unpin it. -->
                <button
                  class="link"
                  aria-label={t.feesLink(plan.label, column.text(outcome))}
                  onclick={(event) => {
                    event.stopPropagation()
                    app.showFees(plan.id)
                    // Where the breakdown is, below the table (far below, on phones).
                    tick().then(() =>
                      document
                        .getElementById('chart')
                        ?.scrollIntoView({ block: 'start', behavior: reducedMotion ? 'auto' : 'smooth' }),
                    )
                  }}>{column.text(outcome)} ›</button
                >
              {:else}
                {column.text(outcome)}
              {/if}
            </td>
          {/each}
        {:else}
          <td class="amount" colspan={columns.length}
            >{t.notOfferedFor(app.purchase)}<Tip about={t.notOffered}>{notOffered}</Tip></td
          >
        {/if}
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
  /* A heading may take two lines, so a column is as wide as its numbers,
     not its heading: seven columns then fit a desktop without scrolling. */
  th.amount {
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
  /* A tap shows at once (the hovered row is highlighted instead). */
  tbody tr:active {
    background: var(--raised-strong);
  }
  /* Tinted in the plan's color, like its line in the chart. */
  tbody tr.highlighted {
    --row-background: color-mix(in srgb, var(--plan-color) 16%, var(--surface));
    background: var(--row-background);
  }
  tbody tr.pinned td:first-child {
    box-shadow: inset 3px 0 var(--plan-color);
  }
  :global(html[dir='rtl']) tbody tr.pinned td:first-child {
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
  .plan div {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .names {
    display: grid;
    line-height: 1.25;
  }
  .broker,
  .warning,
  .note {
    font-size: 0.75rem;
  }
  .broker,
  .note {
    color: var(--weak);
  }
  .warning {
    color: var(--warning);
  }
  /* Long ones wrap, rather than widen the table past its card. */
  .warning,
  .note {
    white-space: normal;
  }
  /* When the table is wider than its card, the amounts scroll sideways
     under the plan. */
  @container (width < 760px) {
    .plan {
      position: sticky;
      inset-inline-start: 0;
      z-index: 1;
      background: var(--row-background, var(--surface));
    }
    tbody tr:hover:not(.highlighted) .plan {
      background: var(--raised);
    }
    .names {
      min-width: 8rem;
      white-space: normal;
    }
  }
  /* A phone: tighter cells, so the plan and its first two amounts fit. */
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
  /* Your own plans are hollow, like their dotted lines in the charts. */
  .mark.yours {
    border: 2px solid var(--plan-color);
    background: none;
  }
  .not-offered .mark {
    opacity: 0.3;
  }
  /* A link to the plan's fee breakdown, colored like one. */
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
</style>
