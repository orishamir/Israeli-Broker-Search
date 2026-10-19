<script lang="ts">
  // The dialog "כבר יש לי תיק בבנק" opens (REDESIGN 6.8): which bank (its
  // branch plan is then ticked and compared), the guide's words for the
  // existing-account case (the recommendation's steps), and "+ מסלול משלכם"
  // for a deal the list doesn't have.
  import type { AppState } from './app.svelte'
  import BankChips from './BankChips.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

<Sheet bind:open={app.portfolioDialogOpen} title={t.whichBank} done={t.close}>
  <div class="body">
    <BankChips {app} />
    <p class="help">{t.portfolioHelp}</p>
    {#if app.recommendation}
      <ol class="steps">
        {#each app.recommendation.steps as step (step.title)}
          <li>
            <strong>{step.title}</strong>
            <span>{step.text}</span>
          </li>
        {/each}
      </ol>
    {/if}
    <button class="add" onclick={() => app.draftNewPlan()}>{t.newOwnPlan}</button>
  </div>
</Sheet>

<style>
  .body {
    display: grid;
    gap: 14px;
    padding: 4px 2px 12px;
  }
  .help {
    margin: 0;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .steps {
    display: grid;
    gap: 10px;
    margin: 0;
    padding-inline-start: 22px;
  }
  .steps span {
    display: block;
    margin-top: 2px;
    color: var(--weak);
    font-size: 0.92rem;
  }
  .add {
    justify-self: start;
  }
</style>
