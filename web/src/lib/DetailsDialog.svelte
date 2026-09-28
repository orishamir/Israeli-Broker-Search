<script lang="ts">
  import { untrack } from 'svelte'
  import { tick } from 'svelte'
  import type { AppState, Details } from './app.svelte'
  import CaveatGroups from './CaveatGroups.svelte'
  import Caveats from './Caveats.svelte'
  import FeesForInputs from './FeesForInputs.svelte'
  import Price from './Price.svelte'
  import PlanEditor from './PlanEditor.svelte'

  let { app }: { app: AppState } = $props()

  let dialog: HTMLDialogElement
  // What the dialog shows; kept after closing, while it fades out.
  let shown = $state.raw<Details | null>(null)

  /** What the dialog shows, whatever changes in it: a draft changes as it's
   * edited. */
  const subject = (details: Details) =>
    details.kind === 'draft'
      ? details.draft.id
      : details.kind === 'plan'
        ? details.plan.id
        : details.kind === 'broker'
          ? details.broker.name
          : 'about'

  $effect(() => {
    if (app.details) {
      // Something else starts at the top; the dialog would keep the last one's
      // scroll.
      const last = untrack(() => shown)
      if (!dialog.open || !last || subject(last) !== subject(app.details)) dialog.scrollTop = 0
      shown = app.details
      if (!dialog.open) dialog.showModal()
      // The about page opens at the section that was asked for, once drawn.
      if (app.details.kind === 'about' && app.details.section !== undefined) {
        const section = app.details.section
        tick().then(() => dialog.querySelector(`#about-${section}`)?.scrollIntoView({ block: 'start' }))
      }
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
  {:else if shown?.kind === 'about'}
    <div class="content">
      <header>
        <h2 id="details-title">About the numbers</h2>
        <button class="close" aria-label="Close" onclick={() => dialog.close()}>✕</button>
      </header>
      {#each app.about.sections as section, index (section.title)}
        <section class="about" id="about-{index}">
          <h3>{section.title}</h3>
          {#each section.paragraphs as paragraph (paragraph)}
            <p>{paragraph}</p>
          {/each}
          {#if section.items.length > 0}
            <ul>
              {#each section.items as item (item.text)}
                <li>
                  {#if item.url}
                    <a href={item.url} target="_blank" rel="noreferrer">{item.text} ↗</a>
                  {:else}
                    {item.text}
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </section>
      {/each}
    </div>
  {:else if shown && broker}
    <!-- Each plan or broker starts afresh, its prices folded. -->
    {#key shown.kind === 'plan' ? shown.plan.id : broker.name}
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
          {broker.tariffDate} · {broker.checked}
          {#if broker.sourceUrl}
            · <a href={broker.sourceUrl} target="_blank" rel="noreferrer">Tariff (PDF) ↗</a>
          {/if}
        </p>

        {#if shown.kind === 'plan'}
          {@const plan = shown.plan}
          {@const tariff = plan.info.tariff}
          <p>{plan.info.description}</p>
          <section class="fees">
            <FeesForInputs {app} {plan} />
          </section>
          <p class="change">
            <button onclick={() => app.draftCopyOf(plan)}>✎ Change these fees</button>
          </p>
          {@const { caveats, others } = app.feesFor(plan)}
          {#if caveats.length > 0}
            <h3>Caveats for {app.purchase}</h3>
            <CaveatGroups groups={caveats} />
          {/if}
          <details>
            <summary>
              All prices{#if others.length > 0}<span class="more">
                  , and {others.length} caveat{others.length === 1 ? '' : 's'} about other choices or amounts</span
                >{/if}
            </summary>
            <div class="tariff">
              <h4>Buying and selling</h4>
              <table>
                <tbody>
                  {#each tariff.trading as { covers, price }, index (index)}
                    <tr><td>{covers}</td><td><Price {price} /></td></tr>
                  {/each}
                  {#if tariff.fractionsOn.length > 0}
                    <tr><td>Fractions of a share</td><td>sold on {tariff.fractionsOn.join(', ')}</td></tr>
                  {/if}
                </tbody>
              </table>
              {#if tariff.tracks.length > 0}
                <!-- Usually every track prices the same thing: then it's said once. -->
                {@const covered = new Set(
                  tariff.tracks.flatMap((track) => track.trading.map(({ covers }) => covers)),
                )}
                <h4>Tracks{covered.size === 1 ? ` for ${[...covered][0]}` : ''}: you choose one</h4>
                <table>
                  <tbody>
                    {#each tariff.tracks as track (track.name)}
                      {#each track.trading as { covers, price }, index (index)}
                        <tr>
                          <td
                            >{track.name}{#if covered.size > 1}: {covers}{/if}</td
                          >
                          <td><Price {price} /></td>
                        </tr>
                      {/each}
                    {/each}
                  </tbody>
                </table>
              {/if}
              {#if tariff.standingOrders.length > 0}
                <h4>Buying by standing order</h4>
                <table>
                  <tbody>
                    {#each tariff.standingOrders as { covers, price }, index (index)}
                      <tr><td>{covers}</td><td><Price {price} /></td></tr>
                    {/each}
                  </tbody>
                </table>
              {/if}
              <h4>Custody</h4>
              <table>
                <tbody>
                  {#each tariff.custody as { covers, price }, index (index)}
                    <tr><td>{covers}</td><td><Price {price} /></td></tr>
                  {:else}
                    <tr><td>Everything</td><td><Price price={{ text: 'none', nothing: true }} /></td></tr>
                  {/each}
                </tbody>
              </table>
              {#if tariff.handling}
                <h4>Handling fee</h4>
                <table>
                  <tbody>
                    <tr><td>The account</td><td><Price price={tariff.handling} /></td></tr>
                  </tbody>
                </table>
              {/if}
              <h4>Conversion</h4>
              <table>
                <tbody>
                  <tr><td>Fee</td><td><Price price={tariff.conversion} /></td></tr>
                  {#if tariff.secondConversion}
                    <tr><td>Or, if less</td><td><Price price={tariff.secondConversion} /></td></tr>
                  {/if}
                  {#if tariff.standingOrderConversion}
                    <tr
                      ><td>By standing order</td><td><Price price={tariff.standingOrderConversion} /></td></tr
                    >
                  {/if}
                  <tr><td>Markup</td><td><Price price={tariff.markup} /></td></tr>
                </tbody>
              </table>
              {#if others.length > 0}
                <h4>Caveats about other choices or amounts</h4>
                <Caveats caveats={others} />
              {/if}
            </div>
          </details>
          <p class="about-link">
            <button class="link" onclick={() => (app.details = { kind: 'about' })}
              >How the numbers are made, and what isn't counted ↗</button
            >
          </p>
        {:else}
          <p>{broker.description}</p>
          {@const caveats = app.brokerCaveats(broker)}
          {#if caveats.length > 0}
            <h3>Caveats for {app.purchase}</h3>
            <CaveatGroups groups={caveats} />
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
    {/key}
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
  .about-link {
    margin: 16px 0 0;
    font-size: 0.9rem;
  }
  /* Scrolled to from the title's links: its heading stays under the sticky
     header otherwise. */
  .about {
    scroll-margin-top: 64px;
  }
  .about p {
    margin: 8px 0;
  }
  .about ul {
    margin: 8px 0;
    padding-left: 20px;
  }
  .about li {
    margin: 4px 0;
  }
  .about a {
    color: var(--accent);
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
