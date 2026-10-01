<script lang="ts">
  import type { CaveatGroup } from './core/core'
  import Sources from './Sources.svelte'
  import Tip from './Tip.svelte'
  import { t } from './text'

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
          {#if caveat.support}<span class="support">{t.why} {caveat.support}.</span>{/if}
          <Sources sources={caveat.sources} />
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
    border-inline-start: 3px solid var(--strong-border);
    /* Square on the stripe's side, round on the other, whichever way the
       text runs. */
    border-start-end-radius: 8px;
    border-end-end-radius: 8px;
    background: var(--raised);
    color: var(--text);
    font-size: 0.9rem;
  }
  .warning li {
    border-inline-start-color: var(--warning);
    /* A whole box: fainter than a flag's tint. */
    background: color-mix(in srgb, var(--warning) 7%, transparent);
  }
  .support {
    display: block;
    margin-top: 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
