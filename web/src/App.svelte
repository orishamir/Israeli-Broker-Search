<script lang="ts">
  // The page: a header, two routes in one layout (the guide, and the
  // workbench while the switch is on), a footer. The layout is package L's
  // to finish (the best-bar on phones, the spacing, the snapshots); the
  // components in it are other packages' (each says which at its top).
  import { setContext } from 'svelte'
  import type { Term } from './lib/core/core'
  import { keepAddress } from './lib/address.svelte'
  import { AppState } from './lib/app.svelte'
  import { OPEN_CARD } from './lib/cards'
  import Answer from './lib/Answer.svelte'
  import Charts from './lib/Charts.svelte'
  import Alternatives from './lib/Alternatives.svelte'
  import DetailsDialog from './lib/DetailsDialog.svelte'
  import Donut from './lib/Donut.svelte'
  import ExpertSwitch from './lib/ExpertSwitch.svelte'
  import Glossary from './lib/Glossary.svelte'
  import Ladder from './lib/Ladder.svelte'
  import LearnCard from './lib/LearnCard.svelte'
  import { decodeLocation } from './lib/link'
  import PortfolioDialog from './lib/PortfolioDialog.svelte'
  import Questions from './lib/Questions.svelte'
  import Share from './lib/Share.svelte'
  import Steps from './lib/Steps.svelte'
  import Within from './lib/Within.svelte'
  import Workbench from './lib/Workbench.svelte'
  import { shekels } from './lib/format'
  import { reducedMotion, reveal } from './lib/motion'
  import { t } from './lib/text'

  // Opened from a link, perhaps: its state comes first.
  const app = new AppState(decodeLocation(location))
  keepAddress(app)
  /** Where to report a mistake: the accessibility statement's last link. */
  // Every tip's "עוד על זה ›" opens a card.
  setContext(OPEN_CARD, (term: Term) => app.openCard(term))

  /** On phones the answer is screens below the questions, so its headline
   * follows along the bottom of the screen until it's reached: while it's
   * below the screen, and no field is being typed in (the keyboard would push
   * the bar up over the field). */
  let answer = $state<HTMLElement>()
  let answerBelow = $state(false)
  let typing = $state(false)
  const placeBar = () => {
    answerBelow = !!answer && answer.getBoundingClientRect().top > innerHeight
  }
  $effect(() => {
    const element = answer
    if (!element) return
    // Watched two ways: the observer sees the answer move as the questions
    // above it change size, and scrolling is watched itself, since a jump
    // straight past it (tapping the bar, a test) crosses no threshold.
    const observer = new IntersectionObserver(placeBar)
    observer.observe(element)
    placeBar()
    return () => observer.disconnect()
  })
  // The bar comes back a moment after the field is left: a tap on a button
  // blurs the field first, and a bar sliding in over that button at once
  // took the tap's release, so the button never answered.
  let typingEnds: ReturnType<typeof setTimeout> | undefined
  const stopTyping = () => {
    clearTimeout(typingEnds)
    typingEnds = setTimeout(() => (typing = false), 350)
  }
  const typingIn = (target: EventTarget | null) =>
    target instanceof HTMLElement &&
    target.matches('input:not([type=checkbox], [type=radio], [type=range]), select')
  const bar = $derived.by(() => {
    const recommendation = app.recommendation
    if (!recommendation) return undefined
    const amount = app.inTodaysMoney ? recommendation.left.today : recommendation.left.nominal
    return { kind: recommendation.kind.name, amount: shekels(amount) }
  })

  const issues = $derived(app.about.accessibility.items.at(-1)?.url)
</script>

<!-- index.html's skeleton repeats the title. -->
<header class="top">
  <h1>{t.title}</h1>
  <nav class="actions">
    <button class="link" onclick={() => (app.glossaryOpen = true)}>{t.glossary}</button>
    <button class="link" onclick={() => app.openAbout()}>{t.aboutLink}</button>
    <Share {app} />
    <ExpertSwitch {app} />
  </nav>
</header>

<!-- Phone: one column, in this order (`order` on each item). Wide: two
     columns, the questions (and the workbench) beside the answer. -->
<svelte:window
  onscroll={placeBar}
  onfocusin={(event) => {
    if (typingIn(event.target)) {
      clearTimeout(typingEnds)
      typing = true
    } else stopTyping()
  }}
  onfocusout={stopTyping}
/>

<div class="layout">
  <div class="column questions-column">
    <div class="item" style:--order="1"><Questions {app} /></div>
    {#if app.expert}
      <div class="item" style:--order="2" transition:reveal><Workbench {app} /></div>
    {/if}
    <div class="item" style:--order="6"><Steps {app} /></div>
  </div>
  <div class="column answer-column" class:stale={app.stale} inert={app.stale}>
    {#if app.inputError !== undefined}
      <p class="card error item wide" style:--order="3" transition:reveal>{t.checkInputs(app.inputError)}</p>
    {/if}
    <div class="item wide" style:--order="3" bind:this={answer}><Answer {app} /></div>
    <div class="item" style:--order="4"><Alternatives {app} /></div>
    <div class="item" style:--order="7"><Donut {app} /></div>
    <div class="item wide" style:--order="5"><Within {app} /></div>
    <div class="item wide" style:--order="8"><Ladder {app} /></div>
    <!-- Package K: the workbench's charts, with the switch on. -->
    {#if app.expert}
      <div class="item wide" style:--order="9" transition:reveal><Charts {app} /></div>
    {/if}
  </div>
</div>

<footer class="page-footer">
  {#each app.footer as line (line)}<span>{line}</span>{/each}
  {#if issues}<a href={issues} target="_blank" rel="noreferrer">{t.reportAMistake} ↗</a>{/if}
  <button class="link" onclick={() => app.openAbout(app.about.sections.length)}
    >{t.accessibilityStatement}</button
  >
</footer>

<!-- The answer's headline along the bottom of a phone's screen while the
     answer is below it; tapping it goes there. -->
{#if bar && answerBelow && !typing}
  <button
    class="best-bar"
    onclick={() => answer?.scrollIntoView({ block: 'start', behavior: reducedMotion ? 'auto' : 'smooth' })}
  >
    <span class="bar-kind">{bar.kind}</span>
    <span class="bar-amount"
      ><bdi>{bar.amount}</bdi>{#if app.inTodaysMoney}<span class="bar-note">{t.bestBarToday}</span>{/if}</span
    >
    <span class="bar-arrow" aria-hidden="true">↓</span>
  </button>
{/if}

<DetailsDialog {app} />
<LearnCard {app} />
<Glossary {app} />
<PortfolioDialog {app} />

<style>
  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
    padding: 20px 24px 0;
  }
  .top h1 {
    margin: 0;
    font-size: 1.4rem;
    letter-spacing: -0.01em;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 16px;
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
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 16px;
    padding: 16px 24px 24px;
    align-items: start;
  }
  /* Phone: the columns dissolve into one, in the order each item says. */
  .column {
    display: contents;
  }
  .item {
    order: var(--order);
    min-width: 0;
  }
  .stale {
    opacity: 0.5;
  }
  .error {
    color: var(--error);
  }
  @media (width < 800px) {
    .top {
      padding: 16px 12px 0;
    }
    .layout {
      padding: 12px;
      gap: 12px;
    }
  }
  @media (width >= 800px) {
    .layout {
      grid-template-columns: 360px minmax(0, 1fr);
      gap: 20px;
    }
    .column {
      display: grid;
      gap: 16px;
      grid-template-columns: minmax(0, 1fr);
      align-content: start;
    }
    .item {
      order: 0;
    }
    .answer-column {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .answer-column .wide {
      grid-column: 1 / -1;
    }
  }
  /* Phones only (the desktop has the answer beside the questions): it slides
     up when it appears and is just gone when it hides. */
  .best-bar {
    display: none;
  }
  @media (width < 800px) {
    .best-bar {
      position: fixed;
      inset: auto 12px calc(10px + env(safe-area-inset-bottom));
      z-index: 2;
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        'kind arrow'
        'amount arrow';
      gap: 0 12px;
      align-items: center;
      padding: 8px 14px;
      border: 1px solid var(--strong-border);
      border-top: 2px solid var(--good);
      background: var(--raised);
      box-shadow: var(--shadow);
      text-align: start;
      transition:
        translate 200ms var(--ease-out),
        opacity 200ms var(--ease-out);
    }
    @starting-style {
      .best-bar {
        opacity: 0;
        translate: 0 12px;
      }
    }
  }
  .bar-kind {
    grid-area: kind;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .bar-amount {
    grid-area: amount;
    font-weight: 600;
  }
  .bar-note {
    margin-inline-start: 8px;
    color: var(--weak);
    font-size: 0.8rem;
    font-weight: 400;
  }
  .bar-arrow {
    grid-area: arrow;
    color: var(--accent);
    font-size: 1.2rem;
  }
  /* Quiet: read once, by whoever looks for it. */
  .page-footer {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    padding: 8px 24px 32px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .page-footer a {
    color: var(--accent);
  }
  @media (width < 800px) {
    .page-footer {
      padding-inline: 12px;
    }
  }
</style>
