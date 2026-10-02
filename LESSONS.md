# Lessons learned

What was learned building this app that `CLAUDE.md`, `web/tests/CLAUDE.md`
and `policies/sources.md` don't already say: traps that cost time, and ways of
working that paid off. Written for whoever works on it next, AI agents most of
all. Read those three first.

## Researching a broker's fees

- Read a tariff PDF with `pdftotext -layout file.pdf -`: Hebrew comes out in
  reading order. Remove the direction marks (U+200E, U+200F, U+202A–U+202C)
  before searching. Rows run right to left (the name on the right, the price
  on the left), and a row's price, period and notes can spill onto the lines
  around it: read a wide window around each match.
- Search under every name a fee goes by. Custody: דמי משמרת, דמי ניהול
  פיקדון/פקדון, דמי ניהול/טיפול פיקדון. The monthly fee: דמי טיפול, דמי שימוש,
  and on comparison sites and forums "דמי ניהול (חשבון)", which reads like
  custody: Excellence's "two free years" are of the ₪15 monthly fee, and its
  package page says "no custody" apart from them. Leumi's tariff never says
  משמרת at all.
- Read each row's fine print, not just its price. What changed the model this
  time:
  - a rate's period ("0.15%" was a year at Altshuler and a quarter at Meitav);
  - a minimum per billing period (Altshuler's ₪75 a month);
  - custody "including units of mutual funds" (most brokers), or a fund
    exemption promised only on the broker's site (IBI);
  - "up to" prices, which are maximums;
  - minimums "in the trade's currency", which become one row per exchange;
  - a US row covering every US security, bonds and funds included
    (Excellence).
- The banks' tariffs come in pieces, and their sites mislead a little:
  - Mizrahi-Tefahot publishes each part as its own PDF
    (`/media/smallbusiness4.pdf` for securities, `arutzyashir.pdf` for the
    direct channels, `specials.pdf` for customer groups), each with its own
    date; `pdfunite` merges the ones used into one file for `policies`.
  - Otsar Hahayal's tariff page links the First International's full
    document (it's the same bank), and beside it a securities part headed
    "for large businesses" with other rates: take the numbers from the full
    document.
  - A bank's conversion markup is measured from its own rates page, as
    Leumi's was: Mizrahi's is plain HTML
    (`/brokerage/foreignexchange/`); the First International's
    (`apps.fibi.co.il/Matach/matach.aspx`) is Windows-1255, so decode it
    before reading, and gives a mid-rate to measure from.
- The Tel Aviv Stock Exchange publishes each member's tariffs and its actual
  average fees
  ([calculator](https://market.tase.co.il/he/market_data/trading_fees)). They
  settle unclear rows and show what customers really pay. Its API works from
  curl with `Content-Type: application/json`,
  `Referer: https://market.tase.co.il/` and a browser user agent:
  - `GET https://api.tase.co.il/api/commissions/loadcommissions` lists the
    choices: members, portfolio sizes, fee keys.
  - `POST https://api.tase.co.il/api/commissions/calc` with
    `{"Commission":"1","CommissionType":1,"OrderBy":1,"StockValue":5,"TaseMemberId":null}`.
  - `CommissionType` 1 is actual averages, 2 is tariffs. Average keys: 1 Tel
    Aviv stocks and bonds, 2 T-bills, 3 foreign securities, 4 custody on
    Israeli securities, 5 custody on foreign ones. Tariff keys: 1 stocks and
    bonds, 2 T-bills, 5 abroad, 6 options, 7 futures, 15 custody, 16–17 orders
    in offerings.
  - `StockValue` is the portfolio in thousands of ₪: 1 up to 25, 2 to 50, 3
    to 75, 4 to 100, 5 to 200, 6 to 400, 7 to 700, 8 to 1,000, 9 above.
    `TaseMemberId` null gives every member (IBI 2303, Altshuler 2618,
    Excellence 2361, Leumi 1107, Meitav 5018).
- Investment houses publish only maximums. Their real joining offers are on
  comparison sites (gemeltop.co.il, tradingil.co.il), and the exchange's
  averages confirm them. What the house itself promises is on its landing
  page (Excellence: lp.xnes.co.il, the "joining package") and in the offer's
  rules (תקנון), a PDF linked from its disclosure page (גילוי נאות); those
  outrank the comparison sites, and turn a reading into a published caveat.
- xnes.co.il's FAQ and disclosure pages are drawn by JavaScript, but their
  text is in the page's inline `__NUXT__` script: grep the raw HTML for
  `question:"` and `answer:"`, with `\u002F` for `/` in links.
- Some broker pages are drawn by JavaScript (IBI's fund page): script the
  web project's Chromium (see Recipes) instead of fetching the HTML; the
  Playwright MCP plugin has no Chrome on this machine. inter-il.com answers
  403 to curl without browser headers (`User-Agent`, `Accept`,
  `Accept-Language` of a normal Chrome get through, as does a normal Chrome
  `userAgent` in `browser.newContext`). IBKR's www site blocks fetchers too,
  but `gdcdyn.interactivebrokers.com` serves the same pages.
- The comparison sites don't list the same things. gemeltop.co.il's per-broker
  pages give the headline prices (Tel Aviv stocks and ETFs, the US minimum,
  the handling fee); tradingil.co.il's per-broker pages spell out bonds,
  index funds, the US price per share, whether the handling fee is offset by
  trade fees, and their own free years; broker.co.il confirms the offset.
  Pepper's investing FAQ is on pepper.co.il's home page (its /invest pages
  are gone).
- The exchange's tariff listing can differ from a broker's document (Meitav's
  non-US rate: 0.3% listed, 0.25% in the PDF). The document is followed and
  the difference noted in `policies/sources.md`.

- The Capital Market Authority's open data (data.gov.il) answers a plain
  `curl` only with the user agent `datagov-external-client`; its dataset
  pages are drawn by script, so link to gemelnet.cma.gov.il and
  bituachnet.cma.gov.il instead. The fee a fund reports
  (`AVG_ANNUAL_MANAGEMENT_FEE`) is a yearly figure that changes every
  January. Bituach Net has no class for savings policies: they're inside
  "policies sold since 2004", with managers' insurance.
- The Bank of Israel's site and the ordinance on Nevo don't answer `curl`;
  Hebrew Wikisource has the Income Tax Ordinance and the fund regulations
  as plain pages, and Kol Zchut the yearly ceilings.

## Researching what a fund keeps back

- **Measure against the index with gross dividends, in shekels at the
  Bank of Israel's rate.** Yahoo's `^SP500TR` (and `^GSPC` for the
  dividends) answers a plain request for daily closes; the Bank of Israel's
  rates come from `edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/
  BOI.STATISTICS/EXR/1.0/RER_USD_ILS?format=csv`, which answers `curl`
  where boi.org.il doesn't. Against the net (30%) index every fund that
  swaps looks better than the index.
- **Maya's fund history** is a POST to
  `maya.tase.co.il/api/v1/funds/mutual/{id}/history` with
  `{"period":4,"fromDate":"…T00:00:00","toDate":"…","pageNumber":n}`, 30
  days a page; a day's price is dated by the New York close it's from. A
  fund's page (`GET …/mutual/{id}`) has its fixed, variable and trustee's
  fees.
- **The exchange's history of an ETF** is a POST to
  `api.tase.co.il/api/security/historyeod` with
  `{"dFrom","dTo","oId","pageNum","pType":"8","TotalRec":1,"lang":"0"}`
  (and the Referer `https://market.tase.co.il/`): 30 days a page, prices in
  agorot, a day without trading has no prices. Chain each day's close over
  its base price, which carries splits. An ETF closes while New York
  trades, so a single day's comparison is off by up to a day's move: average
  the price-to-index ratio over a month at each end. Funder's ETF pages
  (`funder.co.il/etf/{id}`) list the three fees as plain HTML.
- **Tel Aviv's holidays** can leave a September with a handful of pricing
  days: don't ask a month for ten.
- **Gemel Net's returns are before the management fee**, and its older
  reports (resources `91c849ed-…` for 1999–2022, `2016d770-…` for 2023;
  Bituach Net `584e6b69-…`, `672090ba-…`) name and classify a track
  differently: find a track by its number from the latest report. The
  regulator reports a month late.

## Researching the short term's figures

- **The Bank of Israel's deposit rates** are in an Excel behind its
  comparison page, not only in the Power BI dashboard:
  `boi_files/Pikuah/g060a.xls` (an .xlsx whatever its name), found as the
  "כאן" link on `.../boi-equator/deposit/` (the old address redirects
  there). Sheet L7.6.1a is fixed rates, L7.6.2a variable; nine blocks of
  terms across, ten banks and the system in each, a row a month.
  `policies/deposit-rates.py` finds the blocks by their titles.
- **boi.org.il answers `curl` with a Radware challenge** that returns 200
  for any address, so a made-up URL looks real. Read it with a browser
  (the text recipe below); its press releases are named by date
  (`/publications/pressreleases/23-2-26/`), and its list of them loads late.
- **Maya's API** (`maya.tase.co.il/api/v1/funds/mutual`, a POST) takes at
  most 30 funds a page (`pageSize` above 30 is a 400) and ignored every
  classification filter tried, so `money-market-funds.py` reads all ~2,000
  funds and keeps the ones whose `classification.main` is "כספית שקלית".
  Plain Python gets through with a browser's user agent and referer; some
  filter names got a 403 from its firewall instead of an error.
- **A fund's real cost** is `managementFee + trusteeFee`; a load on buying
  shows only as `purchasePrice` above `redemptionPrice` (Barak's 0.1%).
  The distribution fee is the manager's payment to the bank.

## Tools on this machine

- The system Python has no Pillow: `uv run --with pillow python script.py`.
- The Cargo packages are `broker-fees` (the core) and `broker-fees-wasm` (the
  bindings): `cargo test -p broker-fees`.
- `toMatchAriaSnapshot` passes on a page that has more than the snapshot:
  `inputs-desktop.aria.yml` went two brokers behind unnoticed. After adding
  to the inputs, re-record it (`--update-snapshots=all -g "as text"`);
  `--update-snapshots=changed` re-records only the pictures that differ.
- `npx playwright test --update-snapshots tests/x.spec.ts` fails: the flag
  takes an optional mode and swallows the path. Write
  `--update-snapshots=all tests/x.spec.ts`.
- When the permission classifier blocked Bash and WebSearch, the user allowed
  them in `.claude/settings.local.json`.
- `pkill -f <pattern>` kills the shell running the command itself when the
  pattern appears anywhere in that command (exit code 144, nothing after it
  runs). Stop a server by its port instead: `fuser -k 4173/tcp`.
- `npx playwright test --project desktop tests/x.spec.ts` fails: the
  project flag takes the path as a second project. Write `--project=desktop`.
- A `npm run dev` server left running across a package upgrade served
  stale CSS for components edited later: the markup updated, but the
  compiled style module lacked rules the compiler emits for the source
  (checked with `svelte/compiler`'s `compile` in Node). Playwright reuses
  a server on 5173, so it would have tested the stale styles too. Restart
  it: `fuser -k 5173/tcp`, then `npx vite --host 0.0.0.0 --port 5173`.
- Vitest runs the `.svelte.ts` modules against Svelte's *server* build
  unless `resolve.conditions: ['browser']` is set (`vitest.config.ts`):
  effects then never run and deriveds don't track, and only tests that
  read state after changing it notice. A probe that adds to a `SvelteSet`
  and reads a `$derived` of it tells the two apart in a second.
- `cargo mutants -p broker-fees -j 8 --timeout 180` takes about 8 minutes
  for the core's 600-odd mutants (2026-09-29), in copies of the tree, so
  editing meanwhile is fine. It runs only that package's tests: a "missed"
  mutant may be caught by the wasm crate's tests (`--workspace` runs them
  all, and mutates the bindings too). Most of the 95 first misses were
  text (`Explained` strings, `is_nothing`), the editor's row operations and
  the sale's conversion; the tests added for them are in the `tests`
  modules of `describe.rs`, `yours.rs` and `simulation.rs`; the core's
  misses then fell to 20, all equality moves (`>` to `>=`), "any other
  text" replacements, or changes no listed tariff can reach. Even with
  `--workspace` each mutant is tested by its own package's tests only, so
  the bindings' `#[wasm_bindgen]` entry points always show as missed: they
  only run in a browser, where vitest and Playwright cover them.
- `cargo llvm-cov -p broker-fees --summary-only` (needs the
  `llvm-tools-preview` component) put the core at 95% of lines before the
  economics tests; `describe.rs` and `yours.rs` had the gaps.
- Dev-dependencies that only tests use go in `[dev-dependencies]`:
  `proptest` for properties over random inputs, `divan` for benchmarks
  (`[[bench]] harness = false`, since it has its own `main`).

## The core

- `simulate` tries every track, keeps the first of the cheapest and reports
  it in `Outcome::track`; the web passes that to `feesFor` and `copyOf`.
- Listed tariffs keep the document's row order; a copy is rewritten most
  specific first (`most_specific_first`). Only Altshuler's "New customers"
  loses rows in the rewrite, since its offer row is laid over the regular
  list; a test checks that the other plans are only reordered.
- The no-fee baseline, `free_plan`, buys fractions of shares everywhere, so
  money waiting for a whole share counts as lost to fees. It once bought whole
  shares, and a broker selling fractions then beat "no fees". Test plans built
  from it (`priced`) set `fractions_on: vec![]`.
- Property tests need realistic horizons: that bug hid in a 2-year scenario
  and showed at 20 years of ₪2,000 a month. `no_plan_ends_with_more_than_no_fees`
  uses the app's defaults. Check that a new test fails without its fix.
- Exact amounts in `real_tariffs.rs`: $1 = ₪3.70 and €1 = ₪4.625 = $1.25.
  Dollars and euros convert exactly; shekels to dollars don't (1/3.7 has no
  finite decimal), so those cases round with `cents()`.
- Across the WASM boundary:
  - serde-wasm-bindgen rejects a fraction for a `u32`: the web rounds free
    months before sending them.
  - A struct with a `#[serde(flatten)]` field reaches JavaScript as a `Map`,
    which JSON saves as `{}`: `PlanData` has `#[tsify(hashmap_as_object)]`.
  - New optional editor fields take
    `#[serde(default, skip_serializing_if = "Option::is_none")]` and camelCase
    names: TypeScript sees `field?: T`, and older saved plans still load.
- `PlanKey::Listed` is a position in `tariffs::all()`, which shifts when
  brokers are added; your plans find their original by names (`basedOn`:
  broker, plan, track).
- Caveats: `describe::sort_caveats` splits them into the ones that matter to
  a `Buying` (security, exchange, biggest order), grouped by `CaveatKind`
  most serious first, and the rest with what they're about. `FeeLine.mark`
  is the most serious kind among the caveats `.about_fee()` that fee. The
  biggest order is the no-fee plan's `largest_trade`, almost always the sale
  at the end: over 20 years it's far above any per-order threshold, so
  Pepper's "$4 up to $8,000" shows for nearly everyone, and only a short,
  small plan hides it. The web passes a `Purchase` (with the rates, for a
  dollar threshold on a euro trade); without a comparison the biggest order
  is unknown and thresholded caveats show.
- `Broker::may_cost_more` joins the summaries of the "may cost more" caveats
  that matter: one line under the plan in the table. A plan of the user's
  own has no caveats (its fees are their own claim), so it's never flagged.
- Decimal arithmetic through an exchange rate isn't exact: ₪ to $ divides
  by 3.7, which has no finite decimal, so identities such as "deposits =
  value + fees" hold to about 1e-20 and a test must allow a millionth of a
  shekel (`close` in `economics.rs`). Tel Aviv amounts stay exact.
- On the app's defaults every usual plan loses between 0.1% and a third of
  the no-fee value to fees over 20 years (`usual_plans_lose_a_plausible_share`);
  a tariff or model change outside that band deserves a look before the
  band is widened.
- The core's hot paths in release, 2026-09-30 (`cargo bench -p broker-fees`):
  comparing every listed plan (19 of them) 3.9 ms on Tel Aviv and 6.6 ms
  abroad (the tracks), one plan 0.57 ms, a plan's fees in words 3 µs, the
  editor's full view 6.5 µs, a plan through JSON 6.5 µs. The comparison is
  the only call that costs anything on a keystroke. Working out the tax
  added nothing measurable to one plan (0.56 ms the day before, with 13
  plans compared in 2.7 and 4.8 ms).

## The web app

- Only the chosen calculator is on the page (`{#if}` in `App.svelte`);
  the other's state lives on in `AppState` (`app.short`), so switching
  keeps what was typed. Both calculators' deposit fields keep the ids
  `first-deposit` and `monthly-deposit`, so labels and tests find them the
  same way in either.
- A class field set in the constructor and read by a `$derived` field's
  initializer is "used before its initialization" to TypeScript, though a
  derived is only read after the constructor: give the field a default
  where it's declared (`ShortTermState.expert`). Parameter properties
  (`constructor(private x)`) are TypeScript that emits code; keep them out
  of `.svelte.ts`, where Svelte only strips types.
- A flag every place of a kind shares ("August's rates, before a cut") is
  said once, for the kind: in the list of places under the kind's name, and
  in a line under the table. Repeated under each deposit it took 7 of 8
  rows. A place's own flag stays under its row, and the best place's card
  shows either as a ⚠ beside its amount (`WarningSign`).
- Caveat texts repeat across coverages, so `Caveats.svelte` keys by position.
  `app.spec.ts` opens every plan's dialog and fails on any page error
  (`each_key_duplicate` blanked a dialog): it covers new brokers by itself.
- `{@const}` must sit directly in a block (`{#if}`, `{#each}`…), not inside an
  element.
- A tip's words are styled by the component that writes them, not by
  `Tip.svelte`: the snippet keeps its writer's scope, so that component's
  descendant selectors reach into the tip. `PlaceTable`'s `.place div`
  (the row's mark beside its names) laid the "אפיק" tip's kind names out
  beside their paragraphs, since the header cell is `.place` too; the rule
  is `td.place > div` now. Give a header's tip content its own classes, and
  scope cell rules to `td`.
- Cards are spaced by `section ~ section`, not `+`: the lists' dialogs
  (`PlanSheet`, `PlaceSheet`) sit in the page between cards, so "Taking the
  money out" sat flush under "What's compared" until `layoutProblems`
  got a rule for cards that touch.
- A `position: fixed` element inside a `<dialog>` is placed by the window
  only while the dialog has no transform at rest (`translate: 0` or
  `scale: 1` count as one): the list of plans rests at `translate: none`,
  so the plan preview inside it lands beside the row, not offset by the
  dialog. The dialog's own scrolling doesn't clip it either.
- Two dialogs can be open at once: a plan's details or the editor opens
  over the list of plans, and closing it goes back to the list. Tests name
  the one they mean (`details` and `plansList` in `tests/fixtures.ts`),
  and close the list before using the page behind it: a modal dialog makes
  the rest of the page inert.
- Text next to `{#if}` inside an element can lose its space when Prettier
  rewraps the line ("Tracks for" became "Tracksfor"): build such text in one
  expression.
- The details `<dialog>` is one element for everything it shows, so it kept
  the last plan's scroll and its open `<details>`. The plan and broker views
  are `{#key}`ed and scroll back up when the subject changes. A draft isn't
  keyed: `app.details` changes on every keystroke, and rebuilding the editor
  would lose focus.
- A native `<select>` is as wide as its longest option. A long option pushed
  one out of phone dialogs, and capping its width cut the text off instead.
  Keep options short (about 12 characters fit the smallest phones at 16px);
  `layoutProblems` flags cut-off dropdowns.
- Table cells are `nowrap`: a long warning widened the table past its card
  until warnings and notes were allowed to wrap. Sized by what was in it,
  the plan's column still grew with its longest note (498 to 631 px for an
  index fund in the USA, on a 2000 px screen), moving every number when the
  security or exchange changed: the name's column now has a set share of
  the table (30%) and the rest split what's left, so a note makes its row
  taller instead.
- A line that comes and goes in a summary card moves everything under it,
  and an empty line kept for it looked broken to the user (2026-10-01): the
  best plan's warning became a ⚠ beside its amount, its words in a tip and
  under the plan in the table's first row. Measure such a change over
  every security and exchange, at several widths, not in one screenshot.
- `overflow-x: auto` makes `overflow-y` auto too: while the rows slid to
  new places (`animate:flip`), they reached past the table's new bottom,
  and its box showed a vertical scroll bar for a few frames. The box now
  has `overflow-y: hidden`.
- An `IntersectionObserver` fires only when a threshold is crossed: a jump
  straight past the element (`scrollIntoView`, a tap on the phone's bar with
  the best plan) crosses none, and the bar stayed. The bar also listens to
  `scroll`. Its parts sit in named grid areas: with `grid-row` on one item
  and `grid-column` on another, auto-placement put the arrow first.
- Svelte sets a `bind:this` to `null`, not `undefined`, when the element
  unmounts (the summary, on bad inputs): test with `!element`.
- Right to left (Hebrew): logical CSS properties mirror the layout for
  free, but `offsetLeft` stays physical, so the choices' highlight keeps
  `left: 0`. A line mixing numbers, `=` and `·` reads backwards in RTL
  unless it's wrapped in `<bdi dir="ltr">` (the exchange rates). ECharts
  is left to right whatever the page is, which is right for axes. Anything
  placed from a measured box is physical too: the plan preview opened at
  the row's `right`, so in Hebrew it went off the screen, with only a strip
  showing, and no test hovered a plan on the Hebrew page. Floating UI's
  sides are physical as well, so the preview opens `left-start`.
- Two languages, then one: the core's `Text` pairs and `lang.pick` make a
  missing Hebrew a compile error; the web's `he.ts` is typed as `en.ts` for
  the same reason. There was an English page, and a switch that reloaded
  the page (state in the hash) rather than re-rendering, since `AppState`
  reads the core's texts once. The English page was removed on 2026-09-30;
  the English texts stayed, for English names beside the Hebrew ones and
  for links. In Hebrew, a label ending in a plan's label ("על לאומי ·
  אונליין") is a prefix of its siblings' ("…אונליין, 'לאומי 18+'"), where
  the English one ended in "'s fees": find such buttons with `exact`.
- Loading: `index.html` draws the page's shape before any script, with
  pulsing blocks where the words go, and `main.ts` fades it out once the app
  has mounted. Its styles repeat app.css and App.svelte on purpose: it must
  show before they arrive. `vite.config.ts` adds a `<link rel="preload">` for
  the `.wasm` to the built page; without it the browser asked for it only
  after the JavaScript had run, one after the other (check `dist/index.html`,
  and that the wasm request starts with the JavaScript's).
- The chart attachment (`echarts.svelte.ts`) makes the ECharts instance in a
  ResizeObserver callback, once the element has a size, so `setup`'s effects
  live in an `$effect.root`. The option is a `$derived` read only while
  shown, and drawn in a timer set from an animation frame: that runs once the
  frame is painted, so the typed digit and the table show first. When it
  draws is `drawAt` (`glide.ts`, unit-tested): not before the glide in
  flight has all but settled, and while a number is typed (an `input` event
  on a text field, which a module-level listener notes), not before the
  typing pauses for 300 ms, up to 900 ms. Until 2026-10-01 changes under
  250 ms apart snapped instead, which the user found abrupt: typing and
  dragging the years jumped in single frames.
- While a field is being retyped, the results stay, faded and `inert`, under
  the message: `comparison` keeps the last one that worked (`inputError` is
  the message). Taking them off the page made it collapse, and the charts
  drew themselves again from nothing on the next digit. Only a page opened
  from a link with bad inputs has no results to show.
- The `.choices` highlight is placed by `Choices.svelte` in CSS variables;
  `--glide` is 0 until it has been placed once, and while the buttons resize.
- A box whose height is set from its own observed width (the breakdown's
  bars: rows × a row height that depended on `bind:clientWidth`) makes the
  browser report "ResizeObserver loop completed with undelivered
  notifications" as a page error, on the iPad Mini where the width crossed
  the threshold. The row height now follows a media query on the screen
  (`NARROW_SCREEN` in `fee-breakdown.ts`), which the width can't change.
  The test fixture fails a test on any page error, which is how it showed.
- A tip (`.popover[popover]`) must not take its size from where it happens
  to sit before `tip.ts` places it. With `inset: auto` it stayed in the
  flow's position, beside a ? near the screen's right edge, and was as
  narrow as the room there (100 px wide, 692 tall on the iPhone 15): each
  placement widened it, which moved it again, and Safari reported a
  ResizeObserver loop as a page error. It now starts at the corner with
  `width: max-content`, capped by its `max-width`. Only WebKit reported it,
  and only for tips in the table's rightmost cells.
- The charts' options are built by pure functions (`growth-chart.ts`,
  `fee-breakdown.ts`) from plain inputs (lines, bars, what's pinned), not
  from `AppState`: the components map the state to those inputs and wire
  the events. That's what lets the options be unit-tested in Node, and
  keeps a change to pinning from touching the chart's drawing code.

- A box that animates its own height must start the animation before the
  next paint. A ResizeObserver is told after layout, so an animation
  started there showed the new height for one frame and then jumped back
  to glide (the layout-shift API showed 794→770→794). A MutationObserver
  runs right after the DOM changes, before paint: `glideHeight` measures
  there, and only keeps track in a ResizeObserver.
- A per-frame probe (`requestAnimationFrame` + `getBoundingClientRect`)
  runs before ResizeObservers and paint, so it can see a layout that is
  never shown. What was painted is in the layout-shift entries
  (`PerformanceObserver`, `type: 'layout-shift'`, each source's
  `previousRect`/`currentRect`).
- `grid-template-rows: subgrid` computes to `none` on a container
  (`container-type`): containment lays it out on its own. The summary's
  cards (`.stat`) set `container-type: normal` to share the row's lines.
- A table wider than its box gives each column the least its content
  needs and ignores `width`; `min-width` on the cells still holds. The
  tables' container steps rely on it to end the screen on a whole column.
- A line may break before an inline block (a "?" is one) even with no
  space, and a word joiner (`&NoBreak;`) doesn't stop it in Chrome; neither
  does `word-break: keep-all`. A flex row (`.titled`) keeps the "?" beside
  the words. Chrome breaks before a "·" only when the words before it just
  fill the line: a check has to try every width, not every 16 px.

## Charts

- A stacked bar's colors are checked with the dataviz skill's validator
  over every pair (`--pairs all`), not only stack neighbors, when a part
  can be empty: in the short term, a bank that keeps nothing puts the
  saver's part right against the tax. Blue and violet failed there (ΔE 1.8
  for protanopia), so the saver's part is green.
- What a keystroke cost on a phone (4× CPU throttling) was ECharts'
  `setOption` and first frame, about 40 ms of a 55 ms task; the wasm call
  was about 3 ms and Svelte about 1 ms. Measured with the Event Timing API
  (see Recipes): click-to-paint went from 90–190 ms to 30–50 ms once the
  chart drew after the paint.
- The fee breakdown's names used to be wrapped by ECharts (`width`,
  `overflow: 'break'`) in a column beside the bars. In Hebrew they broke at
  half the column's width, and their dots stood far from them: zrender 6
  wraps and cuts by `measureCharWidth`, which gives every character outside
  ASCII the width of 国 (about the font size, twice a Hebrew letter; "·"
  and curly quotes count as much). ECharts 6.1.0 was the newest, and no
  report upstream was found. A text ECharts draws whole is measured right,
  by `measureText`: each name now has a line of its own above its bar, cut
  if need be by `fitName`, which measures on a canvas.
- The name above its bar: ECharts centers a bar in its row, so an empty bar
  series before the fees takes the top of each row, and padding below the
  names moves them up into it. Padding moves a middle-aligned label only
  when it's rich text; plain text stays in the middle (the totals are rich
  text for that reason alone).
- Switching from the fee breakdown back to Lost to fees took 48 ms on the
  throttled phone, back to Value 24: the growth chart read
  `chartView === 'lost'`, false while the breakdown was shown, so its option
  changed and the whole chart was drawn again on coming back (Value was
  false throughout). It now keeps the view it was last shown with
  (`lastLost`). Likewise the breakdown's width, which `bind:clientWidth`
  set to 0 as its view was hidden: that change could still draw it, 0 px
  wide with every name cut to "…", on a canvas the next visit showed for a
  frame. The breakdown keeps its last width, and a chart being hidden
  cancels a draw still waiting. `chart.spec.ts` records what's drawn
  (`recordDrawnText`) to check that coming back draws nothing.
- In the fee breakdown, only a plan's name and its bar picked it: the
  outline around each row (the custom series) was `silent`, so a click
  beside a bar or on its total did nothing, though the outline shows the
  row as one thing. Filled with `transparent` and not silent, it takes the
  pointer for the whole row; the bars and names still get theirs.
- The motion pass of 2026-10-01, measured frame by frame (CDP
  `Page.startScreencast`, then the share of the chart's pixels that change
  between frames: a glide spreads over many frames, a jump lands in one):
  - The fee breakdown drew itself again from nothing at every change: its
    series had no ids, and with `replaceMerge` ECharts removes every
    series an id doesn't map ("all existing removed unless mapped by id",
    `util/model.js`). Ids keep the old order, which is why they had been
    left out (a focused fee must move to the start of the stack): they are
    now the place in the stack, `fee 0` to `fee 4`.
  - A line's update starts from the old layout's points
    (`lineAnimationDiff`), not from where the line is drawn mid-glide, so a
    glide cut short jumps. `drawAt` waits for it; at 85% of its time the
    page's curve has covered 99% of the way, so the next starts then.
  - ECharts doesn't animate a line at all when any point moves more than
    3,000 px (`getBoundingDiff` in `LineView`): years added far past the
    axis start beyond it, so a jump from 6 years to 35 lands at once.
  - Axis labels move by tick value (`groupTransition`): a value on both
    axes slides, the rest appear and vanish at once. ECharts can't fade them.
  - A category axis's labels are its rows, while bars, matched by their
    category's name, slide to their new rows: names jumped ahead of their
    bars. They ride on the bars now: the names on the empty slot above each
    bar, the totals past a clear bar that fills each row to the end.
  - A line's end label is made with `disableLabelAnimation`, so it jumped
    to where its line was going. Cleared (`releaseEndLabels`), its update
    animation keeps it on the line's end, but only once ECharts has its old
    place, and a label without one is faded in from nothing: the first
    change after loading made every end label blink. `valueAnimation`
    stops the fade (and counts the value up as the line draws itself in),
    and `updateLabelLayout()` right after the first draw gives the labels
    their places. Not after later draws: it cuts short the labels' glides.
  - A bar chart's box changes height with its rows: ECharts resizes at
    once, gliding (`resize({ animation })`), and a frame around the box
    eases the page below on the same curve. Deferring the resize to the
    draw cut off the old, taller drawing while a typed number was waited
    for, which `layoutProblems` caught.
  - zrender reads CSS `cubic-bezier()` strings as easing, so the charts use
    the page's curve; `createCubicEasingFunc` gives Svelte the same function
    (import it with `.js`: zrender's exports map passes paths through).
  - Svelte runs an outro's easing in time order (`t = 1 - easing(progress)`),
    so `--ease-in` reads right for what leaves. `slide`'s CSS ends without a
    semicolon, so a property added after it needs one.
- Pinned names were bold and wrapped differently about one render in twenty;
  with a single weight, 120 runs of 120 matched. `--repeat-each=40` shows
  whether a flake is gone.

- The first tap on the growth chart jumped the page on phones: ECharts
  shows a tooltip where the last one was before moving it beside the
  pointer, and the first one starts in the middle of the chart, so for a
  moment it reached past a 360 px screen. The browser zoomed the page out to
  fit it (the layout viewport went to 511×1108 CSS px, scrolled up 328 px),
  and back once the tooltip moved; when the tooltip hid first, the page
  stayed zoomed out. Where it then landed was past the chart's left edge
  (the chart is narrower than twice the tooltip): cut off by the screen in
  English, and in Hebrew the page could scroll sideways to it, since a
  right-to-left page scrolls to what's on its left. `confine: true` (in the
  shared options) keeps where it lands inside the chart; `overflow: clip`
  on the chart card keeps the moment before from widening the page. Only
  Chromium's mobile mode (`isMobile`) zooms; WebKit's iPhone and a desktop
  window don't.
- Pinning the first plan swapped the hint above the chart for "Unpin all",
  taller and narrower, and the zoom hint came and went with the view: the
  chart moved by 2–28 px depending on the width. Alternatives that take
  turns now share one grid cell (`grid-area: 1 / 1`), the unused ones
  `visibility: hidden`, so the cell is as big as the biggest; the zoom hint
  sits under the chart it zooms.

- The chart by deposit (each plan's yearly cost against the deposit, both
  axes logarithmic) wasn't clear even to the user, who built it. Its
  point is now a line under the best plan, for everyone ("The cheapest at
  any monthly deposit from ₪100 to ₪32,000", or "Above ₪5,200 a month,
  Excellence · Typical offer is cheaper"), and the chart shows only with
  More options. `Sweep::around` finds the crossings from the sweep's
  numbers, the costs taken to change steadily between the amounts tried
  on a logarithmic scale (as the chart draws them): near a crossing the
  two plans cost about the same, so its exact place matters little.
- The sweep costs a comparison's work at each of 16 amounts (23 for a
  one-time deposit): 26 ms on a desktop for the six usual plans abroad,
  97 ms for all 13 plans listed then, and four times that on the phone
  profile. Shown for
  everyone, it can't run on the page's thread at every change, so a module
  worker (`new Worker(new URL(...), { type: 'module' })`, which Vite
  bundles with the wasm-bindgen glue as is) runs its own copy of the core.
  Only the latest request waiting is sent, and only the answer to the
  latest request is used; the state matches it by the request object's
  identity (`$state.raw`). Until it comes, the line keeps its words while
  they're about the plan that's still best, and hides, keeping its place,
  when another plan is: it never describes the wrong plan, and the table
  under it never jumps when the words arrive. The worker adds 4–7 MB of
  private memory (the renderer with it, and with its script blocked).

- A `markLine` is drawn in its own layer (`z: 5`), over the series and
  their labels: the short term's rate line ran through the tax amounts.
  `z: 1` puts it under the bars, like the grid's lines. In one layer,
  ECharts lifts labels two steps over their bar (`z2`).

## Playwright

- `getByText` for a chart view's name can also match the text of its
  hidden tip: click a choice through `choice(page, group, name)`.
- Screenshots were the suite's biggest coupling: 208 baselines on 9
  devices, re-recorded (and looked at) whenever a number, a label or a
  padding changed, since full-page pictures contain everything. They went
  down to 10, with the data masked (18 now: see `web/tests/CLAUDE.md`); layout is checked by rules, charts by
  comparing the canvas before and after an action, and the words by the
  core's own description of them (`tests/core.ts`). The run went from 68 s
  and 264 tests to 49 s and 207, the desktop alone 10 s. Asked whether the
  rules were enough, the user chose rules plus a few masked pictures over
  more pictures: two rules were added (text over text, text cut off inside
  its box) with a test that plants each bug to prove the rules fire, and
  eight pictures of the dialog, the editor and the breakdown on the two
  extreme screens. Inside a canvas only a picture sees anything; the bars'
  word-fit is checked by measuring the names in the page's font instead.
- With the core loaded in Node, a browser test can read the page's inputs
  back and ask the core what the page should show (`inputsOnPage`,
  `expectedRows`, `feesFor`): the test then checks the wiring, and the
  numbers stay the Rust tests' business. Ask the core, not the test file,
  for a count of brokers or plans, a price text, the checked date.
- On the iPhone project a tap is a mouse click, so a hover stays behind
  (`app.hovered`): move the mouse away before asserting anything a hover
  changes.
- Look at the real failures before the screenshots: filter the line
  reporter's `Error:` lines, leaving out `toHaveScreenshot` and
  `toMatchAriaSnapshot`.
- An aria snapshot matches partially: adding a name (an `aria-label`) doesn't
  fail it.

- A click that "intercepts pointer events" on another element and then
  succeeds on a retry can hide a real jump: Playwright scrolls again before
  each retry. Log the page's scroll events around the click (a capture
  listener) before blaming the test; that's how the tooltip's zoom showed.

- Headless Chromium draws no scroll bars (Playwright starts it hiding
  them), so `offsetWidth - clientWidth` is 0 there even where a user sees
  one. To catch a scroll bar, watch every frame (`requestAnimationFrame`,
  motion on) for a box whose `overflow-y` is auto or scroll and whose
  `scrollHeight` passes its `clientHeight`.

- `getByRole('status')` also matches an `<output>` (a slider's readout):
  its implicit role is status. Find a live region by its text instead.

## Recipes

Many baselines at once, on one labelled sheet (the script is in the
history of this file; `web/tests/CLAUDE.md` points here): resize each to
420 px wide, five to a row, and read that image instead of each file.

```python
# uv run --with pillow python sheet.py sheet.png a.png b.png …
import sys
from PIL import Image, ImageDraw

out, *paths = sys.argv[1:]
width, tiles = 420, []
for path in paths:
    image = Image.open(path).convert('RGB')
    image = image.resize((width, int(image.height * width / image.width)))
    tile = Image.new('RGB', (width, image.height + 20), (60, 60, 60))
    ImageDraw.Draw(tile).text((4, 4), path.split('/')[-1], fill=(255, 255, 0))
    tile.paste(image, (0, 20))
    tiles.append(tile)
rows = [tiles[i:i + 5] for i in range(0, len(tiles), 5)]
sheet = Image.new('RGB', (width * 5, sum(max(t.height for t in row) for row in rows)))
y = 0
for row in rows:
    for column, tile in enumerate(row):
        sheet.paste(tile, (column * width, y))
    y += max(t.height for t in row)
sheet.thumbnail((2100, 3000))
sheet.save(out)
```

Screenshots of the app outside the tests. Start
`npx vite --port 5173 --strictPort` in `web/` first, and stop it afterwards:
`npm run dev` needs the port.

```js
// node shots.mjs
import { chromium, devices } from '/home/ori/dev/broker-search/web/node_modules/@playwright/test/index.mjs'

const browser = await chromium.launch()
const context = await browser.newContext({ ...devices['Galaxy S24'], reducedMotion: 'reduce' })
const page = await context.newPage()
// Fixed rates, as tests/fixtures.ts has.
await page.route('https://api.frankfurter.dev/**', (route) =>
  route.fulfill({ json: { date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } } }),
)
await page.goto('http://localhost:5173/')
await page.locator('.chart canvas').first().waitFor()
await page.screenshot({ path: 'page.png', fullPage: true })
await browser.close()
```

The text of a page drawn by JavaScript (IBI's fund page):

```js
// node text.mjs https://…  > page.txt
import { chromium } from '/home/ori/dev/broker-search/web/node_modules/@playwright/test/index.mjs'

const browser = await chromium.launch()
const context = await browser.newContext({ userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36', locale: 'he-IL' })
const page = await context.newPage()
await page.goto(process.argv[2], { waitUntil: 'networkidle', timeout: 60000 })
process.stdout.write(await page.evaluate(() => document.body.innerText))
await browser.close()
```

How long the page takes to respond, by interaction: `npm run test:perf`
in `web/` (see "Speed" below), which prints a table and keeps a budget.

A jump that lasts a frame or two, which pictures and videos miss: record
what could move, every frame, in the page, around the action, and print
only the changes (`chart.spec.ts`, "a first tap", does it as a test):

```js
const chart = document.querySelector('.chart')
const log = []
const t0 = performance.now()
let last = ''
const step = () => {
  const state = `${innerWidth}×${innerHeight} ${scrollX},${scrollY} ${chart.getBoundingClientRect().top}`
  if (state !== last) log.push(`${Math.round(performance.now() - t0)}ms ${state}`)
  last = state
  if (performance.now() - t0 < 1500) requestAnimationFrame(step)
}
step()
```

On a phone, `innerWidth` growing past the screen's width means the page
was zoomed out.

## Speed

`npm run test:perf` (`web/tests/perf/speed.spec.ts`), 2026-10-01, on this
machine, a production build, animations on. The phone project throttles
the CPU 4× (about a Galaxy S24). Time to the next paint per interaction,
by the Event Timing API; anything under about 50 ms isn't felt.

| Measure                                         | phone (4×) | desktop |
| ----------------------------------------------- | ---------: | ------: |
| load, to the first chart (ms)                   |    824–919 | 239–272 |
| typing a deposit (ms)                           |      56–64 |      32 |
| typing the one-time deposit (ms)                |      48–56 |      24 |
| switching the security (ms)                     |      88–96 |      24 |
| switching the exchange (ms)                     |      88–96 |      24 |
| switching calculators (ms)                      |      64–72 |      16 |
| opening the list of plans (ms)                  |      24–40 |      16 |
| ticking a broker's four plans (ms)              |      80–88 |      24 |
| switching the chart view (ms)                   |         24 |      24 |
| switching to the chart by deposit (ms)          |         24 |   16–24 |
| frame gap while the chart by deposit draws (ms) |      50–67 |      17 |
| opening a plan's details (ms)                   |         40 |   40–48 |
| opening a tip (ms)                              |         24 |      16 |
| longest frame gap while the rows reorder (ms)   |      33–50 |      17 |
| zooming: slider drag, or six wheel notches (ms) |         48 |   16–32 |
| longest frame gap while zooming (ms)            |          – |      17 |
| hovering across the rows (ms)                   |          – |      16 |
| heap growth over 30 rounds of changes (MB)      |          1 |       1 |

The ranges include two runs on 2026-10-01's evening, after the tables'
set columns and the best card's ⚠, each within a frame of the earlier ones.

Measured 2026-10-01, after the motion pass (glides of 400 ms that aren't cut
short, and typing waited for): typing got faster on the phone (80 → 56–64),
while a click right after another change got one or two frames slower
(security 64 → 88–96, ticking 64 → 80–88). The measures click there and
back, so the second click lands during the first change's glide, and waits
for the chart's frame being drawn. Before, that glide lasted 180 ms or
snapped.

On 2026-09-30 the phone's numbers were up from 2026-09-29 (typing 48 ms,
load 751): the funds, two banks and the tax at the end came in between, and
the default comparison has 9 plans. The list of plans moving into a sheet the same day
changed nothing: measured against the commit before it on the same machine,
each number was the same or a frame better. A frame gap moves by one frame
(17 ms) from run to run: compare runs on a quiet machine, not with the dev
server and a test run going.

Switching calculators (2026-10-01) makes the whole column and results
again, since only the chosen one is on the page: 64 ms on the phone, and
the other numbers stayed where they were.

The sweep moved to a worker the same day: the chart by deposit's frame gap
went from 150 ms to 33 on the phone, and typing the one-time deposit (a
new sweep per keystroke) costs what typing the monthly one does.

The budgets in the spec are about twice these. Two traps in measuring: a
toggle measured three times ends on the other view, so measure there and
back as one action; and a key pressed to reset the zoom goes into a field
that still has focus (the R made the deposit "10000r", which hid the
chart), so click away first.

## Working with the user

- Batch the visible choices into one question (up to four, each with an ASCII
  preview and a recommendation), once the plumbing works and a screenshot
  shows the real problem.
- Check phones at the narrowest size too (`devices['iPhone SE (3rd gen)']`),
  besides the Samsung the user tests on.
- For anything that moves, a live page beats ASCII: the motion options were
  chosen from a published mockup page with each proposal beside the current
  behaviour, opened on the phone too.

- Bug hunts: keep going until the checks find nothing, and first prove
  each check catches the bug on a page that has it (a checker that found
  nothing at 16 px steps had never seen the narrow width where the bug
  shows). The user: "there will be no more problems when you can't find
  any, not when you get tired of searching for them".

## Open ends

- The link preview (`web/public/og.png`) still shows "Broker fees,
  compounded" in English, from before the page was Hebrew only and before
  the title became "כמה יישאר לכם בסוף".
- The short term's deposit rates are the Bank of Israel's for August 2026,
  before September's cut to 3.25%: the user chose them as published and
  dated, with a caveat, over moving each bank by the cut. Update them
  monthly with `policies/deposit-rates.py`.
- Scrolling to the end of a dialog scrolls the page behind it. The fix,
  `overscroll-behavior: contain`, is out because the Baseline plugin rejects
  the property (Safari lacks it only on the page root); a lint exception is
  the user's call.
- The summary and the tables still change height when their words do (a
  long line in the best plan's card, a row gaining a note); they now glide
  there (`glideHeight`), and the line about other deposits waits for typing
  to pause. While a sum is typed in the short term, the table grows and
  shrinks with it (at ₪2 and ₪25 every deposit gets a note about its
  minimum). Holding the tables while typing, as the charts do, is the
  user's call.
- A plan with tracks that isn't ticked has no picked track, so its fee list
  shows all its tracks instead of the cheapest.
- Leumi's markup is a dated reading of its published buy/sell rates
  (`bankleumi.co.il/vgnprod/shearim.asp`, a legacy page whose Hebrew comes
  out garbled but whose numbers are fine: representative, then transfers
  buy/sell, then banknotes buy/sell). Re-measure it when the numbers are
  re-checked. Interactive's market spread is still not counted.
