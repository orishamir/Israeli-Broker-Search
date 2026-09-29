<script lang="ts">
  import type { PriceText } from './core/core'
  import Price from './Price.svelte'
  import { t } from './text'

  /** What the original plan charges for a fee that was changed, and ↺ to go
   * back to it. `what` names it where the price doesn't: "markup". */
  let {
    original,
    price,
    what,
    reset,
  }: { original: string; price: PriceText; what?: string; reset: () => void } = $props()
</script>

<p class="was">
  <span>{t.was(original, what)} <Price {price} /></span>
  <button type="button" class="icon-button" aria-label={t.backTo(original)} onclick={reset}>↺</button>
</p>

<style>
  .was {
    display: flex;
    align-items: center;
    gap: 4px;
    margin: 4px 0 0;
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
