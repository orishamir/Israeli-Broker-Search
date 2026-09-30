<script lang="ts">
  import { tick } from 'svelte'
  import type { AppState } from './app.svelte'
  import NumberField from './NumberField.svelte'
  import Sheet from './Sheet.svelte'
  import type { Place } from './short-term.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** Every place there is to keep the money, to tick: the funds, the banks'
   * deposits, and deposits of your own at a rate you were offered. */
  let { app, open = $bindable(false) }: { app: AppState; open: boolean } = $props()

  const short = $derived(app.short)
  const byKind = $derived(
    short.kinds.map((kind, group) => ({
      kind,
      places: short.listedPlaces.filter(({ key }) => key.kind === 'listed' && key.group === group),
      deposits: kind.places.some(({ rates }) => rates.length > 0),
    })),
  )
  const yours = $derived(short.places.filter((place) => place.yours))
  /** The deposits' term for the months chosen, as the Bank of Israel names it. */
  const term = $derived.by(() => {
    const deposit = short.listedPlaces.find(({ info }) => info.rates.length > 0)
    return short.term === undefined ? undefined : deposit?.info.rates[short.term]?.term
  })

  const details = (place: Place) => (app.details = { kind: 'place', place })

  /** Adds a deposit of your own and puts the caret in its rate. */
  async function add() {
    const deposit = short.addYourDeposit()
    await tick()
    document.getElementById(`deposit-rate-${deposit.id}`)?.focus()
  }
</script>

<Sheet
  bind:open
  title={t.whatsCompared}
  done={t.doneComparing(short.compared.length)}
  onclose={() => (short.hovered = null)}
>
  {#each byKind as { kind, places, deposits } (kind.name)}
    <section>
      <div class="heading">
        <h3>{kind.name}</h3>
        <Tip about={kind.name}><p>{kind.description}</p></Tip>
      </div>
      <p class="date">
        <bdi>{kind.dataOf}</bdi>{#if deposits && term}
          · <bdi>{t.ratesFor(term)}</bdi>{/if}
      </p>
      <!-- A flag the whole kind shares, once; a place's own is on its line. -->
      {#if kind.mayCostMore}<p class="note">⚠ {kind.mayCostMore}</p>{/if}
      {#if deposits && (short.monthlyDeposit ?? 0) > 0}
        <!-- Said where the deposits are ticked: none of them takes it. -->
        <p class="note">{t.monthlyNotForDeposits}</p>
      {/if}
      <ul aria-label={kind.name}>
        {#each places as place (place.id)}
          <li
            class="place"
            onpointerenter={(event) => {
              // Also stands out in the table and the chart. Mouse only: a tap
              // would leave it standing out, with nothing to end the hover.
              if (event.pointerType === 'mouse') short.hovered = place.id
            }}
            onpointerleave={(event) => {
              if (event.pointerType === 'mouse') short.hovered = null
            }}
          >
            <label>
              <input
                type="checkbox"
                style:accent-color={place.color}
                checked={short.selected.has(place.id)}
                onchange={(event) => short.setSelected([place.id], event.currentTarget.checked)}
              />
              <span class="name">{place.info.name}</span>
            </label>
            <span class="pays">{short.paysFor(place)}</span>
            <!-- ⚠ means one thing everywhere: a number that may be too good. -->
            {#if place.info.mayCostMore}
              <button
                class="icon-button warning"
                aria-label={t.mayLeaveLess(place.info.name)}
                onclick={() => details(place)}>⚠</button
              >
            {/if}
            <button class="icon-button" aria-label={t.about(place.info.name)} onclick={() => details(place)}
              >ℹ</button
            >
          </li>
        {/each}
      </ul>
    </section>
  {/each}
  <section>
    <div class="heading">
      <h3>{t.yourDeposits}</h3>
      <Tip about={t.yourDeposits}><p>{t.yourDepositsTip}</p></Tip>
    </div>
    {#if yours.length > 0}
      <ul aria-label={t.yourDeposits}>
        {#each yours as place (place.id)}
          {@const deposit = place.yours!}
          <li class="yours">
            <input
              type="checkbox"
              aria-label={deposit.name}
              style:accent-color={place.color}
              checked={short.selected.has(place.id)}
              onchange={(event) => short.setSelected([place.id], event.currentTarget.checked)}
            />
            <input
              class="deposit-name"
              type="text"
              aria-label={t.depositName}
              value={deposit.name}
              oninput={(event) => short.updateYourDeposit({ ...deposit, name: event.currentTarget.value })}
            />
            <span class="rate">
              <NumberField
                id="deposit-rate-{deposit.id}"
                label={t.depositRate}
                suffix="%"
                step={0.05}
                bind:value={
                  () => deposit.ratePercent,
                  (ratePercent) => short.updateYourDeposit({ ...deposit, ratePercent })
                }
              />
            </span>
            <button
              class="icon-button"
              aria-label={t.deleteDeposit(deposit.name)}
              onclick={() => short.deleteYourDeposit(deposit.id)}>✕</button
            >
          </li>
        {/each}
      </ul>
    {/if}
    <button class="add" onclick={add}>{t.addYourDeposit}</button>
  </section>
</Sheet>

<style>
  section + section {
    margin-top: 20px;
  }
  .heading {
    display: flex;
    align-items: center;
    margin-bottom: 4px;
  }
  h3 {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--weak);
  }
  .date,
  .note {
    margin: 0 0 6px;
    font-size: 0.75rem;
    color: var(--weak);
  }
  .note {
    color: var(--warning);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .place {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    border-radius: 6px;
    transition: background var(--quick);
  }
  .place:hover {
    background: var(--raised);
  }
  .place label {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    cursor: pointer;
  }
  .pays {
    color: var(--weak);
    font-size: 0.8rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  /* A place's ⚠ stays; its ℹ shows on hover, and always on touch screens. */
  @media (hover: hover) {
    .place .icon-button:not(.warning) {
      opacity: 0;
      transition: opacity var(--quick);
    }
    .place:hover .icon-button,
    .place .icon-button:focus-visible {
      opacity: 1;
    }
  }
  .yours {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) 7rem auto;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
  }
  .deposit-name {
    min-width: 0;
  }
  .add {
    margin-top: 8px;
  }
</style>
