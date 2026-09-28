<script lang="ts">
  import type { AppState, Plan } from './app.svelte'
  import HebrewNames from './HebrewNames.svelte'
  import Tip from './Tip.svelte'

  /** `tips`: whether each fee's name explains itself. Not in the hover
   * preview, which can't be hovered into. */
  let { app, plan, tips = true }: { app: AppState; plan: Plan; tips?: boolean } = $props()

  const { fees, markup } = $derived(app.feesFor(plan))
</script>

<!-- The fees that apply to what the user buys. -->
<div class="fees-for">
  <p class="for">For {app.purchase}:</p>
  <dl>
    {#each fees as fee (fee.name)}
      <dt>
        {fee.name}{#if tips}<Tip about={fee.name}>
            <HebrewNames names={fee.hebrewNames} />
            <p>{fee.explanation}</p>
          </Tip>{/if}
      </dt>
      <dd class:missing={fee.missing}>{fee.price}</dd>
    {/each}
    <!-- Part of conversion, the last fee: on its own line under it. -->
    {#if markup}
      <dd class="part">
        <span class="part-name"
          >markup{#if tips}<Tip about={markup.name}>
              <HebrewNames names={markup.hebrewNames} />
              <p>{markup.explanation}</p>
            </Tip>{/if}</span
        >
        {markup.price}
      </dd>
    {/if}
  </dl>
</div>

<style>
  .for {
    margin: 0 0 4px;
    color: var(--weak);
    font-size: 0.85rem;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 3px 14px;
    margin: 0;
  }
  dt {
    color: var(--weak);
  }
  dd {
    margin: 0;
  }
  .part {
    grid-column: 2;
  }
  .part-name {
    margin-right: 6px;
    color: var(--weak);
  }
  .missing {
    color: var(--weak);
    font-style: italic;
  }
  .fees-for {
    container-type: inline-size;
  }
  /* Narrow: each price under its fee, so neither is squeezed. */
  @container (width < 380px) {
    dl {
      grid-template-columns: minmax(0, 1fr);
      gap: 0;
    }
    dd + dt {
      margin-top: 6px;
    }
    .part {
      grid-column: 1;
    }
  }
</style>
