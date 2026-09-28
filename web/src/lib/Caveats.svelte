<script lang="ts">
  import type { CaveatText } from './core/core'

  /** Caveats about other choices than the user's, or larger amounts: each
   * titled with what it's about, since the same words can be about two
   * different things, and labelled with how sure the number is. */
  let { caveats }: { caveats: CaveatText[] } = $props()
</script>

{#if caveats.length > 0}
  <ul>
    <!-- Keyed by position: texts repeat across coverages. -->
    {#each caveats as caveat, index (index)}
      <li class:warning={caveat.kind === 'MayCostMore'}>
        <span class="covers">{caveat.covers} <span class="kind">· {caveat.label}</span></span>
        {caveat.text}
        {#if caveat.support}<span class="support">Why: {caveat.support}.</span>{/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    display: grid;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    padding: 8px 10px;
    border-left: 3px solid var(--strong-border);
    border-radius: 0 8px 8px 0;
    background: var(--raised);
    color: var(--weak);
    font-size: 0.9rem;
  }
  li.warning {
    border-left-color: var(--warning);
  }
  .covers {
    display: block;
    color: var(--text);
    font-weight: 600;
  }
  .kind {
    color: var(--weak);
    font-weight: normal;
  }
  .warning .kind {
    color: var(--warning);
  }
  .support {
    display: block;
    margin-top: 4px;
    font-size: 0.85rem;
  }
</style>
