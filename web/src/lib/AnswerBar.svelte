<script lang="ts">
  // The phone's answer bar (REDESIGN.md §4.3): while the facts are open, a
  // slim bar at the bottom of the screen shows the answer's chip and amount
  // after the change, and "לפני השינוי" what it was when "לשנות" was tapped,
  // so the visitor sees what changed without leaving the field (principle
  // 22). "לתשובה" closes the fields and takes them to the top of the answer:
  // a scroll the visitor asked for. It floats over the page, so nothing on
  // the page moves when it comes or goes (principle 21). A screen reader
  // hears one short sentence when the amount changes.
  import { tick, untrack } from 'svelte'
  import type { AppState } from './app.svelte'
  import { reveal } from './motion'
  import { showAnswer } from './scroll'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const chip = $derived(app.answer?.confidence.state)
  const amount = $derived(app.answerAmount)
  const before = $derived(
    app.amountBeforeEdit !== undefined && app.amountBeforeEdit !== amount ? app.amountBeforeEdit : undefined,
  )

  let heard = $state('')
  let last: string | undefined
  $effect(() => {
    const open = app.factsOpen
    const now = amount
    const fresh = app.answerFresh
    untrack(() => {
      if (!open) {
        heard = ''
        last = undefined
      } else if (last === undefined) last = now
      else if (fresh && now !== undefined && now !== last) {
        last = now
        heard = t.answerNow(now)
      }
    })
  })

  async function toAnswer() {
    app.closeFacts()
    await tick()
    showAnswer()
  }
</script>

{#if app.factsOpen && chip && amount}
  <aside class="bar" aria-label={t.answerAfterChange} transition:reveal>
    <div class="words">
      <div class="now">
        <span class="chip state-{chip}">{t.confidenceChip[chip]}</span>
        <span class="amount">{amount}</span>
      </div>
      {#if before}<p class="before">{t.beforeChange(before)}</p>{/if}
    </div>
    <button type="button" class="to-answer" onclick={toAnswer}
      >{t.toAnswer}<svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"><path d="M12 5v14" /><path d="m19 12-7 7-7-7" /></svg
      ></button
    >
  </aside>
{/if}
<p class="visually-hidden" aria-live="polite">{heard}</p>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px calc(12px + env(safe-area-inset-bottom));
    border-top: 1px solid var(--strong-border);
    background: var(--raised);
    box-shadow: 0 -12px 32px rgb(0 0 0 / 50%);
    pointer-events: auto;
  }
  .words {
    flex: 1;
    min-width: 0;
  }
  .now {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 2px 8px;
  }
  .chip {
    flex: none;
    padding: 0 8px;
    border: 1px solid var(--strong-border);
    border-radius: 999px;
    background: var(--raised-strong);
    font-size: 0.75rem;
    font-weight: 700;
  }
  .chip.state-A {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
  }
  .chip.state-C {
    border-color: var(--warning-edge);
    background: var(--warning-tint);
    color: var(--warning);
  }
  .amount {
    font-size: 1rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .before {
    margin: 2px 0 0;
    color: var(--weak);
    font-size: 0.8rem;
    font-variant-numeric: tabular-nums;
  }
  .to-answer {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 6px;
    min-height: 44px;
    padding: 0 12px;
    font-size: 0.9rem;
  }
</style>
