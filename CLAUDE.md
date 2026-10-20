# Broker search

## What this site is for

The site compares Israeli banks and investment houses, and the funds beside them, for saving and investing. It shows what money comes to, and how fees and tax eat into compounding, across exchanges and ways of holding the same index. In Israel the difference between a mutual fund (קרן מחקה, a kind of קרן נאמנות) and an ETF (קרן סל) can matter more than the choice of broker, so the ways are compared too.

Two kinds of visitor come:

- newcomers, who don't know the strategies (a money‑market fund, a bond, a stock index fund, a mutual fund, a CPI‑linked deposit) and learn by example: **the guide**;
- people who know what they want to compare and see: **the workbench**.

How the guide answers, in the owner's words (2026-10-19), after reviewing the redesign's mockup:

> "We shouldn't project a false narrative of a 'perfect solution' for the customer. We can and should display our suggestion. If it's a 'you can't go wrong in your case' suggestion then maybe we can allow ourselves to display it more confidently. But we have to remember that the choice isn't that simple. (The way to remember that isn't to simply add more tooltips. It should be a UI/UX way.)"

> "You only answer the questions when they are relevant, and the user can clearly see the established connection between what the site asks him to choose, why it's up to him, and what difference it would make."

**The design rules are `REDESIGN.md` Part 1.** They bind every change a visitor can see, in the guide and in the workbench. In short:

1. **Don't present a perfect answer.** Show how sure the site is, and what is the visitor's call, through layout, not disclaimers or tooltips.
2. **Ask only when the answer changes the outcome.** Otherwise state the conclusion.
3. **Ask in context**, at the point of the answer the question decides. Never as a questionnaire up front, and never as a row of assumptions.
4. **A decision is options side by side:** what each leaves, what it asks of you, when it fits, and the ₪ difference. The unchosen option stays visible, and the choice can be changed.
5. **Don't infer preferences from facts.** "No securities account" doesn't mean "won't open one": show the trade‑off.
6. **Ask only what only the visitor can know.** Uncertainty about the world becomes ranges and switch points, never a question.
7. **The same kind of question takes the same form everywhere.**
8. **A decision that depends on another appears only after it.**
9. **Fix causes, not symptoms**, and redesign from the visitor's questions, not from the current screen.
10. **Every visual answers one named question, and can be read without a "?".** Label every mark; never rely on colour alone.
11. **A visual must never contradict the answer:** after tax, in today's money.
12. **Compare like with like**, against what the visitor deposited and against doing nothing (the current account).
13. **Precision matches certainty:** rounded figures and ranges; ties as ties; differences, not ranks; each number once.
14. **Make the last step concrete:** what to search for in the app. Never "מדד" alone, and say what a product is before its trade name.
15. **Compare from the visitor's own bank.**
16. **Be honest with data:** no invented numbers, marked placeholders, and the past said to be the past.
17. **Rank only what earns the same;** the rest is drawn beside, and which indexes to hold is the visitor's call. A fund's name is an example, never a pick.
18. **When a principle strains, question the principle:** propose a fix to the owner rather than bending the page to fit.

Before calling a visible change done, run `REDESIGN.md`'s review checklist.

## The page

One page, Hebrew, right to left, dark, with two routes onto the same engine.

**The guide** (the default):

- **What it takes:** the visitor's facts: what they have now, what they'll add each month, and when they'll need the money; and, if they like, their bank ("הבנק שלי · לא חובה"): where to buy measures from it, and without it from the banks' range.
- **How it answers:** on up to four lines:
  - **קודם**, only when a step must come first;
  - **סוג המקום**;
  - **הדרך**;
  - **איפה**.

  Each line is either stated by the site, with how sure it is, or a decision only the visitor can make. A decision is shown in place, as options side by side with what each would leave.

- **How sure it is:** a chip over the answer: כאן קשה לטעות / ההבדל קטן / תלוי במה שחשוב לכם.
- **Below the answer:**
  - one chart of what every option would leave on the visitor's date, after tax and in today's money, beside what they deposited and what the current account would leave;
  - how to do it, with a name to search for;
  - how much of the gain fees and tax take;
  - what changes if the date moves.

**The workbench** (the "כל האפשרויות" switch in the header) is for visitors who know what to compare: what to buy, the ways to hold it, every lever, the full table and all the charts. The guide and the workbench are one state: the switch only changes what's shown.

**Where the code stands (2026-10-20):** the redesign (`REDESIGN.md` Part 2) is built:

- the answer card with the chip, the four lines and the one decision component;
- the outcome chart, with the yearly return as a field on the stock row;
- "איך עושים את זה" with the identity line, real names to search for (`examples.rs`) and where to buy against the visitor's bank;
- "כמה מהרווח נשאר אצלכם" and the horizon section.

The guide has no assumptions sheet and no override: inflation and the rate are workbench levers. The old answer, the alternatives chart, the donut, the ladder and the follow‑up cards are gone. `docs/redesign/review-2026-10-20.md` is the review against Part 1.

**The engine.** The fee model is Rust compiled to WebAssembly; the UI is Svelte 5 + ECharts.

- Only what earns the same thing is ranked. The safe kinds (a money‑market fund, a fixed‑rate deposit, a government bond held to maturity) are ranked among themselves. So are the index kinds (a broker's index fund or ETF, a provident fund for investment, a study fund, a savings policy).
- The other arena is only drawn beside, with its risk (`Safety`).
- A provident fund for savings, mixes, and personal advice are left out on purpose (`policies/not-modeled.md`).

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
    numbers (the 12/36/60/84-month lines, the 3-month cushion, the horizon
    section's years, `HORIZON_YEARS`) and `check`; `guide/route.rs` is the
    rules table (`REDESIGN.md`, Appendix A), a `Route` per row, and builds
    `recommend` (what, why, steps, the unlock line, the cushion's two parts:
    what the workbench ranks from); `guide/index.rs` prices the index
    kinds through `ways`, `guide/safe.rs` the safe kinds through
    `short_term`, `guide/columns.rs` names the columns of the table
    (`columns_for`). The redesigned answer is `guide/answer.rs`
    (`answer`, `answer_with`: `Answer` with its confidence, lines,
    `Decision`s and `Opt`s, the plan, the outcome, a `How` per row, the
    gain, the horizon rows, the annuity; `Choices` and `Pick` are what the
    visitor chose), `guide/robust.rs` (the versions, the confidence state,
    the switch points, the two noise margins) and `guide/outcome.rs` (the
    rows and their marks, the market range for the visitor's deposits,
    where to buy, the gain split). The guide invents no figure: it asks the
    engines.
  - `examples.rs` (made by `policies/examples.py`): the government bond
    series with their numbers and repayment days (`nearest_bond`), example
    funds on the S&P 500 per way (`for_way`, "דוגמה, לא המלצה"), and the
    money‑market funds' numbers. `annuity.rs`: published conversion
    coefficients, and the monthly pension as a range (`monthly`).
  - `situations.rs`: the chips the guide opens with (a reason Israelis save,
    each with the answers it sets and a different lesson); it replaced
    `examples.rs`.
  - `describe.rs` holds all text shown to users, in Hebrew and English (see
    "Hebrew only"), and its modules `describe/learn.rs` (the learn cards and
    the glossary, each claim sourced and linked), `reasons.rs` (the reasons
    with their numbers, the history sentence, safety tags, the unlock
    line), `steps.rs` (how to do it, step by step, per kind) and
    `answer.rs` (the what-sentence, the amount's notes, the arenas'
    names); the redesigned answer's words are in `card.rs` (the chip,
    title, body, lines, decisions, plan), `outcome.rs` (rows, tags, notes,
    the legend), `how.rs` (the identity line, what to search for, the
    steps, where to buy) and `gain.rs` (the gain split, the horizon rows).
    `yours.rs` holds the user's own plans and the
    editor's fields.
- `crates/wasm`: bindings (wasm-bindgen + tsify), built into the git-ignored
  `web/src/lib/core` by `npm run wasm`; `lib.rs` the plans and comparison,
  `guide.rs` the guide's (`answer`, `outcome_words`, `recommend`,
  `compare_ways`, `ways`, `situations`, `banks`, `columns_for`,
  `learn_card`, `glossary`, `about_all`, `footer`). A struct
  with `#[serde(flatten)]` is written as an object, not a `Map`.
- `web`: the app. One `AppState` (`src/lib/app.svelte.ts`) holds the guide's
  `answers` (with the visitor's `bank`), the `choices`, the `expert` switch
  (persisted as `expert-v2`, in links as `x=1`), the workbench's inputs and
  `overrides` (the workbench's own), the ticked plans and your own, and
  derives the `answer` (`core.answer`, then its `horizons` from a second
  job), the focused row (`focusRow`), the workbench's `recommendation`
  (`core.recommend`, only while the workbench needs it), the `comparison`
  within the arena (`compare` or `compareWays`) and `todaysMoney` (default
  on). `App.svelte` is the header (title, glossary, about, share, the
  switch), the layout in `REDESIGN.md` §2.1's order, and the footer.
  - The guide's components: `Questions` (situation chips, the money and
    horizon fields, the ages for retirement, the optional bank), `AnswerCard` (+ `AnswerLine`,
    `Decision`: the one decision component), `OutcomeChart` (+ `outcome.ts`:
    rows as buttons, marks on one ₪ axis, labels placed in lanes, the
    return field), `HowTo` and `WhereToBuy` (+ `how.ts`: the identity line,
    what to search for, the steps, places against the visitor's bank),
    `GainSplit` (+ `gain.ts`) and `HorizonSection`. The
    workbench's: `Workbench` (what to buy, the ways, the levers, "מה
    משווים" with `PlanSheet` and the editor), `Within` (+ `within.ts`: the
    full table, one component for both arenas with the columns the core
    names), `ExpertSwitch`, `Charts` (growth, lost to fees, fee breakdown,
    by deposit), `LearnCard` and `Glossary` (a `Sheet` with the card; a
    tip's "עוד על זה ›" opens it).
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
    `crossover-chart.ts` and `fee-breakdown.ts` (the guide's own are HTML,
    laid out by `outcome.ts` and `gain.ts`); the components only wire them to ECharts and the state, so
    the options are unit-tested without a browser. `glide.ts` says when a
    chart draws a change and how long it glides, `motion.ts` has the page's
    curves and times. While the inputs can't be compared (a field being
    retyped), the page keeps the last results, faded, under the message
    (`inputError`).
  - Work that costs more than a keystroke goes to a Web Worker with its own
    copy of the core (`sweep.worker.ts`, `sweeper.ts`, jobs named in
    `work.ts`: `answer`, `horizons`, `recommend`, `sweep`, sent in that
    order; the unit tests run them in-line). Each job keeps only its latest
    request, and the state matches each answer to the request it answers.
    The answer goes without its horizon rows (`horizons: false`), and the
    rows follow as their own job, so the card never waits for them. The
    sweep (the ticked plans over a range of deposits, for the chart by
    deposit and the best plan's line about other deposits) is about fees
    alone, while the table ranks after tax. A dated horizon up to 5 years
    (`RECOMMEND_ON_PAGE_UP_TO_MONTHS`) has its answer worked out on the
    page's thread too, so the first screen doesn't wait for the worker.
  - `link.ts` puts the state into the page's address (the Share button),
    `#v=2&…`: the answers (`n`, `m`, horizon `h` as months, `u` or
    `r:<age>:<retire>`, flexibility `fx`, cushion `cu`, situation `si`,
    the bank `bk`, the choices `sw`/`wr`/`ac`/`ho`),
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
- `REDESIGN.md` is the plan of record (rewritten 2026-10-19: the design
  rules, the redesigned answer, and the guide's rules as Appendix A);
  `docs/redesign/mockup.dc.html` is the mockup it describes. The previous
  plan, which built today's page, is `git show d14e955:REDESIGN.md`.

Keep as much logic as possible in Rust; the web side displays what it returns.

## Done means

`cargo test`, `cargo clippy --all-targets` (pedantic, 0 warnings),
`cargo fmt --check`, then in `web/`: `npm run wasm && npm test` (the unit
tests, then the browser tests) and `npm run lint` (prettier, eslint,
stylelint, svelte-check). After a change to how the page behaves when used
(charts, animations, inputs, loading, the worker's jobs), also
`npm run test:perf`. After a change to the guide's rules, run
`cargo mutants -p broker-fees -j 8 --timeout 180 --profile mutants --file 'crates/core/src/guide.rs' --file 'crates/core/src/guide/*.rs' --file crates/core/src/ways.rs --file crates/core/src/history.rs`
(`--file` takes globs and repeats; `--profile mutants`, in `Cargo.toml`, optimises the tests, which take minutes in a debug build and made every mutant time out) and turn the surviving
mutants that matter into tests. It copies the tree for each job, so run it
with no agents' worktrees under `.claude/worktrees` (they once made the
tree 19 GB), and put `TMPDIR` on the big disk if `/tmp` fills up.

"Done means" is for the end of a change. Between the stages of a large
change (the redesign, say), a broken build, failing tests and broken
commits are fine: the owner prefers speed (2026-10-19). Subagents that run
in parallel each get a git worktree. Merge each into `main` as soon as it's
done, then delete the worktree and its branch, so none are left lying
around.

A change a visitor can see is also checked against `REDESIGN.md`'s review
checklist.

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

### Researching a tariff

- **Reading the PDF.** Use `pdftotext -layout file.pdf -`: the Hebrew comes out in reading order.
  - Remove the direction marks (U+200E, U+200F, U+202A–U+202C) before searching.
  - Rows run right to left, the name on the right and the price on the left. A row's price, period and notes can spill onto the lines around it, so read a wide window around each match.
- **Search under every name a fee goes by.**
  - Custody: דמי משמרת, דמי ניהול פיקדון/פקדון, דמי ניהול/טיפול פיקדון.
  - The monthly fee: דמי טיפול, דמי שימוש, and on comparison sites and forums "דמי ניהול (חשבון)", which reads like custody. Excellence's "two free years" are of the ₪15 monthly fee.
  - Leumi's tariff never says משמרת at all.
- **Read each row's fine print, not just its price:**
  - a rate's period ("0.15%" was a year at Altshuler and a quarter at Meitav);
  - a minimum per billing period (Altshuler's ₪75 a month);
  - custody "including units of mutual funds";
  - a fund exemption promised only on the broker's site (IBI);
  - "up to" prices, which are maximums;
  - minimums "in the trade's currency", which become one row per exchange;
  - a US row that covers every US security, bonds and funds included (Excellence).
- **The banks' tariffs come in pieces:**
  - Mizrahi‑Tefahot publishes each part as its own PDF, each with its own date: `/media/smallbusiness4.pdf`, `arutzyashir.pdf`, `specials.pdf`. Merge the ones used with `pdfunite`.
  - Otsar Hahayal's page links the First International's full document; take the numbers from that, not from the "large businesses" securities part beside it.
  - Measure a bank's conversion markup from its own rates page:
    - Mizrahi `/brokerage/foreignexchange/`, plain HTML;
    - the First International `apps.fibi.co.il/Matach/matach.aspx`, Windows‑1255, which gives a mid‑rate;
    - Leumi `bankleumi.co.il/vgnprod/shearim.asp`. The Hebrew comes out garbled but the numbers are fine: representative, then transfers buy and sell, then banknotes buy and sell.
- **The Tel Aviv Stock Exchange** publishes each member's tariffs and its actual average fees ([calculator](https://market.tase.co.il/he/market_data/trading_fees)). They settle unclear rows.
  - **Getting through:** its API answers curl with `Content-Type: application/json`, `Referer: https://market.tase.co.il/` and a browser user agent.
  - **Listing the choices:** `GET https://api.tase.co.il/api/commissions/loadcommissions` lists the members, portfolio sizes and fee keys.
  - **Calculating:** `POST https://api.tase.co.il/api/commissions/calc` with `{"Commission":"1","CommissionType":1,"OrderBy":1,"StockValue":5,"TaseMemberId":null}`.
    - `CommissionType`: 1 is actual averages, 2 is tariffs.
    - Average keys: 1 Tel Aviv stocks and bonds, 2 T‑bills, 3 foreign securities, 4 and 5 custody, Israeli and foreign.
    - Tariff keys: 1 stocks and bonds, 2 T‑bills, 5 abroad, 15 custody.
    - `StockValue` is the portfolio in thousands of ₪, in bands: 1 up to 25, 5 up to 200, 9 above 1,000.
    - `TaseMemberId` null gives every member. IBI is 2303, Altshuler 2618, Excellence 2361, Leumi 1107, Meitav 5018.
  - **When the listing and the document differ** (Meitav: 0.3% listed, 0.25% in the PDF), follow the document, and note the difference in `policies/sources.md`.
- **Joining offers** are on comparison sites; the exchange's averages confirm them.
  - gemeltop.co.il gives the headline prices.
  - tradingil.co.il spells out bonds, index funds, the US price per share, the handling fee's offset and the free years.
  - broker.co.il confirms the offset.
  - What the house itself promises outranks the comparison sites, and turns a reading into a published caveat. Look on its landing page (Excellence: lp.xnes.co.il) and in the offer's rules (תקנון), linked from its disclosure page (גילוי נאות).
- **Pages that fetching won't read:**
  - xnes.co.il's FAQ and disclosure pages are drawn by script, but their text is in the inline `__NUXT__` script. Grep the raw HTML for `question:"` and `answer:"` (`/` stands for `/`).
  - Pages drawn by JavaScript (IBI's fund page): script the web project's Chromium (`web/tests/CLAUDE.md`, "Recipes"). The Playwright MCP plugin has no Chrome on this machine.
  - inter‑il.com answers 403 to curl without a normal Chrome's `User-Agent`, `Accept` and `Accept-Language`.
  - IBKR's www site blocks fetchers, but `gdcdyn.interactivebrokers.com` serves the same pages.
- **The Capital Market Authority's open data** (data.gov.il) answers curl only with the user agent `datagov-external-client`. Its dataset pages are drawn by script, so link to gemelnet.cma.gov.il and bituachnet.cma.gov.il instead.
  - A fund's `AVG_ANNUAL_MANAGEMENT_FEE` is yearly, and changes every January.
  - Savings policies sit inside "policies sold since 2004", with managers' insurance.
- **Sites that don't answer curl:** the Bank of Israel's site and Nevo don't. Hebrew Wikisource has the Income Tax Ordinance and the fund regulations as plain pages, and Kol Zchut has the yearly ceilings.

### Researching what a fund keeps back

- **Measure against the index with gross dividends, in shekels at the Bank of Israel's rate.**
  - Yahoo's `^SP500TR` (and `^GSPC` for the dividends) answers a plain request.
  - The rates come from `edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/BOI.STATISTICS/EXR/1.0/RER_USD_ILS?format=csv`.
  - Against the net (30%) index, every fund that swaps looks better than the index.
- **Maya's fund history:** `POST maya.tase.co.il/api/v1/funds/mutual/{id}/history` with `{"period":4,"fromDate":"…T00:00:00","toDate":"…","pageNumber":n}`.
  - It returns 30 days a page, and a day's price is dated by the New York close it comes from.
  - `GET …/mutual/{id}` has the fund's fixed, variable and trustee's fees.
- **The exchange's history of an ETF:** `POST api.tase.co.il/api/security/historyeod` with `{"dFrom","dTo","oId","pageNum","pType":"8","TotalRec":1,"lang":"0"}` and the Referer `https://market.tase.co.il/`.
  - It returns 30 days a page, prices in agorot, and no price on a day without trading.
  - Chain each day's close over its base price, which carries splits.
  - An ETF closes while New York trades, so average the price‑to‑index ratio over a month at each end.
  - Funder's ETF pages (`funder.co.il/etf/{id}`) list the three fees as plain HTML.
- **Tel Aviv's holidays** can leave a September with a handful of pricing days, so don't ask a month for ten.
- **Gemel Net's returns are before the management fee.** Its older reports name and classify a track differently: find a track by its number from the latest report. The regulator reports a month late.

### Researching the short term's figures

- **The Bank of Israel's deposit rates** are in `boi_files/Pikuah/g060a.xls`, an .xlsx whatever its name. It's the "כאן" link on `.../boi-equator/deposit/`.
  - Sheet L7.6.1a has the fixed rates and L7.6.2a the variable ones. There are nine blocks of terms across, ten banks and the system in each, and a row a month.
  - `policies/deposit-rates.py` finds the blocks by their titles. Run it monthly.
- **boi.org.il answers curl with a Radware challenge** that returns 200 for any address, so a made‑up URL looks real. Read it with a browser instead. Its press releases are named by date (`/publications/pressreleases/23-2-26/`).
- **Maya's fund list** (`POST maya.tase.co.il/api/v1/funds/mutual`) takes at most 30 funds a page and ignored every classification filter tried. So `money-market-funds.py` reads all ~2,000 funds, and keeps those whose `classification.main` is "כספית שקלית".
- **A fund's real cost** is `managementFee + trusteeFee`. A load on buying shows only as a `purchasePrice` above the `redemptionPrice`.

## Tests

Each check goes in the cheapest layer that can catch the bug: Rust, then
the unit tests in Node, then the browser.

- `crates/core/tests/real_tariffs.rs` checks the tariffs against amounts
  worked out by hand from `policies`. A tariff change needs a case there,
  with the arithmetic in a comment; so does a change to the law's rules in
  `vehicles.rs`. Exact amounts there use $1 = ₪3.70 and €1 = ₪4.625 = $1.25:
  dollars and euros convert exactly; shekels to dollars don't (1/3.7 has no
  finite decimal), so those cases round with `cents()`.
- `crates/core/tests/economics.rs` checks what must hold for any plan and
  any investing pattern: where every shekel goes, that no plan beats no
  fees, that raising a fee never helps, the compounding formula, the
  cheapest track; partly as `proptest` properties over random scenarios
  (`PROPTEST_CASES=1000 cargo test` for a deeper run). A change to the
  simulation needs its rule here, and a new test must fail without its fix.
  Properties need realistic horizons (a bug hid in 2-year scenarios and
  showed at 20 years of ₪2,000 a month). Identities through an exchange rate
  hold only to about 1e-20, so allow a millionth of a shekel (`close`). On
  the defaults every usual plan loses between 0.1% and a third of the
  no-fee value over 20 years (`usual_plans_lose_a_plausible_share`): look
  at a change outside that band before widening it.
- `crates/core/tests/guide.rs` has a test for each row of the guide's table
  (`REDESIGN.md`, Appendix A; `situation` and horizon by horizon), the
  cushion's arithmetic, and the acceptance questions as the core answers
  them; `answer.rs` the redesigned answer's rules, among them that the
  outcome always holds the current account and both arenas, and that the
  horizons grow within an arena and change arena once.
  `guide_words.rs` checks the texts exist in both languages and name their
  circumstance (never what they say). `ways.rs` checks the ways and the
  products' costs, `history.rs` the band's arithmetic and ordering. A new
  guide rule needs its row in `guide.rs` and a reason's text; a new kind of
  safe holding needs a hand-worked case in `real_tariffs.rs`.
- **Write an expectation as a rule the core must keep, not as a winner.**
  The previous plan's acceptance table guessed winners and the data
  disagreed; a winner fixed in a test breaks with every monthly data
  update. Let the figure come from the core.
- `cargo bench -p broker-fees` times the calls the page makes on every
  keystroke and the guide's (`recommend`, `answer`, `compare_ways`, divan,
  `benches/hot_paths.rs`); the numbers to expect are below.
- `cargo mutants -p broker-fees -j 8` asks whether the tests would notice a
  change to each line of the core; run it after adding a rule, and turn
  the surviving mutants that matter into tests ("Tools on this machine").
- The web side's layers, and what goes where, are in `web/tests/CLAUDE.md`:
  vitest unit tests beside the code (`src/**/*.test.ts`), Playwright specs
  by part of the app (`questions`, `answer`, `alternatives`, `workbench`,
  `learn`, `acceptance`, besides the kept ones), and the performance suite.

Speed budgets, release build, `cargo bench -p broker-fees` on this machine
(2026-10-19, medians). The guide asks for these once typing pauses and from
the worker, never on a keystroke in the main thread; a change that moves one
by more than a half deserves a look.

| Call                                                         |       Median |
| ------------------------------------------------------------ | -----------: |
| `guide_data` (the tariffs and rates the page builds once)    |        24 µs |
| `recommend`, safe (a date in 5 years: deposit, fund, bond)   |       1.1 ms |
| `recommend`, a cushion first, then an index kind             |       5.7 ms |
| `recommend`, index, 20 years (every way)                     |        19 ms |
| `recommend`, index, no first deposit (tries the unlock line) |        42 ms |
| `compare_ways` (seven ways, each on the plans that hold it)  |        30 ms |
| `compare` every plan, Tel Aviv / USA                         | 5.8 / 9.6 ms |
| `answer`, 3 years safe, without / with horizons (2026-10-20) |   9 / 100 ms |
| `answer`, index, 20 years, without / with horizons           |  31 / 125 ms |
| `answer`, ₪500 a month for 30 years, without / with horizons |  83 / 177 ms |

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
- Sentences about money are the core's (`describe/answer.rs` and its
  siblings), so the Hebrew is written once and both languages are required.
- Dev-dependencies that only tests use go in `[dev-dependencies]`:
  `proptest` for properties, `divan` for benchmarks (`[[bench]] harness =
false`, since it has its own `main`).

## Known traps

- **Svelte:**
  - `$state` deep‑proxies objects, so compare by identity only with `$state.raw`.
  - Never rewrite a field's text on focus: it breaks selecting and typing.
  - A repeated key in a keyed `{#each}` throws at runtime, unseen by svelte‑check. Caveat texts repeat, so `Caveats.svelte` keys by position.
  - `{@const}` must sit directly in a block, not inside an element.
  - A class field read by a `$derived` field's initializer needs a default where it's declared. Keep parameter properties (`constructor(private x)`) out of `.svelte.ts`.
  - `bind:this` becomes `null`, not `undefined`, when its element unmounts.
  - Text next to `{#if}` inside an element can lose its space when Prettier rewraps the line. Build such text in one expression.
  - A tip's words are styled by the component that writes them (the snippet keeps its writer's scope). Give a header's tip content its own classes, and scope cell rules to `td`.
  - The details `<dialog>` is `{#key}`ed per plan and broker, so it scrolls back up. A draft isn't keyed: rebuilding the editor would lose focus.
- **ECharts:**
  - ECharts draws only the workbench's charts; the guide's outcome chart and gain bars are HTML, so their labels are measured on a canvas and placed by `outcome.ts`/`gain.ts`.
  - Names are fitted with `fitName`; hover and the rows' linking go through `dispatchAction`.
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
    where the line is: a glide cut short jumps (`drawAt` waits). ECharts
    doesn't animate a line at all when a point moves more than 3,000 px.
  - Labels that must move with their bars ride on the bars (series labels):
    a category axis's labels stay in place while the bars slide. A line's
    end label doesn't glide unless the attachment lets it
    (`releaseEndLabels`); `valueAnimation` stops it fading in, and one
    `updateLabelLayout()` after the first draw (never later) places them.
  - Events carry `seriesIndex`, not `seriesId`.
  - Never let ECharts wrap or cut a label (`overflow`): zrender 6 guesses
    every letter outside ASCII to be as wide as a Chinese one, so Hebrew
    broke at half the width. Measure on a canvas instead (`fitName`).
  - Tooltips are `confine`d and the chart card clips (`overflow: clip`):
    ECharts shows a tooltip mid-chart before moving it to the pointer, and
    on a phone anything past the screen's edge zooms the page out (a jump);
    past the left edge, a right-to-left page scrolls sideways to it.
  - A `markLine` is drawn over the series and their labels (`z: 5`); `z: 1`
    puts it under the bars.
  - zrender reads CSS `cubic-bezier()` strings as easing, so the charts use the page's curve. `createCubicEasingFunc` gives Svelte the same function: import it with `.js`.
  - A clear part that must take the pointer (a row's outline) is filled `transparent`, not `silent`.
  - What a keystroke costs on a phone is mostly `setOption` and the first frame (about 40 of 55 ms), so draw after the paint.
  - Check a stacked bar's colors with the dataviz skill's validator over every pair (`--pairs all`) when a part can be empty.
- **CSS:**
  - A grid child with a wide table needs `minmax(0, 1fr)`, or the page widens.
  - A table sized by its contents moves every column when a note comes or
    goes: the results table (`Within`) gives the name's column a set share (`width`)
    and split the rest. A warning in the best plan's card is a ⚠ beside its
    amount (`WarningSign`), its words in the table's row below: a line of
    its own moved the table, and an empty line kept for it looked broken.
    A table wider than its box ignores `width` but keeps `min-width`.
  - The tables' box (`.scrolls`) scrolls sideways only: rows sliding to new
    places reach past its bottom for a moment, which showed a vertical
    scroll bar.
  - A chosen button must keep its width: bold wrapped the row (and pinned
    names in bold wrapped differently about one render in twenty).
  - The Baseline plugin rejects `overscroll-behavior` (Safari lacks it on the
    page root) and, until October 2026, `:popover-open`: tips carry an `open`
    class from `tip.ts` instead.
  - Cards are spaced by `section ~ section`, not `+`: dialogs sit between
    cards.
  - A `position: fixed` element inside a `<dialog>` is placed by the window
    only while the dialog has no transform at rest (`translate: none`).
  - A box that animates its own height must start before the next paint:
    `glideHeight` measures in a MutationObserver, not a ResizeObserver. A
    per-frame probe can see a layout that is never painted; what was painted
    is in the layout-shift entries.
  - A height set from the element's own observed width makes a
    ResizeObserver loop (a page error on the iPad): follow a media query
    instead (`NARROW_SCREEN`). A tip starts at the corner with
    `width: max-content`, or it sizes itself by where it happened to sit.
  - `grid-template-rows: subgrid` computes to `none` on a container
    (`container-type`): `.stat` sets `container-type: normal`.
  - A line may break before an inline block (a "?") even with no space;
    keep them in a flex row (`.titled`). Chrome breaks before a "·" only
    when the words before it just fill the line: check every width.
  - Alternatives that take turns share one grid cell (`grid-area: 1 / 1`,
    the unused ones `visibility: hidden`), so the cell is as big as the
    biggest and nothing moves.
  - A native `<select>` is as wide as its longest option: keep options
    about 12 characters.
  - An `IntersectionObserver` fires only when a threshold is crossed: a jump
    straight past the element crosses none (the best bar also listens to
    `scroll`).
- **The core:**
  - `simulate` tries every track, keeps the first of the cheapest, and reports it in `Outcome::track`. The web passes that to `feesFor` and `copyOf`.
  - Listed tariffs keep the document's row order, and a copy is rewritten most specific first (`most_specific_first`).
  - The no‑fee baseline, `free_plan`, buys fractions of shares everywhere. Test plans built from it (`priced`) set `fractions_on: vec![]`.
  - `PlanKey::Listed` is a position in `tariffs::all()`, which shifts when brokers are added. Your plans find their original by names (`basedOn`).
  - The biggest order, which decides `.when_above` caveats, is the no‑fee plan's `largest_trade`: almost always the sale at the end.
  - A plan of the user's own has no caveats.
  - **Across the WASM boundary:**
    - serde‑wasm‑bindgen rejects a fraction for a `u32`;
    - a `#[serde(flatten)]` struct reaches JavaScript as a `Map`, which JSON saves as `{}` (`#[tsify(hashmap_as_object)]`);
    - new optional editor fields take `#[serde(default, skip_serializing_if = "Option::is_none")]` and camelCase names.
- **The core in the browser:** nothing may call it while a module loads
  (a top-level `const x = core.f()`): the WebAssembly isn't there yet, and
  only a browser shows it, since the unit tests load the core first. Read
  it in `AppState`.
- **Loading:** `index.html` draws the page's shape before any script (its
  styles repeat app.css on purpose), and `vite.config.ts` adds a preload for
  the `.wasm`; check `dist/index.html` that the wasm request starts with the
  JavaScript's.

## How the page looks and behaves

`REDESIGN.md` Part 1 decides what is shown and asked. These conventions decide how.

- **Show only what applies.** Caveats and fees are filtered by the chosen security and exchange: nothing about conversion on Tel Aviv. Wording follows the choices ("Your bond can be bought…").
- **Phones matter as much as desktop.** Test on a Samsung, not a Pixel, and on the narrowest iPhone (`devices['iPhone SE (3rd gen)']`). Scrolling with a finger must never pick anything.
- **Jargon is explained where it appears**, by the identity line (`REDESIGN.md` §2.8) and by the learn card behind "מה זה? ›". A "?" tip may add detail. It never carries what's needed to read a chart or to act.
- **The learn card** has one shape (`LearnCard`): the term; its names (the bank's, the ads', in English); what it is, what it's good for, its downside, what it costs, its tax, where you meet it; an example; and what to compare it with. The comparison is chips that open the other card in the same sheet, with a way back. Every claim rests on a source linked under it. A tip stays one paragraph, and ends with "עוד על זה ›" when it has a card.
- **The register.** Never "ההמלצה שלנו", no personal advice, no "you should".
  - "מה שמתאים" is said only of a clear answer (`REDESIGN.md` §2.3, state A).
  - The past is said to be the past ("ב־N מתוך M תקופות של X שנים נגמרה בהפסד"), and no figure is rounded to "never".
- **Amounts are in today's money** by default, with the nominal figure as a note, and every axis says so.
- **Keep the main thing prominent.** Secondary information stays subtle:
  - the exchange rates are folded into one line;
  - the broker's name under each plan is de‑emphasized;
  - ties are shown as ties, without ranks.

  Groups of inputs go in separate cards.

- **One style:**
  - one tooltip style everywhere;
  - outlined buttons, as tall as a choice (33 px); text fields and menus 37 px; tints of the accent and warning from the tokens on `:root`;
  - units inside the fields, beside the number (`₪ 10,000`, `10 %`);
  - every field one width (`--input-field`), so they line up from card to card; the labels take the rest and wrap;
  - a "?" after words that may wrap sits in a flex row with them (`.titled`);
  - while a table scrolls sideways, the screen ends on a whole column (the container steps in `Within`);
  - motion through the tokens in `app.css` (`motion.ts` for script):
    - `--ease-out` for what arrives, and `--ease-in` with less time for what leaves;
    - presses sink a little;
    - the results move to new numbers together for `--settle` (400 ms): the charts glide, the numbers roll, the rows slide, and a box whose words change its height glides to it (`glideHeight`);
    - a chart never cuts a glide short, and waits for a typed number to pause (`HeldWhileTyping`);
    - what comes and goes opens and closes (`transition:reveal`);
    - nothing moves under `prefers-reduced-motion`.
- **Charts:**
  - every mark that matters is labelled directly, and nothing is identified by colour alone (principle 10);
  - values are after tax, in today's money, so the chart agrees with the answer (principle 11);
  - time is shown as an axis, never a slider, and never in 3D;
  - each part of a stacked bar has one colour, the same in every bar;
  - a plan's or a place's own colour is only beside its name (and its outline when pinned);
  - the table and charts are linked, for hovering and pinning; the other lines fade only a little.
- **The dark theme only.** It should feel fast.

## Working with the user

- **Ask first.** Before building any visible choice (layout, wording, what's shown or hidden), offer options with mockups and recommend one, even inside an approved task. Batch the choices into one question: up to four, each with a preview and a recommendation, once the plumbing works and a screenshot shows the real problem.
- **For anything that moves or responds, a live page beats ASCII.** The motion options and the 2026-10-19 redesign were both chosen from published mockups.
- **Feedback points at a cause.** Fix the structure rather than adding an option, a tooltip or a note: an "I don't care" option and more tooltips were rejected (`REDESIGN.md` principle 9).
- **Bug hunts:** keep going until the checks find nothing, and first prove each check catches the bug on a page that has it. In the owner's words: "there will be no more problems when you can't find any, not when you get tired of searching for them".

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
  reads backwards. The charts stay left to right. Floating UI's sides are
  physical too: the plan preview opens `left-start`.
- Every spec drives the Hebrew page, from an English browser (Playwright's
  default locale), which `tests/hebrew.spec.ts` checks still gets Hebrew.
  Specs name the page's words by `t` and the core's by the core
  (`tests/core.ts`: plans by their English label, `securityName`,
  `exchangeName`, `brokerName`), and write Hebrew out only to check
  wording. A Hebrew button label ending in a plan's label is a prefix of
  its siblings': find such buttons with `exact`.

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
  הפקדה; a fund's track on the S&P 500 → מסלול מדד S&P 500 (not a bare
  מסלול עוקב מדד).
- the two arenas → סכום בטוח יותר, מדד S&P 500, לטווח ארוך (`arena_name`); a money
  market fund → קרן כספית; a fixed-rate deposit → פיקדון בריבית קבועה (a
  bank's monthly savings plan is a תוכנית חיסכון, not a פיקדון); a place to
  keep the money, in the table of the safe kinds → אפיק; the Bank of
  Israel's rate → ריבית בנק ישראל.
- an Israeli government bond → אג״ח של מדינת ישראל, said with what it is
  linked to (צמודה למדד המחירים לצרכן / שקלית) and held to maturity; its
  trade name (״ממשלתי צמוד 0529״) only as what to search for; a short one
  is a מק״מ. A CPI-linked deposit → פיקדון צמוד מדד; the safety cushion →
  כרית ביטחון; the yield curve → עקום תשואות; yield to maturity → תשואה
  לפדיון; a sum known in advance → סכום ידוע מראש; money you may need any
  day → כסף שאולי תצטרכו בכל יום; today's money → כסף של היום; compound
  interest → ריבית דריבית; the S&P 500 index → מדד S&P 500; a bank's branch
  price list → תעריף סניף (the plan "Branch or through a banker").
- never a bare "מדד": say which (מדד המחירים לצרכן, מדד S&P 500). Within one
  unit (an option card, a stated line, a chart row with its label, an
  identity line) "המדד" may refer back to the one index named earlier in it,
  never where two are named; say what a product is before its trade name
  (`REDESIGN.md` §2.8; `guide_words.rs`, `bare_index_in_unit`).
- the answer's words (`REDESIGN.md` §2.2–2.4): the chips → כאן קשה לטעות /
  ההבדל קטן / תלוי במה שחשוב לכם; a line's pill → ברור / תיקו / לפי
  הכללים / החלטה שלכם / בחרתם; the plan → התוכנית שלכם; when it fits →
  מתאים אם…; the reference row → נקודת השוואה.

- A " · " shown in Hebrew has a non-breaking space before it (` · `,
  `&nbsp;·` in markup), so a line may end with the dot but never start
  with it. English labels keep a plain space: links find plans by them.
- Hebrew inside English goes in `<bdi lang="he">`. In Rust strings, join a
  phrase's words with `\u{a0}`: a phrase split across lines reads backwards.
  A hyphenated Hebrew word is joined with a non-breaking hyphen
  (`חד\u{2011}פעמית`, `‑` in TypeScript), or a narrow line breaks it
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
contact sheet (`web/tests/CLAUDE.md`, "Recipes"), and only at the ones that changed.

## Tools on this machine

- **Python:** the system Python has no Pillow: `uv run --with pillow python script.py`.
- **Cargo:** the packages are `broker-fees` (the core) and `broker-fees-wasm` (the bindings), as in `cargo test -p broker-fees`.
- **Permissions:** when the permission classifier blocked Bash and WebSearch, the user allowed them in `.claude/settings.local.json`.
- **Killing processes:** `pkill -f <pattern>` kills the shell running it when the pattern appears in that command (exit code 144). Stop a server by its port instead: `fuser -k 4173/tcp`. The same goes for waiting: `until ! pgrep -f "cargo mutants"; do sleep 30; done` matches its own shell and never ends (agents left four such loops running for hours). Run the command itself in the background, or match the process's name (`pgrep -x cargo-mutants`).
- **Playwright's project flag:** `npx playwright test --project desktop tests/x.spec.ts` fails; write `--project=desktop`.
- **Stale dev server:** an `npm run dev` server left running across a package upgrade served stale CSS, and Playwright reuses a server on 5173. Restart it: `fuser -k 5173/tcp`, then `npx vite --host 0.0.0.0 --port 5173`.
- **Vitest:** it runs `.svelte.ts` modules against Svelte's server build unless `resolve.conditions: ['browser']` is set (`vitest.config.ts`). Effects then never run, and deriveds don't track.
- **cargo mutants:** `cargo mutants -p broker-fees -j 8 --timeout 180`, about 8 minutes for the core.
  - It runs only that package's tests; `--workspace` runs them all.
  - The bindings' `#[wasm_bindgen]` entry points always show as missed, since they run only in a browser.
  - The misses that remain after the first round are mostly equality moves (`>` to `>=`) and text replacements.
- **Coverage:** `cargo llvm-cov -p broker-fees --summary-only` needs the `llvm-tools-preview` component.

## Open ends

- **The link preview** (`web/public/og.png`) still shows "Broker fees, compounded" in English, from before the page was Hebrew only.
- **Scrolling a dialog:** scrolling to the end of one scrolls the page behind it. The fix, `overscroll-behavior: contain`, is rejected by the Baseline plugin; a lint exception is the owner's call.
- **Tables while typing:** the summary and the tables still change height when their words do (they glide there). Holding the tables while typing, as the charts do, is the owner's call.
- **Unticked plans with tracks:** such a plan has no picked track, so its fee list shows every track instead of the cheapest.
- **Markups:** re‑measure Leumi's markup when the numbers are re‑checked. Interactive's market spread isn't counted.

## Deploying

The site is GitHub Pages, at
https://orishamir.github.io/Israeli-Broker-Search/, built and deployed by
`.github/workflows/deploy.yml` on every push to `main` (or by hand from the
Actions tab). Its `test` job gates the deploy: the Rust checks and tests on
stable, the unit tests, the linters, and Playwright's desktop project; the
phone projects and the performance suite stay local. Vite's
`base: './'` makes the build work under the repository's path. The
build itself (`npm run build`) never touches the tariffs' PDFs. Push to
`main` only when the owner says so: the deploy runs on every push.
