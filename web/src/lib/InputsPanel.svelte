<script lang="ts">
  import type { AppState, AtEnd } from './app.svelte'
  import BrokerPicker from './BrokerPicker.svelte'
  import Choices from './Choices.svelte'
  import type { Choice } from './core/core'
  import Examples from './Examples.svelte'
  import HebrewNames from './HebrewNames.svelte'
  import { dateText } from './format'
  import { BUYING_INTERVALS } from './link'
  import NumberField from './NumberField.svelte'
  import { lang, t } from './text'
  import { en } from './text/en'
  import Tip from './Tip.svelte'
  import YourPlans from './YourPlans.svelte'

  let { app }: { app: AppState } = $props()

  const atEndChoices: Choice<AtEnd>[] = [
    { value: 'sell', name: t.sell, englishName: en.sell, explanation: t.sellTip, hebrewNames: [] },
    { value: 'hold', name: t.keep, englishName: en.keep, explanation: t.keepTip, hebrewNames: [] },
  ]

  const security = $derived(app.securities.find(({ value }) => value === app.security)!)
  const exchange = $derived(app.exchanges.find(({ value }) => value === app.exchange)!)
  /** "ETF", "index fund", as in "your ETF". */
  const securityNoun = $derived(security.name === 'ETF' ? security.name : security.name.toLowerCase())
</script>

<section class="card">
  <div class="heading row">
    <h3>{t.tryAnExample}</h3>
    <label class="switch">
      <input type="checkbox" bind:checked={app.moreOptions} />
      {t.moreOptions}
    </label>
    <Tip about={t.moreOptions}>{t.moreOptionsTip}</Tip>
  </div>
  <Examples {app} />
</section>

<section class="card">
  <div class="heading">
    <h3>{t.whatYouBuy}</h3>
    <Tip about={t.whatYouBuy}>
      {#if lang === 'he'}
        <p>
          קרנות סל, קרנות מחקות, אג״ח ומניות הן כולן ניירות ערך. הבנקים ובתי ההשקעות גובים עמלות שונות על כל
          סוג. קרן סל וקרן מחקה יכולות להחזיק בדיוק את אותן חברות; מה ששונה הוא היכן קונים אותן, ולכן מה שהבנק
          או בית ההשקעות גובה.
        </p>
      {:else}
        <p>
          ETFs, funds, bonds and stocks are all securities (<bdi lang="he">ניירות ערך</bdi>). Brokers, meaning
          banks and investment houses (<bdi lang="he">בנקים ובתי השקעות</bdi>), charge different fees (<bdi
            lang="he">עמלות</bdi
          >) for each kind. An ETF and an index fund can hold the very same companies; what differs is where
          you buy them, and so what the broker charges.
        </p>
      {/if}
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
      {#if lang === 'he'}
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
      {:else}
        <p>
          Your {securityNoun} can be bought on different stock exchanges (<bdi lang="he">בורסות</bdi>): in Tel
          Aviv, or abroad in dollars or euros.
          {#if app.security === 'Bond'}
            Israel's government bonds trade in Tel Aviv in shekels, and US Treasuries in New York in dollars.
          {:else if app.security === 'Stock'}
            Teva, for example, trades in Tel Aviv in shekels and in New York in dollars.
          {:else}
            The same S&P 500, for example, is sold in Tel Aviv as a <bdi lang="he">קרן סל</bdi> in shekels, and
            in New York as an ETF such as VOO, in dollars.
          {/if}
          Brokers (<bdi lang="he">בנקים ובתי השקעות</bdi>) charge different fees (<bdi lang="he">עמלות</bdi>)
          for each exchange, and abroad some also charge for converting your shekels (<bdi lang="he"
            >המרת מט"ח</bdi
          >).
        </p>
      {/if}
    </Tip>
  </div>
  <Choices label={t.exchange} options={app.exchanges} bind:value={app.exchange} />
  <div class="explained">
    <HebrewNames names={exchange.hebrewNames} english={exchange.englishName} main={exchange.name} />
    <p class="weak">{exchange.explanation}</p>
  </div>
  {#if app.exchange !== 'Tlv'}
    <!-- Rarely changed, so folded away unless the download failed. -->
    <details class="rates" open={app.ratesStatus.kind === 'failed'}>
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
        {:else if lang === 'he'}
          שערי הבנק המרכזי האירופי, בטווח של שבריר אחוז מהשער היציג של בנק ישראל. שנו אותם כדי לנסות שערים
          אחרים.
        {:else}
          The European Central Bank's rates, within a fraction of a percent of the Bank of Israel's
          representative rate (<bdi lang="he">שער יציג</bdi>). Change them to try other rates.
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

    <span>
      <label for="buy-every">{t.buyEvery}</label><Tip about={t.buyEvery}>{t.buyEveryTip}</Tip>
    </span>
    <select id="buy-every" bind:value={app.buyEveryMonths}>
      {#each BUYING_INTERVALS as { months } (months)}
        <option value={months}>{t.intervals[months]}</option>
      {/each}
    </select>

    {#if app.moreOptions}
      <span>
        <label for="deposit-growth">{t.growingBy}</label><Tip about={t.growingBy}>{t.growingByTip}</Tip>
      </span>
      <NumberField
        id="deposit-growth"
        suffix={t.percentAYear}
        step={0.5}
        bind:value={app.depositGrowthPercent}
      />
    {/if}
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

    {#if app.moreOptions}
      <span>
        <label for="inflation">{t.inflation}</label><Tip about={t.inflation}>{t.inflationTip}</Tip>
      </span>
      <NumberField id="inflation" suffix={t.percentAYear} step={0.5} bind:value={app.inflationPercent} />

      <span>
        <span class="label-text">{t.atTheEnd}</span><Tip about={t.atTheEnd}>{t.atTheEndTip}</Tip>
      </span>
      <Choices label={t.atTheEnd} options={atEndChoices} bind:value={app.atEnd} />
    {/if}
  </div>
</section>

<section class="card">
  <div class="heading">
    <h3>{t.brokersAndPlans}</h3>
    <Tip about={t.brokersAndPlansShort}>
      {#if lang === 'he'}
        <p>
          את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. כל אחד מציע כמה מסלולים, לכל
          אחד העמלות שלו: ללאומי, למשל, יש אחד ללאומי טרייד, אחד לקבוצת 18+ שלו ואחד לאפליקציית פפר. סמנו את
          אלה שרוצים להשוות.
        </p>
      {:else}
        <p>
          The same {securityNoun}, on the same exchange, can be bought through many brokers: banks and
          investment houses (<bdi lang="he">בנקים ובתי השקעות</bdi>). Each offers (<bdi lang="he">מציע</bdi>)
          several plans (<bdi lang="he">מסלולים</bdi>), each with its own fees: Leumi, for example, has one
          for Leumi Trade, one for its 18+ group and one for its Pepper app. Tick the ones to compare.
        </p>
      {/if}
    </Tip>
  </div>
  <BrokerPicker {app} />
</section>

<section class="card">
  <div class="heading">
    <h3>{t.yourPlans}</h3>
    <Tip about={t.yourPlans}>
      {#if lang === 'he'}
        <p>
          בנקים ובתי השקעות נותנים לא פעם הנחה בעמלות למי שמתמקח: 0.06% במקום 0.07% שהוצעו, נניח, או מינימום
          נמוך יותר. העתיקו מסלול עם ✎ ושנו את העמלות שאתם חושבים שתוכלו לקבל, או הוסיפו אחד שלא ברשימה עם +
          מסלול חדש.
        </p>
      {:else}
        <p>
          Banks and investment houses often give a discount on fees (<bdi lang="he">הנחה בעמלות</bdi>) if you
          bargain (<bdi lang="he">מיקוח</bdi>): 0.06% instead of the offered 0.07%, say, or a lower minimum.
          Copy a plan with ✎ and change the fees you think you can get, or add one that isn't listed with +
          New plan.
        </p>
      {/if}
    </Tip>
  </div>
  <YourPlans {app} />
</section>

<style>
  section + section {
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
  /* A heading with a switch at its right. */
  .row {
    gap: 6px;
  }
  .row h3 {
    flex: 1;
  }
  .switch {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    user-select: none;
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
