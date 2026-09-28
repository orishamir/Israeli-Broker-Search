# Broker search

Compares what Israeli brokers charge for a given investing pattern: ETFs,
index funds, bonds and stocks, on Tel Aviv or abroad. The fee model is Rust
compiled to WebAssembly; the UI is Svelte 5 + ECharts.

## Layout

- `crates/core` (lib `broker_fees`): `tariffs.rs` holds the real price lists,
  `simulation.rs` runs them over the years, `describe.rs` holds all text shown
  to users (fee names, explanations, caveat kinds, the "About the numbers"
  page, Hebrew names), `yours.rs` holds the user's own plans (changed copies
  of listed plans, and plans of their own) and the editor's fields.
- `crates/wasm`: bindings (wasm-bindgen + tsify), built into the git-ignored
  `web/src/lib/core` by `npm run wasm`.
- `web`: the app; state lives in `src/lib/app.svelte.ts`. Your plans are
  kept in localStorage (`saved.ts`) as core `Plan`s the web side never looks
  inside (`PlanData`).
- `policies`: the brokers' tariff PDFs that `tariffs.rs` is taken from;
  `sources.md` says where each number comes from and how unclear rows were
  read, and `not-modeled.md` lists the fees the app leaves out.

Keep as much logic as possible in Rust; the web side displays what it returns.

## Done means

`cargo test`, `cargo clippy --all-targets` (pedantic, 0 warnings),
`cargo fmt --check`, then in `web/`: `npm run wasm && npm test` and
`npm run lint` (prettier, eslint, stylelint, svelte-check).

**After any Rust change, run `npm run wasm`**, or the dev server and tests
quietly keep running the old core.

## Tariffs

- Investment houses publish only maximums: their "Full tariff" plan. Their
  "Typical offer" plan is what comparison sites list for joining (source and
  date in a caveat), with the full tariff wherever the offer is silent: never
  cheaper than can be shown.
- A plan's US commission tracks: the core uses the cheapest and names it.
  `trade_row` doesn't see tracks; go through `simulate`, `on_track` or
  `describe_fees_for`.
- Each broker's `new_customer_plan` is ticked at first.
- **Nothing is defaulted.** Every plan spells out all its fields, and the fee
  types have no `Default`: a forgotten fee would read as free, the same as a
  waived one. Say `ConversionFee::FREE` / `Markup::NONE` where that's meant,
  and `Markup::MarketRate` where the broker publishes that it converts at the
  market's rate (Interactive: only the market's own spread is left out).
- **Every caveat says how sure it is**, with the constructor that names it:
  `Caveat::published` (the document says so), `reading(text, support)` (an
  unclear row, and what backs the reading), `at_most` (a stand-in on the
  expensive side), `may_cost_more(text, summary)` (the cheap side: the
  summary is flagged under the plan in the results table, and ⚠ means only
  that, everywhere), `not_counted` (a real cost left out, and why).
  `.about_fee(kind)` marks it beside that fee's price; `.when_above(amount)`
  shows it only when the user's biggest order (usually the sale at the end)
  reaches the amount. `.source(&page)` links the page it rests on (the
  broker's site, a comparison site, the exchange, or the tariff itself for
  a reading of its wording); the app shows the links under the caveat, on
  the plan, on the broker and in "About the numbers". Prefer research that
  turns an assumption into a dated reading. Tests enforce a caveat for every
  unpublished or maximum markup, for every offer row that falls back to the
  full tariff, and a source on every reading.

## Tests

- `crates/core/tests/real_tariffs.rs` checks the tariffs against amounts
  worked out by hand from `policies`. A tariff change needs a case there,
  with the arithmetic in a comment.
- Playwright conventions are in `web/tests/CLAUDE.md`.

## Conventions

- Idiomatic Rust: newtypes (`Percent`, `ExchangeRates`), `derive_more`,
  `strum`, `thiserror`. The user is learning Rust from this code.
- Comments explain why, in plain words; no docstrings that restate the name.
- Research an established library before writing a custom solution.
- CSS: tokens on `:root`, container queries (`.card` is a container), and only
  Baseline "widely available" features (stylelint enforces this).
- Prices go through `Price.svelte`, from the core's `PriceText`, so every
  "none" and "not offered" is dimmed the same way.
- All tips go through `Tip.svelte` / `tip.ts`. Popovers need a maximum height
  (Floating UI `size`), or tall ones run off phone screens.
- A button inside a heading becomes part of its accessible name, so put "?"
  beside headings.

## Known traps

- **Svelte:** `$state` deep-proxies objects, so compare by identity only with
  `$state.raw`. Never rewrite a field's text on focus: it breaks selecting and
  typing. A repeated key in a keyed `{#each}` throws at runtime, unseen by
  svelte-check.
- **ECharts:**
  - Never change options on hover: it redraws and loses the click, since a tap
    is a hover and a click at once. Use `dispatchAction` (highlight).
  - On touch screens (`pointer.ts`), ignore `mouseover`, because a scrolling
    finger fires it.
  - Series `id`s make `replaceMerge` keep the old order.
  - Events carry `seriesIndex`, not `seriesId`.
  - Labels wrap unpredictably when some are bolder than others.
- **CSS:**
  - A grid child with a wide table needs `minmax(0, 1fr)`, or the page widens.
  - A chosen button must keep its width: bold wrapped the row.

More traps, tariff research and this machine's tool quirks: `for-ai.md`.

## UI/UX preferences

- **Ask first.** Before building any visible choice (layout, wording, what's
  shown or hidden), offer options with mockups and recommend one, even inside
  an approved task.
- **Users aren't experts.** Explain every jargon term with a visible "?" tip
  that opens on hover or tap. Anything needed to use the app is shown
  outright, not only in a tip.
- **Show only what applies.** Caveats and fees are filtered by the chosen
  security and exchange: nothing about conversion on Tel Aviv. Wording follows
  the choices ("Your bond can be bought…").
- **Phones matter as much as desktop.** Test on a Samsung, not a Pixel.
  Scrolling with a finger must never pick anything.
- **Keep the main thing prominent.** Secondary information stays subtle:
  - the exchange rates are folded into one line;
  - the broker and ID columns are de-emphasized.

  Groups of inputs go in separate cards.
- **One style:**
  - one tooltip style everywhere;
  - outlined buttons;
  - units inside the fields.
- **Charts:**
  - time is shown as an axis, never a slider, and never in 3D;
  - the table and charts are linked, for hovering and pinning;
  - the other lines fade only a little.
- **The dark theme only.** It should feel fast.

## Hebrew

Users are Israeli; English text gives the Hebrew term alongside, e.g. "fees
(עמלות)". Check terms online for what Israelis actually say:
- brokers → בנקים ובתי השקעות, not ברוקרים;
- plans → מסלולים;
- securities → ניירות ערך;
- bonds → אג"ח;
- ETFs → קרנות סל (תעודות סל until 2018); index funds → קרנות מחקות, which
  every tariff prices apart from managed funds (קרנות מנוהלות, mostly free
  to trade), so never say קרן נאמנות for what the app compares.

- Hebrew inside English goes in `<bdi lang="he">`. In Rust strings, join a
  phrase's words with `\u{a0}`: a phrase split across lines reads backwards.
- Label Hebrew names ("In Hebrew", "Also called"), one per line. A list of
  them on one line reads in the wrong order.

## Saving tokens

Never read a whole large file or output (test logs, build output,
`Cargo.lock`, snapshots, `web/src/lib/core`). Narrow it with `grep` (ugrep
here: if a pattern fails, use Python), `head`/`tail`, `sed -n 'X,Yp'` or
`wc -l`. For tests, use
`npx playwright test --reporter=line 2>&1 | tail -30`.

## Not yet

- No deploying or publishing until the app is finished.
