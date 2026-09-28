<script lang="ts">
  import type { Source } from './core/core'

  /** Links to the pages a number rests on, small and last: "Sources: IBI's
   * currency FAQ ↗ · tradingil.co.il on conversion costs ↗". */
  let { sources, label = 'Sources' }: { sources: Source[]; label?: string } = $props()
</script>

{#if sources.length > 0}
  <span class="sources">
    {label}:
    {#each sources as source, index (source.url)}
      {index > 0 ? ' · ' : ''}<a href={source.url} target="_blank" rel="noreferrer">{source.name} ↗</a>
    {/each}
  </span>
{/if}

<style>
  .sources {
    display: block;
    margin-top: 4px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  a {
    color: inherit;
    text-decoration-color: var(--strong-border);
  }
  a:hover {
    color: var(--text);
  }
</style>
