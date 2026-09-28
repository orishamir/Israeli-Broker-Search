<script lang="ts">
  import type { AppState, YourPlan } from './app.svelte'
  import ConversionInputs from './ConversionInputs.svelte'
  import * as core from './core/core'
  import CustodyInputs from './CustodyInputs.svelte'
  import type { Change, Original } from './editor'
  import FeeName from './FeeName.svelte'
  import TradeInputs from './TradeInputs.svelte'
  import Was from './Was.svelte'

  /** The editor's simple view: the fees for what the user buys, each with its
   * price, unit and minimum. */
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
  const fees = $derived(core.simpleFees(yours.plan, app.security, app.exchange, original?.data))
  const { security, exchange } = $derived(app)
</script>

<div class="fees-for">
  <p class="for">For {app.purchase}:</p>
  <dl>
    <dt><FeeName kind={tradeKind} /></dt>
    <dd>
      {#if fees.trade}
        {@const trade = fees.trade}
        <TradeInputs
          fields={trade.fields}
          currency={trade.currency}
          name={tradeKind.name}
          onchange={(fields) => change('trade', (plan) => core.setTrade(plan, security, exchange, fields))}
        />
        {#if trade.was && original}
          {@const was = trade.was}
          <Was
            original={original.name}
            text={was.text}
            reset={() => change('trade', (plan) => core.setTrade(plan, security, exchange, was.fields))}
          />
        {/if}
      {:else}
        <p class="missing">
          not offered
          <button
            type="button"
            onclick={() =>
              change('trade', (plan) =>
                core.setTrade(plan, security, exchange, {
                  kind: 'Percent',
                  amount: 0,
                  min: undefined,
                  max: undefined,
                }),
              )}>+ Add a fee</button
          >
        </p>
      {/if}
      {#if errors.trade}<p class="error">{errors.trade}</p>{/if}
    </dd>

    <dt><FeeName kind={custodyKind} /></dt>
    <dd>
      <CustodyInputs
        fields={fees.custody.fields}
        currency={fees.custody.currency}
        onchange={(fields) => change('custody', (plan) => core.setCustody(plan, exchange, fields))}
      />
      {#if fees.custody.was && original}
        {@const was = fees.custody.was}
        <Was
          original={original.name}
          text={was.text}
          reset={() => change('custody', (plan) => core.setCustody(plan, exchange, was.fields))}
        />
      {/if}
      {#if errors.custody}<p class="error">{errors.custody}</p>{/if}
    </dd>

    {#if fees.conversion}
      {@const conversion = fees.conversion}
      <dt><FeeName kind={conversionKind} /></dt>
      <dd>
        <ConversionInputs
          fields={conversion.fields}
          markup={fees.markup!.fields}
          currency={conversion.currency}
          onchange={(fields) => change('conversion', (plan) => core.setConversion(plan, fields))}
          onmarkupchange={(fields) => change('markup', (plan) => core.setMarkup(plan, fields))}
        />
        {#if conversion.was && original}
          {@const was = conversion.was}
          <Was
            original={original.name}
            text={was.text}
            reset={() => change('conversion', (plan) => core.setConversion(plan, was.fields))}
          />
        {/if}
        {#if fees.markup?.was && original}
          {@const was = fees.markup.was}
          <Was
            original={original.name}
            text="markup {was.text}"
            reset={() => change('markup', (plan) => core.setMarkup(plan, was.fields))}
          />
        {/if}
        {#if errors.conversion}<p class="error">{errors.conversion}</p>{/if}
        {#if errors.markup}<p class="error">{errors.markup}</p>{/if}
      </dd>
    {/if}
  </dl>
</div>

<style>
  /* The same table as a plan's details (FeesForInputs), with fields where
     the prices are, and a line between fees. */
  .fees-for {
    container-type: inline-size;
  }
  .for {
    margin: 0 0 8px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    margin: 0;
  }
  dt {
    /* Level with the first field's text. Spaced by padding rather than a
       column gap, so the lines between fees run unbroken. */
    padding: 7px 14px 0 0;
    color: var(--weak);
  }
  dd {
    margin: 0;
  }
  dt:not(:first-of-type),
  dt:not(:first-of-type) + dd {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  dt:not(:first-of-type) {
    padding-top: 17px;
  }
  /* Narrow: each fee's fields under its name, as in the details. */
  @container (width < 380px) {
    dl {
      grid-template-columns: minmax(0, 1fr);
    }
    dt {
      padding: 0;
    }
    dt:not(:first-of-type) {
      padding-top: 10px;
    }
    dt:not(:first-of-type) + dd {
      margin-top: 6px;
      padding-top: 0;
      border-top: none;
    }
  }
  .missing {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 7px 0 0;
    color: var(--weak);
    font-style: italic;
  }
  .missing button {
    font-style: normal;
  }
  .error {
    margin: 4px 0 0;
    color: var(--error);
    font-size: 0.85rem;
  }
</style>
