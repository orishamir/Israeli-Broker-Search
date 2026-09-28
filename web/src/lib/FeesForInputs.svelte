<script lang="ts">
  import type { AppState, Plan } from './app.svelte'
  import CaveatMark from './CaveatMark.svelte'
  import HebrewNames from './HebrewNames.svelte'
  import Price from './Price.svelte'
  import Tip from './Tip.svelte'

  /** `tips`: whether each fee's name explains itself, and each mark opens
   * its caveats. Not in the hover preview, which can't be hovered into. */
  let { app, plan, tips = true }: { app: AppState; plan: Plan; tips?: boolean } = $props()

  const { fees } = $derived(app.feesFor(plan))
</script>

<!-- The fees that apply to what the user buys, each marked with how sure its
     number is where a caveat is about it. -->
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
      <dd>
        <Price price={fee.price} />{#if fee.mark}<CaveatMark mark={fee.mark} {tips} />{/if}
      </dd>
      <!-- Parts of the fee, each on its own line under it: a standing
           order's price, a second conversion fee, the markup. -->
      {#each fee.parts as part (part.name)}
        <dd class="part">
          <span class="part-name"
            >{part.label}{#if tips}<Tip about={part.name}>
                <HebrewNames names={part.hebrewNames} />
                <p>{part.explanation}</p>
              </Tip>{/if}</span
          >
          <Price price={part.price} />{#if part.mark}<CaveatMark mark={part.mark} {tips} />{/if}
        </dd>
      {/each}
    {/each}
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
