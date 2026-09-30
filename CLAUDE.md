# Broker search

Compares what Israeli brokers charge for a given investing pattern: ETFs,
index funds, bonds and stocks, on Tel Aviv or abroad; and, beside them, what
the same deposits come to in a provident fund for investment, a study fund
or a savings policy, after fees and tax. A provident fund for savings is
left out on purpose (`policies/not-modeled.md`). The fee model is Rust compiled to WebAssembly;
the UI is Svelte 5 + ECharts.

## Layout

- `crates/core` (lib `broker_fees`): `tariffs.rs` holds the real price lists,
  `simulation.rs` runs them over the years (and states an outcome as a
  yearly cost like a fund's fee, in today's money, and over a range of
  deposits: `sweep`, with `Sweep::around` for where the cheapest plan stops
  being cheapest), `vehicles.rs` holds what the law says about each kind of
  account (a brokerage account, a provident fund for investment, a study
  fund, a savings policy): the tax on the gain at the end, the ceiling on
  deposits or their tax-free part, and how long the money is locked, the
  same whoever runs the money, while fees belong to the plan (a fund's is a
  `ManagementFee`). `funds.rs` lists the kinds of fund like brokers
  (`BrokerKind::Funds`), by what savers pay; `listed()` is the brokers, then
  the funds. The comparison ranks by what's left after tax
  (`Outcome::after_tax`) and says why a plan can't be used (`NotOffered`:
  the security isn't sold, a year's deposits pass a fund's ceiling, or its
  money is still locked at the end).
  `describe.rs` holds all text shown
  to users (fee names, explanations, caveat kinds, the "About the numbers"
  page, Hebrew names), in Hebrew and English (see "Hebrew only" below),
  `yours.rs` holds
  the user's own plans (changed copies of listed plans, and plans of their
  own) and the editor's fields, `examples.rs` the ready-made patterns behind
  the example chips.
- `crates/wasm`: bindings (wasm-bindgen + tsify), built into the git-ignored
  `web/src/lib/core` by `npm run wasm`.
- `web`: the app; state lives in `src/lib/app.svelte.ts`. The plans to
  compare are ticked in a list that opens over the inputs from the "What's
  compared" card (`PlanSheet.svelte`; the whole screen on a phone), one
  folded line per broker, then your plans. The page's own
  words are in `src/lib/text/he.ts` (typed as `en.ts`), read through
  `text.ts` (`t`). Your plans are kept in localStorage (`saved.ts`) as
  core `Plan`s the web side never looks inside (`PlanData`). The charts' options are pure functions in
  `growth-chart.ts`, `crossover-chart.ts` (the chart by deposit) and
  `fee-breakdown.ts`; the components only wire them to ECharts and the
  state, so the options are unit-tested without a browser. `link.ts` puts a
  comparison into the page's address (the Share button): the inputs, the
  ticked plans by label, and your own ticked plans as data. The expert
  inputs (growing deposits, inflation, sell or keep) and the chart by
  deposit sit behind the "More options" switch; off, the state sends the
  core the defaults. The sweep (every plan over a range of deposits, for
  that chart and the best plan's line about other deposits) is 10–15
  comparisons' work, so a Web Worker with its own copy of the core does it
  (`sweeper.ts`, `sweep.worker.ts`); the state matches each answer to the
  request it answers. The sweep and the line about other deposits are about
  fees alone (the yearly cost), while the table ranks after tax: the line
  shows only when the best plan is also the cheapest in fees. How the money
  is taken out (at once, or as a pension) is asked only while a ticked plan
  pays a pension and everything is sold. Inflation always lowers the tax
  (2% unless More options sets another); showing amounts in today's money
  is a separate switch there.
- `policies`: the brokers' tariff PDFs that `tariffs.rs` is taken from;
  `sources.md` says where each number comes from and how unclear rows were
  read (the law's rules in `vehicles.rs` and the funds' fees too), and
  `not-modeled.md` lists the fees and taxes the app leaves out.
  `gemel-net.py` works the funds' fees out from the regulator's open data.

Keep as much logic as possible in Rust; the web side displays what it returns.

## Done means

`cargo test`, `cargo clippy --all-targets` (pedantic, 0 warnings),
`cargo fmt --check`, then in `web/`: `npm run wasm && npm test` (the unit
tests, then the browser tests) and `npm run lint` (prettier, eslint,
stylelint, svelte-check). After a change to how the page behaves when used
(charts, animations, inputs, loading), also `npm run test:perf`.

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
- Each broker's `new_customer_plan` is ticked at first; of the funds, only
  the provident fund for investment's (`compared_at_first`).
- A fund is listed as a kind, not a company: "Average fee" from the
  regulator's data (dated in a caveat), the cheapest and dearest company,
  and the most allowed. A fund's plan charges only its `management`; in
  Hebrew it isn't called a מסלול, which for a fund is an investment track
  (מסלול השקעה).
- A plan's `description` says what it is, who can join and on what terms,
  never its prices: the details show those under it, filtered by what the
  user buys (a test rejects percentages, cents and small amounts).
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
  that, everywhere), `not_counted` (a real cost left out, and why). Every
  text, name and source name is a `Text` pair, written `t("English",
  "עברית")`: the compiler refuses a caveat with one language.
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

Each check goes in the cheapest layer that can catch the bug: Rust, then
the unit tests in Node, then the browser.

- `crates/core/tests/real_tariffs.rs` checks the tariffs against amounts
  worked out by hand from `policies`. A tariff change needs a case there,
  with the arithmetic in a comment; so does a change to the law's rules in
  `vehicles.rs`.
- `crates/core/tests/economics.rs` checks what must hold for any plan and
  any investing pattern: where every shekel goes, that no plan beats no
  fees, that raising a fee never helps, the compounding formula, the
  cheapest track; partly as `proptest` properties over random scenarios
  (`PROPTEST_CASES=1000 cargo test` for a deeper run). A change to the
  simulation needs its rule here, and a new test must fail without its fix.
- `cargo bench -p broker-fees` times the calls the page makes on every
  keystroke (divan, `benches/hot_paths.rs`); the numbers to expect are in
  LESSONS.md.
- `cargo mutants -p broker-fees -j 8` asks whether the tests would notice a
  change to each line of the core; run it after adding a rule, and turn
  the surviving mutants that matter into tests (LESSONS.md, "Tools").
- The web side's layers, and what goes where, are in `web/tests/CLAUDE.md`:
  vitest unit tests beside the code (`src/**/*.test.ts`), Playwright specs
  by part of the app, and the performance suite.

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
  - The `chart` attachment draws just after the paint that follows a change,
    so typing shows first, and not at all while its element has no size (a
    hidden view). Both chart views stay mounted; a hidden one keeps its
    canvas. An option must not read what hiding its view changes (its box's
    width, which goes to 0; whether its own view is chosen), or coming back
    draws it all again: keep the value it was last shown with.
  - Never change options on hover: it redraws and loses the click, since a tap
    is a hover and a click at once. Use `dispatchAction` (highlight).
  - On touch screens (`pointer.ts`), ignore `mouseover`, because a scrolling
    finger fires it.
  - Series `id`s make `replaceMerge` keep the old order.
  - Events carry `seriesIndex`, not `seriesId`.
  - Never let ECharts wrap or cut a label (`overflow`): zrender 6 guesses
    every letter outside ASCII to be as wide as a Chinese one, so Hebrew
    broke at half the width. Measure on a canvas instead (`fitName`).
  - Tooltips are `confine`d and the chart card clips (`overflow: clip`):
    ECharts shows a tooltip mid-chart before moving it to the pointer, and
    on a phone anything past the screen's edge zooms the page out (a jump);
    past the left edge, a right-to-left page scrolls sideways to it.
- **CSS:**
  - A grid child with a wide table needs `minmax(0, 1fr)`, or the page widens.
  - A chosen button must keep its width: bold wrapped the row.
  - The Baseline plugin rejects `overscroll-behavior` (Safari lacks it on the
    page root) and, until October 2026, `:popover-open`: tips carry an `open`
    class from `tip.ts` instead.

- **The core in the browser:** nothing may call it while a module loads
  (a top-level `const x = core.f()`): the WebAssembly isn't there yet, and
  only a browser shows it, since the unit tests load the core first. Read
  it in `AppState`.

More traps, tariff research and this machine's tool quirks: `LESSONS.md`.

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
  - units inside the fields;
  - motion through the tokens in `app.css`: `--ease-out` for what arrives,
    `--ease-in` and less time for what leaves; presses sink a little; a
    change closer than 250 ms to the last one (typing) snaps instead of
    gliding; nothing moves under `prefers-reduced-motion`.
- **Charts:**
  - time is shown as an axis, never a slider, and never in 3D;
  - the table and charts are linked, for hovering and pinning;
  - the other lines fade only a little.
- **The dark theme only.** It should feel fast.

## Hebrew only

The page is shown in Hebrew, right to left, whatever the browser's
language: `index.html` says so (`lang="he" dir="rtl"`), and there is no
English interface. The English texts stay beside the Hebrew ones, in the
core and in `en.ts`: the page shows a term's English name under its Hebrew
one ("באנגלית"), and links and saved copies name plans in English.

- Core: `Lang` (`En`, `He`) and `Text { en, he }` in `lib.rs`. Every
  function that makes words takes a `lang`; a fixed text is
  `lang.pick("English", "עברית")`, data is a `Text` (`text[lang]`). Nothing
  falls back to English: a missing translation doesn't compile. Names of
  enums are `Named::name(lang)`; there's no `Display` for user-facing types.
- Wasm: `setLang` once, before anything else; every binding answers in it.
  `main.ts` and the tests set `He`. Links and saved copies name plans by
  their English names (`englishName`, `englishLabel`).
- Web: `t.<key>` from `text.ts`, which is `he.ts`; `he.ts` is typed as
  `en.ts`, so a key missing from either fails svelte-check. `en` itself is
  read only for English names (`englishName` of the page's own choices).
- Hebrew in Rust strings goes on one line, however long (rustfmt leaves
  it), with ״ (U+05F4) for quotes so nothing needs escaping. Second person
  is plural ("שלכם", "סמנו").
- CSS is direction-neutral: logical properties (`margin-inline-start`,
  `text-align: start`, `inset-inline-start`), except where a physical
  measurement is used (`offsetLeft` moves the choices' highlight from the
  left; the plan preview opens on the left; a pinned row's inset mark). A line of numbers with `=` or `·` gets `<bdi dir="ltr">`, or it
  reads backwards. The charts stay left to right.
- Every spec drives the Hebrew page, from an English browser (Playwright's
  default locale), which `tests/hebrew.spec.ts` checks still gets Hebrew.
  Specs name the page's words by `t` and the core's by the core
  (`tests/core.ts`: plans by their English label, `securityName`,
  `exchangeName`, `brokerName`), and write Hebrew out only to check
  wording.

## Hebrew

Users are Israeli; English text gives the Hebrew term alongside, e.g. "fees
(עמלות)". Check terms online for what Israelis actually say:
- brokers → בנקים ובתי השקעות, not ברוקרים;
- plans → מסלולים;
- securities → ניירות ערך;
- bonds → אג"ח;
- ETFs → קרנות סל (תעודות סל until 2018); index funds → קרנות מחקות, which
  every tariff prices apart from managed funds (קרנות מנוהלות, mostly free
  to trade), so never say קרן נאמנות for what the app compares;
- a US commission track (per share, per order) → שיטת חיוב, not מסלול,
  which is a plan; a joining offer → מבצע הצטרפות; custody → דמי משמרת;
  the monthly handling fee → דמי טיפול; the conversion markup → מרווח המרה;
- a provident fund for investment → קופת גמל להשקעה; a study fund → קרן
  השתלמות; a savings policy → פוליסת חיסכון; its fee → דמי ניהול מהצבירה, מהפקדה; a monthly pension →
  קצבה; taking it all at once → משיכה בבת אחת; the deposit ceiling → תקרת
  הפקדה; an index-following track → מסלול עוקב מדד.

- Hebrew inside English goes in `<bdi lang="he">`. In Rust strings, join a
  phrase's words with `\u{a0}`: a phrase split across lines reads backwards.
- Label Hebrew names ("In Hebrew", "Also called"), one per line. A list of
  them on one line reads in the wrong order.

## Saving tokens

Never read a whole large file or output (test logs, build output,
`Cargo.lock`, snapshots, `web/src/lib/core`). Narrow it with `grep` (ugrep
here: if a pattern fails, use Python), `head`/`tail`, `sed -n 'X,Yp'` or
`wc -l`. For tests, use
`npx playwright test --reporter=line 2>&1 | tail -30` and
`npx vitest run 2>&1 | tail -8`; run the one spec a change concerns first
(`web/tests/CLAUDE.md` says which). Look at pictures only through one
contact sheet (LESSONS.md, "Recipes"), and only at the ones that changed.

## Deploying

The site is GitHub Pages, at
https://orishamir.github.io/Israeli-Broker-Search/, built and deployed by
`.github/workflows/deploy.yml` on every push to `main` (or by hand from the
Actions tab). Its `test` job gates the deploy: the Rust checks and tests on
stable, the unit tests, the linters, and Playwright's desktop project with
`--ignore-snapshots`, since the pictures were recorded on this machine's
fonts; the phone projects and the performance suite stay local. Vite's
`base: './'` makes the build work under the repository's path. The
build itself (`npm run build`) never touches the tariffs' PDFs.
