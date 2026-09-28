<script lang="ts">
  import type { AppState } from './app.svelte'
  import PlanPreview, { previewBeside, type Preview } from './PlanPreview.svelte'

  let { app }: { app: AppState } = $props()

  const brokers = $derived(
    app.brokers.map((broker) => ({
      broker,
      plans: app.listedPlans.filter((plan) => plan.broker === broker),
    })),
  )

  let preview = $state<Preview | null>(null)
</script>

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
      {#if app.brokerCaveats(broker).length > 0}
        <button
          class="icon-button warning"
          aria-label="Caveats about {broker.name}"
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
    <ul>
      {#each plans as plan (plan.id)}
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
          <label>
            <input
              type="checkbox"
              style:accent-color={plan.color}
              checked={app.selected.has(plan.id)}
              onchange={(event) => app.setSelected([plan.id], event.currentTarget.checked)}
            />
            {plan.info.name}
          </label>
          <button
            class="icon-button"
            aria-label="Change a copy of {plan.info.name}'s fees"
            onclick={() => app.draftCopyOf(plan)}>✎</button
          >
          <button
            class="icon-button"
            aria-label="About {plan.info.name}"
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
