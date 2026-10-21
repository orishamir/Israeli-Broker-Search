<script lang="ts">
  // The monthly expenses, once the cushion's "not yet" is chosen: a fact only
  // the visitor knows, asked only where it decides something (principle 8).
  // In the cushion's option on the lines, and under its step on the path.
  import type { AppState } from './app.svelte'
  import { reveal } from './motion'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  /** Read and typed; an empty field is not typed yet. */
  const expenses = {
    get: () => (app.answers.cushion?.kind === 'hasNot' ? app.answers.cushion.monthlyExpenses : null),
    set: (value: number | null) => {
      app.answers.cushion = { kind: 'hasNot', monthlyExpenses: value }
    },
  }
</script>

<div class="expenses" transition:reveal>
  <label for="expenses">{t.yourMonthlyExpenses}</label>
  <div class="control">
    <NumberField id="expenses" bind:value={expenses.get, expenses.set} prefix="₪" />
  </div>
</div>

<style>
  .expenses {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px 12px;
  }
  .expenses label {
    flex: 1;
    min-width: 8rem;
    font-size: 0.95rem;
  }
  .expenses .control {
    flex: none;
    width: var(--input-field);
  }
</style>
