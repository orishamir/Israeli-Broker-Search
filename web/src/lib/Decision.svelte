<script lang="ts">
  // The one decision component (REDESIGN.md §2.4): why the question is the
  // visitor's, the options side by side (what each leaves and how sure that
  // is, what it asks, when it fits, a button in the visitor's words), and
  // the ₪ difference under them. The question itself is the line's heading,
  // which names the group. Once one is chosen, it gets the strong border and
  // the others dim but stay, and can still be chosen. Every word is the core's.
  //
  // On a phone (§4.3) the same decision is a comparison table: two options
  // are two columns, with the rows "מה יהיה בסוף", "מה זה דורש" and "מתאים
  // אם", the difference right under them, and the rest of each option's
  // facts behind "עוד על שתי האפשרויות"; three or more are a row each, with
  // the amounts in one column at the end. `.option` is what holds an
  // option's button, the control kept still when it's pressed.
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

{#snippet choose(option: OptData)}
  <button class="choose" aria-pressed={isPressed(option)} onclick={() => press(option)}
    >{option.button}</button
  >
{/snippet}

{#snippet amount(option: OptData)}
  {#if option.amount}<p class="amount">{option.amount}</p>{/if}
  <!-- A second amount in its own unit (a pension a month): one line. -->
  {#if option.also}<p class="also">{option.also}</p>{/if}
{/snippet}

<div class="decision" role="group" aria-labelledby={labelledby} data-decision={decision.id}>
  <p class="why">{decision.why}</p>
  {#if phone.current && columns}
    <!-- The options are the columns: a screen reader names each cell by its
         option. -->
    <table class="options table">
      <thead>
        <tr>
          {#each decision.options as option (keyOf(option.pick))}
            <th scope="col" class="head" class:pressed={isPressed(option)} class:other={isOther(option)}>
              {option.title}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        <tr class="label"><th colspan="2" scope="colgroup">{t.tableEnd}</th></tr>
        <tr>
          {#each decision.options as option (keyOf(option.pick))}
            <td class:other={isOther(option)}>{@render amount(option)}</td>
          {/each}
        </tr>
        {#if anyAsks}
          <tr class="label"><th colspan="2" scope="colgroup">{t.tableAsks}</th></tr>
          <tr>
            {#each decision.options as option (keyOf(option.pick))}
              <td class:other={isOther(option)}>
                {#each option.asks as each, index (index)}<p class="asks">{each}</p>{:else}<p class="none">
                    —
                  </p>{/each}
              </td>
            {/each}
          </tr>
        {/if}
        {#if anyFits}
          <tr class="label"><th colspan="2" scope="colgroup">{t.tableFits}</th></tr>
          <tr>
            {#each decision.options as option (keyOf(option.pick))}
              <td class:other={isOther(option)}>{option.fits ?? '—'}</td>
            {/each}
          </tr>
        {/if}
        <tr class="buttons">
          {#each decision.options as option (keyOf(option.pick))}
            <td
              class="option"
              class:pressed={isPressed(option)}
              class:other={isOther(option)}
              data-pick={keyOf(option.pick)}
            >
              {@render choose(option)}
            </td>
          {/each}
        </tr>
      </tbody>
    </table>
    <!-- What the chosen option asks for besides (the expenses field): under
         the table, the width of the card. -->
    {#if extra}
      {#each decision.options as option (keyOf(option.pick))}{@render extra(option)}{/each}
    {/if}
  {:else if phone.current}
    <div class="options rows">
      {#each decision.options as option (keyOf(option.pick))}
        <div
          class="option row"
          class:pressed={isPressed(option)}
          class:other={isOther(option)}
          data-pick={keyOf(option.pick)}
        >
          <div class="main">
            <h4 class="title">{option.title}</h4>
            {#each option.asks as each, index (index)}<p class="asks">{each}</p>{/each}
          </div>
          <div class="end">{@render amount(option)}</div>
          {#if extra}<div class="extra">{@render extra(option)}</div>{/if}
          {@render choose(option)}
        </div>
      {/each}
    </div>
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

  /* ── On a phone, two options: a table, the options its columns ── */
  .table {
    width: calc(100% + 16px);
    margin-inline: -8px;
    table-layout: fixed;
    border-collapse: separate;
    border-spacing: 8px 0;
    font-size: 0.92rem;
    line-height: 1.5;
  }
  .table th,
  .table td {
    padding: 0 8px 8px;
    vertical-align: top;
    text-align: start;
    font-weight: normal;
    transition: color var(--quick);
  }
  .table .head {
    padding-top: 10px;
    border: 2px solid var(--border);
    border-bottom: none;
    border-radius: var(--radius) var(--radius) 0 0;
    background: var(--raised);
    font-size: 1rem;
    font-weight: 700;
    line-height: 1.4;
  }
  .table .head.pressed {
    border-color: var(--text);
  }
  .table .label th {
    padding: 8px 8px 2px;
    color: var(--weak);
    font-size: 0.8rem;
    font-weight: 700;
  }
  .table tbody .label:not(:first-child) th {
    border-top: 1px solid var(--border);
  }
  .table .amount {
    font-size: 1.05rem;
  }
  /* The button's cell: no box of its own, the button its width. */
  .table .option {
    display: table-cell;
    padding: 4px 0 0;
    border: none;
    border-radius: 0;
    background: none;
  }
  .table .choose,
  .rows .choose {
    width: 100%;
  }
  .table .other,
  .rest-option.other {
    color: var(--weak);
  }
  .asks,
  .none {
    margin: 0;
  }
  .asks + .asks {
    margin-top: 4px;
  }
  .none {
    color: var(--weak);
  }

  /* ── On a phone, three or more: a row each, the amounts at the end ── */
  .rows {
    grid-template-columns: minmax(0, 1fr) minmax(0, 9rem);
    gap: 0;
  }
  .rows .option {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: subgrid;
    gap: 6px 12px;
    padding: 12px;
    margin-top: 8px;
  }
  .rows .option:first-child {
    margin-top: 0;
  }
  .row .main {
    display: grid;
    align-content: start;
    gap: 4px;
    font-size: 0.92rem;
    line-height: 1.5;
  }
  .row .end {
    text-align: end;
  }
  .row .amount {
    font-size: 1.02rem;
  }
  /* No gap above it in a cell, as there is in a card. */
  .table .also,
  .row .also {
    margin-top: 2px;
  }
  .row .extra,
  .row .choose {
    grid-column: 1 / -1;
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
