<script lang="ts">
  // The answer card (REDESIGN.md §2.1–§2.4, §4.4, §4.5): how sure the site is
  // (the chip and its note), a one-line title and what's clear, the four
  // lines, and how many decisions are left. Once every decision is made, or
  // there were none, it becomes the plan: the chip, "התוכנית שלכם", the plan
  // in one sentence, its amount, and "איך הגעתם לזה" (`Path`). Only a sure
  // answer (state A) wears the accent. The words are the core's; the
  // chip's, the pills' and the frame's are the page's.
  import { untrack } from 'svelte'
  import type { AppState } from './app.svelte'
  import AnswerLine from './AnswerLine.svelte'
  import { glideHeight, SETTLE } from './motion'
  import Path from './Path.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const answer = $derived(app.answer)
  /** Every decision made, or none to make. */
  const planned = $derived(answer?.plan !== undefined && answer.planAmount !== undefined)
  /** The plan and the path; or the path alone, held while a step changed on
   * it left a decision open. */
  const onPath = $derived(answer !== undefined && (planned || (app.pathHeld && answer.path.length > 0)))

  // A field changed: the facts are new, and so is the answer's shape.
  $effect(() => {
    void JSON.stringify(app.fields)
    untrack(() => (app.pathHeld = false))
  })

  // The plan's amount as it was before a change on the path, so the visitor
  // sees what it was (principle 22); gone once a field changes.
  let before = $state<string | undefined>()
  let shown: string | undefined
  let heard = $state('')
  /** The plan's height, last shown: the held path keeps its place. */
  let planHeight = $state(0)
  $effect(() => {
    const amount = planned ? answer?.planAmount?.amount : undefined
    const held = app.pathHeld
    untrack(() => {
      if (!held) before = undefined
      else if (amount !== undefined && shown !== undefined && amount !== shown) {
        before = shown
        heard = t.planUpdated(amount)
      }
      if (amount !== undefined) shown = amount
    })
  })

  function toHow(event: MouseEvent) {
    // The page's address holds the shared state (`link.ts`): scroll, don't jump.
    event.preventDefault()
    const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches
    document
      .querySelector('section.how')
      ?.scrollIntoView({ block: 'start', behavior: reduce ? 'auto' : 'smooth' })
  }
</script>

{#if answer}
  {@const state = answer.confidence.state}
  <section
    class="card answer state-{state}"
    class:on-path={onPath}
    aria-labelledby="answer-title"
    aria-busy={!app.answerFresh}
    class:stale={app.stale}
    {@attach glideHeight(SETTLE)}
  >
    <div class="content">
      <div class="confidence">
        <span class="chip">{t.confidenceChip[state]}</span>
        <span class="note">{answer.confidence.note}</span>
      </div>
      {#if onPath && planned && answer.planAmount}
        {@const amount = answer.planAmount}
        <div class="plan" bind:offsetHeight={planHeight}>
          <p class="plan-label">{t.thePlan}</p>
          <h2 id="answer-title" class="plan-sentence">{answer.plan}</h2>
          <div class="amount-row">
            <p class="amount">{amount.amount}</p>
            <p class="when">{amount.when}</p>
            {#if before}<p class="before">{t.beforeChange(before)}</p>{/if}
          </div>
          <p class="basis">
            {amount.basis}{#if amount.example}<span class="example">{amount.example}</span>{/if}
          </p>
          {#if answer.annuity}<p class="annuity">{answer.annuity}</p>{/if}
        </div>
      {:else if onPath}
        <!-- Held on the path, a decision open: as tall as the plan was, so
             the step being used doesn't move (principle 21). -->
        <div class="held" style:min-height={planHeight ? `${planHeight}px` : undefined}>
          <h2 id="answer-title">{answer.title}</h2>
          {#if answer.openDecisions > 0}<p class="left">{t.decisionsLeft(answer.openDecisions)}</p>{/if}
        </div>
      {:else}
        <h2 id="answer-title">{answer.title}</h2>
        <p class="body">{answer.body}</p>
        {#if answer.annuity}<p class="annuity">{answer.annuity}</p>{/if}
      {/if}
      {#if onPath}
        <Path {app} steps={answer.path} />
        {#if planned}
          <a class="to-how" href="#how-heading" onclick={toHow}
            >{t.toHow}<svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.4"
              aria-hidden="true"><path d="M12 5v14" /><path d="m19 12-7 7-7-7" /></svg
            ></a
          >
        {/if}
      {:else}
        <div class="lines">
          {#each answer.lines as line (line.kind)}
            <AnswerLine {app} {line} />
          {/each}
        </div>
        {#if answer.openDecisions > 0}
          <p class="left">{t.decisionsLeft(answer.openDecisions)}</p>
        {/if}
      {/if}
      <p class="visually-hidden" aria-live="polite">{heard}</p>
    </div>
  </section>
{/if}

<style>
  .answer {
    transition: opacity var(--quick);
    /* Scrolled to (a chip on a phone): a little room above it. */
    scroll-margin-top: 12px;
  }
  /* The sure answer alone wears the accent's border. */
  .answer.state-A {
    border: 2px solid var(--accent);
  }
  .answer.stale {
    opacity: 0.55;
  }
  /* See glideHeight: the content's margins counted in its height. */
  .content {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .confidence {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 10px;
  }
  .chip {
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 0.9rem;
    font-weight: 700;
  }
  .state-A .chip {
    background: var(--accent);
    color: var(--on-accent);
  }
  .state-B .chip {
    border: 1px solid var(--strong-border);
    background: var(--raised-strong);
  }
  .state-C .chip {
    border: 1px solid var(--warning-edge);
    background: var(--warning-tint);
    color: var(--warning);
  }
  .note {
    color: var(--weak);
    font-size: 0.92rem;
  }
  h2 {
    margin: 0;
    font-size: 1.45rem;
    font-weight: 600;
    line-height: 1.3;
  }
  .body,
  .annuity {
    max-width: 52rem;
    margin: 0;
    font-size: 1.02rem;
    line-height: 1.6;
  }
  .annuity {
    color: var(--weak);
  }
  .lines {
    border-top: 1px solid var(--border);
  }
  /* The plan, large: what to do in one sentence, then what it comes to. */
  .plan {
    display: grid;
    gap: 6px;
    margin-top: 8px;
  }
  .plan-label {
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
    font-weight: 700;
  }
  .state-A .plan-label {
    color: color-mix(in srgb, var(--accent) 55%, var(--text));
  }
  .plan-sentence {
    max-width: 48rem;
    font-size: 2rem;
    font-weight: 700;
  }
  .amount-row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 20px;
    margin-top: 14px;
  }
  .amount-row p,
  .basis {
    margin: 0;
  }
  .amount {
    font-size: 3.2rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    line-height: 1.05;
  }
  .when {
    font-size: 1rem;
  }
  .before {
    color: var(--weak);
    font-size: 0.9rem;
  }
  .basis {
    color: var(--weak);
    font-size: 0.92rem;
    line-height: 1.55;
  }
  /* A sentence of its own after the basis: the space is the margin's. */
  .example {
    margin-inline-start: 0.3em;
  }
  @container (max-width: 34rem) {
    .plan-sentence {
      font-size: 1.45rem;
    }
    .amount {
      font-size: 2.3rem;
    }
    .example {
      display: block;
      margin-top: 4px;
      margin-inline-start: 0;
    }
  }
  .to-how {
    display: inline-flex;
    align-items: center;
    align-self: flex-start;
    gap: 8px;
    min-height: 44px;
    padding: 0 18px;
    border-radius: 10px;
    background: var(--raised-strong);
    color: var(--text);
    font-weight: 700;
    text-decoration: none;
  }
  .state-A .to-how {
    background: var(--accent);
    color: var(--on-accent);
  }
  .held {
    display: grid;
    align-content: start;
    gap: 10px;
  }
  .left {
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
  }
</style>
