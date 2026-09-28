# Your own plans: plan for implementation

> Designed with the user on 2026-09-28, for a later session to implement.

## Context

Most fee calculators only ask for the common fees (trade, custody); this app
compares real brokers' tariffs. Users still need their own plans:

- **Negotiating**, the main case: "I'm on Pepper, but I think they'll lower
  the trade fee if I call." Israelis call this asking for a discount on fees
  (הנחה בעמלות) by bargaining (מיקוח); both terms appear on bizportal.co.il
  and the hasolidit.com forum.
- **A broker that isn't listed**: the plain-calculator case.

The broker/plan picker gets reworked around this. Every choice below was made
with the user through mockups (2026-09-28). CLAUDE.md's "Ask first" still
applies: ask about any visible detail this plan doesn't settle.

## Design (decided)

### Two ways to make a plan
- **✎ on a listed plan** (in the brokers card), or **"Change these fees"** in
  a listed plan's dialog: opens a *draft copy* in the editor.
- **"+ New plan"** in the Your plans card opens a *draft* with blank fees, a
  name, and an optional broker name.
- A draft has **[Cancel] [Add plan]**. "Add plan" saves it and ticks it, next
  to its original. A plan you already have saves as you type, with
  **[Delete plan]** (confirmed inline: "Delete 'X'? [Delete] [Keep]") and
  **[Done]**.
- Default names: "Pepper, your deal" for a copy; "Your plan", then "Your plan
  2" and so on, for a new one. The user can rename any plan in the title
  field. Names needn't be unique.

### The "Your plans" card (a separate card below "Brokers and plans to compare")
```
┌ BROKERS AND PLANS TO COMPARE ? ─────┐
│ ☑ Bank Leumi                   ⚠ ℹ │
│    ☑ Pepper                    ✎ ℹ │ ← ✎ shows on hover like ℹ;
│    …                                │   always on touch screens
└─────────────────────────────────────┘
┌ YOUR PLANS ? ───────────────────────┐
│ ☑ ◯ Pepper, your deal           ✎  │
│      Copy of Pepper · Bank Leumi    │
│ ☑ ◯ IBI Smart                   ✎  │
│      IBI · your own   (or: Your own)│
│ [+ New plan]                        │
└─────────────────────────────────────┘
Empty: "Think you can get lower fees, or use a broker that isn't
listed? ✎ on a plan changes a copy of it."  [+ New plan]
```
- The **?** tip says banks and investment houses often give a discount on
  fees (<bdi lang="he">הנחה בעמלות</bdi>) if you bargain (<bdi
  lang="he">מיקוח</bdi>). ✎ copies a plan to change what you think you can
  get; + New plan adds one that isn't listed.
- ✎ aria-label: "Change a copy of {plan}'s fees". The hover preview's hint
  becomes "ℹ shows the full details and caveats · ✎ changes a copy of its
  fees". Hovering your plans' rows shows the same preview.

### The editor: the plan's dialog (DetailsDialog), editable for your plans
Simple view, the default:
```
┌ ◯ [Pepper, your deal        ]  ✕ ┐
│ Copy of Pepper · Bank Leumi        │ ← link: opens Pepper's dialog
│     [ Simple │ Full price list ]   │ ← .choices; remembered
│ For an ETF bought in the USA:      │
│ Buy or sell ?                      │
│  [$ 2       ] [per order     ▾]    │ ← % of the trade / per share / per order
│        Pepper: $4 per order ↺     │ ← only when changed; ↺ restores it
│ Custody ?                          │
│  [0.1   %] [a quarter ▾]           │
│  min [₪          ] a quarter       │ ← billing period is plain text here
│ Conversion ?  [0.1 %]  min [$ 3]   │ ← abroad only
│ Conversion markup ?  [        %]   │ ← empty = not published, counted as 0
│ ────────────────────────────────── │
│ Over 20 years: ₪41K in fees, not   │
│ ₪67K: ₪27,300 more than Pepper.    │
│ [Delete plan]               [Done] │ ← draft: [Cancel] [Add plan]
└────────────────────────────────────┘
```
- **Simple = price, unit and minimum for every fee** that applies to the
  chosen security and exchange. Conversion keeps its minimum because it
  matters: Leumi's $5.76 minimum makes a monthly ₪2,000 conversion cost
  about 1%, not 0.16%. Max, billing period, coverage and the least first
  deposit are only in the full view.
- It edits the row the current choice uses (`trade_row`/`custody_row`). With
  Leumi's "Anything on USA, Europe" row, a change on USA changes Europe too.
  That's by design, and visible in the full view.
- If the security isn't offered, the fee reads "not offered" with
  **"+ Add a fee"**, which adds a row for exactly that security on that
  exchange, in the exchange's currency.
- The live line compares the plan with its original (a copy) or with the
  best listed plan (a new plan), using `fees.total` and the `afterSelling`
  difference. If the plan doesn't offer the security: "Not offered for {purchase}".
- A new plan has a name field and **"Broker (optional)"** instead of the
  "Copy of" line.
- Your plans show **no caveats**: the fees are the user's own claim. A
  copy's link to its original leads to them.

Full price list view (the same dialog, after the switch):
```
BUY OR SELL ?                  [+ Row]
┌────────────────────────────────────┐
│ [Anything on Tel Aviv ▾]         ✕ │ ← coverage button (Rust coverage())
│ except Mutual fund on Tel Aviv     │ ← only when a more specific row
│ (its own row above)                │   takes some of it
│ [₪ 4      ] [per order       ▾]    │
│ min [₪   ]  max [₪   ]             │ ← only for % and per share
└────────────────────────────────────┘
CUSTODY ?                      [+ Row]
┌────────────────────────────────────┐
│ [Tel Aviv ▾]                     ✕ │
│ [0.15   %] [a quarter ▾]           │
│ min [₪     ] charged [quarterly ▾] │
└────────────────────────────────────┘
CONVERSION ?
 [0.1 %]  min [$ 3]  max [$ 1,500]
 Markup [        %]
LEAST FIRST DEPOSIT ?  [₪        ]
```
- A row fully taken by others shows "⚠ Never used: other rows cover all of
  it." Each changed row shows "Pepper: …" and ↺ when the original has a row
  with the same coverage.
- **Coverage checklist**: a click-only popover under the button, with its
  own maximum height:
  ```
  Securities: ☑ ETF ☐ Mutual fund ☐ Bond ☑ Stock
  Exchanges:  ☐ Tel Aviv ☑ USA ☐ Europe
  None ticked means all.
  ```
- **Overlaps: the more specific row wins**, automatically: rows keep
  themselves sorted by how many (security, exchange) pairs they cover, fewest
  first, and a stable sort keeps ties in order. There's nothing to arrange.
- **Copies are rewritten once** so their prices stay the same (see
  `most_specific_first`). Only Altshuler's *New customers* changes, because
  its offer row is laid over the regular list and wins by being on top:
  - its regular "ETF on Tel Aviv" and "Mutual fund on Tel Aviv" rows are
    dropped (they never applied);
  - "Stock, Bond on Tel Aviv" becomes "Bond on Tel Aviv".

  Every other plan is unchanged.

### Looks
- Your plans are **dotted lines** in the value and lost charts. The "No fees"
  line stays gray and dashed.
- They get a **hollow dot ◯** in the table, the breakdown names and the
  cards.
- A copy keeps **its original's color**; a new plan takes `PLAN_COLORS[n %
  8]` (n = its position among new plans).
- Subtitles in the table and the Best card: "Your deal · Bank Leumi", "IBI ·
  your own", or "Your own".

### Saving
- **This browser only** (localStorage), wrapped in try/catch. Unreadable
  data is dropped with a console warning. No share link or file for now.

## Implementation

Order: Rust first (with tests), then `npm run wasm`, then the web. Keep the
logic in Rust: TypeScript never looks inside a plan.

### 1. Core: `crates/core`
- **`simulation::compare`** now takes `plans: &[&Plan]` instead of
  `(brokers, &[(usize, usize)])`. `PlanOutcome` gets `index: usize` (the
  position in `plans`) in place of `broker`/`plan`. Update its tests.
- **New `src/yours.rs`** ("the user's own plans: changed copies of listed
  plans, and plans of their own"), re-exported from `lib.rs`. Add it to
  CLAUDE.md's Layout.
  - `TradeFee::pairs()` lists the (Security, Exchange) pairs a row covers
    (empty = all, via `IntoEnumIterator`), and `size()` counts them.
    `CustodyFee` gets the same over exchanges.
  - `Plan::most_specific_first(&self) -> Plan`:
    1. For each pair, find its first-match winner.
    2. A row drops a pair when the winner is an earlier row that isn't
       strictly more specific (`size(winner) >= size(row)`).
    3. Rows left with no pairs are removed.
    4. A remainder that isn't a securities × exchanges product is split:
       group the pairs by exchange, then merge exchanges with the same
       securities.
    5. A row whose coverage didn't change keeps its original vectors, so
       "Anything" stays "Anything".
    6. Stable-sort by `size()`. Custody rows get the same treatment.
  - `Plan::sort_rows(&mut self)`: stable sort by size, run after every edit.
  - `Plan::copy_of(&Plan) -> Plan`: runs `most_specific_first`, names it
    "{name}, your deal", clears `description` and `caveats`, and keeps
    `min_first_deposit`.
  - `Plan::new_own(name) -> Plan`:
    - trade rows: "Anything on Tel Aviv", 0% with ₪ bounds, and "Anything on
      USA, Europe", 0% with $ bounds;
    - custody: one row for everything, 0% `per` Year, `billed` Quarter;
    - conversion: default, with `Markup::UpTo(0)`.
  - Field types (serde + `cfg_attr(ts)` Tsify, camelCase), with Decimals for
    numbers and a currency symbol for units:
    - `PriceKind` (Percent / PerShare / PerOrder; strum Display: "% of the
      trade", "per share", "per order");
    - `TradeFields { kind, amount, min, max, currency }`;
    - `CustodyFields { percent, per, billed, min }`;
    - `ConversionFields { percent, min, max, markup: Option<Decimal> }`,
      where None means `NotPublished`.

    Each getter's result also carries `text` (the row's existing Display:
    "$4 per order"), used for the "Pepper: …" hints.
  - Simple view:
    - `simple_fees(security, exchange) -> SimpleFees { trade: Option<…>,
      custody: Option<…>, conversion: Option<…> }` (conversion is None on
      Tel Aviv);
    - `set_trade(security, exchange, TradeFields)`, which inserts an exact
      row when none covers it;
    - `set_custody(exchange, …)` and `set_conversion(…)`.
  - Full view:
    - `price_list(original: Option<&Plan>) -> PriceList`: each row has its
      coverage vectors, `covers` text, `except: Vec<String>` (the coverage
      text of the rows that take its pairs), `never_used`, its fields, and
      `was: Option<String>`;
    - `add_trade_row` (defaults to the chosen security and exchange),
      `set_trade_row(index, coverage, fields)` and `remove_trade_row`, plus
      the custody equivalents, `set_min_first_deposit` and `rename`.
  - Every setter returns `Result<Plan, InvalidFee>` (thiserror: "can't be
    negative", "the minimum is more than the maximum"). The UI shows the
    message under the field and keeps the last valid plan.
- `describe.rs`: the words for `except`, "never used", and `PriceKind`/`Period`
  select labels. `TradeFee::coverage()` and `CustodyFee::coverage()` already
  give the button text.
- Tests (`yours.rs`):
  - **Equivalence**: for every listed plan × every Security × Exchange,
    `most_specific_first` gives the same trade price and custody row as the
    original.
  - **Only New customers changes** (assert `==` for every other plan), and
    its exact rows are as described above.
  - Splitting, with a synthetic {Stock, Bond} on {Tel Aviv, USA} row losing
    Stock on Tel Aviv.
  - Setters round-trip (`simple_fees(set_x(f)) == f`); a USA edit on Leumi
    Online also changes Europe; "+ Add a fee" for Altshuler ETF in Europe
    inserts an exact row.
  - Validation errors.
  - A serde JSON round-trip, like `real_tariffs.rs:188`.
  - An unchanged copy compares equal to its original.

### 2. WASM: `crates/wasm/src/lib.rs`
- `PlanData`: a `#[serde(transparent)]` newtype over `Plan`, made opaque in
  TypeScript with a branded type.
- `PlanKey` becomes an enum tagged `kind`: `Listed { broker, plan }` |
  `Yours { id }`.
- `Inputs.yourPlans: Vec<{ id, plan: PlanData }>`. `compare_plans` resolves
  each key to a `&Plan` (an unknown id is an error) and maps outcomes back to
  keys. The warning uses the resolved plan.
- New functions, each a thin wrapper that takes `PlanData` and returns a new
  one or its fields:
  - `copyOf(broker, plan)`, `newPlan(name)` and `planInfo(plan)` (reusing
    `PlanInfo`);
  - `feesForPlan(plan, security, exchange)`, which has no broker caveats;
  - `simpleFees`, `setTrade`, `setCustody`, `setConversion` and `priceList`;
  - the row add/set/remove functions, `setMinFirstDeposit` and `rename`.
- Tests: compare with a `Yours` key, an unknown id, and a copy equal to its
  original.

### 3. Web state: `web/src/lib/app.svelte.ts`
- `YourPlan { id: crypto.randomUUID(); plan: PlanData; brokerName: string |
  null; basedOn: { broker: string; plan: string } | null }`.
  - `basedOn` stores **names**, not indices, because indices shift when
    brokers are added (Interactive Israel is next).
  - If the original can't be found, the link and color fall back to a new
    plan's.
- `yourPlans = $state.raw<YourPlan[]>`, loaded from `localStorage['your-plans-v1']`
  and saved by an `$effect`. `editorView = $state<'simple' | 'full'>` is
  saved the same way.
- `Plan` interface:
  - `plans` becomes `$derived`: the listed plans plus your plans, with ids
    `yours:{id}`;
  - add `subtitle: string` and `yours?: YourPlan`, and make `broker?:
    BrokerInfo` optional (listed only);
  - every `plan.broker.name` becomes `plan.subtitle`: the table, the Best
    card and the dialog.
- `feesFor(plan)` goes to `feesFor` or `feesForPlan`. Pull the `compare`
  input object out of `comparison` into a helper, so the editor can reuse it.
- `addYourPlan` (ticks it), `updateYourPlan`, and `deleteYourPlan` (unticks
  and unpins it).
- `Details` gains `{ kind: 'draft'; draft: YourPlan }`.
  `{ kind: 'plan', plan }` with `plan.yours` opens the editor.

### 4. Components (`web/src/lib`)
- **`YourPlans.svelte`** (new card, in `InputsPanel.svelte` after the
  brokers card): rows, empty state, "+ New plan", and a Tip with the text
  above. It follows BrokerPicker's patterns: the checkbox `accent-color`, the
  hover preview, and `app.hovered` for the mouse only.
- **`BrokerPicker.svelte`**: ✎ beside ℹ, the same hover and touch rules as
  ℹ, and the preview hint.
- **`DetailsDialog.svelte`**: a listed plan gets a "Change these fees"
  button under `FeesForInputs`. A draft or a plan of yours renders
  `PlanEditor.svelte`.
- **`PlanEditor.svelte`**:
  - the name field in the title, and the "Copy of …" link or the broker
    field;
  - `Choices` for Simple | Full price list;
  - `SimpleFees.svelte` or `PriceList.svelte`;
  - the live line, a `$derived` compare of the draft against the original
    or all listed plans;
  - the buttons.
- **`SimpleFees.svelte`**:
  - fee names, tips and Hebrew from `feesFor…().fees` (`FeeLine`);
  - `NumberField` with units inside the fields;
  - `<select>`s for the unit and the period;
  - the "Pepper: … ↺" hints, and "not offered" with "+ Add a fee".
- **`PriceList.svelte`**: the row cards, with `CoveragePicker.svelte` for
  coverage.
  - Give `tip.ts` a `{ onHover: false }` option, so the checklist opens only
    on click, tap or keyboard. It keeps Floating UI's `size` maximum height.
  - The checklist is interactive, so it's not a Tip.svelte.
- **`ResultsTable.svelte`**: a ring mark for your plans (`.mark.yours`: a
  border in the plan's color, no fill), and the subtitle.
- **`GrowthChart.svelte`**: `lineStyle.type: 'dotted'` for your plans.
- **`FeeBreakdown.svelte`**:
  - a "◯" dot for your plans;
  - **fix `planAt`**: it finds plans by name (line ~258), which breaks with
    duplicate names. Use plan ids as the yAxis data, with the formatter
    showing `rows[index].plan.info.name`, and find by id.

### 5. Tests (Playwright, every device; import from `./fixtures`)
- New `tests/your-plans.spec.ts`:
  - ✎ opens a draft with the original's fees; changing the trade fee
    updates the live line.
  - Add plan: a ticked row in Your plans, and a table row with
    `.mark.yours` and the subtitle "Your deal · Bank Leumi".
  - Cancel adds nothing. Delete plan with its confirmation unticks and
    unpins it.
  - A reload keeps the plans and the Simple/Full choice.
  - "+ New plan" with a broker name.
  - "not offered" → "+ Add a fee" (Altshuler, ETF, Europe).
  - Full view: add a row, pick its coverage in the checklist, then check the
    except note and the never-used warning; remove the row.
  - The editor over **every security and exchange**.
  - A chart screenshot showing a dotted line.
  - Touch: ✎ is visible without hovering.
- `layout.spec.ts`: new states for Your plans empty and filled, the simple
  editor, the full editor, and the open checklist. The existing rules cover
  them: fields ≥16px, "Simple | Full price list" on one row on the iPhone
  SE, nothing sticking out of the dialog.
- Update the screenshot baselines and **look at every changed PNG**.

## Verification
- `cargo test`, `cargo clippy --all-targets` (0 warnings) and `cargo fmt
  --check`.
- Then in `web/`: `npm run wasm && npm test` (run it twice, since there are
  popovers and timing), and `npm run lint`.
- By hand in the dev server, on desktop and on a Galaxy-sized phone:
  1. Copy Pepper and lower its trade fee: the table, charts and breakdown
     show "Pepper, your deal" dotted, in Pepper's color.
  2. Reload: it's still there.
  3. Switch to Tel Aviv: its own fees show.
  4. Make a new plan from scratch.
