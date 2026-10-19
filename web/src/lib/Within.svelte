<script lang="ts">
  // "איפה הכי זול לקנות אותה": one list for both arenas, its columns named
  // by the core (`recommendation.columns`, `columnsFor`). The guide shows the
  // top three with a link to all; the switch on shows the full table: the
  // plans, or for the S&P 500 the ways, each opening onto the plans that can
  // hold it. Screens s2, s4, s7. The rows are made in `within.ts`.
  import { tick } from 'svelte'
  import { flip } from 'svelte/animate'
  import { SvelteSet } from 'svelte/reactivity'
  import { planId, type AppState } from './app.svelte'
  import { wayColorOf } from './chart-entries'
  import { columnsFor, type ColumnData, type ColumnId } from './core/core'
  import { dateText, typedPercent } from './format'
  import { duration, easeOut, glideHeight, reducedMotion, SETTLE } from './motion'
  import { t } from './text'
  import Tip from './Tip.svelte'
  import { planRows, rankedRows, usedColumns, wayRows, type Row } from './within'

  let { app }: { app: AppState } = $props()

  const recommendation = $derived(app.recommendation)
  /** The ways side by side: the S&P 500, with the switch on. */
  const ways = $derived(
    app.expert && app.thing === 'sp500' && app.waysComparison && 'ways' in app.waysComparison
      ? app.waysComparison
      : undefined,
  )
  /** The plans on one exchange: another index or a stock, with the switch on. */
  const plans = $derived(
    app.expert && app.thing === 'other' && app.comparison && 'results' in app.comparison
      ? app.comparison
      : undefined,
  )
  const indexList = $derived(ways !== undefined || plans !== undefined)
  const arena = $derived(indexList ? 'Index' : (recommendation?.arena ?? 'Safe'))

  const whyNot = (reason: Parameters<AppState['whyNotWords']>[0]) => app.whyNotWords(reason)
  const rows: Row[] = $derived.by(() => {
    if (ways) {
      return wayRows(
        ways.ways,
        wayColorOf(app.wayChoices),
        (key) => app.plansById.get(planId(key))?.label ?? '',
        (way) => app.resultsOf(way.plans),
        whyNot,
      )
    }
    if (plans) return planRows(plans.results, whyNot)
    return recommendation ? rankedRows(recommendation.plans, recommendation.arena, app.todaysMoney) : []
  })
  const columns: ColumnData[] = $derived.by(() => {
    const named = indexList ? columnsFor('Index') : (recommendation?.columns ?? [])
    return usedColumns(named, rows)
  })

  /** The ways opened onto their plans. */
  const opened = new SvelteSet<string>()
  const toggle = (id: string) => {
    if (!opened.delete(id)) opened.add(id)
  }
  const shown = $derived(
    app.expert
      ? rows.flatMap((row) => [row, ...(opened.has(row.id) ? (row.more ?? []) : [])])
      : rows.slice(0, 3),
  )
  const child = (row: Row) => rows.every(({ key }) => key !== row.key)

  /** What a column's "?" says, for the inputs as they are. */
  function explanation(id: ColumnId): string {
    const money = app.inTodaysMoney ? t.inTodaysMoneyNote : ''
    if (id === 'LeftAfterTax') return t.leftAfterTaxTip + money
    if (id === 'CostAYear') return t.costAYearTip + (arena === 'Index' ? t.costAYearProduct : '')
    if (id === 'Tax')
      return t.taxTip(typedPercent(app.inputs.inflationPercent ?? 0), ...app.surtaxTerms) + money
    // How many purchases the monthly deposits make, for the fees' "?".
    const { years, buyEveryMonths, monthlyDeposit, exchange } = app.inputs
    const purchases =
      years && buyEveryMonths && monthlyDeposit
        ? { years, everyMonths: buyEveryMonths, abroad: exchange !== 'Tlv' }
        : undefined
    return t.feesPaidTip(app.sellAtEnd, purchases) + money
  }

  /** The plan's column takes a set share of the table and the amounts split
   * the rest. Sized by what's in them, a long note under one plan widened
   * its column and moved every number when the security or exchange
   * changed; now the note wraps, and only its row grows taller. */
  const width = $derived({ plan: '30%', amount: `${70 / Math.max(columns.length, 1)}%` })

  const dates = $derived(
    app.exchange !== 'Tlv' && app.ratesStatus.kind === 'downloaded'
      ? `${app.dataDates}\u00a0· ${t.ratesOf(dateText(app.ratesStatus.date))}`
      : app.dataDates,
  )
  const title = $derived(ways ? t.waysTable : t.withinTitle)
</script>

{#if app.recommendation || plans || ways}
  <section class="card within">
    <h2>{title}</h2>
    {#if ways}<p class="hint">{t.waysTableHint}</p>{/if}
    {#if rows.length === 0}
      <p class="hint">{t.noPlansTicked}</p>
    {:else}
      <!-- Its height glides as the rows come and go (the S&P 500's ways open
           onto their plans), so the cards below move with it. -->
      <div class="glides" {@attach glideHeight(SETTLE)}>
        <div class="scrolls">
          <table>
            <thead>
              <tr>
                <th class="rank"><span class="visually-hidden">{t.rank}</span></th>
                <th class="plan" style:--share={width.plan}>{ways ? t.wayAndWho : t.plan}</th>
                {#each columns as { id, name } (id)}
                  <th class="amount" style:--share={width.amount}
                    ><span class="titled">{name}<Tip about={name}>{explanation(id)}</Tip></span></th
                  >
                {/each}
              </tr>
            </thead>
            <tbody>
              <!-- Rows glide to their new place when the ranking changes. Plans
                   that can't be used (the security isn't offered, the deposits
                   are over a fund's ceiling, its money is still locked) have no
                   line to highlight. -->
              {#each shown as row (row.key)}
                <tr
                  animate:flip={{ duration: duration(SETTLE), easing: easeOut }}
                  class:not-offered={!row.cells}
                  class:first-unfit={row.firstUnfit}
                  class:child={app.expert && child(row)}
                  class:linked={row.linked}
                  class:highlighted={row.linked && app.highlighted.has(row.id)}
                  class:pinned={row.linked && app.pinned.has(row.id)}
                  style:--plan-color={row.color}
                  onmouseenter={() => (app.hovered = row.linked ? row.id : null)}
                  onmouseleave={() => (app.hovered = null)}
                  onclick={() => row.linked && app.togglePin(row.id)}
                >
                  <td class="rank">{row.rank ?? '–'}</td>
                  <td class="plan">
                    <!-- In the row, so it slides with it. -->
                    {#if row.firstUnfit}<span class="unfit">{t.needBiggerFirstDeposit}</span>{/if}
                    <div>
                      {#if row.color}
                        <span class="mark" class:yours={row.yours} style:--plan-color={row.color}></span>
                      {/if}
                      <span class="names">
                        <span class:best={row.rank === 1}>{row.name}</span>
                        {#if row.subtitle}<span class="broker">{row.subtitle}</span>{/if}
                        {#each row.notes as note (note.text)}
                          <span
                            class={note.kind === 'warning' ? 'warning' : 'note'}
                            class:tax-note={note.kind === 'tax'}
                            class:pension-note={note.kind === 'pension'}>{note.text}</span
                          >
                        {/each}
                        {#if row.more && app.expert}
                          <button
                            class="link more"
                            aria-expanded={opened.has(row.id)}
                            onclick={(event) => {
                              event.stopPropagation()
                              toggle(row.id)
                            }}>{opened.has(row.id) ? t.hideWayPlans : t.showWayPlans(row.more.length)}</button
                          >
                        {/if}
                      </span>
                    </div>
                  </td>
                  {#if row.cells}
                    {#each columns as { id } (id)}
                      <td class="amount">
                        {#if id === 'Fees' && row.feesLink && row.cells[id] !== undefined}
                          <!-- Not also a click on the row, which would unpin it. -->
                          <button
                            class="link"
                            aria-label={t.feesLink(row.name, row.cells[id])}
                            onclick={(event) => {
                              event.stopPropagation()
                              app.showFees(row.id)
                              // Where the breakdown is, below the table (far below, on phones).
                              tick().then(() =>
                                document.getElementById('chart')?.scrollIntoView({
                                  block: 'start',
                                  behavior: reducedMotion ? 'auto' : 'smooth',
                                }),
                              )
                            }}>{row.cells[id]} ›</button
                          >
                        {:else}
                          {row.cells[id] ?? ''}
                        {/if}
                      </td>
                    {/each}
                  {:else if row.absent}
                    <td class="amount" colspan={Math.max(columns.length, 1)}
                      >{row.absent.cell}<Tip about={row.absent.about}>{row.absent.reason}</Tip></td
                    >
                  {/if}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
      {#if !app.expert && rows.length > 3}
        <p class="more-line">
          <button class="link" onclick={() => (app.expert = true)}>{t.allPlans(rows.length)}</button>
          <span class="hint">{t.fullTableHint}</span>
        </p>
      {/if}
    {/if}
    <!-- The same under every broker's plan, so said once. -->
    {#if plans?.usEstateTax}<p class="kind-flag">⚠ {plans.usEstateTax}</p>{/if}
    {#if app.expert && indexList}<p class="dates">{dates}</p>{/if}
  </section>
{/if}

<style>
  .within {
    padding: 14px 4px 4px;
  }
  h2 {
    margin: 0 12px 6px;
    font-size: 1rem;
  }
  .hint {
    margin: 0 12px 6px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .more-line {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 2px 10px;
    margin: 6px 12px 10px;
    font-size: 0.85rem;
  }
  .more-line .hint {
    margin: 0;
  }
  .kind-flag {
    margin: 4px 10px 8px;
    color: var(--warning);
    font-size: 0.75rem;
  }
  .dates {
    margin: 4px 10px 8px;
    color: var(--weak);
    font-size: 0.75rem;
  }
  .scrolls {
    overflow-x: auto;
    /* Not auto too: rows sliding to their new places reach past the table's
       new bottom for a moment, which showed a vertical scroll bar. */
    overflow-y: hidden;
    /* A glow at the left edge, where a right-to-left table ends, while
       there's more to scroll to: the cover moves with the content and hides
       it at the end. Light, not a shadow, which a dark card hid. */
    background:
      linear-gradient(to right, var(--surface) 40%, transparent) left / 40px 100% no-repeat local,
      radial-gradient(farthest-side at 0% 50%, color-mix(in srgb, var(--text) 16%, transparent), transparent)
        left / 20px 100% no-repeat scroll;
  }
  /* See glideHeight: its content's margins counted in its height. */
  .glides > :global(*) {
    display: flow-root;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    font-size: 0.9rem;
  }
  th,
  td {
    padding: 9px 10px;
    text-align: start;
    white-space: nowrap;
  }
  th.plan,
  th.amount {
    width: var(--share);
  }
  th {
    border-bottom: 1px solid var(--border);
    color: var(--weak);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    vertical-align: bottom;
  }
  /* A heading may take two lines, so a column is as wide as its numbers,
     not its heading. */
  th.amount {
    max-width: 6.5rem;
    white-space: normal;
  }
  td {
    border-bottom: 1px solid var(--border);
  }
  tbody tr:last-child td {
    border-bottom: none;
  }
  tbody tr {
    position: relative;
    transition: background var(--quick);
  }
  tbody tr.linked {
    cursor: pointer;
  }
  tbody tr.linked:hover {
    background: var(--raised);
  }
  /* A tap shows at once (the hovered row is highlighted instead). */
  tbody tr.linked:active {
    background: var(--raised-strong);
  }
  /* Tinted in the plan's color, like its line in the chart. */
  tbody tr.highlighted {
    --row-background: color-mix(in srgb, var(--plan-color) 16%, var(--surface));
    background: var(--row-background);
  }
  /* A way's plans under it, set in a little. */
  tbody tr.child .plan {
    padding-inline-start: 24px;
  }
  tbody tr.child td {
    font-size: 0.85rem;
  }
  /* At the row's start, on the right: an inset shadow has no logical form. */
  tbody tr.pinned td:first-child {
    box-shadow: inset -3px 0 var(--plan-color);
  }
  .rank {
    width: 1px;
    padding-inline-end: 0;
    color: var(--weak);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    text-align: end;
  }
  .plan div {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .names {
    display: grid;
    justify-items: start;
    line-height: 1.25;
  }
  .broker,
  .warning,
  .note {
    font-size: 0.75rem;
  }
  .broker,
  .note {
    color: var(--weak);
  }
  .warning {
    color: var(--warning);
  }
  /* Above the first plan the first deposit can't open: why it and the rest
     below come after the others. */
  .unfit {
    display: block;
    margin-block-end: 6px;
    color: var(--weak);
    font-size: 0.75rem;
    white-space: normal;
  }
  /* As readable as the plan's name: it's why the row's tax is what it is,
     and what its amount buys as a pension. */
  .tax-note,
  .pension-note {
    color: var(--text);
  }
  /* Long ones wrap, rather than widen the table past its card. */
  .warning,
  .note {
    white-space: normal;
  }
  .more {
    margin-top: 2px;
    font-size: 0.75rem;
  }
  /* When the table is wider than its card, the amounts scroll sideways
     under the plan. */
  @container (width < 640px) {
    .plan {
      position: sticky;
      inset-inline-start: 0;
      z-index: 1;
      background: var(--row-background, var(--surface));
    }
    tbody tr.linked:hover:not(.highlighted) .plan {
      background: var(--raised);
    }
    .names {
      min-width: 0;
      white-space: normal;
    }
    th,
    td {
      box-sizing: border-box;
    }
    .rank {
      min-width: 1.5rem;
    }
  }
  /* A phone: tighter cells. */
  @container (width < 440px) {
    th,
    td {
      padding: 8px 6px;
    }
  }
  /* While the table scrolls, the screen ends where a column ends: as many
     columns after the plan's as fit have a set width, and the plan's column
     takes the rest of the card, so they show whole and what's past them is
     a swipe away. A share of the table's width (the desktop's way) cut a
     column mid-number. Minimums, since a table wider than its box gives
     each column the least its content needs and ignores widths. */
  @container (width < 316px) {
    th.plan {
      min-width: calc(100cqi - 1.5rem - 1 * 5.6rem);
    }
    th:nth-child(n + 3):nth-child(-n + 3) {
      min-width: 5.6rem;
    }
  }
  @container (316px <= width < 440px) {
    th.plan {
      min-width: calc(100cqi - 1.5rem - 2 * 5.6rem);
    }
    th:nth-child(n + 3):nth-child(-n + 4) {
      min-width: 5.6rem;
    }
  }
  @container (440px <= width < 540px) {
    th.plan {
      min-width: calc(100cqi - 1.5rem - 3 * 6rem);
    }
    th:nth-child(n + 3):nth-child(-n + 5) {
      min-width: 6rem;
    }
  }
  @container (540px <= width < 640px) {
    th.plan {
      min-width: calc(100cqi - 1.5rem - 4 * 6rem);
    }
    th:nth-child(n + 3):nth-child(-n + 6) {
      min-width: 6rem;
    }
  }
  tbody tr.not-offered {
    cursor: default;
  }
  /* Right, not end: numbers run left to right in Hebrew too, so their last
     digits line up on the right. */
  .amount {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .not-offered {
    color: var(--weak);
  }
  /* Why a plan has no numbers is a sentence: it wraps. */
  .not-offered .amount {
    white-space: normal;
  }
  .best {
    font-weight: 700;
  }
  .mark {
    display: inline-block;
    flex: none;
    box-sizing: border-box;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--plan-color);
  }
  /* Your own plans are hollow, like their dotted lines in the charts. */
  .mark.yours {
    border: 2px solid var(--plan-color);
    background: none;
  }
  .not-offered .mark {
    opacity: 0.3;
  }
  /* A link to the plan's fee breakdown, or a way's plans, colored like one. */
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
</style>
