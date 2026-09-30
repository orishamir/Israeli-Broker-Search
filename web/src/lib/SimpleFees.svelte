<script lang="ts">
  import type { AppState, YourPlan } from './app.svelte'
  import ConversionInputs from './ConversionInputs.svelte'
  import * as core from './core/core'
  import CustodyInputs from './CustodyInputs.svelte'
  import { feeKind, type Change, type Original } from './editor'
  import FeeName from './FeeName.svelte'
  import FeePart from './FeePart.svelte'
  import HandlingInputs from './HandlingInputs.svelte'
  import ManagementInputs from './ManagementInputs.svelte'
  import Tip from './Tip.svelte'
  import TradeInputs from './TradeInputs.svelte'
  import Was from './Was.svelte'
  import { t } from './text'

  /** The editor's simple view: the fees for what the user buys, each with its
   * price, unit and minimum. A fund or a policy has its manager's fee alone:
   * it's all such a plan charges, whatever is bought. */
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

  const tradeKind = feeKind('Trade')
  const standingOrderKind = feeKind('StandingOrder')
  const accountKind = feeKind('Account')
  const conversionKind = feeKind('Conversion')
  const secondConversionKind = feeKind('SecondConversion')
  const managementKind = feeKind('Management')
  const fees = $derived(core.simpleFees(yours.plan, app.security, app.exchange, original?.data))
  const { security, exchange } = $derived(app)
</script>

{#if fees.management}
  {@const management = fees.management}
  <div class="fees-for">
    <dl>
      <dt><FeeName kind={managementKind} /></dt>
      <dd>
        <ManagementInputs
          fields={management.fields}
          onchange={(fields) => change('management', (plan) => core.setManagement(plan, fields))}
        />
        {#if management.was && original}
          {@const was = management.was}
          <Was
            original={original.name}
            price={was.price}
            reset={() => change('management', (plan) => core.setManagement(plan, was.fields))}
          />
        {/if}
        {#if errors.management}<p class="error">{errors.management}</p>{/if}
      </dd>
    </dl>
  </div>
{:else}
  <div class="fees-for">
    <p class="for">{t.forPurchase(app.purchase)}</p>
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
              price={was.price}
              reset={() => change('trade', (plan) => core.setTrade(plan, security, exchange, was.fields))}
            />
          {/if}
        {:else}
          <p class="missing">
            {t.notOfferedHere}
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
                )}>{t.addAFee}</button
            >
          </p>
        {/if}
        {#if errors.trade}<p class="error">{errors.trade}</p>{/if}
        {#if fees.trade && fees.sellsFractions !== undefined}
          {@const sells = fees.sellsFractions}
          <p class="fractions">
            <label>
              <input
                type="checkbox"
                checked={sells}
                onchange={(event) =>
                  change('fractions', (plan) =>
                    core.setSellsFractions(plan, exchange, event.currentTarget.checked),
                  )}
              />
              {t.sellsFractions}</label
            ><Tip about={t.fractionsOfAShare}>
              <p>{t.sellsFractionsTip}</p>
            </Tip>
          </p>
          {#if errors.fractions}<p class="error">{errors.fractions}</p>{/if}
        {/if}
        {#if fees.standingOrder}
          {@const standingOrder = fees.standingOrder}
          <FeePart kind={standingOrderKind}>
            <TradeInputs
              fields={standingOrder.fields}
              currency={standingOrder.currency}
              name={standingOrderKind.name}
              onchange={(fields) =>
                change('standingOrder', (plan) => core.setStandingOrder(plan, security, exchange, fields))}
            />
            {#if standingOrder.was && original}
              {@const was = standingOrder.was}
              <Was
                original={original.name}
                price={was.price}
                reset={() =>
                  change('standingOrder', (plan) =>
                    core.setStandingOrder(plan, security, exchange, was.fields),
                  )}
              />
            {/if}
            {#if errors.standingOrder}<p class="error">{errors.standingOrder}</p>{/if}
          </FeePart>
        {/if}
      </dd>

      <!-- One cost of keeping the account, in its two forms. -->
      <dt><FeeName kind={accountKind} /></dt>
      <dd>
        <p class="form">{t.shareOfHoldings}</p>
        <CustodyInputs
          fields={fees.custody.fields}
          currency={fees.custody.currency}
          onchange={(fields) =>
            change('custody', (plan) => core.setCustody(plan, security, exchange, fields))}
        />
        {#if fees.custody.was && original}
          {@const was = fees.custody.was}
          <Was
            original={original.name}
            price={was.price}
            reset={() => change('custody', (plan) => core.setCustody(plan, security, exchange, was.fields))}
          />
        {/if}
        {#if errors.custody}<p class="error">{errors.custody}</p>{/if}
        <p class="form">{t.fixedAmount}</p>
        <HandlingInputs
          fields={fees.handling.fields}
          onchange={(fields) => change('handling', (plan) => core.setHandling(plan, fields))}
        />
        {#if fees.handling.was && original}
          {@const was = fees.handling.was}
          <Was
            original={original.name}
            price={was.price}
            reset={() => change('handling', (plan) => core.setHandling(plan, was.fields))}
          />
        {/if}
        {#if errors.handling}<p class="error">{errors.handling}</p>{/if}
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
              price={was.price}
              reset={() => change('conversion', (plan) => core.setConversion(plan, was.fields))}
            />
          {/if}
          {#if fees.markup?.was && original}
            {@const was = fees.markup.was}
            <Was
              original={original.name}
              what="markup"
              price={was.price}
              reset={() => change('markup', (plan) => core.setMarkup(plan, was.fields))}
            />
          {/if}
          {#if errors.conversion}<p class="error">{errors.conversion}</p>{/if}
          {#if errors.markup}<p class="error">{errors.markup}</p>{/if}
          {#if fees.secondConversion}
            {@const second = fees.secondConversion}
            <FeePart kind={secondConversionKind}>
              <ConversionInputs
                fields={second.fields}
                currency={second.currency}
                name={secondConversionKind.name}
                onchange={(fields) =>
                  change('secondConversion', (plan) => core.setSecondConversion(plan, fields))}
              />
              {#if second.was && original}
                {@const was = second.was}
                <Was
                  original={original.name}
                  price={was.price}
                  reset={() =>
                    change('secondConversion', (plan) => core.setSecondConversion(plan, was.fields))}
                />
              {/if}
              {#if errors.secondConversion}<p class="error">{errors.secondConversion}</p>{/if}
            </FeePart>
          {/if}
          {#if fees.standingOrderConversion}
            {@const byStandingOrder = fees.standingOrderConversion}
            <FeePart kind={standingOrderKind}>
              <ConversionInputs
                fields={byStandingOrder.fields}
                currency={byStandingOrder.currency}
                name={t.conversionByStandingOrder(standingOrderKind.label)}
                onchange={(fields) =>
                  change('standingOrderConversion', (plan) => core.setStandingOrderConversion(plan, fields))}
              />
              {#if byStandingOrder.was && original}
                {@const was = byStandingOrder.was}
                <Was
                  original={original.name}
                  price={was.price}
                  reset={() =>
                    change('standingOrderConversion', (plan) =>
                      core.setStandingOrderConversion(plan, was.fields),
                    )}
                />
              {/if}
              {#if errors.standingOrderConversion}<p class="error">{errors.standingOrderConversion}</p>{/if}
            </FeePart>
          {/if}
        </dd>
      {/if}
    </dl>
  </div>
{/if}

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
  .form {
    margin: 0 0 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  .form + .form,
  :global(.fields) + .form,
  :global(.was) + .form,
  .error + .form {
    margin-top: 8px;
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
  .fractions {
    display: flex;
    align-items: center;
    margin: 8px 0 0;
  }
  .fractions label {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .error {
    margin: 4px 0 0;
    color: var(--error);
    font-size: 0.85rem;
  }
</style>
