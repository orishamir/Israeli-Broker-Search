<script lang="ts">
  // Where to buy, from the visitor's bank (REDESIGN.md §2.9). The places the
  // core lists, with places that tie in one box and no rank inside it
  // (principle 13); the visitor's bank as the reference row, the others as
  // ₪ differences from it (principle 15). Until a bank is named, the banks'
  // range is the reference. The bank is asked once, with the visitor's facts
  // (`Questions`, §2.1), not again here (principle 7).
  import type { WhereToBuyData } from './core/core'
  import { rangeText, tieGroups, versusText } from './how'
  import { t } from './text'

  let { where }: { where: WhereToBuyData } = $props()

  const groups = $derived(tieGroups(where.rows))
</script>

<div class="where">
  <h3>{where.title}</h3>
  {#if where.rows.length > 0}
    <div class="rows">
      {#each groups as group, at (at)}
        {@const tied = group.length > 1}
        <div class="group" class:tied>
          {#if tied}<span class="pill">{t.tie}</span>{/if}
          {#each group as row, place (place)}
            <div class="row" class:yours={row.yours}>
              <span class="name">{row.yours ? t.yourBankRow(row.name) : row.name}</span>
              <span class="amount">
                <bdi>{row.amount}</bdi>{#if row.warning}<span class="sign" aria-hidden="true">⚠</span>{/if}
              </span>
              <span class="versus" class:reference={row.yours}><bdi dir="ltr">{versusText(row)}</bdi></span>
              {#if row.warning}<p class="warning">⚠ {row.warning}</p>{/if}
            </div>
          {/each}
          {#if tied && at === 0 && where.note}<p class="note">{where.note}</p>{/if}
        </div>
      {/each}
      {#if where.bankRange}
        <div class="row yours">
          <span class="name">{t.throughABank}</span>
          <span class="amount"
            ><bdi dir="ltr">{rangeText([where.bankRange[0].today, where.bankRange[1].today])}</bdi></span
          >
          <span class="versus reference">{t.referenceRow}</span>
        </div>
      {/if}
    </div>
  {/if}
  {#if where.rows.length === 0 && where.note}<p class="note">{where.note}</p>{/if}
  {#if where.mayCostMore}<p class="note">{where.mayCostMore}</p>{/if}
  {#if where.unlock}<p class="note">{where.unlock}</p>{/if}
</div>

<style>
  .where {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h3 {
    margin: 0;
    font-size: 1.05rem;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .group {
    display: flex;
    flex-direction: column;
  }
  .group.tied {
    padding: 6px 12px 8px;
    border: 1px solid var(--strong-border);
    border-radius: var(--radius);
  }
  .pill {
    align-self: flex-start;
    padding: 1px 10px;
    border-radius: 999px;
    background: var(--raised);
    color: var(--weak);
    font-size: 0.8rem;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(6.5em, auto);
    gap: 4px 14px;
    align-items: baseline;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  /* On a phone the name takes its own line, the amounts the next. */
  @container (width < 30rem) {
    .row {
      grid-template-columns: auto minmax(0, 1fr);
    }
    .name {
      grid-column: 1 / -1;
    }
  }
  .group.tied .row:last-of-type {
    border-bottom: none;
  }
  .row.yours .name {
    font-weight: 700;
  }
  .amount {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .versus {
    font-variant-numeric: tabular-nums;
    text-align: end;
    white-space: nowrap;
  }
  .versus.reference {
    color: var(--weak);
  }
  .sign {
    margin-inline-start: 6px;
    color: var(--warning);
  }
  .warning {
    grid-column: 1 / -1;
    margin: 0;
    color: var(--warning);
    font-size: 0.85rem;
  }
  .note {
    margin: 4px 0 0;
    color: var(--weak);
    font-size: 0.9rem;
    line-height: 1.55;
  }
</style>
