<script lang="ts">
  // The one decision component (REDESIGN.md §2.4): why the question is the
  // visitor's, the options side by side (what each leaves and how sure that
  // is, what it asks, when it fits, a button in the visitor's words), and
  // the ₪ difference under them. The question itself is the line's heading,
  // which names the group. Once one is chosen, it gets the strong border and
  // the others dim but stay, and can still be chosen. Every word is the core's.
  import type { Snippet } from 'svelte'
  import type { DecisionData, OptData, Pick } from './core/core'

  let {
    decision,
    labelledby,
    onchoose,
    extra,
  }: {
    decision: DecisionData
    /** The id of the question's heading. */
    labelledby: string
    onchoose: (pick: Pick) => void
    /** What an option holds besides its words (the cushion's expenses field). */
    extra?: Snippet<[OptData]>
  } = $props()

  const keyOf = (pick: Pick): string => JSON.stringify(pick)
  const chosen = $derived(decision.chosen === undefined ? undefined : keyOf(decision.chosen))
</script>

<div class="decision" role="group" aria-labelledby={labelledby}>
  <p class="why">{decision.why}</p>
  <div class="options" class:three={decision.options.length > 2}>
    {#each decision.options as option (keyOf(option.pick))}
      {@const pressed = chosen === keyOf(option.pick)}
      <div
        class="option"
        class:pressed
        class:other={chosen !== undefined && !pressed}
        data-pick={keyOf(option.pick)}
      >
        <h4 class="title">{option.title}</h4>
        {#if option.amount}<p class="amount">{option.amount}</p>{/if}
        <!-- A second amount in its own unit (a pension a month): one line. -->
        {#if option.also}<p class="also">{option.also}</p>{/if}
        {#if option.facts.length > 0}
          <ul class="facts">
            {#each option.facts as fact, index (index)}<li>{fact}</li>{/each}
          </ul>
        {/if}
        {#if option.fitsIf}<p class="fits">{option.fitsIf}</p>{/if}
        {@render extra?.(option)}
        <button class="choose" aria-pressed={pressed} onclick={() => pressed || onchoose(option.pick)}
          >{option.button}</button
        >
      </div>
    {/each}
  </div>
  {#if decision.difference}<p class="difference">{decision.difference}</p>{/if}
</div>

<style>
  .decision {
    container-type: inline-size;
    display: grid;
    gap: 12px;
  }
  .why {
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
    line-height: 1.55;
  }
  /* Side by side while each option has room to read; stacked on a phone. */
  .options {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }
  @container (min-width: 34rem) {
    .options:not(.three) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @container (min-width: 50rem) {
    .options.three {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 16px;
    border: 2px solid var(--border);
    border-radius: var(--radius);
    background: var(--raised);
    transition:
      border-color var(--quick),
      background var(--quick),
      color var(--quick);
  }
  .option.pressed {
    border-color: var(--text);
  }
  /* Dimmed, not hidden: still readable (the weak text keeps 4.5:1 on the
     card), still a choice. */
  .option.other {
    background: var(--surface);
    color: var(--weak);
  }
  .title {
    margin: 0;
    font-size: 1.02rem;
    line-height: 1.4;
  }
  .amount {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  /* Close under the amount it goes with, one step smaller. */
  .also {
    margin: -4px 0 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .facts {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.95rem;
    line-height: 1.5;
  }
  .fits {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    line-height: 1.5;
  }
  .choose {
    margin-top: auto;
    min-height: 44px;
    padding: 6px 14px;
    border-color: var(--strong-border);
    font-weight: 600;
  }
  .choose[aria-pressed='true'] {
    border-color: var(--text);
    background: var(--text);
    color: var(--background);
    cursor: default;
  }
  .choose[aria-pressed='true']:hover {
    background: var(--text);
  }
  .difference {
    margin: 0;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--raised);
    font-size: 0.97rem;
    line-height: 1.6;
  }
</style>
