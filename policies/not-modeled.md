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
- Leumi 18+ group: Tel Aviv bonds at 0.35% (online is 0.4%). Benefits don't
  stack, so a group member gets the lower; only the conversion's two prices
  are modelled.
- Pepper: ₪4 and $4 are stated only for orders up to ₪30,000 and $8,000.
- 2026 promotion for new customers: no custody fee in 2026, loss refund
  (above). Ends 31.12.2026, so it's left out.
- Actual averages (Tel Aviv Stock Exchange, June 2026) are well below the
  tariff: custody 0.25% (Israeli) and 0.36% (foreign) a year at ₪100–200
  thousand, against 0.6% and 0.8%. Banks negotiate.

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
- Foreign funds and alternative instruments: 0.275% *plus the broker's cost*
  (only the 0.275% is modelled). Foreign bond redemption: $70.
- Halted or worthless foreign securities: $11 a security a month. ADR custody
  fees are passed through.
- Managed (active) mutual funds of 12 fund managers: no trade fee (the app's
  mutual funds are index funds, which aren't included).
- Another account for the same customer: up to ₪100 a month.
- Canada: CAD 0.01 a share, min CAD 55. Asia: 0.3%, min 30.

## Meitav Trade

- T-bills: 0.1%, min ₪10. Off-exchange trades: 0.3%, min ₪50.
- Canada: 3 CAD cents a share, min CAD 50. Other exchanges: 0.25%, min 25.
- Foreign funds: 0.2%, min $20, *plus clearing and correspondent fees*.
- Custodian fee 0.1% (min ₪30); non-tradable securities $10 a month.
- Hedge funds in trust: 0.15%, min ₪50.

## Excellence Trade

- Custody for managed accounts and "accounts that hold assets rather than
  trade actively": 0.1% a quarter, instead of the general 0.6%. A long-term
  investor might count as one; the full tariff plan uses the general rate.
- US: at most 2% of the trade; the 0.3% track adds the broker's fee.
- Traditional (active) mutual funds: ₪16 a trade. Hedge funds 0.15%–0.35%,
  min 50.
- Some sign-up links offer 3 years without the handling fee instead of 2,
  and say commissions may offset the fee afterwards; neither is published by
  Excellence.
