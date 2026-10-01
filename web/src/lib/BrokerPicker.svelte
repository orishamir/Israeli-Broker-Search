<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity'
  import type { AppState } from './app.svelte'
  import PlanPreview, { previewBeside, type Preview } from './PlanPreview.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** The brokers, or with `funds` the kinds of fund, one line each: its
   * name, which of its plans are ticked, and a tick for all of them. A tap on
   * the line unfolds its plans, each with its own tick. */
  let { app, funds = false }: { app: AppState; funds?: boolean } = $props()

  const id = $props.id()
  const brokers = $derived(
    app.brokers
      .filter((broker) => (broker.kind === 'Funds') === funds)
      .map((broker) => ({
        broker,
        plans: app.listedPlans.filter((plan) => plan.broker === broker),
      })),
  )

  /** The brokers unfolded, by name: all folded at first. */
  const unfolded = new SvelteSet<string>()
  const fold = (name: string) => {
    if (!unfolded.delete(name)) unfolded.add(name)
  }

  let preview = $state<Preview | null>(null)
  const warned = (broker: (typeof brokers)[number]['broker']) =>
    app.brokerCaveats(broker).some((group) => group.kind === 'MayCostMore')
  // Where any line has a ⚠, every line keeps room for one, so the counts stay
  // in one column.
  const anyWarning = $derived(brokers.some(({ broker }) => warned(broker)))
</script>

{#each brokers as { broker, plans }, index (broker.name)}
  {@const ids = plans.map((plan) => plan.id)}
  {@const ticked = plans.filter((plan) => app.selected.has(plan.id))}
  {@const open = unfolded.has(broker.name)}
  {@const line = `${id}-${index}`}
  <div class="broker">
    <div class="row">
      <!-- Named by the broker, as is the line that unfolds it: one ticks,
           the other opens. -->
      <input
        type="checkbox"
        aria-labelledby={line}
        checked={ticked.length === ids.length}
        indeterminate={ticked.length > 0 && ticked.length < ids.length}
        onchange={(event) => app.setSelected(ids, event.currentTarget.checked)}
      />
      <button
        class="fold"
        aria-expanded={open}
        aria-controls="{line}-plans"
        aria-labelledby={line}
        aria-describedby="{line}-ticked {line}-count"
        onclick={() => fold(broker.name)}
      >
        <span class="names">
          <span class="broker-name" id={line}>{broker.name}</span>
          <span class="ticked" id="{line}-ticked">{ticked.map((plan) => plan.info.name).join(', ')}</span>
        </span>
        <span class="count" id="{line}-count">{t.countOf(ticked.length, ids.length)}</span>
        <svg class="chevron" viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 4.5 6 8l3.5-3.5" /></svg>
      </button>
      <!-- ⚠ means one thing everywhere: a number that may be too low. -->
      {#if warned(broker)}
        <button
          class="icon-button warning"
          aria-label={t.mayCostMoreAt(broker.name)}
          onclick={() => (app.details = { kind: 'broker', broker })}>⚠</button
        >
      {:else if anyWarning}
        <span class="icon-slot"></span>
      {/if}
      <button
        class="icon-button"
        aria-label={t.about(broker.name)}
        onclick={() => (app.details = { kind: 'broker', broker })}>ℹ</button
      >
    </div>
    <!-- A fund's tax rule is what sets it apart: said outright, folded or not. -->
    {#if broker.taxRule}<p class="rule">{broker.taxRule}</p>{/if}
    <div class="plans" id="{line}-plans" hidden={!open}>
      <p class="date">{broker.tariffDate}</p>
      <!-- Named, since its plans' names repeat across brokers. -->
      <ul aria-label={broker.name}>
        {#each plans as plan (plan.id)}
          <!-- A fund has no usual plan: its first is only what savers pay on average. -->
          {@const usual = !funds && plan.key.kind === 'listed' && plan.key.plan === broker.newCustomerPlan}
          <li
            class="plan"
            onmouseenter={(event) => (preview = previewBeside(plan, event.currentTarget))}
            onmouseleave={() => (preview = null)}
            onpointerenter={(event) => {
              // Also stands out in the table and the chart. Mouse only: a tap
              // would leave it standing out, with nothing to end the hover.
              if (event.pointerType === 'mouse') app.hovered = plan.id
            }}
            onpointerleave={(event) => {
              if (event.pointerType === 'mouse') app.hovered = null
            }}
          >
            <label class:beside-usual={usual}>
              <input
                type="checkbox"
                style:accent-color={plan.color}
                checked={app.selected.has(plan.id)}
                onchange={(event) => app.setSelected([plan.id], event.currentTarget.checked)}
              />
              {plan.info.name}
            </label>
            <!-- Outside the label: its ? would become part of the checkbox's name. -->
            {#if usual}
              <span class="usual"
                >{t.usual}<Tip about={t.usual}>
                  <p>{broker.usualPlan}</p>
                </Tip></span
              >
            {/if}
            <button
              class="icon-button"
              aria-label={t.changeACopy(plan.label)}
              onclick={() => app.draftCopyOf(plan)}>✎</button
            >
            <button
              class="icon-button"
              aria-label={t.about(plan.label)}
              onclick={() => (app.details = { kind: 'plan', plan })}>ℹ</button
            >
          </li>
        {/each}
      </ul>
    </div>
  </div>
{/each}

<PlanPreview {app} {preview} />

<style>
  .broker + .broker {
    border-top: 1px solid var(--border);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .row > input {
    margin-inline-end: 4px;
  }
  /* The whole line unfolds, all but its tick and its buttons. */
  .fold {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    padding: 7px 6px;
    border: none;
    border-radius: 8px;
    background: none;
    text-align: start;
  }
  .fold:hover {
    background: var(--raised);
  }
  /* A wide line sinks less than a button does, or it would lurch. */
  .fold:active {
    scale: 0.99;
  }
  .names {
    flex: 1;
    display: grid;
    min-width: 0;
  }
  .broker-name {
    font-weight: 600;
  }
  .ticked,
  .count {
    color: var(--weak);
    font-size: 0.8rem;
  }
  .ticked:empty {
    display: none;
  }
  .count {
    white-space: nowrap;
  }
  .chevron {
    flex: none;
    width: 12px;
    height: 12px;
    fill: none;
    stroke: var(--weak);
    stroke-width: 1.5;
    transition: rotate var(--quick);
  }
  [aria-expanded='true'] .chevron {
    rotate: 180deg;
  }
  .rule {
    margin: 0 0 8px;
    margin-inline-start: 30px;
    font-size: 0.8rem;
    line-height: 1.35;
  }
  .plans {
    padding-bottom: 8px;
  }
  .date {
    margin: 0 0 4px;
    margin-inline-start: 30px;
    font-size: 0.75rem;
    color: var(--weak);
  }
  label {
    cursor: pointer;
  }
  .plan label {
    flex: 1;
  }
  /* "usual" follows the name, and the ✎ and ℹ stay at the end. */
  .plan label.beside-usual {
    flex: 0 1 auto;
  }
  .usual {
    flex: 1;
    margin-inline-start: 6px;
    color: var(--weak);
    font-size: 0.75rem;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .plan {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px 4px;
    padding-inline-start: 26px;
    border-radius: 6px;
    transition: background var(--quick);
  }
  .plan:hover {
    background: var(--raised);
  }
  /* A plan's ✎ and ℹ show on hover; they're always there on touch screens. */
  @media (hover: hover) {
    .plan .icon-button {
      opacity: 0;
      transition: opacity var(--quick);
    }
    .plan:hover .icon-button,
    .plan .icon-button:focus-visible {
      opacity: 1;
    }
  }
</style>
