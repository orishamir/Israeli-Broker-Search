# Broker search

## What this site is for

In the user's words (2026-10-19), the essence every design choice is judged
against:

> I created this site to compare banks and investment houses for investing
> in ETFs, and to visually understand clearly how compound interest looks
> like, and specifically how fees hurt that compounding effect: pie charts,
> line graphs, and between investing in different exchanges. I still want
> that. During the research I realized that isn't the only comparison worth
> making: the difference between a mutual fund (קרן נאמנות) and a קרן סל in
> Israel isn't trivial and might be more important than choosing between
> investment houses. And I realized I want both the easy route for
> newcomers, like I was at the time, who don't understand the different
> strategies (money market vs bond ETF vs stock ETF vs mutual fund vs פיקדון
> צמוד מדד) and want to learn by example, and an advanced route for when I
> specifically know what I want to compare and visualize.
>
> When the site chooses something for someone, it must make clear **what** it
> chose (what is it? קרן סל? קרן מחקה? in the terms they meet in their
> bank's app, in ads, around them) and **why** it chose it for them: how
> their circumstances affected the decision. And it never simply tells the
> answer: it tells it while visually showing how their money would look in
> the other choices they could have made and weren't recommended. The site
> guides through questions while simultaneously accepting the more advanced
> ones, and educates in the process.

The five questions it must answer: a dated goal in about five years (and
whether the date is flexible); "I want the S&P 500, which way?"; "₪3,000 a
month, I don't know what for"; "₪1,500 a month for retirement"; "15–20
years from now". The page built on this is one page with two routes: the guide and the
workbench (`REDESIGN.md` is its plan of record).

## The page

One page, Hebrew, right to left, dark, with two routes onto the same
engine. **The guide** (the default) asks a few questions (what for, how
much now and a month, when, and only when it matters whether the date can
move or there is a cushion) and answers "מה שמתאים לכסף כזה": what it
chose, in the words of the bank's app, why (each reason from one of the
saver's circumstances), how to do it, and, drawn beside it, what the same
money would have come to in the kinds it didn't choose. **The workbench**
(the "כל האפשרויות" switch in the header) is for who knows what to
compare: what to buy, the ways to hold it, every lever, the full table and
all the charts. The guide and the workbench are one state: the switch only
changes what is shown. The fee model is Rust compiled to WebAssembly; the
UI is Svelte 5 + ECharts. Only what earns the same thing is ranked: the
safe kinds (a money market fund, a fixed-rate deposit, a government bond
held to maturity) among themselves, and the index kinds (a broker's index
fund or ETF, a provident fund for investment, a study fund, a savings
policy) among themselves; the other arena is only drawn beside, with its
risk (`Safety`). A provident fund for savings, mixes, and personal advice
are left out on purpose (`policies/not-modeled.md`).

## Layout

- `crates/core` (lib `broker_fees`):
  - `tariffs.rs` holds the real price lists (brokers' plans and, per bank,
    its "Branch or through a banker" plan); `simulation.rs` runs them over
    the years (and states an outcome as a yearly cost, in today's money,
    and over a range of deposits: `sweep`, `Sweep::around`). The comparison
    ranks by what's left after tax (`Outcome::after_tax`); within a shekel
    it is a tie, and a tie goes to the plan with no may-cost-more caveat
    (`compare_with_broker_caveats`); a first deposit that would open a better
    plan is reported (`unlock`); a bond held to maturity is redeemed with no
    sale fee (`Scenario.redeems_at_maturity`). `NotOffered` says why a plan
    can't be used (the security isn't sold, no matching track, a year's
    deposits pass a fund's ceiling, the money is still locked); the safe
    kinds have their own (`short_term::NotOffered`: a deposit past 60
    months, a bond past the curve's end).
  - `vehicles.rs` holds what the law says about each kind of account (tax on
    the gain, the deposit ceiling or its tax-free part, how long the money
    is locked); `funds.rs` lists the kinds of fund like brokers
    (`BrokerKind::Funds`), by what savers pay, with their S&P 500 tracks.
  - `products.rs`: what holds the index for the saver and what that costs on
    its own (`Product`, `Costs`; a fund's fees, the dividends' tax nobody
    gives back, the tracking gap), so the expected return is the index's and
    every plan keeps back its product's share. `ways.rs`: a `Way` is a
    product in an account (an index fund through a bank, an ETF in Tel Aviv,
    a US ETF, a provident fund's track); `compare_ways` ranks the ways, each
    on the plans that can hold it.
  - `history.rs` (+ `history/sp500.rs`, made by `policies/sp500-history.py`):
    what every stretch of 1 to 30 years did since 1926 (`band`); the page
    draws it as the band and the guide says it in words. It ends in June
    2023 (`sources.md`).
  - `short_term.rs` and `short_term/`: the safe kinds for 1 to 600 months:
    `money_market.rs` (the funds' reported fees), `deposits.rs` (each bank's
    fixed-rate deposit, to 60 months), `government_bonds.rs` (shekel bonds
    held to maturity, nominal and CPI-linked, from the Bank of Israel's yield
    curve, through the brokers' fees).
  - `guide.rs` and `guide/`: the recommendation. `guide.rs` has the answers
    (`Answers`, `Horizon`, `Flexibility`, `Cushion`, `Situation`), the fixed
    numbers (the 12/36/60/84-month lines, the 3-month cushion, the ladder's
    years), `follow_ups` and `check`; `guide/route.rs` is REDESIGN.md §3's
    table, a `Route` per row, and builds `recommend` (what, why, steps, the
    alternatives, the unlock line, the cushion's two parts), `ladder` (the
    same money at 1, 3, 5, 10, 20 years) and `alternatives` (each kind's line
    month by month plus the history band); `guide/index.rs` prices the index
    kinds through `ways`, `guide/safe.rs` the safe kinds through
    `short_term`, `guide/series.rs` draws the lines and names the columns of
    the table (`columns_for`). The guide invents no figure: it asks the
    engines.
  - `situations.rs`: the chips the guide opens with (a reason Israelis save,
    each with the answers it sets and a different lesson); it replaced
    `examples.rs`.
  - `describe.rs` holds all text shown to users, in Hebrew and English (see
    "Hebrew only"), and its modules `describe/learn.rs` (the learn cards and
    the glossary, each claim sourced and linked), `reasons.rs` (the reasons
    with their numbers, why-not lines, the history sentence, safety tags,
    the unlock line), `steps.rs` (how to do it, step by step, per kind) and
    `answer.rs` (the what-sentence, the amount's notes, follow-up cards, the
    bank stand-in note). `yours.rs` holds the user's own plans and the
    editor's fields.
- `crates/wasm`: bindings (wasm-bindgen + tsify), built into the git-ignored
  `web/src/lib/core` by `npm run wasm`; `lib.rs` the plans and comparison,
  `guide.rs` the guide's (`recommend`, `ladder`, `alternatives`,
  `compare_ways`, `ways`, `situations`, `follow_ups`, `banks`, `columns_for`,
  `history_band`, `learn_card`, `glossary`, `about_all`, `footer`). A struct
  with `#[serde(flatten)]` is written as an object, not a `Map`.
- `web`: the app. One `AppState` (`src/lib/app.svelte.ts`) holds the guide's
  `answers`, the `expert` switch (persisted as `expert-v2`, in links as
  `x=1`), the workbench's inputs and `overrides` (a kind or way the user chose
  on "לשנות" or by tapping an alternative), the ticked plans and your own,
  and derives the `recommendation` (`core.recommend`), the `comparison`
  within the arena (`compare` or `compareWays`) and `todaysMoney` (default
  on). `App.svelte` is the header (title, glossary, about, share, the
  switch), the layout and the footer, and the best bar on phones.
  - Components: `Questions` (situation chips, the money and horizon fields,
    follow-up cards), `Answer` (the sentence with "מה זה? ›" and "לשנות ›",
    the amount, reasons, unlock line, cushion plan), `Alternatives` (+
    `alternatives-chart.ts`: the lines per kind, the band, rows with the safety
    tag and "why not"; tap a row to continue with it), `Within` (+ `within.ts`:
    "איפה הכי זול לקנות": the top three, or in expert mode the full table, one
    component for both arenas with the columns the core names), `Steps`,
    `Donut` (+ `donut.ts`: where every shekel goes), `Ladder`, `Workbench`
    (what to buy, the ways, the levers, "מה משווים" with `PlanSheet` and the
    editor), `ExpertSwitch`, `Charts` (growth, lost to fees, fee breakdown,
    by deposit; in the guide only growth and the donut), `LearnCard` and
    `Glossary` (a `Sheet` with the card; a tip's "עוד על זה ›" opens it),
    `PortfolioDialog` and `BankChips` (the situation "כבר יש לי תיק בבנק").
    `kinds.ts` turns the core's kinds and ways into the keys that links and
    series ids use; `colors.ts` is the palette (a color per kind, per fee part,
    per plan); `horizon.ts` is the horizon field's number-and-unit.
  - The plans to compare are ticked in a list that opens from the "What's
    compared" card (`PlanSheet.svelte`; the whole screen on a phone), one
    folded line per broker, then your plans. The page's own words (labels,
    buttons, the switch, the footer) are in `src/lib/text/he.ts` (typed as
    `en.ts`), read through `text.ts` (`t`); every sentence about money,
    kinds, reasons, steps and cards comes from the core. Your plans are kept
    in localStorage (`saved.ts`) as core `Plan`s the web side never looks
    inside (`PlanData`); nothing else is remembered but the switch.
  - The charts' options are pure functions in `growth-chart.ts`,
    `crossover-chart.ts`, `fee-breakdown.ts`, `alternatives-chart.ts` and
    `donut.ts`; the components only wire them to ECharts and the state, so
    the options are unit-tested without a browser. `glide.ts` says when a
    chart draws a change and how long it glides, `motion.ts` has the page's
    curves and times. While the inputs can't be compared (a field being
    retyped), the page keeps the last results, faded, under the message
    (`inputError`).
  - Work that costs more than a keystroke goes to a Web Worker with its own
    copy of the core (`sweep.worker.ts`, `sweeper.ts`, jobs named in
    `work.ts`: `sweep`, `recommend`, `ladder`, `alternatives`; the unit tests
    run them in-line). Each job keeps only its latest request, and the state
    matches each answer to the request it answers. The sweep (the ticked
    plans over a range of deposits, for the chart by deposit and the best
    plan's line about other deposits) is about fees alone, while the table
    ranks after tax. The ladder and the alternatives are asked once typing
    pauses.
  - `link.ts` puts the state into the page's address (the Share button),
    `#v=2&…`: the answers (`n`, `m`, horizon `h` as months, `u` or
    `r:<age>:<retire>`, flexibility `fx`, cushion `cu`, situation `si`),
    `x=1` for expert, the workbench's inputs when expert or overridden, the
    ticked plans by English label and your own as data. A link without `v=2`
    opens nothing.
  - Inflation lowers the tax (2% unless the workbench sets another), except
    on a broker's holdings abroad, whose gain the law measures against the
    exchange rate; a large sale also pays the surtax, at its least. Amounts
    are shown in today's money by default, the nominal figure as a note.
- `policies`: the brokers' tariff PDFs that `tariffs.rs` is taken from;
  `sources.md` says where each number comes from and how unclear rows were
  read (the law's rules in `vehicles.rs`, the funds' fees, the guide's rules,
  the history, the yield curve), and `not-modeled.md` lists what the app and
  the guide leave out. Scripts: `gemel-net.py` (funds' fees), `money-market-funds.py`,
  `deposit-rates.py` (the Bank of Israel's), `yield-curve.py` (government
  bonds), `sp500-history.py` (Shiller's data), `index-tracking.py` (what the
  funds holding the index keep back).
- `REDESIGN.md` is the plan this was built from; `docs/redesign/screens/`
  the screens (`mock.html` is the source of truth for wording and layout).

Keep as much logic as possible in Rust; the web side displays what it returns.

## Done means

`cargo test`, `cargo clippy --all-targets` (pedantic, 0 warnings),
`cargo fmt --check`, then in `web/`: `npm run wasm && npm test` (the unit
tests, then the browser tests) and `npm run lint` (prettier, eslint,
stylelint, svelte-check). After a change to how the page behaves when used
(charts, animations, inputs, loading, the worker's jobs), also
`npm run test:perf`. After a change to the guide's rules, run
`cargo mutants -p broker-fees -j 8 --timeout 180 --file 'crates/core/src/guide.rs' --file 'crates/core/src/guide/*.rs' --file crates/core/src/ways.rs --file crates/core/src/history.rs`
(about 15 minutes; `--file` takes globs and repeats) and turn the surviving
mutants that matter into tests. It copies the tree for each job, so with
agents' worktrees under `.claude/worktrees` (19 GB) run it from a fresh
`git clone` with `TMPDIR` on the big disk, or `/tmp` fills up.

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
  regulator's data (dated in a caveat), with the cheapest and dearest
  company and the most allowed shown beside it, not as plans of their own
  (`Broker::spread`). A fund's plan charges only its `management`; in
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
  summary is flagged under the plan in the results table, and of the
  caveats only these are marked ⚠; besides, ⚠ marks a plan the inputs
  don't fit, as a first deposit too small, and a price-list row that's
  never used), `not_counted` (a real cost left out, and why). Every
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
- `crates/core/tests/guide.rs` has a test for each row of the guide's table
  (REDESIGN.md §3; `situation` and horizon by horizon), the follow-ups, the
  cushion's arithmetic, that the ladder's rungs are monotone within an
  arena, that the alternatives always hold the current account and the other
  arena, and the acceptance questions as the core answers them.
  `guide_words.rs` checks the texts exist in both languages and name their
  circumstance (never what they say). `ways.rs` checks the ways and the
  products' costs, `history.rs` the band's arithmetic and ordering. A new
  guide rule needs its row in `guide.rs` and a reason's text; a new kind of
  safe holding needs a hand-worked case in `real_tariffs.rs`.
- `cargo bench -p broker-fees` times the calls the page makes on every
  keystroke and the guide's (`recommend`, `ladder`, `compare_ways`, divan,
  `benches/hot_paths.rs`); the numbers to expect are in LESSONS.md.
- `cargo mutants -p broker-fees -j 8` asks whether the tests would notice a
  change to each line of the core; run it after adding a rule, and turn
  the surviving mutants that matter into tests (LESSONS.md, "Tools").
- The web side's layers, and what goes where, are in `web/tests/CLAUDE.md`:
  vitest unit tests beside the code (`src/**/*.test.ts`), Playwright specs
  by part of the app (`questions`, `answer`, `alternatives`, `workbench`,
  `learn`, `acceptance`, besides the kept ones), and the performance suite.

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
  - **The alternatives chart** (`alternatives-chart.ts`): every series has an
    `id`, the kind's `kindKey`, never its position; the history's band is
    two `stack`ed series (the lower one's line transparent, the upper one's
    `areaStyle`), not rows and never hovered; only three lines get an end
    label (the recommended kind, the stock fund, the current account), the
    rest are named in the rows under the chart; names are fitted with
    `fitName`, never wrapped or cut; hover and the rows' linking go through
    `dispatchAction`.
  - **The donut** (`donut.ts`): labels are short (הפקדה / רווח / עמלות / מס /
    הקרן עצמה) with `overflow: 'none'`; a slice under 0.5% is listed under
    the donut, not labelled on it.
  - The `chart` attachment draws never before the paint that follows a
    change, so typing shows first, and not at all while its element has no
    size (a hidden view). Both chart views stay mounted; a hidden one keeps its
    canvas. An option must not read what hiding its view changes (its box's
    width, which goes to 0; whether its own view is chosen), or coming back
    draws it all again: keep the value it was last shown with.
  - Never change options on hover: it redraws and loses the click, since a tap
    is a hover and a click at once. Use `dispatchAction` (highlight).
  - On touch screens (`pointer.ts`), ignore `mouseover`, because a scrolling
    finger fires it.
  - Every series needs an `id`: with `replaceMerge`, one without is drawn
    again from nothing at every change. Ids keep the old order, so where the
    order changes (a focused fee stacked first) the id is the place, not
    what's in it.
  - A line's next glide starts from where the last one was going, not from
    where the line is: a glide cut short jumps (`drawAt` waits).
  - Labels that must move with their bars ride on the bars (series labels):
    a category axis's labels stay in place while the bars slide. A line's
    end label doesn't glide unless the attachment lets it
    (`releaseEndLabels`).
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
  - A table sized by its contents moves every column when a note comes or
    goes: the results table (`Within`) gives the name's column a set share (`width`)
    and split the rest. A warning in the best plan's card is a ⚠ beside its
    amount (`WarningSign`), its words in the table's row below: a line of
    its own moved the table, and an empty line kept for it looked broken.
  - The tables' box (`.scrolls`) scrolls sideways only: rows sliding to new
    places reach past its bottom for a moment, which showed a vertical
    scroll bar.
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
- **A recommendation always says what, why and how, and draws the
  alternatives.** What: the thing's name in the words of the bank's app, the
  ads and around (`מה זה? ›` opens its learn card). Why: each reason comes
  from one of the saver's circumstances, with their numbers. How: the steps
  (`Steps`). And the money in the kinds it did not choose is drawn beside it
  (`Alternatives`, `Donut`), never only told; every kind keeps a "why not".
  Safe and index kinds are never ranked against each other.
- **The register** is "מה שמתאים לכסף כזה", never "ההמלצה שלנו": the page
  says what fits money like this, not what to do; no personal advice, no
  "you should". Where the past is shown it is said to be the past
  ("ב־N מתוך M תקופות של X שנים נגמרה בהפסד"), and no figure is rounded to
  "never".
- **The learn card** has one shape (`LearnCard`, screen s5): the term, its
  names (the bank's, the ads', in English), what it is, what it is good
  for, its downside, what it costs, its tax, where you meet it, an example,
  and what to compare it with (chips that open the other card in the same
  sheet, with a way back). Every claim rests on a source linked under it.
  A tip stays one paragraph and ends with "עוד על זה ›" when it has a card.
- **Amounts are in today's money** by default, with the nominal figure as a
  note, and the growth chart's axis says so.
- **Keep the main thing prominent.** Secondary information stays subtle:
  - the exchange rates are folded into one line;
  - the broker's name under each plan, and the rank, are de-emphasized.

  Groups of inputs go in separate cards.

- **One style:**
  - one tooltip style everywhere;
  - outlined buttons, as tall as a choice (33 px); text fields and menus
    37 px; tints of the accent and warning from the tokens on `:root`;
  - units inside the fields, beside the number (`₪ 10,000`, `10 %`);
  - every field in the questions and the workbench one width (`--input-field`),
    so they line up from card to card; the labels take the rest and wrap;
  - a "?" after words that may wrap sits in a flex row with them
    (`.titled`, the inputs' label spans): inline, a line can break before
    it and leave it alone;
  - while a table scrolls sideways, the screen ends on a whole column (the
    container steps in `Within`);
  - motion through the tokens in `app.css` (`motion.ts` for script):
    `--ease-out` for what arrives, `--ease-in` and less time for what
    leaves; presses sink a little; the results move to new numbers together
    for `--settle` (400 ms): the charts glide, the summary's numbers roll
    and the rows slide, and a box whose words change its height glides to
    it (`glideHeight`: the summary, the tables, a swapped description). A
    chart never cuts a glide short, follows a dragged slider in shorter
    glides, and waits for a typed number to pause, as the line about other
    deposits does (`HeldWhileTyping`); what comes and goes opens and closes
    (`transition:reveal`); nothing moves under `prefers-reduced-motion`.
- **Charts:**
  - time is shown as an axis, never a slider, and never in 3D;
  - each part of a stacked bar has one color, the same in every bar; a
    plan's or a place's own color is only beside its name (and its outline
    when pinned);
  - the table and charts are linked, for hovering and pinning;
  - the other lines fade only a little.
- **The dark theme only.** It should feel fast.

## Hebrew only

The page is shown in Hebrew, right to left, whatever the browser's
language: `index.html` says so (`lang="he" dir="rtl"`), and there is no
English interface. The English texts stay beside the Hebrew ones, in the
core and in `en.ts`: the page shows a term's English name under its Hebrew
one ("באנגלית"; not an exchange's, a place known by its Hebrew name:
`Explained::english_name`), and links and saved copies name plans in
English.

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
  left; the plan preview opens on the left; a pinned row's inset mark; the
  tables' amounts, right-aligned so their last digits line up). A line of numbers with `=` or `·` gets `<bdi dir="ltr">`, or it
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
  the monthly handling fee → דמי טיפול; the two as one fee →
  דמי משמרת וטיפול (not דמי ניהול חשבון, too near a fund's דמי ניהול),
  and, with a fund's fee, in the fee breakdown → דמי ניהול ומשמרת; the
  conversion markup → מרווח המרה;
- a provident fund for investment → קופת גמל להשקעה; a study fund → קרן
  השתלמות; a savings policy → פוליסת חיסכון; its fee → דמי ניהול מהצבירה, מהפקדה; a monthly pension →
  קצבה; taking it all at once → משיכה בבת אחת; the deposit ceiling → תקרת
  הפקדה; an index-following track → מסלול עוקב מדד.
- the two arenas → סכום בטוח יותר, מדד לטווח ארוך (`arena_name`); a money
  market fund → קרן כספית; a fixed-rate deposit → פיקדון בריבית קבועה (a
  bank's monthly savings plan is a תוכנית חיסכון, not a פיקדון); a place to
  keep the money, in the table of the safe kinds → אפיק; the Bank of
  Israel's rate → ריבית בנק ישראל.
- the redesign's terms: a government bond held to maturity → אג״ח ממשלתית
  לפדיון (a short one is a מק״מ); a CPI-linked deposit → פיקדון צמוד מדד;
  the safety cushion → כרית ביטחון; the yield curve → עקום תשואות;
  yield to maturity → תשואה לפדיון; a sum known in advance → סכום ידוע
  מראש; money you may need any day → כסף שאולי תצטרכו בכל יום; "this
  fits money like this" → מה שמתאים לכסף כזה; today's money → כסף של
  היום; compound interest → ריבית דריבית; the S&P 500 index → מדד S&P 500;
  a bank's branch price list → תעריף סניף (the plan "Branch or through a
  banker"). Prefer "אג״ח ממשלתית לפדיון" to a bare "אג״ח" where the kind
  matters, and "מסלול עוקב מדד" for a fund's index track.

- A " · " shown in Hebrew has a non-breaking space before it (`\u00a0· `,
  `&nbsp;·` in markup), so a line may end with the dot but never start
  with it. English labels keep a plain space: links find plans by them.
- Hebrew inside English goes in `<bdi lang="he">`. In Rust strings, join a
  phrase's words with `\u{a0}`: a phrase split across lines reads backwards.
  A hyphenated Hebrew word is joined with a non-breaking hyphen
  (`חד\u{2011}פעמית`, `\u2011` in TypeScript), or a narrow line breaks it
  at the hyphen. A prefix before a number (`מ-₪100`, `ל-12`) never breaks.
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
stable, the unit tests, the linters, and Playwright's desktop project; the
phone projects and the performance suite stay local. Vite's
`base: './'` makes the build work under the repository's path. The
build itself (`npm run build`) never touches the tariffs' PDFs.
