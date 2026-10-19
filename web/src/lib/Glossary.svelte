<script lang="ts">
  // STUB (package L fills it): the glossary index in a `Sheet`
  // (`app.glossaryOpen`, `app.glossary`); a term opens its card
  // (`app.openCard`). Screen s5.
  import type { AppState } from './app.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

<Sheet bind:open={app.glossaryOpen} title={t.glossary} done={t.close}>
  <ul data-stub="Glossary">
    {#each app.glossary as card (card.term)}
      <li><button onclick={() => app.openCard(card.term)}>{card.name}</button></li>
    {/each}
  </ul>
</Sheet>
