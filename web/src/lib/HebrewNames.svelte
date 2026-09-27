<script lang="ts">
  /** What a term is called in Hebrew: the usual name first, then others. */
  let { names }: { names: string[] } = $props()

  const [main, ...others] = $derived(names)
</script>

{#if main}
  <dl>
    <dt>In Hebrew</dt>
    <dd><bdi lang="he">{main}</bdi></dd>
    {#if others.length > 0}
      <dt>Also called</dt>
      <!-- One per line: side by side, Hebrew names read in the wrong order. -->
      <dd class="others">
        {#each others as other (other)}<bdi lang="he">{other}</bdi>{/each}
      </dd>
    {/if}
  </dl>
{/if}

<style>
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 2px 10px;
    margin: 0;
  }
  dt {
    color: var(--weak);
  }
  dd {
    margin: 0;
  }
  .others {
    display: grid;
    justify-items: start;
  }
</style>
