<script lang="ts">
  // The page: a header, two routes in one layout (the guide, and the
  // workbench while the switch is on), a footer. The guide's answer is in
  // REDESIGN.md §2.1's order: the card, the outcome chart, how to do it,
  // the gain, the horizons. The workbench adds its levers beside the
  // questions, and its table and charts under the answer.
  import { setContext } from 'svelte'
  import type { Term } from './lib/core/core'
  import { keepAddress } from './lib/address.svelte'
  import { AppState } from './lib/app.svelte'
  import { OPEN_CARD } from './lib/cards'
  import AnswerBar from './lib/AnswerBar.svelte'
  import AnswerCard from './lib/AnswerCard.svelte'
  import DecisionPill from './lib/DecisionPill.svelte'
  import GainSplit from './lib/GainSplit.svelte'
  import HorizonSection from './lib/HorizonSection.svelte'
  import HowTo from './lib/HowTo.svelte'
  import OutcomeChart from './lib/OutcomeChart.svelte'
  import Charts from './lib/Charts.svelte'
  import DetailsDialog from './lib/DetailsDialog.svelte'
  import ExpertSwitch from './lib/ExpertSwitch.svelte'
  import Glossary from './lib/Glossary.svelte'
  import LearnCard from './lib/LearnCard.svelte'
  import { decodeLocation } from './lib/link'
  import Questions from './lib/Questions.svelte'
  import Share from './lib/Share.svelte'
  import Within from './lib/Within.svelte'
  import Workbench from './lib/Workbench.svelte'
  import { reveal } from './lib/motion'
  import { t } from './lib/text'

  // Opened from a link, perhaps: its state comes first.
  const app = new AppState(decodeLocation(location))
  keepAddress(app)
  /** Where to report a mistake: the accessibility statement's last link. */
  // Every tip's "עוד על זה ›" opens a card.
  setContext(OPEN_CARD, (term: Term) => app.openCard(term))

  const issues = $derived(app.about.accessibility.items.at(-1)?.url)
  /** The answer shown is the last one: the worker is at the next. */
  const awaiting = $derived(!app.answerFresh && !app.stale)
  /** The phone's dock: room kept at the page's end, so its last lines can
   * be scrolled above it. */
  let dockHeight = $state(0)
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
<div class="layout">
  <div class="column questions-column">
    <div class="item" style:--order="1"><Questions {app} /></div>
    {#if app.expert}
      <div class="item" style:--order="2" transition:reveal><Workbench {app} /></div>
    {/if}
  </div>
  <div class="column answer-column" class:stale={app.stale} inert={app.stale}>
    {#if app.inputError !== undefined}
      <p class="card error item wide" style:--order="3" transition:reveal>{t.checkInputs(app.inputError)}</p>
    {/if}
    <!-- The answer's parts: while the worker works out the answer for what
         the visitor just did, the last one stays, faded after a moment (the
         review's N1), as the inputs' error state fades it. -->
    <div class="item wide answers" class:awaiting style:--order="3"><AnswerCard {app} /></div>
    <div class="item wide answers" class:awaiting style:--order="3"><OutcomeChart {app} /></div>
    <div class="item wide answers" class:awaiting style:--order="3"><HowTo {app} /></div>
    <div class="item wide answers" class:awaiting style:--order="3"><GainSplit {app} /></div>
    <!-- The workbench's table and charts, with the switch on. -->
    {#if app.expert}
      <div class="item wide" style:--order="5" transition:reveal><Within {app} /></div>
      <div class="item wide" style:--order="9" transition:reveal><Charts {app} /></div>
    {/if}
    <!-- Last: its rows come from the worker a moment after the answer, and
         grow it, so nothing anyone is working with sits below it. -->
    <div class="item wide" style:--order="10"><HorizonSection {app} /></div>
  </div>
</div>

<footer class="page-footer">
  {#each app.footer as line (line)}<span>{line}</span>{/each}
  {#if issues}<a href={issues} target="_blank" rel="noreferrer">{t.reportAMistake} ↗</a>{/if}
  <button class="link" onclick={() => app.openAbout(app.about.sections.length)}
    >{t.accessibilityStatement}</button
  >
</footer>

<!-- The phone's (REDESIGN.md §4.3): over the page's bottom edge, the sign of
     a decision opened out of sight, and the answer while the facts are open. -->
<div class="dock" bind:offsetHeight={dockHeight}>
  <DecisionPill {app} />
  <AnswerBar {app} />
</div>
<div class="dock-room" style:height="{dockHeight}px" aria-hidden="true"></div>

<DetailsDialog {app} />
<LearnCard {app} />
<Glossary {app} />

<style>
  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
    padding: 20px 24px 0;
    max-width: var(--page-max);
    margin-inline: auto;
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
    max-width: var(--page-max);
    margin-inline: auto;
  }
  /* Phone: the columns dissolve into one, in the order each item says. */
  .column {
    display: contents;
  }
  .item {
    order: var(--order);
    min-width: 0;
  }
  /* A part with nothing to show (the gain, where fees and tax decide
     nothing) takes no gap either. */
  .item:empty {
    display: none;
  }
  .stale {
    opacity: 0.5;
  }
  .answers {
    transition: opacity var(--quick);
  }
  /* Only once it's been a moment: an answer that comes at once never
     flickers. */
  .answers.awaiting {
    opacity: 0.55;
    transition: opacity var(--quick) var(--awaiting-after);
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
  .dock {
    position: fixed;
    inset-inline: 0;
    bottom: 0;
    z-index: 5;
    display: grid;
    pointer-events: none;
  }
  @media (width >= 800px) {
    .dock {
      display: none;
    }
  }
  /* Quiet: read once, by whoever looks for it. */
  .page-footer {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    padding: 8px 24px 32px;
    max-width: var(--page-max);
    margin-inline: auto;
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
