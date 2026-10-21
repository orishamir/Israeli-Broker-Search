<script lang="ts">
  // One step of "איך הגעתם לזה" (REDESIGN.md §4.5). Three kinds look
  // different:
  // - the visitor's: a filled circle with a tick, and "שינוי", which opens
  //   the decision again here, in the step's place, with the one decision
  //   component;
  // - the site's: a filled circle with a tick, the line's pill (ברור, or
  //   תיקו where it is one), and "למה זה ברור?" where the core explains it;
  // - which index, still the visitor's outside the site: an empty circle and
  //   "בחירה שלכם", as calm as "ברור". Nothing else: what to consider when
  //   choosing an index isn't written yet (§4.11).
  // A decision a change here opened is a step still open: its question and
  // options, until chosen. The step's head never moves while it's used: what
  // opens and closes is under it.
  import type { AppState } from './app.svelte'
  import type { DecisionData, OptData, PathStepData, Pick } from './core/core'
  import Decision from './Decision.svelte'
  import ExpensesField from './ExpensesField.svelte'
  import { reveal } from './motion'
  import { t } from './text'
  import WhyClear from './WhyClear.svelte'

  let {
    app,
    step,
    decision,
    last,
    open,
    ontoggle,
    onchoose,
    onkeep,
  }: {
    app: AppState
    step: PathStepData
    /** The decision the step stands for, if the visitor's. */
    decision: DecisionData | undefined
    last: boolean
    /** Its decision is shown: opened again, or still to answer. */
    open: boolean
    ontoggle: () => void
    onchoose: (pick: Pick) => void
    onkeep: () => void
  } = $props()

  const key = $derived(step.reopens ?? step.line ?? 'Index')
  const settled = $derived(step.kind === 'Visitor' || step.kind === 'Site')
  const tag = $derived(
    step.kind === 'Site' && step.pill
      ? t.linePill[step.pill]
      : step.kind === 'Outside'
        ? t.stepYours
        : step.kind === 'Open'
          ? t.pillYourCall
          : undefined,
  )
  let whyOpen = $state(false)

  /** The cushion's "not yet": its expenses are asked under the step, or in
   * the option while the decision is open again. */
  const notYet = $derived(step.reopens === 'Cushion' && app.answers.cushion?.kind === 'hasNot')
  const notYetOption = (option: OptData): boolean =>
    'Cushion' in option.pick && option.pick.Cushion === 'HasNot' && notYet
</script>

{#snippet expensesField(option: OptData)}
  {#if notYetOption(option)}<ExpensesField {app} />{/if}
{/snippet}

<li class="step kind-{step.kind}" class:last data-step={key}>
  <div class="rail" aria-hidden="true">
    <span class="node" class:filled={settled}>
      {#if settled}
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"
          ><path d="M20 6 9 17l-5-5" /></svg
        >
      {/if}
    </span>
    <span class="joint"></span>
  </div>
  <div class="body">
    <div class="step-head">
      <p class="label">
        <span>{step.label}</span>
        {#if tag}<span class="tag" class:open-tag={step.kind === 'Open'}>{tag}</span>{/if}
      </p>
      {#if step.kind === 'Visitor'}
        <button
          class="change"
          aria-expanded={open}
          aria-controls="step-{key}"
          aria-label={open ? undefined : `${t.stepChange}: ${step.label}`}
          onclick={ontoggle}>{open ? t.stepClose : t.stepChange}</button
        >
      {/if}
    </div>
    <div class="content" id="step-{key}">
      {#if open && decision}
        <h4 class="question" id="question-{key}">{decision.question}</h4>
        <Decision
          {decision}
          labelledby="question-{key}"
          {onchoose}
          {onkeep}
          extra={decision.id === 'Cushion' ? expensesField : undefined}
        />
      {:else}
        <p class="choice">{step.choice}</p>
        <p class="hinge">{step.hinge}</p>
        {#if notYet}<ExpensesField {app} />{/if}
        {#if step.whyClear}
          <button
            class="link why"
            aria-expanded={whyOpen}
            aria-controls="why-{key}"
            onclick={() => (whyOpen = !whyOpen)}
            >{whyOpen ? t.whyClearHide : t.whyClear}<svg
              class="chevron"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg
            ></button
          >
          {#if whyOpen}
            <div transition:reveal><WhyClear why={step.whyClear} id="why-{key}" /></div>
          {/if}
        {/if}
      {/if}
    </div>
  </div>
</li>

<style>
  /* The rail (a circle, and the line down to the next step), then the step. */
  .step {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr);
    column-gap: 14px;
  }
  .rail {
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .node {
    display: grid;
    flex: none;
    place-items: center;
    box-sizing: border-box;
    width: 26px;
    height: 26px;
    border: 2px solid var(--text);
    border-radius: 50%;
    color: var(--background);
  }
  .node.filled {
    background: var(--text);
  }
  /* The sure answer alone wears the accent (§2.3). */
  :global(.state-A) .node {
    border-color: var(--accent);
  }
  :global(.state-A) .node.filled {
    background: var(--accent);
    color: var(--on-accent);
  }
  .kind-Open .node {
    border-color: var(--warning-edge);
  }
  .joint {
    flex: 1;
    width: 2px;
    min-height: 16px;
    background: var(--strong-border);
  }
  .last .joint {
    background: transparent;
  }
  .body {
    min-width: 0;
    padding: 2px 0 22px;
  }
  .last .body {
    padding-bottom: 8px;
  }
  /* The label and its tag, and "שינוי" at the row's end. */
  .step-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 26px;
  }
  .label {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
    margin: 0;
    color: var(--weak);
    font-size: 0.88rem;
    font-weight: 700;
  }
  /* "ברור" and "בחירה שלכם" in one neutral look: neither is a warning. */
  .tag {
    padding: 0 8px;
    border-radius: 999px;
    background: var(--raised-strong);
    color: var(--text);
    font-size: 0.78rem;
  }
  .tag.open-tag {
    border: 1px solid var(--warning-edge);
    background: var(--warning-tint);
    color: var(--warning);
  }
  .change {
    flex: none;
    min-height: 40px;
    padding: 0 14px;
    border-color: var(--strong-border);
    background: transparent;
    font-size: 0.9rem;
  }
  .content {
    display: grid;
    gap: 4px;
  }
  .choice {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 700;
    line-height: 1.4;
  }
  .hinge {
    max-width: 52rem;
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
    line-height: 1.55;
  }
  .question {
    margin: 2px 0 8px;
    font-size: 1.05rem;
    line-height: 1.45;
  }
  .content > :global(.expenses) {
    margin-top: 8px;
    max-width: 30rem;
  }
  .why {
    display: inline-flex;
    align-items: center;
    justify-self: start;
    gap: 6px;
    min-height: 40px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--accent);
    font-size: 0.92rem;
    font-weight: 600;
  }
  .why:hover {
    background: transparent;
    text-decoration: underline;
  }
  [aria-expanded='true'] .chevron {
    rotate: 180deg;
  }
</style>
