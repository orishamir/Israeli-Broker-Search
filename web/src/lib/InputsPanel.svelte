<script lang="ts">
  import type { AppState } from './app.svelte'
  import BrokerPicker from './BrokerPicker.svelte'
  import Choices from './Choices.svelte'
  import HebrewNames from './HebrewNames.svelte'
  import NumberField from './NumberField.svelte'
  import Tip from './Tip.svelte'
  import YourPlans from './YourPlans.svelte'

  let { app }: { app: AppState } = $props()

  const buyingIntervals = [
    { months: 1, name: 'month' },
    { months: 2, name: '2 months' },
    { months: 3, name: '3 months' },
    { months: 6, name: '6 months' },
    { months: 12, name: 'year' },
  ]

  const security = $derived(app.securities.find(({ value }) => value === app.security)!)
  const exchange = $derived(app.exchanges.find(({ value }) => value === app.exchange)!)
  /** "ETF", "mutual fund", as in "your ETF". */
  const securityNoun = $derived(security.name === 'ETF' ? security.name : security.name.toLowerCase())
</script>

<section class="card">
  <div class="heading">
    <h3>What you buy</h3>
    <Tip about="What you buy">
      <p>
        ETFs, funds, bonds and stocks are all securities (<bdi lang="he">ניירות ערך</bdi>). Brokers, meaning
        banks and investment houses (<bdi lang="he">בנקים ובתי השקעות</bdi>), charge different fees (<bdi
          lang="he">עמלות</bdi
        >) for each kind.
      </p>
      {#each app.securities as option (option.value)}
        <div class="term">
          <strong>{option.name}</strong>
          <HebrewNames names={option.hebrewNames} />
          <p class="weak">{option.explanation}</p>
        </div>
      {/each}
    </Tip>
  </div>
  <Choices label="Security" options={app.securities} bind:value={app.security} />
  <!-- Always shown: the English names are the ones people don't know. -->
  <div class="explained">
    <HebrewNames names={security.hebrewNames} />
    <p class="weak">{security.explanation}</p>
  </div>
</section>

<section class="card">
  <div class="heading">
    <h3>Traded on</h3>
    <Tip about="Traded on">
      <p>
        Your {securityNoun} can be bought on different stock exchanges (<bdi lang="he">בורסות</bdi>): in Tel
        Aviv, or abroad in dollars or euros. Brokers (<bdi lang="he">בנקים ובתי השקעות</bdi>) charge different
        fees (<bdi lang="he">עמלות</bdi>) for each exchange, and abroad some also charge for converting your
        shekels (<bdi lang="he">המרת מט"ח</bdi>).
      </p>
    </Tip>
  </div>
  <Choices label="Exchange" options={app.exchanges} bind:value={app.exchange} />
  <div class="explained">
    <HebrewNames names={exchange.hebrewNames} />
    <p class="weak">{exchange.explanation}</p>
  </div>
  {#if app.exchange !== 'Tlv'}
    <!-- Rarely changed, so folded away unless the download failed. -->
    <details class="rates" open={app.ratesStatus.kind === 'failed'}>
      <summary>
        $1 = ₪{app.ilsPerUsd ?? '?'} · €1 = ₪{app.ilsPerEur ?? '?'}
        <span class="weak">
          {#if app.ratesStatus.kind === 'downloading'}
            · downloading today's…
          {:else if app.ratesStatus.kind === 'downloaded'}
            · {app.ratesStatus.date}
          {:else}
            · couldn't download today's
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
          Couldn't download today's rates ({app.ratesStatus.error}). Check these defaults.
        {:else}
          European Central Bank rates, for converting your shekels. Change them to try other rates.
        {/if}
      </p>
    </details>
  {/if}
</section>

<section class="card">
  <h3>Deposits</h3>
  <div class="fields">
    <label for="first-deposit">One-time deposit</label>
    <NumberField id="first-deposit" prefix="₪" step={1000} bind:value={app.firstDeposit} />

    <label for="monthly-deposit">Every month</label>
    <NumberField id="monthly-deposit" prefix="₪" step={100} bind:value={app.monthlyDeposit} />

    <span>
      <label for="buy-every">Buy every</label><Tip about="Buy every">
        Deposits wait as cash until the next purchase. Buying less often means paying fewer minimum fees, but
        the cash doesn't grow while it waits.
      </Tip>
    </span>
    <select id="buy-every" bind:value={app.buyEveryMonths}>
      {#each buyingIntervals as { months, name } (months)}
        <option value={months}>{name}</option>
      {/each}
    </select>
  </div>
</section>

<section class="card">
  <h3>Expectations</h3>
  <div class="fields">
    <span>
      <label for="yearly-return">Yearly return</label><Tip about="Yearly return">
        How much the security grows a year, in its own currency. The S&P 500 has averaged about 10%.
      </Tip>
    </span>
    <NumberField id="yearly-return" suffix="%" step={0.5} bind:value={app.yearlyReturnPercent} />

    <label for="years">Years</label>
    <div class="range">
      <input id="years" type="range" min="1" max="50" bind:value={app.years} />
      <output for="years">{app.years}</output>
    </div>

    {#if app.sharePriceSymbol}
      <span>
        <label for="share-price">Share price</label><Tip about="Share price">
          Today's price of one share. Brokers here sell whole shares only, so a deposit too small for a share
          waits for the next purchase. It grows with the yearly return.
        </Tip>
      </span>
      <NumberField id="share-price" prefix={app.sharePriceSymbol} bind:value={app.sharePrice} />
    {/if}
  </div>
</section>

<section class="card">
  <div class="heading">
    <h3>Brokers and plans to compare</h3>
    <Tip about="Brokers and plans">
      <p>
        The same {securityNoun}, on the same exchange, can be bought through many brokers: banks and
        investment houses (<bdi lang="he">בנקים ובתי השקעות</bdi>). Each offers (<bdi lang="he">מציע</bdi>)
        several plans (<bdi lang="he">מסלולים</bdi>), each with its own fees. Tick the ones to compare.
      </p>
    </Tip>
  </div>
  <BrokerPicker {app} />
</section>

<section class="card">
  <div class="heading">
    <h3>Your plans</h3>
    <Tip about="Your plans">
      <p>
        Banks and investment houses often give a discount on fees (<bdi lang="he">הנחה בעמלות</bdi>) if you
        bargain (<bdi lang="he">מיקוח</bdi>). Copy a plan with ✎ and change the fees you think you can get, or
        add one that isn't listed with + New plan.
      </p>
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
    text-align: right;
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
