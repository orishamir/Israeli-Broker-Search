<script lang="ts">
  import type { AppState } from './app.svelte'
  import type { Coverage, Exchange, Security } from './core/core'
  import { tip } from './tip'

  /** What a row of the price list covers, as a button that opens a
   * checklist. Ticks apply when the checklist closes: the rows sort
   * themselves by what they cover, so a row applied on each tick could move
   * away from its open checklist. `securities` is left out for custody,
   * which only depends on the exchange. */
  let {
    app,
    covers,
    coverage,
    onchange,
  }: {
    app: AppState
    covers: string
    coverage: { securities?: Security[]; exchanges: Exchange[] }
    onchange: (coverage: Coverage) => void
  } = $props()

  let popover = $state<HTMLElement>()
  /** Ticks not applied yet. */
  let edits = $state<{ securities: Security[]; exchanges: Exchange[] } | null>(null)
  const shown = $derived(edits ?? { securities: coverage.securities ?? [], exchanges: coverage.exchanges })
  const id = $props.id()

  function toggle<T>(list: T[], item: T, on: boolean): T[] {
    return on ? [...list, item] : list.filter((each) => each !== item)
  }

  function apply() {
    if (edits) onchange(edits)
    edits = null
  }
</script>

<button
  type="button"
  class="coverage"
  aria-haspopup="true"
  {@attach popover && tip(popover, { hover: false, below: true, onClose: apply })}>{covers} ▾</button
>
<div class="popover checklist" popover="manual" role="group" aria-label="What it covers" bind:this={popover}>
  {#if coverage.securities}
    <p class="heading" id="{id}-securities">Securities</p>
    <div class="options" role="group" aria-labelledby="{id}-securities">
      {#each app.securities as security (security.value)}
        <label>
          <input
            type="checkbox"
            checked={shown.securities.includes(security.value)}
            onchange={(event) =>
              (edits = {
                ...shown,
                securities: toggle(shown.securities, security.value, event.currentTarget.checked),
              })}
          />
          {security.name}
        </label>
      {/each}
    </div>
  {/if}
  <p class="heading" id="{id}-exchanges">Exchanges</p>
  <div class="options" role="group" aria-labelledby="{id}-exchanges">
    {#each app.exchanges as exchange (exchange.value)}
      <label>
        <input
          type="checkbox"
          checked={shown.exchanges.includes(exchange.value)}
          onchange={(event) =>
            (edits = {
              ...shown,
              exchanges: toggle(shown.exchanges, exchange.value, event.currentTarget.checked),
            })}
        />
        {exchange.name}
      </label>
    {/each}
  </div>
  <p class="note">None ticked means all.</p>
</div>

<style>
  .coverage {
    max-width: 100%;
    text-align: left;
    font-weight: 600;
  }
  .heading {
    color: var(--weak);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .options {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px 12px;
  }
  label {
    cursor: pointer;
  }
  .note {
    color: var(--weak);
  }
</style>
