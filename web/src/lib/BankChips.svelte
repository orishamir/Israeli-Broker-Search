<script lang="ts">
  // The existing-portfolio situation's question: which bank. Chips for the
  // banks the core models, and "another bank", whose stand-in the answer says.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const asked = $derived(
    typeof app.answers.situation === 'object' && 'ExistingPortfolio' in app.answers.situation,
  )
  const bank = $derived(
    typeof app.answers.situation === 'object' && 'ExistingPortfolio' in app.answers.situation
      ? app.answers.situation.ExistingPortfolio.bank
      : undefined,
  )
</script>

{#if asked}
  <div class="banks" role="group" aria-label={t.whichBank}>
    {#each app.banks as choice (choice.bank)}
      <button
        class="chip"
        class:chosen={bank === choice.bank}
        aria-pressed={bank === choice.bank}
        onclick={() => app.setBank(choice.bank)}
      >
        {choice.name}
      </button>
    {/each}
    <button
      class="chip"
      class:chosen={bank === undefined}
      aria-pressed={bank === undefined}
      onclick={() => app.setBank(undefined)}
    >
      {t.otherBank}
    </button>
  </div>
  {#if bank === undefined}<p class="note">{t.bankStandIn}</p>{/if}
{/if}

<style>
  .banks {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chip {
    min-height: 33px;
    padding: 4px 14px;
    border-radius: 999px;
  }
  .chip.chosen {
    border-color: var(--accent);
    background: var(--accent-tint);
  }
  .note {
    margin: 8px 0 0;
    color: var(--weak);
    font-size: 0.85rem;
  }
</style>
