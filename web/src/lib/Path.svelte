<script lang="ts">
  // "איך הגעתם לזה" (REDESIGN.md §4.5): a step for each line and decision of
  // the answer, in its order, each with its label, the choice in bold and
  // one line on what it hinged on. A step the visitor decided opens its
  // decision again in its place ("שינוי"); choosing closes it back, and the
  // plan above changes while the step stays where it is (principle 21).
  import type { AppState } from './app.svelte'
  import type { DecisionId, PathStepData, Pick } from './core/core'
  import PathStep from './PathStep.svelte'
  import { holdStill } from './scroll'
  import { t } from './text'

  let { app, steps }: { app: AppState; steps: PathStepData[] } = $props()

  /** The step whose decision is open again, if any. */
  let reopened = $state<DecisionId | undefined>()

  const keyOf = (step: PathStepData): string => step.reopens ?? step.line ?? step.label

  const decisionOf = (id: DecisionId | undefined) => {
    if (id === undefined) return undefined
    for (const line of app.answer?.lines ?? [])
      if (line.state.type === 'decision' && line.state.decision.id === id) return line.state.decision
    return undefined
  }

  /** Keeps the step's head where it is while the plan above changes. */
  function hold(id: DecisionId) {
    holdStill(
      () => document.querySelector(`section.answer [data-step="${id}"] .step-head`),
      () => app.answerFresh,
    )
  }

  function toggle(id: DecisionId) {
    reopened = reopened === id ? undefined : id
  }

  function choose(id: DecisionId, pick: Pick) {
    hold(id)
    reopened = undefined
    app.chooseOnPath(pick)
  }
</script>

<div class="path">
  <h3 class="path-heading">{t.pathHeading}</h3>
  <ol>
    {#each steps as step, index (keyOf(step))}
      {@const decision = decisionOf(step.reopens)}
      <PathStep
        {app}
        {step}
        {decision}
        last={index === steps.length - 1}
        open={step.kind === 'Open' || (step.reopens !== undefined && reopened === step.reopens)}
        ontoggle={() => step.reopens && toggle(step.reopens)}
        onchoose={(pick) => step.reopens && choose(step.reopens, pick)}
        onkeep={() => (reopened = undefined)}
      />
    {/each}
  </ol>
</div>

<style>
  .path {
    padding-top: 18px;
    border-top: 1px solid var(--border);
  }
  .path-heading {
    margin: 0 0 12px;
    color: var(--weak);
    font-size: 1rem;
  }
  ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
