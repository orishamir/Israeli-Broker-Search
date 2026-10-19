<script lang="ts">
  // The visitor's facts (REDESIGN.md §2.1): the situations strip (chips,
  // each with a "?" that explains it), the money fields, the horizon (a
  // number and a unit menu; "don't know yet"; for retirement, the two ages),
  // the bank (optional), and the way out to the workbench. Nothing else is asked here: what only the visitor
  // can decide is asked in the answer, where it decides (§2.2). Everything
  // it changes goes through `app`.
  import type { AppState } from './app.svelte'
  import type { Bank } from './core/core'
  import { defaultUnit, monthsOfTyped, shown, type Unit } from './horizon'
  import { reveal } from './motion'
  import { oneColumn, showAnswer } from './scroll'
  import NumberField from './NumberField.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  /** The strip: the workbench situation is the link under it, not a chip. */
  const chips = $derived(app.situations.filter((each) => !each.workbench))
  const months = $derived(app.horizonMonths)

  // The unit the user picked; until then, what reads best. A chip starts over.
  let pickedUnit = $state<Unit>()
  let pickedFor = $state<string>()
  const unit = $derived<Unit>(pickedFor === app.situationId && pickedUnit ? pickedUnit : defaultUnit(months))

  const horizon = $derived(app.answers.horizon)
  const retirement = $derived(
    typeof horizon === 'object' && 'Retirement' in horizon ? horizon.Retirement : undefined,
  )

  // The bank: optional, a fact like the others (§2.1). "Another bank" names
  // none to the core, as no answer does: the reference is then the banks'
  // range. It is only remembered here, so the menu says what was picked.
  const OTHER = 'other'
  let otherBank = $state(false)
  const bankValue = $derived(app.answers.bank ?? (otherBank ? OTHER : ''))

  /** A tip stands above or below the whole strip, over none of the chips. */
  let chipsBox = $state<HTMLElement>()

  /** A chip fills the facts; where the answer is under them (a phone), the
   * page goes to it, since that is what the chip changed. */
  function choose(id: string) {
    app.setSituation(id)
    if (oneColumn()) requestAnimationFrame(() => showAnswer())
  }
</script>

<div class="stack">
  <section class="card questions">
    <div class="titled heading">
      <h2>{t.whatForTitle}</h2>
      <Tip about={t.whatForTitle}>{t.whatForTip}</Tip>
      {#if app.exampleShown}<span class="example" transition:reveal>{t.exampleTag}</span>{/if}
    </div>
    <p class="hint">{t.whatForHint}</p>
    <div class="chips" role="group" aria-label={t.whatForTitle} bind:this={chipsBox}>
      {#each chips as situation (situation.id)}
        <!-- The chip only chooses; its "?" alone explains it, so a tap never
             leaves a tip open over the page (the review's X11). -->
        <span class="titled pair">
          <button
            class="chip"
            class:chosen={app.situationId === situation.id}
            aria-pressed={app.situationId === situation.id}
            onclick={() => choose(situation.id)}
          >
            {situation.name}
          </button>
          <Tip about={situation.name} keepClear={chipsBox}>
            <p><strong>{situation.name}</strong></p>
            <p>{situation.explanation}</p>
            <p>{situation.lesson}</p>
          </Tip>
        </span>
      {/each}
    </div>
    <button class="link" onclick={() => app.knowsWhatToBuy()}>{t.iKnowWhatToBuy}</button>
  </section>

  <section class="card money">
    <div class="field-row">
      <label for="now" class="label">{t.iHaveNow}</label>
      <div class="control"><NumberField id="now" bind:value={app.answers.now} prefix="₪" /></div>
    </div>
    <div class="field-row">
      <label for="monthly" class="label">{t.iAddMonthly}</label>
      <div class="control"><NumberField id="monthly" bind:value={app.answers.monthly} prefix="₪" /></div>
    </div>
    <div class="field-row">
      <label for="horizon" class="label">{t.neededIn}</label>
      <div class="control horizon">
        <NumberField
          id="horizon"
          bind:value={
            () => shown(months, unit),
            (value) => {
              if (value === null && horizon === 'Unknown') return
              app.setHorizonMonths(monthsOfTyped(value, unit))
            }
          }
          placeholder={t.horizonUnknownPlaceholder}
          label={t.neededIn}
        />
        <select
          aria-label={t.timeUnit}
          value={unit}
          onchange={(event) => {
            pickedUnit = event.currentTarget.value as Unit
            pickedFor = app.situationId
          }}
        >
          <option value="years">{t.yearsUnit}</option>
          <option value="months">{t.monthsUnit}</option>
        </select>
      </div>
    </div>
    {#if retirement}
      <div class="field-row" transition:reveal>
        <label for="age" class="label">{t.iAmAge}</label>
        <div class="control">
          <NumberField
            id="age"
            bind:value={
              () => retirement?.age ?? null,
              (value) => app.setHorizonRetirement(value ?? 0, retirement?.retire_at ?? 60)
            }
          />
        </div>
      </div>
      <div class="field-row" transition:reveal>
        <label for="retire-at" class="label">{t.retireAtAge}</label>
        <div class="control">
          <NumberField
            id="retire-at"
            bind:value={
              () => retirement?.retire_at ?? null,
              (value) => app.setHorizonRetirement(retirement?.age ?? 0, value ?? 0)
            }
          />
        </div>
      </div>
    {/if}
    <div class="field-row">
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
  </section>

  <p class="updates">{t.answerUpdatesBelow}</p>
</div>

<style>
  .stack {
    display: grid;
    gap: 16px;
  }
  .heading {
    justify-content: flex-start;
  }
  .example {
    padding: 1px 10px;
    border: 1px solid var(--accent);
    border-radius: 999px;
    background: var(--accent-tint);
    font-size: 0.8rem;
  }
  h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .hint {
    margin: 6px 0 12px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }
  .pair {
    align-items: center;
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
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: 0.9rem;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
  /* Each label beside its field, every field one width, so they line up from
     card to card; the label takes the rest and wraps. */
  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 10px;
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
  /* The number and its unit share the width every field has, so the label
     keeps the room to read on a phone. */
  .control.horizon {
    width: var(--input-field);
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
  .horizon :global(.field) {
    flex: 1;
    min-width: 0;
  }
  .horizon select {
    flex: none;
  }
</style>
