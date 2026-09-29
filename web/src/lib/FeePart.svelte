<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { FeeKindChoice } from './core/core'
  import HebrewNames from './HebrewNames.svelte'
  import Tip from './Tip.svelte'

  /** A second price under a fee's fields, as its part: a standing order's
   * price under buying, a second fee under conversion. */
  let { kind, children }: { kind: FeeKindChoice; children: Snippet } = $props()
</script>

<div class="part">
  <p class="part-name">
    {kind.label}<Tip about={kind.name}>
      <HebrewNames names={kind.hebrewNames} />
      <p>{kind.explanation}</p>
    </Tip>
  </p>
  {@render children()}
</div>

<style>
  /* Set in from the fee's own fields, with a line joining it to them. */
  .part {
    margin-top: 10px;
    padding-inline-start: 10px;
    border-inline-start: 2px solid var(--border);
  }
  .part-name {
    margin: 0 0 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
