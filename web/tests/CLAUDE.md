# Tests of the web app

Loaded when working on the tests. How to run them is in the root
CLAUDE.md ("Done means", "Saving tokens").

Three layers, cheapest first; put a test in the cheapest layer that can
catch the bug:

1. **Rust** (`cargo test`): every number and every word the core produces.
2. **Unit tests in Node** (`npm run test:unit`, vitest, `src/**/*.test.ts`):
   the TypeScript side, with the real core loaded from the built
   WebAssembly by `src/test-setup.ts`. What the app's state does
   (`app.test.svelte.ts`: a test of a module with runes is itself a
   `.svelte.ts` module), what the charts' options say (`growth-chart.ts`
   and `fee-breakdown.ts` are pure functions the components hand to
   ECharts), number formatting. Milliseconds each.
3. **Playwright** (`npm run test:browser`, `tests/*.spec.ts`): only what
   needs a browser: that the page shows what the core says, and that
   clicks, taps, hovers, keys and scrolling do what they should.

## Playwright

- A spec per part of the app, so a change runs its own spec first:

  | Changed                                                                        | Spec         |
  | ------------------------------------------------------------------------------ | ------------ |
  | InputsPanel, Examples, NumberField, Choices, PlanSheet, BrokerPicker, rates.ts | `inputs`     |
  | ResultsTable, the stats in App.svelte, PlanPreview                             | `results`    |
  | GrowthChart, CrossoverChart, echarts.svelte.ts                                 | `chart`      |
  | FeeBreakdown                                                                   | `breakdown`  |
  | DetailsDialog, FeesForInputs, Caveat\*, Sources, the about page                | `details`    |
  | Tip, tip.ts, HebrewNames                                                       | `tips`       |
  | PlanEditor, SimpleFees, PriceList, YourPlans, CoveragePicker, saved            | `editor`     |
  | app.css, anything about size or position                                       | `layout`     |
  | main.ts, index.html, Share, link.ts                                            | `app`        |
  | FamilySwitch, the short term (short-term.svelte.ts, ShortTermInputs, Place\*)  | `short-term` |
  | text.ts, text/\*.ts, index.html's language, anything right-to-left             | `hebrew`     |

  `npx playwright test --project=desktop tests/editor.spec.ts`. The `=`
  matters: `--project desktop tests/x.spec.ts` reads the path as a second
  project name.

- **Expectations come from the core, not from the test.** `tests/core.ts`
  loads the WebAssembly in Node: `inputsOnPage` reads the page's fields and
  ticks back as `Inputs`, `expectedRows` is the table the core gives for
  them, and `feesFor`, `compare` and `about` give the words a dialog should
  show. A tariff change then changes no browser test; the numbers
  themselves are checked in Rust (`real_tariffs.rs`, `economics.rs`).
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
  `layout.spec.ts` also runs on six more screen sizes; to cover another,
  add it to `layoutOnly`.
- **Plans are ticked in the list of plans**, a dialog over the inputs
  (`PlanSheet.svelte`), each broker folded to one line. `fixtures.ts` opens
  it and unfolds for you: `tickBrokers`, `tickPlan`, `planInList` (a plan's
  row, with its tick, ✎ and ℹ), `aboutPlan`, `copyPlan`. A plan's details or
  the editor open over the list, which stays open under them: name the
  dialog you mean (`details`, `plansList`), and `closePlans` before using
  the page again, which is inert while a dialog is open. `inputsOnPage`
  reads the ticks even with the list closed.
- Import `test`/`expect` from `./fixtures`: it mocks the exchange rates,
  waits until the app is drawn, and fails the test on any error thrown in
  the page (which is how a ResizeObserver loop on the iPad was found).
- **Rules before pictures.** `layoutProblems` (`layout.spec.ts`) checks
  every device and state: no sideways scroll, nothing sticking out of a
  card or dialog, no text drawn over other text, no text cut off inside its
  own box, fields ≥16px, choices on one row, dropdowns not cut off, an open
  tip on the screen, no card flush against the one above it. A test plants
  each of those bugs and checks the rules notice, so a rule that never
  fires can't pass for a clean page. A new view, dialog or choice gets a
  state there; a new class of layout bug gets a rule, not a screenshot. `breakdown.spec.ts` records what the chart
  draws (`fillText`) and checks that every plan's name is drawn whole, on
  one line: none cut to fit the screen.
- **Pictures, 18 in all, for what rules can't see** (a color, an
  alignment): the start page on each of the nine devices, and, on the
  narrowest phone and the desktop only (`@pictures`), a plan's details, the
  editor in both views, and the fee breakdown. What comes from the tariffs
  is masked, so they change only when the page itself does; the breakdown's
  bars can't be, since the amounts are the picture, so a tariff change
  re-records those two. A growth chart is checked by comparing its canvas
  before and after (`canvasPicture`): zooming must change it and resetting
  restore it; what the lines look like is a unit test of the options.
  Baselines pass differences up to 0.2% of pixels, so assert a subtle change
  (an opacity) instead. `npx playwright test --update-snapshots=all
tests/layout.spec.ts` re-records them; look at the changed PNGs before
  committing, on one contact sheet (LESSONS.md, "Recipes").
- Charts are canvases: click at positions computed from the chart's box,
  after scrolling it into view. Real swipes need CDP
  `Input.dispatchTouchEvent` (see `breakdown.spec.ts`), because `tap()`
  can't scroll.
- Animations are off (`reducedMotion`). Before a screenshot or Esc, call
  `away(page)`, so no hover tip is under the pointer; `closeDialog` does.
  Before a full-page picture, `fitScreenToPage`: Chromium phones otherwise
  lose touch emulation.
- On the iPhone project a tap is a mouse click, so a hover stays behind
  (`app.hovered`): move the mouse away before asserting anything a hover
  changes.
- Hidden content (a closed `<details>`, a popover) still has a size, so use
  `checkVisibility`. Mobile WebKit has no mouse wheel.
- The chart draws just after the paint that follows a change, and only while
  its view is shown; `expect` retries cover that (`expect.poll` for a
  canvas picture). They cover the sweep too, which a worker answers a
  moment after the inputs change; a unit test answers it itself
  (`answerSweep`).
- The chart by deposit is offered only with More options: check it first.
- `eslint-plugin-playwright` runs in `npm run lint`: a missing `await`, an
  `expect` in a condition, a `test.skip`. Keep conditionals in helpers, not
  in test bodies.
- In CI (`.github/workflows/deploy.yml`) only the desktop project runs, with
  `--ignore-snapshots`: the runner's fonts differ from the ones the pictures
  were recorded with. Phones and the performance suite are run here.
- A full run takes about 85 s (283 tests on 9 devices), the desktop alone
  50 s. Run it twice after timing-related changes.

## Performance: `npm run test:perf`

`tests/perf/speed.spec.ts`, with its own config (`playwright.perf.config.ts`):
a production build served by `vite preview`, animations on, one test at a
time, and the phone's CPU throttled 4×. It measures each interaction's time
to the next paint (the Event Timing API, as Chrome's INP), the longest gap
between frames while the rows reorder and the chart zooms, and how much the
heap grows over 30 rounds of changes; prints a table; and fails on the
budgets at the top of the spec. Run it on a quiet machine: the numbers it
should give are in LESSONS.md ("Speed"). The build takes about a minute.
