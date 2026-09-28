# Where the numbers come from

Checked on 29 September 2026 (`tariffs::checked`, shown in the app).
`tariffs.rs` is taken from these; each broker's `source_url` points at its
main document, and each caveat links the page it rests on (`Caveat::source`),
which the app shows under the caveat, on the plan and broker, and grouped by
broker in "About the numbers". Every reading of an unclear row below is a `Caveat::reading`
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
  its rules, `תקנון מבצע הצטרפות אלטשולר שחם טרייד.pdf` (January 2026). Its
  [FAQ](https://www.as-invest.co.il/trade/faq/): trading on the Israeli and
  US exchanges only, so nothing in Europe is offered, the tariff's "foreign
  bonds and funds" row included. The exchange's June 2026 averages show its
  customers paying 0% custody, which backs reading the offer's "no
  management fees" as covering custody.
- **Bank Leumi:** `תעריפוני עמלות לאומי.pdf`, dated 29 June 2026, still the
  current one. Leumi Trade is the bank's online channel: its prices are the
  tariff's appendix E (direct channels); customer groups (18+) and Pepper
  are appendix A. [Pepper's site](https://www.pepper.co.il/) (its FAQ) states
  the same ₪4 and $4 up to ₪30,000 and $8,000 an order, says larger orders
  are priced "by the tariff", and custody of 0.15% a quarter. Its March
  2026 package (a year without trade, minimum and currency fees, for
  customers who move their salary) isn't counted.
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
  deposit ₪15,000, fractional US shares, no trade fee on managed funds of 12
  fund managers (managed: "not index funds, ETFs or foreign funds listed on
  the exchange"), and "no custody on any fund in the system, index funds
  included", as a benefit for every IBI TRADE customer
  ([page](https://www.ibi.co.il/solutions/zero-balance-managed-funds/), drawn
  by JavaScript: read it in a browser). Typical offer (gemeltop): 0.08%, min
  ₪2.35; abroad min $7.50; ₪15 a month. tradingil adds: bonds 0.08%, min
  ₪2.35; index funds 0.08%, min ₪5; US 1¢ a share, min $7.50; the handling
  fee taken off by the month's trade fees; two free years (its own deal,
  not counted); no custody, no conversion fee (spread only). The exchange's
  tariff listing shows IBI's custody as ₪50 a month, which is the document's
  handling fee; the document's 0.1% a quarter is followed.
- **Meitav Trade:** `תעריפון מלא מיטב טרייד.pdf`, version 01/2025, linked as the
  current one. The exchange's tariff listing (July 2026) has 0.3% for its
  trades outside the US against the document's 0.25%; the document is
  followed. The site: no custody fee, no conversion fee, minimum deposit
  ₪5,000. Typical offer (gemeltop, tradingil): index ETFs 0.07%, stocks and
  bonds 0.08%, min ₪4.65; managed funds free; US 1¢ a share, min $5 (IB
  system) or $7.50 (ViewTrade); ₪15 a month, free for the first 2 years,
  taken off by the month's trade fees (broker.co.il).
- **Excellence Trade:** `תעריפון מלא אקסלנס טרייד.pdf` (July 2026). The site: 2
  years without the handling fee, fractional foreign shares, conversion at 2
  agorot a dollar, "at most ₪200 on $10,000"
  ([article](https://www.xnes.co.il/academy/trading/account-fees/), 7 July
  2025). Typical offer (gemeltop, tradingil): 0.07%, min ₪3, for stocks,
  ETFs and index funds; bonds 0.06%, min ₪3; traditional funds ₪16; US 1¢ a
  share, min $6 (or $5 on a partner broker's system); ₪15 a month, which
  "can be offset" by trade fees; minimum deposit ₪10,000 (₪4,500 until 10
  October 2026 through tradingil, not counted).

## Conversion markups

Every investment house's tariff says "up to 0.7%"; what is actually charged,
and the banks' unpublished markups, come from these:

- **tradingil.co.il's comparison of conversion costs** (September 2026):
  Altshuler 0.7%; IBI 0.7% below $15,000 and 0.5% above; Meitav 2.1 agorot a
  dollar (about 0.7% at ₪3); Excellence 2 agorot; Interactive 0.002% with
  the ₪10 minimum. The 0.7% ones are readings, at the tariffs' maximum.
- **IBI's currency FAQ**
  ([page](https://www.ibi.co.il/solutions/trading/forms-important-information/foreign-currency-faq/)):
  no fee, only a spread written in each customer's fee appendix, worked as
  an example at 0.7%.
- **Altshuler's currency page**
  ([page](https://www.as-invest.co.il/trade/currency_exchange/)): no fee, the
  rate shown is the market's plus the markup and is final.
- **Excellence's article** (July 2025): 2 agorot a dollar, at most ₪200 on
  $10,000, used for the full tariff too, on top of its 0.1% fee (at most).
- **Leumi's published exchange rates**
  ([page](https://www.bankleumi.co.il/vgnprod/shearim.asp), 28 September
  2026): transfers-and-checks buy ₪3.0395 and sell ₪3.0960 against a
  representative ₪3.0660 for the dollar (0.86% under, 0.98% over), and
  ₪3.4575, ₪3.5218 and ₪3.4877 for the euro. Read as a markup of 0.9% each
  way for every Leumi plan, a dated reading instead of the earlier "not
  published, counted as 0". Its digital channels can quote an agreed rate
  during trading hours, which may be a little better.

## Readings of unclear rows

- A typical offer prices only what the comparison sites list. gemeltop lists
  Tel Aviv stocks and ETFs and the US minimum; tradingil (September 2026)
  adds Tel Aviv bonds at all three, index funds at Excellence and IBI, and
  spells out 1¢ a share in the US. Everything else (Meitav's index funds, US
  bonds and funds, Europe) is the full tariff's, even where the tariff prices
  it in the same row as the discounted securities, and the plan's caveats
  say so.
- The typical offers' only holding cost is the monthly handling fee, as the
  comparison sites list it (banks: "plus custody"); they charge no custody.
  Their ₪15 is taken off by the month's trade fees: tradingil says so for
  IBI and Excellence, broker.co.il for Meitav and IBI.
- Leumi 18+ pays 0.35% on Tel Aviv bonds (appendix A) with the branch's
  bounds (₪27, ₪7,000), or Online's 0.4% (₪26, ₪6,300), whichever is less.
  Modelled as 0.35% within Online's bounds, which is within ₪1 of that on
  any order.
- Pepper's ₪4 row names stocks, T-bills and bonds. ETFs are read as included,
  since the tariff's own "stocks and bonds" row counts ETFs and index funds
  in (part 4, footnote 4); index funds, which aren't traded on the exchange,
  are given Online's price.
- IBI's pages disagree on the minimum deposit: its fund page and the
  comparison sites say ₪15,000, the legal note under its trading Q&A
  ₪20,000. ₪15,000 is used.
- "Not offered" says the most general gap that's true, from the plan's rows:
  nothing on that exchange (Altshuler in Europe, Interactive on Tel Aviv),
  the security nowhere (index funds at Interactive), or just that pair.
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
