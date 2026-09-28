# Where the numbers come from

Checked on 28 September 2026 (`tariffs::checked`, shown in the app).
`tariffs.rs` is taken from these; each broker's `source_url` points at its
main document. Every reading of an unclear row below is a `Caveat::reading`
in `tariffs.rs`, with what supports it, and the app shows it under "Our
reading"; a stand-in with nothing to go on is `at_most` or `may_cost_more`.

## Full tariffs and typical offers

Israeli investment houses publish a *full tariff* (תעריפון מלא): the most they
may charge. New customers are offered far less, agreed in the phone call that
opens the account, and the offer isn't published by Meitav, Excellence or IBI.
So those three have two plans each:

- **Full tariff:** the document's prices, the worst case.
- **Typical offer:** what comparison sites list as the terms for joining
  (gemeltop.co.il, 6 September 2026, cross-checked with tradingil.co.il),
  plus what the broker's own site promises. Anything neither states is priced
  as the full tariff, and the plan's caveats say which.

The Tel Aviv Stock Exchange publishes each member's *actual* average fees
([calculator](https://market.tase.co.il/he/market_data/trading_fees), data of
June 2026). They confirm the typical offers: Tel Aviv stocks and bonds cost on
average 0.073% (Altshuler), 0.082% (Meitav), 0.085% (Excellence and IBI) for a
₪100–200 thousand portfolio, against full tariffs of 0.15%–0.4%. The same data
confirmed two unclear tariff rows: Altshuler's custody is 0.15% a *year*, and
Meitav's 0.15% is a quarter (0.6% a year).

## Per broker

- **Altshuler Shaham Trade:** `תעריפון מלא אלטשולר שחם טרייד.pdf` (last changed
  25 May 2025, identical to the site's); new-customer offer from
  [trading_benefits](https://www.as-invest.co.il/trade/trading_benefits/) and
  its rules, `תקנון מבצע הצטרפות אלטשולר שחם טרייד.pdf` (January 2026).
- **Bank Leumi:** `תעריפוני עמלות לאומי.pdf`, dated 29 June 2026, still the
  current one. Leumi Trade is the bank's online channel: its prices are the
  tariff's appendix E (direct channels).
- **Interactive Israel:** `אינטרקטיב ישראל תעריפון-עמלות-23.09.2026-1.pdf`, the
  current one, and [its commission page](https://www.inter-il.com/commission/):
  conversion ₪10 for up to ₪500,000 "and at the live rate" (ולפי שער רציף),
  no custody or handling fee, no minimum deposit. Tel Aviv trading is for
  institutional clients only
  ([page](https://www.inter-il.com/israeli-stock-market/)). The account is
  held at Interactive Brokers, whose
  [spot currencies page](https://www.interactivebrokers.com/en/pricing/commissions-spot-currencies.php)
  says it passes market quotes through with "no mark-ups on quotes" and
  charges a separate commission, and that its automatic conversion service
  instead moves the rate by up to 0.03%; ILS is among its 28 spot currencies.
- **IBI:** `תעריפון עמלות אי.בי.אי.pdf`, dated 1 July 2026. The site: minimum
  deposit ₪15,000, fractional US shares, no trade fee on managed funds, and
  "no custody on any fund in the system, index funds included", as a
  benefit for every IBI TRADE customer
  ([page](https://www.ibi.co.il/solutions/zero-balance-managed-funds/)).
  Typical offer (gemeltop): 0.08%, min ₪2.35; abroad min $7.50; ₪15 a month.
- **Meitav Trade:** `תעריפון מלא מיטב טרייד.pdf`, version 01/2025, linked as the
  current one. The site: no custody fee, no conversion fee, minimum deposit
  ₪5,000. Typical offer (gemeltop, tradingil): index ETFs 0.07%, other Tel
  Aviv securities 0.08%, min ₪4.65; US 1¢ a share, min $5; ₪15 a month, free
  for the first 2 years.
- **Excellence Trade:** `תעריפון מלא אקסלנס טרייד.pdf` (July 2026). The site: 2
  years without the handling fee, fractional foreign shares, conversion at 2
  agorot a dollar
  ([article](https://www.xnes.co.il/academy/trading/account-fees/)). Typical
  offer (gemeltop, tradingil): 0.07%, min ₪3; US 1¢ a share, min $6; ₪15 a
  month; minimum deposit ₪10,000.

## Readings of unclear rows

- A typical offer prices only what the comparison sites say it does: Tel
  Aviv stocks and ETFs, and US stocks and ETFs. Everything else (Tel Aviv
  bonds and index funds, US bonds and funds, Europe) is the full tariff's,
  even where the tariff prices it in the same row as the discounted
  securities, and the plan's caveats say so.
- The typical offers' only holding cost is the monthly handling fee, as the
  comparison sites list it (banks: "plus custody"); they charge no custody.
- IBI's full tariff charges custody of 0.1% a quarter on the account, but
  none on Tel Aviv index funds, per its site's promise above. Foreign funds
  aren't "in the system" (they're bought through the foreign broker), so they
  keep the 0.1%.
- US commission tracks: the customer picks one when opening the account, and
  the app compares the plan on the cheapest for the user's inputs, naming it.
  Excellence's tracks (row 6, "securities abroad, US only") price every US
  security, bonds and funds included; Altshuler's, IBI's and Meitav's only
  stocks and ETFs.
- Leumi, Meitav, Altshuler and Excellence charge custody on mutual fund units
  too: their tariffs say so ("including units of mutual funds").
- Leumi 18+: benefits don't stack, and each fee is the better of the group's
  and the online price (the tariff's first page).
- Leumi's standing-order price is only for buying (note 10 on page 30).
