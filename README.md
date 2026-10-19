# Broker fees, compounded

**https://orishamir.github.io/Israeli-Broker-Search/**

What saving in Israel comes to, after fees and tax. The site is in Hebrew.

## What it's for

It began as a comparison of banks and investment houses for investing in
ETFs, and a way to see compound interest, and how fees eat into it, in
lines and pies, across exchanges. Research showed that isn't the only
comparison worth making: in Israel the difference between a mutual fund
(קרן נאמנות) and an ETF (קרן סל) can matter more than the choice of
broker. And two kinds of people come: newcomers who don't know the
strategies (a money market fund, a bond ETF, a stock ETF, a mutual fund, a
CPI-linked deposit) and want to learn by example, and people who know
exactly what they want to compare and see. So the site guides through
questions while accepting the advanced ones, and educates on the way. It
doesn't pretend to know your whole situation: it says what is clear and how
sure it is, and puts the choices only you can make in front of you, at the
point they matter, with what each would leave you. Whatever it names, it
says what the thing is, in the words your bank's app and the ads use, and
what to search for.

## Two routes, one page

**The guide.** Say what you have now, what you'll add each month, when
you'll need the money, and where you bank. The page answers on a few lines:
the type of place (safe, or a stock index), the way (a deposit, a money
market fund, a bond of the State of Israel, an ETF on the S&P 500, a
provident fund...) and where. Each line is either stated, with how sure the
site is (כאן קשה לטעות, ההבדל קטן, תלוי במה שחשוב לכם), or a decision only
you can make, shown in place as options side by side with what each would
leave you; nothing is asked unless the answer changes the outcome. Below
it: every option on one chart of what it would come to on your date, after
tax and in today's money, beside what you put in and what a current account
would leave; how to do it, with what to search for in your bank's app; how
much of the gain fees and tax take; and what changes if the date moves.
Each term opens a card that explains it, and chips start from a reason
people save. The guide's answer is being rebuilt to this design
(`REDESIGN.md`); the live page still shows the previous one.

**The workbench.** Switch on "כל האפשרויות" when you know what you want to
compare: what to buy (the S&P 500, another index or stock, bonds, short-term
money), the ways to hold it side by side (an index fund through a bank, an
ETF in Tel Aviv or in the USA, a provident fund's index track), the
exchange, the rates you expect, growing deposits, how often you buy, sell
or keep, a pension, and your own plans copied from any listed one. The
full table ranks by what is left after tax, and the charts show growth,
what fees took, the breakdown by fee and the result by deposit.

## What is compared

For the long term, every plan's price list runs over the years, month by
month: purchases, currency conversions, custody, handling fees, and the sale
and the tax at the end; and what the fund holding the index keeps back
before you see it. Beside the brokers it runs the same deposits in a
provident fund for investment (קופת גמל להשקעה), a study fund (קרן השתלמות)
and a savings policy (פוליסת חיסכון), each by its own tax rules. For the
shorter term it compares a money market fund (קרן כספית), a bank's
fixed-rate deposit (פיקדון בריבית קבועה) and a shekel government bond held
to maturity (אג״ח ממשלתית לפדיון), from the Bank of Israel's published
rates. Only what earns the same thing is ranked: the safe kinds among
themselves, the index kinds among themselves.

Banks and investment houses (בנקים ובתי השקעות) covered: Altshuler Shaham
Trade, Bank Leumi (including Leumi Trade and Pepper), Bank Hapoalim, Bank
Discount, the First International Bank (Beinleumi), Excellence Trade, IBI,
Interactive Israel, Meitav Trade, Mizrahi-Tefahot Bank and Bank Otsar
Hahayal. Investment houses publish only their full tariffs; their "Typical
offer" plans are what comparison sites list for new customers, and every
number says how sure it is. A fund's fee is agreed person by person, so each
kind of fund is listed by what savers pay on average, from the regulator's
data, and by the cheapest and dearest company. You can also copy any plan
and change the fees you were offered.

## About the numbers

Every price comes from the broker's own tariff document or site; the documents
are in `policies/`, and `policies/sources.md` says where each number comes
from, how unclear rows were read, and the reasons behind each of the guide's
rules. The funds' fees, the banks' deposit rates, the government yield
curve, the index's history and what funds holding the index keep back are
worked out from open data by the scripts beside them (`gemel-net.py`,
`money-market-funds.py`, `deposit-rates.py`, `yield-curve.py`,
`sp500-history.py`, `index-tracking.py`; sources: the regulator, the Bank of
Israel, the stock exchange, Maya, Robert Shiller's data). The S&P 500's
history ends in June 2023 and is the past, not a promise.
`policies/not-modeled.md` lists the fees and taxes left out and what the
guide doesn't do (mixes, personal advice, pension funds). The page's "About
the numbers" explains the method and links every source.

Tariffs and rates change several times a year. This is a comparison tool, not
financial advice: check the current terms before opening an account.

## Development

The fee model is Rust, compiled to WebAssembly; the UI is Svelte 5 with
ECharts. `CLAUDE.md` describes the layout, the conventions and the design
rules in short; `REDESIGN.md` has the design rules in full and the plan of
record.

```sh
cargo test                       # the core: tariffs, economics, bindings
cd web && npm ci && npm run wasm # build the WebAssembly into web/src/lib/core
npm run dev                      # the app, at http://localhost:5173
npm test                         # unit tests, then the browser tests
```

Every push to `main` runs the tests and deploys the site to GitHub Pages
(`.github/workflows/deploy.yml`).

## License

MIT (see `LICENSE`). The tariff documents in `policies/` are the brokers'
own publications, kept so that every number can be checked against its
source.
