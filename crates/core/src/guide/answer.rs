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

use crate::describe::{card, gain as gain_words, how as how_words, outcome as row_words};
use crate::history::{self, DepositsBand};
use crate::ways::WayOutcome;
use crate::{Text, Vehicle};

use super::index::{Run, open_to_anyone};
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
    Amount, Answers, Arena, Bank, CUSHION_MONTHS, Cushion, Data, Flexibility, Horizon,
    InvalidAnswers, Kind, LADDER_YEARS, NO_DATE_SUM_YEARS, Overrides, STAND_IN_MONTHLY_EXPENSES,
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
    pub value: Option<Amount>,
    /// At most 3.
    pub facts: Vec<Text>,
    /// "מתאים אם…".
    pub fits_if: Option<Text>,
    pub button: Text,
    /// The chart's rows it stands for.
    pub rows: Vec<RowId>,
}

/// "מה יהיה לכם ב‑<month year>" (§2.6).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outcome {
    /// "October 2029", or "in 3 years" when the page gave no date.
    pub date: Text,
    /// The chart's title: "מה יהיה לכם באוקטובר 2029".
    pub title: Text,
    pub deposited: Amount,
    pub current_account: Amount,
    /// The ₪ axis, today's money: a little below the lowest bad case to a
    /// little above the highest typical value.
    pub axis: (Decimal, Decimal),
    /// At most 5.
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub id: RowId,
    pub kind: Kind,
    pub name: Text,
    pub tags: Vec<Text>,
    pub mark: Mark,
    /// The figure shown: the known sum, the fund's at today's rate, the
    /// index's at the expected return.
    pub value: Amount,
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
    /// The identity line (§2.8), its parts in order.
    pub identity: Vec<Text>,
    pub search: Option<Search>,
    pub steps: Vec<Text>,
    /// Why it isn't offered here, for an excluded row.
    pub not_offered: Option<Text>,
    pub where_to_buy: Option<WhereToBuy>,
}

/// What to type in the app. `placeholder` marks a slot the research hasn't
/// filled yet (a bond series' name, a fund's number): shown as such.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Search {
    pub text: Text,
    pub placeholder: bool,
}

/// Where to buy, from the visitor's bank (§2.9).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhereToBuy {
    pub title: Text,
    pub rows: Vec<Provider>,
    /// Until the visitor names a bank: the lowest and highest the banks'
    /// online plans leave.
    pub bank_range: Option<(Amount, Amount)>,
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
}

/// One row of "ואם התאריך יזוז?" (§2.9).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HorizonRow {
    pub years: u32,
    /// Which answer about the date this row is for, while the visitor
    /// hasn't given one and the two differ; `None` when it doesn't matter.
    pub flexibility: Option<Flexibility>,
    pub deposited: Amount,
    pub kind: Kind,
    pub name: Text,
    pub low: Amount,
    pub high: Amount,
    /// On every ₪1 put in: low and high.
    pub per_shekel: (Decimal, Decimal),
}

/// The answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Answer {
    pub confidence: Confidence,
    pub title: Text,
    pub body: Text,
    pub lines: Vec<Line>,
    /// "התוכנית שלכם", once every decision is made.
    pub plan: Option<Text>,
    pub open_decisions: u32,
    pub outcome: Outcome,
    /// The row "how to do it" shows, from the choices.
    pub focus: Option<RowId>,
    /// One for each row.
    pub how: Vec<How>,
    /// Only where fees or tax decide something on the page.
    pub gain: Option<Gain>,
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
}

fn place_of(answers: &Answers, overrides: Overrides) -> Place_ {
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
            Place_::Index { .. } => Some(Arena::Index),
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
        Place_::Safe(_) | Place_::Index { .. } => {}
    }
    (safe, index)
}

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
        Kind::GovernmentBond { .. } => SafeWay::Bond,
        Kind::FixedDeposit | Kind::LinkedDeposit => SafeWay::Deposit,
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

// ─────────────────────────── The index side ───────────────────────────

struct IndexSide<'a> {
    run: Run<'a>,
    /// The way the rules chose: the first that anyone can open.
    lead: usize,
    /// A study fund that leaves more than the lead by more than the margin.
    study: Option<usize>,
    /// The next way anyone can open, for how clear the lead is.
    next: Option<usize>,
    months: u32,
    level: Decimal,
    history: Option<DepositsBand>,
    /// Each way at the history's worst, median and best, in today's money.
    market: Vec<Option<(Decimal, Decimal, Decimal)>>,
    unlock: Option<super::UnlockLine>,
    also: Option<super::AlsoPossible>,
}

impl IndexSide<'_> {
    fn way(&self, at: usize) -> &WayOutcome {
        &self.run.ways[at]
    }

    fn left(&self, at: usize) -> Option<Amount> {
        self.way(at)
            .after_tax()
            .map(|nominal| super::safe::amount(nominal, self.level))
    }
}

fn index_side<'a>(
    index_answers: &Answers,
    settings: &Settings,
    data: &'a Data,
    deposited: Decimal,
) -> Option<IndexSide<'a>> {
    let overrides = Overrides {
        arena: Some(Arena::Index),
        kind: settings
            .overrides
            .kind
            .filter(|kind| kind.arena() == Some(Arena::Index)),
    };
    let decided = route::decide_choosing(index_answers, settings, overrides, data);
    let (run, lead) = decided.run?;
    let level = decided.level;
    let months = decided.months;
    let margin = kind_margin(deposited);
    let lead_left = run.ways[lead].after_tax()? / level;
    let study = run.ways.iter().position(|way| {
        way.way.vehicle == Vehicle::StudyFund
            && way
                .after_tax()
                .is_some_and(|left| left / level - lead_left > margin)
    });
    let next = run
        .ways
        .iter()
        .enumerate()
        .position(|(at, way)| at != lead && open_to_anyone(way.way) && way.best().is_some());
    let history = history::deposits_band(index_answers.now, index_answers.monthly, months);
    let base = robust::versions(settings, None)[0];
    // Only the rows the chart shows: the lead and a study fund.
    let market = run
        .ways
        .iter()
        .enumerate()
        .map(|(at, way)| {
            if at != lead && Some(at) != study {
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
            };
            Some((at(history.worst)?, at(history.median)?, at(history.best)?))
        })
        .collect();
    let unlock = run.unlock(&run.ways[lead], level, &settings.rates);
    let also = run.also_possible(&run.ways[lead], level, &settings.rates);
    Some(IndexSide {
        run,
        lead,
        study,
        next,
        months,
        level,
        history,
        market,
        unlock,
        also,
    })
}

// ─────────────────────────── Building it ───────────────────────────

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
    /// The months of the outcome chart.
    months: u32,
    deposited: Amount,
    current_account: Amount,
    kind_margin: Decimal,
    provider_margin: Decimal,
    safe: SafeSide,
    index: Option<IndexSide<'a>>,
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
    let facts = facts(answers, *choices, settings, data);
    let mut answer = build(&facts, horizons);
    if !horizons {
        answer.horizons.clear();
    }
    Ok(answer)
}

fn facts<'a>(
    answers: &'a Answers,
    choices: Choices,
    settings: &'a Settings,
    data: &'a Data,
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
    let (mut safe_answers, index_answers) = arena_answers(answers, place);
    let safe_months = |safe_answers: &Answers| match safe_answers.horizon {
        Horizon::Months(months) => months,
        Horizon::Retirement { .. } | Horizon::Unknown => route::index_years(&index_answers) * 12,
    };
    // A first guess of the deposits for the margins; the chart's are below.
    let (rough, _) = yardsticks(answers, safe_months(&safe_answers), settings.inflation);
    let index = index_side(&index_answers, settings, data, rough.today);
    // Where the rules put the money in the index, the data decides how sure
    // that is: if the index's worst stretch of the past, after fees and tax,
    // ends below the best safe way, safety or growth is the visitor's call.
    if place == (Place_::Index { chosen: false })
        && let Some(index) = &index
        && let Some((worst, _, _)) = index.market[index.lead]
    {
        let asked = Place_::Asked {
            no_date: answers.horizon == Horizon::Unknown,
        };
        let (safe_try, _) = arena_answers(answers, asked);
        let base = robust::versions(settings, None)[0];
        let best_safe = safe_pool(&safe_try)
            .into_iter()
            .filter_map(|kind| safe_value(kind, &safe_try, index.months, &base, data))
            .max();
        if best_safe.is_some_and(|best| worst < best) {
            place = asked;
            safe_answers = safe_try;
        }
    }
    let arena = place.arena(answers.flexibility);
    let safe_months = safe_months(&safe_answers);
    let months = match (arena, &index) {
        (Some(Arena::Index), Some(index)) => index.months,
        _ => safe_months,
    };
    let (deposited, current_account) = yardsticks(&safe_answers, months, settings.inflation);
    let history = index.as_ref().and_then(|index| index.history);
    let versions = robust::versions(settings, history.as_ref());
    let base = versions[0];
    let pool = safe_pool(&safe_answers);
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
    Facts {
        answers,
        choices,
        settings,
        data,
        place,
        arena: if index.is_none() && arena == Some(Arena::Index) {
            Some(Arena::Safe)
        } else {
            arena
        },
        safe_answers,
        bank,
        existing,
        months,
        deposited,
        current_account,
        kind_margin: kind_margin(deposited.today),
        provider_margin: provider_margin(deposited.today),
        safe,
        index,
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
        }
    }

    fn best(&self) -> Option<&Place> {
        self.places.first()
    }

    /// The best's tie group.
    fn tied(&self) -> Vec<&Place> {
        self.places
            .iter()
            .zip(&self.ties)
            .filter(|(_, tie)| **tie == 0)
            .map(|(place, _)| place)
            .take(3)
            .collect()
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

fn build(f: &Facts<'_>, with_horizons: bool) -> Answer {
    let mut lines: Vec<Line> = Vec::new();
    let safe_standing = safe_standing(f);
    // The cushion, before everything else, when the money goes to the index.
    let cushion = cushion_line(f);
    if let Some(line) = cushion {
        lines.push(line);
    }
    lines.push(place_line(f));
    let (way_line, chosen) = way_line(f, safe_standing);
    lines.push(way_line);
    let (where_line, where_info) = where_line(f, chosen);
    lines.push(where_line);
    let state = match (f.place, f.arena) {
        (Place_::Asked { .. }, _) => ConfidenceState::C,
        // Above the lines the data already chose: the worst past stretch
        // still beats the safe ways.
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
        .map(|row| {
            how_of(
                f,
                row,
                where_info.as_ref().filter(|info| Some(row.id) == info.row),
            )
        })
        .collect();
    let axis = axis(&rows, f.deposited, f.current_account);
    let outcome = Outcome {
        date: card::date(f.settings.today, f.months),
        title: card::outcome_title(f.settings.today, f.months),
        deposited: f.deposited,
        current_account: f.current_account,
        axis,
        rows,
    };
    let plan = (open_decisions == 0 && !waiting)
        .then(|| plan_text(f, chosen, where_info.as_ref()))
        .flatten();
    let (title, body) = title_and_body(f, state, open_decisions > 0 || waiting);
    let gain = gain(f, &lines, chosen, where_info.as_ref());
    Answer {
        confidence: Confidence {
            state,
            note: card::chip_note(state, f.arena, open_decisions > 0 || waiting),
        },
        title,
        body,
        lines,
        plan,
        open_decisions,
        outcome,
        focus,
        how,
        gain,
        horizons: if with_horizons {
            horizons(f)
        } else {
            Vec::new()
        },
        annuity: matches!(f.answers.horizon, Horizon::Retirement { .. })
            .then(card::annuity_placeholder),
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
/// owner's decision 4).
fn cushion_line(f: &Facts<'_>) -> Option<Line> {
    if f.arena != Some(Arena::Index)
        || f.answers.monthly <= Decimal::ZERO
        || f.answers.needs_any_day
    {
        return None;
    }
    let expenses = match f.answers.cushion {
        Some(Cushion::HasNot { monthly_expenses }) => (monthly_expenses, false),
        _ => (STAND_IN_MONTHLY_EXPENSES, true),
    };
    let target = expenses.0 * Decimal::from(CUSHION_MONTHS);
    let missing = (target - f.answers.now).max(Decimal::ZERO);
    // A sum now that is already the cushion leaves nothing to ask.
    if missing.is_zero() {
        return None;
    }
    let months_to_fill = (missing / f.answers.monthly).ceil();
    let words = card::cushion_decision(
        f.answers.monthly,
        target,
        months_to_fill,
        expenses.0,
        expenses.1,
    );
    let chosen = f.answers.cushion.map(|cushion| {
        Pick::Cushion(match cushion {
            Cushion::Has => CushionPick::Has,
            Cushion::HasNot { .. } => CushionPick::HasNot,
        })
    });
    let index_row = f
        .index
        .as_ref()
        .map(|index| Kind::Way(index.way(index.lead).way));
    let picks = [
        (
            Pick::Cushion(CushionPick::Has),
            index_row.into_iter().collect::<Vec<_>>(),
            None,
        ),
        (
            Pick::Cushion(CushionPick::HasNot),
            std::iter::once(Kind::MoneyMarketFund)
                .chain(index_row)
                .collect(),
            None,
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
        .map(|(words, (pick, rows, value))| Opt {
            pick: *pick,
            title: words.title,
            amount: words.amount,
            value: *value,
            facts: words.facts.into_iter().take(3).collect(),
            fits_if: words.fits_if,
            button: words.button,
            rows: rows.clone(),
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

fn place_line(f: &Facts<'_>) -> Line {
    match f.place {
        Place_::Safe(because) => line(
            LineKind::PlaceType,
            LineState::Stated {
                text: card::place_safe(because, f.months),
                pill: match because {
                    SafeBecause::AnyDay | SafeBecause::Short | SafeBecause::KnownDate => {
                        Pill::Clear
                    }
                    SafeBecause::Rules | SafeBecause::Chosen => Pill::Rules,
                },
            },
        ),
        Place_::Index { chosen } => line(
            LineKind::PlaceType,
            LineState::Stated {
                text: card::place_index(&card::PlaceIndexFigures {
                    months: f.months,
                    history: f.index.as_ref().and_then(|index| index.history),
                    worst: f
                        .index
                        .as_ref()
                        .and_then(|index| index.market[index.lead].map(|(worst, _, _)| worst)),
                    safe: f
                        .safe
                        .groups
                        .first()
                        .map(|group| (group.lead.kind, group.lead.left.today)),
                    deposited: f.deposited.today,
                }),
                pill: if chosen { Pill::Rules } else { Pill::Clear },
            },
        ),
        Place_::Asked { no_date } => {
            let safe_low = f
                .safe
                .groups
                .iter()
                .map(|group| group.lead.left)
                .min_by(|a, b| a.today.cmp(&b.today));
            let safe_high = f.safe.groups.first().map(|group| group.lead.left);
            let index = f.index.as_ref();
            let growth = index.and_then(|index| index.left(index.lead));
            let worst = index.and_then(|index| index.market[index.lead].map(|(worst, _, _)| worst));
            let brokerage =
                index.is_some_and(|index| index.way(index.lead).way.vehicle == Vehicle::Brokerage);
            let words = card::place_type_decision(&card::PlaceTypeFigures {
                no_date,
                today: f.settings.today,
                months: f.months,
                safe_low: safe_low.map(|amount| amount.today),
                safe_high: safe_high.map(|amount| amount.today),
                growth: growth.map(|amount| amount.today),
                index_return: f.settings.index_return,
                deposited: f.deposited.today,
                worst,
                history: index.and_then(|index| index.history),
                brokerage,
                safe_kinds: f.safe.groups.iter().map(|group| group.way).collect(),
            });
            let safe_rows: Vec<RowId> = f
                .safe
                .groups
                .iter()
                .flat_map(|group| {
                    std::iter::once(group.lead.kind).chain(group.variant.as_ref().map(|v| v.kind))
                })
                .collect();
            let index_rows: Vec<RowId> = index
                .map(|index| Kind::Way(index.way(index.lead).way))
                .into_iter()
                .collect();
            let picks = [
                (Pick::Flexibility(Flexibility::Fixed), safe_rows, safe_high),
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
    match f.arena {
        None => (line(LineKind::Way, LineState::Waiting), None),
        Some(Arena::Safe) => safe_way_line(f, safe_standing),
        Some(Arena::Index) => index_way_line(f),
    }
}

fn safe_way_line(f: &Facts<'_>, safe_standing: Standing) -> (Line, Option<Chosen>) {
    let groups = &f.safe.groups;
    let Some(first) = groups.first() else {
        return (line(LineKind::Way, LineState::Waiting), None);
    };
    if groups.len() < 2 || safe_standing == Standing::Clear {
        let text = card::safe_way_stated(first.way, first.lead.kind, f.months);
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
    let bond = f.safe.group(SafeWay::Bond);
    let bond_at_your_bank = bond.and_then(|bond| {
        let places = Where::new(
            super::outcome::safe_places(&bond.lead, f.data, f.bank),
            f.provider_margin,
        );
        places.yours.map(|at| places.places[at].left.today)
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
        rate: f.settings.rate,
        switch,
        bond_at_your_bank,
        flips: safe_standing != Standing::Tie,
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

/// The engine's split of a way's best plan.
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
    Some(split(&outcome, f.deposited.today))
}

/// What the איפה line found, for the parts below.
struct WhereInfo {
    row: Option<RowId>,
    places: Where,
    /// The account question, when it was asked or settled.
    account: Option<Account>,
    /// A decision about the account was shown.
    decided: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm for each way, each with its places"
)]
fn where_line(f: &Facts<'_>, chosen: Option<Chosen>) -> (Line, Option<WhereInfo>) {
    if f.existing && f.arena == Some(Arena::Index) {
        return holdings_line(f);
    }
    let Some(chosen) = chosen else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    match chosen {
        Chosen::Safe(SafeWay::MoneyMarketFund) => (
            line(
                LineKind::Where,
                LineState::Stated {
                    text: card::where_fund(),
                    pill: Pill::Clear,
                },
            ),
            None,
        ),
        Chosen::Safe(SafeWay::Deposit) => {
            let Some(group) = f.safe.group(SafeWay::Deposit) else {
                return (line(LineKind::Where, LineState::Waiting), None);
            };
            let places = Where::new(
                super::outcome::safe_places(&group.lead, f.data, f.bank),
                f.provider_margin,
            );
            let best = places.best().map_or_else(
                || group.lead.choice.provider.clone(),
                |place| place.provider.clone(),
            );
            let yours = places.yours.map(|at| places.places[at].left.today);
            let text =
                card::where_deposit(&best, places.best().map(|place| place.left.today), yours);
            (
                line(
                    LineKind::Where,
                    LineState::Stated {
                        text,
                        pill: Pill::Rules,
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
            match way.way.vehicle {
                Vehicle::Brokerage => {
                    let places = Where::new(
                        way_places(&index.run, way, f.bank, f.existing, index.level),
                        f.provider_margin,
                    );
                    account_line(f, row, places, false)
                }
                vehicle => {
                    let places = Where::new(
                        way_places(&index.run, way, f.bank, f.existing, index.level),
                        f.provider_margin,
                    );
                    (
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
                    )
                }
            }
        }
    }
}

/// "Open a new account, or buy through your bank?": a decision only when the
/// visitor's bank (or, until it's named, the best bank) is behind the best
/// by more than the providers' margin, and staying isn't dominated anyway.
fn account_line(
    f: &Facts<'_>,
    row: Option<RowId>,
    places: Where,
    moot: bool,
) -> (Line, Option<WhereInfo>) {
    let Some(best) = places.best() else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    let tied: Vec<Text> = places
        .tied()
        .iter()
        .map(|place| place.provider.clone())
        .collect();
    let spread = places
        .tied()
        .last()
        .map_or(Decimal::ZERO, |last| best.left.today - last.left.today);
    let reference = places.reference();
    let gap = reference.map(|reference| best.left.today - reference.today);
    let yours_name = places.yours.map(|at| places.places[at].provider.clone());
    let range = places.range.map(|(low, high)| (low.today, high.today));
    if moot || gap.is_none_or(|gap| gap <= f.provider_margin) {
        let text = card::where_stated(&tied, spread, gap, yours_name.as_ref(), range, moot);
        let info = WhereInfo {
            row,
            places,
            account: Some(Account::NewAccount),
            decided: false,
        };
        return (
            line(
                LineKind::Where,
                LineState::Stated {
                    text,
                    pill: Pill::Tie,
                },
            ),
            Some(info),
        );
    }
    let years = f.months.div_ceil(12);
    let best_cost = best.cost_a_year;
    let yours_cost = places.yours.and_then(|at| places.places[at].cost_a_year);
    let words = card::account_decision(&card::AccountFigures {
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

/// "What's in the portfolio?" for a portfolio already at a bank (the
/// owner's decision 8).
fn holdings_line(f: &Facts<'_>) -> (Line, Option<WhereInfo>) {
    let Some(index) = &f.index else {
        return (line(LineKind::Where, LineState::Waiting), None);
    };
    let way = index.way(index.lead);
    let places = Where::new(
        way_places(&index.run, way, f.bank, true, index.level),
        f.provider_margin,
    );
    let saves = places
        .best()
        .zip(places.reference())
        .map(|(best, yours)| best.left.today - yours.today);
    let tied: Vec<Text> = places
        .tied()
        .iter()
        .map(|place| place.provider.clone())
        .collect();
    let words = card::holdings_decision(saves, f.provider_margin, &tied, f.months);
    let row = Kind::Way(way.way);
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
            LineKind::Where,
            LineState::Decision(decision(
                DecisionId::Holdings,
                words,
                &picks,
                chosen.map(Pick::Holdings),
            )),
        ),
        Some(WhereInfo {
            row: Some(row),
            places,
            account: chosen.map(|holdings| match holdings {
                Holdings::IndexFunds => Account::NewAccount,
                Holdings::Managed | Holdings::DontKnow => Account::YourBank,
            }),
            decided: true,
        }),
    )
}

// ─────────────────────────── The chart ───────────────────────────

fn rows(f: &Facts<'_>, chosen: Option<Chosen>) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    let safe_active = |way: SafeWay| match (f.arena, chosen) {
        (Some(Arena::Safe), Some(Chosen::Safe(chosen))) => chosen == way,
        (None | Some(Arena::Safe), _) => true,
        (Some(Arena::Index), _) => {
            way == SafeWay::MoneyMarketFund
                && matches!(f.answers.cushion, Some(Cushion::HasNot { .. }))
        }
    };
    let fund_range = f
        .safe
        .group(SafeWay::MoneyMarketFund)
        .and_then(|fund| fund_range(f, fund));
    let cushion_months = (f.arena == Some(Arena::Index))
        .then(|| route::cushion_to_fill(f.answers).map(|(_, months)| months))
        .flatten();
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
    let lead_row = f
        .index
        .as_ref()
        .map(|index| index_row(f, index, index.lead, chosen));
    let study_row = f
        .index
        .as_ref()
        .and_then(|index| index.study.map(|study| index_row(f, index, study, chosen)));
    let safe_row = |priced: &Priced, way: SafeWay| Row {
        id: priced.kind,
        kind: priced.kind,
        name: row_words::name(priced.kind),
        tags: row_words::tags(priced.kind, f.arena),
        mark: match (priced.kind, fund_range) {
            (Kind::MoneyMarketFund, Some((low, high))) => Mark::RateRange { low, high },
            _ => Mark::Known,
        },
        value: priced.left,
        active: safe_active(way),
        excluded: None,
        note: match priced.kind {
            // A bar that's drawn says what it spans (principle 10).
            Kind::MoneyMarketFund => match (cushion_months, fund_range) {
                (Some(months), range) => Some(row_words::cushion_note(
                    months,
                    range.map(|_| f.settings.rate),
                )),
                (None, Some(_)) => Some(row_words::rate_note(f.settings.rate)),
                (None, None) => None,
            },
            _ => None,
        },
    };
    if let Some(Arena::Index) = f.arena {
        rows.extend(lead_row);
        rows.extend(study_row);
        rows.extend(safe_rows.iter().map(|(priced, way)| safe_row(priced, *way)));
    } else {
        rows.extend(
            safe_rows
                .iter()
                .take(4)
                .map(|(priced, way)| safe_row(priced, *way)),
        );
        rows.extend(lead_row);
    }
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
    let mark = match (market, index.history) {
        (Some((worst, median, best)), Some(history)) => Mark::Market {
            worst: at_end(worst, index.months, f.settings.inflation),
            median: at_end(median, index.months, f.settings.inflation),
            assumed,
            best: at_end(best, index.months, f.settings.inflation),
            losses: row_words::losses(&history, f.answers.monthly > Decimal::ZERO),
        },
        _ => Mark::Expected,
    };
    let active = excluded.is_none()
        && match (f.arena, chosen) {
            (Some(Arena::Index), Some(Chosen::Index(chosen))) => chosen == at,
            (None | Some(Arena::Index), _) => true,
            (Some(Arena::Safe), _) => false,
        };
    Row {
        id: kind,
        kind,
        name: row_words::name(kind),
        tags: row_words::tags(kind, f.arena),
        mark,
        value: assumed,
        active,
        excluded,
        note: None,
    }
}

fn focus_row(f: &Facts<'_>, chosen: Chosen) -> Option<RowId> {
    match chosen {
        Chosen::Safe(way) => f.safe.group(way).map(|group| group.lead.kind),
        Chosen::Index(at) => f.index.as_ref().map(|index| Kind::Way(index.way(at).way)),
    }
}

/// From a little below the lowest bad case to a little above the highest
/// typical value; an excluded row's range doesn't stretch it.
fn axis(rows: &[Row], deposited: Amount, current_account: Amount) -> (Decimal, Decimal) {
    let mut low = deposited.today.min(current_account.today);
    let mut high = deposited.today.max(current_account.today);
    for row in rows {
        low = low.min(row.value.today);
        high = high.max(row.value.today);
        if row.excluded.is_some() {
            continue;
        }
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
    ((low - pad).max(Decimal::ZERO), high + pad)
}

// ─────────────────────────── How ───────────────────────────

fn how_of(f: &Facts<'_>, row: &Row, info: Option<&WhereInfo>) -> How {
    let ctx = how_words::Context {
        kind: row.kind,
        now: f.answers.now,
        monthly: f.answers.monthly,
        months: f.months,
        today: f.settings.today,
        cushion: (f.arena == Some(Arena::Index))
            .then(|| route::cushion_to_fill(f.answers))
            .flatten(),
        account: info.and_then(|info| info.account),
    };
    How {
        row: row.id,
        name: row.name.clone(),
        identity: how_words::identity(&ctx),
        search: how_words::search(&ctx).map(|(text, placeholder)| Search { text, placeholder }),
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

fn where_to_buy(f: &Facts<'_>, row: &Row, info: Option<&WhereInfo>) -> WhereToBuy {
    let built: Option<Where> = match (info, row.kind) {
        (None, Kind::Way(way)) => f.index.as_ref().and_then(|index| {
            let at = index
                .run
                .ways
                .iter()
                .position(|outcome| outcome.way == way)?;
            Some(Where::new(
                way_places(&index.run, index.way(at), f.bank, f.existing, index.level),
                f.provider_margin,
            ))
        }),
        (None, kind @ (Kind::GovernmentBond { .. } | Kind::FixedDeposit)) => f
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
            title: card::where_title(false),
            rows: Vec::new(),
            bank_range: None,
            note: (row.kind == Kind::MoneyMarketFund).then(card::where_fund),
            asks_bank: false,
            unlock: None,
            may_cost_more: None,
        };
    };
    let yours = places.yours.map(|at| places.places[at].left);
    let mut shown: Vec<Provider> = Vec::new();
    for (place, tie) in places.places.iter().zip(&places.ties) {
        let keep = *tie <= 1 && shown.len() < 4 || place.yours;
        if !keep {
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
        });
    }
    let is_lead = f
        .index
        .as_ref()
        .is_some_and(|index| row.kind == Kind::Way(index.way(index.lead).way));
    let lead_index = f.index.as_ref().filter(|_| is_lead);
    let asks_bank = f.bank.is_none()
        && row.kind != Kind::MoneyMarketFund
        && places
            .best()
            .zip(places.range)
            .is_some_and(|(best, (low, _))| best.left.today - low.today > f.provider_margin);
    WhereToBuy {
        title: card::where_title(yours.is_some()),
        rows: shown,
        bank_range: if yours.is_none() { places.range } else { None },
        note: card::where_note(
            places.ties.iter().take_while(|tie| **tie == 0).count(),
            places
                .tied()
                .last()
                .zip(places.best())
                .map(|(last, best)| best.left.today - last.left.today),
        ),
        asks_bank,
        unlock: lead_index
            .and_then(|index| index.unlock.as_ref())
            .map(|unlock| {
                Text::owned(
                    crate::describe::reasons::unlock_text(unlock, crate::Lang::En),
                    crate::describe::reasons::unlock_text(unlock, crate::Lang::He),
                )
            }),
        may_cost_more: lead_index
            .and_then(|index| index.also.as_ref())
            .map(|also| {
                Text::owned(
                    crate::describe::answer::also_possible_line(also, crate::Lang::En),
                    crate::describe::answer::also_possible_line(also, crate::Lang::He),
                )
            }),
    }
}

// ─────────────────────────── The plan, the title ───────────────────────────

fn plan_text(f: &Facts<'_>, chosen: Option<Chosen>, info: Option<&WhereInfo>) -> Option<Text> {
    let chosen = chosen?;
    let cushion = (f.arena == Some(Arena::Index))
        .then(|| route::cushion_to_fill(f.answers))
        .flatten();
    let tied: Vec<Text> = info.map_or_default(|info| {
        info.places
            .tied()
            .iter()
            .map(|place| place.provider.clone())
            .collect()
    });
    let yours = info.and_then(|info| {
        info.places
            .yours
            .map(|at| info.places.places[at].provider.clone())
    });
    let gap = info.and_then(|info| {
        info.places
            .best()
            .zip(info.places.reference())
            .map(|(best, yours)| best.left.today - yours.today)
    });
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
        cushion,
        tied: &tied,
        account: info.and_then(|info| info.account),
        yours: yours.as_ref(),
        gap,
        holdings: if f.existing { f.choices.holdings } else { None },
    }))
}

fn title_and_body(f: &Facts<'_>, state: ConfidenceState, open: bool) -> (Text, Text) {
    let index = f.index.as_ref();
    let fund = f
        .safe
        .group(SafeWay::MoneyMarketFund)
        .map(|fund| fund.lead.left.today);
    card::title_and_body(&card::TitleFigures {
        state,
        arena: f.arena,
        asked: matches!(f.place, Place_::Asked { .. }),
        today: f.settings.today,
        months: f.months,
        fixed_date: matches!(f.answers.horizon, Horizon::Months(_)),
        index_left: index
            .and_then(|index| index.left(index.lead))
            .map(|amount| amount.today),
        index_return: f.settings.index_return,
        fund,
        current_account: f.current_account.today,
        deposited: f.deposited.today,
        safe_low: f
            .safe
            .groups
            .iter()
            .map(|group| group.lead.left.today)
            .min(),
        safe_high: f.safe.groups.first().map(|group| group.lead.left.today),
        safe_lead: f.safe.groups.first().map(|group| group.lead.kind),
        open,
        retirement: matches!(f.answers.horizon, Horizon::Retirement { .. }),
    })
}

// ─────────────────────────── The gain ───────────────────────────

/// Only where fees or tax decide something on the page: the account (fees)
/// or the study fund (tax).
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
    let mut rows: Vec<(Text, Split)> = Vec::new();
    if decided(DecisionId::Account) || decided(DecisionId::Holdings) {
        let info = info.filter(|info| info.decided)?;
        let Some(Chosen::Index(at)) = chosen else {
            return None;
        };
        let way = index.way(at);
        let best = info.places.best()?;
        let reference = info
            .places
            .yours
            .map(|at| &info.places.places[at])
            .or_else(|| {
                // Until the bank is named: the best bank's.
                info.places.places.iter().find(|place| place.bank.is_some())
            })?;
        for place in [best, reference] {
            let cand = &index.run.cands[place.cand?];
            let outcome = plan_outcome(cand, way.way, &index.run.base, base, rates)?;
            rows.push((place.provider.clone(), split(&outcome, f.deposited.today)));
        }
    } else if decided(DecisionId::Wrapper) {
        for at in [Some(index.lead), index.study] {
            let at = at?;
            rows.push((
                row_words::name(Kind::Way(index.way(at).way)),
                best_split(f, index, at)?,
            ));
        }
    } else {
        return None;
    }
    let amount = |today: Decimal| at_end(today, f.months, f.settings.inflation);
    let gain_rows: Vec<GainRow> = rows
        .iter()
        .map(|(name, split)| GainRow {
            name: name.clone(),
            kept: amount(split.kept),
            tax: amount(split.tax),
            fees: amount(split.fees),
            product: amount(split.product),
        })
        .collect();
    let [(better_name, better), (worse_name, worse)] = rows.as_slice() else {
        return None;
    };
    Some(Gain {
        lead: gain_words::lead(f.deposited.today),
        takeaway: gain_words::takeaway(better_name, part(better), worse_name, part(worse)),
        rows: gain_rows,
    })
}

fn part(split: &Split) -> gain_words::Part {
    gain_words::Part {
        kept: split.kept,
        tax: split.tax,
        fees: split.fees + split.product,
    }
}

// ─────────────────────────── The horizons ───────────────────────────

/// "What if the date moves?": the same money at 1, 3, 5, 10 and 20 years,
/// under the visitor's answer about the date, or both while it's unanswered
/// and the two differ.
fn horizons(f: &Facts<'_>) -> Vec<HorizonRow> {
    let flexibilities: Vec<Option<Flexibility>> = match f.answers.flexibility {
        Some(flexibility) => vec![Some(flexibility)],
        None => vec![Some(Flexibility::Fixed), Some(Flexibility::CanWait)],
    };
    let mut rows = Vec::new();
    for years in LADDER_YEARS {
        let months = years * 12;
        let mut seen: Vec<(Option<Flexibility>, Arena)> = Vec::new();
        for flexibility in &flexibilities {
            let answers = Answers {
                situation: None,
                horizon: Horizon::Months(months),
                flexibility: *flexibility,
                needs_any_day: false,
                cushion: Some(Cushion::Has),
                ..f.answers.clone()
            };
            let arena = route::natural_route(&answers).arena();
            if seen.iter().any(|(_, seen)| *seen == arena) {
                continue;
            }
            seen.push((*flexibility, arena));
            if let Some(row) = horizon_row(f, &answers, years, *flexibility) {
                rows.push(row);
            }
        }
        // One answer is enough where both lead to the same place.
        if seen.len() == 1
            && f.answers.flexibility.is_none()
            && let Some(last) = rows.last_mut()
        {
            last.flexibility = None;
        }
    }
    rows
}

fn horizon_row(
    f: &Facts<'_>,
    answers: &Answers,
    years: u32,
    flexibility: Option<Flexibility>,
) -> Option<HorizonRow> {
    let decided = route::decide_choosing(answers, f.settings, Overrides::default(), f.data);
    let months = decided.months;
    let (deposited, _) = yardsticks(answers, months, f.settings.inflation);
    let base = f.versions[0];
    let (low, high) = match decided.kind {
        Kind::MoneyMarketFund => {
            let at = |rate: Decimal| {
                let version = Version {
                    rate: crate::Percent(rate.max(Decimal::ZERO)),
                    ..base
                };
                safe_value(Kind::MoneyMarketFund, answers, months, &version, f.data)
                    .map(|today| at_end(today, months, base.inflation))
            };
            (
                at(base.rate.0 - robust::RATE_STEP).unwrap_or(decided.left),
                at(base.rate.0 + robust::RATE_STEP).unwrap_or(decided.left),
            )
        }
        Kind::Way(_) => {
            let (run, at) = decided.run.as_ref()?;
            match history::deposits_band(answers.now, answers.monthly, months) {
                Some(history) => {
                    let value = |rate| {
                        way_value(
                            run,
                            &run.ways[*at],
                            &Version {
                                varies: Varies::History,
                                index_return: rate,
                                ..base
                            },
                            &f.settings.rates,
                        )
                        .map(|today| at_end(today, months, base.inflation))
                    };
                    (value(history.worst)?, value(history.median)?)
                }
                None => (decided.left, decided.left),
            }
        }
        _ => (decided.left, decided.left),
    };
    let per = |amount: Amount| {
        if deposited.today.is_zero() {
            Decimal::ZERO
        } else {
            (amount.today / deposited.today).round_dp(2)
        }
    };
    Some(HorizonRow {
        years,
        flexibility,
        deposited,
        kind: decided.kind,
        name: row_words::name(decided.kind),
        low,
        high,
        per_shekel: (per(low), per(high)),
    })
}
