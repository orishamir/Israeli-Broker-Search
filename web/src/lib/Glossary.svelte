<script lang="ts">
  // The glossary index in a `Sheet` (screen s5): every card by its Hebrew
  // name, with the English one under it; a term opens its card.
  import type { Term } from './core/core'
  import type { AppState } from './app.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  // The card replaces the index, which "כל המונחים" brings back.
  const open = (term: Term) => {
    app.glossaryOpen = false
    app.openCard(term)
  }
</script>

<Sheet bind:open={app.glossaryOpen} title={t.glossary} done={t.close}>
  <p class="hint">{t.glossaryHint}</p>
  <ul>
    {#each app.glossary as card (card.term)}
      <li>
        <button onclick={() => open(card.term)}
          ><span class="name">{card.name}</span><bdi lang="en" class="english">{card.english}</bdi></button
        >
      </li>
    {/each}
  </ul>
</Sheet>

<style>
  .hint {
    margin: 0 0 8px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li + li {
    border-top: 1px solid var(--border);
  }
  button {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 2px 12px;
    width: 100%;
    padding: 10px 4px;
    border: none;
    background: none;
    text-align: start;
  }
  button:hover {
    background: var(--raised);
  }
  .english {
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
