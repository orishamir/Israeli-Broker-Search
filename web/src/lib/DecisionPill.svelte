<script lang="ts">
  // The phone's sign of a decision that opened out of sight (REDESIGN.md
  // §4.3, principle 22): a choice opened it below the screen, and the page
  // doesn't scroll by itself, so a pill at the bottom says so,
  // "נפתחה החלטה: <its question>", and takes the visitor there when tapped.
  // The button they pressed stays where it was. The pill goes once that
  // decision is on screen, or at the visitor's next change.
  import type { AppState } from './app.svelte'
  import { reveal } from './motion'
  import { decisionElement, inSight, showDecision } from './scroll'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const opened = $derived(app.openedOutOfSight)

  // The next change is the visitor's: the pill was about the one before.
  $effect(() => {
    if (opened && app.answerRequest !== opened.request) app.openedOutOfSight = undefined
  })

  // On screen, by the visitor's scrolling or the pill's: done.
  $effect(() => {
    if (!opened) return
    const check = () => {
      const group = decisionElement(opened.id)
      if (!group || inSight(group)) app.openedOutOfSight = undefined
    }
    addEventListener('scroll', check, { passive: true })
    addEventListener('resize', check, { passive: true })
    return () => {
      removeEventListener('scroll', check)
      removeEventListener('resize', check)
    }
  })
</script>

{#if opened}
  <button type="button" class="pill" onclick={() => showDecision(opened.id)} transition:reveal>
    <span>{t.newDecision(opened.question)}</span><svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"><path d="M12 5v14" /><path d="m19 12-7 7-7-7" /></svg
    >
  </button>
{/if}
<!-- One short sentence for a screen reader, the pill's own words. -->
<p class="visually-hidden" aria-live="polite">{opened ? t.newDecision(opened.question) : ''}</p>

<style>
  .pill {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-self: center;
    max-width: calc(100% - 24px);
    min-height: 44px;
    margin: 0 12px 12px;
    padding: 8px 16px;
    border: 1px solid var(--warning-edge);
    border-radius: 999px;
    background: var(--raised);
    box-shadow: 0 8px 24px rgb(0 0 0 / 50%);
    color: var(--text);
    font-size: 0.92rem;
    font-weight: 600;
    text-align: start;
    pointer-events: auto;
  }
  .pill svg {
    flex: none;
    color: var(--warning);
  }
</style>
