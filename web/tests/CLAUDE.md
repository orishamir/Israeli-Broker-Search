# Tests of the web app

Loaded when working on the tests. How to run them is in the root
CLAUDE.md ("Done means", "Saving tokens").

Three layers, cheapest first; put a test in the cheapest layer that can
catch the bug:

1. **Rust** (`cargo test`): every number and every word the core produces.
2. **Unit tests in Node** (`npm run test:unit`, vitest, `src/**/*.test.ts`):
   the TypeScript side, with the real core loaded from the built
   WebAssembly by `src/test-setup.ts`. What the app's state does
   (`app.test.svelte.ts` and `workbench.test.svelte.ts`: a test of a module
   with runes is itself a `.svelte.ts` module; the answer and its horizons
   are the core's, the switch changes only the view, the workbench follows
   the answer), the answer card on the real core (`answer-card.test.svelte.ts`),
   where the outcome chart's marks go (`outcome.test.svelte.ts`), the gain
   bars (`gain.test.ts`), how-to and where to buy (`how.test.svelte.ts`),
   what the workbench's charts' options say (`growth-chart.ts`,
   `fee-breakdown.ts` and `crossover-chart.ts` are pure functions the
   components hand to ECharts), the links (`link.test.ts`: every field
   round-trips), the glossary lookup, the worker's request matching
   (`sweeper.test.ts`: the answer and its horizons are two jobs), number
   formatting. Milliseconds each.
3. **Playwright** (`npm run test:browser`, `tests/*.spec.ts`): only what
   needs a browser: clicks, taps, hovers, keys and scrolling, and the
   layout. Not what the page says: those words are the core's or `t`'s, and
   checking them again here broke on every rewording and caught nothing.

## Playwright

- A spec per part of the app, so a change runs its own spec first:

  | Changed                                                                                 | Spec         |
  | --------------------------------------------------------------------------------------- | ------------ |
  | Questions, NumberField, horizon.ts, the situations                                      | `questions`  |
  | AnswerCard, AnswerLine, Decision, the plan, the cushion's expenses field                | `answer`     |
  | OutcomeChart, outcome.ts, HowTo, WhereToBuy, how.ts, the return field                   | `outcome`    |
  | Workbench, ExpertSwitch, Within, PlanSheet, BrokerPicker, rates.ts, WarningSign         | `workbench`  |
  | LearnCard, Glossary, Tip's "עוד על זה", the about page                                  | `learn`      |
  | the situations, end to end (`REDESIGN.md` §2.5)                                         | `acceptance` |
  | GrowthChart, CrossoverChart, Charts, echarts.svelte.ts                                  | `chart`      |
  | FeeBreakdown                                                                            | `breakdown`  |
  | DetailsDialog, FeesForInputs, Caveat\*, Sources                                         | `details`    |
  | Tip, tip.ts, HebrewNames                                                                | `tips`       |
  | PlanEditor, SimpleFees, PriceList, YourPlans, CoveragePicker, saved                     | `editor`     |
  | app.css, anything about size or position (every screen has a `@phone` state)            | `layout`     |
  | address.svelte.ts (refresh, Back), the example label                                    | `address`    |
  | main.ts, index.html, Share, link.ts (opens a `v=2` link and checks the page)            | `app`        |
  | text.ts, text/\*.ts, index.html's language, anything right-to-left                      | `hebrew`     |
  | the phone's flow: collapsed facts, AnswerBar, DecisionPill, scroll.ts, Decision's table | `phone`      |

  GainSplit and HorizonSection have no spec of their own: their states are
  in `layout`, their numbers unit-tested, a horizon row's click measured in
  the performance suite. The previous plan's guide (`Answer`, `Steps`,
  `Alternatives`, `Donut`, `Ladder`, `PortfolioDialog`, `AssumptionsSheet`,
  `BankChips`, the best bar on phones) went with REDESIGN.md §2.13, and
  `alternatives.spec.ts` with it.

  `npx playwright test --project=desktop tests/editor.spec.ts`. The `=`
  matters: `--project desktop tests/x.spec.ts` reads the path as a second
  project name.

- **Expectations come from the core, not from the test.** `tests/core.ts`
  loads the WebAssembly in Node. For the guide, a spec says what it typed
  (`enter` in `fixtures.ts` types it, answers in the card the decisions it
  names, and returns the `AnswersInput`) and asks the core what that means:
  `answerFor` (the answer for the answers and the choices, under
  `guideSettings`, the page's own at its start; a settings patch for the
  return field), `decisionsOf` (its decisions, line by line), `situations`,
  `learnCard`, `glossary`; `recommendation` is the workbench's own guide.
  `expectAnswerFor` is the common check: the core's chip, title and as many
  decision groups as it asks. `decide` chooses an option by the button the
  core words for it. For the
  workbench, `inputsOnPage` reads its fields and ticks back as `Inputs`
  (with plans on one exchange chosen), `expectedRows` is the table the core
  gives for them, and `feesFor`, `compare`, `compareWays` and `about` give
  the words a dialog should show. A tariff change then changes no browser
  test; the numbers themselves are checked in Rust (`real_tariffs.rs`,
  `economics.rs`, `guide.rs`).
  The page is in Hebrew, and so is the core there; tests name plans by
  their English label, "Leumi · Pepper" (`listed`, `rowOf`), as links do,
  since plan names repeat across brokers, and `listed(...).label` is the
  Hebrew one the page shows. The page's own words come from `t`
  (`src/lib/text.ts`), a security's or exchange's name from
  `securityName`/`exchangeName`, a broker's from `brokerName`.
- **Ports.** `PORT` sets the dev server's (5173 by default) and the
  preview's (4173): Playwright reuses a server already on the port, which
  may be another worktree's. `PORT=5180 npx playwright test …`.
- **Tags decide the devices** (`playwright.config.ts`): every test runs on
  the desktop; `@phone` also on the Galaxy and the iPhone (WebKit);
  `@touch` only on those two (tapping, the zoom slider, real swipes), with
  `@chromium` when it needs the debugging protocol, which WebKit lacks.
  Split a test by device rather than branching on `isMobile` inside it.
  `layout.spec.ts` also runs on the narrowest iPhone; to cover another
  screen, add it to `layoutOnly`.
- **The workbench is where plans are compared**: `workbench(page)` turns the
  switch on (by its label: the input is hidden behind the track, `setExpert`)
  and chooses "another index or a stock" (or the `thing` you pass), then
  waits for the exchange rates. The kept specs (`chart`, `breakdown`,
  `editor`, `tips`, `layout`) start there. Its table (`section.within`) is
  shown only with the switch on: the guide's where-to-buy is `WhereToBuy`. **Plans are ticked in the list
  of plans**, a dialog over the inputs (`PlanSheet.svelte`), each broker
  folded to one line. `fixtures.ts` opens it (and the workbench) and
  unfolds for you: `tickBrokers`, `tickPlan`, `planInList` (a plan's
  row, with its tick, ✎ and ℹ), `aboutPlan`, `copyPlan`. A plan's details or
  the editor open over the list, which stays open under them: name the
  dialog you mean (`details`, `plansList`), and `closePlans` before using
  the page again, which is inert while a dialog is open. `inputsOnPage`
  reads the ticks even with the list closed.
- Import `test`/`expect` from `./fixtures`: it mocks the exchange rates,
  waits until the app is drawn, and fails the test on any error thrown in
  the page (which is how a ResizeObserver loop on the iPad was found).
- **Layout by rules.** `layoutProblems` (`layout.spec.ts`) checks
  every device and state: no sideways scroll, nothing sticking out of a
  card or dialog, no text drawn over other text, no text cut off inside its
  own box, fields ≥16px, choices on one row, dropdowns not cut off, an open
  tip on the screen, no card flush against the one above it, the answer's
  targets (a decision's options, the chart's rows, the horizon rows) at
  least 44 px tall. A test plants
  each of those bugs and checks the rules notice, so a rule that never
  fires can't pass for a clean page. A new view, dialog or choice gets a
  state there; a new class of layout bug gets a rule, not a screenshot.
  Two tests check what changing the inputs does: over every security and
  exchange, the workbench table's columns stay put; and, with motion on, no
  box but the page scrolls up and down while the rows slide.
  `breakdown.spec.ts` records what the chart draws (`fillText`) and checks
  that every plan's name is drawn whole, on one line: none cut to fit the
  screen.
- **No pictures.** A growth chart is checked by comparing its canvas
  before and after (`canvasPicture`): zooming must change it and resetting
  restore it; what the lines look like is a unit test of the options.
- Charts are canvases: click at positions computed from the chart's box,
  after scrolling it into view. Real swipes need CDP
  `Input.dispatchTouchEvent` (see `breakdown.spec.ts`), because `tap()`
  can't scroll.
- Animations are off (`reducedMotion`). A test about motion itself (an end
  label riding on its line, `chart.spec.ts`) turns them on with
  `test.use({ reducedMotion: 'no-preference' })`, and waits until nothing is
  drawn before acting: the lines draw themselves in after loading. Before
  Esc, call `away(page)`, so no hover tip is under the pointer;
  `closeDialog` does.
- On the iPhone project a tap is a mouse click, so a hover stays behind
  (`app.hovered`): move the mouse away before asserting anything a hover
  changes.
- Hidden content (a closed `<details>`, a popover) still has a size, so use
  `checkVisibility`. Mobile WebKit has no mouse wheel.
- `getByText` for a chart view's name can also match its hidden tip: click a
  choice through `choice(page, group, name)`. Two buttons may share a name
  across cards ("לשנות ›"): scope by card. A reload loses the workbench's
  thing chosen (it follows the recommendation): call `workbench` again.
  `getByRole('status')` also matches an `<output>` (a slider's readout).
- A click that "intercepts pointer events" and then succeeds on a retry can
  hide a real jump: Playwright scrolls again before each retry. Log the
  page's scroll events around the click before blaming the test.
- Headless Chromium draws no scroll bars, so `offsetWidth - clientWidth` is
  0 even where a user sees one: to catch one, watch every frame for a box
  whose `overflow-y` is auto or scroll and whose `scrollHeight` passes its
  `clientHeight`.
- `--repeat-each=40` shows whether a flake is gone.
- The owner found most tests "not useful and waste time" (2026-10-18):
  pictures, aria snapshots and browser tests that re-checked the core's
  words went. Don't bring them back; a new check goes in the cheapest layer
  that catches the bug.
- The chart draws after the paint that follows a change, only while its
  view is shown, and while a number is typed, once the typing pauses for
  300 ms (`glide.ts`, whose timing is unit-tested); `expect` retries cover
  that (`expect.poll` for a canvas picture). They cover the sweep too, which a worker answers a
  moment after the inputs change; a unit test answers it itself
  (`answerSweep`).
- The charts of the workbench (`#chart`: growth, breakdown, by deposit) are
  offered only with the switch on, and the chart by deposit only for plans
  on one exchange. The guide's outcome chart (`section.outcome`) is HTML,
  not a canvas: its rows are buttons (`button.press`).
- **Acceptance follows the data.** `acceptance.spec.ts` checks the worked
  examples of `REDESIGN.md` §2.5 by their structure (which decisions
  appear, on which line, their options, the order of those that depend on
  others, the confidence state), as rules the core must keep and the page
  must show, never by amounts or a winner. Where §2.5 and the data differ
  (the bonus's growth: where is an account decision, not stated), the test
  follows the data and says so in a comment.
- `eslint-plugin-playwright` runs in `npm run lint`: a missing `await`, an
  `expect` in a condition, a `test.skip`. Keep conditionals in helpers, not
  in test bodies.
- In CI (`.github/workflows/deploy.yml`) only the desktop project runs.
  Phones and the performance suite are run here.
- A full run takes about 1.5 minutes (254 tests on 4 devices), the desktop
  alone (about 100) 50 s; the first run after `npm run wasm` can take a few
  minutes more, while Vite starts cold. Run it twice after timing-related
  changes.

## Performance: `npm run test:perf`

`tests/perf/speed.spec.ts`, with its own config (`playwright.perf.config.ts`):
a production build served by `vite preview`, animations on, one test at a
time, and the phone's CPU throttled 4×. It measures each interaction's time
to the next paint (the Event Timing API, as Chrome's INP), the longest gap
between frames while the rows reorder and the chart zooms, and how much the
heap grows over 30 rounds of changes. First the guide (typing the money,
the situation chips, a long horizon that the worker answers, choosing an
option in a decision, focusing a chart row, choosing the bank,
typing the yearly return, a horizon row, the glossary, the switch), then
the workbench with plans on one exchange (the old page's measures); prints
a table; and fails on the budgets at the top of the spec. Run it on a quiet
machine; the build takes about a minute. What it gave on 2026-10-20 (a
production build, this machine, a dev server idle beside it; the budgets in
the spec are about twice these):

| Measure                                            | phone (4×) | desktop |
| -------------------------------------------------- | ---------: | ------: |
| load, to the first answer (ms)                     |        670 |     234 |
| guide: typing the money now (ms)                   |         88 |      32 |
| guide: typing the monthly amount (ms)              |         64 |      24 |
| guide: switching the situation (ms)                |         72 |      16 |
| guide: typing a long horizon (worker) (ms)         |         64 |      24 |
| guide: choosing an option in a decision (ms)       |         80 |      24 |
| guide: focusing a row of the chart (ms)            |         32 |      24 |
| guide: choosing the bank (ms)                      |         72 |      16 |
| guide: typing the yearly return (ms)               |         24 |      16 |
| guide: a horizon row, and back (ms)                |         88 |      24 |
| guide: opening the glossary (ms)                   |         24 |      16 |
| guide: the switch on and off (ms)                  |         72 |      24 |
| guide: frame gap while the money changes (ms)      |         50 |      17 |
| workbench: typing a deposit (ms)                   |        112 |      32 |
| workbench: typing the one-time deposit (ms)        |        136 |      40 |
| workbench: switching what to buy (ms)              |        104 |      24 |
| workbench: switching the security, exchange (ms)   |     88–112 |      24 |
| workbench: ticking a broker's plans (ms)           |        152 |      32 |
| workbench: switching the chart view, by deposit    |      24–40 |      24 |
| workbench: opening a plan's details (ms)           |         48 |      72 |
| workbench: frame gap, rows reorder, by deposit     |      50–83 |      17 |
| zoom: slider drag, or six wheel notches (ms)       |         40 |      32 |
| heap growth over 30 rounds, workbench / guide (MB) |      1 / 0 |   1 / 0 |

The workbench's keystrokes cost a phone more than on 2026-10-19 (80 → 112,
72 → 136 ms): each now works out the guide's answer on the page as well as
the comparison, and the answer is dearer than the old recommendation (2 to
24 ms in Node up to 5 years; a date not known goes to the worker for that
reason). Still within budget, but near it for ticking a broker's plans.

Anything under about 50 ms isn't felt. A frame gap moves by one frame
(17 ms) from run to run, so compare runs on a quiet machine, not with the
dev server and a test run going. Two traps in measuring: a toggle measured
three times ends on the other view, so measure there and back as one
action; and a key pressed to reset the zoom goes into a field that still
has focus, so click away first.

## Recipes

Many screenshots at once, on one labelled sheet: resize each to 420 px
wide, five to a row, and look at that one image instead of each file.

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

Screenshots of the app outside the tests: start
`npx vite --port 5173 --strictPort` in `web/` first, and stop it
afterwards (`npm run dev` needs the port).

```js
// node shots.mjs
import { chromium, devices } from '/home/ori/dev/broker-search/web/node_modules/@playwright/test/index.mjs'

const browser = await chromium.launch()
const context = await browser.newContext({
  ...devices['Galaxy S24'],
  reducedMotion: 'reduce',
})
const page = await context.newPage()
// Fixed rates, as tests/fixtures.ts has.
await page.route('https://api.frankfurter.dev/**', (route) =>
  route.fulfill({
    json: { date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } },
  }),
)
await page.goto('http://localhost:5173/')
await page.locator('section.answer #answer-title').waitFor()
await page.screenshot({ path: 'page.png', fullPage: true })
await browser.close()
```

The text of a page drawn by JavaScript (a broker's page; the root
CLAUDE.md, "Researching a tariff"):

```js
// node text.mjs https://…  > page.txt
import { chromium } from '/home/ori/dev/broker-search/web/node_modules/@playwright/test/index.mjs'

const browser = await chromium.launch()
const context = await browser.newContext({
  userAgent:
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36',
  locale: 'he-IL',
})
const page = await context.newPage()
await page.goto(process.argv[2], { waitUntil: 'networkidle', timeout: 60000 })
process.stdout.write(await page.evaluate(() => document.body.innerText))
await browser.close()
```

A jump that lasts a frame or two, which pictures and videos miss: record
what could move, every frame, around the action, and print only the
changes (`chart.spec.ts`, "a first tap", does it as a test). On a phone,
`innerWidth` growing past the screen's width means the page zoomed out.

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
