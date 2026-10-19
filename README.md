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
questions while accepting the advanced ones, and educates on the way: when
it chooses something for you it says what that thing is, in the words your
bank's app and the ads use, why it fits your circumstances, and it draws
how your money would have looked in the choices it didn't pick.

## Two routes, one page

**The guide.** Answer a few questions: what the money is for, how much now
and a month, when you need it (and, for a goal three to seven years away,
whether the date can move; with no date, whether you already have a
cushion). The page answers "מה שמתאים לכסף כזה": **what** it chose, in the
words of your bank's app and the ads (a קרן סל, a קרן מחקה, a קרן כספית, a
פיקדון, an אג״ח ממשלתית לפדיון, a קופת גמל להשקעה...), **why** it fits your
circumstances, and **how** to do it step by step; and it draws what the same
money would have come to in the kinds it didn't choose, with the S&P 500's
past range (since 1926) beside it and where every shekel goes (deposits,
gain, fees, tax). Each term opens a card that explains it. A ladder shows
the same money at 1, 3, 5, 10 and 20 years, and chips start from a reason
people save.

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
Trade, Bank Leumi (including Leumi Trade and Pepper), Excellence Trade, IBI,
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
ECharts. `CLAUDE.md` describes the layout and the conventions, and
`REDESIGN.md` the plan the page was built from.

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
