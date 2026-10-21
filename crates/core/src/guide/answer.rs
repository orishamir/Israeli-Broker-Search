//! The answer as REDESIGN.md Part 2 lays it out: how sure the site is (the
//! chip), the four lines (קודם, סוג המקום, הדרך, איפה), each stated or a
//! decision only the visitor can make, the plan once every decision is made,
//! and the parts below it: what every option leaves on the date, how to do
//! it, how much of the gain stays, and what changes if the date moves.
//!
//! It is built on the rules' own machinery ([`super::route`]): the routes
//! decide the arena and rank the kinds and the ways; this module asks them
//! once for each arena, reruns the options under the versions of
//! [`super::robust`], and lays the results out. A decision appears only when
//! its answers lead to different outcomes (principle 2), and one that
//! depends on another only after it (principle 8).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::describe::how::Fact;
use crate::describe::{
    card, gain as gain_words, how as how_words, outcome as row_words, path as path_words,
};
use crate::history::{self, DepositsBand};
use crate::ways::WayOutcome;
use crate::{Text, Vehicle};

use rust_decimal::prelude::ToPrimitive;

use super::index::{Run, RunAsked, open_to_anyone};
use super::outcome::{
    Place, Split, at_end, bank_range, plan_outcome, safe_priced, safe_value, split, tie_groups,
    way_places, way_value, yardsticks,
};
use super::robust::{
    self, Standing, Varies, Version, kind_margin, provider_margin, standing, switch_rate,
};
use super::route::{self, Route};
use super::safe::Priced;
use super::{
    Amount, Answers, Arena, Bank, CUSHION_MONTHS, Cushion, Data, Flexibility, HORIZON_YEARS,
    Horizon, InvalidAnswers, Kind, NO_DATE_SUM_YEARS, Overrides, STAND_IN_MONTHLY_EXPENSES,
    Settings, Situation, check,
};

// ─────────────────────────── What the visitor chose ───────────────────────────

/// The decisions the visitor made on the answer that aren't facts about the
/// money: the facts (whether the date can move, the cushion) stay in
/// [`Answers`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Choices {
    #[serde(default)]
    pub safe_way: Option<SafeWay>,
    #[serde(default)]
    pub wrapper: Option<Wrapper>,
    #[serde(default)]
    pub account: Option<Account>,
    #[serde(default)]
    pub holdings: Option<Holdings>,
}

/// The safe ways, grouped by what they ask of the saver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum SafeWay {
    /// In the bank's app, sold any day; the rate changes.
    MoneyMarketFund,
    /// An Israeli government bond held to maturity: a trading account.
    Bond,
    /// A fixed-rate deposit: locked until its term ends.
    Deposit,
}

/// Where the money for the index builds up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Wrapper {
    /// The way anyone can open that leaves the most.
    Anyone,
    /// A study fund the saver has (self-employed, or through an employer).
    StudyFund,
}

/// Where a product bought on an exchange is bought.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Account {
    /// A new account at the place that leaves the most.
    NewAccount,
    /// Through the saver's own bank.
    YourBank,
}

/// What a portfolio already at a bank holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Holdings {
    IndexFunds,
    Managed,
    DontKnow,
}

/// The cushion's two answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum CushionPick {
    Has,
    HasNot,
}

/// An option's choice, as the page sends it back: the facts go into
/// [`Answers`] (`Cushion`, `Flexibility`), the rest into [`Choices`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Pick {
    Cushion(CushionPick),
    Flexibility(Flexibility),
    SafeWay(SafeWay),
    Wrapper(Wrapper),
    Account(Account),
    Holdings(Holdings),
}

/// Which decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum DecisionId {
    /// "Do you already have 3 months of expenses aside?"
    Cushion,
    /// "If the market is down then, can you wait?"
    PlaceType,
    SafeWay,
    Wrapper,
    Account,
    Holdings,
}

// ─────────────────────────── The answer ───────────────────────────

/// A row of the outcome chart is one kind of place.
pub type RowId = Kind;

/// The chip (§2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum ConfidenceState {
    /// כאן קשה לטעות: the same direction wins in every version, by far.
    A,
    /// ההבדל קטן: within the margin, or the order flips between versions.
    B,
    /// תלוי במה שחשוב לכם: the higher expected outcome has a downside only
    /// the visitor can weigh.
    C,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Confidence {
    pub state: ConfidenceState,
    pub note: Text,
}

/// The four lines, in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum LineKind {
    /// קודם: a step that must come first (the cushion).
    Before,
    /// סוג המקום: safe, or the stock index.
    PlaceType,
    /// הדרך: a deposit, a fund, a bond, an ETF, a study fund…
    Way,
    /// איפה: which provider.
    Where,
}

/// How sure a stated line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Pill {
    /// ברור.
    Clear,
    /// תיקו.
    Tie,
    /// לפי הכללים: the site's rules, not a margin, decide it.
    Rules,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum LineState {
    Stated {
        text: Text,
        pill: Pill,
    },
    Decision(Decision),
    /// Depends on a decision above: "יתברר אחרי ההחלטה למעלה".
    Waiting,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Line {
    pub kind: LineKind,
    pub label: Text,
    pub state: LineState,
}

/// A decision (§2.4): the question, why it's the visitor's, the options side
/// by side, and the difference.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Decision {
    pub id: DecisionId,
    pub question: Text,
    pub why: Text,
    pub options: Vec<Opt>,
    /// The ₪ difference, and what could reverse it.
    pub difference: Option<Text>,
    pub chosen: Option<Pick>,
}

/// One option of a decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Opt {
    pub pick: Pick,
    pub title: Text,
    /// What it leaves, with how certain that is in words.
    pub amount: Text,
    /// A second amount in another unit, under the first: the monthly
    /// pension a provident fund's sum would pay (the review's N7).
    pub also: Option<Text>,
    pub value: Option<Amount>,
    /// At most 3.
    pub facts: Vec<Text>,
    /// Those of `facts` that say what it asks of the visitor: the phone's
    /// table gives them the row "מה זה דורש", the rest go behind "עוד"
    /// (REDESIGN.md §4.3).
    pub asks: Vec<Text>,
    /// "מתאים אם…".
    pub fits_if: Option<Text>,
    /// The same without its "מתאים אם:", for the phone's table, whose row
    /// is called that.
    pub fits: Option<Text>,
    pub button: Text,
    /// The chart's rows it stands for.
    pub rows: Vec<RowId>,
}

/// "מה יהיה לכם ב‑<month year>" (§2.6, amended by §4.6).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outcome {
    /// "October 2029", or "in 3 years" when the page gave no date.
    pub date: Text,
    /// The chart's title: "מה יהיה לכם באוקטובר 2029".
    pub title: Text,
    /// Under the title (§4.6): "בכסף של היום, אחרי עמלות ומס. המניות לפי
    /// מדד S&P 500, לדוגמה. הקישו על שורה…", the stocks' part only where a
    /// stock row is drawn.
    pub caption: Text,
    pub deposited: Amount,
    pub current_account: Amount,
    /// "הפקדתם" and "בעו״ש", drawn on the chart as labelled lines (§4.6).
    pub lines: Vec<RefLine>,
    /// The ₪ axis, today's money: a little below the lowest bad case to a
    /// little above the highest typical value.
    pub axis: (Decimal, Decimal),
    /// At most 5, and only options the visitor could choose (§4.6): no row
    /// for what the rules leave out.
    pub rows: Vec<Row>,
}

/// Which reference line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum RefLineKind {
    Deposited,
    CurrentAccount,
}

/// A reference line on the chart, with its label and its amount.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RefLine {
    pub which: RefLineKind,
    /// "הפקדתם", "בעו״ש".
    pub label: Text,
    pub value: Amount,
    /// Rounded as every amount that rests on an assumption is: "₪136,000".
    pub amount: Text,
}

/// Which mark on a stock row's band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum BandMarkKind {
    Worst,
    Assumed,
    Median,
    Best,
}

/// A mark on a stock row's band, labelled with its amount (§4.6):
/// "הגרועה בעבר ₪281 אלף".
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BandMark {
    pub which: BandMarkKind,
    /// "הגרועה בעבר", "10% בשנה", "חציון העבר", "הטובה".
    pub label: Text,
    pub value: Amount,
    /// Short, for the chart: "₪281 אלף", "₪1.23 מיליון".
    pub amount: Text,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub id: RowId,
    pub kind: Kind,
    pub name: Text,
    pub tags: Vec<Text>,
    /// The facts under the name, as plain text (§4.6): "הריבית משתנה, בבנק
    /// או בבית השקעות" — [`Self::tags`] joined.
    pub facts: Text,
    pub mark: Mark,
    /// A stock row's band's marks, lowest first, each labelled with its
    /// amount (§4.6); empty for any other mark.
    pub marks: Vec<BandMark>,
    /// The figure shown: the known sum, the fund's at today's rate, the
    /// index's at the expected return.
    pub value: Amount,
    /// [`Self::value`] in words, rounded to how sure it is (principle 13):
    /// a sum known in advance to the shekel, one resting on a rate or a
    /// return to three figures ("בערך ₪427,000").
    pub amount: Text,
    /// Its figure lies beyond the axis (an excluded row, which doesn't
    /// stretch it): drawn as an arrow at the edge, with its amount.
    pub tail: bool,
    /// Consistent with the visitor's choices (rows dim otherwise).
    pub active: bool,
    /// Why the rules leave it out, if they do (it stays, dimmed).
    pub excluded: Option<Text>,
    pub note: Option<Text>,
}

/// How a row's figure is drawn.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Mark {
    /// A dot: the sum is known.
    Known,
    /// A dot on a short bar: the fund's figure at the rate versions.
    RateRange { low: Amount, high: Amount },
    /// A bar from the historical worst to the best for this horizon and
    /// these deposits, ticks at the median and at the expected return.
    Market {
        worst: Amount,
        median: Amount,
        assumed: Amount,
        best: Amount,
        /// "ב‑N מתוך M תקופות…": the past said as the past.
        losses: Text,
    },
    /// The index where the history doesn't cover the horizon (under a year,
    /// over 30): only the expected return's figure.
    Expected,
}

/// "איך עושים את זה" for one row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct How {
    pub row: RowId,
    pub name: Text,
    /// The identity line (§2.8), its parts in order. Replaced on the page by
    /// [`Self::facts`] (§4.8).
    pub identity: Vec<Text>,
    /// The facts as labelled rows (§4.8): מה זה, כמה מרוויחה, מתי אפשר
    /// למשוך, מטבע, the same questions in the same order for every product.
    pub facts: Vec<Fact>,
    pub search: Option<Search>,
    pub steps: Vec<Text>,
    /// Why it isn't offered here, for an excluded row.
    pub not_offered: Option<Text>,
    pub where_to_buy: Option<WhereToBuy>,
}

/// What to type in the app. `placeholder` marks a slot the data can't fill
/// (no bond series is repaid by the date): shown as such, and saying why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Search {
    pub text: Text,
    pub placeholder: bool,
    /// The pages its names and numbers come from.
    pub sources: Vec<crate::Page>,
}

/// Where to buy, from the visitor's bank (§2.9).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhereToBuy {
    pub title: Text,
    /// The places other than the banks while the visitor hasn't named one
    /// (the banks are [`Self::reference`] then), or every place shown, the
    /// visitor's bank marked `yours`.
    pub rows: Vec<Provider>,
    /// Until the visitor names a bank: the lowest and highest the banks'
    /// online plans leave.
    pub bank_range: Option<(Amount, Amount)>,
    /// "דרך בנק" as one row, the reference point (§4.7), until the visitor
    /// names a bank: from the dearest bank to the cheapest.
    pub reference: Option<BankReference>,
    /// "באינטראקטיב: בין ₪88,500 ל‑₪105,000 יותר מאשר דרך בנק, לאורך 30
    /// שנים." (§4.7), against the visitor's bank once named.
    pub difference: Option<Text>,
    pub note: Option<Text>,
    /// Naming the visitor's bank would change what's shown here.
    pub asks_bank: bool,
    /// What a bigger first deposit would open.
    pub unlock: Option<Text>,
    /// A plan that leaves more but may cost more.
    pub may_cost_more: Option<Text>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Provider {
    pub name: Text,
    pub left: Amount,
    /// Against the visitor's bank's row, when there is one.
    pub versus_yours: Option<Amount>,
    /// The reference row ("נקודת השוואה").
    pub yours: bool,
    /// Rows with the same number are a tie; 0 is the best's.
    pub tie: u8,
    /// Its price may be too low ("May cost more: …"): marked ⚠ on the row.
    pub warning: Option<Text>,
    /// [`Self::left`] in words, rounded as the chart's rows are.
    pub amount: Text,
}

/// "דרך בנק", one row for the banks (§4.7).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BankReference {
    /// "דרך בנק".
    pub name: Text,
    /// "נקודת השוואה, מהיקר ועד מזרחי באונליין".
    pub note: Text,
    pub low: Amount,
    pub high: Amount,
    /// "בערך ₪322,000 עד ₪339,000", rounded as the rows are.
    pub amount: Text,
}

/// "כמה מהרווח נשאר אצלכם" (§2.7).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Gain {
    pub lead: Text,
    pub rows: Vec<GainRow>,
    pub takeaway: Text,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GainRow {
    pub name: Text,
    /// What stays above the deposits.
    pub kept: Amount,
    pub tax: Amount,
    /// The broker's and the manager's.
    pub fees: Amount,
    /// The product's own cost (a fund's fee, dividends' tax lost).
    pub product: Amount,
    /// The parts, each with its name and amount (§4.7), in the bar's order:
    /// נשאר לכם, מס, עמלות, הקרן עצמה. A part of nothing is left out.
    pub parts: Vec<GainPart>,
    /// Beside the name: "פטורה ממס" where the gain pays no tax.
    pub note: Option<Text>,
}

/// Which part of a gain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum GainPartKind {
    Kept,
    Tax,
    Fees,
    /// "הקרן עצמה": the product's own cost, a colour of its own (§4.7).
    Fund,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GainPart {
    pub which: GainPartKind,
    /// "נשאר לכם", "מס", "עמלות", "הקרן עצמה".
    pub label: Text,
    pub value: Amount,
    /// "₪292,000", rounded as every amount there is.
    pub amount: Text,
}

/// One row of "ואם התאריך יזוז?" (§4.10): what the answer would show if the
/// date were `months` away, so that moving the date opens exactly it. The
/// dates are built around the visitor's (`dates_around`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HorizonRow {
    pub months: u32,
    /// "בעוד 5 שנים", "בעוד 30 שנה".
    pub when: Text,
    /// The row is the visitor's own date.
    pub current: bool,
    /// "התאריך שלכם", on the visitor's own row.
    pub label: Option<Text>,
    pub deposited: Amount,
    /// [`Self::deposited`] in words: "₪28,600" (the column is "הפקדתם").
    pub deposits: Text,
    /// "מה מתאים אז": one, or one for each answer about the date where the
    /// visitor hasn't given it and the two lead to different places.
    pub fits: Vec<HorizonFit>,
}

/// What fits at a row's date, and what it would come to ("כמה יהיה").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HorizonFit {
    /// Which answer about the date it is for; `None` where it doesn't
    /// matter there. Opening it gives that answer.
    pub flexibility: Option<Flexibility>,
    /// [`Self::flexibility`] in words: "אם תצטרכו בדיוק אז:", "אם תוכלו
    /// לחכות:".
    pub condition: Option<Text>,
    /// The way the answer would settle on, or the first of those it would
    /// leave to the visitor.
    pub kind: Kind,
    /// That way, or "X, Y או Z" where the answer would leave the way to the
    /// visitor.
    pub name: Text,
    pub low: Amount,
    pub high: Amount,
    /// The range in words, rounded to how sure it is.
    pub range: Text,
    /// What the range is made of: "ידוע כבר היום", the past's stretches...
    pub range_note: Text,
}

// ─────────────────────────── The plan and the path ───────────────────────────

/// The plan's amount (§4.5): "בערך ₪427,000", "ב‑2056, אם המדד יעלה 10%
/// בשנה", "בכסף של היום, אחרי מס. תפקידו בערך ₪136,000.", and for stocks
/// "החישוב לפי מדד S&P 500, כדוגמה למדד רחב."
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanAmount {
    pub value: Amount,
    pub amount: Text,
    pub when: Text,
    pub basis: Text,
    pub example: Option<Text>,
}

/// How a step of "איך הגעתם לזה" was settled (§4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum StepKind {
    /// The visitor chose it: "שינוי" opens the decision again in place.
    Visitor,
    /// The site stated it: "ברור", and "למה זה ברור?" where it has an
    /// explanation.
    Site,
    /// Still the visitor's, outside the site: which index ("בחירה שלכם").
    Outside,
    /// A decision still open: the page answers it in place. Only while a
    /// choice made on the path opened another decision (principle 8), so
    /// the path stays where the visitor is using it (principles 21, 22).
    Open,
}

/// One step of "איך הגעתם לזה" (§4.5): a line or decision of the answer, in
/// its order, or "איזה מדד".
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathStep {
    /// The line it stands for; `None` for "איזה מדד".
    pub line: Option<LineKind>,
    /// The line's label (§4.1), or "איזה מדד".
    pub label: Text,
    pub kind: StepKind,
    /// A site's step: how sure the line is (ברור / תיקו / לפי הכללים).
    pub pill: Option<Pill>,
    /// What was chosen, in bold: "יש לכם כבר כרית ביטחון", "קרן סל".
    pub choice: Text,
    /// One line on what it hinged on, besides money.
    pub hinge: Text,
    /// A visitor's step: the decision "שינוי" opens again.
    pub reopens: Option<DecisionId>,
    /// A site's step that has one: "למה זה ברור?".
    pub why_clear: Option<WhyClear>,
}

/// "למה זה ברור?" (§4.5): how unlikely a loss was, the count, the crises,
/// four amounts, and that it is the past.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhyClear {
    /// "הפסד אחרי 30 שנה הוא מאוד לא סביר: בעבר זה לא קרה אף פעם."
    pub lead: Text,
    /// "בדקנו כל תקופה של 30 שנה מאז 1926, 810 תקופות, …".
    pub count: Text,
    /// "התקופות האלה עברו דרך המשבר של 1929, …".
    pub crises: Text,
    /// What's deposited, the safe way, stocks in the worst period, stocks at
    /// the yearly return.
    pub amounts: Vec<Figure>,
    /// "זה מה שקרה בעבר, לא הבטחה. … הנתונים של מדד S&P 500, אחרי עמלות
    /// ומס, בכסף של היום."
    pub past: Text,
}

/// A labelled amount.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Figure {
    pub label: Text,
    pub value: Amount,
    pub amount: Text,
}

/// The answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Answer {
    pub confidence: Confidence,
    pub title: Text,
    pub body: Text,
    pub lines: Vec<Line>,
    /// "התוכנית שלכם", once every decision is made: one sentence, "₪500
    /// בחודש לקרן סל על מדד מניות רחב, בחשבון חדש באינטראקטיב" (§4.5).
    pub plan: Option<Text>,
    /// The plan's amount, under it (§4.5).
    pub plan_amount: Option<PlanAmount>,
    /// "איך הגעתם לזה" (§4.5): a step for each line and decision, in the
    /// answer's order, and "איזה מדד" for stocks. The page shows it with
    /// the plan; while a decision is open it holds that decision as an
    /// open step, for a visitor who changed a step and so opened another.
    pub path: Vec<PathStep>,
    pub open_decisions: u32,
    pub outcome: Outcome,
    /// The row "how to do it" shows, from the choices.
    pub focus: Option<RowId>,
    /// One for each row.
    pub how: Vec<How>,
    /// Only where fees or tax decide something on the page.
    pub gain: Option<Gain>,
    /// Whether "what if the date moves?" has rows for this answer, worked out
    /// or not: money that may be needed any day has no date to move (the
    /// review's N10), so the page holds no place for the section.
    pub moves: bool,
    pub horizons: Vec<HorizonRow>,
    /// For retirement: the monthly pension it could buy.
    pub annuity: Option<Text>,
}

// ─────────────────────────── Where the money goes ───────────────────────────

/// Why the place type is stated as safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeBecause {
    AnyDay,
    /// A year or less.
    Short,
    /// 13 to 36 months.
    KnownDate,
    /// 37 to 59 months: even a date that can move is too near.
    Rules,
    /// The visitor (or the workbench) chose it.
    Chosen,
    /// The cushion, still to be put aside, takes all the time there is:
    /// every shekel goes to it, in a money market fund.
    Cushion,
}

/// What decides the place type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place_ {
    Safe(SafeBecause),
    Index {
        chosen: bool,
    },
    /// The visitor's call: whether the date can move (or, with no date, the
    /// money can wait five years).
    Asked {
        no_date: bool,
    },
    /// A portfolio already at a bank: the question is what it costs where
    /// it is, not where to put it (principles 9 and 17).
    Existing,
}

fn place_of(answers: &Answers, overrides: Overrides) -> Place_ {
    if matches!(answers.situation, Some(Situation::ExistingPortfolio { .. })) {
        return Place_::Existing;
    }
    let natural = route::natural_route(answers);
    let forced = route::route_for(answers, overrides);
    if let Route::Forced(arena) = forced {
        return match arena {
            Arena::Safe => Place_::Safe(SafeBecause::Chosen),
            Arena::Index => Place_::Index { chosen: true },
        };
    }
    if answers.needs_any_day {
        return Place_::Safe(SafeBecause::AnyDay);
    }
    match answers.horizon {
        Horizon::Unknown if answers.monthly.is_zero() => Place_::Asked { no_date: true },
        Horizon::Retirement { .. } | Horizon::Unknown => Place_::Index { chosen: false },
        Horizon::Months(months) if months <= super::SAFE_ONLY_UP_TO_MONTHS => {
            Place_::Safe(SafeBecause::Short)
        }
        Horizon::Months(months) if months <= super::SHORT_GOAL_UP_TO_MONTHS => {
            Place_::Safe(SafeBecause::KnownDate)
        }
        Horizon::Months(months) if months < super::INDEX_IF_FLEXIBLE_FROM_MONTHS => {
            Place_::Safe(SafeBecause::Rules)
        }
        Horizon::Months(months) if months <= super::FLEXIBILITY_ASKED_UP_TO_MONTHS => {
            Place_::Asked { no_date: false }
        }
        Horizon::Months(_) => match natural.arena() {
            Arena::Index => Place_::Index { chosen: false },
            Arena::Safe => Place_::Safe(SafeBecause::Rules),
        },
    }
}

impl Place_ {
    fn arena(self, flexibility: Option<Flexibility>) -> Option<Arena> {
        match self {
            Place_::Safe(_) => Some(Arena::Safe),
            Place_::Index { .. } | Place_::Existing => Some(Arena::Index),
            Place_::Asked { .. } => flexibility.map(|flexibility| match flexibility {
                Flexibility::Fixed => Arena::Safe,
                Flexibility::CanWait => Arena::Index,
            }),
        }
    }
}

/// The answers as each arena's engine is asked them.
fn arena_answers(answers: &Answers, place: Place_) -> (Answers, Answers) {
    let mut safe = answers.clone();
    let mut index = answers.clone();
    // The cushion is a line of its own here: the index is worked out as if
    // the money went to it from the start.
    index.cushion = Some(Cushion::Has);
    match place {
        Place_::Asked { no_date: true } if answers.monthly.is_zero() => {
            let months = NO_DATE_SUM_YEARS * 12;
            safe.horizon = Horizon::Months(months);
            safe.needs_any_day = true;
            index.horizon = Horizon::Months(months);
            index.flexibility = Some(Flexibility::CanWait);
        }
        Place_::Asked { .. } => {
            safe.flexibility = Some(Flexibility::Fixed);
            index.flexibility = Some(Flexibility::CanWait);
        }
        Place_::Safe(_) | Place_::Index { .. } | Place_::Existing => {}
    }
    (safe, index)
}

/// The months money that may be needed any day is shown over, when the
/// visitor gave no date: a year.
const ANY_DAY_MONTHS: u32 = 12;

/// The safe kinds compared for the answers: the rules' pool, and for a date
/// 3 to 7 years away the money market fund and the linked bond beside the
/// kinds whose sum is known (the owner's decision 11).
fn safe_pool(safe_answers: &Answers) -> Vec<Kind> {
    let route = route::natural_route(safe_answers);
    let mut pool = route::safe_pool(route, safe_answers);
    if matches!(route, Route::FixedDate | Route::FlexibleShort) {
        for kind in [Kind::MoneyMarketFund, Kind::GovernmentBond { linked: true }] {
            if !pool.contains(&kind) {
                pool.push(kind);
            }
        }
    }
    pool
}

// ─────────────────────────── The safe side ───────────────────────────

/// A safe way: its best kind, and the other kind of the same way (a bond
/// linked to the CPI or not).
#[derive(Debug, Clone)]
struct Group {
    way: SafeWay,
    lead: Priced,
    variant: Option<Priced>,
}

fn safe_way_of(kind: Kind) -> SafeWay {
    match kind {
        Kind::GovernmentBond { .. } | Kind::TreasuryBill => SafeWay::Bond,
        Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit => SafeWay::Deposit,
        _ => SafeWay::MoneyMarketFund,
    }
}

fn groups_of(priced: Vec<Priced>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for priced in priced {
        let way = safe_way_of(priced.kind);
        match groups.iter_mut().find(|group| group.way == way) {
            Some(group) if priced.left.today > group.lead.left.today => {
                group.variant = Some(std::mem::replace(&mut group.lead, priced));
            }
            Some(group) => group.variant = Some(priced),
            None => groups.push(Group {
                way,
                lead: priced,
                variant: None,
            }),
        }
    }
    groups.sort_by_key(|group| std::cmp::Reverse(group.lead.left.today));
    groups
}

struct SafeSide {
    months: u32,
    groups: Vec<Group>,
}

impl SafeSide {
    fn group(&self, way: SafeWay) -> Option<&Group> {
        self.groups.iter().find(|group| group.way == way)
    }

    /// What a group's best kind leaves in a version. The rate moves only the
    /// fund; the index's return and the funds' fees move no safe kind.
    fn value(&self, group: &Group, answers: &Answers, version: &Version, data: &Data) -> Decimal {
        let affected = match version.varies {
            Varies::Rate => group.way == SafeWay::MoneyMarketFund,
            Varies::Inflation => true,
            Varies::Nothing | Varies::IndexReturn | Varies::History | Varies::FundFee => false,
        };
        if !affected {
            return group.lead.left.today;
        }
        std::iter::once(&group.lead)
            .chain(&group.variant)
            .filter_map(|priced| safe_value(priced.kind, answers, self.months, version, data))
            .max()
            .unwrap_or(group.lead.left.today)
    }
}

// ─────────────────────────── The cushion ───────────────────────────

/// The cushion of the two-part plan (Appendix A): the deposits of its
/// months, in a money market fund, kept there to the end; the index takes
/// the monthly money after it.
#[derive(Debug, Clone, Copy)]
struct CushionPart {
    target: Decimal,
    /// The months its deposits take.
    fill: u32,
    /// What it leaves at the end, in the fund.
    left: Amount,
    /// Its deposits, the tax and the fund's fees, in today's money.
    deposited: Decimal,
    tax: Decimal,
    fees: Decimal,
}

/// The cushion's deposits for `fill` months in the best money market fund,
/// then held to the end of `months`: one path, worked out by the short
/// term's engine in two steps (the tax of the first is paid at its end, a
/// little sooner than it would be, so it never flatters the plan).
fn cushion_part(
    answers: &Answers,
    target: Decimal,
    fill: u32,
    months: u32,
    version: &Version,
    data: &Data,
) -> Option<CushionPart> {
    let fill = fill.min(months);
    let fund = |first: Decimal, monthly: Decimal, months: u32| {
        super::safe::price(
            Kind::MoneyMarketFund,
            data,
            &crate::short_term::Scenario {
                first_deposit: first,
                monthly_deposit: monthly,
                months,
                rate: version.rate,
                inflation: version.inflation,
            },
            Decimal::ONE,
        )
    };
    let filled = fund(answers.now, answers.monthly, fill)?;
    let at_fill = super::safe::price_level(version.inflation, fill);
    let at_end = super::safe::price_level(version.inflation, months);
    let (end, tax, fees) = if fill < months {
        let held = fund(filled.outcome.after_tax, Decimal::ZERO, months - fill)?;
        (
            held.outcome.after_tax,
            filled.outcome.tax / at_fill + held.outcome.tax / at_end,
            filled.outcome.kept / at_fill + held.outcome.kept / at_end,
        )
    } else {
        (
            filled.outcome.after_tax,
            filled.outcome.tax / at_end,
            filled.outcome.kept / at_end,
        )
    };
    Some(CushionPart {
        target,
        fill,
        left: super::safe::amount(end, at_end),
        deposited: super::safe::deposited_in_todays_money(
            answers.now,
            answers.monthly,
            fill,
            version.inflation,
        ),
        tax,
        fees,
    })
}

// ─────────────────────────── The index side ───────────────────────────

/// How a way's ends in the past's stretches stand, on the basis of the
/// figures beside them: after fees and tax, in today's money, for the
/// visitor's own deposits (principle 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Losses {
    /// The stretches that would have left less than was deposited.
    below_deposited: u32,
    /// ... and less than the best safe way, where one is compared.
    below_safe: Option<u32>,
}

struct IndexSide<'a> {
    run: Run<'a>,
    /// The way the rules chose: the first that anyone can open.
    lead: usize,
    /// A study fund that leaves more than the lead by more than the margin.
    study: Option<usize>,
    /// The next way anyone can open, for how clear the lead is.
    next: Option<usize>,
    /// The ways drawn beside the lead for retirement (Appendix A): the
    /// brokerage account, the study fund, the policy.
    beside: Vec<usize>,
    months: u32,
    level: Decimal,
    history: Option<DepositsBand>,
    /// The months before the monthly money reaches the index.
    monthly_from: u32,
    /// The cushion the plan fills first, part of every figure.
    cushion: Option<CushionPart>,
    /// Each way at the history's worst, median and best, in today's money.
    market: Vec<Option<(Decimal, Decimal, Decimal)>>,
    /// Each way's past, on the figures' basis, where it's drawn.
    losses: Vec<Option<Losses>>,
    unlock: Option<super::UnlockLine>,
    also: Option<super::AlsoPossible>,
}

impl IndexSide<'_> {
    fn way(&self, at: usize) -> &WayOutcome {
        &self.run.ways[at]
    }

    /// What the cushion adds to every figure, in today's money.
    fn offset(&self) -> Decimal {
        self.cushion
            .map_or(Decimal::ZERO, |cushion| cushion.left.today)
    }

    fn left(&self, at: usize) -> Option<Amount> {
        self.way(at).after_tax().map(|nominal| {
            let way = super::safe::amount(nominal, self.level);
            Amount {
                today: way.today + self.offset(),
                nominal: way.nominal + self.offset() * self.level,
            }
        })
    }

    /// A way's value in a version, the cushion included.
    fn value(&self, at: usize, version: &Version, rates: &crate::ExchangeRates) -> Option<Decimal> {
        way_value(&self.run, self.way(at), version, rates).map(|left| left + self.offset())
    }

    /// The places of a way, the cushion added to what each leaves.
    fn places(&self, at: usize, bank: Option<Bank>, existing: bool) -> Vec<Place> {
        let offset = self.offset();
        way_places(&self.run, self.way(at), bank, existing, self.level)
            .into_iter()
            .map(|mut place| {
                place.left.today += offset;
                place.left.nominal += offset * self.level;
                place
            })
            .collect()
    }

    /// The gain's split of one plan's outcome, the cushion's part added.
    fn split(&self, outcome: &crate::simulation::Outcome, deposited: Decimal) -> Split {
        match self.cushion {
            None => split(outcome, deposited),
            Some(cushion) => {
                let way = split(outcome, deposited - cushion.deposited);
                Split {
                    kept: way.kept + cushion.left.today - cushion.deposited,
                    tax: way.tax + cushion.tax,
                    fees: way.fees + cushion.fees,
                    product: way.product,
                }
            }
        }
    }

    /// The ways the chart draws: the lead, a study fund, those beside.
    fn drawn(&self) -> Vec<usize> {
        let mut drawn = vec![self.lead];
        drawn.extend(self.study);
        for at in &self.beside {
            if !drawn.contains(at) {
                drawn.push(*at);
            }
        }
        drawn
    }
}

/// What the answer asks of the index's side.
#[derive(Debug, Clone, Copy)]
struct IndexAsk {
    deposited: Decimal,
    parts: Parts,
    run: RunAsked,
    cushion: Option<CushionPart>,
    existing: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "the run, the ways drawn and their past, in order"
)]
fn index_side<'a>(
    index_answers: &Answers,
    settings: &Settings,
    data: &'a Data,
    ask: IndexAsk,
) -> Option<IndexSide<'a>> {
    let overrides = Overrides {
        arena: Some(Arena::Index),
        kind: settings
            .overrides
            .kind
            .filter(|kind| kind.arena() == Some(Arena::Index)),
    };
    let decided = route::decide_asking(index_answers, settings, overrides, data, ask.run);
    let (run, mut lead) = decided.run?;
    // A portfolio is moved as it is: the place is compared on what it
    // holds, an index fund in Tel Aviv, not on another product.
    if ask.existing
        && let Some(at) = run.ways.iter().position(|way| {
            way.way.vehicle == Vehicle::Brokerage
                && way.way.product == crate::Product::IsraeliIndexFund
                && way.best().is_some()
        })
    {
        lead = at;
    }
    let level = decided.level;
    let months = decided.months;
    let margin = kind_margin(ask.deposited);
    let lead_left = run.ways[lead].after_tax()? / level;
    let study = if ask.existing {
        None
    } else {
        run.ways.iter().position(|way| {
            way.way.vehicle == Vehicle::StudyFund
                && way
                    .after_tax()
                    .is_some_and(|left| left / level - lead_left > margin)
        })
    };
    let next = run
        .ways
        .iter()
        .enumerate()
        .position(|(at, way)| at != lead && open_to_anyone(way.way) && way.best().is_some());
    // Retirement teaches that the way out decides the tax: the other
    // accounts are drawn beside the provident fund.
    let beside: Vec<usize> = if matches!(index_answers.horizon, Horizon::Retirement { .. }) {
        [
            Vehicle::Brokerage,
            Vehicle::StudyFund,
            Vehicle::SavingsPolicy,
        ]
        .into_iter()
        .filter_map(|vehicle| {
            run.ways
                .iter()
                .position(|way| way.way.vehicle == vehicle && way.best().is_some())
        })
        .filter(|at| *at != lead && Some(*at) != study)
        .collect()
    } else {
        Vec::new()
    };
    let history = history::deposits_band_from(
        index_answers.now,
        index_answers.monthly,
        ask.run.monthly_from,
        months,
    );
    let base = robust::versions(settings, None)[0];
    let offset = ask
        .cushion
        .map_or(Decimal::ZERO, |cushion| cushion.left.today);
    let mut drawn = vec![lead];
    drawn.extend(study);
    drawn.extend(&beside);
    let market = run
        .ways
        .iter()
        .enumerate()
        .map(|(at, way)| {
            if !drawn.contains(&at) {
                return None;
            }
            let history = history.as_ref()?;
            let at = |rate| {
                way_value(
                    &run,
                    way,
                    &Version {
                        varies: Varies::History,
                        index_return: rate,
                        ..base
                    },
                    &settings.rates,
                )
                .map(|left| left + offset)
            };
            Some((at(history.worst)?, at(history.median)?, at(history.best)?))
        })
        .collect();
    // Only where to buy says them, which a horizon row doesn't show; the
    // unlock line tries bigger first deposits, most of a row's time.
    let (unlock, also) = if ask.parts == Parts::ForAHorizonRow {
        (None, None)
    } else {
        let with_cushion = |amount: Amount| Amount {
            today: amount.today + offset,
            nominal: amount.nominal + offset * level,
        };
        (
            run.unlock(&run.ways[lead], level, &settings.rates)
                .map(|mut unlock| {
                    unlock.after_tax = with_cushion(unlock.after_tax);
                    unlock.instead_of = with_cushion(unlock.instead_of);
                    unlock
                }),
            run.also_possible(&run.ways[lead], level, &settings.rates)
                .map(|mut also| {
                    also.left = with_cushion(also.left);
                    also
                }),
        )
    };
    let ways = run.ways.len();
    Some(IndexSide {
        run,
        lead,
        study,
        next,
        beside,
        months,
        level,
        history,
        monthly_from: ask.run.monthly_from,
        cushion: ask.cushion,
        market,
        losses: vec![None; ways],
        unlock,
        also,
    })
}

/// How many of the past's stretches leave a way below `target` (today's
/// money), on the figures' basis: the end after fees and tax rises with the
/// stretch's rate, so the rate at which the way ends at `target` splits
/// them. Found by halving between the worst and the best stretch.
fn stretches_below(
    index: &IndexSide<'_>,
    index_answers: &Answers,
    at: usize,
    target: Decimal,
    settings: &Settings,
) -> Option<u32> {
    let history = index.history?;
    let (worst, _, best) = index.market.get(at).copied().flatten()?;
    if target <= worst {
        return Some(0);
    }
    if target > best {
        return Some(history.windows);
    }
    let base = robust::versions(settings, None)[0];
    let (mut low, mut high) = (history.worst.0, history.best.0);
    for _ in 0..10 {
        let middle = (low + high) / Decimal::TWO;
        let version = Version {
            varies: Varies::History,
            index_return: crate::Percent(middle),
            ..base
        };
        if index.value(at, &version, &settings.rates)? < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    history::stretches_below(
        index_answers.now,
        index_answers.monthly,
        index.monthly_from,
        index.months,
        crate::Percent((low + high) / Decimal::TWO),
    )
}

// ─────────────────────────── Building it ───────────────────────────

/// The cushion's question, when it is asked: what it takes and what each
/// answer leaves.
#[derive(Debug, Clone, Copy)]
struct CushionAsk {
    target: Decimal,
    months_to_fill: u32,
    expenses: Decimal,
    stand_in: bool,
    /// What "I have one" and "not yet" leave at the date.
    has: Option<Amount>,
    has_not: Option<Amount>,
    /// The cushion would take all the time there is.
    takes_all: bool,
}

/// Everything the lines and the parts below them are made from.
struct Facts<'a> {
    answers: &'a Answers,
    choices: Choices,
    settings: &'a Settings,
    data: &'a Data,
    place: Place_,
    arena: Option<Arena>,
    safe_answers: Answers,
    bank: Option<Bank>,
    existing: bool,
    /// The visitor gave a date: "in October 2029" is theirs to read only
    /// then (not with no date, not for money needed any day).
    dated: bool,
    /// The months of the outcome chart.
    months: u32,
    deposited: Amount,
    current_account: Amount,
    kind_margin: Decimal,
    provider_margin: Decimal,
    safe: SafeSide,
    index: Option<IndexSide<'a>>,
    cushion: Option<CushionAsk>,
    versions: Vec<Version>,
}

/// The answer to `answers`, with the visitor's `choices`.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn answer(
    answers: &Answers,
    choices: &Choices,
    settings: &Settings,
    data: &Data,
) -> Result<Answer, InvalidAnswers> {
    answer_with(answers, choices, settings, data, true)
}

/// [`answer`], with the horizon section left empty unless `horizons`: it
/// costs most of a short answer's time, so the page can show the rest first.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn answer_with(
    answers: &Answers,
    choices: &Choices,
    settings: &Settings,
    data: &Data,
    horizons: bool,
) -> Result<Answer, InvalidAnswers> {
    check(answers)?;
    let facts = facts(answers, *choices, settings, data, Parts::All { horizons });
    let mut answer = build(&facts, Parts::All { horizons });
    if !horizons {
        answer.horizons.clear();
    }
    Ok(answer)
}

/// What the cushion's other answer leaves at the date, worked out as the
/// answer to it would be, with its own best way and plan: with `fill`
/// (the target and its months) the path through the fund, without it the
/// money in the index from the start. A function of its own, and never
/// inlined, so that `facts`, under which the engines run deep, keeps a
/// small frame in a debug build.
#[inline(never)]
fn other_path(
    answers: &Answers,
    plain_index: &Answers,
    settings: &Settings,
    data: &Data,
    ask: IndexAsk,
    fill: Option<(Decimal, u32)>,
    months: u32,
) -> Option<Amount> {
    // Only the lead's figure is read: no unlock line, which tries bigger
    // first deposits, most of the time otherwise.
    let ask = IndexAsk {
        parts: Parts::ForAHorizonRow,
        ..ask
    };
    let side = match fill {
        None => index_side(plain_index, settings, data, ask)?,
        Some((target, fill)) => {
            let base = robust::versions(settings, None)[0];
            let part = cushion_part(answers, target, fill, months, &base, data)?;
            let waits = Answers {
                now: Decimal::ZERO,
                ..plain_index.clone()
            };
            let ask = IndexAsk {
                run: RunAsked {
                    monthly_from: part.fill,
                    ..ask.run
                },
                cushion: Some(part),
                ..ask
            };
            index_side(&waits, settings, data, ask)?
        }
    };
    side.left(side.lead)
}

#[allow(
    clippy::too_many_lines,
    reason = "the arena, the cushion's path and the margins, in order"
)]
fn facts<'a>(
    answers: &'a Answers,
    choices: Choices,
    settings: &'a Settings,
    data: &'a Data,
    parts: Parts,
) -> Facts<'a> {
    let existing_bank = match answers.situation {
        Some(Situation::ExistingPortfolio { bank }) => Some(bank),
        _ => None,
    };
    let existing = existing_bank.is_some();
    let bank = answers.bank.or(existing_bank.flatten());
    // The guide offers no override of its own (the owner, 2026-10-19): the
    // workbench's stays in `settings` for the workbench.
    let mut place = place_of(answers, Overrides::default());
    let (mut safe_answers, plain_index) = arena_answers(answers, place);
    let index_answers = plain_index.clone();
    let index_months = route::index_years(&index_answers) * 12;
    // Money that may be needed any day, with no date of the visitor's, is
    // shown over a year, as an example: no term the visitor never gave (and
    // makam is held for its year, REDESIGN.md §4.9).
    let safe_months = |safe_answers: &Answers| match safe_answers.horizon {
        Horizon::Months(months) => months,
        Horizon::Unknown if answers.needs_any_day => ANY_DAY_MONTHS,
        Horizon::Retirement { .. } | Horizon::Unknown => index_months,
    };
    let base = robust::versions(settings, None)[0];
    // A first guess of the deposits for the margins; the chart's are below.
    let (rough, _) = yardsticks(answers, safe_months(&safe_answers), settings.inflation);
    // The two-part plan (finding 1): with no cushion yet, the monthly money
    // goes to a money market fund until the cushion is put aside, and only
    // then to the index; the cushion stays in the fund. One path, which
    // every figure of the index follows.
    let to_fill = route::cushion_to_fill(answers).filter(|_| answers.monthly > Decimal::ZERO);
    let takes_all = to_fill.is_some_and(|(_, fill)| fill + 12 > index_months);
    let path = to_fill
        .filter(|_| !takes_all)
        .and_then(|(target, fill)| cushion_part(answers, target, fill, index_months, &base, data));
    let index_answers = if path.is_some() {
        Answers {
            now: Decimal::ZERO,
            ..index_answers
        }
    } else {
        index_answers
    };
    // The index on some deposits: the cushion's months, when it has some,
    // go to the fund first.
    let ask = |cushion: Option<CushionPart>| IndexAsk {
        deposited: rough.today,
        parts,
        run: RunAsked {
            monthly_from: cushion.map_or(0, |cushion| cushion.fill),
            every_branch: existing,
        },
        cushion,
        existing,
    };
    let mut index = index_side(&index_answers, settings, data, ask(path));
    // Where the rules put the money in the index, the data decides how sure
    // that is (finding 4): only if the index's worst stretch of the past,
    // after fees and tax, beats the safe ways' best version (the rate half
    // a point higher) by more than the margin is it stated; otherwise
    // safety or growth is the visitor's call.
    if place == (Place_::Index { chosen: false })
        && let Some(index) = &index
        && let Some((worst, _, _)) = index.market[index.lead]
    {
        let asked = Place_::Asked {
            no_date: answers.horizon == Horizon::Unknown,
        };
        let (safe_try, _) = arena_answers(answers, asked);
        let high = Version {
            varies: Varies::Rate,
            rate: crate::Percent(base.rate.0 + robust::RATE_STEP),
            ..base
        };
        let best_safe = safe_pool(&safe_try)
            .into_iter()
            .filter_map(|kind| safe_value(kind, &safe_try, index.months, &high, data))
            .max();
        let (deposited, _) = yardsticks(answers, index.months, settings.inflation);
        if best_safe.is_some_and(|best| worst - best <= kind_margin(deposited.today)) {
            place = asked;
            safe_answers = safe_try;
        }
    }
    let mut arena = place.arena(answers.flexibility);
    if index.is_none() && arena == Some(Arena::Index) {
        arena = Some(Arena::Safe);
    }
    // The cushion is asked when the money goes to the index and comes in
    // every month (the owner's decision 4), and isn't there yet.
    let expenses = answers
        .cushion
        .and_then(Cushion::expenses)
        .unwrap_or((STAND_IN_MONTHLY_EXPENSES, true));
    let target = expenses.0 * Decimal::from(CUSHION_MONTHS);
    let missing = (target - answers.now).max(Decimal::ZERO);
    let cushion_asked = arena == Some(Arena::Index)
        && answers.monthly > Decimal::ZERO
        && !answers.needs_any_day
        && !existing
        && !missing.is_zero();
    let months_to_fill = (missing.checked_div(answers.monthly))
        .and_then(|months| months.ceil().to_u32())
        .unwrap_or(u32::MAX);
    let months = index.as_ref().map_or(index_months, |index| index.months);
    let all = months_to_fill.saturating_add(12) > months;
    // Each option of the cushion leaves what choosing it shows (principle
    // 4): the other path is worked out as the answer to it would be. A
    // horizon row reads no option, so it skips that.
    let other = if !cushion_asked || parts == Parts::ForAHorizonRow || (all && path.is_none()) {
        None
    } else {
        let fill = (path.is_none()).then_some((target, months_to_fill));
        other_path(
            answers,
            &plain_index,
            settings,
            data,
            ask(None),
            fill,
            months,
        )
    };
    let cushion = cushion_asked.then(|| {
        let index = index.as_ref();
        let level = super::safe::price_level(settings.inflation, months);
        let amount = |today: Decimal| Amount {
            today,
            nominal: today * level,
        };
        let fund_alone =
            || safe_value(Kind::MoneyMarketFund, answers, months, &base, data).map(amount);
        let (has, has_not) = match index {
            Some(index) if index.cushion.is_some() => (other, index.left(index.lead)),
            Some(index) => (
                index.left(index.lead),
                if all { fund_alone() } else { other },
            ),
            None => (None, fund_alone()),
        };
        CushionAsk {
            target,
            months_to_fill,
            expenses: expenses.0,
            stand_in: expenses.1,
            has,
            has_not,
            takes_all: all,
        }
    });
    // "Not yet", where the cushion takes all the time there is: every
    // shekel goes to it, in a money market fund.
    if takes_all
        && arena == Some(Arena::Index)
        && matches!(answers.cushion, Some(Cushion::HasNot { .. }))
    {
        arena = Some(Arena::Safe);
        safe_answers.needs_any_day = true;
        if let Place_::Index { .. } = place {
            place = Place_::Safe(SafeBecause::Cushion);
        }
    }
    let safe_months = safe_months(&safe_answers);
    let months = match (arena, &index) {
        (Some(Arena::Index), Some(index)) => index.months,
        _ => safe_months,
    };
    let (deposited, current_account) = yardsticks(&safe_answers, months, settings.inflation);
    let history = index.as_ref().and_then(|index| index.history);
    let versions = robust::versions(settings, history.as_ref());
    // The cushion that takes all the time there is goes to a money market
    // fund, as its words say.
    let pool = if place == Place_::Safe(SafeBecause::Cushion) {
        vec![Kind::MoneyMarketFund]
    } else {
        safe_pool(&safe_answers)
    };
    let priced: Vec<Priced> = pool
        .into_iter()
        .filter_map(|kind| safe_priced(kind, &safe_answers, months, &base, data))
        .collect();
    let mut safe = SafeSide {
        months,
        groups: groups_of(priced),
    };
    if safe.groups.is_empty()
        && let Some(fund) = safe_priced(Kind::MoneyMarketFund, &safe_answers, months, &base, data)
    {
        safe.groups = groups_of(vec![fund]);
    }
    // The past, on the basis of the figures beside it (finding 3): how many
    // stretches would have left less than was deposited, and the lead less
    // than the best safe way.
    if parts != Parts::ForAHorizonRow
        && let Some(index) = &mut index
    {
        let (put_in, _) = yardsticks(answers, index.months, settings.inflation);
        let best_safe = safe.groups.first().map(|group| group.lead.left.today);
        for at in index.drawn() {
            let below_deposited =
                stretches_below(index, &index_answers, at, put_in.today, settings);
            let below_safe = (at == index.lead)
                .then(|| {
                    best_safe
                        .and_then(|safe| stretches_below(index, &index_answers, at, safe, settings))
                })
                .flatten();
            index.losses[at] = below_deposited.map(|below_deposited| Losses {
                below_deposited,
                below_safe,
            });
        }
    }
    Facts {
        answers,
        choices,
        settings,
        data,
        place,
        arena,
        safe_answers,
        bank,
        existing,
        dated: !matches!(answers.horizon, Horizon::Unknown) && !answers.needs_any_day,
        months,
        deposited,
        current_account,
        kind_margin: kind_margin(deposited.today),
        provider_margin: provider_margin(deposited.today),
        safe,
        index,
        cushion,
        versions,
    }
}

/// How the first safe way stands against the second, in the versions that
/// move them.
fn safe_standing(f: &Facts<'_>) -> Standing {
    let [first, second, ..] = f.safe.groups.as_slice() else {
        return Standing::Clear;
    };
    let pairs: Vec<(Decimal, Decimal)> = f
        .versions
        .iter()
        .filter(|version| {
            matches!(
                version.varies,
                Varies::Nothing | Varies::Rate | Varies::Inflation
            )
        })
        .map(|version| {
            (
                f.safe.value(first, &f.safe_answers, version, f.data),
                f.safe.value(second, &f.safe_answers, version, f.data),
            )
        })
        .collect();
    standing(&pairs, f.kind_margin)
}

/// How the index's lead stands against the next way anyone can open.
fn way_standing(f: &Facts<'_>, index: &IndexSide<'_>) -> Standing {
    let Some(next) = index.next else {
        return Standing::Clear;
    };
    let pairs: Vec<(Decimal, Decimal)> = f
        .versions
        .iter()
        .filter_map(|version| {
            Some((
                way_value(
                    &index.run,
                    index.way(index.lead),
                    version,
                    &f.settings.rates,
                )?,
                way_value(&index.run, index.way(next), version, &f.settings.rates)?,
            ))
        })
        .collect();
    standing(&pairs, f.kind_margin)
}

/// What the visitor's choices make of the way: the safe way, or the index's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Chosen {
    Safe(SafeWay),
    /// A way of the index, by its place in the run.
    Index(usize),
}

/// The bank places of a product and the reference from the visitor's bank.
struct Where {
    places: Vec<Place>,
    ties: Vec<u8>,
    yours: Option<usize>,
    range: Option<(Amount, Amount)>,
    /// The visitor's bank is in the stated answer beside the tie group: it
    /// needs no new account, and is behind by no more than the margin that
    /// would make the account a question (the review's N2).
    yours_joins: bool,
}

impl Where {
    fn new(places: Vec<Place>, margin: Decimal) -> Self {
        let ties = tie_groups(&places, margin);
        let yours = places.iter().position(|place| place.yours);
        let range = bank_range(&places);
        Where {
            places,
            ties,
            yours,
            range,
            yours_joins: false,
        }
    }

    /// Lets the visitor's bank join the stated answer when it is behind the
    /// best by no more than `margin`, the margin between kinds: staying needs
    /// no new account, so within it moving is no question, and the answer
    /// mustn't send the visitor to open one (the review's N2).
    fn join_yours(mut self, margin: Decimal) -> Self {
        self.yours_joins = match (self.yours, self.best()) {
            (Some(at), Some(best)) => {
                self.ties[at] != 0 && best.left.today - self.places[at].left.today <= margin
            }
            _ => false,
        };
        self
    }

    /// The names the line and the plan give: the visitor's bank first when
    /// it joins, then the tie group.
    fn line_names(&self) -> Vec<Text> {
        // The visitor's bank is named as theirs, first, when it joins and
        // also when it is in the tie itself (principle 15): "לאומי, …,
        // הפועלים" read as if their bank were one more.
        let at = self
            .yours
            .filter(|at| self.yours_joins || self.ties[*at] == 0);
        let yours = at.map(|at| &self.places[at].provider);
        yours
            .map(card::your_bank)
            .into_iter()
            .chain(
                self.tied_names()
                    .into_iter()
                    .filter(|name| Some(name) != yours),
            )
            .collect()
    }

    /// How far apart the names the line gives are.
    fn line_spread(&self) -> Decimal {
        match self.yours.filter(|_| self.yours_joins) {
            Some(at) => {
                let high = self.tied().iter().map(|place| place.left.today).max();
                high.map_or(Decimal::ZERO, |high| high - self.places[at].left.today)
            }
            None => self.spread(),
        }
    }

    fn best(&self) -> Option<&Place> {
        self.places.first()
    }

    /// The best's tie group: every place within the margin of it, and no
    /// other (finding 10).
    fn tied(&self) -> Vec<&Place> {
        self.places
            .iter()
            .zip(&self.ties)
            .filter(|(_, tie)| **tie == 0)
            .map(|(place, _)| place)
            .collect()
    }

    /// The tie group's names, as a sentence names them.
    fn tied_names(&self) -> Vec<Text> {
        self.tied()
            .iter()
            .map(|place| place.provider.clone())
            .collect()
    }

    /// How far apart the tie group is: its highest less its lowest.
    fn spread(&self) -> Decimal {
        let tied = self.tied();
        let high = tied.iter().map(|place| place.left.today).max();
        let low = tied.iter().map(|place| place.left.today).min();
        high.zip(low)
            .map_or(Decimal::ZERO, |(high, low)| high - low)
    }

    /// The visitor's bank's place, once they named it.
    fn yours_name(&self) -> Option<Text> {
        self.yours.map(|at| self.places[at].provider.clone())
    }

    /// What the visitor's bank leaves: theirs, or the best bank's while it's
    /// not known (so the gap is never overstated).
    fn reference(&self) -> Option<Amount> {
        match self.yours {
            Some(at) => Some(self.places[at].left),
            None => self.range.map(|(_, high)| high),
        }
    }
}

/// How much of the answer to build: all of it, or only what a horizon row
/// reads (the lines, the chart's rows, the focus), which skips how to do it
/// and the gain: most of a row's time otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Parts {
    All { horizons: bool },
    ForAHorizonRow,
}

#[allow(
    clippy::too_many_lines,
    reason = "the lines, the chart, the plan and the path, in the card's order"
)]
fn build(f: &Facts<'_>, parts: Parts) -> Answer {
    let details = parts != Parts::ForAHorizonRow;
    let mut lines: Vec<Line> = Vec::new();
    let safe_standing = safe_standing(f);
    // A decision appears after the one it depends on (finding 7): asked
    // only once the money can wait, the cushion comes right after the
    // place type, and keeps its label: it is what's done first in time.
    let cushion = cushion_line(f);
    let after_place = matches!(f.place, Place_::Asked { .. });
    let cushion = if after_place {
        cushion
    } else {
        lines.extend(cushion);
        None
    };
    lines.push(place_line(f));
    lines.extend(cushion);
    let (way_line, chosen) = way_line(f, safe_standing);
    lines.push(way_line);
    let (where_line, where_info) = where_line(f, chosen);
    lines.push(where_line);
    let state = match (f.place, f.arena) {
        (Place_::Existing, _) => existing_state(f),
        (Place_::Asked { .. }, _) => ConfidenceState::C,
        // Above the lines the data already chose: the worst past stretch
        // still beats the safe ways' best version by more than the margin.
        (Place_::Index { .. }, _) => ConfidenceState::A,
        (_, Some(Arena::Safe)) if safe_standing == Standing::Clear => ConfidenceState::A,
        _ => ConfidenceState::B,
    };
    let open_decisions = u32::try_from(
        lines
            .iter()
            .filter(|line| matches!(&line.state, LineState::Decision(decision) if decision.chosen.is_none()))
            .count(),
    )
    .unwrap_or(u32::MAX);
    let waiting = lines.iter().any(|line| line.state == LineState::Waiting);
    let rows = rows(f, chosen);
    let focus = chosen.and_then(|chosen| focus_row(f, chosen));
    let how: Vec<How> = rows
        .iter()
        .filter(|_| details)
        .map(|row| {
            how_of(
                f,
                row,
                where_info.as_ref().filter(|info| Some(row.id) == info.row),
            )
        })
        .collect();
    let (axis, rows) = axis(rows, f.deposited, f.current_account);
    let stocks = rows.iter().any(|row| matches!(row.kind, Kind::Way(_)));
    let outcome = Outcome {
        date: card::date(f.settings.today, f.months, f.dated),
        title: card::outcome_title(f.settings.today, f.months, f.dated),
        caption: row_words::caption(stocks),
        deposited: f.deposited,
        current_account: f.current_account,
        lines: vec![
            RefLine {
                which: RefLineKind::Deposited,
                label: row_words::deposited_line(),
                value: f.deposited,
                amount: Text::owned(
                    card::about(f.deposited.today),
                    card::about(f.deposited.today),
                ),
            },
            RefLine {
                which: RefLineKind::CurrentAccount,
                label: row_words::current_account_line(),
                value: f.current_account,
                amount: Text::owned(
                    card::about(f.current_account.today),
                    card::about(f.current_account.today),
                ),
            },
        ],
        axis,
        rows,
    };
    let plan = (open_decisions == 0 && !waiting)
        .then(|| plan_text(f, chosen, where_info.as_ref()))
        .flatten();
    let plan_amount = plan
        .as_ref()
        .and_then(|_| plan_amount(f, chosen, where_info.as_ref()));
    let path = if details {
        path(f, &lines, chosen, where_info.as_ref())
    } else {
        Vec::new()
    };
    let (title, body) = title_and_body(f, state, open_decisions > 0 || waiting);
    let gain = details
        .then(|| gain(f, &lines, chosen, where_info.as_ref()))
        .flatten();
    let annuity = details.then(|| annuity(f, chosen, &lines)).flatten();
    Answer {
        confidence: Confidence {
            state,
            note: card::chip_note(state, f.arena, open_decisions > 0 || waiting, f.existing),
        },
        title,
        body,
        lines,
        plan,
        plan_amount,
        path,
        open_decisions,
        outcome,
        focus,
        how,
        gain,
        moves: !f.answers.needs_any_day,
        horizons: if parts == (Parts::All { horizons: true }) && !f.answers.needs_any_day {
            horizons(f)
        } else {
            Vec::new()
        },
        annuity,
    }
}

/// How sure the site is about a portfolio's place: moving clearly saves
/// (beyond the margin between kinds, against the named bank, or the cheapest
/// branch plan while none is named), or it's about the same.
fn existing_state(f: &Facts<'_>) -> ConfidenceState {
    let Some(index) = &f.index else {
        return ConfidenceState::B;
    };
    let places = Where::new(index.places(index.lead, f.bank, true), f.provider_margin);
    let gap = places
        .best()
        .zip(places.reference())
        .map(|(best, reference)| best.left.today - reference.today);
    if gap.is_some_and(|gap| gap > f.kind_margin) {
        ConfidenceState::A
    } else {
        ConfidenceState::B
    }
}

// ─────────────────────────── The lines ───────────────────────────

fn line(kind: LineKind, state: LineState) -> Line {
    Line {
        kind,
        label: card::label(kind),
        state,
    }
}

/// The cushion's question: asked when the money goes to the index (as the
/// answer, or as the option chosen) and money comes in every month (the
/// owner's decision 4). Each option shows what it leaves at the date: the
/// two-part plan is one path (finding 1).
fn cushion_line(f: &Facts<'_>) -> Option<Line> {
    let ask = f.cushion?;
    let lead = f.index.as_ref().map(|index| index.way(index.lead).way);
    let words = card::cushion_decision(&card::CushionFigures {
        monthly: f.answers.monthly,
        target: ask.target,
        months_to_fill: ask.months_to_fill,
        expenses: ask.expenses,
        stand_in: ask.stand_in,
        has: ask.has.map(|amount| amount.today),
        has_not: ask.has_not.map(|amount| amount.today),
        takes_all: ask.takes_all,
        index_return: f.settings.index_return,
    });
    let chosen = f.answers.cushion.map(|cushion| {
        Pick::Cushion(match cushion {
            Cushion::Has => CushionPick::Has,
            Cushion::HasNot { .. } => CushionPick::HasNot,
        })
    });
    let index_row: Vec<RowId> = lead.map(Kind::Way).into_iter().collect();
    let picks = [
        (Pick::Cushion(CushionPick::Has), index_row.clone(), ask.has),
        (
            Pick::Cushion(CushionPick::HasNot),
            if ask.takes_all {
                vec![Kind::MoneyMarketFund]
            } else {
                index_row
            },
            ask.has_not,
        ),
    ];
    Some(line(
        LineKind::Before,
        LineState::Decision(decision(DecisionId::Cushion, words, &picks, chosen)),
    ))
}

/// A decision from its words and, for each option, its pick, its rows and
/// its value.
fn decision(
    id: DecisionId,
    words: card::DecisionWords,
    picks: &[(Pick, Vec<RowId>, Option<Amount>)],
    chosen: Option<Pick>,
) -> Decision {
    let options = words
        .options
        .into_iter()
        .zip(picks)
        .map(|(words, (pick, rows, value))| {
            let facts: Vec<Text> = words.facts.into_iter().take(3).collect();
            let asks = words
                .asks
                .into_iter()
                .filter(|asks| facts.contains(asks))
                .collect();
            let fits = words.fits_if.as_ref().map(card::without_fits_if);
            Opt {
                pick: *pick,
                title: words.title,
                amount: words.amount,
                also: words.also,
                value: *value,
                facts,
                asks,
                fits_if: words.fits_if,
                fits,
                button: words.button,
                rows: rows.clone(),
            }
        })
        .collect();
    Decision {
        id,
        question: words.question,
        why: words.why,
        options,
        difference: words.difference,
        chosen,
    }
}

/// The past of the index's lead, on the figures' basis, as the words take it.
fn past_of(f: &Facts<'_>) -> Option<card::Past> {
    let index = f.index.as_ref()?;
    let history = index.history?;
    let losses = index.losses.get(index.lead).copied().flatten()?;
    Some(card::Past {
        below_deposited: losses.below_deposited,
        below_safe: losses.below_safe,
        windows: history.windows,
        months: history.months,
        since: history.from.year(),
    })
}

/// The kinds the safe side holds, best first.
fn safe_kinds(f: &Facts<'_>) -> Vec<Kind> {
    f.safe.groups.iter().map(|group| group.lead.kind).collect()
}

fn place_line(f: &Facts<'_>) -> Line {
    match f.place {
        Place_::Existing => line(
            LineKind::PlaceType,
            LineState::Stated {
                text: card::place_existing(),
                pill: Pill::Clear,
            },
        ),
        Place_::Safe(because) => line(
            LineKind::PlaceType,
            LineState::Stated {
                text: card::place_safe(because, f.months, past_of(f).as_ref()),
                pill: match because {
                    SafeBecause::AnyDay
                    | SafeBecause::Short
                    | SafeBecause::KnownDate
                    | SafeBecause::Cushion => Pill::Clear,
                    SafeBecause::Rules | SafeBecause::Chosen => Pill::Rules,
                },
            },
        ),
        Place_::Index { chosen } => line(
            LineKind::PlaceType,
            LineState::Stated {
                text: card::place_index(&card::PlaceIndexFigures {
                    months: f.months,
                    past: past_of(f),
                    safe: f.safe.groups.first().map(|group| group.lead.kind),
                }),
                pill: if chosen { Pill::Rules } else { Pill::Clear },
            },
        ),
        Place_::Asked { no_date } => {
            let fund = f.safe.group(SafeWay::MoneyMarketFund);
            let fund_range = fund.and_then(|fund| fund_range(f, fund));
            let leads = f.safe.groups.iter().map(|group| group.lead.left.today);
            let ends = leads.chain(
                fund_range
                    .into_iter()
                    .flat_map(|(low, high)| [low.today, high.today]),
            );
            let (safe_low, safe_high) = ends.fold((None, None), |(low, high), value| {
                (
                    Some(low.map_or(value, |low: Decimal| low.min(value))),
                    Some(high.map_or(value, |high: Decimal| high.max(value))),
                )
            });
            let safe_best = f.safe.groups.first().map(|group| group.lead.left);
            let index = f.index.as_ref();
            let growth = index.and_then(|index| index.left(index.lead));
            let worst = index.and_then(|index| index.market[index.lead].map(|(worst, _, _)| worst));
            let lead = index.map(|index| index.way(index.lead).way);
            let words = card::place_type_decision(&card::PlaceTypeFigures {
                no_date,
                today: f.settings.today,
                months: f.months,
                safe_low,
                safe_high,
                safe_best: safe_best.map(|amount| amount.today),
                growth: growth.map(|amount| amount.today),
                index_return: f.settings.index_return,
                below_deposited: worst.is_some_and(|worst| worst < f.deposited.today),
                past: past_of(f),
                lead,
                safe_kinds: safe_kinds(f),
                fund_range: fund_range.map(|(low, high)| (low.today, high.today)),
                retirement: matches!(f.answers.horizon, Horizon::Retirement { .. }),
                annuity: annuity_of(f, index.map(|index| index.lead))
                    .as_ref()
                    .map(card::annuity_also),
            });
            let safe_rows: Vec<RowId> = f
                .safe
                .groups
                .iter()
                .flat_map(|group| {
                    std::iter::once(group.lead.kind).chain(group.variant.as_ref().map(|v| v.kind))
                })
                .collect();
            let index_rows: Vec<RowId> = lead.map(Kind::Way).into_iter().collect();
            let picks = [
                (Pick::Flexibility(Flexibility::Fixed), safe_rows, safe_best),
                (Pick::Flexibility(Flexibility::CanWait), index_rows, growth),
            ];
            line(
                LineKind::PlaceType,
                LineState::Decision(decision(
                    DecisionId::PlaceType,
                    words,
                    &picks,
                    f.answers.flexibility.map(Pick::Flexibility),
                )),
            )
        }
    }
}

fn way_line(f: &Facts<'_>, safe_standing: Standing) -> (Line, Option<Chosen>) {
    if f.existing {
        return holdings_line(f);
    }
    match f.arena {
        None => (line(LineKind::Way, LineState::Waiting), None),
        Some(Arena::Safe) => safe_way_line(f, safe_standing),
        Some(Arena::Index) => index_way_line(f),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the options, the switch point and the flip, one decision"
)]
fn safe_way_line(f: &Facts<'_>, safe_standing: Standing) -> (Line, Option<Chosen>) {
    let groups = &f.safe.groups;
    let Some(first) = groups.first() else {
        return (line(LineKind::Way, LineState::Waiting), None);
    };
    if groups.len() < 2 || safe_standing == Standing::Clear {
        let text = card::safe_way_stated(
            first.lead.kind,
            (!f.answers.needs_any_day).then_some(f.months),
            groups.len() < 2,
        );
        return (
            line(
                LineKind::Way,
                LineState::Stated {
                    text,
                    pill: Pill::Clear,
                },
            ),
            Some(Chosen::Safe(first.way)),
        );
    }
    // Three options at most, grouped by what they ask of the saver.
    let shown: Vec<&Group> = groups.iter().take(3).collect();
    let base = f.versions[0];
    let fund = f.safe.group(SafeWay::MoneyMarketFund);
    let rate_range = fund.and_then(|fund| fund_range(f, fund));
    let best_other = shown
        .iter()
        .find(|group| group.way != SafeWay::MoneyMarketFund);
    // Where the fund and the best other way trade places.
    let switch = fund.zip(best_other).and_then(|(fund, other)| {
        let other_value = other.lead.left.today;
        switch_rate(|rate| {
            let version = Version { rate, ..base };
            Some(
                safe_value(fund.lead.kind, &f.safe_answers, f.months, &version, f.data)?
                    - other_value,
            )
        })
    });
    // A bond through a bank (finding 9): the visitor's, once named, or the
    // banks' range while not.
    let bond_at_bank = f.safe.group(SafeWay::Bond).and_then(|bond| {
        let places = Where::new(
            super::outcome::safe_places(&bond.lead, f.data, f.bank),
            f.provider_margin,
        );
        let bill = bond.lead.kind == Kind::TreasuryBill;
        match (places.yours, places.range) {
            (Some(at), _) => Some((
                bill,
                card::AtBank::Named(
                    places.places[at].provider.clone(),
                    places.places[at].left.today,
                ),
            )),
            (None, Some((low, high))) => Some((bill, card::AtBank::Range(low.today, high.today))),
            (None, None) => None,
        }
    });
    let options: Vec<card::SafeOption> = shown
        .iter()
        .map(|group| card::SafeOption {
            way: group.way,
            kind: group.lead.kind,
            left: group.lead.left.today,
            variant: group
                .variant
                .as_ref()
                .map(|variant| (variant.kind, variant.left.today)),
            rate_range: if group.way == SafeWay::MoneyMarketFund {
                rate_range.map(|(low, high)| (low.today, high.today))
            } else {
                None
            },
        })
        .collect();
    let words = card::safe_way_decision(&card::SafeWayFigures {
        options: &options,
        today: f.settings.today,
        months: f.months,
        dated: f.dated,
        rate: f.settings.rate,
        switch,
        bond_at_bank,
        flips: robust::order_flips(
            safe_standing,
            rate_range.map(|(low, high)| (low.today, high.today)),
            &shown
                .iter()
                .filter(|group| group.way != SafeWay::MoneyMarketFund)
                .map(|group| group.lead.left.today)
                .collect::<Vec<_>>(),
        ),
    });
    let picks: Vec<(Pick, Vec<RowId>, Option<Amount>)> = shown
        .iter()
        .map(|group| {
            let rows = std::iter::once(group.lead.kind)
                .chain(group.variant.as_ref().map(|variant| variant.kind))
                .collect();
            (Pick::SafeWay(group.way), rows, Some(group.lead.left))
        })
        .collect();
    let chosen = f
        .choices
        .safe_way
        .filter(|way| shown.iter().any(|group| group.way == *way));
    (
        line(
            LineKind::Way,
            LineState::Decision(decision(
                DecisionId::SafeWay,
                words,
                &picks,
                chosen.map(Pick::SafeWay),
            )),
        ),
        chosen.map(Chosen::Safe),
    )
}

/// The fund's figure at the rate ±0.5 point.
fn fund_range(f: &Facts<'_>, fund: &Group) -> Option<(Amount, Amount)> {
    let base = f.versions[0];
    let at = |rate: Decimal| {
        let version = Version {
            rate: crate::Percent(rate.max(Decimal::ZERO)),
            ..base
        };
        safe_value(fund.lead.kind, &f.safe_answers, f.months, &version, f.data)
            .map(|today| at_end(today, f.months, base.inflation))
    };
    Some((
        at(base.rate.0 - robust::RATE_STEP)?,
        at(base.rate.0 + robust::RATE_STEP)?,
    ))
}

fn index_way_line(f: &Facts<'_>) -> (Line, Option<Chosen>) {
    let Some(index) = &f.index else {
        return (line(LineKind::Way, LineState::Waiting), None);
    };
    let lead = index.way(index.lead).way;
    let lead_left = index.left(index.lead);
    if let Some(study) = index.study {
        let study_left = index.left(study);
        let lead_split = best_split(f, index, index.lead);
        let study_split = best_split(f, index, study);
        let words = card::wrapper_decision(&card::WrapperFigures {
            lead,
            lead_left: lead_left.map(|amount| amount.today),
            study_left: study_left.map(|amount| amount.today),
            index_return: f.settings.index_return,
            monthly: f.answers.monthly,
            mostly_tax: lead_split
                .zip(study_split)
                .is_some_and(|(lead, study)| lead.tax - study.tax > study.fees - lead.fees),
        });
        let picks = [
            (
                Pick::Wrapper(Wrapper::Anyone),
                vec![Kind::Way(lead)],
                lead_left,
            ),
            (
                Pick::Wrapper(Wrapper::StudyFund),
                vec![Kind::Way(index.way(study).way)],
                study_left,
            ),
        ];
        let chosen = f.choices.wrapper.map(|wrapper| match wrapper {
            Wrapper::Anyone => Chosen::Index(index.lead),
            Wrapper::StudyFund => Chosen::Index(study),
        });
        return (
            line(
                LineKind::Way,
                LineState::Decision(decision(
                    DecisionId::Wrapper,
                    words,
                    &picks,
                    f.choices.wrapper.map(Pick::Wrapper),
                )),
            ),
            chosen,
        );
    }
    let pill = match way_standing(f, index) {
        Standing::Clear => Pill::Clear,
        Standing::Tie => Pill::Tie,
        Standing::Flips | Standing::Behind => Pill::Rules,
    };
    let tied_with = (pill == Pill::Tie)
        .then(|| index.next.map(|next| index.way(next).way))
        .flatten();
    (
        line(
            LineKind::Way,
            LineState::Stated {
                text: card::index_way_stated(lead, tied_with),
                pill,
            },
        ),
        Some(Chosen::Index(index.lead)),
    )
}

/// The engine's split of a way's best plan, the cushion's part included.
fn best_split(f: &Facts<'_>, index: &IndexSide<'_>, at: usize) -> Option<Split> {
    let way = index.way(at);
    let best = way.best()?;
    let outcome = plan_outcome(
        &index.run.cands[best.index],
        way.way,
        &index.run.base,
        &f.versions[0],
        &f.settings.rates,
    )?;
    Some(index.split(&outcome, f.deposited.today))
}

/// What the איפה line found, for the parts below.
struct WhereInfo {
    row: Option<RowId>,
    places: Where,
    /// The account question, when it was asked or settled.
    account: Option<Account>,
    /// A decision about the account (or a move) was shown.
    decided: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm for each way, each with its places"
)]
fn where_line(f: &Facts<'_>, chosen: Option<Chosen>) -> (Line, Option<WhereInfo>) {
    if f.existing {
        return transfer_line(f);
    }
    let Some(chosen) = chosen else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    let bank_name = f.bank.map(|bank| super::outcome::bank_name(f.data, bank));
    match chosen {
        Chosen::Safe(SafeWay::MoneyMarketFund) => (
            line(
                LineKind::Where,
                LineState::Stated {
                    text: card::where_fund(bank_name.as_ref()),
                    pill: Pill::Clear,
                },
            ),
            None,
        ),
        Chosen::Safe(SafeWay::Deposit) => {
            let Some(group) = f.safe.group(SafeWay::Deposit) else {
                return (line(LineKind::Where, LineState::Waiting), None);
            };
            // A deposit at another bank needs an account there, as a move
            // to a house does: the same margin lets the visitor's bank join.
            let places = Where::new(
                super::outcome::safe_places(&group.lead, f.data, f.bank),
                f.provider_margin,
            )
            .join_yours(f.kind_margin);
            let tied = places.line_names();
            let tied = if tied.is_empty() {
                vec![group.lead.choice.provider.clone()]
            } else {
                tied
            };
            let yours = places.yours.map(|at| &places.places[at]);
            let text = card::where_deposit(&card::DepositWhere {
                tied: &tied,
                spread: places.line_spread(),
                yours: yours.map(|place| &place.provider),
                behind: yours
                    .zip(places.best())
                    .map(|(yours, best)| best.left.today - yours.left.today),
                yours_tied: places.yours_joins
                    || yours.is_some_and(|place| {
                        places.tied().iter().any(|tied| std::ptr::eq(*tied, place))
                    }),
            });
            (
                line(
                    LineKind::Where,
                    LineState::Stated {
                        text,
                        pill: if tied.len() > 1 {
                            Pill::Tie
                        } else {
                            Pill::Rules
                        },
                    },
                ),
                Some(WhereInfo {
                    row: Some(group.lead.kind),
                    places,
                    account: None,
                    decided: false,
                }),
            )
        }
        Chosen::Safe(SafeWay::Bond) => {
            let Some(group) = f.safe.group(SafeWay::Bond) else {
                return (line(LineKind::Where, LineState::Waiting), None);
            };
            let places = Where::new(
                super::outcome::safe_places(&group.lead, f.data, f.bank),
                f.provider_margin,
            );
            // Through the bank, a bond that leaves no more than the fund in
            // the same app is no reason to stay: the account isn't asked.
            let fund = f
                .safe
                .group(SafeWay::MoneyMarketFund)
                .map(|fund| fund.lead.left.today);
            let moot = places
                .reference()
                .zip(fund)
                .is_some_and(|(bank, fund)| bank.today <= fund);
            account_line(f, Some(group.lead.kind), places, moot)
        }
        Chosen::Index(at) => {
            let Some(index) = &f.index else {
                return (line(LineKind::Where, LineState::Waiting), None);
            };
            let way = index.way(at);
            let row = Some(Kind::Way(way.way));
            let places = Where::new(index.places(at, f.bank, false), f.provider_margin);
            match way.way.vehicle {
                Vehicle::Brokerage => account_line(f, row, places, false),
                vehicle => (
                    line(
                        LineKind::Where,
                        LineState::Stated {
                            text: card::where_fund_company(vehicle),
                            pill: if vehicle == Vehicle::StudyFund {
                                Pill::Clear
                            } else {
                                Pill::Rules
                            },
                        },
                    ),
                    Some(WhereInfo {
                        row,
                        places,
                        account: None,
                        decided: false,
                    }),
                ),
            }
        }
    }
}

/// "Open a new account, or buy through your bank?": a decision only when the
/// visitor's bank (or, until it's named, the best bank) is behind the best
/// by more than the margin between kinds (finding 8: the account is a
/// choice between ways of buying, as the safe ways are), and staying isn't
/// dominated anyway. Otherwise the line states it, ties among the places
/// by the providers' margin.
fn account_line(
    f: &Facts<'_>,
    row: Option<RowId>,
    places: Where,
    moot: bool,
) -> (Line, Option<WhereInfo>) {
    let Some(best) = places.best() else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    let tied = places.tied_names();
    let best_left = best.left.today;
    let reference = places.reference();
    let gap = reference.map(|reference| best.left.today - reference.today);
    let yours_name = places.yours_name();
    let range = places.range.map(|(low, high)| (low.today, high.today));
    if moot || gap.is_none_or(|gap| gap <= f.kind_margin) {
        let places = if moot {
            places
        } else {
            places.join_yours(f.kind_margin)
        };
        let tied = places.line_names();
        let text = card::where_stated(&card::WhereFigures {
            kind: row,
            tied: &tied,
            spread: places.line_spread(),
            best: best_left,
            gap,
            yours: yours_name.as_ref(),
            joined: places.yours_joins,
            range,
            margin: f.kind_margin,
            moot,
        });
        let pill = if tied.len() == 1 && (moot || gap.is_none()) {
            Pill::Clear
        } else {
            Pill::Tie
        };
        let info = WhereInfo {
            row,
            // Joined, staying is as good as moving: no new account is asked for.
            account: (!places.yours_joins).then_some(Account::NewAccount),
            places,
            decided: false,
        };
        return (
            line(LineKind::Where, LineState::Stated { text, pill }),
            Some(info),
        );
    }
    let years = f.months.div_ceil(12);
    let best_cost = best.cost_a_year;
    let yours_cost = places.yours.and_then(|at| places.places[at].cost_a_year);
    let words = card::account_decision(&card::AccountFigures {
        kind: row,
        tied: &tied,
        best: best.left.today,
        best_cost,
        yours: yours_name.as_ref(),
        yours_left: reference.map(|reference| reference.today),
        yours_cost,
        range,
        gap: gap.unwrap_or(Decimal::ZERO),
        years,
    });
    let rows: Vec<RowId> = row.into_iter().collect();
    let picks = [
        (
            Pick::Account(Account::NewAccount),
            rows.clone(),
            Some(best.left),
        ),
        (Pick::Account(Account::YourBank), rows, reference),
    ];
    let chosen = f.choices.account;
    let info = WhereInfo {
        row,
        places,
        account: chosen,
        decided: true,
    };
    (
        line(
            LineKind::Where,
            LineState::Decision(decision(
                DecisionId::Account,
                words,
                &picks,
                chosen.map(Pick::Account),
            )),
        ),
        Some(info),
    )
}

/// A portfolio's places: what it would pay elsewhere, against the named
/// bank's branch plan or every bank's while none is named.
fn existing_places(f: &Facts<'_>, index: &IndexSide<'_>) -> Where {
    Where::new(index.places(index.lead, f.bank, true), f.provider_margin)
}

/// What moving would save: against the named bank's branch plan, or from
/// the dearest bank's to the cheapest's while none is named.
fn saves(places: &Where) -> Option<(Decimal, Decimal)> {
    let best = places.best()?.left.today;
    match (places.yours, places.range) {
        (Some(at), _) => {
            let gap = best - places.places[at].left.today;
            Some((gap, gap))
        }
        (None, Some((low, high))) => Some((best - high.today, best - low.today)),
        (None, None) => None,
    }
}

/// "What's in the portfolio?" for a portfolio already at a bank (finding
/// 2): the visitor's question is what it costs, so this is the first
/// decision, on the way's line.
fn holdings_line(f: &Facts<'_>) -> (Line, Option<Chosen>) {
    let Some(index) = &f.index else {
        return (line(LineKind::Way, LineState::Waiting), None);
    };
    let places = existing_places(f, index);
    let words = card::holdings_decision(&card::HoldingsFigures {
        saves: saves(&places),
        margin: f.kind_margin,
        tied: &places.tied_names(),
        months: f.months,
        named: places.yours_name().as_ref(),
    });
    let row = Kind::Way(index.way(index.lead).way);
    let picks = [
        (
            Pick::Holdings(Holdings::IndexFunds),
            vec![row],
            places.best().map(|best| best.left),
        ),
        (Pick::Holdings(Holdings::Managed), vec![row], None),
        (Pick::Holdings(Holdings::DontKnow), vec![row], None),
    ];
    let chosen = f.choices.holdings;
    (
        line(
            LineKind::Way,
            LineState::Decision(decision(
                DecisionId::Holdings,
                words,
                &picks,
                chosen.map(Pick::Holdings),
            )),
        ),
        chosen.map(|_| Chosen::Index(index.lead)),
    )
}

/// Where, for a portfolio: after "index funds or shares", the move stated
/// (the holdings as they are, no sale, no tax) to the places against the
/// bank's branch plan; after the others, what to check.
fn transfer_line(f: &Facts<'_>) -> (Line, Option<WhereInfo>) {
    let (Some(index), Some(holdings)) = (&f.index, f.choices.holdings) else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    let places = existing_places(f, index);
    let row = Some(Kind::Way(index.way(index.lead).way));
    let saves = saves(&places);
    let worth = saves.is_some_and(|(low, _)| low > f.kind_margin);
    let tied = places.tied_names();
    let text = match holdings {
        Holdings::IndexFunds => card::where_transfer(&card::TransferFigures {
            tied: &tied,
            spread: places.spread(),
            saves,
            margin: f.kind_margin,
            named: places.yours_name().as_ref(),
            months: f.months,
        }),
        Holdings::Managed => card::where_managed(),
        Holdings::DontKnow => card::where_dont_know(),
    };
    let pill = match holdings {
        Holdings::IndexFunds if tied.len() > 1 || !worth => Pill::Tie,
        Holdings::IndexFunds => Pill::Clear,
        Holdings::Managed | Holdings::DontKnow => Pill::Rules,
    };
    (
        line(LineKind::Where, LineState::Stated { text, pill }),
        Some(WhereInfo {
            row,
            places,
            account: Some(if worth {
                Account::NewAccount
            } else {
                Account::YourBank
            }),
            decided: holdings == Holdings::IndexFunds,
        }),
    )
}

// ─────────────────────────── The chart ───────────────────────────

fn rows(f: &Facts<'_>, chosen: Option<Chosen>) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    // In the index's arena the plan's row holds the cushion too: the fund's
    // own row is the other arena, drawn beside (finding 1).
    let safe_active = |way: SafeWay| match (f.arena, chosen) {
        (Some(Arena::Safe), Some(Chosen::Safe(chosen))) => chosen == way,
        (None | Some(Arena::Safe), _) => true,
        (Some(Arena::Index), _) => false,
    };
    let fund_range = f
        .safe
        .group(SafeWay::MoneyMarketFund)
        .and_then(|fund| fund_range(f, fund));
    let mut safe_rows: Vec<(&Priced, SafeWay)> = f
        .safe
        .groups
        .iter()
        .flat_map(|group| {
            std::iter::once((&group.lead, group.way))
                .chain(group.variant.as_ref().map(|v| (v, group.way)))
        })
        .collect();
    safe_rows.sort_by_key(|(priced, _)| std::cmp::Reverse(priced.left.today));
    let index_rows: Vec<Row> = f.index.as_ref().map_or_else(Vec::new, |index| {
        let drawn = if f.arena == Some(Arena::Index) {
            index.drawn()
        } else {
            vec![index.lead]
        };
        drawn
            .into_iter()
            .map(|at| index_row(f, index, at, chosen))
            .collect()
    });
    let safe_row = |priced: &Priced, way: SafeWay| Row {
        id: priced.kind,
        kind: priced.kind,
        name: row_words::name(priced.kind),
        tags: row_words::tags(priced.kind, f.arena),
        facts: row_words::facts(priced.kind, f.arena),
        mark: match (priced.kind, fund_range) {
            (Kind::MoneyMarketFund, Some((low, high))) => Mark::RateRange { low, high },
            _ => Mark::Known,
        },
        marks: Vec::new(),
        value: priced.left,
        amount: row_words::amount(priced.kind, priced.left.today),
        tail: false,
        active: safe_active(way),
        excluded: None,
        // A bar that's drawn says what it spans (principle 10).
        note: (priced.kind == Kind::MoneyMarketFund && fund_range.is_some())
            .then(|| row_words::rate_note(f.settings.rate)),
    };
    if f.existing {
        // Money already invested: selling it for a safe way is no option
        // here (the visitor asked what it costs where it is).
        rows.extend(index_rows);
    } else if let Some(Arena::Index) = f.arena {
        rows.extend(index_rows);
        rows.extend(safe_rows.iter().map(|(priced, way)| safe_row(priced, *way)));
    } else {
        rows.extend(
            safe_rows
                .iter()
                .take(4)
                .map(|(priced, way)| safe_row(priced, *way)),
        );
        rows.extend(index_rows);
    }
    // Only options the visitor could choose get a row (§4.6): what the
    // rules leave out isn't drawn, and says why in its line instead.
    rows.retain(|row| row.excluded.is_none());
    rows.truncate(5);
    rows
}

fn index_row(f: &Facts<'_>, index: &IndexSide<'_>, at: usize, chosen: Option<Chosen>) -> Row {
    let way = index.way(at).way;
    let kind = Kind::Way(way);
    let assumed = index.left(at).unwrap_or(Amount {
        today: Decimal::ZERO,
        nominal: Decimal::ZERO,
    });
    let excluded = match f.place {
        Place_::Safe(because) if because != SafeBecause::Chosen => {
            Some(row_words::excluded(because, f.months))
        }
        _ => None,
    };
    let market = index.market.get(at).copied().flatten();
    let losses = index.losses.get(at).copied().flatten();
    let mark = match (market, index.history) {
        (Some((worst, median, best)), Some(history)) => Mark::Market {
            worst: at_end(worst, index.months, f.settings.inflation),
            median: at_end(median, index.months, f.settings.inflation),
            assumed,
            best: at_end(best, index.months, f.settings.inflation),
            losses: losses.map_or_else(
                || Text::new("", ""),
                |losses| {
                    row_words::losses(&row_words::PastCount {
                        below: losses.below_deposited,
                        windows: history.windows,
                        months: history.months,
                        since: history.from.year(),
                    })
                },
            ),
        },
        _ => Mark::Expected,
    };
    let marks = match &mark {
        Mark::Market {
            worst,
            median,
            assumed,
            best,
            ..
        } => {
            let mut marks = vec![
                (BandMarkKind::Worst, row_words::worst_mark(), *worst),
                (
                    BandMarkKind::Assumed,
                    row_words::assumed_mark(f.settings.index_return),
                    *assumed,
                ),
                (BandMarkKind::Median, row_words::median_mark(), *median),
                (BandMarkKind::Best, row_words::best_mark(), *best),
            ];
            marks.sort_by_key(|(_, _, value)| value.today);
            marks
                .into_iter()
                .map(|(which, label, value)| BandMark {
                    which,
                    label,
                    value,
                    amount: row_words::compact(value.today),
                })
                .collect()
        }
        Mark::Known | Mark::RateRange { .. } | Mark::Expected => Vec::new(),
    };
    let active = excluded.is_none()
        && match (f.arena, chosen) {
            (Some(Arena::Index), Some(Chosen::Index(chosen))) => chosen == at,
            (Some(Arena::Index), None) => at == index.lead || Some(at) == index.study,
            (None, _) => true,
            (Some(Arena::Index), Some(Chosen::Safe(_))) | (Some(Arena::Safe), _) => false,
        };
    Row {
        id: kind,
        kind,
        name: row_words::name(kind),
        tags: row_words::tags(kind, f.arena),
        facts: row_words::facts(kind, f.arena),
        mark,
        marks,
        value: assumed,
        amount: row_words::amount(kind, assumed.today),
        tail: false,
        active,
        excluded,
        note: index
            .cushion
            .map(|cushion| row_words::path_note(cushion.target, cushion.fill)),
    }
}

fn focus_row(f: &Facts<'_>, chosen: Chosen) -> Option<RowId> {
    match chosen {
        Chosen::Safe(way) => f.safe.group(way).map(|group| group.lead.kind),
        Chosen::Index(at) => f.index.as_ref().map(|index| Kind::Way(index.way(at).way)),
    }
}

/// From a little below the lowest bad case to a little above the highest
/// typical value. An excluded row neither stretches it nor shows its
/// figure inside: beyond the axis it is drawn as a tail (finding 11).
fn axis(
    mut rows: Vec<Row>,
    deposited: Amount,
    current_account: Amount,
) -> ((Decimal, Decimal), Vec<Row>) {
    let mut low = deposited.today.min(current_account.today);
    let mut high = deposited.today.max(current_account.today);
    for row in rows.iter().filter(|row| row.excluded.is_none()) {
        low = low.min(row.value.today);
        high = high.max(row.value.today);
        match &row.mark {
            Mark::RateRange {
                low: bottom,
                high: top,
            } => {
                low = low.min(bottom.today);
                high = high.max(top.today);
            }
            Mark::Market { worst, median, .. } => {
                low = low.min(worst.today);
                high = high.max(median.today);
            }
            Mark::Known | Mark::Expected => {}
        }
    }
    let pad = (high - low) * rust_decimal_macros::dec!(0.05);
    let (low, high) = ((low - pad).max(Decimal::ZERO), high + pad);
    for row in &mut rows {
        row.tail = row.excluded.is_some() && (row.value.today > high || row.value.today < low);
    }
    ((low, high), rows)
}

// ─────────────────────────── How ───────────────────────────

fn how_of(f: &Facts<'_>, row: &Row, info: Option<&WhereInfo>) -> How {
    let ctx = how_words::Context {
        kind: row.kind,
        now: f.answers.now,
        monthly: f.answers.monthly,
        months: f.months,
        today: f.settings.today,
        dated: f.dated,
        flexibility: f.answers.flexibility,
        cushion: f
            .index
            .as_ref()
            .and_then(|index| index.cushion)
            .filter(|_| f.arena == Some(Arena::Index))
            .map(|cushion| (cushion.target, cushion.fill)),
        account: info.and_then(|info| info.account),
        bank: f.bank.map(|bank| super::outcome::bank_name(f.data, bank)),
        existing: f.existing,
        pension_terms: pension_terms(f, row.kind),
    };
    let tie = (row.kind == Kind::MoneyMarketFund)
        .then(|| {
            super::outcome::money_market_tie(
                &f.safe_answers,
                f.months,
                &f.versions[0],
                f.provider_margin,
            )
        })
        .flatten();
    How {
        row: row.id,
        name: row.name.clone(),
        identity: how_words::identity(&ctx),
        facts: how_words::facts(&ctx),
        // A portfolio moved as it is buys nothing: no name to search for.
        search: (!f.existing)
            .then(|| how_words::search(&ctx, tie.as_ref()))
            .flatten()
            .map(|slot| Search {
                text: slot.text,
                placeholder: slot.placeholder,
                sources: slot.sources,
            }),
        // An option the rules exclude says why, and gives no steps.
        steps: if row.excluded.is_some() {
            Vec::new()
        } else {
            how_words::steps(&ctx)
        },
        not_offered: row.excluded.clone(),
        where_to_buy: Some(where_to_buy(f, row, info)),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the places, the banks as one row, the difference and the notes"
)]
fn where_to_buy(f: &Facts<'_>, row: &Row, info: Option<&WhereInfo>) -> WhereToBuy {
    let built: Option<Where> = match (info, row.kind) {
        (None, Kind::Way(way)) => f.index.as_ref().and_then(|index| {
            let at = index
                .run
                .ways
                .iter()
                .position(|outcome| outcome.way == way)?;
            Some(Where::new(
                index.places(at, f.bank, f.existing),
                f.provider_margin,
            ))
        }),
        (
            None,
            kind @ (Kind::GovernmentBond { .. }
            | Kind::FixedDeposit
            | Kind::AnyDayDeposit
            | Kind::TreasuryBill),
        ) => f
            .safe
            .groups
            .iter()
            .flat_map(|group| std::iter::once(&group.lead).chain(&group.variant))
            .find(|priced| priced.kind == kind)
            .map(|priced| {
                Where::new(
                    super::outcome::safe_places(priced, f.data, f.bank),
                    f.provider_margin,
                )
            }),
        _ => None,
    };
    let places = info.map(|info| &info.places).or(built.as_ref());
    let Some(places) = places else {
        return WhereToBuy {
            title: card::where_title(false, f.existing, &row.name),
            rows: Vec::new(),
            bank_range: None,
            reference: None,
            difference: None,
            note: (row.kind == Kind::MoneyMarketFund).then(|| {
                card::where_fund(
                    f.bank
                        .map(|bank| super::outcome::bank_name(f.data, bank))
                        .as_ref(),
                )
            }),
            asks_bank: false,
            unlock: None,
            may_cost_more: None,
        };
    };
    let yours = places.yours.map(|at| places.places[at].left);
    let is_lead = f
        .index
        .as_ref()
        .is_some_and(|index| row.kind == Kind::Way(index.way(index.lead).way));
    let lead_index = f.index.as_ref().filter(|_| is_lead);
    // The plan that leaves more but may cost more carries its warning on
    // its own row; a line says it only when its row isn't shown.
    let also = lead_index.and_then(|index| index.also.as_ref());
    let also_name = also.map(|also| super::safe::joined(&also.choice.provider, &also.choice.plan));
    // Until the visitor names a bank, the banks are one row, the reference
    // point (§4.7): from the dearest to the cheapest.
    // (A deposit is at a bank whichever it is: its places stay the banks.
    // So do they where no house takes the money, as a bond with no first
    // deposit: a reference needs something to be a reference for.)
    let banks_as_one = yours.is_none()
        && places.range.is_some()
        && !f.existing
        && places.places.iter().any(|place| place.bank.is_none())
        && !matches!(
            row.kind,
            Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit
        );
    let mut shown: Vec<Provider> = Vec::new();
    for (place, tie) in places.places.iter().zip(&places.ties) {
        let keep = *tie <= 1 && shown.len() < 4 || place.yours;
        if !keep || (banks_as_one && place.bank.is_some()) {
            continue;
        }
        shown.push(Provider {
            name: place.name.clone(),
            left: place.left,
            versus_yours: yours.filter(|_| !place.yours).map(|yours| Amount {
                today: place.left.today - yours.today,
                nominal: place.left.nominal - yours.nominal,
            }),
            yours: place.yours,
            tie: *tie,
            warning: also
                .filter(|_| also_name.as_ref() == Some(&place.name))
                .map(how_words::row_warning),
            amount: row_words::amount(row.kind, place.left.today),
        });
    }
    let warned = shown.iter().any(|row| row.warning.is_some());
    let asks_bank = f.bank.is_none()
        && row.kind != Kind::MoneyMarketFund
        && places
            .best()
            .zip(places.range)
            .is_some_and(|(best, (low, _))| best.left.today - low.today > f.provider_margin);
    let reference = places.range.filter(|_| banks_as_one).map(|(low, high)| {
        let cheapest = places
            .places
            .iter()
            .filter(|place| place.bank.is_some())
            .max_by(|a, b| a.left.today.cmp(&b.left.today))
            .map(|place| place.name.clone());
        BankReference {
            name: card::through_a_bank(),
            note: card::bank_reference_note(cheapest.as_ref()),
            low,
            high,
            amount: card::range_said(Some(row.kind), low.today, high.today),
        }
    });
    let difference = where_difference(f, row, places);
    // The tie the rows show: with the banks as one row, the houses'.
    let note = {
        let tied: Vec<Decimal> = shown
            .iter()
            .filter(|row| row.tie == 0)
            .map(|row| row.left.today)
            .collect();
        let spread = tied
            .iter()
            .max()
            .zip(tied.iter().min())
            .map_or(Decimal::ZERO, |(high, low)| high - low);
        card::where_note(tied.len(), spread)
    };
    WhereToBuy {
        title: card::where_title(yours.is_some(), f.existing, &row.name),
        rows: shown,
        bank_range: if yours.is_none() { places.range } else { None },
        reference,
        difference,
        note,
        asks_bank,
        unlock: lead_index
            .and_then(|index| index.unlock.as_ref())
            .map(how_words::unlock),
        may_cost_more: also.filter(|_| !warned).map(how_words::may_cost_more),
    }
}

// ─────────────────────────── The plan, the title ───────────────────────────

fn plan_text(f: &Facts<'_>, chosen: Option<Chosen>, info: Option<&WhereInfo>) -> Option<Text> {
    let chosen = chosen?;
    let cushion = f
        .index
        .as_ref()
        .and_then(|index| index.cushion)
        .filter(|_| f.arena == Some(Arena::Index))
        .map(|cushion| (cushion.target, cushion.fill));
    let tied: Vec<Text> = info.map_or_default(|info| info.places.line_names());
    let yours = info.and_then(|info| info.places.yours_name());
    let kind = match chosen {
        Chosen::Safe(way) => f.safe.group(way)?.lead.kind,
        Chosen::Index(at) => Kind::Way(f.index.as_ref()?.way(at).way),
    };
    Some(card::plan(&card::PlanFigures {
        kind,
        now: f.answers.now,
        monthly: f.answers.monthly,
        today: f.settings.today,
        months: f.months,
        dated: f.dated,
        cushion,
        tied: &tied,
        // "בחשבון חדש" only where the visitor chose it: a place the site
        // states may hold an account they already have (principle 5).
        account: info
            .filter(|info| info.decided)
            .and_then(|info| info.account),
        yours: yours.as_ref(),
        bank: f
            .bank
            .map(|bank| super::outcome::bank_name(f.data, bank))
            .as_ref(),
        holdings: if f.existing { f.choices.holdings } else { None },
    }))
}

fn title_and_body(f: &Facts<'_>, state: ConfidenceState, open: bool) -> (Text, Text) {
    let worst = f
        .index
        .as_ref()
        .and_then(|index| index.market[index.lead].map(|(worst, _, _)| worst));
    card::title_and_body(&card::TitleFigures {
        state,
        arena: f.arena,
        asked: matches!(f.place, Place_::Asked { .. }),
        existing: f.existing,
        months: f.months,
        dated: f.dated,
        fixed_date: matches!(f.answers.horizon, Horizon::Months(_)),
        safe_kinds: safe_kinds(f),
        open,
        retirement: matches!(f.answers.horizon, Horizon::Retirement { .. }),
        below_deposited: worst.is_some_and(|worst| worst < f.deposited.today),
        lead: f.index.as_ref().map(|index| index.way(index.lead).way),
        any_day: f.answers.needs_any_day,
    })
}

/// The monthly pension the provident fund for investment would pay from the
/// way at `at`, for retirement: nothing for another way, or for an age the
/// payers' tables don't cover.
fn annuity_of(f: &Facts<'_>, at: Option<usize>) -> Option<crate::annuity::Estimate> {
    let Horizon::Retirement { age, retire_at } = f.answers.horizon else {
        return None;
    };
    let index = f.index.as_ref()?;
    let at = at?;
    if index.way(at).way.vehicle != Vehicle::InvestmentGemel {
        return None;
    }
    // The fund's own sum: a cushion kept in a money market fund isn't in it.
    let sum = index.left(at)?.today - index.offset();
    let today = f.settings.today?;
    let starts_in = today.year() + i32::try_from(retire_at - age).ok()?;
    crate::annuity::monthly(sum, retire_at, starts_in)
}

/// Whose coefficients the pension of a provident fund's row rests on, and on
/// what terms: said in its steps, not in the option (the review's N7).
fn pension_terms(f: &Facts<'_>, kind: Kind) -> Option<Text> {
    let Kind::Way(way) = kind else { return None };
    let index = f.index.as_ref()?;
    let at = index
        .run
        .ways
        .iter()
        .position(|outcome| outcome.way == way)?;
    annuity_of(f, Some(at)).map(|estimate| estimate.assumes)
}

/// For retirement, once the money goes to the index: the pension of the way
/// chosen. While the place type is the visitor's, chosen or not, the
/// pension is the growth option's second amount instead (finding 11: an
/// option's result isn't shown before it's chosen; principle 13: nor twice
/// after).
fn annuity(f: &Facts<'_>, chosen: Option<Chosen>, lines: &[Line]) -> Option<Text> {
    if f.arena != Some(Arena::Index) {
        return None;
    }
    let asked = lines.iter().any(|line| {
        matches!(&line.state, LineState::Decision(decision) if decision.id == DecisionId::PlaceType)
    });
    let index = f.index.as_ref()?;
    let at = match chosen {
        Some(Chosen::Index(at)) => at,
        Some(Chosen::Safe(_)) => return None,
        None => index.lead,
    };
    if asked && at == index.lead {
        return None;
    }
    annuity_of(f, Some(at)).map(|estimate| card::annuity(&estimate, f.settings.index_return))
}

// ─────────────────────────── The gain ───────────────────────────

/// Only where fees or tax decide something on the page: the account or the
/// move (fees), the study fund (tax), or retirement's way out (tax).
#[allow(
    clippy::too_many_lines,
    reason = "each decision fees or tax decide, then the bars' parts"
)]
fn gain(
    f: &Facts<'_>,
    lines: &[Line],
    chosen: Option<Chosen>,
    info: Option<&WhereInfo>,
) -> Option<Gain> {
    let index = f.index.as_ref()?;
    let decided = |id: DecisionId| {
        lines
            .iter()
            .any(|line| matches!(&line.state, LineState::Decision(decision) if decision.id == id))
    };
    let base = &f.versions[0];
    let rates = &f.settings.rates;
    // Each row: its label, the name a sentence uses, and its split. The
    // takeaway is about the last two: the decision fees or tax decide last.
    let mut rows: Vec<(Text, Text, Split)> = Vec::new();
    let moved = f.existing && info.is_some_and(|info| info.decided);
    if decided(DecisionId::Account) || moved {
        let info = info.filter(|info| info.decided)?;
        let Some(Chosen::Index(at)) = chosen else {
            return None;
        };
        let way = index.way(at);
        let best = info.places.best()?;
        let named = info.places.yours.map(|at| &info.places.places[at]);
        let reference = named.or_else(|| {
            // Until the bank is named: the bank that leaves most, said as
            // such, as the difference measures from it (the review's N4).
            info.places
                .places
                .iter()
                .filter(|place| place.bank.is_some())
                .max_by(|a, b| a.left.today.cmp(&b.left.today))
        })?;
        if best.cand == reference.cand {
            return None;
        }
        // The way not chosen stays beside them (principle 4): the study fund
        // the wrapper's decision offered.
        let unchosen = index
            .study
            .filter(|study| decided(DecisionId::Wrapper) && *study != at);
        if let Some(study) = unchosen {
            let name = row_words::name(Kind::Way(index.way(study).way));
            rows.push((name.clone(), name, best_split(f, index, study)?));
        }
        for place in [best, reference] {
            let cand = &index.run.cands[place.cand?];
            let outcome = plan_outcome(cand, way.way, &index.run.base, base, rates)?;
            let label = match (place.yours, named.is_none() && place.cand == reference.cand) {
                (true, _) => gain_words::your_bank(&place.provider),
                (false, true) if f.existing => gain_words::cheapest_branch(&place.provider),
                (false, true) => gain_words::cheapest_bank(&place.provider),
                (false, false) => place.provider.clone(),
            };
            let label = if unchosen.is_some() {
                gain_words::way_at(way.way, &label)
            } else {
                label
            };
            rows.push((
                label,
                place.provider.clone(),
                index.split(&outcome, f.deposited.today),
            ));
        }
    } else if decided(DecisionId::Wrapper) {
        for at in [Some(index.lead), index.study] {
            let at = at?;
            let name = row_words::name(Kind::Way(index.way(at).way));
            rows.push((name.clone(), name, best_split(f, index, at)?));
        }
    } else if matches!(f.answers.horizon, Horizon::Retirement { .. })
        && f.arena == Some(Arena::Index)
    {
        // Retirement's lesson: the way out decides the tax, so the
        // provident fund beside the brokerage account.
        let brokerage = index
            .beside
            .iter()
            .copied()
            .find(|at| index.way(*at).way.vehicle == Vehicle::Brokerage)?;
        for at in [brokerage, index.lead] {
            let name = row_words::name(Kind::Way(index.way(at).way));
            rows.push((name.clone(), name, best_split(f, index, at)?));
        }
    } else {
        return None;
    }
    // Worked out at the expected return: rounded as the headline amounts are.
    let amount =
        |today: Decimal| at_end(gain_words::rounded(today), f.months, f.settings.inflation);
    let gain_rows: Vec<GainRow> = rows
        .iter()
        .map(|(name, _, split)| {
            let (kept, tax, fees, product) = (
                amount(split.kept),
                amount(split.tax),
                amount(split.fees),
                amount(split.product),
            );
            let parts = [
                (GainPartKind::Kept, kept),
                (GainPartKind::Tax, tax),
                (GainPartKind::Fees, fees),
                (GainPartKind::Fund, product),
            ]
            .into_iter()
            .filter(|(_, value)| value.today > Decimal::ZERO)
            .map(|(which, value)| GainPart {
                which,
                label: gain_words::part_label(which),
                value,
                amount: gain_words::part_amount(value.today),
            })
            .collect();
            GainRow {
                name: name.clone(),
                kept,
                tax,
                fees,
                product,
                parts,
                note: tax.today.is_zero().then(gain_words::untaxed),
            }
        })
        .collect();
    let [.., (_, first_name, first), (_, second_name, second)] = rows.as_slice() else {
        return None;
    };
    Some(Gain {
        lead: gain_words::lead(f.deposited),
        takeaway: gain_words::takeaway(first_name, part(first), second_name, part(second)),
        rows: gain_rows,
    })
}

/// The takeaway's figures: the fees apart from the fund's own cost, which
/// the bars draw as a part of its own (§4.7), so the sentence agrees with
/// them (principle 11).
fn part(split: &Split) -> gain_words::Part {
    gain_words::Part {
        kept: split.kept,
        tax: split.tax,
        fees: split.fees,
        product: split.product,
    }
}

// ─────────────────────────── The plan's amount and the path ───────────────────────────

/// The kind the plan is about.
fn plan_kind(f: &Facts<'_>, chosen: Option<Chosen>) -> Option<Kind> {
    match chosen? {
        Chosen::Safe(way) => Some(f.safe.group(way)?.lead.kind),
        Chosen::Index(at) => Some(Kind::Way(f.index.as_ref()?.way(at).way)),
    }
}

/// The plan's amount (§4.5): what the place chosen leaves, the date, how
/// sure, the deposits, and for stocks what the figures were measured on.
fn plan_amount(
    f: &Facts<'_>,
    chosen: Option<Chosen>,
    info: Option<&WhereInfo>,
) -> Option<PlanAmount> {
    let kind = plan_kind(f, chosen)?;
    let row_value = match chosen? {
        Chosen::Safe(way) => f.safe.group(way)?.lead.left,
        Chosen::Index(at) => f.index.as_ref()?.left(at)?,
    };
    // Through a bank not named: the banks' range, its low end the figure.
    let (value, range) = match info {
        Some(info) if info.account == Some(Account::YourBank) && info.places.yours.is_none() => {
            match info.places.range {
                Some((low, high)) => (low, Some((low, high))),
                None => (row_value, None),
            }
        }
        Some(info) if info.account == Some(Account::YourBank) => {
            (info.places.reference().unwrap_or(row_value), None)
        }
        Some(info) if info.account == Some(Account::NewAccount) => {
            (info.places.best().map_or(row_value, |best| best.left), None)
        }
        _ => (row_value, None),
    };
    let sureness = match kind {
        Kind::Way(way) => {
            path_words::Sureness::Index(f.settings.index_return, way.vehicle == Vehicle::Brokerage)
        }
        kind if row_words::known(kind) => path_words::Sureness::Known,
        _ => path_words::Sureness::Rate,
    };
    let (amount, when, basis, example) = path_words::plan_amount(
        value.today,
        sureness,
        f.settings.today,
        f.months,
        f.dated,
        f.deposited.today,
    );
    let amount = match range {
        Some((low, high)) => card::range_said(Some(kind), low.today, high.today),
        None => amount,
    };
    Some(PlanAmount {
        value,
        amount,
        when,
        basis,
        example,
    })
}

/// "איך הגעתם לזה" (§4.5): a step for each line, in the answer's order, and
/// "איזה מדד" after the place type where the money goes to stocks.
fn path(
    f: &Facts<'_>,
    lines: &[Line],
    chosen: Option<Chosen>,
    info: Option<&WhereInfo>,
) -> Vec<PathStep> {
    // Before the product is chosen (a decision still open), the index the
    // money goes to, as the place type says.
    let product = plan_kind(f, chosen)
        .or_else(|| {
            let index = f.index.as_ref().filter(|_| f.arena == Some(Arena::Index))?;
            Some(Kind::Way(index.way(index.lead).way))
        })
        .map_or_else(|| Text::new("", ""), row_words::name);
    let mut steps = Vec::new();
    for line in lines {
        let step = match &line.state {
            LineState::Waiting => continue,
            LineState::Decision(decision) => {
                let Some(pick) = decision.chosen else {
                    steps.push(PathStep {
                        line: Some(line.kind),
                        label: line.label.clone(),
                        kind: StepKind::Open,
                        pill: None,
                        choice: decision.question.clone(),
                        hinge: decision.why.clone(),
                        reopens: Some(decision.id),
                        why_clear: None,
                    });
                    continue;
                };
                let words = decision_step(f, decision, pick, &product, info);
                PathStep {
                    line: Some(line.kind),
                    label: line.label.clone(),
                    kind: StepKind::Visitor,
                    pill: None,
                    choice: words.choice,
                    hinge: words.hinge,
                    reopens: Some(decision.id),
                    why_clear: None,
                }
            }
            LineState::Stated { text, pill } => {
                let (words, why_clear) = stated_step(f, line.kind, text, chosen, info);
                PathStep {
                    line: Some(line.kind),
                    label: line.label.clone(),
                    kind: StepKind::Site,
                    pill: Some(*pill),
                    choice: words.choice,
                    hinge: words.hinge,
                    reopens: None,
                    why_clear,
                }
            }
        };
        let place_type = line.kind == LineKind::PlaceType;
        steps.push(step);
        // Which index is still the visitor's, outside the site (principle 17).
        if place_type && f.arena == Some(Arena::Index) && !f.existing {
            let (label, words) = path_words::which_index();
            steps.push(PathStep {
                line: None,
                label,
                kind: StepKind::Outside,
                pill: None,
                choice: words.choice,
                hinge: words.hinge,
                reopens: None,
                why_clear: None,
            });
        }
    }
    steps
}

/// A decided step's words, from the option chosen.
fn decision_step(
    f: &Facts<'_>,
    decision: &Decision,
    pick: Pick,
    product: &Text,
    info: Option<&WhereInfo>,
) -> path_words::StepWords {
    let no_date = matches!(f.place, Place_::Asked { no_date: true });
    let lead_way = f.index.as_ref().map(|index| index.way(index.lead).way);
    match pick {
        Pick::Cushion(CushionPick::Has) => path_words::cushion_has(f.answers.monthly, product),
        Pick::Cushion(CushionPick::HasNot) => {
            let ask = f.cushion;
            path_words::cushion_first(
                ask.map_or(Decimal::ZERO, |ask| ask.target),
                f.answers.monthly,
                product,
                ask.is_some_and(|ask| ask.takes_all),
            )
        }
        Pick::Flexibility(Flexibility::Fixed) => {
            path_words::safe_chosen(f.settings.today, f.months, f.dated, no_date)
        }
        Pick::Flexibility(Flexibility::CanWait) => {
            path_words::stocks_chosen(f.settings.today, f.months, f.dated, no_date)
        }
        Pick::SafeWay(way) => {
            let kind_of = |way: SafeWay| f.safe.group(way).map(|group| group.lead.kind);
            let kind = kind_of(way).unwrap_or(Kind::MoneyMarketFund);
            // Two options, the fund against a sum known in advance: "the
            // simplest, or knowing today" (the question it asked).
            let other = decision
                .options
                .iter()
                .filter_map(|option| match option.pick {
                    Pick::SafeWay(other) if other != way => kind_of(other),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let pair = decision.options.len() == 2;
            match (kind, other.as_slice()) {
                (Kind::MoneyMarketFund, [other]) if pair && row_words::known(*other) => {
                    path_words::simplest_fund(*other, &row_words::name(*other))
                }
                (kind, [other])
                    if pair && *other == Kind::MoneyMarketFund && row_words::known(kind) =>
                {
                    path_words::known_in_advance(&row_words::name(kind))
                }
                (Kind::MoneyMarketFund, _) => path_words::safe_fund(),
                (Kind::AnyDayDeposit, _) => path_words::safe_any_day_deposit(),
                (Kind::TreasuryBill, _) => path_words::safe_bill(),
                (Kind::GovernmentBond { .. }, _) => path_words::safe_bond(),
                _ => path_words::safe_deposit(f.settings.today, f.months, f.dated),
            }
        }
        Pick::Wrapper(Wrapper::Anyone) => path_words::wrapper_anyone(
            lead_way.map_or_else(|| Text::new("", ""), card::way_short_text),
        ),
        Pick::Wrapper(Wrapper::StudyFund) => path_words::wrapper_study(
            &lead_way.map_or_else(|| Text::new("", ""), card::way_short_text),
        ),
        Pick::Account(Account::NewAccount) => path_words::new_account(&card::in_names(
            &info.map_or_else(Vec::new, |info| info.places.tied_names()),
        )),
        Pick::Account(Account::YourBank) => path_words::your_bank(f.bank.is_some()),
        Pick::Holdings(Holdings::IndexFunds) => path_words::holdings_index_funds(),
        Pick::Holdings(Holdings::Managed) => path_words::holdings_managed(),
        Pick::Holdings(Holdings::DontKnow) => path_words::holdings_dont_know(),
    }
}

/// A stated step's words, and "למה זה ברור?" where it has one.
fn stated_step(
    f: &Facts<'_>,
    kind: LineKind,
    text: &Text,
    chosen: Option<Chosen>,
    info: Option<&WhereInfo>,
) -> (path_words::StepWords, Option<WhyClear>) {
    match kind {
        LineKind::PlaceType => match f.place {
            Place_::Index { .. } => (path_words::stocks_stated(f.months), why_clear(f)),
            Place_::Safe(SafeBecause::Short | SafeBecause::KnownDate | SafeBecause::Rules) => {
                (path_words::safe_stated(text), why_clear(f))
            }
            Place_::Existing => (path_words::existing_stated(text), None),
            _ => (path_words::safe_stated(text), None),
        },
        LineKind::Way => {
            let name = plan_kind(f, chosen).map_or_else(|| Text::new("", ""), row_words::name);
            let why = if let Some(Chosen::Safe(_)) = chosen {
                path_words::safe_way_why(f.safe.groups.len() < 2)
            } else {
                let tied = f.index.as_ref().and_then(|index| {
                    let next = index.next?;
                    (way_standing(f, index) == Standing::Tie)
                        .then(|| row_words::full_name(Kind::Way(index.way(next).way)))
                });
                path_words::index_way_why(tied.as_ref())
            };
            (path_words::way_stated(&name, why), None)
        }
        LineKind::Where => {
            let choice = match (chosen, info) {
                (Some(Chosen::Safe(SafeWay::MoneyMarketFund)), _) => card::where_fund_short(
                    f.bank
                        .map(|bank| super::outcome::bank_name(f.data, bank))
                        .as_ref(),
                ),
                (_, Some(info)) if !info.places.line_names().is_empty() => {
                    card::names_text(&info.places.line_names())
                }
                (Some(Chosen::Index(at)), _) => f.index.as_ref().map_or_else(
                    || Text::new("", ""),
                    |index| card::where_fund_company_short(index.way(at).way.vehicle),
                ),
                _ => Text::new("", ""),
            };
            (path_words::where_stated(choice, text), None)
        }
        LineKind::Before => (path_words::where_stated(text.clone(), text), None),
    }
}

/// "למה זה ברור?" for the place type stated (§4.5): the count of the past's
/// stretches of the span, the crises, the four amounts. Only where the
/// history's span is the visitor's (whole years), so the amounts compare.
fn why_clear(f: &Facts<'_>) -> Option<WhyClear> {
    let index = f.index.as_ref()?;
    let history = index.history?;
    if index.months != f.months {
        return None;
    }
    let losses = index.losses.get(index.lead).copied().flatten()?;
    let (worst, _, _) = index.market.get(index.lead).copied().flatten()?;
    let safe = f.safe.groups.first()?;
    let assumed = index.left(index.lead)?;
    let (deposited, _) = yardsticks(f.answers, index.months, f.settings.inflation);
    let worst = at_end(worst, index.months, f.settings.inflation);
    let since = history.from.year();
    let safe_name = row_words::name(safe.lead.kind);
    let words = path_words::why_clear(&path_words::WhyFigures {
        months: history.months,
        windows: history.windows,
        below: losses.below_deposited,
        since,
        now: f.answers.now,
        monthly: f.answers.monthly,
        stocks: f.arena == Some(Arena::Index),
        worst_beats_safe: worst.today > safe.lead.left.today,
        safe: safe_name.clone(),
        index_return: f.settings.index_return,
    });
    let stock = Kind::Way(index.way(index.lead).way);
    let figure = |label: Text, kind: Kind, value: Amount| Figure {
        label,
        value,
        amount: row_words::amount(kind, value.today),
    };
    Some(WhyClear {
        lead: words.lead,
        count: words.count,
        crises: words.crises,
        amounts: vec![
            figure(path_words::deposited_label(), stock, deposited),
            figure(
                path_words::safe_label(&safe_name),
                safe.lead.kind,
                safe.lead.left,
            ),
            figure(path_words::worst_label(since), stock, worst),
            figure(
                path_words::assumed_label(f.settings.index_return),
                stock,
                assumed,
            ),
        ],
        past: words.past,
    })
}

/// Where to buy's difference (§4.7): the best places against the banks'
/// range, or the visitor's bank once named. None where a bank is the best.
fn where_difference(f: &Facts<'_>, row: &Row, places: &Where) -> Option<Text> {
    // A deposit's places are all banks: the line says how they stand.
    let deposit = matches!(
        row.kind,
        Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit
    );
    if f.existing || deposit {
        return None;
    }
    let best = places.best()?;
    if best.bank.is_some() || best.yours {
        return None;
    }
    let names: Vec<Text> = places
        .tied()
        .iter()
        .filter(|place| place.bank.is_none())
        .map(|place| place.provider.clone())
        .collect();
    let (gap, yours) = match (places.yours, places.range) {
        (Some(at), _) => {
            let gap = best.left.today - places.places[at].left.today;
            ((gap, gap), Some(places.places[at].provider.clone()))
        }
        (None, Some((low, high))) => (
            (best.left.today - high.today, best.left.today - low.today),
            None,
        ),
        (None, None) => return None,
    };
    (gap.1 > Decimal::ZERO)
        .then(|| card::where_difference(Some(row.kind), &names, gap, yours.as_ref(), f.months))
}

// ─────────────────────────── The horizons ───────────────────────────

/// "What if the date moves?" (§4.10): the same money at dates built around
/// the visitor's (`dates_around`). Each row is what the answer itself would
/// show with the date there and everything else as it is, so that moving
/// the date opens exactly the row. Where the visitor hasn't said whether
/// the date can move and that decides the place there, a fit for each
/// answer.
fn horizons(f: &Facts<'_>) -> Vec<HorizonRow> {
    let mut rows = Vec::new();
    for (horizon, current) in dates_around(f.answers.horizon) {
        let at = Answers {
            horizon,
            ..f.answers.clone()
        };
        if check(&at).is_err() {
            continue;
        }
        let answered = |flexibility| Answers {
            flexibility: Some(flexibility),
            ..at.clone()
        };
        let (fixed, can_wait) = (answered(Flexibility::Fixed), answered(Flexibility::CanWait));
        let mut facts_at = facts(&at, f.choices, f.settings, f.data, Parts::ForAHorizonRow);
        let months = facts_at.months;
        let deposited = facts_at.deposited;
        let mut fits = Vec::new();
        if facts_at.arena.is_some() {
            fits.extend(horizon_fit(&facts_at, None));
        } else {
            // The two answers' facts differ only in the arena: both arenas
            // run the row's whole years, so the engines needn't be worked
            // out again.
            for (flexibility, answers) in [
                (Flexibility::Fixed, &fixed),
                (Flexibility::CanWait, &can_wait),
            ] {
                facts_at.answers = answers;
                facts_at.arena = match facts_at.place.arena(Some(flexibility)) {
                    Some(Arena::Index) if facts_at.index.is_none() => Some(Arena::Safe),
                    arena => arena,
                };
                fits.extend(horizon_fit(&facts_at, Some(flexibility)));
            }
        }
        if fits.is_empty() {
            continue;
        }
        rows.push(HorizonRow {
            months: horizon_months(horizon).unwrap_or(months),
            when: gain_words::when(horizon_months(horizon).unwrap_or(months)),
            current,
            label: current.then(gain_words::your_date),
            deposited,
            deposits: gain_words::deposits(deposited),
            fits,
        });
    }
    rows
}

/// The months a horizon gives, if it gives a date.
fn horizon_months(horizon: Horizon) -> Option<u32> {
    match horizon {
        Horizon::Months(months) => Some(months),
        Horizon::Retirement { age, retire_at } => Some(retire_at.saturating_sub(age) * 12),
        Horizon::Unknown => None,
    }
}

/// The dates of the horizon section (§4.10), each with whether it is the
/// visitor's own: from the list 1, 3, 5, 10 and 20 years, the closest date
/// before the visitor's, and 5 years (or 1, if 5 is that closest date or
/// after it) when it's earlier still; the visitor's own date, months
/// included; the list's next date after it, or from 20 years on the
/// visitor's date plus 5 years. With no date, the list itself.
fn dates_around(horizon: Horizon) -> Vec<(Horizon, bool)> {
    let Some(months) = horizon_months(horizon) else {
        return HORIZON_YEARS
            .iter()
            .map(|years| (Horizon::Months(years * 12), false))
            .collect();
    };
    let list: Vec<u32> = HORIZON_YEARS.iter().map(|years| years * 12).collect();
    let mut dates: Vec<u32> = Vec::new();
    if let Some(&before) = list.iter().rev().find(|&&date| date < months) {
        let earlier = if before > 5 * 12 { 5 * 12 } else { 12 };
        if earlier < before {
            dates.push(earlier);
        }
        dates.push(before);
    }
    let longest = *list.last().unwrap_or(&240);
    let after = if months >= longest {
        Some(months + 5 * 12)
    } else {
        list.iter().copied().find(|&date| date > months)
    };
    let mut out: Vec<(Horizon, bool)> = dates
        .into_iter()
        .map(|date| (moved_horizon(horizon, date), false))
        .collect();
    out.push((horizon, true));
    out.extend(after.map(|date| (moved_horizon(horizon, date), false)));
    out
}

/// The date `months` away, as the page's fields set it: a retirement keeps
/// the age and moves the age of retiring (by whole years).
fn moved_horizon(horizon: Horizon, months: u32) -> Horizon {
    match horizon {
        Horizon::Retirement { age, .. } => Horizon::Retirement {
            age,
            retire_at: age.saturating_add(months / 12),
        },
        Horizon::Months(_) | Horizon::Unknown => Horizon::Months(months),
    }
}

/// One fit of a row, from the answer at that date: the way it settles on,
/// or the ways it leaves to the visitor (for the index, the one anyone can
/// open: a study fund is only for those who have one), and the range their
/// marks span.
fn horizon_fit(f: &Facts<'_>, flexibility: Option<Flexibility>) -> Option<HorizonFit> {
    let answer = build(f, Parts::ForAHorizonRow);
    let rows = &answer.outcome.rows;
    let open_way = answer.lines.iter().find_map(|line| match &line.state {
        LineState::Decision(decision)
            if line.kind == LineKind::Way && decision.chosen.is_none() =>
        {
            Some(decision)
        }
        _ => None,
    });
    let row_of = |id: &RowId| rows.iter().find(|row| row.id == *id);
    // The rows the row stands for, and the ones it names (an option's first).
    let (shown, named): (Vec<&Row>, Vec<&Row>) = match (answer.focus, open_way) {
        (Some(focus), _) => {
            let row = row_of(&focus)?;
            (vec![row], vec![row])
        }
        (None, Some(decision)) => {
            let options: Vec<&Opt> = decision
                .options
                .iter()
                .filter(|option| option.pick != Pick::Wrapper(Wrapper::StudyFund))
                .collect();
            (
                options
                    .iter()
                    .flat_map(|option| option.rows.iter().filter_map(row_of))
                    .collect(),
                options
                    .iter()
                    .filter_map(|option| option.rows.first().and_then(row_of))
                    .collect(),
            )
        }
        (None, None) => {
            let active: Vec<&Row> = rows
                .iter()
                .filter(|row| row.active && row.excluded.is_none())
                .filter(|row| row.kind.arena() == f.arena)
                .collect();
            let first = *active.first()?;
            (active, vec![first])
        }
    };
    let first = *named.first()?;
    let ends = |row: &Row| match &row.mark {
        Mark::Known | Mark::Expected => (row.value, row.value),
        Mark::RateRange { low, high } => (*low, *high),
        Mark::Market { worst, median, .. } => (*worst, *median),
    };
    let low = shown
        .iter()
        .map(|row| ends(row).0)
        .min_by(|a, b| a.today.cmp(&b.today))?;
    let high = shown
        .iter()
        .map(|row| ends(row).1)
        .max_by(|a, b| a.today.cmp(&b.today))?;
    let rate = shown
        .iter()
        .any(|row| matches!(row.mark, Mark::RateRange { .. }));
    let range_of = if shown.len() > 1 && f.arena == Some(Arena::Safe) {
        gain_words::RangeOf::Ways { rate }
    } else {
        match first.mark {
            Mark::Known => gain_words::RangeOf::Known,
            Mark::RateRange { .. } => gain_words::RangeOf::Rate,
            Mark::Market { .. } => gain_words::RangeOf::Past,
            Mark::Expected => gain_words::RangeOf::Expected(f.settings.index_return),
        }
    };
    // Options can stand on the same row's kind (what's in a portfolio: every
    // answer keeps the same fund), so each name is said once.
    let mut titles: Vec<Text> = Vec::new();
    for row in &named {
        if !titles.contains(&row.name) {
            titles.push(row.name.clone());
        }
    }
    Some(HorizonFit {
        flexibility,
        condition: flexibility.map(gain_words::condition),
        kind: first.kind,
        name: gain_words::either(&titles),
        low,
        high,
        range: gain_words::range(low, high),
        range_note: gain_words::range_note(range_of),
    })
}

#[cfg(test)]
mod tests {
    //! The private parts the answer is made of, where a rule about the
    //! answer as a whole can't reach them: the arithmetic of the cushion's
    //! path, the index's figures with the cushion in them, the axis, and the
    //! places a line names.

    use std::sync::LazyLock;

    use rust_decimal_macros::dec;

    use super::*;
    use crate::ExchangeRates;

    static DATA: LazyLock<Data> = LazyLock::new(Data::listed);
    static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
        let mut settings = Settings::new(ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap());
        settings.today = Some(time::macros::date!(2026 - 10 - 01));
        settings
    });

    fn answers(now: Decimal, monthly: Decimal, months: u32) -> Answers {
        Answers {
            situation: None,
            now,
            monthly,
            horizon: Horizon::Months(months),
            flexibility: None,
            cushion: None,
            needs_any_day: false,
            bank: Some(Bank::Hapoalim),
        }
    }

    fn facts_of(answers: Answers) -> Facts<'static> {
        let answers: &'static Answers = Box::leak(Box::new(answers));
        facts(
            answers,
            Choices::default(),
            &SETTINGS,
            &DATA,
            Parts::All { horizons: false },
        )
    }

    fn base() -> Version {
        robust::versions(&SETTINGS, None)[0]
    }

    // ─────────────────────────── Where the money goes ───────────────────────────

    /// Appendix A's lines, each on its side.
    #[test]
    fn the_place_follows_the_lines() {
        let place = |answers: Answers| place_of(&answers, Overrides::default());
        let no_date = |now, monthly| Answers {
            horizon: Horizon::Unknown,
            ..answers(now, monthly, 1)
        };
        assert_eq!(
            place(no_date(dec!(10_000), dec!(0))),
            Place_::Asked { no_date: true }
        );
        assert_eq!(
            place(no_date(dec!(0), dec!(1000))),
            Place_::Index { chosen: false }
        );
        let at = |months| place(answers(dec!(10_000), dec!(0), months));
        assert_eq!(at(12), Place_::Safe(SafeBecause::Short));
        assert_eq!(at(13), Place_::Safe(SafeBecause::KnownDate));
        assert_eq!(at(36), Place_::Safe(SafeBecause::KnownDate));
        assert_eq!(at(37), Place_::Safe(SafeBecause::Rules));
        assert_eq!(at(59), Place_::Safe(SafeBecause::Rules));
        assert_eq!(at(60), Place_::Asked { no_date: false });
        assert_eq!(at(84), Place_::Asked { no_date: false });
    }

    /// A sum with no date is asked as five years: the safe side as money
    /// that may be needed any day, the index's as money that can wait.
    #[test]
    fn a_sum_with_no_date_is_asked_as_five_years() {
        let sum = Answers {
            horizon: Horizon::Unknown,
            ..answers(dec!(10_000), dec!(0), 1)
        };
        let (safe, index) = arena_answers(&sum, Place_::Asked { no_date: true });
        let five_years = Horizon::Months(NO_DATE_SUM_YEARS * 12);
        assert_eq!(safe.horizon, five_years);
        assert!(safe.needs_any_day);
        assert_eq!(index.horizon, five_years);
        assert_eq!(index.flexibility, Some(Flexibility::CanWait));
        // A date that can move: each side takes its own answer, on the date.
        let dated = answers(dec!(10_000), dec!(0), 72);
        let (safe, index) = arena_answers(&dated, Place_::Asked { no_date: false });
        assert_eq!(safe.flexibility, Some(Flexibility::Fixed));
        assert_eq!(index.flexibility, Some(Flexibility::CanWait));
        assert_eq!(safe.horizon, Horizon::Months(72));
        assert!(!safe.needs_any_day);
    }

    // ─────────────────────────── The safe side ───────────────────────────

    /// A way's two kinds (a bond linked or not) are led by the one that
    /// leaves more, whichever comes first.
    #[test]
    fn a_ways_lead_is_its_best_kind() {
        let scenario = crate::short_term::Scenario {
            first_deposit: dec!(100_000),
            monthly_deposit: dec!(0),
            months: 48,
            rate: SETTINGS.rate,
            inflation: SETTINGS.inflation,
        };
        let price = |linked| {
            super::super::safe::price(
                Kind::GovernmentBond { linked },
                &DATA,
                &scenario,
                Decimal::ONE,
            )
            .unwrap()
        };
        let (linked, unlinked) = (price(true), price(false));
        assert_ne!(linked.left.today, unlinked.left.today);
        let best = linked.left.today.max(unlinked.left.today);
        for order in [
            vec![linked.clone(), unlinked.clone()],
            vec![unlinked, linked],
        ] {
            let groups = groups_of(order);
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].lead.left.today, best);
            assert!(groups[0].variant.as_ref().unwrap().left.today < best);
        }
    }

    /// The rate moves only the money market fund; inflation moves every
    /// safe kind; the index's return and the funds' fees move none.
    #[test]
    fn a_version_moves_the_safe_kinds_it_is_about() {
        let f = facts_of(answers(dec!(100_000), dec!(2000), 36));
        let fund = f.safe.group(SafeWay::MoneyMarketFund).unwrap();
        let bond = f.safe.group(SafeWay::Bond).unwrap();
        let mut seen = Vec::new();
        for version in &f.versions {
            let value = |group| f.safe.value(group, &f.safe_answers, version, f.data);
            let (fund_moved, bond_moved) = (
                value(fund) != fund.lead.left.today,
                value(bond) != bond.lead.left.today,
            );
            match version.varies {
                Varies::Rate => assert!(fund_moved && !bond_moved, "{version:?}"),
                Varies::Inflation => assert!(fund_moved && bond_moved, "{version:?}"),
                _ => assert!(!fund_moved && !bond_moved, "{version:?}"),
            }
            seen.push(version.varies);
        }
        assert!(seen.contains(&Varies::Rate) && seen.contains(&Varies::Inflation));
    }

    // ─────────────────────────── The cushion ───────────────────────────

    /// The cushion's path: its months' deposits in the fund, then held there
    /// to the end, the tax and fees of each step in the money of its own end.
    #[test]
    fn the_cushion_is_filled_then_held() {
        let version = base();
        let saver = answers(dec!(5000), dec!(2000), 120);
        let fund = |first, monthly, months| {
            super::super::safe::price(
                Kind::MoneyMarketFund,
                &DATA,
                &crate::short_term::Scenario {
                    first_deposit: first,
                    monthly_deposit: monthly,
                    months,
                    rate: version.rate,
                    inflation: version.inflation,
                },
                Decimal::ONE,
            )
            .unwrap()
            .outcome
        };
        let level = |months| super::super::safe::price_level(version.inflation, months);
        let part = cushion_part(&saver, dec!(24_000), 10, 120, &version, &DATA).unwrap();
        let filled = fund(saver.now, saver.monthly, 10);
        let held = fund(filled.after_tax, Decimal::ZERO, 110);
        assert_eq!(part.fill, 10);
        assert_eq!(part.left.nominal, held.after_tax);
        assert_eq!(part.left.today, held.after_tax / level(120));
        assert_eq!(part.tax, filled.tax / level(10) + held.tax / level(120));
        assert_eq!(part.fees, filled.kept / level(10) + held.kept / level(120));
        assert!(part.left.nominal > filled.after_tax, "held, it grows");
        // Filled to the end, and past it: one step.
        let once = fund(saver.now, saver.monthly, 120);
        for fill in [120, 200] {
            let part = cushion_part(&saver, dec!(24_000), fill, 120, &version, &DATA).unwrap();
            assert_eq!(part.fill, 120);
            assert_eq!(part.left.nominal, once.after_tax);
            assert_eq!(part.tax, once.tax / level(120));
            assert_eq!(part.fees, once.kept / level(120));
        }
    }

    // ─────────────────────────── The index side ───────────────────────────

    /// With no cushion yet, the cushion is part of every figure of the
    /// index: what it leaves is added to each way's and each place's, and
    /// its deposits, tax and fees to the gain's split.
    #[test]
    fn every_index_figure_holds_the_cushion() {
        let saver = Answers {
            cushion: Some(Cushion::HasNot {
                monthly_expenses: None,
            }),
            ..answers(dec!(0), dec!(1000), 120)
        };
        let f = facts_of(saver.clone());
        let index = f.index.as_ref().unwrap();
        let cushion = index.cushion.expect("the path");
        let offset = index.offset();
        assert_eq!(offset, cushion.left.today);
        assert!(offset > Decimal::ZERO);
        let lead = index.lead;
        let way = super::super::safe::amount(index.way(lead).after_tax().unwrap(), index.level);
        let left = index.left(lead).unwrap();
        assert_eq!(left.today, way.today + offset);
        assert_eq!(left.nominal, way.nominal + offset * index.level);
        let rates = &SETTINGS.rates;
        assert_eq!(
            index.value(lead, &base(), rates),
            way_value(&index.run, index.way(lead), &base(), rates).map(|way| way + offset)
        );
        let bare = way_places(&index.run, index.way(lead), f.bank, false, index.level);
        let places = index.places(lead, f.bank, false);
        assert!(!places.is_empty());
        for (place, bare) in places.iter().zip(&bare) {
            assert_eq!(place.left.today, bare.left.today + offset);
            assert_eq!(place.left.nominal, bare.left.nominal + offset * index.level);
        }
        // The split: the way's on its own deposits, the cushion's beside it.
        let best = index.way(lead).best().unwrap();
        let outcome = plan_outcome(
            &index.run.cands[best.index],
            index.way(lead).way,
            &index.run.base,
            &base(),
            rates,
        )
        .unwrap();
        let deposited = f.deposited.today;
        let own = deposited - cushion.deposited;
        let alone = split(&outcome, own);
        assert_eq!(alone.kept, outcome.after_tax - own);
        assert_eq!(alone.fees + alone.product, outcome.fees.total());
        assert_eq!(alone.product, outcome.fees.product);
        let both = index.split(&outcome, deposited);
        assert_eq!(
            both.kept,
            alone.kept + cushion.left.today - cushion.deposited
        );
        assert_eq!(both.tax, alone.tax + cushion.tax);
        assert_eq!(both.fees, alone.fees + cushion.fees);
        assert_eq!(both.product, alone.product);
        // A bigger first deposit leaves more than these deposits do, the
        // cushion in both.
        let unlock = index
            .unlock
            .as_ref()
            .expect("no first deposit: an unlock line");
        assert!(unlock.instead_of.today > left.today, "{unlock:?}");
        assert!(unlock.after_tax.today > unlock.instead_of.today);
        for amount in [unlock.after_tax, unlock.instead_of] {
            let at_the_date = amount.today * index.level;
            assert!(
                (amount.nominal - at_the_date).abs() < dec!(0.01),
                "{amount:?}"
            );
        }
    }

    /// The index's other ways: the next is another way anyone can open; a
    /// study fund is drawn only when it leaves more than the margin more.
    #[test]
    fn the_index_s_next_and_study_fund_are_what_they_say() {
        let mut drawn = [0, 0];
        for monthly in [dec!(500), dec!(50)] {
            let months = if monthly == dec!(50) { 120 } else { 360 };
            let f = facts_of(answers(dec!(0), monthly, months));
            let index = f.index.as_ref().unwrap();
            let next = index.next.expect("another way");
            assert_ne!(next, index.lead);
            assert!(open_to_anyone(index.way(next).way));
            assert!(index.way(next).best().is_some());
            let lead = index.left(index.lead).unwrap().today;
            for at in 0..index.run.ways.len() {
                if index.way(at).way.vehicle != Vehicle::StudyFund {
                    assert_ne!(index.study, Some(at));
                    continue;
                }
                let beyond = index.left(at).unwrap().today - lead > f.kind_margin;
                assert_eq!(index.study == Some(at), beyond, "{monthly}");
                drawn[usize::from(beyond)] += 1;
            }
        }
        assert!(drawn[0] > 0 && drawn[1] > 0, "{drawn:?}");
    }

    /// How many past stretches leave a way below a figure: none below its
    /// worst, all above its best, about half at its median; and for the
    /// lead, how many below the best safe way, which leaves more than was
    /// deposited, so at least as many.
    #[test]
    fn the_stretches_below_a_figure_follow_the_market_range() {
        let saver = answers(dec!(0), dec!(500), 360);
        let f = facts_of(saver.clone());
        let index = f.index.as_ref().unwrap();
        let history = index.history.unwrap();
        let lead = index.lead;
        let (worst, median, best) = index.market[lead].unwrap();
        let below = |target| stretches_below(index, &saver, lead, target, &SETTINGS);
        assert_eq!(below(worst - dec!(1)), Some(0));
        assert_eq!(below(worst), Some(0));
        assert_eq!(below(best + dec!(1)), Some(history.windows));
        let half = below(median).unwrap();
        assert!(half.abs_diff(history.windows / 2) <= 3, "{half}");
        let losses = index.losses[lead].unwrap();
        let below_safe = losses.below_safe.expect("against the best safe way");
        assert!(below_safe >= losses.below_deposited);
        for at in index.drawn().into_iter().filter(|at| *at != lead) {
            assert!(index.losses[at].is_some_and(|losses| losses.below_safe.is_none()));
        }
    }

    /// The pension is the provident fund's own sum's, without the cushion
    /// kept in a money market fund beside it, from the year of retiring.
    #[test]
    fn the_pension_is_the_funds_own_sum() {
        let saver = Answers {
            horizon: Horizon::Retirement {
                age: 30,
                retire_at: 67,
            },
            cushion: Some(Cushion::HasNot {
                monthly_expenses: None,
            }),
            ..answers(dec!(0), dec!(3000), 0)
        };
        let f = facts_of(saver);
        let index = f.index.as_ref().unwrap();
        assert_eq!(index.way(index.lead).way.vehicle, Vehicle::InvestmentGemel);
        assert!(index.offset() > Decimal::ZERO);
        let sum = index.left(index.lead).unwrap().today - index.offset();
        assert_eq!(
            annuity_of(&f, Some(index.lead)),
            crate::annuity::monthly(sum, 67, 2026 + 37)
        );
    }

    // ─────────────────────────── The chart's axis ───────────────────────────

    fn row(value: Decimal, mark: Mark, excluded: bool) -> Row {
        Row {
            id: Kind::MoneyMarketFund,
            kind: Kind::MoneyMarketFund,
            name: Text::new("", ""),
            tags: Vec::new(),
            facts: Text::new("", ""),
            marks: Vec::new(),
            mark,
            value: Amount {
                today: value,
                nominal: value,
            },
            amount: Text::new("", ""),
            tail: false,
            active: true,
            excluded: excluded.then(|| Text::new("out", "בחוץ")),
            note: None,
        }
    }

    fn amount(value: Decimal) -> Amount {
        Amount {
            today: value,
            nominal: value,
        }
    }

    /// From 5% below the lowest bad case to 5% above the highest typical
    /// value: a market bar's best and an excluded row stretch nothing, and
    /// an excluded row beyond the ends is a tail.
    #[test]
    fn the_axis_spans_what_is_typical() {
        let rows = vec![
            row(
                dec!(120),
                Mark::RateRange {
                    low: amount(dec!(110)),
                    high: amount(dec!(130)),
                },
                false,
            ),
            row(
                dec!(150),
                Mark::Market {
                    worst: amount(dec!(70)),
                    median: amount(dec!(140)),
                    assumed: amount(dec!(150)),
                    best: amount(dec!(400)),
                    losses: Text::new("", ""),
                },
                false,
            ),
            row(dec!(500), Mark::Known, true),
            row(dec!(10), Mark::Known, true),
            row(dec!(100), Mark::Known, true),
            row(dec!(154), Mark::Known, true),
            row(dec!(66), Mark::Known, true),
        ];
        let ((low, high), rows) = axis(rows, amount(dec!(100)), amount(dec!(90)));
        // 70 to 150, and 5% of the 80 between them each way.
        assert_eq!((low, high), (dec!(66), dec!(154)));
        let tails: Vec<bool> = rows.iter().map(|row| row.tail).collect();
        assert_eq!(tails, [false, false, true, true, false, false, false]);
        // Never below zero.
        let (axis, _) = axis(
            vec![row(dec!(100), Mark::Known, false)],
            amount(dec!(1)),
            amount(dec!(1)),
        );
        assert_eq!(axis.0, Decimal::ZERO);
    }

    // ─────────────────────────── The places ───────────────────────────

    fn place(name: &str, left: Decimal, yours: bool) -> Place {
        Place {
            name: Text::owned(name.to_owned(), name.to_owned()),
            provider: Text::owned(name.to_owned(), name.to_owned()),
            left: amount(left),
            bank: yours.then_some(Bank::Hapoalim),
            yours,
            cost_a_year: None,
            cand: None,
        }
    }

    fn names(texts: &[Text]) -> Vec<String> {
        texts.iter().map(|text| text.en.to_string()).collect()
    }

    /// The tie group is every place within the providers' margin of the
    /// best; the visitor's bank behind it joins the line within the margin
    /// between kinds, named first, and the line's spread then runs from it.
    #[test]
    fn the_visitors_bank_joins_within_the_kinds_margin() {
        let places = || {
            vec![
                place("A", dec!(10_000), false),
                place("B", dec!(9900), false),
                place("Y", dec!(9500), true),
            ]
        };
        let apart = Where::new(places(), dec!(200));
        assert_eq!(apart.ties, [0, 0, 1]);
        assert_eq!(names(&apart.tied_names()), ["A", "B"]);
        assert_eq!(apart.tied().len(), 2);
        assert_eq!(apart.spread(), dec!(100));
        assert_eq!(
            apart.yours_name().map(|name| name.en.to_string()),
            Some("Y".to_owned())
        );
        let joined = Where::new(places(), dec!(200)).join_yours(dec!(1000));
        assert!(joined.yours_joins);
        let your_bank = card::your_bank(&Text::owned("Y".to_owned(), "Y".to_owned()));
        assert_eq!(
            names(&joined.line_names()),
            [your_bank.en.to_string(), "A".into(), "B".into()]
        );
        assert_eq!(joined.line_spread(), dec!(500));
        let behind = Where::new(places(), dec!(200)).join_yours(dec!(400));
        assert!(!behind.yours_joins);
        assert_eq!(names(&behind.line_names()), ["A", "B"]);
        assert_eq!(behind.line_spread(), dec!(100));
        // Already in the tie group, it joins nothing.
        let tied = Where::new(
            vec![
                place("A", dec!(10_000), false),
                place("Y", dec!(9950), true),
            ],
            dec!(200),
        )
        .join_yours(dec!(1000));
        assert!(!tied.yours_joins);
    }
}
