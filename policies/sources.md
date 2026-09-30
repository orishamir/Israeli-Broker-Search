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
- **Excellence Trade:** `תעריפון מלא אקסלנס טרייד.pdf` (July 2026). The site:
  fractional foreign shares, conversion at 2 agorot a dollar, "at most ₪200
  on $10,000"
  ([article](https://www.xnes.co.il/academy/trading/account-fees/), 7 July
  2025). Its joining package
  ([page](https://lp.xnes.co.il/bursa_xnes_trade/join-package2/), 29
  September 2026) states outright: no non-execution fee, no currency fee and
  no custody, and, apart from those, two years without the handling fee,
  "₪180 a year". The two years' rules,
  `תקנון פטור מדמי טיפול לשנתיים אקסלנס טרייד.pdf` (12 October 2025 to 31
  December 2026, linked from
  [its disclosure page](https://www.xnes.co.il/trading/full-disclosure/),
  drawn by JavaScript): "no monthly handling fee" for 24 months, for a first
  deposit of ₪15,000, every other fee in the customer's fee appendix
  unchanged; a deposit above ₪15,000 within 30 days adds a third year (not
  counted). Typical offer (gemeltop, tradingil): 0.07%, min ₪3, for stocks,
  ETFs and index funds; bonds 0.06%, min ₪3; traditional funds ₪16; US 1¢ a
  share, min $6 (or $5 on a partner broker's system); ₪15 a month, which
  "can be offset" by trade fees; minimum deposit ₪10,000 (the two years'
  rules say ₪15,000, tradingil ₪4,500 until 10 October 2026; neither
  counted). broker.co.il and Hasolidit call the ₪15 "דמי ניהול", which reads
  like custody: it's the handling fee.

- **Mizrahi-Tefahot Bank:** `תעריפון מזרחי טפחות - ניירות ערך, ערוצים ישירים, קבוצות אוכלוסייה, מטבע חוץ.pdf`:
  the bank publishes its tariff in parts, merged here in that order: part 4
  (securities, updated 2 April 2024), appendix E (direct channels, 12 August
  2025), appendix A (customer groups, 13 January 2026) and part 5 (foreign
  currency), all linked from
  [its tariff page](https://www.mizrahi-tefahot.co.il/interest-list-small-buisness/)
  as `smallbusiness4.pdf`, `arutzyashir.pdf`, `specials.pdf` and
  `smallbusiness5.pdf`. Online is appendix E's website column (its "PC
  service" column differs a little: conversion 0.1425%); the branch prices
  are part 4's. Custody has no online discount. Appendix A gives the same
  securities discounts to four groups (the young, soldiers, discharged
  soldiers, students): Tel Aviv at 0.4% "not less than the minimum fee",
  half the conversion fee, and a rate benefit of 3 pro mille; pensioners get
  only the conversion discount. Its
  [disclosure of actual fees](https://www.mizrahi-tefahot.co.il/brokerage/amalot/)
  for the first half of 2026 shows Israeli stocks and bonds at 0.13%–0.32%,
  foreign securities at 0.12%–0.34% and custody at 0.08%–0.44% a year by
  portfolio size: well under the tariff, as at every bank.
- **Bank Otsar Hahayal:** `תעריפון הבינלאומי ליחידים (אוצר החייל) 16.09.2026.pdf`,
  the First International Bank's tariff for individuals and small businesses
  of 16 September 2026, linked from
  [Otsar's tariff page](https://www.bankotsar.co.il/private/general/commissionsrate/individuals/)
  (Otsar Hahayal was merged into the First International, which keeps it as
  a brand; the same file is on fibi.co.il). Online is appendix E, which part
  4's note 9 repeats for Tel Aviv: the website, app and trading-systems
  columns agree. Standing orders: part 4, note 13 (index funds on Tel Aviv,
  minimum ₪7) and note 10 (ETFs abroad, minimum $4.50), which give only the
  minimum. That page also links a securities part headed "tariff for large
  businesses" (`ניירות-ערך-31.pdf`, 0.7% and other rates); the full
  document is followed. The
  [Otzar Habitachon club page](https://www.bankotsar.co.il/private/account/armedforces/defense/securitiesbenefits/)
  (the security forces' family, until 30 November 2030): Tel Aviv 0.2% by
  a banker or 0.175% in direct channels, both at least ₪10; abroad 0.3%, at
  least $18; custody waived; no double benefits. The
  [Top Trade page](https://www.bankotsar.co.il/private/capitalmarket/toptrade/)
  (its fine print: ages 18–30, a portfolio of up to ₪200,000, independent
  and not advised; the bank may change or end it): ₪5 an order on Tel Aviv,
  $5 abroad, no custody, no fee for an unexecuted order. Its
  [conscripts' version](https://www.bankotsar.co.il/private/account/types/soldiers/)
  waives everything until 31 December 2026 (not counted). The bank's
  [comparison table for the first half of 2026](https://www.bankotsar.co.il/media/ds1hsij5/%D7%9E%D7%97%D7%A6%D7%99%D7%AA-%D7%A8%D7%90%D7%A9%D7%95%D7%A0%D7%94-2026-%D7%98%D7%91%D7%9C%D7%AA-%D7%9E%D7%99%D7%93%D7%A2-%D7%94%D7%A9%D7%95%D7%95%D7%90%D7%AA%D7%99-%D7%9C%D7%90%D7%AA%D7%A8-%D7%94%D7%90%D7%99%D7%A0%D7%98%D7%A8%D7%A0%D7%98-%D7%94%D7%91%D7%99%D7%A0%D7%9C%D7%90%D7%95%D7%9E%D7%99.pdf)
  shows actual averages of 0.02%–0.19% on Israeli stocks and bonds,
  0.08%–0.23% on foreign securities and custody of 0.25%–0.43% (Israeli)
  and 0.28%–0.37% (foreign) a year.

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

- **Mizrahi-Tefahot's published exchange rates**
  ([page](https://www.mizrahi-tefahot.co.il/brokerage/foreignexchange/), 29
  September 2026): transfers-and-checks buy ₪3.0462 and sell ₪3.0953 against
  a representative ₪3.0720 for the dollar (0.84% under, 0.76% over), and
  ₪3.4539, ₪3.5098 and ₪3.4857 for the euro (0.91% under, 0.69% over). Read
  as a markup of 0.8% each way; the young customer groups' rate benefit of
  3 pro mille takes theirs to 0.5%. The tariff's part 5 converts at the
  dealing room's quote rate or the day's uniform rate, which is this one.
- **The First International's published exchange rates**
  ([page](https://apps.fibi.co.il/Matach/matach.aspx), 29 September 2026,
  the parent bank's, for Otsar Hahayal): transfers-and-checks buy ₪3.0493
  and sell ₪3.1047 around a mid-rate of ₪3.0770 for the dollar (0.9% each
  way; 0.74% under and 1.06% over the representative ₪3.0720), and
  ₪3.4636, ₪3.5266 and ₪3.4951 for the euro. Read as a markup of 0.9% each
  way. The page is served in Windows-1255.

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
  Excellence's joining package says so itself (above), so its plan's caveat
  is published, not a reading. Their ₪15 is taken off by the month's trade
  fees: tradingil says so for IBI and Excellence, broker.co.il for Meitav
  and IBI.
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
- Mizrahi-Tefahot's young customer groups pay 0.4% on Tel Aviv "not less
  than the minimum fee": modelled with the tariff's ₪50 minimum and ₪10,750
  maximum. Whether Online's ₪45 minimum applies too isn't said (up to ₪5 on
  orders under ₪9,600), nor whether the halved conversion fee is half of
  Online's rather than the branch's; the plan takes the better of the halved
  branch fee and Online's, and says both are "at most".
- Otsar Hahayal's standing-order notes give only a lower minimum (₪7, $4.50),
  not the rate; Online's rate is assumed and the plan is flagged "may cost
  more" (0.64% and 0.85% by a banker). ETFs on Tel Aviv aren't in the note,
  so by standing order they're priced as Online.
- The Otzar Habitachon club's 0.3% abroad is above Online's 0.25% on orders
  over $15,600, and its terms allow no double benefits: the club's price is
  used throughout ("at most"). Conversion isn't in the club's terms or on
  Top Trade's page: Online's is used, since the direct-channel prices are
  the tariff's own for every customer on the website or app, not a club
  benefit.
- Top Trade's rows name stocks and bonds (Tel Aviv) and stocks, funds and
  bonds (abroad); ETFs and index funds are read as included, as in the
  tariff's own row (part 4, notes 3 and 14). Its prices are stated for a
  portfolio of up to ₪200,000; a larger one is assumed to pay the same, and
  the plan is flagged.

## Tax and the vehicles

Checked on 30 September 2026 (`vehicles::checked`). `vehicles.rs` holds what
the law says about each kind of account, whoever runs it: the tax on the gain
at the end, and how much may be deposited in a year. The table ranks by
what's left after the tax (`Outcome::after_tax`).

- **Inflation, 2% a year unless the user sets another**
  (`simulation::USUAL_INFLATION`): the middle of the Bank of Israel's target
  of 1–3%, "as it has been since 2003"
  ([the bank's review of the target, November 2024](https://boi.org.il/en/communication-and-publications/press-releases/6-11-24/)).

- **25% of the real gain, everywhere:** the Income Tax Ordinance
  ([Hebrew Wikisource](https://he.wikisource.org/wiki/פקודת_מס_הכנסה)),
  section 91(b)(1): an individual pays "at a rate of no more than 25%" on a
  real capital gain. Section 88 defines the real gain as the gain less the
  inflationary amount, "the part of the gain by which the adjusted cost
  exceeds the cost": so a loss pays nothing, and falling prices never lower
  a cost. [Meitav Trade's guide](https://www.meitav.co.il/trade/capital_market_guide/capital_market_tax/)
  says the same in plain words.
- **Provident fund for investment (קופת גמל להשקעה):** Kol Zchut, a rights
  guide backed by the Ministry of Justice.
  - [The fund](https://www.kolzchut.org.il/he/קופת_גמל_להשקעה): no more than
    ₪83,641 in a calendar year (2026), in all of a person's funds together;
    no tax benefit on deposits; tax is paid only on withdrawal. Menora
    Mivtachim gives the ceiling to the agora, ₪83,641.09.
  - [Its fees](https://www.kolzchut.org.il/he/דמי_ניהול_בקופת_גמל_להשקעה): at
    most 4% of each deposit and 1.05% a year of the balance, under section
    2(a) of the management-fee regulations (`LARGEST_GEMEL_FEE`).
  - [The pension](https://www.kolzchut.org.il/he/פטור_ממס_על_רווחים_בקופת_גמל_להשקעה_המשולמת_כקצבה_לאחר_גיל_60):
    from 60, taken as a monthly pension, the gains are exempt whatever else
    the saver earns; a lump sum, or any withdrawal before 60, pays 25% of the
    real gain. Its example (₪200,000 of real gain) is a test in
    `vehicles.rs`.
- **Study fund (קרן השתלמות):** Kol Zchut, on
  [the fund](https://www.kolzchut.org.il/he/קרן_השתלמות) and on
  [the self-employed's](https://www.kolzchut.org.il/he/קרן_השתלמות_לעובד_עצמאי).
  - A self-employed saver deposits what they like. The gains are exempt if
    no more than ₪20,566 was deposited in the year (2026) and the money is
    taken out after 6 years; "sums deposited beyond this ceiling are charged
    capital gains tax at 25%". Taken out sooner, it's taxed as income, so
    the plan isn't offered for fewer than 6 years (3 for studies or at
    retirement age, not modelled).
  - An employee can't open one without the employer, who deposits up to
    7.5% of the salary (₪14,140.80 a year) against the employee's 2.5%
    (₪4,713.60), to a salary of ₪15,712 a month (2026). The app counts the
    self-employed's fund; a caveat says so.
  - Its fee: "at a rate of no more than 2% a year" of the balance, and
    nothing from deposits, in the management-fee regulations
    ([Hebrew Wikisource](https://he.wikisource.org/wiki/תקנות_הפיקוח_על_שירותים_פיננסיים_(קופות_גמל)_(דמי_ניהול)),
    `LARGEST_STUDY_FUND_FEE`); the same page has the provident fund's 1.05%
    and 4%.
- **Savings policy (פוליסת חיסכון):** no ceiling, 25% of the real gain on
  withdrawal, no pension, and moving to another insurer is a taxed
  withdrawal: [Analyst](https://www.analyst.co.il/articles/savings-policy-and-investment-provident-fund/),
  [Menora Mivtachim](https://www.menoramivt.co.il/general/articles-fellow/gemel-invest-differnces),
  [Bizportal](https://www.bizportal.co.il/longtermsavings/news/article/20038012).
  Menora states a largest fee of 2% of the balance a year. It isn't entered:
  confirm it against the regulation when the policies' prices are.

How the rules were read:

- **What the holdings cost** is what was paid to buy them ("the amount the
  taxpayer spent to acquire the asset", section 88): the order's fee, the
  conversion, and a manager's share of the deposit. Custody and the monthly
  fee aren't cost: "the expenses incurred in that same tax year" (management
  and custody fees for securities) are deducted from the proceeds, by
  regulation 6(a) of the regulations on computing the capital gain on
  securities, 2002
  ([Nevo](https://www.nevo.co.il/law_html/law01/999_087.htm)). Someone who
  sells only at the end deducts the last year's; the earlier years' had no
  sale to come off (`simulate_prices`). In a fund the gain is what comes out
  less what went in, so every fee comes off it.
- **Prices** rise at the inflation the user gives, the same every month.
- **The ceiling** is checked on each year of the period as if it were a
  calendar year, and rises with prices as the law raises it each January.
  So does the study fund's tax-free amount, which can fall if prices do.
- **A study fund's taxed part** is, of each year's deposits, the share
  beyond that year's tax-free amount: that share of what's bought in the
  year, with its gains, is taxed at the end like a gain at a broker.
- **A saver too young for the pension** at the end of the period is taxed
  as for a lump sum: the money is counted then, not left to wait.

## Funds and policies

Checked on 30 September 2026 (`funds::checked`). `funds.rs` lists each kind
of fund as a whole, not each company: a saver's fee is agreed person by
person, and the companies' averages are within 0.2% of each other.

The fees are the Capital Market Authority's, from the monthly reports of
the funds and insurers, as its open data gives them (data.gov.il, the
datasets `gemelnet` and `insurance`, shown on
[Gemel Net](https://gemelnet.cma.gov.il/) and
[Bituach Net](https://bituachnet.cma.gov.il/)). `gemel-net.py` here works
them out; run it for the report of August 2026 (`202608`) to get the numbers
below. Each track's average fee is weighted by the money in it, and tracks
that report no fee (new ones) are left out.

- **Provident fund for investment:** the tracks classified קופת גמל להשקעה
  and open to everyone (כלל האוכלוסיה), 123 tracks of 11 companies, ₪99
  billion. All together 0.6168% of the balance and 0.0010% of deposits:
  "Average fee" is 0.62% and nothing on deposits. By company, Harel is the
  cheapest at 0.5519% ("Cheapest company", 0.55%) and Mor the dearest at
  0.7188% ("Dearest company", 0.72%). The funds of one employer's or one
  sector's workers (teachers, the electric company) charge less and aren't
  open to others. "Legal maximum" is the regulations' (above).
- **Study fund:** the tracks classified קרנות השתלמות and open to
  everyone, without the self-managed ones (names with ניהול אישי or IRA,
  whose savers pay a broker's trade fees on top): 131 tracks of 11
  companies. All together 0.6125% of the balance and nothing from deposits
  ("Average fee", 0.61%); Migdal the cheapest at 0.5281% (0.53%), Mor the
  dearest at 0.6981% (0.70%).
- **The figure is a yearly one:** it changed in January 2025 and January
  2026 for nearly every fund and in no other month, so it's taken as what
  savers paid over the year before. The app says "data of August 2026" and
  that it last changed in January.
- **Savings policy:** Bituach Net reports the insurers' investment policies
  together, as "policies sold since 2004": 168 tracks of 8 insurers, ₪351
  billion, 0.9393% of the balance, and 1.7663% of deposits in the 141 that
  report a fee on deposits. They include managers' insurance (ביטוח מנהלים),
  a pension product with a fee on deposits; savings policies aren't reported
  apart. "Average fee" is that 0.94% with nothing on deposits, shown as our
  reading, since Menora Mivtachim calls the fee on the balance a savings
  policy's only cost. "Highest fee" is the 2% of the balance that Menora and
  Bizportal give as the most; the regulation behind it wasn't read.
- **What a fund's own investing costs:** the regulations on direct expenses
  ([Hebrew Wikisource](https://he.wikisource.org/wiki/תקנות_הפיקוח_על_שירותים_פיננסיים_(קופות_גמל)_(הוצאות_ישירות_בשל_ביצוע_עסקאות)))
  cap outside managers' fees and the like at 0.25% of a fund's assets a
  year. Not counted, like an ETF's own fee.

## Money for the short term

Checked on 1 October 2026 (`short_term::deposits::checked`,
`short_term::money_market::checked`). `short_term.rs` compares a sum at
the start, and perhaps an amount at the start of every month, kept for 1 to
60 months: in a money market fund, or at a bank on a fixed-rate deposit,
which takes the sum alone. Everything earns about the Bank of Israel's
rate; the comparison is what reaches the saver after what the place keeps
and the tax.

- **Today's rate, 3.25%** (`short_term::TODAYS_RATE`), since the Monetary
  Committee's cut of 1 September 2026, the third in a row
  ([ynet](https://www.ynet.co.il/economy/article/cdo59v3t1),
  [Bizportal](https://www.bizportal.co.il/general/news/article/20040979)).
  The cut before it, on 6 July, took the rate to 3.5%, so it was 3.5% all
  August ([TheMarker](https://www.themarker.com/news/macroeconomics/2026-07-06/ty-article-live/0000019f-376f-d22f-a9ff-f77f73af0000)).
  A fund is taken to earn the rate the saver expects on average over the
  months, today's unless they say otherwise.
- **Deposits:** the Bank of Israel's comparison of the banks' deposit rates
  ([קו המשווה](https://www.boi.org.il/information/bank-paymnts/financial-education/campaigns/boi-equator/deposit/)),
  from the Excel behind it
  ([g060a.xls](https://www.boi.org.il/boi_files/Pikuah/g060a.xls), sheet
  L7.6.1a): each bank's average yearly rate on the fixed-rate, unlinked
  shekel deposits households opened with it in a month, by term, from June
  2024. `deposit-rates.py` here prints them (`uv run --with openpyxl python
  policies/deposit-rates.py 2026-08`), rounded to two places; the file
  says 14 September 2026 and ends with August.
  - Ten banks and the whole system ("מערכת", the banks' average). Otsar
    Hahayal isn't among them.
  - A 0 is a rate left out: the page says it publishes none for a bank's
    segment with fewer than ten deposits in the month, or a kind of rate
    under 5% of the month's deposits. The app shows no rate, and doesn't
    compare that bank for that term.
  - The terms are read as each taking the ones longer than the term before
    it, up to its own: "עד חודש", "חודש עד 3 חודשים", "3 חודשים עד 6
    חודשים", "6 חודשים עד שנה", "שנה עד שנתיים", "שנתיים עד 3 שנים", "3 שנים
    עד 5 שנים". The daily and weekly terms are left out.
  - The rates are shown as published for August, dated, and every deposit
    is flagged as August's, before the September cut (the user's choice,
    over moving each bank by the cut).
  - A rate is a year's: part of a year earns that part of the year's
    interest, and a whole year's interest joins the deposit.
  - [Calcalist](https://www.calcalist.co.il/market/article/bytylkzgbl)
    (13 November 2025): the Bank of Jerusalem and Mizrahi-Tefahot are the
    only banks that open a deposit for someone without a current account
    with them; One Zero offered 5.5% a year to those joining its premium
    tracks. Its 6.00% for "6 months to a year" in August, with no rate for
    any other term, is read as such an offer.
- **Money market funds:** Maya, the Tel Aviv Stock Exchange's list of
  mutual funds ([maya.tase.co.il](https://maya.tase.co.il/he/funds/mutual-funds)),
  with each fund's fees as its manager reports them. `money-market-funds.py`
  here works them out from Maya's API (the list, 30 funds a page). The
  shekel money market funds (classified כספית שקלית, with and without
  corporate bonds) were 44 funds holding ₪184,638 million, rates of 30
  September 2026. The ₪213 billion the news gives
  ([Bizportal](https://www.bizportal.co.il/mutualfunds/news/article/20041406))
  includes the 27 funds in dollars and euros and the 2 fixed-term ones
  (כספית מתחדשת).
  - A fund costs its saver the management fee and the trustee's fee, both
    taken from its assets. The distribution fee (עמלת הפצה, 0.1% or 0.35%)
    is paid by the manager to the bank out of its fee.
  - "Average fee": both fees, weighted by assets, 0.169%. "Cheapest fund":
    Ayalon's Dolphin (דולפין כספית שקלית), 0% and 0.01% for the trustee,
    under a year old, [its page](https://maya.tase.co.il/he/funds/mutual-funds/5141098)
    showing no load (שיעור הוספה 0%). Barak's fund also charges 0.01%, but
    loads 0.1% on buying (its purchase price is above its redemption
    price), so it isn't the cheapest. "Dearest fund": Meitav's (מיטב
    כספית), 0.25% and 0.01%.
  - Custody: banks may not charge it on a money market fund or on makam, a
    Bank of Israel rule from 1 January 2013
    ([its announcement](https://boi.org.il/publications/pressreleases/%D7%94%D7%9E%D7%A4%D7%A7%D7%97-%D7%A2%D7%9C-%D7%94%D7%91%D7%A0%D7%A7%D7%99%D7%9D-%D7%A0%D7%95%D7%A7%D7%98-%D7%91%D7%A6%D7%A2%D7%93%D7%99%D7%9D-%D7%9C%D7%94%D7%A4%D7%97%D7%AA%D7%AA-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-%D7%A2%D7%91%D7%95%D7%A8-%D7%9E%D7%A9%D7%A7%D7%99-%D7%94%D7%91%D7%99%D7%AA-%D7%95%D7%94%D7%A2%D7%A1%D7%A7%D7%99%D7%9D-%D7%94%D7%A7%D7%98%D7%A0%D7%99%D7%9D/),
    28 November 2012), which Leumi's and First International's price lists
    here repeat ("לא ניתן לגבות דמי ניהול פקדון ניירות ערך עבור מילווה קצר
    מועד או עבור קרן כספית").
  - Trade fees: Leumi's price list leaves mutual funds out of its trade fee
    row ("למעט קרנות נאמנות"), read as none at a bank. At an investment house
    there can be one, not counted: Excellence's full tariff charges ₪16 a
    trade on managed funds, and IBI trades only some managers' funds free.
  - What a fund earns: taken as the rate less its fees. Maya's 12-month
    returns to September 2026 for the largest funds were 4.14%–4.30% after
    fees, while the rate came down from 4.5% (November 2025) to 3.25%.
- **The tax:** the Income Tax Ordinance
  ([Hebrew Wikisource](https://he.wikisource.org/wiki/פקודת_מס_הכנסה)).
  - A deposit: section 125ג(ג)(1), interest paid on an asset not linked to
    the index is taxed at 15% (and 125ג(א) counts a discount, as on makam,
    as interest). All of the interest, inflation or not.
  - A fund: section 91(ב)(1), 25% of the real capital gain, as for any
    security; a unit of an exempt fund (קרן נאמנות פטורה) is sold under
    it (91(ב1)(1ב)). The 15% of section 91(ב)(3) is for unlinked bonds,
    commercial paper and loans themselves, not a fund's units.
