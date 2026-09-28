<script lang="ts">
  import type { AppState } from './app.svelte'
  import PlanPreview, { previewBeside, type Preview } from './PlanPreview.svelte'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  const brokers = $derived(
    app.brokers.map((broker) => ({
      broker,
      plans: app.listedPlans.filter((plan) => plan.broker === broker),
    })),
  )

  let preview = $state<Preview | null>(null)
</script>

<p class="intro">Ticked at first: each broker's usual plan, the one a new customer gets.</p>

{#each brokers as { broker, plans } (broker.name)}
  {@const ids = plans.map((plan) => plan.id)}
  {@const count = ids.filter((id) => app.selected.has(id)).length}
  <div class="broker">
    <div class="row">
      <label class="broker-name">
        <input
          type="checkbox"
          checked={count === ids.length}
          indeterminate={count > 0 && count < ids.length}
          onchange={(event) => app.setSelected(ids, event.currentTarget.checked)}
        />
        {broker.name}
      </label>
      <!-- ⚠ means one thing everywhere: a number that may be too low. -->
      {#if app.brokerCaveats(broker).some((group) => group.kind === 'MayCostMore')}
        <button
          class="icon-button warning"
          aria-label="May cost more at {broker.name}: see why"
          onclick={() => (app.details = { kind: 'broker', broker })}>⚠</button
        >
      {/if}
      <button
        class="icon-button"
        aria-label="About {broker.name}"
        onclick={() => (app.details = { kind: 'broker', broker })}>ℹ</button
      >
    </div>
    <p class="date">{broker.tariffDate}</p>
    <!-- Named, since its plans' names repeat across brokers. -->
    <ul aria-label={broker.name}>
      {#each plans as plan (plan.id)}
        {@const usual = plan.key.kind === 'listed' && plan.key.plan === broker.newCustomerPlan}
        <li
          class="row"
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
              >usual<Tip about="usual">
                <p>
                  The plan a new customer usually gets from {broker.name}: an investment house's joining offer
                  rather than its full tariff, a bank's online prices rather than a customer group's. The
                  usual plans are compared at first; tick others to add them.
                </p>
              </Tip></span
            >
          {/if}
          <button
            class="icon-button"
            aria-label="Change a copy of {plan.label}'s fees"
            onclick={() => app.draftCopyOf(plan)}>✎</button
          >
          <button
            class="icon-button"
            aria-label="About {plan.label}"
            onclick={() => (app.details = { kind: 'plan', plan })}>ℹ</button
          >
        </li>
      {/each}
    </ul>
  </div>
{/each}

<PlanPreview {app} {preview} />

<style>
  .broker + .broker {
    margin-top: 14px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  label {
    cursor: pointer;
  }
  .row label {
    flex: 1;
  }
  /* "usual" follows the name, and the ✎ and ℹ stay at the end. */
  .row label.beside-usual {
    flex: 0 1 auto;
  }
  .usual {
    flex: 1;
    margin-left: 6px;
    color: var(--weak);
    font-size: 0.75rem;
  }
  .intro {
    margin: 0 0 10px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .broker-name {
    font-weight: 600;
  }
  .date {
    margin: 0 0 4px 24px;
    font-size: 0.75rem;
    color: var(--weak);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    padding: 2px 4px 2px 22px;
    border-radius: 6px;
    transition: background var(--quick);
  }
  li:hover {
    background: var(--raised);
  }
  /* A plan's ✎ and ℹ show on hover; they're always there on touch screens. */
  @media (hover: hover) {
    li .icon-button {
      opacity: 0;
      transition: opacity var(--quick);
    }
    li:hover .icon-button,
    li .icon-button:focus-visible {
      opacity: 1;
    }
  }
</style>
