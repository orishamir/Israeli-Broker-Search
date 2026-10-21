<script lang="ts">
  // The one decision component (REDESIGN.md §2.4): why the question is the
  // visitor's, the options side by side (what each leaves and how sure that
  // is, what it asks, when it fits, a button in the visitor's words), and
  // the ₪ difference under them. The question itself is the line's heading,
  // which names the group. Once one is chosen, it gets the strong border and
  // the others dim but stay, and can still be chosen. Every word is the core's.
  //
  // On a phone (§4.3, amended 2026-10-21) each option is a card, and the
  // card is the choice: it opens with the visitor's answer and a radio's
  // mark, then where that leads, what it leaves and what it asks. Two cards
  // stand side by side with their rows shared ("מה זה דורש", "מתאים אם"),
  // the difference right under them, and the rest of each option's facts
  // behind "עוד על שתי האפשרויות"; three or more are a row each, with the
  // amounts in one column at the end. `.option` is what holds an option's
  // button, the control kept still when it's pressed.
  import type { Snippet } from 'svelte'
  import { MediaQuery } from 'svelte/reactivity'
  import type { DecisionData, OptData, Pick } from './core/core'
  import { reveal } from './motion'
  import { PHONE } from './scroll'
  import { t } from './text'

  let {
    decision,
    labelledby,
    onchoose,
    onkeep,
    extra,
  }: {
    decision: DecisionData
    /** The id of the question's heading. */
    labelledby: string
    onchoose: (pick: Pick) => void
    /** The option already chosen, chosen again: keeps it (the path closes
     * the decision it opened again). Without it, nothing happens. */
    onkeep?: () => void
    /** What an option holds besides its words (the cushion's expenses field). */
    extra?: Snippet<[OptData]>
  } = $props()

  const keyOf = (pick: Pick): string => JSON.stringify(pick)
  const chosen = $derived(decision.chosen === undefined ? undefined : keyOf(decision.chosen))
  const isPressed = (option: OptData) => chosen === keyOf(option.pick)
  const isOther = (option: OptData) => chosen !== undefined && !isPressed(option)
  const press = (option: OptData) => (isPressed(option) ? onkeep?.() : onchoose(option.pick))

  const phone = new MediaQuery(PHONE)
  /** Two columns, or a row each. */
  const columns = $derived(decision.options.length === 2)
  const anyAsks = $derived(decision.options.some((option) => option.asks.length > 0))
  const anyFits = $derived(decision.options.some((option) => option.fits !== undefined))
  /** The rows two cards share: the answer, where it leads, what it leaves,
   * and what it asks and when it fits, where any option says. */
  /** Where the answer is the option's name ("קרן כספית"), "← קרן כספית"
   * would only say it again. */
  const titled = (option: OptData) => option.title !== option.button
  const anyTitled = $derived(decision.options.some(titled))
  const rows = $derived(2 + Number(anyTitled) + Number(anyAsks) + Number(anyFits))
  /** What the table leaves out, by option: its other facts, and, in rows,
   * when it fits. */
  const rest = $derived(
    decision.options.map((option) => [
      ...option.facts.filter((fact) => !option.asks.includes(fact)),
      ...(!columns && option.fitsIf ? [option.fitsIf] : []),
    ]),
  )
  const anyRest = $derived(rest.some((facts) => facts.length > 0))
  /** "עוד" is open for this question: another decision in the same place
   * (another example) starts closed. */
  let moreFor = $state<string | undefined>()
  const more = $derived(moreFor === decision.question)
  const moreId = $props.id()
</script>

{#snippet choose(option: OptData, card = false)}
  <button class="choose" aria-pressed={isPressed(option)} onclick={() => press(option)}
    >{#if card}<span class="mark" aria-hidden="true"></span>{/if}{option.button}</button
  >
{/snippet}

{#snippet amount(option: OptData)}
  {#if option.amount}<p class="amount">{option.amount}</p>{/if}
  {#if option.basis}<p class="basis">{option.basis}</p>{/if}
  <!-- A second amount in its own unit (a pension a month): one line. -->
  {#if option.also}<p class="also">{option.also}</p>{/if}
{/snippet}

<div class="decision" role="group" aria-labelledby={labelledby} data-decision={decision.id}>
  <p class="why">{decision.why}</p>
  {#if phone.current}
    <!-- Each option is one card, and the card is the choice: it opens with
         the visitor's answer (the button, stretched over the card), then
         where it leads, what it leaves, and what it asks. Two cards side by
         side share their rows, so the same facts line up across. -->
    <div class="options cards" class:columns class:rows={!columns} style:--rows={rows}>
      {#each decision.options as option (keyOf(option.pick))}
        <div
          class="option tile"
          class:pressed={isPressed(option)}
          class:other={isOther(option)}
          data-pick={keyOf(option.pick)}
        >
          {@render choose(option, true)}
          {#if columns}
            {#if titled(option)}
              <h4 class="to"><span aria-hidden="true">←&nbsp;</span>{option.title}</h4>
            {:else if anyTitled}
              <div class="to"></div>
            {/if}
            <div class="end">{@render amount(option)}</div>
            {#if anyAsks}
              <div class="cell">
                <p class="label">{t.tableAsks}</p>
                {#each option.asks as each, index (index)}<p class="asks">{each}</p>{/each}
              </div>
            {/if}
            {#if anyFits}
              <div class="cell">
                <p class="label">{t.tableFits}</p>
                <p class="asks">{option.fits}</p>
              </div>
            {/if}
          {:else}
            <div class="main">
              {#if titled(option)}
                <h4 class="to"><span aria-hidden="true">←&nbsp;</span>{option.title}</h4>
              {/if}
              {#each option.asks as each, index (index)}<p class="asks">{each}</p>{/each}
            </div>
            <div class="end">{@render amount(option)}</div>
            {#if extra}<div class="extra">{@render extra(option)}</div>{/if}
          {/if}
        </div>
      {/each}
    </div>
    <!-- What the chosen option asks for besides (the expenses field): under
         the cards, the width of the card. -->
    {#if extra && columns}
      {#each decision.options as option (keyOf(option.pick))}{@render extra(option)}{/each}
    {/if}
  {:else}
    <div class="options" class:three={decision.options.length > 2}>
      {#each decision.options as option (keyOf(option.pick))}
        <div
          class="option"
          class:pressed={isPressed(option)}
          class:other={isOther(option)}
          data-pick={keyOf(option.pick)}
        >
          <h4 class="title">{option.title}</h4>
          {@render amount(option)}
          {#if option.facts.length > 0}
            <ul class="facts">
              {#each option.facts as fact, index (index)}<li>{fact}</li>{/each}
            </ul>
          {/if}
          {#if option.fitsIf}<p class="fits">{option.fitsIf}</p>{/if}
          {@render extra?.(option)}
          {@render choose(option)}
        </div>
      {/each}
    </div>
  {/if}
  {#if decision.difference}<p class="difference">{decision.difference}</p>{/if}
  {#if phone.current && anyRest}
    <button
      type="button"
      class="more"
      aria-expanded={more}
      aria-controls={moreId}
      onclick={() => (moreFor = more ? undefined : decision.question)}
      >{t.moreAboutOptions(decision.options.length)}<svg
        class:turned={more}
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"><path d="m6 9 6 6 6-6"></path></svg
      ></button
    >
    <div id={moreId} class="rest" class:columns>
      {#if more}
        {#each decision.options as option, at (keyOf(option.pick))}
          {#if rest[at].length > 0}
            <div class="rest-option" class:other={isOther(option)} transition:reveal>
              <h5>{option.title}</h5>
              <ul class="facts">
                {#each rest[at] as fact, index (index)}<li>{fact}</li>{/each}
              </ul>
            </div>
          {/if}
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .decision {
    container-type: inline-size;
    display: grid;
    gap: 12px;
  }
  .why {
    margin: 0;
    color: var(--weak);
    font-size: 0.95rem;
    line-height: 1.55;
  }
  /* Side by side while each option has room to read; stacked on a phone. */
  .options {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }
  @container (min-width: 34rem) {
    .options:not(.three) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @container (min-width: 50rem) {
    .options.three {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 16px;
    border: 2px solid var(--border);
    border-radius: var(--radius);
    background: var(--raised);
    transition:
      border-color var(--quick),
      background var(--quick),
      color var(--quick);
  }
  .option.pressed {
    border-color: var(--text);
  }
  /* Dimmed, not hidden: still readable (the weak text keeps 4.5:1 on the
     card), still a choice. */
  .option.other {
    background: var(--surface);
    color: var(--weak);
  }
  .title {
    margin: 0;
    font-size: 1.02rem;
    line-height: 1.4;
  }
  .amount {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  /* Close under the amount it goes with, one step smaller. */
  .also {
    margin: -4px 0 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .facts {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.95rem;
    line-height: 1.5;
  }
  .fits {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    line-height: 1.5;
  }
  .choose {
    margin-top: auto;
    min-height: 44px;
    padding: 6px 14px;
    border-color: var(--strong-border);
    font-weight: 600;
  }
  .choose[aria-pressed='true'] {
    border-color: var(--text);
    background: var(--text);
    color: var(--background);
    cursor: default;
  }
  .choose[aria-pressed='true']:hover {
    background: var(--text);
  }
  .difference {
    margin: 0;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--raised);
    font-size: 0.97rem;
    line-height: 1.6;
  }

  /* Close under the amount, smaller: what it rests on. */
  .basis {
    margin: -6px 0 0;
    color: var(--weak);
    font-size: 0.85rem;
    line-height: 1.4;
  }

  /* ── On a phone: a card per option, the card the choice ── */
  .cards {
    gap: 8px;
    font-size: 0.85rem;
    line-height: 1.45;
  }
  /* Two side by side, their rows shared, so the facts line up across. */
  .cards.columns {
    grid-template-rows: repeat(var(--rows), auto);
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  /* Three or more: a row each, the amounts in one column at the end. */
  .cards.rows {
    grid-template-columns: minmax(0, 1fr) minmax(0, 8.5rem);
  }
  .tile {
    position: relative;
    transition:
      border-color var(--quick),
      background var(--quick),
      color var(--quick),
      scale var(--quick);
    display: grid;
    gap: 0;
    padding: 10px;
  }
  .columns .tile {
    grid-row: 1 / span var(--rows);
    grid-template-rows: subgrid;
  }
  .rows .tile {
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    column-gap: 12px;
  }
  .tile:not(.pressed):hover {
    border-color: var(--strong-border);
  }
  .tile:has(.choose:focus-visible) {
    box-shadow: var(--focus-ring);
  }
  /* The visitor's answer, a radio's mark before it; the whole card takes
     the tap. */
  .tile .choose {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 0;
    padding: 0 0 8px;
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    background: none;
    color: inherit;
    text-align: start;
    font-size: 0.92rem;
    font-weight: 700;
    line-height: 1.35;
  }
  .rows .tile .choose {
    grid-column: 1 / -1;
  }
  /* Over the card's other parts: as grid items, they paint after it. */
  .tile .choose::after {
    content: '';
    position: absolute;
    z-index: 1;
    inset: 0;
    border-radius: var(--radius);
  }
  .tile .choose:hover,
  .tile .choose[aria-pressed='true'],
  .tile .choose[aria-pressed='true']:hover {
    background: none;
    color: inherit;
  }
  .tile .choose:focus-visible {
    outline: none;
  }
  /* The card sinks, not its answer: a scaled button would hold its own
     tap (a transform is the containing block of its ::after), and the tap
     would end on the amount under it. */
  .tile .choose:active {
    scale: none;
  }
  .tile:has(.choose:active) {
    scale: 0.98;
    transition-duration: 60ms;
  }
  .mark {
    flex: none;
    box-sizing: border-box;
    width: 18px;
    height: 18px;
    margin-top: 1px;
    border: 2px solid var(--strong-border);
    border-radius: 50%;
    transition:
      border-color var(--quick),
      background var(--quick);
  }
  .choose[aria-pressed='true'] .mark {
    border-color: var(--text);
    background: var(--text);
    box-shadow: inset 0 0 0 3px var(--raised);
  }
  .to {
    margin: 0;
    padding-top: 8px;
    color: var(--weak);
    font-size: 0.8rem;
    font-weight: 700;
    line-height: 1.35;
  }
  .tile .end {
    display: grid;
    align-content: start;
    gap: 6px;
    padding: 4px 0 8px;
  }
  .tile .amount {
    font-size: 1rem;
  }
  .tile .basis {
    margin-top: -6px;
    font-size: 0.78rem;
  }
  .tile .also {
    margin-top: -2px;
    font-size: 0.82rem;
  }
  .cell {
    padding: 8px 0;
    border-top: 1px solid var(--border);
  }
  .cell:last-child {
    padding-bottom: 0;
  }
  .label {
    margin: 0 0 2px;
    color: var(--weak);
    font-size: 0.72rem;
    font-weight: 700;
  }
  .asks {
    margin: 0;
  }
  .asks + .asks {
    margin-top: 4px;
  }
  .rows .main {
    display: grid;
    align-content: start;
    gap: 4px;
    padding-block: 8px 2px;
  }
  .rows .main .to {
    padding: 0;
  }
  .rows .end {
    text-align: end;
    padding-top: 8px;
  }
  /* Above the card's tap, so its field takes the pointer. */
  .rows .extra {
    position: relative;
    z-index: 2;
    grid-column: 1 / -1;
  }
  .rest-option.other {
    color: var(--weak);
  }

  /* "עוד על שתי האפשרויות": the rest of each option's facts. */
  .more {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    justify-self: start;
    min-height: 44px;
    padding: 0 4px;
    border: none;
    background: none;
    color: var(--accent);
    font-size: 0.92rem;
  }
  .more:hover {
    background: none;
    text-decoration: underline;
  }
  .more svg {
    transition: rotate var(--quick);
  }
  .more svg.turned {
    rotate: 180deg;
  }
  .rest {
    display: grid;
    gap: 12px;
  }
  .rest:empty {
    display: none;
  }
  .rest.columns {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .rest h5 {
    margin: 0 0 4px;
    font-size: 0.92rem;
  }
  .rest .facts {
    font-size: 0.9rem;
  }
</style>
