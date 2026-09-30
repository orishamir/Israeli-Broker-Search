<script lang="ts">
  import type { AppState, Family } from './app.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** Which calculator: two wide choices under the title, each saying what
   * it compares. Everything below belongs to the chosen one. */
  let { app }: { app: AppState } = $props()

  const id = $props.id()
  const families: { value: Family; name: string; kinds: string }[] = [
    { value: 'long', name: t.longTerm, kinds: t.longTermKinds },
    { value: 'short', name: t.shortTerm, kinds: t.shortTermKinds },
  ]
</script>

<div class="families">
  <fieldset>
    <legend class="visually-hidden">{t.calculators}</legend>
    {#each families as { value, name, kinds } (value)}
      <label class:chosen={app.family === value}>
        <!-- Named by the calculator; what it compares describes it. -->
        <input
          class="visually-hidden"
          type="radio"
          name="{id}-family"
          {value}
          bind:group={app.family}
          aria-labelledby="{id}-{value}"
          aria-describedby="{id}-{value}-kinds"
        />
        <span class="name" id="{id}-{value}">{name}</span>
        <span class="kinds" id="{id}-{value}-kinds">{kinds}</span>
      </label>
    {/each}
  </fieldset>
  <Tip about={t.calculators}><p>{t.calculatorsTip}</p></Tip>
</div>

<style>
  .families {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 14px;
  }
  fieldset {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: none;
  }
  label {
    display: grid;
    align-content: start;
    gap: 1px;
    padding: 9px 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
    cursor: pointer;
    transition:
      background var(--quick),
      border-color var(--quick),
      scale var(--quick);
  }
  label:hover {
    border-color: var(--strong-border);
  }
  /* A press sinks a little. */
  label:active {
    scale: 0.99;
  }
  label:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .chosen,
  .chosen:hover {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 13%, var(--surface));
  }
  .name {
    color: var(--weak);
    font-size: 1rem;
    font-weight: 650;
  }
  .chosen .name {
    color: var(--text);
  }
  .kinds {
    color: var(--weak);
    font-size: 0.8rem;
    line-height: 1.35;
  }
  /* A phone: each name on one line. */
  @media (width < 500px) {
    .families {
      gap: 4px;
    }
    fieldset {
      gap: 6px;
    }
    label {
      padding: 8px;
    }
    .name {
      font-size: 0.9rem;
    }
    .kinds {
      font-size: 0.75rem;
    }
  }
</style>
