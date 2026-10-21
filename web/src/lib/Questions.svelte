<script lang="ts">
  // The visitor's facts (REDESIGN.md §4.2): "הכסף שלכם", with the examples
  // behind "דוגמאות" and a status line that says whose numbers the fields
  // hold; the money; "מתי תצטרכו את הכסף?"; the bank (optional); and the
  // way out to the workbench. Nothing else is asked here: what only the
  // visitor can decide is asked in the answer, where it decides (§2.2).
  // Nothing above or beside a field appears, goes or resizes while it's
  // used (principle 21): the status line keeps one height for every state
  // it can show, and the box under "מתי" one for every answer.
  import type { AppState } from './app.svelte'
  import type { Bank } from './core/core'
  import Examples from './Examples.svelte'
  import NumberField from './NumberField.svelte'
  import { t } from './text'
  import WhenField from './WhenField.svelte'

  let { app }: { app: AppState } = $props()

  // The bank: optional, a fact like the others (§2.1). "Another bank" names
  // none to the core, as no answer does: the reference is then the banks'
  // range. It is only remembered here, so the menu says what was picked.
  const OTHER = 'other'
  let otherBank = $state(false)
  const bankValue = $derived(app.answers.bank ?? (otherBank ? OTHER : ''))

  const example = $derived(app.exampleOnScreen)
</script>

<div class="stack">
  <section class="card questions" class:example aria-labelledby="money-title">
    <div class="title">
      <h2 id="money-title">{t.moneyTitle}</h2>
      <Examples {app} />
    </div>
    <!-- Every state the line can show takes turns in one cell, the unseen
         ones hidden, so the line is as tall as the tallest of them. -->
    <div class="status" aria-live="polite">
      {#each app.examples as each (each.id)}
        <span class="state unseen" aria-hidden="true">
          <span class="pill">{t.exampleOf(each.name)}</span>
          <span class="back"><span aria-hidden="true">↺</span> {t.backToYours}</span>
        </span>
      {/each}
      {#if example}
        <span class="state">
          <span class="pill">{t.exampleOf(example.name)}</span>
          {#if app.canGoBack}
            <button type="button" class="back link" onclick={() => app.backToYours()}>
              <span aria-hidden="true">↺</span>
              {t.backToYours}
            </button>
          {:else}
            <span class="hint">{t.makeThemYours}</span>
          {/if}
        </span>
      {:else}
        <span class="state"><span class="hint">{t.yourNumbers}</span></span>
      {/if}
    </div>

    <div class="field-row">
      <label for="now" class="label">{t.iHaveNow}</label>
      <div class="control"><NumberField id="now" bind:value={app.answers.now} prefix="₪" /></div>
    </div>
    <label class="invested">
      <input
        type="checkbox"
        checked={app.invested}
        onchange={(event) => app.setInvested(event.currentTarget.checked)}
      />
      <span>{t.alreadyInvested}</span>
    </label>
    <div class="field-row">
      <label for="monthly" class="label">{t.iAddMonthly}</label>
      <div class="control"><NumberField id="monthly" bind:value={app.answers.monthly} prefix="₪" /></div>
    </div>

    <WhenField {app} />

    <div class="field-row bank">
      <label for="bank" class="label titled">
        <span>{t.myBank}</span><span class="optional">{t.optional}</span>
      </label>
      <div class="control">
        <select
          id="bank"
          value={bankValue}
          onchange={(event) => {
            const value = event.currentTarget.value
            otherBank = value === OTHER
            app.setBank(value === '' || value === OTHER ? undefined : (value as Bank))
          }}
        >
          <option value="">{t.noBankChosen}</option>
          {#each app.banks as choice (choice.bank)}
            <option value={choice.bank}>{choice.name}</option>
          {/each}
          <option value={OTHER}>{t.otherBank}</option>
        </select>
      </div>
    </div>
    <button class="link workbench" onclick={() => app.knowsWhatToBuy()}>{t.iKnowWhatToBuy}</button>
  </section>

  <p class="updates">{t.answerUpdatesBelow}</p>
</div>

<style>
  .stack {
    display: grid;
    gap: 16px;
  }
  /* The examples' list floats over the fields, placed from the card. */
  .questions {
    position: relative;
  }
  .title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .status {
    display: grid;
    margin-top: 4px;
    font-size: 0.85rem;
  }
  .state {
    display: flex;
    flex-wrap: wrap;
    grid-area: 1 / 1;
    align-items: center;
    gap: 4px 10px;
    min-height: 30px;
  }
  .unseen {
    visibility: hidden;
  }
  .pill {
    padding: 1px 10px;
    border: 1px solid color-mix(in srgb, var(--accent) 60%, transparent);
    border-radius: 999px;
    background: var(--accent-tint);
  }
  .hint {
    color: var(--weak);
  }
  .back {
    font-weight: 600;
  }
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: 0.9rem;
  }
  .back.link {
    font-size: inherit;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
  /* While an example is on screen, its fields are tinted. */
  .example :global(.field) {
    border-color: color-mix(in srgb, var(--accent) 50%, transparent);
    background: color-mix(in srgb, var(--accent) 12%, var(--raised));
  }
  .invested {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    margin-top: 4px;
    font-size: 0.88rem;
    cursor: pointer;
  }
  .invested input {
    flex: none;
    width: 18px;
    height: 18px;
    margin: 0;
    accent-color: var(--accent);
  }
  /* Each label beside its field, every field one width, so they line up from
     card to card; the label takes the rest and wraps. */
  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 6px;
  }
  .field-row.bank {
    margin-top: 16px;
  }
  .label {
    flex: 1;
    min-width: 0;
  }
  .label.titled {
    flex-wrap: wrap;
    justify-content: flex-start;
    column-gap: 8px;
  }
  .optional {
    color: var(--weak);
    font-size: 0.85rem;
  }
  .control {
    display: flex;
    flex: none;
    gap: 8px;
    width: var(--input-field);
  }
  .workbench {
    margin-top: 14px;
  }
  .updates {
    margin: 0;
    color: var(--weak);
    font-size: 0.9rem;
    text-align: center;
  }
  /* Beside the answer, there is nothing below to point to. */
  @media (min-width: 800px) {
    .updates {
      display: none;
    }
  }
</style>
