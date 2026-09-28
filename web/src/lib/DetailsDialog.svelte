<script lang="ts">
  import type { AppState, Details } from './app.svelte'
  import Caveats from './Caveats.svelte'
  import FeesForInputs from './FeesForInputs.svelte'
  import PlanEditor from './PlanEditor.svelte'

  let { app }: { app: AppState } = $props()

  let dialog: HTMLDialogElement
  // What the dialog shows; kept after closing, while it fades out.
  let shown = $state.raw<Details | null>(null)

  $effect(() => {
    if (app.details) {
      shown = app.details
      if (!dialog.open) dialog.showModal()
    } else if (dialog.open) {
      dialog.close()
    }
  })

  const broker = $derived(
    shown?.kind === 'broker' ? shown.broker : shown?.kind === 'plan' ? shown.plan.broker : undefined,
  )
  const brokerPlans = $derived(app.listedPlans.filter((plan) => plan.broker === broker))
  /** One of the user's own plans, as it is now: it changes while it's open. */
  const yours = $derived(
    shown?.kind === 'plan' && shown.plan.yours ? app.plansById.get(shown.plan.id)?.yours : undefined,
  )
</script>

<!-- Clicking outside the dialog (on the backdrop) closes it; Esc too. -->
<dialog
  bind:this={dialog}
  aria-labelledby="details-title"
  onclose={() => (app.details = null)}
  onclick={(event) => {
    if (event.target === dialog) dialog.close()
  }}
>
  {#if shown?.kind === 'draft'}
    <div class="content">
      <PlanEditor
        {app}
        yours={shown.draft}
        draft
        onchange={(draft) => (app.details = { kind: 'draft', draft })}
        close={() => dialog.close()}
      />
    </div>
  {:else if yours}
    <div class="content">
      <PlanEditor
        {app}
        {yours}
        draft={false}
        onchange={(changed) => app.updateYourPlan(changed)}
        close={() => dialog.close()}
      />
    </div>
  {:else if shown && broker}
    <div class="content">
      <header>
        <h2 id="details-title">
          {#if shown.kind === 'plan'}
            <span class="mark" style:background={shown.plan.color}></span>{shown.plan.info.name}
          {:else}
            {broker.name}
          {/if}
        </h2>
        <button class="close" aria-label="Close" onclick={() => dialog.close()}>✕</button>
      </header>
      <p class="source">
        {#if shown.kind === 'plan'}
          <button class="link" onclick={() => (app.details = { kind: 'broker', broker })}
            >{broker.name}</button
          > ·
        {/if}
        {broker.tariffDate}
        {#if broker.sourceUrl}
          · <a href={broker.sourceUrl} target="_blank" rel="noreferrer">Tariff (PDF) ↗</a>
        {/if}
      </p>

      {#if shown.kind === 'plan'}
        {@const plan = shown.plan}
        <p>{plan.info.description}</p>
        <section class="fees">
          <FeesForInputs {app} {plan} />
        </section>
        <p class="change">
          <button onclick={() => app.draftCopyOf(plan)}>✎ Change these fees</button>
        </p>
        {@const caveats = app.feesFor(plan).caveats}
        {@const otherCaveats = [...broker.caveats, ...plan.info.caveats].filter(
          ({ text }) => !caveats.includes(text),
        )}
        {#if caveats.length > 0}
          <h3>Caveats for {app.purchase}</h3>
          <Caveats notes={caveats} />
        {/if}
        <details>
          <summary>
            Full tariff{#if otherCaveats.length > 0}<span class="more">
                , and {otherCaveats.length} caveat{otherCaveats.length === 1 ? '' : 's'} about other choices</span
              >{/if}
          </summary>
          <div class="tariff">
            <h4>Buying and selling</h4>
            <table>
              <tbody>
                {#each plan.info.tariff.trading as { covers, price }, index (index)}
                  <tr><td>{covers}</td><td>{price}</td></tr>
                {/each}
              </tbody>
            </table>
            <h4>Custody</h4>
            <table>
              <tbody>
                {#each plan.info.tariff.custody as { covers, price }, index (index)}
                  <tr><td>{covers}</td><td>{price}</td></tr>
                {/each}
              </tbody>
            </table>
            <h4>Conversion</h4>
            <table>
              <tbody>
                <tr><td>Fee</td><td>{plan.info.tariff.conversion}</td></tr>
                <tr><td>Markup</td><td>{plan.info.tariff.markup}</td></tr>
              </tbody>
            </table>
            {#if otherCaveats.length > 0}
              <h4>Caveats about other choices</h4>
              <Caveats notes={otherCaveats} />
            {/if}
          </div>
        </details>
      {:else}
        <p>{broker.description}</p>
        {@const caveats = app.brokerCaveats(broker)}
        {#if caveats.length > 0}
          <h3>Caveats for {app.purchase}</h3>
          <Caveats notes={caveats} />
        {/if}
        <h3>Plans</h3>
        <ul class="plans">
          {#each brokerPlans as plan (plan.id)}
            <li>
              <button onclick={() => (app.details = { kind: 'plan', plan })}>
                <span class="plan-name"
                  ><span class="mark" style:background={plan.color}></span>{plan.info.name}</span
                >
                <span class="plan-description">{plan.info.description}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</dialog>

<style>
  dialog {
    width: min(620px, calc(100vw - 32px));
    max-height: min(85dvh, 900px);
    padding: 0;
    border: 1px solid var(--strong-border);
    border-radius: 16px;
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
    /* Fades and rises in, and back out; the backdrop blurs the page. */
    opacity: 0;
    translate: 0 12px;
    scale: 0.98;
    transition:
      opacity 200ms ease-out,
      translate 200ms ease-out,
      scale 200ms ease-out,
      overlay 200ms allow-discrete,
      display 200ms allow-discrete;
  }
  dialog[open] {
    opacity: 1;
    translate: 0;
    scale: 1;
  }
  @starting-style {
    dialog[open] {
      opacity: 0;
      translate: 0 12px;
      scale: 0.98;
    }
  }
  dialog::backdrop {
    background: rgb(0 0 0 / 0);
    backdrop-filter: blur(0);
    transition:
      background 200ms,
      backdrop-filter 200ms,
      overlay 200ms allow-discrete,
      display 200ms allow-discrete;
  }
  dialog[open]::backdrop {
    background: rgb(0 0 0 / 0.55);
    backdrop-filter: blur(3px);
  }
  @starting-style {
    dialog[open]::backdrop {
      background: rgb(0 0 0 / 0);
      backdrop-filter: blur(0);
    }
  }

  .content {
    padding: 20px 24px 24px;
  }
  /* The title stays while the rest scrolls. */
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    margin: -20px -24px 0;
    padding: 20px 24px 4px;
    background: var(--surface);
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 1.3rem;
  }
  h3 {
    margin: 20px 0 8px;
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--weak);
  }
  h4 {
    margin: 14px 0 6px;
    font-size: 0.9rem;
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
    margin: 4px 0 16px;
    color: var(--weak);
    font-size: 0.9rem;
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
  .fees {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--raised);
  }
  .change {
    margin: 10px 0 0;
  }
  .mark {
    display: inline-block;
    width: 12px;
    height: 12px;
    margin-right: 8px;
    border-radius: 3px;
  }

  details {
    margin-top: 20px;
    border-top: 1px solid var(--border);
    padding-top: 12px;
    /* Lets the content's height animate to its natural size (Chrome). */
    interpolate-size: allow-keywords;
  }
  details::details-content {
    block-size: 0;
    overflow: hidden;
    transition:
      block-size 250ms ease-out,
      content-visibility 250ms allow-discrete;
  }
  details[open]::details-content {
    block-size: auto;
  }
  .more {
    color: var(--weak);
    font-weight: normal;
  }
  summary {
    cursor: pointer;
    font-weight: 600;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }
  td {
    padding: 5px 8px;
    vertical-align: top;
  }
  tr:nth-child(odd) {
    background: var(--raised);
  }
  td:first-child {
    color: var(--weak);
  }

  .plans {
    display: grid;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .plans button {
    display: grid;
    gap: 2px;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
    border-radius: 10px;
  }
  .plan-name {
    font-weight: 600;
  }
  .plan-description {
    color: var(--weak);
    font-size: 0.9rem;
  }
</style>
