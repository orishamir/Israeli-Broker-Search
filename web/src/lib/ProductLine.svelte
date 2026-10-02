<script lang="ts">
  import type { AppState } from './app.svelte'
  import Choices from './Choices.svelte'
  import type { Choice, Product } from './core/core'
  import Price from './Price.svelte'
  import Sources from './Sources.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'
  import WarningSign from './WarningSign.svelte'

  /** The fund that holds the index for the banks' and investment houses'
   * plans, and what it keeps back a year, under the exchange: two buttons
   * where the purchase has a choice (an ETF in Tel Aviv), one line
   * otherwise, nothing for a share or a bond. */
  let { app }: { app: AppState } = $props()

  const choices: Choice<Product>[] = $derived(
    app.products.map(({ product, choiceName, price, explanation }) => ({
      value: product,
      name: `${choiceName}\u00a0· ${price.text}`,
      englishName: undefined,
      explanation,
      hebrewNames: [],
    })),
  )
</script>

{#if app.heldProduct}
  {@const held = app.heldProduct}
  <div class="product">
    <div class="titled">
      <span class="label">{t.theFundYouBuy}</span><Tip about={t.theFundYouBuy}>{t.theFundYouBuyTip}</Tip>
    </div>
    {#if choices.length > 1}
      <Choices label={t.theFundYouBuy} options={choices} bind:value={app.product} />
    {/if}
    <p class="held">
      {held.name}&nbsp;· <Price price={held.price} />{#if held.warning}<WarningSign
          text={held.warning}
        />{/if}<Tip about={held.name}>
        <p>{held.explanation}</p>
        <!-- The groups' own "?" would open a tip inside this one: their
             labels are words here. -->
        {#each held.caveats as group (group.kind)}
          <p class="group" class:warning={group.kind === 'MayCostMore'}>{group.label}</p>
          {#each group.caveats as caveat, index (index)}
            <p>
              {caveat.text}
              {#if caveat.support}{t.why} {caveat.support}.{/if}
              <Sources sources={caveat.sources} />
            </p>
          {/each}
        {/each}
      </Tip>
    </p>
  </div>
{/if}

<style>
  .product {
    margin-top: 12px;
    font-size: 0.85rem;
  }
  .label {
    font-weight: 600;
  }
  .product :global(.choices) {
    margin-top: 8px;
  }
  .held {
    margin: 8px 0 0;
  }
  .group {
    margin: 12px 0 0;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--weak);
  }
  .group.warning {
    color: var(--warning);
  }
</style>
