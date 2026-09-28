<script lang="ts">
  import type { PriceText } from './core/core'
  import Tip from './Tip.svelte'

  /** A price in words, dimmed when it's nothing: "none", "not offered". A
   * "not offered" says how general the gap is in its ?. */
  let { price }: { price: PriceText } = $props()
</script>

<span class:nothing={price.nothing}>{price.text}</span>{#if price.reason}<Tip about="Not offered"
    >{price.reason}</Tip
  >{/if}

<style>
  .nothing {
    color: var(--weak);
    font-style: italic;
  }
</style>
