<script lang="ts">
  import { planId, type AppState, type EditorView, type YourPlan } from './app.svelte'
  import Choices from './Choices.svelte'
  import * as core from './core/core'
  import type { Choice, PlanData } from './core/core'
  import type { Change, Original } from './editor'
  import PriceList from './PriceList.svelte'
  import SimpleFees from './SimpleFees.svelte'
  import { t } from './text'
  import { en } from './text/en'

  /** One of the user's own plans, in the details dialog. A draft (not added
   * yet) has Cancel and Add plan; a plan already added saves as it changes. */
  let {
    app,
    yours,
    draft,
    onchange,
    close,
  }: {
    app: AppState
    yours: YourPlan
    draft: boolean
    onchange: (yours: YourPlan) => void
    close: () => void
  } = $props()

  const views: Choice<EditorView>[] = [
    { value: 'simple', name: t.simple, englishName: en.simple, explanation: t.simpleTip, hebrewNames: [] },
    {
      value: 'full',
      name: t.fullPriceList,
      englishName: en.fullPriceList,
      explanation: t.fullPriceListTip,
      hebrewNames: [],
    },
  ]

  const info = $derived(core.planInfo(yours.plan))
  /** A fund's or a policy's plan: its manager's fee is all there is to change. */
  const managed = $derived(info.tariff.management !== undefined)
  const originalPlan = $derived(app.originalOf(yours))
  const original: Original | undefined = $derived(
    originalPlan?.key.kind === 'listed'
      ? {
          name: originalPlan.info.name,
          data: core.listedPlan(
            originalPlan.key.broker,
            originalPlan.key.plan,
            app.originalTrack(yours, originalPlan),
          ),
        }
      : undefined,
  )
  /** A draft isn't in the app's plans yet: it gets its color when added. */
  const color = $derived(
    app.plansById.get(planId({ kind: 'yours', id: yours.id }))?.color ?? originalPlan?.color ?? 'var(--weak)',
  )

  /** Why a field couldn't be saved, by field. */
  let errors = $state<Record<string, string>>({})
  const change: Change = (field, update) => {
    let plan: PlanData
    try {
      plan = update(yours.plan)
    } catch (error) {
      errors[field] = error instanceof Error ? error.message : String(error)
      return
    }
    delete errors[field]
    onchange({ ...yours, plan })
  }

  let confirmingDelete = $state(false)
</script>

<header>
  <!-- The dialog's name; the field shows it. -->
  <h2 id="details-title" class="visually-hidden">{info.name}</h2>
  <span class="ring" style:--plan-color={color}></span>
  <input
    class="name"
    type="text"
    aria-label={t.planName}
    autocomplete="off"
    bind:value={() => info.name, (name) => change('name', (plan) => core.rename(plan, name))}
  />
  <button class="close" aria-label={t.close} onclick={close}>✕</button>
</header>
{#if errors.name}<p class="error">{errors.name}</p>{/if}

<p class="source">
  {#if originalPlan}
    {t.copyOfWord}
    {#if draft}
      {originalPlan.info.name}
    {:else}
      <button class="link" onclick={() => (app.details = { kind: 'plan', plan: originalPlan })}
        >{originalPlan.info.name}</button
      >
    {/if}&nbsp;· {originalPlan.subtitle}
  {:else}
    <label class="broker">
      {t.broker}
      <input
        type="text"
        placeholder={t.optional}
        autocomplete="off"
        value={yours.brokerName ?? ''}
        oninput={(event) => onchange({ ...yours, brokerName: event.currentTarget.value || null })}
      />
    </label>
  {/if}
</p>

<!-- A fund's fee is all it charges: there's no price list to open. -->
{#if !managed}
  <div class="view">
    <Choices label={t.view} options={views} bind:value={app.editorView} />
  </div>
{/if}

{#if managed || app.editorView === 'simple'}
  <div class="fees">
    <SimpleFees {app} {yours} {original} {change} {errors} />
  </div>
{:else}
  <PriceList {app} {yours} {original} {change} {errors} />
{/if}

<footer>
  {#if draft}
    <button onclick={close}>{t.cancel}</button>
    <button
      class="primary"
      onclick={() => {
        app.addYourPlan(yours)
        close()
      }}>{t.addPlan}</button
    >
  {:else if confirmingDelete}
    <span class="confirm">{t.deleteQuestion(info.name)}</span>
    <button
      class="danger"
      onclick={() => {
        app.deleteYourPlan(yours.id)
        close()
      }}>{t.delete}</button
    >
    <button onclick={() => (confirmingDelete = false)}>{t.keepPlan}</button>
  {:else}
    <button onclick={() => (confirmingDelete = true)}>{t.deletePlan}</button>
    <button class="primary" onclick={close}>{t.done}</button>
  {/if}
</footer>

<style>
  /* The name stays while the rest scrolls, as in the other dialogs. */
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    margin: -20px -24px 0;
    padding: 20px 24px 8px;
    background: var(--surface);
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .ring {
    flex: none;
    width: 10px;
    height: 10px;
    border: 2px solid var(--plan-color);
    border-radius: 50%;
  }
  .name {
    flex: 1;
    font-weight: 600;
  }
  .close {
    border: none;
    background: transparent;
    color: var(--weak);
    padding: 2px 8px;
  }
  .close:hover {
    color: var(--text);
    background: var(--raised);
  }
  .source {
    margin: 4px 0 12px;
    color: var(--weak);
    font-size: 0.9rem;
  }
  .broker {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .broker input {
    flex: 1;
    max-width: 16rem;
    color: var(--text);
  }
  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
  }
  .link:hover {
    background: none;
    text-decoration: underline;
  }
  .view {
    margin-bottom: 14px;
  }
  .fees {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--raised);
  }
  .fees :global(.field),
  .fees :global(select) {
    background: var(--surface);
  }
  .error {
    margin: 4px 0 0;
    color: var(--error);
    font-size: 0.85rem;
  }
  footer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  /* Delete goes on the left, away from the main button. */
  footer > :first-child:not(.primary) {
    margin-inline-end: auto;
  }
  .confirm {
    margin-inline-end: auto;
  }
  .primary {
    border-color: var(--accent);
    color: var(--accent);
  }
  .danger {
    border-color: var(--error);
    color: var(--error);
  }
</style>
