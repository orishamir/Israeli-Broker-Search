<script lang="ts">
  // A term's card in a `Sheet` (screen s5): what it is, why it's good, its
  // downside, cost, tax, where else it's met, an example; the cards to set it
  // beside open in the same sheet, with a way back. Every sentence is the
  // core's; the labels are the page's.
  import type { AppState } from './app.svelte'
  import Sheet from './Sheet.svelte'
  import Sources from './Sources.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const card = $derived(app.cardTerm ? app.cardFor(app.cardTerm) : undefined)
  // Kept while the sheet fades out, so it doesn't empty first.
  let shown = $state.raw<NonNullable<typeof card>>()
  $effect(() => {
    if (card) shown = card
  })
  const sections = $derived(
    shown
      ? [
          [t.cardWhat, shown.what],
          [t.cardGoodFor, shown.goodFor],
          [t.cardDownside, shown.downside],
          [t.cardCost, shown.cost],
          [t.cardTax, shown.tax],
          [t.cardWhereMet, shown.whereMet],
        ]
      : [],
  )
  let body = $state<HTMLElement>()
  // Another card starts at the top, not where the last was scrolled to.
  $effect(() => {
    void app.cardTerm
    const scroller = body?.closest('.groups')
    if (scroller) scroller.scrollTop = 0
  })
  const toGlossary = () => {
    app.closeCard()
    app.glossaryOpen = true
  }
</script>

<Sheet
  bind:open={() => app.cardTerm !== undefined, (open) => !open && app.closeCard()}
  title={shown?.name ?? ''}
  done={t.close}
>
  {#if shown}
    <div class="card-body" bind:this={body}>
      {#if app.cardStack.length > 1}
        <button class="back" onclick={() => app.cardBack()}>{t.cardBack}</button>
      {/if}
      <p class="names">
        <bdi lang="en">{shown.english}</bdi>
        <span class="calls"><span class="label">{t.cardInTheBank}</span> {shown.asTheBankCallsIt}</span>
        <span class="calls"><span class="label">{t.cardInAds}</span> {shown.asAdsCallIt}</span>
      </p>
      {#each sections as [label, text] (label)}
        <h3>{label}</h3>
        <p>{text}</p>
      {/each}
      <p class="example">{shown.example}</p>
      <Sources sources={shown.sources} />
      <div class="chips">
        {#each shown.compareWith as term (term)}
          <button class="chip" onclick={() => app.openCard(term)}
            >{t.cardVersus(app.cardFor(term).name)}</button
          >
        {/each}
        <button class="chip" onclick={toGlossary}>{t.allTerms}</button>
      </div>
    </div>
  {/if}
</Sheet>

<style>
  .card-body {
    display: grid;
    gap: 4px;
  }
  .back {
    justify-self: start;
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
  }
  .back:hover {
    background: none;
    text-decoration: underline;
  }
  .names {
    display: grid;
    gap: 2px;
    margin: 0 0 4px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .label {
    color: var(--weak);
  }
  h3 {
    margin: 12px 0 0;
    color: var(--weak);
    font-size: 0.85rem;
    font-weight: 400;
  }
  p {
    margin: 0;
  }
  .example {
    margin-top: 14px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }
  .chip {
    padding: 6px 14px;
    border-radius: 999px;
    color: var(--accent);
    font-size: 0.9rem;
  }
</style>
