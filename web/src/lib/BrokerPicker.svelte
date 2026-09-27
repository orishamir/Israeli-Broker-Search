<script lang="ts">
  import { fade } from 'svelte/transition'
  import type { AppState, Plan } from './app.svelte'
  import FeesForInputs from './FeesForInputs.svelte'
  import { duration } from './motion'

  let { app }: { app: AppState } = $props()

  const brokers = $derived(
    app.brokers.map((broker) => ({
      broker,
      plans: app.plans.filter((plan) => plan.broker === broker),
    })),
  )

  /** The plan under the mouse, and where to show its preview: beside it,
   * going down from it, or up from it when it's low in the window. */
  let preview = $state<{ plan: Plan; left: number; top?: number; bottom?: number } | null>(null)

  function showPreview(plan: Plan, row: HTMLElement) {
    const { top, bottom, right } = row.getBoundingClientRect()
    const left = right + 16
    preview =
      top < innerHeight / 2 ? { plan, left, top: top - 8 } : { plan, left, bottom: innerHeight - bottom - 8 }
  }
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
          onmouseenter={(event) => showPreview(plan, event.currentTarget)}
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
            aria-label="About {plan.info.name}"
            onclick={() => (app.details = { kind: 'plan', plan })}>ℹ</button
          >
        </li>
      {/each}
    </ul>
  </div>
{/each}

<!-- Shown after a moment of hovering; it glides from plan to plan. Not on
     touch screens, where there's no hovering. -->
{#if preview && !app.details}
  {@const plan = preview.plan}
  <div
    class="popover preview"
    style:top={preview.top === undefined ? undefined : `${preview.top}px`}
    style:bottom={preview.bottom === undefined ? undefined : `${preview.bottom}px`}
    style:left="{preview.left}px"
    style:--plan-color={plan.color}
    in:fade={{ delay: 350, duration: duration(150) }}
    out:fade={{ duration: duration(100) }}
  >
    <strong>{plan.info.name}</strong>
    <span class="broker-of">{plan.broker.name}</span>
    <p>{plan.info.description}</p>
    <FeesForInputs {app} {plan} tips={false} />
    <p class="hint">ℹ shows the full details and caveats</p>
  </div>
{/if}

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
  /* The ℹ of a plan shows on hover; it's always there on touch screens. */
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
  .preview {
    position: fixed;
    z-index: 10;
    width: 360px;
    max-width: 360px;
    border-left: 3px solid var(--plan-color);
    pointer-events: none;
    transition:
      top 180ms ease-out,
      bottom 180ms ease-out;
  }
  @media (hover: none) {
    .preview {
      display: none;
    }
  }
  .broker-of {
    margin-left: 6px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .preview p {
    margin: 6px 0 10px;
  }
  .preview .hint {
    margin: 10px 0 0;
    font-size: 0.8rem;
    color: var(--weak);
  }
</style>
