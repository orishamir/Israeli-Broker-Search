<script lang="ts">
  // One line of the answer (REDESIGN.md §2.2): its name (קודם, סוג המקום,
  // הדרך, איפה), then what the site states with how sure it is, or the
  // question only the visitor can answer with its options, or, while it
  // depends on a decision above, that it waits for it. The cushion's "not
  // yet", once chosen, opens the monthly-expenses field: a fact only the
  // visitor knows, asked only once it decides something (principle 8).
  import type { AppState } from './app.svelte'
  import type { LineData, OptData, Pick } from './core/core'
  import Decision from './Decision.svelte'
  import ExpensesField from './ExpensesField.svelte'
  import { holdChosen } from './scroll'
  import { t } from './text'

  let { app, line }: { app: AppState; line: LineData } = $props()

  const state = $derived(line.state)
  const id = $derived(`line-${line.kind}`)
  const pill = $derived(
    state.type === 'stated'
      ? t.linePill[state.pill]
      : state.type === 'waiting'
        ? t.pillWaiting
        : state.decision.chosen
          ? t.pillChosen
          : t.pillYourCall,
  )
  const pillKind = $derived(
    state.type === 'stated'
      ? 'stated'
      : state.type === 'waiting'
        ? 'waiting'
        : state.decision.chosen
          ? 'chosen'
          : 'open',
  )

  /** Chooses, keeping the chosen option in view while the answer changes. */
  function choose(pick: Pick) {
    const key = JSON.stringify(pick)
    const kind = line.kind
    const decisionId = state.type === 'decision' ? state.decision.id : undefined
    holdChosen(
      () =>
        [...document.querySelectorAll<HTMLElement>(`section.answer [data-line="${kind}"] .option`)].find(
          (option) => option.dataset.pick === key,
        ) ??
        // The last choice made the plan: its step stands where the option was.
        document.querySelector(`section.answer [data-step="${decisionId}"]`),
      () => app.answerFresh,
      // A decision this opens out of sight gets the pill.
      (opened) => app.announceDecision(opened),
    )
    app.choose(pick)
  }

  /** "Not yet", chosen: the option that holds the field. */
  const notYetChosen = (option: OptData): boolean =>
    'Cushion' in option.pick && option.pick.Cushion === 'HasNot' && app.answers.cushion?.kind === 'hasNot'
</script>

{#snippet expensesField(option: OptData)}
  {#if notYetChosen(option)}<ExpensesField {app} />{/if}
{/snippet}

<div class="line" data-line={line.kind} class:asks={state.type === 'decision'}>
  <div class="head">
    <span class="label">{line.label}</span>
    {#if state.type === 'stated'}
      <p class="said">{state.text}</p>
    {:else if state.type === 'waiting'}
      <p class="said waiting">{t.lineWaiting}</p>
    {:else}
      <h3 class="question" {id}>{state.decision.question}</h3>
    {/if}
    <span class="pill {pillKind}">{pill}</span>
  </div>
  {#if state.type === 'decision'}
    <Decision
      decision={state.decision}
      labelledby={id}
      onchoose={choose}
      extra={state.decision.id === 'Cushion' ? expensesField : undefined}
    />
  {/if}
</div>

<style>
  .line {
    display: grid;
    gap: 12px;
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
  }
  /* The name, what's said or asked, and the pill on one row; on a narrow
     card the name goes above. */
  .head {
    display: grid;
    grid-template-columns: 6.5rem minmax(0, 1fr) auto;
    align-items: baseline;
    gap: 12px;
  }
  @container (max-width: 34rem) {
    .head {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .label {
      grid-column: 1 / -1;
    }
  }
  .label {
    color: var(--weak);
    font-size: 0.92rem;
    font-weight: 700;
  }
  .said,
  .question {
    margin: 0;
    font-size: 1rem;
    line-height: 1.55;
  }
  .question {
    font-weight: 700;
  }
  .waiting {
    color: var(--weak);
  }
  .pill {
    padding: 2px 10px;
    border: 1px solid transparent;
    border-radius: 999px;
    font-size: 0.82rem;
    font-weight: 700;
    white-space: nowrap;
  }
  /* What the site says, one neutral look whatever its word; the visitor's
     call stands out in the warning's tint; what's chosen is inverted. The
     accent is the sure answer's alone. */
  .pill.stated {
    background: var(--raised-strong);
  }
  .pill.waiting {
    border-color: var(--border);
    color: var(--weak);
  }
  .pill.open {
    border-color: var(--warning-edge);
    background: var(--warning-tint);
    color: var(--warning);
  }
  .pill.chosen {
    background: var(--text);
    color: var(--background);
  }
</style>
