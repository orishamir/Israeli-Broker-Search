<script lang="ts">
  import type { CaveatGroup } from './core/core'
  import Tip from './Tip.svelte'

  /** The caveats that matter to what the user buys, under a label per kind
   * (how sure the number is), most serious first. A reading says what
   * supports it. */
  let { groups }: { groups: CaveatGroup[] } = $props()
</script>

{#each groups as group (group.kind)}
  <section class="group" class:warning={group.kind === 'MayCostMore'}>
    <!-- The ? beside the heading, not in it: it would join the heading's name. -->
    <div class="label">
      <h4>{group.label}</h4>
      <Tip about={group.label}>{group.explanation}</Tip>
    </div>
    <ul>
      <!-- Keyed by position: the same words can be about two different things. -->
      {#each group.caveats as caveat, index (index)}
        <li>
          {caveat.text}
          {#if caveat.support}<span class="support">Why: {caveat.support}.</span>{/if}
        </li>
      {/each}
    </ul>
  </section>
{/each}

<style>
  .group + .group {
    margin-top: 14px;
  }
  .label {
    display: flex;
    align-items: center;
    margin-bottom: 6px;
  }
  h4 {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--weak);
  }
  .warning h4 {
    color: var(--warning);
  }
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
    color: var(--text);
    font-size: 0.9rem;
  }
  .warning li {
    border-left-color: var(--warning);
    background: rgb(242 193 78 / 0.07);
  }
  .support {
    display: block;
    margin-top: 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
