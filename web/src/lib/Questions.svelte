<script lang="ts">
  // STUB (package I fills it): the situations strip (chips with tips), the
  // money fields, the horizon field (number + unit menu חודשים/שנים; "עוד לא
  // יודעים"; retirement shows age fields), the follow-up cards
  // (`transition:reveal`, from `app.followUps`) and the expert exit link.
  // Screen s1. Everything it changes goes through `app` (see the API at the
  // top of app.svelte.ts); nothing here may call the core.
  import type { AppState } from './app.svelte'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const months = $derived(app.horizonMonths)
</script>

<section class="card questions" data-stub="Questions">
  <h2>{t.whatForTitle}</h2>
  <p class="chips">
    {#each app.situations as situation (situation.id)}
      <button class:chosen={app.situationId === situation.id} onclick={() => app.setSituation(situation.id)}>
        {situation.name}
      </button>
    {/each}
  </p>
  <button class="link" onclick={() => app.knowsWhatToBuy()}>{t.iKnowWhatToBuy}</button>
  <label>
    {t.iHaveNow}
    <NumberField id="now" bind:value={app.answers.now} prefix="₪" />
  </label>
  <label>
    {t.iAddMonthly}
    <NumberField id="monthly" bind:value={app.answers.monthly} prefix="₪" />
  </label>
  <label>
    {t.neededIn}
    <NumberField id="years" value={months === undefined ? null : months / 12} suffix={t.years} />
  </label>
  <button class="link" onclick={() => app.setHorizonUnknown()}>{t.dontKnowYet}</button>
</section>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chosen {
    border-color: var(--accent);
  }
</style>
