<script lang="ts">
  // "למה זה ברור?", opened in place under its step (REDESIGN.md §4.5): how
  // unlikely a loss was, said as the past; the count of the stretches since
  // 1926; the crises they went through; four amounts, from what's deposited
  // up; and that it is the past, on the S&P 500's data. Every word and
  // number is the core's.
  import type { WhyClearData } from './core/core'

  let { why, id }: { why: WhyClearData; id: string } = $props()
</script>

<div class="why-clear" {id}>
  <p class="lead">{why.lead}</p>
  <p>{why.count}</p>
  <p>{why.crises}</p>
  <dl class="amounts">
    {#each why.amounts as figure, index (index)}
      <div class="figure">
        <dt>{figure.label}</dt>
        <dd>{figure.amount}</dd>
      </div>
    {/each}
  </dl>
  <p class="past">{why.past}</p>
</div>

<style>
  .why-clear {
    display: grid;
    gap: 8px;
    margin-top: 8px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--raised);
    line-height: 1.55;
  }
  p {
    margin: 0;
  }
  .lead {
    font-size: 1.05rem;
    font-weight: 700;
  }
  /* A label and its amount on a row, the amounts lined up at the end; the
     row stops at --rows-max, so an amount stays near its label. */
  .amounts {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 6px 20px;
    max-width: var(--rows-max);
    margin: 6px 0 0;
  }
  .figure {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
  }
  dt {
    color: var(--weak);
  }
  dd {
    margin: 0;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .past {
    color: var(--weak);
    font-size: 0.88rem;
  }
</style>
