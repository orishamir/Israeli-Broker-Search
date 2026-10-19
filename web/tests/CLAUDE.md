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
   with runes is itself a `.svelte.ts` module; answers to recommendation, the
   switch changes only the view, overrides), what the charts' options say
   (`growth-chart.ts`, `fee-breakdown.ts`, `alternatives-chart.ts` and
   `donut.ts` are pure functions the components hand to ECharts), the links
   (`link.test.ts`: every field round-trips), the glossary lookup, the
   worker's request matching (`sweeper.test.ts`), number formatting. Milliseconds each.
3. **Playwright** (`npm run test:browser`, `tests/*.spec.ts`): only what
   needs a browser: clicks, taps, hovers, keys and scrolling, and the
   layout. Not what the page says: those words are the core's or `t`'s, and
   checking them again here broke on every rewording and caught nothing.

## Playwright

- A spec per part of the app, so a change runs its own spec first:

  | Changed                                                                         | Spec           |
  | ------------------------------------------------------------------------------- | -------------- |
  | Questions, BankChips, NumberField, horizon.ts, the situations                   | `questions`    |
  | Answer, Steps, PortfolioDialog, the unlock line and the cushion's two parts     | `answer`       |
  | Alternatives, alternatives-chart.ts, Donut, Ladder, linked.svelte.ts            | `alternatives` |
  | Workbench, ExpertSwitch, Within, PlanSheet, BrokerPicker, rates.ts, WarningSign | `workbench`    |
  | LearnCard, Glossary, Tip's "עוד על זה", the about page                          | `learn`        |
  | the user's five questions and the audit's tasks (REDESIGN.md §8), end to end    | `acceptance`   |
  | GrowthChart, CrossoverChart, Charts, echarts.svelte.ts                          | `chart`        |
  | FeeBreakdown                                                                    | `breakdown`    |
  | DetailsDialog, FeesForInputs, Caveat\*, Sources                                 | `details`      |
  | Tip, tip.ts, HebrewNames                                                        | `tips`         |
  | PlanEditor, SimpleFees, PriceList, YourPlans, CoveragePicker, saved             | `editor`       |
  | app.css, anything about size or position (every screen has a `@phone` state)    | `layout`       |
  | main.ts, index.html, Share, link.ts (opens a `v=2` link and checks the page)    | `app`          |
  | text.ts, text/\*.ts, index.html's language, anything right-to-left              | `hebrew`       |

  The specs for the redesign's screens (`questions`, `answer`,
  `alternatives`, `workbench`, `learn`, `acceptance`) replace `inputs`,
  `results` and `short-term`, which went with the two calculators, the
  More options switch and the examples. `FamilySwitch`, `InputsPanel`,
  `ResultsTable`, `ShortTermInputs` and `Place*` no longer exist.

  `npx playwright test --project=desktop tests/editor.spec.ts`. The `=`
  matters: `--project desktop tests/x.spec.ts` reads the path as a second
  project name.

- **Expectations come from the core, not from the test.** `tests/core.ts`
  loads the WebAssembly in Node. For the guide, a spec says what it typed
  (`enter` in `fixtures.ts` types it and returns the `AnswersInput`) and
  asks the core what that means: `recommendation`, `ladderFor` and
  `alternativesFor` (with `guideSettings`, the page's own at its start, and
  `NO_OVERRIDES` or the kind chosen over the guide), `situations`,
  `learnCard`, `glossary`, `followUps`. `expectAnswerFor` is the common
  check: the sentence the core makes and, for one sum, its amount. For the
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
  `editor`, `tips`, `layout`) start there. **Plans are ticked in the list
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
  tip on the screen, no card flush against the one above it. A test plants
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
- The chart draws after the paint that follows a change, only while its
  view is shown, and while a number is typed, once the typing pauses for
  300 ms (`glide.ts`, whose timing is unit-tested); `expect` retries cover
  that (`expect.poll` for a canvas picture). They cover the sweep too, which a worker answers a
  moment after the inputs change; a unit test answers it itself
  (`answerSweep`).
- The charts of the workbench (`#chart`: growth, breakdown, by deposit) are
  offered only with the switch on, and the chart by deposit only for plans
  on one exchange. The guide has its own chart (`.alternatives .chart`) and
  the donut (`section.donut .chart`): a spec says which with its class.
- **Acceptance follows the data.** `acceptance.spec.ts` asserts what the
  core's `recommend` says for the inputs a person types, never a kind the
  plan expected: where REDESIGN.md §8 and the data differ, the test follows
  the data and says so in a comment.
- `eslint-plugin-playwright` runs in `npm run lint`: a missing `await`, an
  `expect` in a condition, a `test.skip`. Keep conditionals in helpers, not
  in test bodies.
- In CI (`.github/workflows/deploy.yml`) only the desktop project runs.
  Phones and the performance suite are run here.
- A full run takes about 1.5 minutes (235 tests on 4 devices), the desktop
  alone (about 85) 25 s; the first run after `npm run wasm` can take a few
  minutes more, while Vite starts cold. Run it twice after timing-related
  changes.

## Performance: `npm run test:perf`

`tests/perf/speed.spec.ts`, with its own config (`playwright.perf.config.ts`):
a production build served by `vite preview`, animations on, one test at a
time, and the phone's CPU throttled 4×. It measures each interaction's time
to the next paint (the Event Timing API, as Chrome's INP), the longest gap
between frames while the rows reorder and the chart zooms, and how much the
heap grows over 30 rounds of changes. First the guide (typing the money,
the situation chips, a long horizon that the worker answers, a ladder rung,
"continue anyway", the cards, the switch), then the workbench with plans on
one exchange (the old page's measures); prints a table; and fails on the
budgets at the top of the spec. Run it on a quiet machine: the numbers it
should give are in LESSONS.md ("Speed"). The build takes about a minute.
