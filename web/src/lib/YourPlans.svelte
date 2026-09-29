<script lang="ts">
  import type { AppState } from './app.svelte'
  import PlanPreview, { previewBeside, type Preview } from './PlanPreview.svelte'
  import { t } from './text'

  /** The user's own plans, to tick for comparing and to open in the editor. */
  let { app }: { app: AppState } = $props()

  const plans = $derived(app.plans.filter((plan) => plan.yours))
  let preview = $state<Preview | null>(null)
</script>

{#if plans.length === 0}
  <p class="empty">
    {t.thinkLowerFees}
  </p>
{:else}
  <ul>
    {#each plans as plan (plan.id)}
      <li
        onmouseenter={(event) => (preview = previewBeside(plan, event.currentTarget))}
        onmouseleave={() => (preview = null)}
        onpointerenter={(event) => {
          // As in the brokers card: mouse only.
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
          <span class="ring" style:--plan-color={plan.color}></span>
          <span class="names">
            <span>{plan.info.name}</span>
            <span class="subtitle">
              {plan.original ? t.copyOf(plan.original.info.name, plan.original.subtitle) : plan.subtitle}
            </span>
          </span>
        </label>
        <button
          class="icon-button"
          aria-label={t.change(plan.info.name)}
          onclick={() => (app.details = { kind: 'plan', plan })}>✎</button
        >
      </li>
    {/each}
  </ul>
{/if}
<button class="add" onclick={() => app.draftNewPlan()}>{t.newPlan}</button>

<PlanPreview {app} {preview} />

<style>
  .empty {
    margin: 0 0 10px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  ul {
    margin: 0 0 10px;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 3px 4px;
    border-radius: 6px;
    transition: background var(--quick);
  }
  li:hover {
    background: var(--raised);
  }
  label {
    flex: 1;
    display: flex;
    align-items: start;
    gap: 6px;
    min-width: 0;
    cursor: pointer;
  }
  input {
    margin: 4px 0 0;
  }
  /* Hollow, like the dotted lines of your plans in the charts. */
  .ring {
    flex: none;
    width: 7px;
    height: 7px;
    margin-top: 6px;
    border: 2px solid var(--plan-color);
    border-radius: 50%;
  }
  .names {
    display: grid;
    min-width: 0;
  }
  .subtitle {
    color: var(--weak);
    font-size: 0.8rem;
  }
</style>
