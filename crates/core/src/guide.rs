//! The guide: from a few answers about the money (what for, how much now and
//! a month, when) to what fits money like that, why, and what the other
//! choices would have come to. The rules are REDESIGN.md §3's table, one
//! [`route`] for each row; the numbers that draw their lines are the
//! constants below.
//!
//! The guide never invents a figure of its own. It asks the engines that
//! already exist: the safe kinds (a money market fund, a fixed-rate deposit,
//! a government bond held to maturity) through [`crate::short_term`], the
//! index kinds (an index fund or ETF through a broker, a provident fund for
//! investment, a study fund, a savings policy) through [`crate::ways`], and
//! the current account (money left as it is) from the deposits themselves.
//! Safe kinds and index kinds are never ranked against each other: each
//! arena is ranked by what is left after tax, and the other arena is only
//! drawn beside it, with its own risk ([`Safety`]).
//!
//! Every amount is shown in today's money, with the nominal figure beside it
//! ([`Amount`]).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::history::Band;
use crate::ways::Way;
use crate::{Broker, ExchangeRates, Lang, Named, Percent, Text};

mod index;
mod route;
mod safe;
mod series;

pub use route::{alternatives, ladder, recommend};
pub use series::{Alternatives, BandSeries, Column, ColumnId, KindSeries, columns_for};

// ─────────────────────────── The fixed numbers ───────────────────────────

/// A known date this far or less goes in the safe arena, whatever else.
pub const SAFE_ONLY_UP_TO_MONTHS: u32 = 12;
/// Up to here a known date is a short goal: a deposit, a money market fund
/// or a bond are compared.
pub const SHORT_GOAL_UP_TO_MONTHS: u32 = 36;
/// The question "can the date move?" is asked for goals more than this far...
pub const FLEXIBILITY_ASKED_ABOVE_MONTHS: u32 = 36;
/// ... up to and including this far (7 years).
pub const FLEXIBILITY_ASKED_UP_TO_MONTHS: u32 = 84;
/// With a date that can move, from here (5 years) the index kinds fit.
pub const INDEX_IF_FLEXIBLE_FROM_MONTHS: u32 = 60;
/// With a fixed date, up to here (7 years) the safe kinds fit; over it the
/// index kinds do.
pub const SAFE_IF_FIXED_UP_TO_MONTHS: u32 = 84;
/// How many months of expenses make a cushion. The words say "3–6"; the
/// plan takes the low end.
pub const CUSHION_MONTHS: u32 = 3;
/// The horizons of the ladder, in years.
pub const LADDER_YEARS: [u32; 5] = [1, 3, 5, 10, 20];
/// The years the index kind that can be opened any time is worked out for
/// when the saver has no date.
pub const REFERENCE_YEARS: u32 = 10;
/// The age the saver is taken to have when they didn't say: it only decides
/// when a provident fund's pension would open and the surtax's years.
pub const USUAL_AGE: u32 = 30;
/// What one share costs, in the exchange's currency, where it matters (an
/// ETF abroad is bought in whole shares): what the app's inputs start with.
pub const USUAL_SHARE_PRICE: Decimal = dec!(500);
/// The index's expected return a year, unless the saver says otherwise.
pub const USUAL_INDEX_RETURN: Percent = Percent(dec!(10));
/// The longest horizon the guide works out: the app's, 50 years.
pub const LONGEST_MONTHS: u32 = crate::short_term::MOST_MONTHS;

/// How the branch plans are named in the tariffs (the plan of an order given
/// to a banker or at a branch).
pub(crate) const BRANCH_PLAN: &str = "Branch or through a banker";

// ─────────────────────────── The answers ───────────────────────────

/// A situation that changes what the guide does beyond the money's own
/// figures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Situation {
    /// "I already have a portfolio at a bank" (כבר יש לי תיק בבנק): the
    /// bank's branch price list is compared beside the rest.
    ExistingPortfolio {
        /// `None` is "another bank": the branch plan that leaves the least
        /// of those listed stands in for it, and the answer says so.
        bank: Option<Bank>,
    },
    /// "I know what I want to buy" (אני יודע מה אני רוצה לקנות): the index
    /// kinds, every way side by side, as the workbench shows them.
    KnowsWhatToBuy,
}

/// The banks whose branch price list is modelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Bank {
    Leumi,
    MizrahiTefahot,
    OtsarHahayal,
}

impl Bank {
    /// The bank's short name in the tariffs, in English.
    pub(crate) fn short_name(self) -> &'static str {
        match self {
            Bank::Leumi => "Leumi",
            Bank::MizrahiTefahot => "Mizrahi",
            Bank::OtsarHahayal => "Otsar Hahayal",
        }
    }
}

/// How long the money is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Horizon {
    Months(u32),
    /// "I don't know yet" (עוד לא יודעים).
    Unknown,
    /// Saving for retirement: the saver's age today, and the age they stop
    /// working. The money is kept for the years between.
    Retirement {
        age: u32,
        retire_at: u32,
    },
}

/// Whether the date can move, asked of goals 3 to 7 years away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Flexibility {
    /// The date is set: a payment due on a day.
    Fixed,
    /// A year or two later would do: the saver can wait out a bad market.
    CanWait,
}

/// Whether there is money set aside for a rainy day, asked when there is no
/// date and money comes in every month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cushion {
    Has,
    HasNot {
        /// ₪ a month.
        monthly_expenses: Decimal,
    },
}

/// What the saver answered. Amounts in ₪.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answers {
    pub situation: Option<Situation>,
    /// Put in at the start.
    pub now: Decimal,
    /// Put in every month, the first too.
    pub monthly: Decimal,
    pub horizon: Horizon,
    pub flexibility: Option<Flexibility>,
    pub cushion: Option<Cushion>,
    /// The money may be needed on any day (a cushion): only a place that
    /// pays it out the next day fits.
    pub needs_any_day: bool,
}

/// Why the answers can't be worked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidAnswers {
    #[error("the horizon must be at least a month")]
    NoHorizon,
    #[error("the horizon is longer than the guide works out")]
    HorizonTooLong,
    #[error("nothing is put in")]
    NoMoney,
    #[error("an amount is negative")]
    NegativeAmount,
    #[error("the amounts are too large to work with")]
    TooLarge,
    #[error("monthly expenses must be more than nothing")]
    NoExpenses,
    #[error("retirement must be after today")]
    RetirementNotAhead,
}

impl InvalidAnswers {
    /// Why, in the user's words.
    #[must_use]
    pub fn text(self, lang: Lang) -> &'static str {
        match self {
            InvalidAnswers::NoHorizon => lang.pick(
                "The money has to be kept for at least a month.",
                "צריך לשמור את הכסף לפחות חודש.",
            ),
            InvalidAnswers::HorizonTooLong => lang.pick(
                "The calculator works out up to 50 years.",
                "המחשבון מחשב עד 50 שנה.",
            ),
            InvalidAnswers::NoMoney => lang.pick(
                "Enter a sum now, or an amount every month.",
                "כתבו סכום עכשיו, או סכום לכל חודש.",
            ),
            InvalidAnswers::NegativeAmount => {
                lang.pick("An amount can't be negative.", "סכום לא יכול להיות שלילי.")
            }
            InvalidAnswers::TooLarge => lang.pick(
                "The amounts are too large to work with.",
                "הסכומים גדולים מדי לחישוב.",
            ),
            InvalidAnswers::NoExpenses => lang.pick(
                "Enter what you spend in a month, to size the cushion.",
                "כתבו כמה אתם מוציאים בחודש, כדי לחשב את הכרית.",
            ),
            InvalidAnswers::RetirementNotAhead => lang.pick(
                "The age you retire at has to be after your age today.",
                "הגיל שבו פורשים צריך להיות אחרי הגיל היום.",
            ),
        }
    }
}

// ─────────────────────────── What the guide knows ───────────────────────────

/// The app's settings that the guide takes from the workbench: its defaults
/// are the app's own.
pub struct Settings {
    /// The index's expected return a year (default [`USUAL_INDEX_RETURN`]).
    pub index_return: Percent,
    /// How much prices rise a year (default 2%).
    pub inflation: Percent,
    /// The Bank of Israel's rate the saver expects (default
    /// [`crate::short_term::TODAYS_RATE`]).
    pub rate: Percent,
    pub rates: ExchangeRates,
    /// The plans ticked in the workbench. `None` is the plan a new customer
    /// usually gets at each broker, and a fund's average (the branch plan of
    /// the saver's bank is added to either, for the situation that says
    /// so).
    pub ticked: Option<Vec<PlanRef>>,
    pub overrides: Overrides,
}

impl Settings {
    /// The app's defaults, with today's exchange rates.
    #[must_use]
    pub fn new(rates: ExchangeRates) -> Self {
        Settings {
            index_return: USUAL_INDEX_RETURN,
            inflation: crate::simulation::USUAL_INFLATION,
            rate: crate::short_term::TODAYS_RATE,
            rates,
            ticked: None,
            overrides: Overrides::default(),
        }
    }
}

/// A plan, by its place in [`Data::brokers`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRef {
    pub broker: usize,
    pub plan: usize,
}

/// What the expert chose over the guide.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Overrides {
    /// The arena, whatever the rules say.
    pub arena: Option<Arena>,
    /// The kind, whichever the ranking puts first (its arena follows).
    pub kind: Option<Kind>,
}

/// The data the guide works on: built once (it is the tariffs and the
/// published rates), then lent to every call.
pub struct Data {
    /// The brokers, then the funds ([`crate::listed`]).
    pub brokers: Vec<Broker>,
    /// The safe kinds' places ([`crate::short_term::kinds`]).
    pub safe: Vec<crate::short_term::Kind>,
}

impl Data {
    /// Everything the app lists.
    #[must_use]
    pub fn listed() -> Self {
        Data {
            brokers: crate::listed(),
            safe: crate::short_term::kinds(),
        }
    }
}

// ─────────────────────────── The answer ───────────────────────────

/// The two arenas: what is ranked together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Arena {
    /// Money market funds, deposits, government bonds held to maturity.
    Safe,
    /// Index funds and ETFs through brokers, provident funds for investment,
    /// study funds, savings policies on an index track.
    Index,
}

/// What the money is put in, in the terms people meet in the bank's app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Kind {
    /// קרן כספית.
    MoneyMarketFund,
    /// פיקדון בריבית קבועה.
    FixedDeposit,
    /// פיקדון צמוד מדד. The Bank of Israel's data the app reads has no such
    /// rates (REDESIGN.md §4.4 was not built), so no answer names it yet.
    LinkedDeposit,
    /// אג״ח ממשלתי שקלי, or צמוד מדד, held to maturity.
    GovernmentBond { linked: bool },
    /// An index kind: the product, the account it sits in, where it's bought.
    Way(Way),
    /// The money left in a current account, earning nothing: the yardstick.
    CurrentAccount,
}

impl Kind {
    /// The arena it is ranked in; the current account is nobody's.
    #[must_use]
    pub fn arena(self) -> Option<Arena> {
        match self {
            Kind::MoneyMarketFund
            | Kind::FixedDeposit
            | Kind::LinkedDeposit
            | Kind::GovernmentBond { .. } => Some(Arena::Safe),
            Kind::Way(_) => Some(Arena::Index),
            Kind::CurrentAccount => None,
        }
    }

    /// How sure its end sum is. A bond linked to the index keeps its
    /// purchasing power rather than a sum, which is what "known in advance"
    /// means for the saver.
    #[must_use]
    pub fn safety(self) -> Safety {
        match self {
            Kind::MoneyMarketFund => Safety::RateChanges,
            Kind::FixedDeposit | Kind::GovernmentBond { linked: false } | Kind::CurrentAccount => {
                Safety::SumKnown
            }
            Kind::LinkedDeposit | Kind::GovernmentBond { linked: true } => Safety::RealValueKnown,
            Kind::Way(_) => Safety::CanFall,
        }
    }
}

impl Named for Kind {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Kind::MoneyMarketFund => lang.pick("Money market fund (קרן כספית)", "קרן כספית"),
            Kind::FixedDeposit => lang.pick(
                "Fixed-rate deposit (פיקדון בריבית קבועה)",
                "פיקדון בריבית קבועה",
            ),
            Kind::LinkedDeposit => {
                lang.pick("CPI-linked deposit (פיקדון צמוד מדד)", "פיקדון צמוד מדד")
            }
            Kind::GovernmentBond { linked: false } => {
                lang.pick("Government bond (אג״ח ממשלתי שקלי)", "אג״ח ממשלתי שקלי")
            }
            Kind::GovernmentBond { linked: true } => lang.pick(
                "CPI-linked government bond (אג״ח ממשלתי צמוד מדד)",
                "אג״ח ממשלתי צמוד מדד",
            ),
            Kind::Way(way) => way.name(lang),
            Kind::CurrentAccount => lang.pick("Current account (עו״ש)", "עו״ש, בלי להשקיע"),
        }
    }
}

impl Kind {
    /// Its name in both languages.
    #[must_use]
    pub fn text(self) -> Text {
        Text::new(self.name(Lang::En), self.name(Lang::He))
    }
}

/// How sure the end sum is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Safety {
    /// What it comes to is known when it's bought.
    SumKnown,
    /// Safe, but the rate it earns changes with the Bank of Israel's.
    RateChanges,
    /// Linked to the index: what it buys is known in advance, the sum in
    /// shekels is not.
    RealValueKnown,
    /// The market decides, and can be down on the day.
    CanFall,
}

/// An amount in ₪: in today's money, which is the one shown, and the
/// nominal figure beside it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    pub today: Decimal,
    pub nominal: Decimal,
}

/// Each circumstance of the saver's that a reason can come from, with the
/// numbers the reason's words need. Every [`Reason`] names the one that
/// produced it, so the page can say how the answers affected the choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::IntoStaticStr)]
pub enum Circumstance {
    /// The money may be needed on any day.
    NeedsAnyDay,
    /// A year or less.
    ShortHorizon { months: u32 },
    /// A goal 13 to 36 months away, on a date.
    KnownDate { months: u32 },
    /// A date that can't move, 3 to 7 years away.
    FixedDate { months: u32 },
    /// A date that can move: from 5 years the index kinds become reasonable.
    FlexibleDate { months: u32 },
    /// Seven years and more (or five with a date that can move).
    LongHorizon { months: u32 },
    /// No date yet.
    NoDate,
    /// One sum, no monthly deposits (a fixed-rate deposit takes only that).
    OneSum,
    /// Money comes in every month.
    MonthlyDeposits,
    /// The kind earns about the Bank of Israel's rate, as it goes.
    EarnsBankRate { rate: Percent },
    /// A fund's rate isn't promised for so long.
    RateNotPromised { rate: Percent },
    /// A deposit is locked until its term ends.
    DepositLocked { months: u32 },
    /// The end sum is known when the kind is bought.
    SumKnownInAdvance,
    /// A linked bond or deposit: its real value is known in advance, its
    /// sum in shekels is not.
    RealValueKnownInAdvance,
    /// A bond can be sold before it is repaid, at the day's price.
    BondCanBeSold,
    /// The rule for the index arena: if it's down on the day, you wait.
    IfItsDownYouWait { months: u32 },
    /// What the index did over stretches this long (the band's numbers come
    /// from the context).
    HistoryOfTheHorizon { years: Decimal },
    /// What safety would have cost over so long.
    WhatSafetyCosts,
    /// The cheapest way after tax, and what its price is a year.
    CheapestAfterTax { cost_a_year: Option<Percent> },
    /// Prices rise: a current account loses to them.
    PricesRise { inflation: Percent },
    /// Retirement: the saver's age today, and the age they retire at.
    Retirement { age: u32, retire_at: u32 },
    /// A provident fund for investment's pension is tax-free from 60.
    TaxFreePension { age_at_the_end: u32 },
    /// ... and the saver won't be 60 yet.
    PensionNeedsSixty { age_at_the_end: u32 },
    /// A provident fund for investment takes up to a ceiling each year.
    DepositCeiling { ceiling: Decimal },
    /// A study fund opens only for some savers (the self-employed, or
    /// through an employer).
    StudyFundNeedsEligibility,
    /// No cushion yet: the cushion comes first.
    NoCushion {
        monthly_expenses: Decimal,
        target: Decimal,
    },
    /// There is a cushion.
    HasCushion,
    /// The branch price list of the saver's bank is among those compared.
    ExistingPortfolio,
    /// The saver said what they want to buy: every way is compared.
    KnowsWhatToBuy,
    /// The saver chose this kind or arena themselves.
    ChosenByYou,
}

/// A line of the answer: why, with the circumstance it comes from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reason {
    pub text: Text,
    pub from: Circumstance,
}

/// What the words need to put a circumstance across: the answers and the
/// figures the guide arrived at. Owned, so the words can be written without
/// borrowing from the guide.
#[derive(Debug, Clone)]
pub struct Context {
    pub answers: Answers,
    /// The months the money is kept, as the guide worked them out (a
    /// retirement's years, a ladder rung's).
    pub months: u32,
    pub arena: Arena,
    /// What is recommended.
    pub kind: Kind,
    pub left: Amount,
    pub put_in: Amount,
    pub current_account: Amount,
    /// What the index did over stretches as long as the horizon, where the
    /// history covers it.
    pub band: Option<Band>,
    /// The Bank of Israel's rate, the index's expected return and prices'
    /// rise, as the guide used them.
    pub rate: Percent,
    pub index_return: Percent,
    pub inflation: Percent,
}

/// The place or plan chosen within a kind, by name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Choice {
    /// The bank, investment house or fund company.
    pub provider: Text,
    /// The plan, the fund, the track.
    pub plan: Text,
}

/// One line of a ranking.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ranked {
    pub kind: Kind,
    pub name: Text,
    pub left: Amount,
    /// What it loses a year to fees (and, for an index kind, to the product
    /// itself), as a fund's fee is stated.
    pub cost_a_year: Option<Percent>,
    /// Its price may be too low: marked ⚠, and a tie goes to the one without.
    pub may_cost_more: bool,
    /// Why, in words ("May cost more: …"), where `may_cost_more` is.
    pub warning: Option<Text>,
    /// Fees paid over the years (the index kinds' lines only).
    pub fees: Option<Amount>,
    /// Tax paid on the gain, where the engine states it.
    pub tax: Option<Amount>,
    /// The first deposit opens it.
    pub fits: bool,
    /// It is the saver's own bank's branch plan.
    pub yours: bool,
}

/// What a bigger first deposit would open.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnlockLine {
    /// ₪, the first deposit that opens it.
    pub first_deposit: Decimal,
    /// ₪, the first deposit the saver has now: both figures below include
    /// the difference, which the headline's figure doesn't.
    pub first_deposit_now: Decimal,
    pub plan: Choice,
    pub kind: Kind,
    /// What it leaves at that deposit.
    pub after_tax: Amount,
    /// What the recommended kind leaves at that same deposit.
    pub instead_of: Amount,
}

/// A kind the saver could have chosen and wasn't offered, drawn beside the
/// answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Alternative {
    pub kind: Kind,
    pub name: Text,
    pub arena: Option<Arena>,
    /// What it comes to by its own assumptions, when it can be worked out.
    pub left: Option<Amount>,
    pub safety: Safety,
    /// Why it isn't what fits, naming the circumstance.
    pub why_not: Reason,
    /// What the index did over the horizon: carried by index kinds.
    pub band: Option<Band>,
    /// Where its money would go, when it can be worked out.
    pub breakdown: Option<Breakdown>,
}

/// The first part of a two-part answer: the cushion first.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CushionPlan {
    /// ₪, three months of expenses.
    pub target: Decimal,
    /// How many months of saving fill it.
    pub months_to_fill: u32,
    /// What the monthly money goes to after that.
    pub then: Kind,
    /// The kinds that can be opened any time, ranked, at `then_years`: the
    /// first is `then`.
    pub then_ranked: Vec<Ranked>,
    /// The years `then_ranked` is worked out for.
    pub then_years: u32,
}

/// A question to ask now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum FollowUp {
    /// Can the date move?
    Flexibility,
    /// The saver's age and the age they retire at.
    AgeAndRetirement,
    /// Is there a cushion?
    Cushion,
    /// What are the monthly expenses?
    MonthlyExpenses,
}

/// How a step is done, in one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub title: String,
    pub text: String,
}

/// A step in both languages.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StepText {
    pub title: Text,
    pub text: Text,
}

/// Where every shekel went, in today's money.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Breakdown {
    pub put_in: Decimal,
    /// Fees to the broker and the manager.
    pub fees: Decimal,
    /// What the product holding the index took on its own (a fund's fee, the
    /// tax it lost on dividends): `fees + product` is every fee there was.
    pub product: Decimal,
    /// What a place kept of the rate (a bank paying less than the rate).
    pub kept_by_the_place: Decimal,
    pub tax: Decimal,
    pub left: Decimal,
}

/// The answer.
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// What the saver answered: the steps need the monthly amount.
    pub answers: Answers,
    pub arena: Arena,
    pub kind: Kind,
    /// The cheapest place or plan within the kind.
    pub choice: Choice,
    /// The arena's kinds, ranked by what is left after tax.
    pub ranked: Vec<Ranked>,
    /// The places or plans of the kind, ranked.
    pub plans: Vec<Ranked>,
    pub left: Amount,
    pub put_in: Amount,
    /// The same deposits left in a current account.
    pub current_account: Amount,
    pub breakdown: Breakdown,
    /// The months the money is kept.
    pub months: u32,
    pub reasons: Vec<Reason>,
    pub alternatives: Vec<Alternative>,
    /// What the index did over stretches as long as the horizon.
    pub band: Option<Band>,
    pub unlock: Option<UnlockLine>,
    pub steps: Vec<StepText>,
    pub follow_ups: Vec<FollowUp>,
    pub two_parts: Option<CushionPlan>,
}

// ─────────────────────────── The questions ───────────────────────────

/// The questions to show now, given what is answered so far. Nothing else is
/// ever asked: the date's flexibility for goals 3 to 7 years away, the ages
/// for retirement, the cushion and the month's expenses when there is no
/// date and money comes in every month.
#[must_use]
pub fn follow_ups(answers: &Answers) -> Vec<FollowUp> {
    let mut asked = Vec::new();
    match answers.horizon {
        Horizon::Months(months)
            if months > FLEXIBILITY_ASKED_ABOVE_MONTHS
                && months <= FLEXIBILITY_ASKED_UP_TO_MONTHS =>
        {
            asked.push(FollowUp::Flexibility);
        }
        Horizon::Retirement { .. } => asked.push(FollowUp::AgeAndRetirement),
        Horizon::Unknown if answers.monthly > Decimal::ZERO => {
            asked.push(FollowUp::Cushion);
            if matches!(answers.cushion, Some(Cushion::HasNot { .. })) {
                asked.push(FollowUp::MonthlyExpenses);
            }
        }
        Horizon::Months(_) | Horizon::Unknown => {}
    }
    asked
}

/// Whether `answers` can be worked on.
///
/// # Errors
///
/// The first thing wrong with them.
pub fn check(answers: &Answers) -> Result<(), InvalidAnswers> {
    if answers.now < Decimal::ZERO || answers.monthly < Decimal::ZERO {
        return Err(InvalidAnswers::NegativeAmount);
    }
    let months = match answers.horizon {
        Horizon::Months(months) => {
            if months == 0 {
                return Err(InvalidAnswers::NoHorizon);
            }
            months
        }
        Horizon::Unknown => 1,
        Horizon::Retirement { age, retire_at } => {
            if retire_at <= age {
                return Err(InvalidAnswers::RetirementNotAhead);
            }
            (retire_at - age) * 12
        }
    };
    if months > LONGEST_MONTHS {
        return Err(InvalidAnswers::HorizonTooLong);
    }
    let largest = dec!(1_000_000_000_000);
    if answers.now > largest || answers.monthly > largest {
        return Err(InvalidAnswers::TooLarge);
    }
    if answers.now.is_zero() && answers.monthly.is_zero() {
        return Err(InvalidAnswers::NoMoney);
    }
    if let Some(Cushion::HasNot { monthly_expenses }) = answers.cushion
        && monthly_expenses <= Decimal::ZERO
    {
        return Err(InvalidAnswers::NoExpenses);
    }
    Ok(())
}

/// One rung of the ladder: what the same money does if it's needed after
/// this long.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rung {
    /// The horizon. For the rung "any day" it is the first year, which is
    /// how long the money market fund's figure is worked out for.
    pub months: u32,
    /// The money can be needed on any day (set for a sum and no monthly
    /// amount).
    pub any_day: bool,
    pub arena: Arena,
    pub kind: Kind,
    pub choice: Choice,
    pub left: Amount,
    pub safety: Safety,
}
