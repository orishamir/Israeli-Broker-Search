<script lang="ts">
  import type { PriceText } from './core/core'
  import Price from './Price.svelte'

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
  <span>{what ? `${original}: ${what}` : `${original}:`} <Price {price} /></span>
  <button type="button" class="icon-button" aria-label="Back to {original}'s fee" onclick={reset}>↺</button>
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
