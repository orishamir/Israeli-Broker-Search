# The redesign: a guide that teaches by example, and a workbench for those who know what to compare

Plan of record, 2026-10-19. Written for an agent (use subagents per work
package; §9 gives the dependency order). The goal is a **complete, working
redesign**, not a migration: nothing is kept for compatibility with the old
page (old share links, old UI preferences in localStorage, the two-calculator
switch). What is kept is every tariff, rule, test and source the core already
has, and the lessons in CLAUDE.md, LESSONS.md and `web/tests/CLAUDE.md`,
which still bind.

Read first: CLAUDE.md (whole), `web/tests/CLAUDE.md`, LESSONS.md "Working
with the user", "The core", "The web app", "Charts". Then the screens in
`docs/redesign/screens/` (`s1.png` … `s7.png`, rendered from `mock.html`,
which is the source of truth for wording and layout; `render.mjs` re-renders
them). Build to those screens. The proposal that led here, with the same
screens and captions: https://claude.ai/artifact/9ausZFVU8CCsvf7SGMvqJ4.

---

## 1. The essence (goes at the top of CLAUDE.md and README.md, as written)

The user's statement of what the site is, 2026-10-19:

> I created this site to compare banks and investment houses for investing in
> ETFs, and to visually understand clearly how compound interest looks like,
> and specifically how fees hurt that compounding effect: pie charts, line
> graphs, and between investing in different exchanges. I still want that.
> During the research I realized that isn't the only comparison worth making:
> the difference between a mutual fund (קרן נאמנות) and a קרן סל in Israel
> isn't trivial and might be more important than choosing between investment
> houses. And I realized I want both the easy route for newcomers, like I was
> at the time, who don't understand the different strategies (money market vs
> bond ETF vs stock ETF vs mutual fund vs פיקדון צמוד מדד) and want to learn
> by example, and an advanced route for when I specifically know what I want
> to compare and visualize.
>
> The site must answer: "I want to save for something 5 years from now" (the
> answer is sophisticated: can I tolerate ±1 year? the market may be down
> then; a money market fund isn't for that long; maybe a target-maturity
> instrument); "I want the S&P 500, I know my stuff, which way?" (compare
> visually: exchanges, brokers, ETF or mutual fund, maybe a קופת גמל if I'm
> retiring); "I make ₪20,000 a month and can save ₪3,000, but I don't know
> for what"; "I'm saving ₪1,500 a month for retirement" (probably a קופת גמל,
> for the tax benefit); "I'm saving for 15–20 years from now".
>
> When the site chooses something for someone, it must make clear **what** it
> chose (what is it? קרן סל? קרן מחקה? in the terms they meet in their bank's
> app, in ads, around them, so they become familiar with the topics and how
> they work) and **why** it chose it for them: how their circumstances
> affected the decision. And it never simply tells the answer: it tells it
> while visually showing how their money would look in the other choices they
> could have made and weren't recommended. The site guides through questions
> while simultaneously accepting the more advanced ones, and educates in the
> process.

Design consequences, all decided with the user on 2026-10-19:

- **One page, two routes, one engine.** Route A, the guide: three questions
  everyone can answer (what for, how much now and monthly, when), one or two
  follow-ups only when the answer needs them, then the answer. Route B, the
  workbench: "I know what I want to buy", pick the thing and the ways to
  compare it, every lever. The switch **כל האפשרויות** turns the guide's
  answer into the workbench in place; the guide's recommendation sets the
  workbench's inputs, so the expert sees exactly what produced the answer.
- **The horizon picks the arena; arenas never mix in a ranking.** Safe kinds
  (money market funds, deposits, government bonds held to maturity) and
  index kinds (index funds and ETFs through brokers, provident funds for
  investment, study funds, savings policies on an index track) are ranked
  each within its arena by what's left after tax, exactly as today. Across
  arenas the page *draws* the alternatives with each kind's own assumption
  and its risk, it never ranks them.
- **The answer card** holds: what (the kind, in the bank's words, with "מה
  זה? ›" to its learn card), how much (today's money, after tax; the future
  figure beside it), the current-account yardstick, why (reason by reason,
  each naming the circumstance), the alternatives drawn (with the index's
  historical band), the cheapest way within the kind, how it's done (steps),
  and where every shekel went (a donut).
- **Register:** "מה שמתאים לכסף כזה" (what fits money like this), by rules
  the page shows as the reasons. Never "ההמלצה שלנו". Footer:
  "המחשבון מסביר ומדגים לפי כללים שכתובים בו: הוא לא מכיר אתכם ולא מחליף
  ייעוץ אישי." and "לא קשור לאף בנק, בית השקעות או קופה."
- **The product's own cost is counted again** (the 2026-10-02 model,
  commit 494105b, reverted in d10c0eb): each way loses its fund's management
  fee, the dividend tax its structure loses, and its measured gap from the
  index. The workbench's "which way" table and the guide's "cheapest way"
  both use it.
- **Five years with a flexible date → the index kinds**, with the rule said
  ("if it's down then, you wait") and the history beside it. Fixed date →
  the safe kinds until 7 years.
- **New safe kinds now:** government bonds held to maturity (nominal and
  CPI-linked), fixed-rate deposits up to five years (the data already has
  them), CPI-linked deposits where banks publish rates (§5.4).
- **The index's history** on the alternatives chart: the band of what N-year
  periods did, and "in N of M such periods it ended lower", in dollars, said
  so.
- **The cushion:** for monthly saving with no goal, ask whether a cushion
  exists and the monthly expenses; the answer is two parts.
- **Today's money is the default** everywhere; the future figure is a note.
- **Ties are broken against a caveat**; the unlock line says what a bigger
  first deposit opens; the current account is the yardstick.
- **The title stays** "כמה יישאר לכם בסוף". Phones matter as much as desktops.

---

## 2. What stays, what goes

### Stays (reuse, don't rewrite)

- `crates/core`: `tariffs.rs`, `vehicles.rs`, `funds.rs`, `simulation.rs`,
  `short_term.rs` and `short_term/*`, `yours.rs`, `money.rs`, `percent.rs`,
  `Text`/`Lang`/`Named`, `Caveat` and its constructors, the tests in
  `tests/real_tariffs.rs` and `tests/economics.rs`, the bench, the policies
  and `sources.md`/`not-modeled.md`, the data scripts.
- `crates/wasm`: the binding style (wasm-bindgen + tsify, `set_lang` first,
  every binding answers in the language), the plan editor bindings
  (`copy_of`, `new_plan`, `set_*`, `price_list`, `simple_fees`…), `about`,
  `sweep`/`around`, `fees_for`, `broker_caveats`, data dates.
- `web`: Svelte 5 + ECharts; `app.css` tokens and motion; `Tip.svelte`/
  `tip.ts`; `Sheet.svelte`, `PlanSheet.svelte`, `BrokerPicker.svelte`,
  `PlanEditor.svelte` and its parts, `PriceList.svelte`, `YourPlans.svelte`,
  `DetailsDialog.svelte`, `Caveat*.svelte`, `Sources.svelte`,
  `NumberField.svelte`, `Choices.svelte`, `Price.svelte`, `WarningSign`,
  `Share.svelte`; the chart components and their pure option modules
  (`growth-chart.ts`, `fee-breakdown.ts`, `crossover-chart.ts`,
  `interest-split.ts`, `glide.ts`, `end-label.ts`, `echarts.svelte.ts`,
  `linked.svelte.ts`, `colors.ts`, `motion.ts`, `pointer.ts`, `format.ts`,
  `numbers.ts`, `typing.ts`, `held.svelte.ts`, `slider.ts`); the sweep
  worker (`sweeper.ts`, `sweep.worker.ts`); `saved.ts` and the `PlanData`
  storage of your plans (the core's own serialization; keep the key);
  `text.ts`/`he.ts`/`en.ts` (rewritten in content, same mechanism);
  the Playwright fixtures and layering.
- The deploy workflow, the accessibility statement (update its date and
  content), `index.html` (`lang="he" dir="rtl"`).

### Goes (delete, no shims)

- The two-calculator switch and everything that assumes it: `FamilySwitch.
  svelte`, `family` in the state, `short-term.svelte.ts` as a separate state
  (its logic moves into the one state), `InputsPanel.svelte`,
  `ShortTermInputs.svelte`, `Examples.svelte` and the two example cards,
  `PlaceSheet.svelte`/`PlaceTable.svelte` as separate pieces (one list
  component serves both arenas, §6.6), the three statistic cards in
  `App.svelte`, the "More options" switch (replaced by כל האפשרויות).
- The share-link format (new format §6.9; old links open the default page,
  no parsing of old parameters). Persisted UI preferences other than your
  plans (`more-options`, `editor-view`…): new keys, new meanings.
- `examples.rs` as long/short scenarios (replaced by `situations.rs`).
- Default exchange USA, nominal amounts by default, the editor opening in
  full mode.

---

## 3. The guide's rules (encode in `guide.rs`, test row by row, show as reasons)

Inputs the guide knows (`Answers`): situation (optional), money now, money a
month, horizon (months; or Unknown; or Retirement with the saver's age and
retirement age), flexibility (asked at 36–84 months: Fixed / CanWait), cushion
(asked when horizon Unknown and monthly > 0: Has / HasNot with monthly
expenses), needs-any-day (set by the cushion situation). The expected index
return, inflation and the Bank of Israel's rate are the app's defaults (10%,
2%, the published rate) unless the workbench changes them.

| The money | Arena and what fits | Reasons the user reads (shape; exact texts in `describe.rs`, both languages) | Drawn beside it |
|---|---|---|---|
| Needs-any-day, or horizon ≤ 12 months | Safe. Rank the safe kinds that fit; a money market fund wins unless a deposit leaves more *and* needs-any-day is off. | "כסף שאולי תצטרכו בכל יום…": reachable next day; earns about the Bank of Israel's rate. A deposit: locked, said so. | Deposits by term, the money fund, the current account. |
| 13–36 months, a known date | Safe. Rank: fixed deposit for the term, money market fund, government bond maturing at the date. | The horizon; one sum or monthly (deposits take one sum); what's known in advance. | The others; a stock index fund with its band. |
| 37–84 months, date Fixed | Safe. Rank among the kinds whose end sum is **known in advance** (bond to maturity, fixed deposit ≤ 60 months); the money fund is an alternative ("הריבית משתנה"). | Fixed date → stocks could be down exactly then; so long → the money fund's rate isn't promised; one sum → the deposit locks, the bond can be sold. | A stock index fund with its band and "ב־N מתוך M תקופות של X שנים נגמרה בהפסד". |
| 37–59 months, CanWait | Safe, as Fixed, but the reason says that from 5 years with flexibility stocks become reasonable. | | Both arenas. |
| 60–84 months, CanWait | **Index.** Rank the index kinds (the ways, §4.3) by what's left after tax. | "אם הבורסה תהיה בירידה אז, תחכו": the rule; the history sentence for this horizon; what safety would have cost. | The safe kinds, the current account. |
| ≥ 85 months (7 years and more) | Index. Rank the ways. | The horizon against the index's history for it; the cheapest way and why (fees after tax; the product's own cost). | The safe kinds ("מה עולה הביטחון לאורך זמן כזה"), the current account. |
| Retirement (age, retire-at) | Index, with the provident fund for investment **as a pension** when the saver is ≥ 60 at the end; the study fund shown with who can open one. | The tax-free pension against 25% in a brokerage account; the ceiling; withdrawable any time. | The brokerage account after tax, the study fund, the policy, the current account. |
| Horizon Unknown, monthly > 0 | Two-part answer: the cushion first if HasNot (money market fund until 3 months of expenses; months to fill = target ÷ monthly); then an index kind that can be opened any time (provident fund for investment or a broker's index fund, whichever leaves more). Then the ladder. | No cushion → sell in a fall; no date → a place you can open any time that grows. | The ladder (§4.5). |
| Horizon Unknown, one sum | The ladder, and its lesson. | | |
| Situation "כבר יש לי תיק בבנק" | The bank's branch price list ticked; the guide dialog (where to find your fees; transfer, don't sell; managed funds' fees aren't counted); the editor in simple mode. | | The same comparison. |
| "אני יודע מה אני רוצה לקנות" | The workbench, with the ways to compare. | | Everything. |

Fixed numbers, decided: arena boundary 7 years for a fixed date, 5 years with
flexibility; flexibility asked from 3 to 7 years; cushion target 3 months of
expenses (the words say "3–6"); deposits only up to 60 months (the Bank of
Israel publishes none longer); the ladder's horizons 1, 3, 5, 10, 20 years
(plus "any day" when there's a sum and no monthly amount).

Within an arena, "what's left after tax" decides, with: equal to the shekel
counts as a tie, and a tie goes to the plan without a may-cost-more caveat
(§4.6). A plan the first deposit can't open is never the answer; the unlock
line says what opens it.

---

## 4. The core

Keep as much logic as possible in Rust; the web side displays what it
returns. Everything below has English and Hebrew texts (`Text`), tests, and
`clippy --all-targets` pedantic clean.

### 4.1 `products.rs` and the ways (recover, then extend)

- `git show 494105b` is the reference: `crates/core/src/products.rs`
  (`Product`, `Costs { published, trailed, dividends_paid_out_taxed }`,
  `Product::for_purchase(security, exchange)`, `track_of(vehicle)`,
  `costs()`, `caveats()`), the `Scenario.product` field and
  `Fees.product` in `simulation.rs`, the funds' S&P 500 tracks in `funds.rs`,
  the tests added to `economics.rs` and `real_tariffs.rs`, the bench change,
  `policies/index-tracking.py`, and the LESSONS.md section "Researching
  what a fund keeps back" (`git show d10c0eb^:LESSONS.md`, line 102) and
  the `sources.md`/`not-modeled.md` sections. Recover them onto today's
  code (the tax model changed on 2026-10-19: foreign securities are taxed on
  the whole nominal gain, the surtax at its least; keep that).
- Re-run `policies/index-tracking.py` to the latest month and update the
  constants and the hand-worked cases. The recipes are in the memory and
  the script.
- New `ways.rs`: a **way** is how the saver holds the index:
  `Way { product: Product, vehicle: Vehicle, exchange: Option<Exchange> }`,
  listed by `Way::for_index()`: an index fund in Tel Aviv through a broker,
  an ETF in Tel Aviv, an ETF in the US, an ETF in Europe, the provident
  fund for investment's track, the study fund's, the savings policy's.
  `compare_ways(scenario, plans, rates) -> Vec<WayOutcome { way, best:
  PlanOutcome, cost_a_year: Percent /* product + fees */, plans: Vec<
  PlanOutcome> }>` runs each way's plans (the brokers that sell that
  security on that exchange; the fund kind for a track) and picks the
  cheapest per way; sorted by the best's after-tax, the tie rule applied.
  The guide's index arena recommendation is the first way; the workbench's
  table is the whole list (expandable to every plan of a way).
- Tests: a product's cost never makes a plan cheaper (economics); each way's
  cheapest plan is the minimum of its plans; the fund tracks' fees match
  the data; the ETF-in-Tel-Aviv way is dearer than the index-fund way at
  the defaults (a data assertion, dated in the test's comment).

### 4.2 Government bonds held to maturity (new kind, both arenas' data)

- Source: the Bank of Israel's SDMX portal, dataflow `BOI.STATISTICS/ZCM`
  ("ציפיות לאינפלציה ועקום 0"): `https://edge.boi.org.il/FusionEdgeServer/
  sdmx/v2/data/dataflow/BOI.STATISTICS/ZCM/1.0?format=csv&lastNObservations=1`.
  Series: nominal zero-coupon yields `ZC_TSB_ZND_{01,02,03,04,05,07,10,15}Y_MA`
  (NOMINAL_REAL=N, DATA_TYPE=ZC_YTM), real `ZC_TSB_ZRD_{01…20}Y_MA`
  (NOMINAL_REAL=R), monthly averages, latest 2026-09 at the time of writing
  (5-year nominal 3.72%, 1-year real 1.59%). Write `policies/yield-curve.py`
  that prints `crates/core/src/short_term/government_bonds.rs`: the curve by
  term, the month, the source `Page`. Interpolate linearly between the
  published terms for the saver's horizon; beyond 15 years not offered.
- Model: a shekel government bond bought once (or monthly, each purchase at
  the then-current yield assumed equal to today's) and held to the horizon
  earns the zero-coupon yield for that term; tax 15% of the nominal gain
  (unlinked; `policies/sources.md` "Tax and the vehicles" has the
  ordinance's sections, 91(ב)(3)); CPI-linked: the real yield plus
  inflation, tax 25% of the real gain. Fees: the brokers' bond fees already
  in `tariffs.rs` (`Security::Bond`, `Exchange::Tlv`): run the chosen
  broker plan through `simulate` with `yearly_return` = the yield, and
  `sell_at_end` as the redemption (no sale fee at redemption: add a flag
  `redeems_at_maturity` to the scenario that skips the sale fee and the
  conversion; the tariffs charge nothing for a redemption, cite the
  tariffs' rows in `sources.md`). The kind's "cheapest plan" is the broker
  ranking on bonds, which exists today.
- Liquidity: can be sold any day at the day's price (`Liquidity::AnyDay`
  with a note that the price moves until maturity: `Safety::SumKnown` only
  when held to the end).
- Caveats: yields are a monthly average, dated (`Caveat::at_most`? no:
  `published`, with the month); coupons are paid out and reinvested at the
  same yield in the model (`not_counted` for the reinvestment risk); the
  bond's name as the bank app shows it is an example ("ממשלתי שקלי 0331"
  is the format: verify the current series' names on TASE and cite).
- Tests: a hand-worked case in `real_tariffs.rs` (₪50,000, 5 years, 3.72%,
  tax 15%, Excellence's bond fees) with the arithmetic in a comment; the
  real variant with 2% inflation; interpolation at 6 years; not offered at
  16.

### 4.3 Deposits for any horizon, the money market fund for any horizon

- `short_term::LONGEST` goes; the safe arena's engine takes any number of
  months up to the app's longest horizon (50 years) so the alternatives
  chart can draw a money market fund over 20 years. Deposits beyond 60
  months → `NotOffered::TooLong` with the reason "הבנק המרכזי לא מפרסם
  ריביות לפיקדונות ארוכים מ־5 שנים". `Term::UpToFiveYears` already exists
  and `policies/deposit-rates.py` already reads "3 שנים עד 5 שנים".
- Fixed deposits with monthly saving stay `TakesOneSum` (banks' monthly
  savings plans aren't published), as today.
- The money market fund's rate over long horizons is the Bank of Israel's
  rate the saver expects (today's default) less the fee, as today; the
  alternatives chart labels it "הריבית משתנה".

### 4.4 CPI-linked deposits (try; report)

- The Bank of Israel's workbook `g060a.xls` has sheets L7.6.1a (fixed,
  unlinked: used), L7.6.2a (variable, unlinked), L7.6.3a/4a (credit). **No
  CPI-linked deposit rates there** (verified 2026-10-19). Look for each
  big bank's published CPI-linked deposit rates (their deposit pages list
  "פיקדון צמוד מדד" with a real rate by term). If at least three banks
  publish them with dates, add the kind: real rate + inflation, 25% of the
  real gain (ordinance 125ג: linked deposits are taxed on the real
  interest), locked to the end, `Caveat::published` with the page. If not,
  list it in `not-modeled.md` with the reason and the CPI-linked government
  bond as the kind that covers the need. Either way, the learn card
  "פיקדון צמוד מדד" exists (§4.8).

### 4.5 `guide.rs`: the recommendation, the ladder, the cushion

```
pub struct Answers { situation: Option<Situation>, now: Decimal, monthly: Decimal,
                     horizon: Horizon, flexibility: Option<Flexibility>,
                     cushion: Option<Cushion>, needs_any_day: bool }
pub enum Horizon { Months(u32), Unknown, Retirement { age: u32, retire_at: u32 } }
pub enum Flexibility { Fixed, CanWait }
pub enum Cushion { Has, HasNot { monthly_expenses: Decimal } }
pub enum Arena { Safe, Index }
pub enum Kind { MoneyMarketFund, FixedDeposit, LinkedDeposit, GovernmentBond { linked: bool },
                Way(Way), CurrentAccount }
pub enum Safety { SumKnown, RateChanges, CanFall }
pub struct Reason { text: Text, from: Circumstance }     // the circumstance that produced it
pub struct Alternative { kind: Kind, outcome: Outcome-like summary, safety: Safety,
                         why_not: Text, band: Option<Band> }
pub struct Recommendation { arena: Arena, kind: Kind, within: WithinKind /* the cheapest plan or way,
                            and the ranked list */, left: Money-in-today's-money + nominal,
                            current_account: Decimal, reasons: Vec<Reason>,
                            alternatives: Vec<Alternative>, unlock: Option<Unlock>,
                            steps: Vec<Step>, follow_ups: Vec<FollowUp>, two_parts: Option<CushionPlan> }
pub fn recommend(answers: &Answers, settings: &Settings, data: &Data) -> Result<Recommendation, InvalidAnswers>
pub fn ladder(answers: &Answers, …) -> Vec<Rung { horizon, kind, left, safety }>
pub fn follow_ups(answers: &Answers) -> Vec<FollowUp>   // which questions to show now
```

- `recommend` implements §3 exactly; each row of the table is a test in
  `tests/guide.rs` with the expected arena, kind and the reasons'
  `Circumstance`s. Every recommendation's alternatives include the current
  account and at least one kind of the other arena. The index arena's
  alternatives carry the `Band` from `history.rs` for the horizon.
- The cushion plan: `CushionPlan { target, months_to_fill, then: Kind }`.
- `follow_ups`: flexibility when 36 < months ≤ 84; age and retire-at when
  the situation is retirement; cushion and expenses when Unknown and
  monthly > 0. Nothing else is ever asked.
- Settings: expected index return, inflation, the Bank of Israel's rate, the
  exchange rates, the plans ticked (the workbench's), the expert's
  overrides (a chosen way or kind; a chosen arena).
- The alternatives' outcomes are computed with each kind's own assumption
  (§3's "drawn beside it"): the safe kinds through the safe engine, the
  index kinds through `compare_ways`, the current account as deposits
  deflated (`Scenario::deposited_in_todays_money`). All in today's money.
- Performance: `recommend` runs on every keystroke like `compare` today;
  the ladder (5–6 recommendations) and the alternatives' extra kinds run in
  the Web Worker (`sweep.worker.ts` gains a message kind). Add both to
  `benches/hot_paths.rs`; budgets in LESSONS.md "Speed".

### 4.6 `simulation.rs` changes

- Ties: the sort key becomes `(is_err, !fits, Reverse(after_tax rounded to
  the shekel), has_may_cost_more_for(scenario), Reverse(after_tax))`. Add
  `Plan::has_may_cost_more(security, exchange, largest_trade) -> bool` from
  its caveats (the `may_cost_more` kind, `about_fee`/`when_above` filters
  as the details apply them). Test: raising a plan from "no caveat" to
  "may cost more" never moves it above an equal plan (economics).
- `unlock(plans, scenario, rates) -> Option<Unlock { first_deposit, plan,
  after_tax, best_fit_after_tax }>`: the smallest `min_first_deposit` among
  plans the scenario doesn't fit; re-run the comparison with that first
  deposit; report the best plan then, if it beats the best fitting plan at
  that same deposit. Test: the reported deposit opens the plan; nothing
  reported when every plan fits.
- `redeems_at_maturity` (§4.2).
- `Comparison::in_todays_money` stays; the web defaults to it.

### 4.7 `situations.rs` (replaces `examples.rs`)

Each situation sets `Answers` (and, for the workbench one, the ways), with a
name and an explanation in both languages, and a lesson. The list, in this
order, with the chips' names: חיסכון מהמשכורת (1,500/month, 30 years),
לילד (300/month, 18 years), לדירה או למטרה בתאריך (100,000 + 2,000/month,
3 years, Fixed), כרית ביטחון (30,000, needs-any-day, 6 months), בונוס
(15,000 once, Unknown → the ladder), ירושה (500,000 once, 15 years),
לפרישה (3,000/month, age 45, retire at 65), עוד לא יודעים (0 + 3,000/month,
Unknown, HasNot, expenses 17,000), כבר יש לי תיק בבנק (200,000 once, 20
years, the branch plans ticked, the guide dialog opened), אני יודע מה אני
רוצה לקנות (the workbench with S&P 500 and every way). Keep the test that
every situation can be run and that each teaches a distinct lesson (the
explanation names it).

### 4.8 `describe.rs`: the words

All new text lives here, `Text` pairs, second person plural, the terms in
CLAUDE.md "Hebrew" (add: אג״ח ממשלתית לפדיון, פיקדון צמוד מדד, כרית ביטחון,
עקום תשואות, תשואה לפדיון, מסלול עוקב מדד, "סכום ידוע מראש").

- **Learn cards** (`LearnCard { term, name, english, as_the_bank_calls_it,
  as_ads_call_it, what, good_for, downside, cost, tax, where_met, example,
  compare_with: Vec<Term> }`), one per: קרן סל, קרן מחקה (and קרן נאמנות
  as the family), קרן כספית, פיקדון בריבית קבועה, פיקדון צמוד מדד, אג״ח
  ממשלתית (לפדיון), מק״מ (as a card even before it's a kind), קופת גמל
  להשקעה, קרן השתלמות, פוליסת חיסכון, קופת גמל לחיסכון (why it isn't here),
  חשבון מסחר, דמי ניהול, דמי משמרת וטיפול, עמלת קנייה ומכירה, מרווח המרה,
  מס רווחי הון, אינפלציה וכסף של היום, ריבית דריבית, מדד S&P 500, בורסה
  (תל אביב / ארה״ב / אירופה), קצבה, תקרת הפקדה, כרית ביטחון, תשואה לפדיון,
  ריבית בנק ישראל. The shape is `s5.png`. Every claim in a card is checked
  against a source and the source linked (`Page`), as caveats are. The
  short tips (`Explained`) stay one paragraph; a card is reached from the
  tip's "עוד על זה ›".
- **Reasons**: a text per `Circumstance` × arena, with the numbers filled
  (horizon, months, the history sentence for the horizon, the ceiling, the
  age).
- **Alternatives' why-not**: per kind × the circumstance that excluded it.
- **Steps** (`steps_for(recommendation)`): per kind: broker account (open
  which plan and its offer's condition, buy what and how it's called in the
  app, the monthly order, when to sell, the tax), provident fund for
  investment (open online, the cheapest company by name from `funds.rs`'s
  spread, the index track, the standing order, the pension at 60, early
  withdrawal), study fund (who can open; via the employer), savings policy,
  money market fund (buy by its TASE number through the bank or broker;
  most banks charge nothing; investment houses may charge per trade: check
  and caveat, report 5 §7), fixed deposit (in the bank's app, the term,
  breaking needs the bank's consent), government bond (the series' name
  format, hold to maturity, the automatic tax), CPI-linked deposit.
- **The glossary index** (`glossary(lang) -> Vec<LearnCard>` in a fixed
  order) and the header's "איך המספרים מחושבים" (`about`), merged into one
  page for both arenas with the new sections (the guide's rules in words,
  the history's method, the yield curve).
- **The footer texts** (§1 register).
- **The short-term money fund's name** on the cheapest fund's row and card
  (`short_term/money_market.rs` has the names).
- **Interactive's Tel Aviv reason**: a per-broker not-offered reason, so it
  says "רק ללקוחות מוסדיים".
- **The estate-tax note** ends with how to avoid it (a fund registered in
  Ireland or Israel).

### 4.9 `history.rs` and `policies/sp500-history.py`

- Source: Shiller's monthly data, `http://www.econ.yale.edu/~shiller/data/
  ie_data.xls` (live, 1.6 MB; sheet "Data": date, S&P composite price,
  dividend, CPI…). Build a monthly total-return index (price with the
  month's dividend/12 reinvested), nominal dollars, from 1926-01 (say why
  1926 in the script: the common convention; the data starts 1871 and the
  script can take a flag). For each horizon in years {1,2,3,4,5,6,7,8,9,
  10,12,15,20,25,30}: over every rolling window, the annualized return;
  write worst, 10th percentile, median, 90th percentile, best, the share of
  windows that ended lower than they started, the number of windows and
  the data's first and last month, into `crates/core/src/history/sp500.rs`.
- `history::band(years) -> Band { worst, p10, median, p90, best, share_lower,
  windows, from, to }` (interpolate between listed horizons). The chart's
  band uses worst and best applied as constant yearly rates (state that it's
  an approximation: real paths wander), the sentence uses `share_lower` and
  `windows` ("ב־N מתוך M תקופות של X שנים מאז 1926, בדולרים"). Everything
  said in dollars; the caption adds that a shekel fund also moves with the
  exchange rate.
- Tests: worst ≤ p10 ≤ median ≤ p90 ≤ best; share in [0, 1]; share for 20
  years ≤ share for 5 years; the 20-year worst is positive (nominal, a known
  fact; if the data says otherwise the test tells us). Record the numbers
  in `sources.md`.

### 4.10 Branch tariffs (data)

For each bank in `tariffs.rs` (Leumi, Mizrahi-Tefahot, Otsar Hahayal; since
19 October 2026 also Hapoalim, Discount and Beinleumi): add
the plan "בסניף או דרך בנקאי" from the same PDF in `policies/` (the
non-direct channel's rows: trading, custody, handling, conversion), unticked
at first, with a hand-worked case per bank in `real_tariffs.rs` and the rows'
readings in `sources.md`. Where the PDF has no such column, say so in
`not-modeled.md`. These plans are what the situation "כבר יש לי תיק בבנק"
ticks.

---

## 5. Data scripts and sources (all in `policies/`, documented in `sources.md`)

| Script | Writes | Source |
|---|---|---|
| `yield-curve.py` (new) | `short_term/government_bonds.rs` | Bank of Israel SDMX `BOI.STATISTICS/ZCM` (nominal `ZC_TSB_ZND_*`, real `ZC_TSB_ZRD_*`, monthly) |
| `sp500-history.py` (new) | `history/sp500.rs` | Shiller `ie_data.xls` |
| `index-tracking.py` (recovered) | `products.rs` constants | Maya, TASE, Gemel Net (recipes in the script and LESSONS) |
| `deposit-rates.py` (as is) | `short_term/deposits.rs` | Bank of Israel `g060a.xls` L7.6.1a (fixed, unlinked, to 5 years) |
| `money-market-funds.py`, `gemel-net.py` (as is; re-run) | funds' fees | Maya, Gemel Net |
| CPI-linked deposits (§4.4) | `short_term/linked_deposits.rs` if found | the banks' pages |

`sources.md` gains sections: "The guide's rules" (the table of §3 and why
each boundary), "Government bonds and the yield curve", "The index's
history", "Branch tariffs", "CPI-linked deposits"; `not-modeled.md` gains:
what the guide doesn't do (mixes and portfolios, personal advice, pension
funds and קופת גמל לחיסכון, the reinvestment of coupons, bond price risk
before maturity, the exchange rate in the history), and the removed items
when they're modelled.

---

## 6. The web app

Build to `docs/redesign/screens/`. One page, Hebrew, right to left, dark.

### 6.1 State (`app.svelte.ts`, one class)

- `answers` (the guide's inputs; `$state`), `expert: boolean` (the switch;
  persisted, in links), workbench inputs: `thing` (S&P 500 / another index or
  stock / bonds / short-term money), `ways: Set<Way>`, `security`,
  `exchange`, `sharePrice`, `yearlyReturnPercent`, `inflationPercent`,
  `boiRatePercent`, `depositGrowthPercent`, `buyEveryMonths`, `atEnd`,
  `wayOut`, `age`, exchange rates and their status, the ticked plans and
  your plans (as today), `overrides` (a kind or way the user chose on
  "לשנות", an arena override). The recommendation sets the workbench's
  derived inputs unless overridden; turning the switch on changes nothing
  but what's shown.
- `recommendation` (`$derived` from `core.recommend`), `comparison` (within
  the arena: `compare` or `compareWays`), `ladder` and `alternatives` from
  the worker (request/answer matching as `sweepAnswer` does), `sweep`,
  `inputError` and the stale-results behaviour (keep).
- Today's money default on (`todaysMoney = true`), the nominal figure as a
  note.

### 6.2 Components

| Component | Does | Screen |
|---|---|---|
| `Questions.svelte` | situations strip (chips with tips), the money fields, the horizon field (number + unit menu חודשים/שנים; "עוד לא יודעים" link; retirement shows age fields), the follow-up card(s) (`transition:reveal`), the expert exit link | s1 |
| `Answer.svelte` | "מה שמתאים לכסף כזה": the what-sentence with "מה זה? ›" and "לשנות ›", the amount (today's money; nominal note), the current-account line, the reasons, the unlock line, the two-part cushion plan | s2, s3, s6 |
| `Alternatives.svelte` + `alternatives-chart.ts` | the chart (lines per kind, the band as two stacked transparent series with an area, end labels for three lines only, no label wrapping, ids per kind), the rows under it (kind, safety tag, assumption, amount, "why not", tap to continue with it anyway) | s2, s3 |
| `Within.svelte` | "איפה הכי זול לקנות אותה": the top three plans or ways with a link to all; in expert mode the full table (`ResultsTable`-like: one component for both arenas with kind-specific columns: נשאר אחרי מס, עמלות/עלות בשנה, מס; for the safe arena ריבית נטו and מתי אפשר למשוך) | s2, s4, s7 |
| `Steps.svelte` | "איך עושים את זה" | s2, s3 |
| `Donut.svelte` + `donut.ts` | "לאן הולך כל שקל בסוף": deposits, gain kept, fees (and the product's own cost as its colour), tax; tabs to see an alternative's | s2, s3 |
| `Ladder.svelte` | the rungs; tap sets the horizon | s6 |
| `LearnCard.svelte`, `Glossary.svelte` | a `Sheet` with the card; the glossary index in the header | s5 |
| `Workbench.svelte` | what to buy, the ways (multi-tick), the levers (security, exchange, rates, unit price, expectations, buy every, growing deposits, keep/sell, pension), "מה משווים" with the plan sheet and the editor | s4, s7 |
| `ExpertSwitch.svelte` | כל האפשרויות in the header; persisted | all |
| `Charts` (existing) | growth, lost to fees, breakdown (with "הקרן עצמה"), by deposit, interest split, plus the donut; tabs as today; in guide mode only growth and the donut are offered | s4 |
| `App.svelte` | header (title, glossary, about, share, switch), the layout (§6.4), the footer (§1), the best-bar on phones | |

Delete the components listed in §2 "Goes".

### 6.3 Charts' rules (CLAUDE.md "Known traps" bind; add)

- The alternatives chart: every series has an `id` (the kind); the band's
  two series are `stack`ed with the lower one's line transparent and the
  upper one's `areaStyle` (see `mock.html`); labels never wrap or truncate
  (`fitName`); only three end labels (the recommended kind, the stock
  fund, the current account); hover highlights via `dispatchAction`; the
  table-and-chart linking (`linked.svelte.ts`) covers the alternatives'
  rows.
- The donut's labels are short (הפקדה / רווח / עמלות / מס / הקרן עצמה) and
  `overflow: 'none'`; a slice under 0.5% is listed under the donut rather
  than labelled on it.
- Today's money everywhere; the growth chart's y-axis says so.

### 6.4 Layout

- Phone: one column: header, questions, answer, alternatives, within,
  steps, donut, (ladder), charts; the best-bar at the bottom while the
  answer is below the fold (keep `placeBar`).
- Desktop (≥ 800px): two columns as `s7.png`: the questions column (and, in
  expert mode, the workbench cards; "איך עושים את זה" joins it when there's
  room), the answer column (answer, alternatives beside the donut, within,
  charts).
- Stable layout rules (CLAUDE.md "Stable layout" / LESSONS): follow-up cards
  and the unlock line open and close with `reveal`; nothing reserves empty
  space; heights glide (`glideHeight`).

### 6.5 Texts

`he.ts`/`en.ts` keep only the page's own words (labels of the fields,
buttons, headers, the switch, the footer); every sentence about money, kinds,
reasons, steps and cards comes from the core. Delete the keys that die with
the old page.

### 6.6 One list for both arenas

Replace `ResultsTable` + `PlaceTable` with one `Within` list whose columns the
core names (`Column` list per arena from a binding), so the safe arena's
rows (fund/deposit/bond) and the index arena's (plans/ways) render through
the same component, with the fixed name-column share and the whole-column
scroll steps (CLAUDE.md "CSS").

### 6.7 Learn layer wiring

Every `Tip` whose subject has a card gets "עוד על זה ›" (the core says
which: `Explained.card: Option<Term>`); every "מה זה? ›" opens the card;
the glossary lists all cards; a card's "לעומת" chips open the other card
in the same sheet (a back button).

### 6.8 The situation "כבר יש לי תיק בבנק"

Ticks the user's bank's branch plan (asks which bank with chips inside the
answer: לאומי / מזרחי-טפחות / אוצר החייל / בנק אחר → the most expensive
branch plan as a stand-in, said so), opens a dialog with the guide's words
(§4.8 steps for the existing-account case), and offers "+ מסלול משלכם" in
simple mode.

### 6.9 Links (`link.ts`, new format)

`?v=2&…`: the answers (now `n`, monthly `m`, horizon `h` as months or `u`
or `r:<age>:<retire>`, flexibility `fx`, cushion `cu` with expenses,
situation `si`), `x=1` for expert, the workbench inputs when expert or
overridden (as today's keys), the ticked plans by English label, your
ticked plans as data (as today). No parsing of links without `v=2`. Unit
tests round-trip every field; the Playwright `app.spec.ts` opens a link and
checks the page.

### 6.10 Persistence

Keep your plans and your deposits as they are (the core's `PlanData` and the
deposit record). New keys for the switch (`expert-v2`) and the editor view.
Nothing else is remembered.

---

## 7. Tests (each in the cheapest layer; expectations from the core)

- **Rust** (new files): `tests/guide.rs` (every row of §3; follow-ups; the
  cushion arithmetic; the ladder's rungs are monotone in horizon within an
  arena; alternatives always include the current account and the other
  arena; the reasons' texts are non-empty in both languages and name their
  circumstance), `tests/history.rs` (§4.9), `tests/ways.rs` (§4.1),
  additions to `real_tariffs.rs` (branch tariffs, the bond case, the linked
  bond, CPI-linked deposits if added) and `economics.rs` (ties, unlock,
  product cost never helps, the redemption charges no sale fee, the current
  account equals deposits deflated). `cargo mutants -p broker-fees` on
  `guide.rs`, `ways.rs`, `history.rs`: turn the surviving mutants that
  matter into tests.
- **Unit (vitest)**: the state's wiring (answers → recommendation; the
  switch changes nothing but the view; overrides), `alternatives-chart.ts`
  and `donut.ts` options (ids, the band's two series, labels, colours from
  `colors.ts`), the ladder, `link.ts` round trips, the glossary lookup, the
  worker's request matching.
- **Playwright** (new table in `web/tests/CLAUDE.md`): `questions.spec.ts`
  (typing, chips, follow-ups appearing, the horizon unit menu),
  `answer.spec.ts` (the sentence, "מה זה", "לשנות", the unlock line, the
  two-part plan), `alternatives.spec.ts` (rows ↔ chart linking, "continue
  anyway"), `workbench.spec.ts` (the switch, the ways, the full table, the
  charts), `learn.spec.ts` (cards from tips, the glossary, back),
  `acceptance.spec.ts` (§8), plus the kept `editor`, `details`, `tips`,
  `chart`, `breakdown`, `layout` (states and snapshots regenerated; the
  planted-bug checks kept), `hebrew`, `app`. Tags decide devices as today;
  every new screen gets a `@phone` state in `layout.spec.ts`.
- **Perf**: `npm run test:perf` budgets kept (LESSONS "Speed"); the ladder
  and the alternatives' extra kinds in the worker; measure and record.

---

## 8. Acceptance: the user's five questions, and the audit's five tasks

Each is a Playwright test in `acceptance.spec.ts` (expectations from the
core via `tests/core.ts`: `recommend`, `ladder`) and a manual pass on a
Samsung-sized phone and a desktop before the work is called done.

| # | Inputs | Expected |
|---|---|---|
| a1 | ₪50,000 now, 0 monthly, 5 years, Fixed | Safe arena; a government bond to maturity or a 5-year deposit (whichever the data leaves more) named in the bank's words; reasons name the fixed date, the horizon, the one sum; alternatives include a stock index fund with the band and the "N of M" sentence; the within-list is the broker ranking on bonds; steps; donut. |
| a2 | same, CanWait | Index arena; the cheapest way (expected: an index fund in Tel Aviv); the reason says "if it's down, you wait" with the 5-year history; the safe kinds drawn. |
| b | "אני יודע מה אני רוצה לקנות", S&P 500, all ways, ₪10,000 + ₪2,000/month, 20 years | The ways table with the product's own cost; the breakdown shows "הקרן עצמה"; all charts; the switch on. |
| c | 0 now, ₪3,000/month, Unknown, HasNot, expenses ₪17,000 | Two-part answer: cushion ₪51,000 in a money market fund, 17 months; then the provident fund for investment or a broker's index fund; the ladder with 5 rungs. |
| d | 0 now, ₪1,500/month, retirement, age 40, retire at 60 | The provident fund for investment as a pension; the tax-free reason; alternatives: brokerage after tax, study fund with who can open, policy, current account. |
| e | ₪10,000 + ₪2,000/month, 17 years | Index arena; the history sentence for 17 years; the safe kinds drawn with "what safety costs". |
| T1 | ₪15,000 once, Unknown (the bonus) | The ladder; tapping 20 years gives the index arena with the tie rule applied (Excellence before Altshuler at equal shekels, or the ⚠ on the card if Altshuler is strictly best). |
| T2 | 0 + ₪500/month, 30 years | Index; the provident fund for investment with "אפשר למשוך בכל עת" and the track; the unlock line names the plan a ₪5,000 first deposit opens. |
| T3 | ₪100,000 + ₪2,000/month, 3 years, Fixed | Safe; the money market fund (deposits take one sum); the stock fund drawn with its band; "continue anyway" works. |
| T4 | "כבר יש לי תיק בבנק", Leumi, ₪200,000, 20 years | The branch plan ticked and ranked; the dialog's words; the editor in simple mode. |
| T5 | the defaults | Today's money; the current-account line answers "do I need to care". |

---

## 9. Work packages, order and ownership (for subagents)

Dependencies in brackets. Each package ends with `cargo test`, `cargo
clippy --all-targets` (0 warnings), `cargo fmt --check`, and, when it touches
the wasm or the web, `npm run wasm && npm test` and `npm run lint`; a package
that changes how the page behaves also runs `npm run test:perf`. **After any
Rust change, `npm run wasm`.**

1. **WP1 products and ways** [none]: §4.1. Recover 494105b, adapt to the
   2026-10-19 tax model, re-run `index-tracking.py`, `ways.rs`, tests.
2. **WP2 safe kinds** [none]: §4.2, §4.3, §4.4. `yield-curve.py`, government
   bonds, deposits for any horizon, CPI-linked deposits (try), tests,
   `sources.md`.
3. **WP3 history** [none]: §4.9. `sp500-history.py`, `history.rs`, tests,
   `sources.md`.
4. **WP4 simulation rules** [none]: §4.6. Ties, unlock, redemption, tests.
5. **WP5 branch tariffs** [none]: §4.10.
6. **WP6 guide** [WP1–4]: §4.5, §4.7. `guide.rs`, `situations.rs`, the
   cushion, the ladder, `tests/guide.rs`, mutants, bench.
7. **WP7 words** [WP6 types; can draft earlier]: §4.8. Learn cards, reasons,
   why-nots, steps, glossary, about, footer, the fund's name, Interactive's
   reason, the estate-tax note. Every claim sourced.
8. **WP8 bindings** [WP6, WP7]: `recommend`, `ladder`, `alternatives`,
   `compare_ways`, `learn_card`, `glossary`, `situations`, `follow_ups`,
   `columns_for(arena)`, `history_band`; remove the dead ones; tsify types.
9. **WP9 web** [WP8; scaffold against `mock.html` from day one]: §6. State,
   components, charts' option modules with unit tests, layout, links,
   persistence, the worker, the best-bar, motion.
10. **WP10 specs and perf** [WP9]: §7, §8. Regenerate `layout.spec.ts`
    snapshots after the layout is final; record perf numbers in LESSONS.
11. **WP11 docs** [all]: CLAUDE.md (new opening with §1's essence; "Layout"
    rewritten for the new modules and components; "Tests" table; "UI/UX
    preferences" gains: a recommendation always says what, why, how and
    draws the alternatives; the learn-card shape; the register; "Hebrew"
    gains the new terms; "Known traps" gains §6.3), README.md (the essence,
    the two routes, what's compared, the sources, "Development"),
    LESSONS.md (recovered section, the redesign's lessons, perf numbers),
    `web/tests/CLAUDE.md`, `policies/sources.md`, `policies/not-modeled.md`,
    the accessibility statement (date and content), `web/public/og.png`
    (a new picture of the answer screen, in Hebrew), the memory of this
    plan's outcome.

Run WP1–WP5 in parallel; WP6–WP8 next; WP9 in parallel with WP7's drafting;
WP10 and WP11 last, with a final full "Done means" run and the §8 manual
pass. Commit per work package with a message that says what and why; push
to `main` only when the user says so (the deploy workflow runs on push).

---

## 10. Decisions the agent makes alone (report them), and ones it doesn't

Decide and report: the exact chip order; colours for the new kinds (use the
`dataviz` validator against `colors.ts`); the ladder's rungs' wording; the
cushion's rounding; the bond series' name format after checking TASE; the
history's starting year flag default (1926); the columns' exact shares; the
worker's message shapes; where "איך עושים את זה" sits on desktop at each
width; the glossary's order; error texts for impossible answers (a horizon
of 0, expenses of 0 with HasNot).

Ask the user (stop and report, don't guess): any new tariff row whose
reading is unclear (CLAUDE.md "Tariffs"); a history figure that contradicts
the texts (e.g. a 20-year loss); a source for a learn-card claim that can't
be found; a perf budget that can't be met; anything in `mock.html` that
turns out impossible or misleading once real numbers run through it.

---

## 11. Done means

All of CLAUDE.md "Done means", plus: every row of §8 passes on the desktop
project and the `@phone` projects; `cargo mutants` on the new modules with
the survivors judged; the perf suite within budget with the new charts; the
docs of WP11 updated; the five questions of §1 answered on the live dev
server by hand, on a phone-sized viewport, with screenshots on one contact
sheet for the user's review; the old page's components, texts, link
parameters and preferences gone from the tree (`grep` for `family`,
`FamilySwitch`, `moreOptions`, `shortTermExamples` finds nothing).

---

## 12. Amendments

- **No named index (2026-10-19).** The guide never suggests an index, a
  fund or a ticker. Its stock kind is named as a kind, "קרן מחקה מדד מניות
  רחב" (`Way::kind_name`, for the guide's what-sentences, reasons, steps,
  alternatives, ladder and tables); the workbench's `Way::name` may keep a
  ticker as an example in its expert list. The costs and the history band
  still come from S&P 500 data, and the page says so once, in the history's
  caveat (`basis_sentence`: "להדגמה, ההיסטוריה והעלויות נמדדות על מדד S&P
  500"). `guide_words.rs` fails if a guide text names one.
