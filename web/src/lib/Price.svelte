<script lang="ts">
  import type { PriceText } from './core/core'

  /** A price in words, dimmed when it's nothing: "none", "not offered". */
  let { price }: { price: PriceText } = $props()
</script>

<span class:nothing={price.nothing}>{price.text}</span>

<style>
  .nothing {
    color: var(--weak);
    font-style: italic;
  }
</style>
