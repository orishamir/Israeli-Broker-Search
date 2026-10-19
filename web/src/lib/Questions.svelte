<script lang="ts">
  // Screen s1: the situations strip (chips, each with a tip on hover), the
  // money fields, the horizon (a number and a unit menu; "don't know yet";
  // for retirement, age fields), the follow-up cards the answers still need
  // (they come and go with `transition:reveal`), and the way out to the
  // workbench. Everything it changes goes through `app`.
  import type { Attachment } from 'svelte/attachments'
  import type { AppState } from './app.svelte'
  import { defaultUnit, monthsOfTyped, shown, type Unit } from './horizon'
  import { reveal } from './motion'
  import NumberField from './NumberField.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'
  import { tip } from './tip'

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

  const popovers = $state<HTMLElement[]>([])
  /** The cards to show: the guide's follow-ups, the expenses field with the
   * cushion's (or on its own, if the cushion's question isn't asked). */
  const cards = $derived(app.followCards.filter((each) => each.followUp !== 'MonthlyExpenses'))
  const expensesAsked = $derived(
    app.followUps.includes('MonthlyExpenses') ||
      (app.answers.cushion?.kind === 'hasNot' && cards.some((each) => each.followUp === 'Cushion')),
  )

  /** The tip's placement is for hovering; a tap picks the chip. */
  const hoverOnly = (popover: HTMLElement): Attachment<HTMLElement> => tip(popover, { onClick: false })

  function setCushion(has: boolean) {
    app.answers.cushion = has ? { kind: 'has' } : { kind: 'hasNot', monthlyExpenses: null }
  }
  const expenses = {
    get: () => (app.answers.cushion?.kind === 'hasNot' ? app.answers.cushion.monthlyExpenses : null),
    set: (value: number | null) => {
      app.answers.cushion = { kind: 'hasNot', monthlyExpenses: value }
    },
  }
</script>

<div class="stack">
  <section class="card questions">
    <div class="titled heading">
      <h2>{t.whatForTitle}</h2>
      <Tip about={t.whatForTitle}>{t.whatForTip}</Tip>
    </div>
    <p class="hint">{t.whatForHint}</p>
    <div class="chips" role="group" aria-label={t.whatForTitle}>
      {#each chips as situation, index (situation.id)}
        {@const popover = popovers[index]}
        <button
          class="chip"
          class:chosen={app.situationId === situation.id}
          aria-pressed={app.situationId === situation.id}
          onclick={() => app.setSituation(situation.id)}
          {@attach popover && hoverOnly(popover)}
        >
          {situation.name}
        </button>
      {/each}
    </div>
    {#each chips as situation, index (situation.id)}
      <div class="popover" popover="manual" role="tooltip" bind:this={popovers[index]}>
        <p><strong>{situation.name}</strong></p>
        <p>{situation.explanation}</p>
        <p>{situation.lesson}</p>
      </div>
    {/each}
    <button class="link" onclick={() => app.knowsWhatToBuy()}>{t.iKnowWhatToBuy}</button>
  </section>

  <section class="card money">
    <div class="top">
      <button
        class="chip"
        class:chosen={horizon === 'Unknown'}
        aria-pressed={horizon === 'Unknown'}
        onclick={() => app.setHorizonUnknown()}
      >
        {t.dontKnowYet}
      </button>
    </div>
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
  </section>

  {#each cards as card (card.followUp)}
    <section class="card follow" transition:reveal>
      {#if card.followUp === 'Flexibility'}
        <h3>{card.title}</h3>
        <p class="hint">{card.text}</p>
        <div class="answers">
          <button
            class="chip"
            class:chosen={app.answers.flexibility === 'Fixed'}
            aria-pressed={app.answers.flexibility === 'Fixed'}
            onclick={() => (app.answers.flexibility = 'Fixed')}>{card.answers[0]}</button
          >
          <button
            class="chip"
            class:chosen={app.answers.flexibility === 'CanWait'}
            aria-pressed={app.answers.flexibility === 'CanWait'}
            onclick={() => (app.answers.flexibility = 'CanWait')}>{card.answers[1]}</button
          >
        </div>
      {:else if card.followUp === 'AgeAndRetirement'}
        <h3>{card.title}</h3>
        <p class="hint">{card.text}</p>
        <div class="field-row">
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
        <div class="field-row">
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
      {:else if card.followUp === 'Cushion'}
        <h3>{card.title}</h3>
        <p class="hint">{card.text}</p>
        <div class="answers">
          <button
            class="chip"
            class:chosen={app.answers.cushion?.kind === 'has'}
            aria-pressed={app.answers.cushion?.kind === 'has'}
            onclick={() => setCushion(true)}>{card.answers[0]}</button
          >
          <button
            class="chip"
            class:chosen={app.answers.cushion?.kind === 'hasNot'}
            aria-pressed={app.answers.cushion?.kind === 'hasNot'}
            onclick={() => setCushion(false)}>{card.answers[1]}</button
          >
        </div>
        {#if expensesAsked}
          <div class="field-row" transition:reveal>
            <label for="expenses" class="label">{app.expensesLabel}</label>
            <div class="control">
              <NumberField id="expenses" bind:value={expenses.get, expenses.set} prefix="₪" />
            </div>
          </div>
        {/if}
      {/if}
    </section>
  {/each}

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
  h2,
  h3 {
    margin: 0;
    font-size: 1.05rem;
  }
  h3 {
    font-size: 1rem;
    line-height: 1.5;
  }
  .hint {
    margin: 6px 0 12px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .chips,
  .answers,
  .top {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chips {
    margin-bottom: 12px;
  }
  .top {
    margin-bottom: 12px;
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
  .follow {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    background: color-mix(in srgb, var(--accent) 6%, var(--surface));
  }
  .follow .answers {
    margin-bottom: 4px;
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
