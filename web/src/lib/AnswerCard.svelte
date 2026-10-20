<script lang="ts">
  // The answer card (REDESIGN.md §2.1–§2.4): how sure the site is (the chip
  // and its note), a one-line title and what's clear, the four lines, and
  // then the plan once every decision is made, or how many are left. Only a
  // sure answer (state A) wears the accent. The words are the core's; the
  // chip's, the pills' and the frame's are the page's.
  import type { AppState } from './app.svelte'
  import AnswerLine from './AnswerLine.svelte'
  import { glideHeight, SETTLE } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const answer = $derived(app.answer)
</script>

{#if answer}
  {@const state = answer.confidence.state}
  <section
    class="card answer state-{state}"
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
      <h2 id="answer-title">{answer.title}</h2>
      <p class="body">{answer.body}</p>
      {#if answer.annuity}<p class="annuity">{answer.annuity}</p>{/if}
      <div class="lines">
        {#each answer.lines as line (line.kind)}
          <AnswerLine {app} {line} />
        {/each}
      </div>
      {#if answer.openDecisions === 0 && answer.plan}
        <div class="plan">
          <h3>{t.thePlan}</h3>
          <p>{answer.plan}</p>
        </div>
      {:else if answer.openDecisions > 0}
        <p class="left">{t.decisionsLeft(answer.openDecisions)}</p>
      {/if}
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
  .plan {
    display: grid;
    gap: 4px;
    padding: 14px 16px;
    border: 1px solid var(--strong-border);
    border-radius: var(--radius);
    background: var(--raised);
  }
  .state-A .plan {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    background: var(--accent-tint);
  }
  .plan h3 {
    margin: 0;
    font-size: 0.95rem;
  }
  .plan p {
    margin: 0;
    font-size: 1.05rem;
    line-height: 1.6;
  }
  .left {
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
  }
</style>
