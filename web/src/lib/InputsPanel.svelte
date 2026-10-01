<script lang="ts">
  import type { AppState, AtEnd, WayOut } from './app.svelte'
  import Choices from './Choices.svelte'
  import type { Choice } from './core/core'
  import Examples from './Examples.svelte'
  import HebrewNames from './HebrewNames.svelte'
  import { compactPercent, dateText } from './format'
  import { BUYING_INTERVALS } from './link'
  import NumberField from './NumberField.svelte'
  import PlanSheet from './PlanSheet.svelte'
  import { t } from './text'
  import { en } from './text/en'
  import { reveal } from './motion'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()

  /** Whether the list of plans to compare is open. */
  let choosing = $state(false)

  const atEndChoices: Choice<AtEnd>[] = [
    { value: 'sell', name: t.sell, englishName: en.sell, explanation: t.sellTip, hebrewNames: [] },
    { value: 'hold', name: t.keep, englishName: en.keep, explanation: t.keepTip, hebrewNames: [] },
  ]

  /** How the money is taken out, for the age a ticked plan's pension opens at. */
  const wayOutChoices = (pensionFromAge: number): Choice<WayOut>[] => [
    {
      value: 'atOnce',
      name: t.allAtOnce,
      englishName: en.allAtOnce,
      explanation: t.allAtOnceTip,
      hebrewNames: [],
    },
    {
      value: 'pension',
      name: t.asAPension,
      englishName: en.asAPension,
      explanation: t.asAPensionTip(pensionFromAge),
      hebrewNames: ['קצבה'],
    },
  ]

  const security = $derived(app.securities.find(({ value }) => value === app.security)!)
  const exchange = $derived(app.exchanges.find(({ value }) => value === app.exchange)!)
</script>

<section class="card">
  <h3>{t.tryAnExample}</h3>
  <Examples examples={app.examples} apply={(example) => app.applyExample(example)} />
</section>

<section class="card">
  <div class="heading">
    <h3>{t.whatYouBuy}</h3>
    <Tip about={t.whatYouBuy}>
      <p>
        קרנות סל, קרנות מחקות, אג״ח ומניות הן כולן ניירות ערך, והבנקים ובתי ההשקעות גובים על כל סוג עמלות
        שונות. קרן סל וקרן מחקה יכולות להחזיק בדיוק את אותן חברות: ההבדל הוא איך קונים אותן, ולכן גם כמה
        משלמים בעמלות.
      </p>
      {#each app.securities as option (option.value)}
        <div class="term">
          <strong>{option.name}</strong>
          <HebrewNames names={option.hebrewNames} english={option.englishName} main={option.name} />
          <p class="weak">{option.explanation}</p>
        </div>
      {/each}
    </Tip>
  </div>
  <Choices label={t.security} options={app.securities} bind:value={app.security} />
  <!-- Always shown: the English names are the ones people don't know. -->
  <div class="explained">
    <HebrewNames names={security.hebrewNames} english={security.englishName} main={security.name} />
    <p class="weak">{security.explanation}</p>
  </div>
</section>

<section class="card">
  <div class="heading">
    <h3>{t.tradedOn}</h3>
    <Tip about={t.tradedOn}>
      <p>
        את {security.name} אפשר לקנות בבורסות שונות: בתל אביב, או בחו״ל בדולרים או באירו.
        {#if app.security === 'Bond'}
          אג״ח ממשלת ישראל נסחרות בתל אביב בשקלים, ואג״ח ממשלת ארה״ב בניו יורק בדולרים.
        {:else if app.security === 'Stock'}
          טבע, למשל, נסחרת בתל אביב בשקלים ובניו יורק בדולרים.
        {:else}
          אותו S&P 500, למשל, נמכר בתל אביב כקרן סל בשקלים, ובניו יורק כקרן סל כמו VOO, בדולרים.
        {/if}
        הבנקים ובתי ההשקעות גובים עמלות שונות בכל בורסה, ובחו״ל חלקם גובים גם על המרת השקלים.
      </p>
    </Tip>
  </div>
  <Choices label={t.exchange} options={app.exchanges} bind:value={app.exchange} />
  <div class="explained">
    <HebrewNames names={exchange.hebrewNames} english={exchange.englishName} main={exchange.name} />
    <p class="weak">{exchange.explanation}</p>
  </div>
  {#if app.exchange !== 'Tlv'}
    <!-- Rarely changed, so folded away unless the download failed. -->
    <details class="rates" open={app.ratesStatus.kind === 'failed'} transition:reveal>
      <summary>
        <!-- Left to right even in Hebrew, or the equals signs read backwards. -->
        <bdi dir="ltr">$1 = ₪{app.ilsPerUsd ?? '?'} · €1 = ₪{app.ilsPerEur ?? '?'}</bdi>
        <span class="weak">
          {#if app.ratesStatus.kind === 'downloading'}
            {t.downloadingRates}
          {:else if app.ratesStatus.kind === 'downloaded'}
            · {dateText(app.ratesStatus.date)}
          {:else}
            {t.couldntDownloadRates}
          {/if}
        </span>
      </summary>
      <div class="fields">
        <label for="usd">$1 =</label>
        <NumberField id="usd" prefix="₪" step={0.01} bind:value={app.ilsPerUsd} />
        <label for="eur">€1 =</label>
        <NumberField id="eur" prefix="₪" step={0.01} bind:value={app.ilsPerEur} />
      </div>
      <p class="caption">
        {#if app.ratesStatus.kind === 'failed'}
          {t.couldntDownloadCheck(app.ratesStatus.error)}
        {:else}
          שערי הבנק המרכזי האירופי, שרחוקים מהשער היציג של בנק ישראל בשבריר אחוז לכל היותר. אפשר לשנות אותם
          כדי לנסות שערים אחרים.
        {/if}
      </p>
    </details>
  {/if}
</section>

<section class="card">
  <h3>{t.deposits}</h3>
  <div class="fields">
    <label for="first-deposit">{t.oneTimeDeposit}</label>
    <NumberField id="first-deposit" prefix="₪" step={1000} bind:value={app.firstDeposit} />

    <label for="monthly-deposit">{t.everyMonth}</label>
    <NumberField id="monthly-deposit" prefix="₪" step={100} bind:value={app.monthlyDeposit} />
  </div>
</section>

<section class="card">
  <h3>{t.expectations}</h3>
  <div class="fields">
    <span>
      <label for="yearly-return">{t.yearlyReturn}</label><Tip about={t.yearlyReturn}>{t.yearlyReturnTip}</Tip>
    </span>
    <NumberField id="yearly-return" suffix="%" step={0.5} bind:value={app.yearlyReturnPercent} />

    <label for="years">{t.years}</label>
    <div class="range">
      <input id="years" type="range" min="1" max="50" bind:value={app.years} />
      <output for="years">{app.years}</output>
    </div>

    {#if app.sharePriceSymbol}
      <span>
        <label for="share-price">{t.sharePrice}</label><Tip about={t.sharePrice}>{t.sharePriceTip}</Tip>
      </span>
      <NumberField id="share-price" prefix={app.sharePriceSymbol} bind:value={app.sharePrice} />
    {/if}
  </div>
</section>

<!-- What's compared, in short; the list itself opens in a sheet. -->
<section class="card">
  <div class="heading">
    <h3>{t.whatsCompared}</h3>
    <Tip about={t.whatsCompared}>
      <p>
        את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. לכל אחד יש כמה מסלולים, ולכל
        מסלול עמלות משלו. לצידם יש קופות ופוליסות, שמשקיעות את הכסף בשבילכם. ב״הוספה והסרה״ בוחרים מה להשוות,
        ושם אפשר גם להוסיף מסלול משלכם.
      </p>
    </Tip>
  </div>
  <p class="note">{t.tickedAtFirst}</p>
  {#if app.compared.length > 0}
    <ul class="compared" aria-label={t.whatsCompared}>
      {#each app.compared as { plan, name } (plan.id)}
        <li><span class="mark" class:yours={plan.yours} style:--plan-color={plan.color}></span>{name}</li>
      {/each}
    </ul>
  {/if}
  <div class="choose">
    <button onclick={() => (choosing = true)}>{t.addOrRemove}</button>
    <span class="count">{t.comparedOf(app.compared.length, app.plans.length)}</span>
  </div>
</section>
<PlanSheet {app} bind:open={choosing} />

<!-- Asked only while it changes something: a ticked plan pays a pension. -->
{#if app.pensionFromAge !== undefined}
  {@const from = app.pensionFromAge}
  <section class="card" transition:reveal>
    <div class="heading">
      <h3>{t.takingTheMoneyOut}</h3>
      <Tip about={t.takingTheMoneyOut}>{t.takingTheMoneyOutTip(from)}</Tip>
    </div>
    <div class="fields">
      <!-- Across the card, under its heading, which says what is chosen:
           beside a label the two wouldn't fit on one row. -->
      <div class="whole-row">
        <Choices label={t.takingTheMoneyOut} options={wayOutChoices(from)} bind:value={app.wayOut} />
      </div>
      {#if app.wayOut === 'pension'}
        <span>
          <label for="age">{t.yourAge}</label><Tip about={t.yourAge}>{t.yourAgeTip}</Tip>
        </span>
        <NumberField id="age" step={1} bind:value={app.age} />
        {#if app.age !== null}
          {@const then = Math.floor(app.age) + app.years}
          <p class="whole-row note">{then >= from ? t.oldEnough(then, from) : t.tooYoung(then, from)}</p>
        {/if}
      {/if}
    </div>
  </section>
{/if}

<!-- Last: what most people leave as it is. Off, the core is sent the
     defaults its ? names. -->
<section class="card">
  <div class="heading more">
    <h3>
      <label class="switch">
        <input type="checkbox" bind:checked={app.moreOptions} />
        {t.moreOptions}
      </label>
    </h3>
    <Tip about={t.moreOptions}>{t.moreOptionsTip(compactPercent(app.usualInflation))}</Tip>
  </div>
  {#if app.moreOptions}
    <div class="fields more" transition:reveal>
      <span class="wraps">
        <label for="deposit-growth">{t.growingBy}</label><Tip about={t.growingBy}>{t.growingByTip}</Tip>
      </span>
      <NumberField
        id="deposit-growth"
        suffix={t.percentAYear}
        step={0.5}
        bind:value={app.depositGrowthPercent}
      />

      <span>
        <label for="buy-every">{t.buyEvery}</label><Tip about={t.buyEvery}>{t.buyEveryTip}</Tip>
      </span>
      <select id="buy-every" bind:value={app.buyEveryMonths}>
        {#each BUYING_INTERVALS as { months } (months)}
          <option value={months}>{t.intervals[months]}</option>
        {/each}
      </select>

      <span>
        <label for="inflation">{t.inflation}</label><Tip about={t.inflation}>{t.inflationTip}</Tip>
      </span>
      <NumberField id="inflation" suffix={t.percentAYear} step={0.5} bind:value={app.inflationPercent} />

      <span class="whole-row">
        <label class="switch">
          <input type="checkbox" bind:checked={app.todaysMoney} />
          {t.todaysMoney}
        </label>
        <Tip about={t.todaysMoney}>{t.todaysMoneyTip}</Tip>
      </span>

      <span>
        <span class="label-text">{t.atTheEnd}</span><Tip about={t.atTheEnd}>{t.atTheEndTip}</Tip>
      </span>
      <Choices label={t.atTheEnd} options={atEndChoices} bind:value={app.atEnd} />
    </div>
  {/if}
</section>

<style>
  /* Not only the next one: the lists' dialogs sit between some cards. */
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
  /* More options: its fields bring the space under its heading, so that
     it opens and closes smoothly, and the heading is alone while it's off. */
  .heading.more {
    margin-bottom: 0;
  }
  .fields.more {
    padding-top: 10px;
  }
  .switch {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    user-select: none;
  }
  /* A card whose heading is its switch: as small as the other headings, and
     tall enough for a finger. */
  h3 .switch {
    min-height: 24px;
    font-size: inherit;
  }
  .caption {
    margin: 10px 0 6px;
    font-size: 0.8rem;
    color: var(--weak);
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
  .fields select {
    width: 100%;
  }
  /* A long label on two lines, so its column stays as narrow as in the
     other cards and "At the end" keeps both choices on one row. */
  .fields > .wraps {
    max-width: 8em;
    white-space: normal;
  }
  /* Across both columns: a switch with its ?, a note under a field. */
  .fields > .whole-row {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    white-space: normal;
  }
  .note {
    margin: 0;
    font-size: 0.8rem;
    line-height: 1.5;
    color: var(--weak);
  }
  /* The plans compared, each with its color as in the table. */
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
  /* Your own plans are hollow, like their dotted lines in the charts. */
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
  .explained {
    margin-top: 10px;
    font-size: 0.8rem;
    line-height: 1.5;
  }
  .explained p {
    margin: 6px 0 0;
  }
  .weak {
    color: var(--weak);
  }
  .rates {
    margin-top: 12px;
    font-size: 0.8rem;
  }
  .rates summary {
    width: fit-content;
    cursor: pointer;
    font-variant-numeric: tabular-nums;
  }
  .rates summary .weak {
    white-space: nowrap;
  }
  .rates summary:hover {
    color: var(--accent);
  }
  .rates .fields {
    margin-top: 8px;
    font-size: 0.9rem;
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
    min-width: 2ch;
    text-align: end;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
  /* One security in the "What you buy" tip. */
  .term {
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
  .term p {
    margin: 4px 0 0;
  }
</style>
