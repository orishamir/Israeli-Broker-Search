<script lang="ts">
  // Screens s2, s3, s6: "what fits money like this". What the guide chose,
  // in the terms people meet ("מה זה? ›" opens its card), the amount, why,
  // what a bigger first deposit would open, and, for money with no cushion,
  // the plan in two parts. "לשנות ›" offers the other kinds; the sentences
  // are the core's, the frame here is the page's.
  import type { AppState } from './app.svelte'
  import BankChips from './BankChips.svelte'
  import { shekels } from './format'
  import { kindKey } from './kinds'
  import { reveal } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const recommendation = $derived(app.recommendation)
  const plan = $derived(recommendation?.twoParts)

  let choosing = $state(false)
  const existing = $derived(
    typeof app.answers.situation === 'object' && 'ExistingPortfolio' in app.answers.situation,
  )

  // The bank's branch plan is ticked as the recommendation names it (6.8).
  $effect(() => {
    if (existing && app.recommendationFresh) app.tickYourBranch()
  })
</script>

<section class="card answer">
  <div class="top">
    <h2>{t.answerTitle}</h2>
  </div>
  {#if recommendation}
    <div class="what">
      <p>
        {#each [...recommendation.what, ...recommendation.whatThen] as span, at (at)}{#if span.bold}<strong
              >{span.text}</strong
            >{:else}{span.text}{/if}{/each}
      </p>
      <p class="actions">
        {#if recommendation.kind.card}
          {@const card = recommendation.kind.card}
          <button class="link" onclick={() => app.openCard(card)}>{t.whatIsIt}</button>
        {/if}
        <button class="link" aria-expanded={choosing} onclick={() => (choosing = !choosing)}
          >{t.changeKind}</button
        >
        {#if app.overridden}
          <button class="link" onclick={() => app.clearOverrides()}>{t.backToRecommendation}</button>
        {/if}
      </p>
    </div>
    {#if choosing}
      <div class="picker" transition:reveal>
        <h3>{t.chooseKindTitle}</h3>
        <div class="options">
          {#each recommendation.arenaChoices as choice (choice.arena)}
            <button
              class="chip"
              onclick={() => {
                app.chooseArena(choice.arena)
                choosing = false
              }}>{choice.name}</button
            >
          {/each}
        </div>
        <ul>
          {#each recommendation.ranked as ranked (kindKey(ranked.kind))}
            <li>
              <button
                class="kind"
                class:chosen={kindKey(ranked.kind) === kindKey(recommendation.kind.kind)}
                onclick={() => {
                  app.chooseKind(ranked.kind)
                  choosing = false
                }}
              >
                <span>{ranked.name}</span>
                <span class="amount">{shekels(ranked.left.today)}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
    {#if existing}
      <div class="bank"><BankChips {app} /></div>
    {/if}

    {#if !plan}
      <div class="amount-box">
        <div class="big">{shekels(recommendation.left.today)}</div>
        <p class="sub">
          {recommendation.leftNote.today}{#if recommendation.leftNote.nominal}
            &nbsp;· <bdi>{recommendation.leftNote.nominal}</bdi>{/if}
        </p>
      </div>
    {/if}
    <p class="base">{recommendation.currentAccountLine}</p>
    {#if recommendation.alsoPossibleText}
      <p class="also" transition:reveal>{recommendation.alsoPossibleText}</p>
    {/if}

    <h3>{t.whyThis}</h3>
    <ul class="reasons">
      {#each recommendation.reasons as reason (reason.text)}
        <li>{reason.text}</li>
      {/each}
    </ul>

    {#if recommendation.unlockText}
      <p class="unlock" transition:reveal>{recommendation.unlockText}</p>
    {/if}

    {#if recommendation.movePortfolioText}
      <p class="move" transition:reveal>
        {recommendation.movePortfolioText}
        {#each recommendation.movePortfolioSources as source (source.url)}
          <a href={source.url} target="_blank" rel="noopener noreferrer">{source.name}</a>{' '}
        {/each}
      </p>
    {/if}
  {/if}
</section>

<style>
  .answer {
    border-top: 3px solid var(--good);
  }
  h2 {
    margin: 0;
    color: var(--weak);
    font-size: 0.9rem;
    font-weight: 500;
  }
  h3 {
    margin: 16px 0 6px;
    font-size: 1rem;
  }
  .what p {
    margin: 8px 0 0;
    font-size: 1.15rem;
    line-height: 1.6;
  }
  .what .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    margin-top: 2px;
    font-size: 0.9rem;
  }
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: inherit;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
  .picker {
    margin-top: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--raised);
  }
  .picker h3 {
    margin-top: 0;
  }
  .picker .options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 8px;
  }
  .chip {
    min-height: 33px;
    padding: 4px 14px;
    border-radius: 999px;
  }
  .picker ul {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .kind {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    min-height: 33px;
    text-align: start;
  }
  .kind.chosen {
    border-color: var(--accent);
    background: var(--accent-tint);
  }
  .amount {
    font-variant-numeric: tabular-nums;
  }
  .bank {
    margin-top: 12px;
  }
  .big {
    margin-top: 8px;
    font-size: 2rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .sub,
  .base {
    margin: 4px 0 0;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .base {
    margin-top: 10px;
  }
  .move {
    margin: 14px 0 0;
    font-size: 0.9rem;
  }
  .move a {
    display: inline-block;
    margin-inline-end: 8px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .also {
    margin: 6px 0 0;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .reasons {
    display: grid;
    gap: 8px;
    margin: 0;
    padding-inline-start: 20px;
  }
  .unlock {
    margin: 14px 0 0;
    padding: 8px 12px;
    border: 1px solid var(--warning-edge);
    border-radius: 8px;
    background: var(--warning-tint);
    font-size: 0.9rem;
  }
</style>
