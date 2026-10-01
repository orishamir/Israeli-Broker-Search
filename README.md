# Broker fees, compounded

**https://orishamir.github.io/Israeli-Broker-Search/**

What saving in Israel comes to, after fees and tax. The site is in Hebrew,
with two calculators, since only what earns the same thing can be ranked.

**Investing for the long term (השקעה לטווח ארוך).** Pick what you buy (an
ETF, an index fund, a bond or a stock), where (Tel Aviv, the USA or
Europe), how much and for how long, and the app runs every plan's price
list over those years, month by month: purchases, currency conversions,
custody, handling fees, and the sale and the tax at the end. Beside the
brokers it runs the same deposits in a provident fund for investment (קופת
גמל להשקעה), a study fund (קרן השתלמות) and a savings policy (פוליסת
חיסכון), each by its own tax rules. It ranks the plans by what you'd have
left after tax, shows what each fee took, and states each plan's fees as a
yearly cost, the way a fund states its management fee (דמי ניהול).

Banks and investment houses (בנקים ובתי השקעות) covered: Altshuler Shaham
Trade, Bank Leumi (including Leumi Trade and Pepper), Excellence Trade, IBI,
Interactive Israel, Meitav Trade, Mizrahi-Tefahot Bank and Bank Otsar
Hahayal. Investment houses publish only their full tariffs; their "Typical
offer" plans are what comparison sites list for new customers, and every
number says how sure it is. A fund's fee is agreed person by person, so each
kind of fund is listed by what savers pay on average, from the regulator's
data, and by the cheapest and dearest company. You can also copy any plan
and change the fees you were offered.

**Saving for the short term (חיסכון לטווח קצר).** A sum, and perhaps an
amount every month, kept for 1 to 60 months: in a money market fund (קרן
כספית), at the funds' reported fees, or, the sum alone, on a bank's
fixed-rate deposit (פיקדון בריבית קבועה), at the rate the Bank of Israel
publishes for each bank, or at a rate you were offered.

## About the numbers

Every price comes from the broker's own tariff document or site; the documents
are in `policies/`, and `policies/sources.md` says where each number comes
from and how unclear rows were read. The funds' fees and the banks' deposit
rates are worked out from open data by the scripts beside them
(`gemel-net.py`, `money-market-funds.py`, `deposit-rates.py`).
`policies/not-modeled.md` lists the fees and taxes left out and why. Each
calculator's "About the numbers" page explains its method and links every
source.

Tariffs and rates change several times a year. This is a comparison tool, not
financial advice: check the current terms before opening an account.

## Development

The fee model is Rust, compiled to WebAssembly; the UI is Svelte 5 with
ECharts. `CLAUDE.md` describes the layout and the conventions.

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
