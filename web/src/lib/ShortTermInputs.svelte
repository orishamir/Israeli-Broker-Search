<script lang="ts">
  import type { AppState } from './app.svelte'
  import Examples from './Examples.svelte'
  import { compactPercent, percent } from './format'
  import NumberField from './NumberField.svelte'
  import PlaceSheet from './PlaceSheet.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** The short-term calculator's inputs: how much, for how long, the rate
   * expected, and what's compared. */
  let { app }: { app: AppState } = $props()

  const short = $derived(app.short)
  /** Whether the list of places to compare is open. */
  let choosing = $state(false)
</script>

<section class="card">
  <h3>{t.tryAnExample}</h3>
  <Examples examples={short.examples} apply={(example) => short.applyExample(example)} />
</section>

<section class="card">
  <h3>{t.deposits}</h3>
  <div class="fields">
    <label for="first-deposit">{t.oneTimeDeposit}</label>
    <NumberField id="first-deposit" prefix="₪" step={1000} bind:value={short.firstDeposit} />

    <label for="monthly-deposit">{t.everyMonth}</label>
    <NumberField id="monthly-deposit" prefix="₪" step={100} bind:value={short.monthlyDeposit} />
  </div>
</section>

<section class="card">
  <h3><label for="months">{t.forHowLong}</label></h3>
  <div class="range">
    <input id="months" type="range" min="1" max={short.longest} bind:value={short.months} />
    <output for="months">{t.monthsCount(short.months)}</output>
  </div>
</section>

<section class="card">
  <h3>{t.expectations}</h3>
  <div class="fields">
    <span>
      <label for="rate">{t.boiRate}</label><Tip about={t.boiRate}>{t.boiRateTip}</Tip>
    </span>
    <NumberField id="rate" suffix="%" step={0.25} bind:value={short.ratePercent} />
  </div>
  <p class="note">{t.boiRateNote(percent(short.todaysRate))}</p>
</section>

<!-- What's compared, in short; the list itself opens in a sheet. -->
<section class="card">
  <div class="heading">
    <h3>{t.whatsCompared}</h3>
    <Tip about={t.whatsCompared}><p>{t.shortWhatsComparedTip}</p></Tip>
  </div>
  <p class="note">{t.shortTickedAtFirst}</p>
  {#if short.compared.length > 0}
    <ul class="compared" aria-label={t.whatsCompared}>
      {#each short.compared as place (place.id)}
        <li>
          <span class="mark" class:yours={place.yours} style:--plan-color={place.color}></span>{place.label}
        </li>
      {/each}
    </ul>
  {/if}
  <div class="choose">
    <button onclick={() => (choosing = true)}>{t.addOrRemove}</button>
    <span class="count">{t.countOf(short.compared.length, short.places.length)}</span>
  </div>
</section>
<PlaceSheet {app} bind:open={choosing} />

<!-- Last, as in the long term: one switch for both calculators. -->
<section class="card">
  <div class="heading">
    <h3>
      <label class="switch">
        <input type="checkbox" bind:checked={app.moreOptions} />
        {t.moreOptions}
      </label>
    </h3>
    <Tip about={t.moreOptions}>{t.shortMoreOptionsTip(compactPercent(short.usualInflation))}</Tip>
  </div>
  {#if app.moreOptions}
    <div class="fields">
      <span>
        <label for="inflation">{t.inflation}</label><Tip about={t.inflation}>{t.shortInflationTip}</Tip>
      </span>
      <NumberField id="inflation" suffix={t.percentAYear} step={0.5} bind:value={short.inflationPercent} />
    </div>
  {/if}
</section>

<style>
  /* Not only the next one: the list's dialog sits before the last card. */
  section ~ section {
    margin-top: 12px;
  }
  h3 {
    margin: 0 0 10px;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--weak);
  }
  /* A heading and its ?, which isn't part of the heading's name. */
  .heading {
    display: flex;
    align-items: center;
    margin-bottom: 10px;
  }
  .heading h3 {
    margin: 0;
  }
  /* More options while it's off: a heading alone in its card. */
  .heading:last-child {
    margin-bottom: 0;
  }
  /* A card whose heading is its switch: as small as the other headings, and
     tall enough for a finger. */
  .switch {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 24px;
    cursor: pointer;
    user-select: none;
  }
  .fields {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px 12px;
    align-items: center;
  }
  .fields > label,
  .fields > span {
    white-space: nowrap;
  }
  .note {
    margin: 10px 0 0;
    font-size: 0.8rem;
    line-height: 1.5;
    color: var(--weak);
  }
  .heading + .note {
    margin: 0;
  }
  .range {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .range input {
    flex: 1;
    min-width: 0;
  }
  output {
    min-width: 6ch;
    text-align: end;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    white-space: nowrap;
  }
  /* The places compared, each with its color as in the table. */
  .compared {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin: 10px 0 0;
    padding: 0;
    list-style: none;
  }
  .compared li {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--raised);
    font-size: 0.8rem;
  }
  .mark {
    flex: none;
    box-sizing: border-box;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--plan-color);
  }
  /* Your own deposits are hollow, like their dotted lines in the chart. */
  .mark.yours {
    border: 2px solid var(--plan-color);
    background: none;
  }
  .choose {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 10px;
    margin-top: 12px;
  }
  .count {
    color: var(--weak);
    font-size: 0.8rem;
  }
</style>
