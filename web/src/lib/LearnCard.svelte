<script lang="ts">
  // STUB (package L fills it): the learn card in a `Sheet`: the card of
  // `app.cardTerm` (`app.cardFor`), its "לעומת" chips opening the other card
  // in the same sheet (`app.openCard`), a back button (`app.cardBack`), its
  // sources. Screen s5. Closing calls `app.closeCard()`.
  import type { AppState } from './app.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const card = $derived(app.cardTerm ? app.cardFor(app.cardTerm) : undefined)
</script>

<Sheet
  bind:open={() => app.cardTerm !== undefined, (open) => !open && app.closeCard()}
  title={card?.name ?? ''}
  done={t.close}
>
  {#if card}
    <p data-stub="LearnCard">{card.what}</p>
  {/if}
</Sheet>
