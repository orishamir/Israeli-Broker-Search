<script lang="ts">
  // The rungs (`app.ladder`: the same money after 1, 3, 5, 10, 20 years, or
  // for any day): what fits each, with how sure its sum is. Tapping a rung
  // sets the horizon (`app.setHorizonMonths(rung.months)`), which turns the
  // question into the one the guide answers. Screen s6.
  import type { AppState } from './app.svelte'
  import { shekels } from './format'
  import { HeldWhileTyping } from './held.svelte'
  import { glideHeight } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.ladder)
  const monthly = $derived(app.answers.monthly)
</script>

{#if held.current}
  <section class="card ladder" {@attach glideHeight()}>
    <div>
      <h2>{t.ladderTitle}</h2>
      {#if monthly}
        <p class="lead">{t.ladderLead(shekels(monthly))}</p>
      {/if}
      <div class="rungs">
        <div class="head" aria-hidden="true">
          <span>{t.ladderWhen}</span><span>{t.ladderMost}</span>
        </div>
        {#each held.current as rung (`${rung.months}-${rung.anyDay}`)}
          <button type="button" class="rung" onclick={() => app.setHorizonMonths(rung.months)}>
            <span class="when">{rung.anyDay ? t.ladderRungAnyDay : t.ladderRung(rung.months)}</span>
            <span class="what">
              {rung.kind.name}
              <span
                class="tag"
                class:sure={rung.kind.safety === 'SumKnown' || rung.kind.safety === 'RealValueKnown'}
                class:risky={rung.kind.safety === 'CanFall'}>{rung.kind.safetyTag}</span
              >
            </span>
            <bdi class="amount" dir="ltr">{shekels(rung.left.today)}</bdi>
          </button>
        {/each}
      </div>
      <p class="hint">{t.ladderTap}</p>
    </div>
  </section>
{/if}

<style>
  .lead,
  .hint {
    margin: 0 0 8px;
    color: var(--weak);
    font-size: 0.85rem;
  }

  .rungs {
    display: grid;
  }

  .head,
  .rung {
    display: grid;
    grid-template-columns: 4.5rem minmax(0, 1fr) auto;
    gap: 8px;
    align-items: baseline;
    padding: 8px 0;
    text-align: start;
  }

  .head {
    color: var(--weak);
    font-size: 0.75rem;
  }

  /* A whole row is a button, drawn as a row. */
  .rung {
    border: none;
    border-block-start: 1px solid var(--border);
    border-radius: 0;
    background: none;
  }

  .rung:hover:not(:disabled) {
    border-color: var(--border);
    border-block-start-color: var(--border);
    background: var(--accent-tint);
  }

  .when {
    color: var(--weak);
  }

  .what {
    text-align: start;
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 2px 8px;
  }

  .tag {
    color: var(--weak);
    font-size: 0.75rem;
  }

  .tag.sure {
    color: var(--good);
  }

  .tag.risky {
    color: var(--warning);
  }

  .amount {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
</style>
