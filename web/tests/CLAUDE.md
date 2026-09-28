# Playwright tests

Loaded when working on the tests. How to run them is in the root
CLAUDE.md ("Done means", "Saving tokens").

- Playwright runs every test on desktop, Galaxy S24 and iPhone 15 (WebKit).
  `layout.spec.ts` also runs on six more screen sizes; to cover another, add
  it to `layoutOnly` in `playwright.config.ts`.
- Import `test`/`expect` from `./fixtures`: it mocks the exchange rates, and
  waits until the app is drawn, since a half-drawn page looks settled to
  screenshots.
- `layoutProblems` (`layout.spec.ts`) checks, on every device and state:
  - no sideways scroll;
  - nothing sticks out of a card or dialog, except inside a sideways scroller;
  - fields are ≥16px, or iPhones zoom in;
  - each `.choices` group stays on one row.

  A new view, dialog or choice gets a state there. A new class of layout bug
  gets a rule there, not a one-off assertion. Tests loop over every option
  (all securities and exchanges), not just the defaults.

- Split mouse and touch tests with `isMobile`, and give each `test.skip` a
  reason. Real swipes need CDP `Input.dispatchTouchEvent` on a Chromium phone
  (see `breakdown.spec.ts`), because `tap()` can't scroll.
- Charts are canvases: click at positions computed from the chart's box, after
  scrolling it into view.
- Animations are off (`reducedMotion`).
- Before a screenshot, call `page.mouse.move(0, 0)` so no hover tip opens;
  also before Esc in a dialog, or Esc closes the tip under the pointer
  instead. Before a full-page one, call `fitScreenToPage`: Chromium phones
  otherwise lose touch emulation.
- Screenshot baselines pass differences up to 0.2% of pixels, which misses
  subtle changes such as a line's opacity; assert those instead. Plain
  `--update-snapshots` re-records only failing ones, so after a small visible
  change (a renamed label) use `--update-snapshots=all`. Look at the changed
  PNGs before committing them.
- Hidden content (a closed `<details>`, a popover) still has a size, so use
  `checkVisibility`. Mobile WebKit has no mouse wheel.
- A full run takes about a minute. Run it twice after timing-related changes.
