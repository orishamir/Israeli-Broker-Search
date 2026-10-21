# Fees the app doesn't model

Found while reading the tariffs (September 2026). Each may be too small or too
rare to matter for a long-term investor; decide one by one whether to add it.
Amounts are the tariff's maximums unless marked as an offer.

## Every broker

- **Third-party fees on US trades:** SEC fee, FINRA TAF, ECN, ORF, charged on
  top of the commission (Altshuler, IBI, Meitav, Excellence, Interactive).
- **Dividend and interest handling:** Altshuler 0.3% of the payment, Meitav
  0.3%; Excellence charges for dividend reinvestment "as agreed". A US
  fund's dividends are reinvested at no cost; a share's or a bond's aren't
  simulated (the return is taken as total return).
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

- **Tax on what a share or a bond pays along the way:** 25% of a share's
  dividends, 15% or 25% of a bond's interest, at each payment at a broker,
  never inside a fund. For a fund it's counted, in what the fund keeps back
  (`products.rs`); for a share or a bond held directly, the return is taken
  as total return, so it isn't. On a 4% bond it would be about 1% a year.
- **Unlinked shekel bonds:** taxed at 15% of the whole gain (section
  91(b)(3)), not 25% of the real one. The app doesn't know which bond is
  bought, so every bond is taxed as 25% of the real gain.
- **The surtax beyond its least:** the surtax is counted as if the saver had
  no other income in the year of the sale (5% of the gain above the
  threshold); a salary that year adds 3% of the part of all income above
  it. Selling over several years, which can avoid it, isn't modelled.
- **Custody and the monthly fee as deductions:** see `sources.md`.
- **Losses set against other gains, and withdrawing in parts.**
- **Products other than the S&P 500's:** every ETF and index fund is
  taken to hold the S&P 500, the only index measured (`products.rs`). An
  Israeli ETF on the TA-125 costs less than one on the S&P 500, and a bond
  fund other things again. Currency-hedged funds (מנוטרלות מט״ח) aren't
  compared: their return in shekels isn't the index's.
- **Reinvesting a US fund's dividends:** they're bought with the next
  purchase at no cost of their own.
- **A proposed cap on the pension exemption:** in June 2026 a Finance
  Ministry committee recommended limiting the tax-free gains of a provident
  fund for investment to ₪200,000 a saver
  ([Globes](https://www.globes.co.il/news/article.aspx?did=1001547467),
  [Bizportal](https://www.bizportal.co.il/longtermsavings/news/article/20034793)).
  Not law: the exemption is modelled whole, and the fund's caveat says so.
  Check again when the pension's rules change.
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
- **A fund's other tracks:** each fund is compared on its S&P 500 track
  alone, against an S&P 500 fund at a broker, and isn't compared with a
  share or a bond. A managed track (equity, general) holds Israeli shares and
  bonds and hedges the dollar, so it earns something else.
- **A savings policy's own average fee:** not published apart from the
  insurers' other policies; the app's is a reading.

## Money for the short term

What `short_term.rs` leaves out (October 2026). The sources are in
`sources.md`, "Money for the short term".

- **Saving every month at a bank:** a monthly amount goes into the funds,
  but a fixed-rate deposit takes one sum, so with one no bank is compared. A
  bank's savings plan (תוכנית חיסכון), its way to deposit monthly, isn't
  compared: the Bank of Israel doesn't publish its rates.
- **Variable-rate deposits for a term:** only fixed-rate ones. The daily
  deposit (פיקדון יומי, a deposit that can be taken out any day) is
  compared, for money that may be needed any day only (REDESIGN.md §4.9),
  at each bank's average variable rate for September; its rate is taken
  to move with the Bank of Israel's point for point, which the banks don't
  promise. Variable deposits for a term (a week, a month, more) aren't.
- **Taking a deposit out early:** the money is taken to stay to the end of
  the term. Sooner, where the bank allows it, a deposit earns less or
  nothing, by its terms.
- **Renewing shorter deposits:** one deposit for the whole period. Rolling a
  month's deposit over, at rates to come, isn't compared.
- **Linked and foreign-currency deposits and funds:** shekel ones only.
- **Makam** is compared only for money that may be needed any day, for a
  wait of up to a year; for a date within a year it isn't (the guide's
  short route still compares the fund and a deposit). **A fund's cash track
  (מסלול כספי) and fixed-term money market funds (קרן כספית מתחדשת):** not
  yet listed.
- **What a fund really earns:** taken as the Bank of Israel's rate less its
  fees. Funds hold bills, short bonds, deposits and, most of them, some
  corporate bonds, and can earn a little more or less.
- **A trade fee on a fund at an investment house:** none is counted, as at a
  bank. Excellence charges ₪16 a trade on managed funds; IBI trades some
  managers' funds free.
- **A lower tax:** 15% and 25% are the most an individual pays on interest
  and on a capital gain; someone with a low enough income may pay less.

## What the guide doesn't do

The guide (`crates/core/src/guide.rs`) answers for one kind of holding at a
time. Left out (October 2026); the rules it does follow are in `sources.md`,
"The guide's rules".

- **Mixes and portfolios:** no split between stocks and bonds, no
  allocation, no rebalancing. The answer is one kind, with the others drawn
  beside it so the saver sees what a mix would sit between.
- **Personal advice:** it says what fits money like this given the answers,
  not what a person should do. It doesn't ask about debts, income, other
  savings, a mortgage, or risk tolerance beyond the date and the cushion.
- **Pension funds (קרנות פנסיה) and the provident fund for savings (קופת גמל
  לחיסכון):** not compared. Pension funds are a different product (a pension
  with insurance), and the provident fund for savings is left out on
  purpose (see "Tax and the vehicles"). The glossary has a card for the
  provident fund for savings that says why it isn't here.
- **The study fund's eligibility:** it needs an employer who pays into one,
  or self-employment. The guide shows it with "who can open one" and never
  chooses it for someone; it can't tell whether the saver qualifies.
- **The exchange rate in the history:** the S&P 500's history is in
  dollars. What an Israeli saw in shekels moved with the rate too; the band
  doesn't show that, and the page says the past is in dollars.
- **The history's end:** June 2023 (Shiller's data). Later years are not in
  the band.
- **Coupons, bond prices, CPI-linked deposits:** see "Government bonds and
  CPI-linked deposits". A CPI-linked deposit is not modelled; the CPI-linked
  government bond is the stand-in.
- **A different expected return per kind:** the index kinds share the
  expected return the saver types (10% unless changed); the safe kinds
  earn the Bank of Israel's rate or the curve's yield. Both are
  assumptions the page states, not forecasts.
- **Taxes beyond the modelled ones:** a low earner's lower rate, the estate
  tax's avoidance beyond the note, and the surtax beyond its least.

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

## Bank Hapoalim

- Its clubs (appendix A and the tariff's population groups: the young
  "Poalim Tzair", soldiers, students, seniors and others) take a set
  percentage off the tariff's rows, in the tariff's pages, some off the
  securities custody and conversion, some off the trades. None is a plan.
- Orders by phone or other "other channels" (appendix E has columns for the
  clerk and for direct channels only): the two are modelled.
- Treasury bills (מק"מ): 0.11% at a branch (min ₪19), 0.10% direct (min ₪15).
- The conversion of up to $99 is a flat $5.50 at a branch and $3.30 direct
  (a caveat says the minimum used is higher).
- Third-party costs (the exchange's payments in Tel Aviv; abroad the broker's
  and the custodian's, ADR fees) as actually charged, in a caveat.
- Mutual funds' distribution fee, options and futures, short-selling lending.

## Bank Discount

- The bank's benefits for seniors (0.20% on Tel Aviv shares and bonds and
  0.04% custody a quarter, its page says) and for hi-tech employees apply "to
  the tariff on the customer" and may be withdrawn: left out, as the clubs of
  the other banks.
- Treasury bills (מק"מ): 0.1%, min ₪25, and 0.09%, min ₪22.50 direct;
  government bonds issued at a discount of up to a year, 0.12%.
- Conversions up to $10 are free and up to $100 half price (a caveat).
- Abroad: the agent's fee, the SEC fee, the broker's, the custodian's and the
  SWIFT costs, as actually charged (a caveat). Custody abroad 0.25% a quarter
  plus the custodian's costs abroad.
- Mutual funds' distribution fee (0.1% to 0.35% a year), options and futures.
- Mercantile Discount, the bank's subsidiary, has its own tariff.

## Beinleumi

- Everything that is said about Otsar Hahayal above, except the club and
  Top Trade, which are Otsar's own: one tariff, one file.

## Altshuler Shaham Trade

- T-bills (מק"מ): 0.1%, min ₪3.5. Options, futures, hedge funds (0.3%,
  min ₪100). Off-exchange trades: min ₪50.
- The new-customer offer's lasting waiver needs an account with "frequent
  trading", at the company's discretion, and a balance of at least ₪5,000 at
  all times; after a year without activity it's withdrawn (offer rules,
  January 2026). The waiver is assumed, flagged as may cost more.

## Bank Leumi (Leumi Trade)

- A minimum fee is never more than 27% (online) or 30% (branch) of the trade.
- The first year opened through the app: custody at most 50% of the tariff
  (a caveat says so).
- "Concentrated purchase to portfolio" in Leumi Trade: a percentage fee with
  no minimum; whether it can buy a single fund isn't said (a caveat says so).
- Leumi 18+ bonds are modelled as 0.35% within Online's bounds; the true
  "better of the group's and Online's price" differs by at most ₪1, on
  orders between ₪6,500 and ₪7,700.
- Pepper: ₪4 and $4 are stated only for orders up to ₪30,000 and $8,000; its
  site says larger orders are priced by the tariff, without saying which
  row. They're priced at the online rows, the least the tariff allows, and
  flagged in the table; a banker's rows cost more.
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
- The handling fee is waived for the account's whole life for those who
  join by 31 December 2026 (modelled). Those who join later get two free
  years, or three for more than ₪10,000 deposited within 30 days, then ₪15
  a month: a caveat says so. tradingil's ₪100 gift and minimum deposit of
  ₪4,500 until 10 October 2026, and gemeltop's refund of commissions, aren't
  published by Excellence.

## Branch plans

- **Banks that aren't in the app** (One Zero, Mercantile Discount, Massad,
  Jerusalem, Union, Yahav, Pagi and the rest): there is no tariff in
  `policies`, so no branch plan. The app stands in the dearest
  branch plan of the six that are; the guide calls it "another bank", not
  the bank it was taken from. One Zero publishes its tariff
  (`onezerobank.com/company/information/2250`) but its files sit behind a
  Cloudflare check that a script can't pass, so it wasn't read; it takes
  a person to download the "full tariff" PDF and the "securities fees:
  comparative information" PDF into `policies`.
- **Otsar Hahayal's "other channels" column** (trading systems and the
  telephone dealing room, cheaper than the clerk's): left out; the clerk's
  column is the branch plan and the website's is Online.
- **The investment houses** (Altshuler, Excellence, IBI, Meitav, Interactive)
  are not banks and their tariffs have no branch column (they list a Full tariff and offers), so they have no branch plan.
- **Leumi's standing order through a banker**: priced as an ordinary order
  (see `sources.md`, "Branch tariffs").

## Government bonds and CPI-linked deposits

What `short_term/government_bonds.rs` leaves out (October 2026). The
sources are in `sources.md`, "Government bonds and the yield curve".

- **CPI-linked deposits (פיקדון צמוד מדד):** not modelled. The Bank of
  Israel publishes none (its Excel has fixed and variable unlinked
  deposits only), and the banks' own pages gave a dated rate for one bank
  at most (Leumi, 1.00% over the index, undated). The CPI-linked
  government bond covers the need: the same protection from prices, at a
  published real yield.
- **Reinvesting the coupons:** a bond pays interest along the way; the
  model has it reinvested at the bond's own yield. In reality it is
  reinvested at the day's rates.
- **The bond's price before it is repaid:** sold early, the price moves
  with interest rates. The model holds to the end, so the sum is known.
- **The curve changing:** each monthly deposit is taken to buy at today's
  curve. Later deposits will meet other yields.
- **Bonds under a year or over 15 years:** not compared; the Bank of
  Israel's curve starts at one year and ends at fifteen here.
- **Makam's own yield:** taken as the curve's one-year yield, whatever the
  wait (up to a year); a bill repaid sooner earns its own, which the curve
  doesn't give. Its custody is counted as the broker's plan charges for
  bonds, although banks may not charge custody on makam: the figure is on
  the dear side. Sold before it's repaid, its price moves a little; the
  model holds it to the end of the wait.
- **The exact bond:** a bond repaid on the day the saver's wait ends is
  taken to exist; the series' dates are on the stock exchange's list.
- **The price of a redemption:** counted as free; see `sources.md`.
- **Deposits past five years:** the Bank of Israel publishes no rates, so
  a deposit isn't offered for them (`NotOffered::TooLong`); the money
  market fund goes to 50 years at the rate the saver expects less its fee.
