<script lang="ts">
  import { tick } from 'svelte'
  import { flip } from 'svelte/animate'
  import type { AppState, Result } from './app.svelte'
  import { shekels } from './format'
  import { duration } from './motion'
  import Tip from './Tip.svelte'

  let { app, results }: { app: AppState; results: Result[] } = $props()

  const columns = [
    { title: 'Value held', explanation: 'What the investment is worth at the end, without selling.' },
    {
      title: 'Value if sold',
      explanation:
        "What you'd get in shekels by selling everything at the end, after the sell fee and converting back. Before tax.",
    },
    {
      title: 'Fees paid',
      explanation: 'Every fee charged: purchases, conversions, custody, and selling at the end.',
    },
    {
      title: 'Lost to fees',
      explanation:
        'How much less you end up with than with no fees at all, after selling. Bigger than the fees paid, because money paid in fees stops growing.',
    },
  ]
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
    {#each results as { plan, outcome, warning }, index (plan.id)}
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
            </span>
          </div>
        </td>
        {#if outcome}
          <td class="amount">{shekels(outcome.held)}</td>
          <td class="amount">{shekels(outcome.afterSelling)}</td>
          <td class="amount">
            <!-- Not also a click on the row, which would unpin it. -->
            <button
              class="link"
              aria-label="{plan.info.name} fees: {shekels(outcome.fees.total)}, see what they went to"
              onclick={(event) => {
                event.stopPropagation()
                app.showFees(plan.id)
                // Where the breakdown is, below the table (far below, on phones).
                tick().then(() => document.getElementById('chart')?.scrollIntoView({ block: 'start' }))
              }}>{shekels(outcome.fees.total)} ›</button
            >
          </td>
          <td class="amount">{shekels(outcome.lostToFees)}</td>
        {:else}
          <td class="amount" colspan="4">Not offered for {app.purchase}</td>
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
    padding: 9px 12px;
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
  .warning {
    font-size: 0.75rem;
  }
  .broker {
    color: var(--weak);
  }
  .warning {
    color: var(--warning);
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
