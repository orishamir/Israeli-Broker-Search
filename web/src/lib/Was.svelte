<script lang="ts">
  /** What the original plan charges for a fee that was changed, and ↺ to go
   * back to it. */
  let { original, text, reset }: { original: string; text: string; reset: () => void } = $props()
</script>

<p class="was">
  {original}: {text}
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
