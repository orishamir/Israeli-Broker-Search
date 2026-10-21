<script lang="ts">
  // "מתי תצטרכו את הכסף?" (REDESIGN.md §4.2): four answers in a 2×2 group,
  // and under them one box that holds the chosen answer's fields or
  // sentence. The four take turns in one grid cell, so the box is as tall
  // as the tallest of them at any width, and nothing below it moves when
  // the answer changes (principle 21). The fields that aren't shown keep
  // their numbers (principle 22).
  import type { AppState } from './app.svelte'
  import Choices from './Choices.svelte'
  import type { Choice } from './core/core'
  import { WHENS, type When } from './examples'
  import { monthsOfTyped, shown, type Unit } from './horizon'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  /** A choice the page names itself: no tip, no card. */
  const named = <T extends string>(value: T, name: string): Choice<T> => ({
    value,
    name,
    englishName: undefined,
    explanation: '',
    card: undefined,
    hebrewNames: [],
  })
  const names: Record<When, string> = {
    date: t.whenDate,
    retirement: t.whenRetirement,
    anyDay: t.whenAnyDay,
    unknown: t.whenUnknown,
  }
  const whens = WHENS.map((when) => named(when, names[when]))
  const units = [named<Unit>('years', t.yearsUnit), named<Unit>('months', t.monthsUnit)]
</script>

<div class="when">
  <!-- The group is named by its radiogroup's label; this is its words on screen. -->
  <p class="question" aria-hidden="true">{t.whenTitle}</p>
  <div class="answers">
    <Choices label={t.whenTitle} options={whens} bind:value={() => app.when, (when) => app.setWhen(when)} />
  </div>
  <div class="box">
    <div class="alternative row" class:hidden={app.when !== 'date'} inert={app.when !== 'date'}>
      <label for="horizon">{t.whenIn}</label>
      <div class="number">
        <NumberField
          id="horizon"
          bind:value={
            () => shown(app.datedMonths ?? undefined, app.horizonUnit),
            (value) => app.setHorizonMonths(monthsOfTyped(value, app.horizonUnit))
          }
        />
      </div>
      <div class="units">
        <Choices label={t.timeUnit} options={units} bind:value={app.horizonUnit} />
      </div>
    </div>
    <div class="alternative row" class:hidden={app.when !== 'retirement'} inert={app.when !== 'retirement'}>
      <span class="pair">
        <label for="age">{t.whenAgeToday}</label>
        <span class="age">
          <NumberField
            id="age"
            bind:value={() => app.ageToday, (value) => app.setHorizonRetirement(value, app.retireAt)}
          />
        </span>
      </span>
      <span class="pair">
        <label for="retire-at">{t.whenRetireAt}</label>
        <span class="age">
          <NumberField
            id="retire-at"
            bind:value={() => app.retireAt, (value) => app.setHorizonRetirement(app.ageToday, value)}
          />
        </span>
      </span>
    </div>
    <p class="alternative sentence" class:hidden={app.when !== 'anyDay'}>{t.whenAnyDaySentence}</p>
    <p class="alternative sentence" class:hidden={app.when !== 'unknown'}>{t.whenUnknownSentence}</p>
  </div>
</div>

<style>
  .when {
    margin-top: 16px;
  }
  .question {
    margin: 0 0 8px;
  }
  /* Two rows of two, every button as wide as the others. */
  .answers :global(.choices) {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
  }
  .answers :global(.choices label) {
    display: grid;
    place-items: center;
    min-height: 40px;
    box-sizing: border-box;
    text-align: center;
  }
  .box {
    display: grid;
    margin-top: 8px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--raised);
    font-size: 0.9rem;
  }
  /* The four share one cell: the box is as tall as the tallest. */
  .alternative {
    grid-area: 1 / 1;
    align-self: center;
  }
  .hidden {
    visibility: hidden;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 8px;
  }
  .pair {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .number {
    width: 5.5rem;
  }
  .age {
    width: 4rem;
  }
  .sentence {
    margin: 0;
  }
</style>
