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
  פיקדון/פקדון, דמי ניהול/טיפול פיקדון. The monthly fee: דמי טיפול, דמי שימוש.
  Leumi's tariff never says משמרת at all.
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
  averages confirm them.
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

## Tools on this machine

- The system Python has no Pillow: `uv run --with pillow python script.py`.
- The Cargo packages are `broker-fees` (the core) and `broker-fees-wasm` (the
  bindings): `cargo test -p broker-fees`.
- `npx playwright test --update-snapshots tests/x.spec.ts` fails: the flag
  takes an optional mode and swallows the path. Write
  `--update-snapshots=all tests/x.spec.ts`.
- When the permission classifier blocked Bash and WebSearch, the user allowed
  them in `.claude/settings.local.json`.
- `pkill -f <pattern>` kills the shell running the command itself when the
  pattern appears anywhere in that command (exit code 144, nothing after it
  runs). Stop a server by its port instead: `fuser -k 4173/tcp`.

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

## The web app

- Caveat texts repeat across coverages, so `Caveats.svelte` keys by position.
  `app.spec.ts` opens every plan's dialog and fails on any page error
  (`each_key_duplicate` blanked a dialog): it covers new brokers by itself.
- `{@const}` must sit directly in a block (`{#if}`, `{#each}`…), not inside an
  element.
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
  until warnings and notes were allowed to wrap.
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
  frame is painted, so the typed digit and the table show first. Changes
  under 250 ms apart don't glide.
- The `.choices` highlight is placed by `Choices.svelte` in CSS variables;
  `--glide` is 0 until it has been placed once, and while the buttons resize.

## Charts

- What a keystroke cost on a phone (4× CPU throttling) was ECharts'
  `setOption` and first frame, about 40 ms of a 55 ms task; the wasm call
  was about 3 ms and Svelte about 1 ms. Measured with the Event Timing API
  (see Recipes): click-to-paint went from 90–190 ms to 30–50 ms once the
  chart drew after the paint.
- Category-axis labels with `width` and `overflow: 'break'` break inside a
  word that doesn't fit ("Excellenc/e"): size the column for the longest word
  as it's drawn, padded if pinned.
- Pinned names were bold and wrapped differently about one render in twenty;
  with a single weight, 120 runs of 120 matched. `--repeat-each=40` shows
  whether a flake is gone.

## Playwright

- On the iPhone project a tap is a mouse click, so a hover stays behind
  (`app.hovered`): move the mouse away before asserting anything a hover
  changes.
- Look at the real failures before the screenshots: filter the line
  reporter's `Error:` lines, leaving out `toHaveScreenshot` and
  `toMatchAriaSnapshot`.
- An aria snapshot matches partially: adding a name (an `aria-label`) doesn't
  fail it.

## Recipes

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

How long the page takes to respond, by interaction, on a throttled phone.
The Event Timing API gives each event's time to the next paint (at least
16 ms, rounded to 8); anything over about 50 ms is felt.

```js
// node latency.mjs  (the app served on 4173)
import { chromium, devices } from '/home/ori/dev/broker-search/web/node_modules/@playwright/test/index.mjs'

const browser = await chromium.launch()
const context = await browser.newContext(devices['Galaxy S24'])
const page = await context.newPage()
await page.addInitScript(() => {
  window.events = []
  new PerformanceObserver((list) => {
    for (const e of list.getEntries()) window.events.push([e.name, e.duration])
  }).observe({ type: 'event', durationThreshold: 16 })
})
const cdp = await context.newCDPSession(page)
await cdp.send('Emulation.setCPUThrottlingRate', { rate: 4 })
await page.goto('http://localhost:4173/')
await page.locator('.chart canvas').first().waitFor()
const worst = async (label, act) => {
  await page.evaluate(() => (window.events = []))
  await act()
  await page.waitForTimeout(300)
  const events = await page.evaluate(() => window.events)
  console.log(label, Math.max(0, ...events.map(([, d]) => d)), 'ms')
}
await worst('typing', () => page.getByLabel('Every month').pressSequentially('2500', { delay: 120 }))
await worst('security', () => page.getByRole('radiogroup', { name: 'Security' }).getByText('Bond').click())
await worst('breakdown', () => page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click())
await browser.close()
```

Many baselines at once: paste them into one labelled sheet, and read that
image instead of each file.

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
    ImageDraw.Draw(tile).text((4, 4), path, fill=(255, 255, 0))
    tile.paste(image, (0, 20))
    tiles.append(tile)
rows = [tiles[i:i + 3] for i in range(0, len(tiles), 3)]
sheet = Image.new('RGB', (width * 3, sum(max(t.height for t in row) for row in rows)))
y = 0
for row in rows:
    for column, tile in enumerate(row):
        sheet.paste(tile, (column * width, y))
    y += max(t.height for t in row)
sheet.save(out)
```

## Working with the user

- Batch the visible choices into one question (up to four, each with an ASCII
  preview and a recommendation), once the plumbing works and a screenshot
  shows the real problem.
- Check phones at the narrowest size too (`devices['iPhone SE (3rd gen)']`),
  besides the Samsung the user tests on.
- For anything that moves, a live page beats ASCII: the motion options were
  chosen from a published mockup page with each proposal beside the current
  behaviour, opened on the phone too.

## Open ends

- Scrolling to the end of a dialog scrolls the page behind it. The fix,
  `overscroll-behavior: contain`, is out because the Baseline plugin rejects
  the property (Safari lacks it only on the page root); a lint exception is
  the user's call.
- A plan with tracks that isn't ticked has no picked track, so its fee list
  shows all its tracks instead of the cheapest.
- Leumi's markup is a dated reading of its published buy/sell rates
  (`bankleumi.co.il/vgnprod/shearim.asp`, a legacy page whose Hebrew comes
  out garbled but whose numbers are fine: representative, then transfers
  buy/sell, then banknotes buy/sell). Re-measure it when the numbers are
  re-checked. Interactive's market spread is still not counted.
