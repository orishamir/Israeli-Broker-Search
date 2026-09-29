# Broker fees, compounded

**https://orishamir.github.io/Israeli-Broker-Search/**

What Israeli brokers' fees cost you over the years. Pick what you buy (an ETF,
an index fund, a bond or a stock), where (Tel Aviv, the USA or Europe), how
much and for how long, and the app runs every broker's price list over those
years, month by month: purchases, currency conversions, custody, handling fees
and the sale at the end. It ranks the plans by what you'd have left, shows what
each fee took, and states each plan's fees as a yearly cost, the way a fund
states its management fee (דמי ניהול).

Banks and investment houses (בנקים ובתי השקעות) covered: Altshuler Shaham
Trade, Bank Leumi (including Leumi Trade and Pepper), Excellence Trade, IBI,
Interactive Israel and Meitav Trade. Investment houses publish only their full
tariffs; their "Typical offer" plans are what comparison sites list for new
customers, and every number says how sure it is. You can also copy any plan
and change the fees you were offered.

## About the numbers

Every price comes from the broker's own tariff document or site; the documents
are in `policies/`, and `policies/sources.md` says where each number comes
from and how unclear rows were read. `policies/not-modeled.md` lists the fees
left out and why. The app's "About the numbers" page explains the method and
links every source.

Tariffs change several times a year. This is a comparison tool, not financial
advice: check the broker's current tariff before opening an account.

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
