# Fees the app doesn't model

Found while reading the tariffs (September 2026). Each may be too small or too
rare to matter for a long-term investor; decide one by one whether to add it.
Amounts are the tariff's maximums unless marked as an offer.

## Every broker

- **Third-party fees on US trades:** SEC fee, FINRA TAF, ECN, ORF, charged on
  top of the commission (Altshuler, IBI, Meitav, Excellence, Interactive).
- **Dividend and interest handling:** Altshuler 0.3% of the payment, Meitav
  0.3%; Excellence charges for dividend reinvestment "as agreed". Dividends
  themselves aren't simulated (the return is taken as total return).
- **Real-time data and advanced trading systems:** up to ₪3,000 a month
  (IBI, Meitav, Excellence, Altshuler); Interactive charges for live US quotes.
- **Joining gifts and refunds** (one-off):
  - Altshuler ₪200;
  - Meitav ₪100;
  - IBI ₪300 (₪500 for the security forces);
  - Excellence refunds commissions up to ₪20,000 (per a comparison site);
  - Interactive refunds up to $50 of commissions in the first three months;
  - Leumi (2026 only): refunds up to ₪3,000 of the year's losses or
    commissions, for portfolios from ₪100,000.
- **Credit and debit interest**, including Interactive's possible negative
  interest on positive balances.
- **Moving securities to another broker:** ₪20–₪35 a security (Israeli),
  $10–$40 (foreign).
- **Orders not executed or cancelled:** Leumi ₪25; the investment houses
  charge nothing.

## Tax and the vehicles

What `vehicles.rs` and the tax at the end leave out (September 2026). The
sources are in `sources.md`, "Tax and the vehicles".

- **Tax on dividends along the way:** 25% of each payment at a broker, for a
  security that pays out; less inside a fund or an accumulating ETF. On the
  S&P 500 it's about 0.3% a year. The return is taken as total return.
- **Unlinked shekel bonds:** taxed at 15% of the whole gain (section
  91(b)(3)), not 25% of the real one. The app doesn't know which bond is
  bought, so every security is taxed as 25% of the real gain.
- **The exchange rate as the index:** for a security in foreign currency
  the law measures the real gain against the exchange rate (section 88,
  "מדד"). Exchange rates stay the same in the simulation, so the price
  index is used for every security: the shekel is taken to weaken as prices
  rise.
- **The surtax (מס יסף):** 3% more, and 2% more on income from capital, on
  what passes ₪721,560 in a year. A sale large enough pays it in any
  vehicle that taxes the gain.
- **Custody and the monthly fee as deductions:** see `sources.md`.
- **Losses set against other gains, and withdrawing in parts.**
- **What the investment itself charges:** an ETF's or fund's own yearly fee
  at a broker, and a provident fund's direct expenses (הוצאות ישירות, up to
  0.25% a year on managed tracks, close to nothing on index ones). Both are
  left out, on both sides.
- **A proposed cap on the pension exemption:** in June 2026 a Finance
  Ministry committee recommended limiting the tax-free gains of a provident
  fund for investment to ₪200,000 a saver
  ([Globes](https://www.globes.co.il/news/article.aspx?did=1001547467),
  [Bizportal](https://www.bizportal.co.il/longtermsavings/news/article/20034793)).
  Not law: the exemption is modelled whole. Check again before the pension
  is shown.
- **Tax benefits on deposits:** the self-employed may deduct deposits to
  a study fund of up to 4.5% of their income, to ₪13,203 a year (2026). Not
  counted, by the user's decision; a caveat says the fund may be worth
  more.
- **An employee's study fund:** it needs the employer, who pays three
  quarters of it. The app counts a self-employed saver's fund; the
  employer's share isn't counted.
- **A provident fund for savings (קופת גמל לחיסכון):** left out, by the
  user's decision (30 September 2026). The first ₪38,412 deposited each year
  (2026) is pension money: worth its tax benefit on the way in, and taken
  out as a pension taxed by the saver's income. Only what's above that goes
  by תיקון 190 (from 60: a tax-free pension, or a lump sum at 15% of the
  nominal gain for those with a pension of ₪5,306 a month), per
  [Meitav](https://www.meitav.co.il/provident_pension/amendment_190/).
  Counting it needs the saver's income, tax bracket and what their pension
  already uses. "What isn't counted" on the page says so.
- **Each company's own fee:** the funds are listed by kind, at what savers
  pay on average and at the cheapest and dearest company (`sources.md`,
  "Funds and policies"). Self-managed funds (ניהול אישי, IRA) aren't listed.
- **A managed track's own return:** a fund is taken to earn what the chosen
  security does before fees, which only an index-following track does.
- **A savings policy's own average fee:** not published apart from the
  insurers' other policies; the app's is a reading.

## Mizrahi-Tefahot Bank

- Custody cap: ₪10,750 a quarter on each of Tel Aviv and foreign holdings.
  Matters from about ₪5.5 million.
- A minimum fee is never more than 25% of the trade (part 4, note 6).
- T-bills (מק"מ): 0.11% at a branch (min ₪19), 0.10% online (min ₪17.10).
- Trading through the "PC service" channel: slightly different prices
  (conversion 0.1425% instead of 0.133%).
- Custodian fee 0.1% (min ₪25, max ₪2,000), a special service; broker and
  custodian expenses abroad, passed through (a caveat says so).
- Mutual funds' distribution fee where the bank has no agreement with the
  manager: 0.1%–0.35% a year; index funds exempt.
- Pensioners: half the conversion fee only. The young groups' other
  benefits (free account operations, credit) aren't securities fees.
- Actual averages (its disclosure, first half of 2026) are well below the
  tariff: 0.13%–0.32% on Israeli stocks and bonds, custody 0.08%–0.44% a
  year. Banks negotiate.

## Bank Otsar Hahayal

- Custody caps: ₪4,000 a security and ₪10,000 a deposit, each a quarter.
  Matter from about ₪2 million a security and ₪5 million a deposit.
- A fee is never more than 30% of the trade (part 4, notes 2 and 5).
- Third-party costs (part 11), "as actually charged" and a caveat says so:
  the exchange's trading and clearing fees in Tel Aviv; abroad the broker's
  (up to 6¢ a share in the US, 0.18% elsewhere), the custodian's (up to $5 an
  order in the US, €200 in Europe; safekeeping up to 0.72% a year of the
  holding), SEC fees, ADR/GDR fees, fund expenses up to 9%.
- T-bills: 0.11% at a branch (min ₪16), 0.1% online (min ₪15).
- Orders not executed: ₪25 through a banker, free online.
- Students and yeshiva students (appendix A): Tel Aviv 0.40% and abroad
  0.45% with the tariff's bounds, custody 0.1365% (Israeli) and 0.1393%
  (foreign) a quarter, 33% off the conversion fee and a rate benefit of
  0.45%. Worse than Online on trades and better on custody and conversion;
  whether the two combine isn't said, so it isn't a plan yet.
- Otzar Habitachon club: a custodian fee of 0.01% (min ₪15, max ₪450), T-bills
  0.05%, bond redemption free, ₪9 for an unexecuted order through a banker.
- Top Trade: nothing is stated for a portfolio above ₪200,000 (flagged in
  the table). Its conscripts' version (every fee waived until 31 December
  2026, up to ₪300,000 and 150 orders) is a caveat only.
- HighTech 10, the First International's club for hi-tech employees, on
  Otsar's site too: independent traders get 0.1% (min ₪5) on Tel Aviv, 0.1%
  (min $8) abroad and no custody for two years, then the club's 0.18%, 0.22%
  and 0.1% a quarter. Its leaflet (December 2022) ends every securities
  benefit on 31 December 2026, so it's left out.
- Actual averages (the bank's table, first half of 2026): 0.02%–0.19% on
  Israeli stocks and bonds, custody 0.25%–0.43% a year, against the
  tariff's 0.78%.

## Altshuler Shaham Trade

- T-bills (מק"מ): 0.1%, min ₪3.5. Options, futures, hedge funds (0.3%,
  min ₪100). Off-exchange trades: min ₪50.
- The new-customer offer requires an *active* account and a balance of at
  least ₪5,000 at all times; after a year without activity the benefits may
  be withdrawn (offer rules, January 2026). Shown as a caveat only.

## Bank Leumi (Leumi Trade)

- Custody caps: ₪3,700 a security a quarter, ₪11,500 a deposit (Tel Aviv);
  ₪7,400 and ₪23,000 abroad. Matter from about ₪2.5 million a security.
- A minimum fee is never more than 27% (online) or 30% (branch) of the trade.
- The first year opened through the app: custody at most 50% of the tariff.
- "Concentrated purchase to portfolio" in Leumi Trade: a percentage fee with
  no minimum.
- Leumi 18+ bonds are modelled as 0.35% within Online's bounds; the true
  "better of the group's and Online's price" differs by at most ₪1, on
  orders between ₪6,500 and ₪7,700.
- Pepper: ₪4 and $4 are stated only for orders up to ₪30,000 and $8,000; its
  site says larger orders are priced by the tariff, without saying which
  row. Flagged in the table; a tiered price would model it.
- Pepper's package of March 2026: a year without trade, minimum and currency
  fees and a grant of up to ₪1,500, for customers who move their salary to
  Pepper. One year only.
- 2026 promotion for new customers: no custody fee in 2026, loss refund
  (above). Ends 31.12.2026, so it's left out.
- Actual averages (Tel Aviv Stock Exchange, June 2026) are well below the
  tariff: custody 0.25% (Israeli) and 0.36% (foreign) a year at ₪100–200
  thousand, against 0.6% and 0.8%. Banks negotiate.
- The conversion markup is the bank's transfers-and-checks rate, measured at
  0.86% under and 0.98% over the representative rate on 28 September 2026
  and modelled as 0.9% each way. An agreed rate quoted live in the app may
  be tighter; the rates move daily.

## Interactive Israel

- US stocks: at most 2% of the trade's value (only matters for penny stocks).
- Tel Aviv: only for institutional clients, so not offered in the app. (Its
  price would be 0.06%, min ₪10, plus 0.08% custody.)
- Mutual funds: the tariff's row is unclear (EUR "7.5", USD "Transaction 30,
  minimum 7.5"); left as not offered.
- Converting dollars back to shekels may follow the dollar row (0.005%, min
  $5) rather than the shekel one (min ₪10); ₪10 is assumed both ways.
- Conversions are at the market's live rate (its site), so the only cost
  beyond the fee is the currency market's own bid-ask spread, a few
  hundredths of a percent; Interactive Brokers' automatic conversion service,
  which the automatic investment plan may use, moves the rate by up to 0.03%
  instead of a fee (₪0.60 on ₪2,000).
- Bonds are priced on face value (0.2%); the app uses the trade's value.
- Other exchanges (UK, Canada, Asia...) and options, futures, warrants.

## IBI

- US tracks for OTC and special trades: 5¢ a share, min $14, or 2%.
- Conversions of $15,000 or more get a 0.5% markup instead of 0.7%
  (tradingil); 0.7% is charged on all, and the plan says so above $15,000.
- Foreign funds and alternative instruments: 0.275% *plus the broker's cost*
  (only the 0.275% is modelled). Foreign bond redemption: $70.
- Halted or worthless foreign securities: $11 a security a month. ADR custody
  fees are passed through.
- Managed (active) mutual funds of 12 fund managers: no trade fee (the app's
  index funds aren't among them).
- tradingil's joining offer adds two free years of the handling fee;
  gemeltop's doesn't, so the fee is charged from the first month.
- Another account for the same customer: up to ₪100 a month.
- Canada: CAD 0.01 a share, min CAD 55. Asia: 0.3%, min 30.

## Meitav Trade

- T-bills: 0.1%, min ₪10. Off-exchange trades: 0.3%, min ₪50.
- Canada: 3 CAD cents a share, min CAD 50. Other exchanges: 0.25%, min 25.
- Foreign funds: 0.2%, min $20, *plus clearing and correspondent fees*.
- Custodian fee 0.1% (min ₪30); non-tradable securities $10 a month.
- Hedge funds in trust: 0.15%, min ₪50.
- A funds-only track ("מסלול קרנות ללא עמלות"): no handling, custody or trade
  fees, for accounts of at least ₪100,000 that hold managed (active) funds
  only. Not for the app's index funds and ETFs.
- tradingil's deal for reservists until 30 September 2026: ₪200 and four
  free years of the handling fee.

## Excellence Trade

- Custody for managed accounts and "accounts that hold assets rather than
  trade actively": 0.1% a quarter, instead of the general 0.6%. A long-term
  investor might count as one; the full tariff plan uses the general rate.
- US: at most 2% of the trade; the 0.3% track adds the broker's fee.
- Traditional (active) mutual funds: ₪16 a trade. Hedge funds 0.15%–0.35%,
  min 50.
- A third year without the handling fee: the rules of Excellence's two free
  years give it for a deposit above ₪15,000 within 30 days of opening, and
  some sign-up links offer 3 years outright. tradingil's ₪100 gift and
  minimum deposit of ₪4,500 until 10 October 2026, and gemeltop's refund of
  commissions, aren't published by Excellence. The offset of the fee by
  trade fees is modelled.
