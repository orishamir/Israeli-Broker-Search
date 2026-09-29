<script lang="ts">
  import { tick } from 'svelte'
  import { flip } from 'svelte/animate'
  import type { AppState, Result } from './app.svelte'
  import type { OutcomeData } from './core/core'
  import { percent, shekels } from './format'
  import { duration, reducedMotion } from './motion'
  import Tip from './Tip.svelte'

  let { app, results }: { app: AppState; results: Result[] } = $props()

  interface Column {
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
    const money = app.inTodaysMoney
      ? ' In today’s money: divided by how much prices will have risen by then.'
      : ''
    const columns: Column[] = [
      {
        title: 'Yearly cost',
        explanation:
          'What the fees come to as a yearly charge on your holdings, the way a fund states its management fee: paying this share of your holdings every year, and nothing else, would leave you the same. Compare it with a fund’s fee, or with the same plan at another deposit. 100% when nothing is left.',
        text: (outcome) => percent(outcome.yearlyCostPercent),
      },
      {
        title: 'Lost to fees',
        explanation: `How much less you end up with${selling ? ', after selling,' : ''} than with no fees and buying every month: the fees, plus the growth they and money waiting for a purchase would have earned. ₪10 a month in fees over 20 years is ₪2,400 paid, but about ₪7,000 lost at 10% a year.${money}`,
        text: (outcome) => shekels(outcome.lostToFees),
      },
    ]
    if (selling) {
      columns.push({
        title: 'Value if sold',
        explanation: `What you'd get in shekels by selling everything at the end, after the sell fee and converting back. Before tax. Abroad, that's one more trade fee and one more conversion.${money}`,
        text: (outcome) => shekels(outcome.afterSelling),
      })
    }
    columns.push(
      {
        title: 'Fees paid',
        explanation: `Every fee charged: purchases, conversions, custody, handling${selling ? ', and selling at the end' : ''}. Over 20 years of buying every month, that is 240 purchases and, abroad, 240 conversions.${money}`,
        text: (outcome) => shekels(outcome.fees.total),
      },
      {
        title: 'Value held',
        explanation: `What the investment is worth at the end, without selling.${money}`,
        text: (outcome) => shekels(outcome.held),
      },
    )
    return columns
  })
</script>

<table>
  <thead>
    <tr>
      <th class="rank"><span class="visually-hidden">Rank</span></th>
      <th class="plan">Plan</th>
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
              {#if column.title === 'Fees paid'}
                <!-- Not also a click on the row, which would unpin it. -->
                <button
                  class="link"
                  aria-label="{plan.label} fees: {column.text(outcome)}, see what they went to"
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
            >Not offered for {app.purchase}<Tip about="Not offered">{notOffered}</Tip></td
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
    text-align: left;
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
  .rank {
    width: 1px;
    padding-right: 0;
    color: var(--weak);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    text-align: right;
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
      left: 0;
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
