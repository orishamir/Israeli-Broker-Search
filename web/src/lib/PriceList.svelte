<script lang="ts">
  import type { AppState, YourPlan } from './app.svelte'
  import ConversionInputs from './ConversionInputs.svelte'
  import * as core from './core/core'
  import CoveragePicker from './CoveragePicker.svelte'
  import CustodyInputs from './CustodyInputs.svelte'
  import type { Change, Original } from './editor'
  import FeeName from './FeeName.svelte'
  import NumberField from './NumberField.svelte'
  import Tip from './Tip.svelte'
  import TradeInputs from './TradeInputs.svelte'
  import Was from './Was.svelte'

  /** The editor's full view: every row of the price list, for every
   * security and exchange. Where rows overlap, the more specific one counts;
   * the core keeps them sorted that way. */
  let {
    app,
    yours,
    original,
    change,
    errors,
  }: {
    app: AppState
    yours: YourPlan
    original: Original | undefined
    change: Change
    errors: Record<string, string>
  } = $props()

  const [tradeKind, custodyKind, conversionKind] = core.feeKinds()
  const list = $derived(core.priceList(yours.plan, original?.data))
  const id = $props.id()
</script>

<section>
  <div class="section-heading">
    <h4><FeeName kind={tradeKind} /></h4>
    <button
      type="button"
      onclick={() => change('trading', (plan) => core.addTradeRow(plan, app.security, app.exchange))}
      >+ Row</button
    >
  </div>
  {#each list.trading as row, index (index)}
    {@const coverage = { securities: row.securities, exchanges: row.exchanges }}
    <div class="row" class:never-used={row.neverUsed}>
      <div class="row-heading">
        <CoveragePicker
          {app}
          covers={row.covers}
          {coverage}
          onchange={(coverage) =>
            change(`trading-${index}`, (plan) => core.setTradeRow(plan, index, coverage, row.fee.fields))}
        />
        <button
          type="button"
          class="icon-button"
          aria-label="Remove the row for {row.covers}"
          onclick={() => change('trading', (plan) => core.removeTradeRow(plan, index))}>✕</button
        >
      </div>
      {#if row.neverUsed}
        <p class="note warning">⚠ Never used: more specific rows cover all of it.</p>
      {:else if row.except}
        <p class="note">{row.except}</p>
      {/if}
      <TradeInputs
        fields={row.fee.fields}
        currency={row.fee.currency}
        name="{tradeKind.name}, {row.covers}"
        full
        onchange={(fields) =>
          change(`trading-${index}`, (plan) => core.setTradeRow(plan, index, coverage, fields))}
      />
      {#if row.fee.was && original}
        {@const was = row.fee.was}
        <Was
          original={original.name}
          text={was.text}
          reset={() =>
            change(`trading-${index}`, (plan) => core.setTradeRow(plan, index, coverage, was.fields))}
        />
      {/if}
      {#if errors[`trading-${index}`]}<p class="error">{errors[`trading-${index}`]}</p>{/if}
    </div>
  {:else}
    <p class="note">Nothing can be bought: add a row.</p>
  {/each}
  {#if errors.trading}<p class="error">{errors.trading}</p>{/if}
</section>

<section>
  <div class="section-heading">
    <h4><FeeName kind={custodyKind} /></h4>
    <button type="button" onclick={() => change('custody', (plan) => core.addCustodyRow(plan, app.exchange))}
      >+ Row</button
    >
  </div>
  {#each list.custody as row, index (index)}
    {@const coverage = { securities: [], exchanges: row.exchanges }}
    <div class="row" class:never-used={row.neverUsed}>
      <div class="row-heading">
        <CoveragePicker
          {app}
          covers={row.covers}
          coverage={{ exchanges: row.exchanges }}
          onchange={(coverage) =>
            change(`custody-${index}`, (plan) => core.setCustodyRow(plan, index, coverage, row.fee.fields))}
        />
        <button
          type="button"
          class="icon-button"
          aria-label="Remove the custody row for {row.covers}"
          onclick={() => change('custody', (plan) => core.removeCustodyRow(plan, index))}>✕</button
        >
      </div>
      {#if row.neverUsed}
        <p class="note warning">⚠ Never used: more specific rows cover all of it.</p>
      {:else if row.except}
        <p class="note">{row.except}</p>
      {/if}
      <CustodyInputs
        fields={row.fee.fields}
        currency={row.fee.currency}
        full
        onchange={(fields) =>
          change(`custody-${index}`, (plan) => core.setCustodyRow(plan, index, coverage, fields))}
      />
      {#if row.fee.was && original}
        {@const was = row.fee.was}
        <Was
          original={original.name}
          text={was.text}
          reset={() =>
            change(`custody-${index}`, (plan) => core.setCustodyRow(plan, index, coverage, was.fields))}
        />
      {/if}
      {#if errors[`custody-${index}`]}<p class="error">{errors[`custody-${index}`]}</p>{/if}
    </div>
  {:else}
    <p class="note">No custody fee.</p>
  {/each}
  {#if errors.custody}<p class="error">{errors.custody}</p>{/if}
</section>

<section>
  <h4><FeeName kind={conversionKind} /></h4>
  <ConversionInputs
    fields={list.conversion.fields}
    markup={list.markup.fields}
    currency={list.conversion.currency}
    full
    onchange={(fields) => change('conversion', (plan) => core.setConversion(plan, fields))}
    onmarkupchange={(fields) => change('markup', (plan) => core.setMarkup(plan, fields))}
  />
  {#if list.conversion.was && original}
    {@const was = list.conversion.was}
    <Was
      original={original.name}
      text={was.text}
      reset={() => change('conversion', (plan) => core.setConversion(plan, was.fields))}
    />
  {/if}
  {#if list.markup.was && original}
    {@const was = list.markup.was}
    <Was
      original={original.name}
      text="markup {was.text}"
      reset={() => change('markup', (plan) => core.setMarkup(plan, was.fields))}
    />
  {/if}
  {#if errors.conversion}<p class="error">{errors.conversion}</p>{/if}
  {#if errors.markup}<p class="error">{errors.markup}</p>{/if}
</section>

<section>
  <h4>
    <label for="{id}-deposit">Least first deposit</label><Tip about="Least first deposit">
      <p>The least the account can be opened with. The table warns when your one-time deposit is less.</p>
    </Tip>
  </h4>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-deposit"
        prefix="₪"
        placeholder="none"
        bind:value={
          () => list.minFirstDeposit ?? null,
          (amount) => change('deposit', (plan) => core.setMinFirstDeposit(plan, amount))
        }
      />
    </div>
  </div>
  {#if errors.deposit}<p class="error">{errors.deposit}</p>{/if}
</section>

<style>
  section + section {
    margin-top: 16px;
  }
  h4 {
    margin: 0 0 6px;
    font-size: 0.9rem;
    font-weight: 600;
  }
  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 6px;
  }
  .section-heading h4 {
    margin: 0;
  }
  .row {
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--raised);
  }
  .row + .row {
    margin-top: 8px;
  }
  .row :global(.field),
  .row :global(select) {
    background: var(--surface);
  }
  .row-heading {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }
  .note {
    margin: -2px 0 8px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  .warning {
    color: var(--warning);
  }
  .error {
    margin: 4px 0 0;
    color: var(--error);
    font-size: 0.85rem;
  }
</style>
