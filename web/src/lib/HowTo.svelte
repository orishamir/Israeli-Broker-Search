<script lang="ts">
  // "איך עושים את זה" (REDESIGN.md §2.1 item 4, §2.8): the row the visitor
  // chose above or tapped in the chart: its name, the identity line, what to
  // search for in the app, the steps (or why the rules leave it out, with a
  // way to compare it anyway), and where to buy it. An empty state until a
  // row is chosen.
  import type { AppState } from './app.svelte'
  import { howFor, placeholderParts } from './how'
  import { t } from './text'
  import WhereToBuy from './WhereToBuy.svelte'

  let { app }: { app: AppState } = $props()

  const how = $derived(howFor(app.answer, app.focusedRow))
  const placeholders = $derived(how?.search?.placeholder === true)

  /** "להשוות בכל זאת": the workbench, on what the rules left out. */
  function compareAnyway() {
    if (how && typeof how.row === 'object' && 'Way' in how.row) app.chooseThing('sp500')
    app.expert = true
  }
</script>

{#if app.answer}
  <section class="card how" aria-labelledby="how-heading" aria-live="polite">
    <div class="heads">
      <span class="eyebrow">{t.howHeading}</span>
      <h2 id="how-heading">{how?.name ?? t.howNothingYet}</h2>
    </div>
    {#if !how}
      <p class="empty">{t.howEmpty}</p>
    {:else}
      {#if how.identity.length > 0}
        <ul class="identity">
          {#each how.identity as part, at (at)}
            <li>{part}</li>
          {/each}
        </ul>
      {/if}
      {#if how.search}
        <div class="search">
          <span class="label">{t.searchLabel}</span>
          <span class="what">
            {#each placeholderParts(how.search.text) as part, at (at)}
              {#if part.placeholder}<mark>{part.text}</mark>{:else}{part.text}{/if}
            {/each}
          </span>
        </div>
        {#if placeholders}<p class="placeholder-note">{t.placeholderNote}</p>{/if}
      {/if}
      {#if how.notOffered}
        <p class="not-offered">{how.notOffered}</p>
        <button type="button" class="link" onclick={compareAnyway}>{t.compareAnyway}</button>
      {:else if how.steps.length > 0}
        <ol class="steps">
          {#each how.steps as step, at (at)}
            <li>{step}</li>
          {/each}
        </ol>
      {/if}
      {#if how.whereToBuy && !how.notOffered}
        <WhereToBuy where={how.whereToBuy} />
      {/if}
    {/if}
  </section>
{/if}

<style>
  .how {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .heads {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .eyebrow {
    color: var(--weak);
    font-size: 0.85rem;
    font-weight: 600;
  }
  h2 {
    margin: 0;
    font-size: 1.25rem;
  }
  .empty,
  .not-offered {
    margin: 0;
    line-height: 1.6;
  }
  .empty {
    color: var(--weak);
  }
  .identity {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .identity li {
    padding: 5px 12px;
    border: 1px solid var(--strong-border);
    border-radius: 999px;
    font-size: 0.9rem;
  }
  .search {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
    align-items: baseline;
    padding: 12px 16px;
    border-radius: var(--radius);
    background: var(--raised);
  }
  .search .label {
    font-weight: 700;
  }
  mark {
    padding: 0 4px;
    border: 1px dashed var(--warning-edge);
    border-radius: 4px;
    background: var(--warning-tint);
    color: inherit;
  }
  .placeholder-note {
    margin: -8px 0 0;
    color: var(--weak);
    font-size: 0.82rem;
  }
  .steps {
    display: grid;
    gap: 8px;
    margin: 0;
    padding-inline-start: 22px;
    line-height: 1.55;
  }
  .link {
    align-self: flex-start;
    min-height: 33px;
    padding: 4px 0;
    border: none;
    background: none;
    color: var(--accent);
  }
</style>
