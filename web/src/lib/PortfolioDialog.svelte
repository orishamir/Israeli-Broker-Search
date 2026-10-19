<script lang="ts">
  // STUB (package I fills it): the dialog the "כבר יש לי תיק בבנק" situation
  // opens (`app.portfolioDialogOpen`): where to find the portfolio's fees in
  // the bank's app, with the guide's words for the existing-account case
  // (the recommendation's steps). REDESIGN.md 6.8.
  import type { AppState } from './app.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

<Sheet bind:open={app.portfolioDialogOpen} title={t.whichBank} done={t.close}>
  <p data-stub="PortfolioDialog">
    {#each app.banks as bank (bank.bank)}
      <button onclick={() => app.setBank(bank.bank)}>{bank.name}</button>
    {/each}
  </p>
</Sheet>
