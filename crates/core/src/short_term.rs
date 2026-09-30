//! Money kept for the short term: a sum put somewhere safe for some months,
//! until it's needed. Wherever it's kept it earns about the Bank of Israel's
//! rate; what differs is how much of that reaches the saver, the tax on it,
//! and when the money can come out. The long term, where money rides a
//! market, is [`crate::simulation`].
//!
//! Two kinds of place are compared: a bank's fixed-rate deposit
//! ([`deposits`]), at the rates the Bank of Israel publishes for each bank,
//! and a money market fund ([`money_market`]), at the fees the funds report.
//! `policies/sources.md` says where each number comes from, and
//! `policies/not-modeled.md` what's left out.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use strum::{EnumCount, IntoEnumIterator};

use time::Date;

use crate::{Caveat, Lang, Named, Page, Percent, TariffDate, Text};

pub mod deposits;
pub mod money_market;

/// The Bank of Israel's rate when the numbers were checked, since 1
/// September 2026: what the saver is taken to expect unless they say
/// otherwise.
pub const TODAYS_RATE: Percent = Percent(dec!(3.25));

/// The pages behind [`TODAYS_RATE`]: the cut to it, and the one before, to
/// 3.5%, which the deposits' August rates were set at.
#[must_use]
pub fn todays_rate_sources() -> Vec<Page> {
    let page = |en: &'static str, he: &'static str, url: &str| Page {
        name: Text::new(en, he),
        url: url.to_owned(),
    };
    vec![
        page(
            "ynet on the cut to 3.25% (1 September 2026)",
            "ynet על הורדת הריבית ל-3.25% (1 בספטמבר 2026)",
            "https://www.ynet.co.il/economy/article/cdo59v3t1",
        ),
        page(
            "Bizportal on the cut to 3.25%",
            "ביזפורטל על הורדת הריבית ל-3.25%",
            "https://www.bizportal.co.il/general/news/article/20040979",
        ),
        page(
            "TheMarker on the cut to 3.5% (6 July 2026)",
            "TheMarker על הורדת הריבית ל-3.5% (6 ביולי 2026)",
            "https://www.themarker.com/news/macroeconomics/2026-07-06/ty-article-live/0000019f-376f-d22f-a9ff-f77f73af0000",
        ),
    ]
}

/// The longest the money can be kept, in months: the Bank of Israel
/// publishes deposit rates for terms of up to five years.
pub const LONGEST: u32 = 60;

/// The most that can be kept, in ₪: far more than anyone keeps on deposit,
/// and far enough below the largest `Decimal` that growing it can't
/// overflow.
const LARGEST_AMOUNT: Decimal = dec!(1_000_000_000_000);

/// Every kind of place the app compares, in the order to offer them.
#[must_use]
pub fn kinds() -> Vec<Kind> {
    vec![money_market::kind(), deposits::kind()]
}

// ─────────────────────────── The saver's inputs ───────────────────────────

/// A sum put away at the start, perhaps more every month, for some months,
/// and the rate the saver expects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    /// ₪, put in at the start.
    pub first_deposit: Decimal,
    /// ₪, put in at the start of every month, the first too. A fixed-rate
    /// deposit takes none: it's one sum.
    pub monthly_deposit: Decimal,
    /// How long it's kept, from 1 to [`LONGEST`] months.
    pub months: u32,
    /// The Bank of Israel's rate, on average over the months, as the saver
    /// expects it: what a money market fund earns before its fee. A
    /// deposit's rate is fixed when it's opened, whatever the rate does.
    pub rate: Percent,
    /// How much prices rise a year: a fund's gain is taxed beyond it.
    pub inflation: Percent,
}

/// Why a [`Scenario`] can't be worked out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidScenario {
    #[error("nothing is put in")]
    NoAmount,
    #[error("an amount is negative")]
    NegativeAmount,
    #[error("the amounts are too large to work with")]
    TooLarge,
    #[error("the months must be from 1 to {LONGEST}")]
    Months,
    #[error("the rate and inflation can't be −100% or less")]
    RateBelowMinus100,
}

impl InvalidScenario {
    /// What the user is told, in `lang`.
    #[must_use]
    pub fn text(self, lang: Lang) -> &'static str {
        match self {
            InvalidScenario::NoAmount => lang.pick(
                "put something in, at the start or every month",
                "צריך להפקיד משהו, בהתחלה או כל חודש",
            ),
            InvalidScenario::NegativeAmount => lang.pick(
                "the amounts can't be negative",
                "הסכומים לא יכולים להיות שליליים",
            ),
            InvalidScenario::TooLarge => lang.pick(
                "the amounts are too large to calculate",
                "הסכומים גדולים מדי בשביל החישוב",
            ),
            InvalidScenario::Months => lang.pick(
                "the money can be kept for 1 to 60 months",
                "אפשר לחסוך לתקופה של חודש עד 60 חודשים",
            ),
            InvalidScenario::RateBelowMinus100 => lang.pick(
                "the rate and inflation can't be −100% or less",
                "הריבית והאינפלציה לא יכולות להיות −100% או פחות",
            ),
        }
    }
}

impl Scenario {
    /// Whether it can be worked out: something put in and nothing absurd, a
    /// period the figures cover, and rates above −100%.
    ///
    /// # Errors
    ///
    /// The first thing wrong with it.
    pub fn check(&self) -> Result<(), InvalidScenario> {
        if self.first_deposit < Decimal::ZERO || self.monthly_deposit < Decimal::ZERO {
            return Err(InvalidScenario::NegativeAmount);
        }
        if !(1..=LONGEST).contains(&self.months) {
            return Err(InvalidScenario::Months);
        }
        let deposited = self
            .monthly_deposit
            .checked_mul(Decimal::from(self.months))
            .and_then(|monthly| monthly.checked_add(self.first_deposit))
            .ok_or(InvalidScenario::TooLarge)?;
        if deposited.is_zero() {
            return Err(InvalidScenario::NoAmount);
        }
        if deposited > LARGEST_AMOUNT {
            return Err(InvalidScenario::TooLarge);
        }
        let above_minus_100 = |rate: Percent| rate.0 > -Decimal::ONE_HUNDRED;
        if !above_minus_100(self.rate) || !above_minus_100(self.inflation) {
            return Err(InvalidScenario::RateBelowMinus100);
        }
        Ok(())
    }

    /// Every shekel put in, in ₪.
    #[must_use]
    pub fn deposited(&self) -> Decimal {
        self.first_deposit + self.monthly_deposit * Decimal::from(self.months)
    }

    /// What's put in at the start of each month, by month (0 is the first),
    /// the one-time sum with the first month's.
    fn deposits(&self) -> impl Iterator<Item = (u32, Decimal)> + '_ {
        (0..self.months).map(|month| {
            let first = if month == 0 {
                self.first_deposit
            } else {
                Decimal::ZERO
            };
            (month, first + self.monthly_deposit)
        })
    }

    /// What the money would be worth at the start (index 0) and after each
    /// month at the Bank of Israel's rate, with nothing kept and no tax.
    #[must_use]
    pub fn at_the_rate(&self) -> Vec<Decimal> {
        self.grown(monthly(self.rate, Percent(Decimal::ZERO)))
    }

    /// What's put in at the start (index 0) and what it's worth after each
    /// month, every deposit growing by `factor` a month from its month on.
    fn grown(&self, factor: Decimal) -> Vec<Decimal> {
        let start = self.first_deposit + self.monthly_deposit;
        // Each month: what was there, with the month's deposit, grows a month.
        let month_ends = self.deposits().scan(Decimal::ZERO, |value, (_, deposit)| {
            *value = (*value + deposit) * factor;
            Some(*value)
        });
        std::iter::once(start).chain(month_ends).collect()
    }

    /// How much of what's earned only kept up with prices: each deposit
    /// raised by inflation from its month to the end, less the deposits.
    fn kept_up_with_prices(&self) -> Decimal {
        let prices = yearly(self.inflation);
        self.deposits()
            .map(|(month, deposit)| {
                deposit * (years_of(prices, self.months - month) - Decimal::ONE)
            })
            .sum()
    }
}

// ─────────────────────────── Where the money is kept ───────────────────────────

/// A kind of place, and each one of it the app lists: the banks' fixed-rate
/// deposits, or the money market funds. What a [`crate::Broker`] is to its
/// plans.
#[derive(Debug, Clone, PartialEq)]
pub struct Kind {
    /// "Fixed-rate deposit"
    pub name: Text,
    /// What it is and how it works, in plain words.
    pub description: Text,
    /// When the figures are from.
    pub data_of: TariffDate,
    /// When they were last taken from their source.
    pub checked: Date,
    /// Where the figures can be seen.
    pub source: Page,
    /// Like a place's caveats, for ones that apply to every place of the kind.
    pub caveats: Vec<Caveat>,
    pub places: Vec<Place>,
}

/// One place to keep the money: a bank's deposit, or a money market fund at
/// a fee.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    /// "Bank Hapoalim", "Average fund"
    pub name: Text,
    /// Its name beside its kind's, where names repeat: "Hapoalim".
    pub short_name: Text,
    /// Who it is and on what terms, in plain words: not what it pays, which
    /// is shown for the saver's months.
    pub description: Text,
    pub pays: Pays,
    /// Assumptions and gaps behind its numbers, and what they rest on.
    pub caveats: Vec<Caveat>,
    /// Whether it's ticked when the calculator opens: the averages, the
    /// cheapest fund and the five big banks, where most people bank.
    pub compared_at_first: bool,
}

/// A deposit at a rate the saver typed, such as one a bank offered them:
/// the same rate for any term, taxed and locked like the banks'.
#[must_use]
pub fn your_deposit(rate: Percent) -> Place {
    Place {
        name: Text::new("Your deposit", "הפיקדון שלכם"),
        short_name: Text::new("Your deposit", "הפיקדון שלכם"),
        description: Text::new(
            "A fixed-rate deposit at the rate you typed: what a bank offered you, say.",
            "פיקדון בריבית קבועה, בריבית שהקלדתם: למשל מה שבנק הציע לכם.",
        ),
        pays: Pays::Fixed(Rates::new([Some(rate); Term::COUNT])),
        caveats: vec![],
        compared_at_first: true,
    }
}

/// What a place pays on the money, which also decides the tax and when the
/// money can come out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pays {
    /// A bank's deposit: a yearly rate fixed when it's opened, for the whole
    /// term, by how long the term is.
    Fixed(Rates),
    /// A money market fund: the Bank of Israel's rate as it goes, less the
    /// fund's yearly fee.
    TheRateLess(Percent),
}

impl Pays {
    /// How the law taxes what it earns.
    #[must_use]
    pub fn tax(self) -> Tax {
        match self {
            Pays::Fixed(_) => Tax::OfInterest,
            Pays::TheRateLess(_) => Tax::OfRealGain,
        }
    }

    /// When the money can come out without losing what it earned.
    #[must_use]
    pub fn liquidity(self) -> Liquidity {
        match self {
            Pays::Fixed(_) => Liquidity::AtTheEnd,
            Pays::TheRateLess(_) => Liquidity::AnyDay,
        }
    }
}

/// A deposit's term, as the Bank of Israel groups deposits in its figures:
/// each takes the terms longer than the one before it, up to its own
/// ([`Term::for_months`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter, strum::EnumCount)]
pub enum Term {
    UpToAMonth,
    UpToThreeMonths,
    UpToSixMonths,
    UpToAYear,
    UpToTwoYears,
    UpToThreeYears,
    UpToFiveYears,
}

impl Term {
    /// The longest deposit it takes, in months.
    #[must_use]
    pub fn longest(self) -> u32 {
        match self {
            Term::UpToAMonth => 1,
            Term::UpToThreeMonths => 3,
            Term::UpToSixMonths => 6,
            Term::UpToAYear => 12,
            Term::UpToTwoYears => 24,
            Term::UpToThreeYears => 36,
            Term::UpToFiveYears => LONGEST,
        }
    }

    /// The term a deposit for `months` falls in: the first that's long
    /// enough. None past five years.
    #[must_use]
    pub fn for_months(months: u32) -> Option<Term> {
        Term::iter().find(|term| months <= term.longest())
    }
}

/// In the Bank of Israel's words.
impl Named for Term {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Term::UpToAMonth => lang.pick("Up to a month", "עד חודש"),
            Term::UpToThreeMonths => lang.pick("1 to 3 months", "חודש עד 3 חודשים"),
            Term::UpToSixMonths => lang.pick("3 to 6 months", "3 חודשים עד 6 חודשים"),
            Term::UpToAYear => lang.pick("6 months to a year", "6 חודשים עד שנה"),
            Term::UpToTwoYears => lang.pick("1 to 2 years", "שנה עד שנתיים"),
            Term::UpToThreeYears => lang.pick("2 to 3 years", "שנתיים עד 3 שנים"),
            Term::UpToFiveYears => lang.pick("3 to 5 years", "3 שנים עד 5 שנים"),
        }
    }
}

/// A bank's average yearly rate on fixed-rate deposits for each term, the
/// shortest first. None where the Bank of Israel published none: too few
/// deposits of that term in the month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rates([Option<Percent>; Term::COUNT]);

impl Rates {
    /// The rates, one for each [`Term`], the shortest first.
    #[must_use]
    pub const fn new(by_term: [Option<Percent>; Term::COUNT]) -> Self {
        Rates(by_term)
    }

    /// The rate for deposits of `term`, if one was published.
    #[must_use]
    pub fn of(self, term: Term) -> Option<Percent> {
        self.0[term as usize]
    }
}

/// How the Income Tax Ordinance taxes what the money earned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tax {
    /// [`Tax::ON_INTEREST`] of all the interest, as on anything not linked
    /// to the price index (section 125ג(ג)(1)). The bank withholds it when
    /// it pays.
    OfInterest,
    /// [`Tax::ON_REAL_GAIN`] of the gain beyond inflation, on selling, as
    /// for any security (section 91(ב)(1)). A gain that only keeps up with
    /// prices isn't taxed.
    OfRealGain,
}

impl Tax {
    pub const ON_INTEREST: Percent = Percent(dec!(15));
    pub const ON_REAL_GAIN: Percent = Percent(dec!(25));

    /// The tax on `earned`, of which `kept_up` only kept up with prices.
    /// Nothing is paid on a loss.
    #[must_use]
    pub fn on(self, earned: Decimal, kept_up: Decimal) -> Decimal {
        match self {
            Tax::OfInterest => Tax::ON_INTEREST.of(earned.max(Decimal::ZERO)),
            Tax::OfRealGain => Tax::ON_REAL_GAIN.of((earned - kept_up).max(Decimal::ZERO)),
        }
    }
}

/// "15% of all the interest"
impl Named for Tax {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Tax::OfInterest => lang.pick("15% of all the interest", "15% מכל הריבית"),
            Tax::OfRealGain => lang.pick(
                "25% of the gain beyond inflation",
                "25% מהרווח שמעבר לאינפלציה",
            ),
        }
    }
}

/// When the money can come out without losing what it earned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Liquidity {
    /// Any business day: a fund's units are sold at the day's price.
    AnyDay,
    /// At the end of the term: a fixed-rate deposit is locked until then.
    AtTheEnd,
}

impl Named for Liquidity {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Liquidity::AnyDay => lang.pick("Any day", "בכל יום"),
            Liquidity::AtTheEnd => lang.pick("At the end of the term", "בסוף התקופה"),
        }
    }
}

// ─────────────────────────── What it comes to ───────────────────────────

/// What keeping the money in one place comes to. Amounts in ₪.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// What it's worth at the start (index 0) and after each month, before
    /// tax: a deposit's interest as it builds up.
    pub value_by_month: Vec<Decimal>,
    /// What it earned before tax.
    pub earned: Decimal,
    /// The tax on that.
    pub tax: Decimal,
    /// What's left at the end, once taxed.
    pub after_tax: Decimal,
    /// What the place kept of what the money would earn at the Bank of
    /// Israel's rate: a fund's fee, or what a bank pays below the rate.
    /// Negative where a bank pays more than the rate.
    pub kept: Decimal,
    /// That as a yearly charge on the money, the way a fund's fee is: at the
    /// rate less this, the money ends where the place leaves it. Negative
    /// where a bank pays more than the rate.
    pub yearly_cost: Percent,
    /// What the money earned after tax, as a rate a year.
    pub yearly_after_tax: Percent,
}

/// Why a place can't be used for a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotOffered {
    /// A fixed-rate deposit is one sum. Saving every month at a bank is a
    /// savings plan (תוכנית חיסכון), whose rates the Bank of Israel doesn't
    /// publish.
    #[error("a fixed-rate deposit takes one sum, not monthly deposits")]
    TakesOneSum,
    /// The bank published no rate for deposits of this term: too few were
    /// opened in the month.
    #[error("no rate was published for deposits of this term")]
    NoRate { term: Term },
}

/// Keeps `scenario`'s money in `place`, or says why it can't be kept there.
/// Check the scenario first ([`Scenario::check`]).
///
/// # Errors
///
/// A deposit, when money is put in every month or its bank published no
/// rate for the scenario's term.
///
/// # Panics
///
/// If the scenario's months are past [`LONGEST`]: check it first.
pub fn simulate(place: &Place, scenario: &Scenario) -> Result<Outcome, NotOffered> {
    let value_by_month = match place.pays {
        Pays::Fixed(rates) => {
            if !scenario.monthly_deposit.is_zero() {
                return Err(NotOffered::TakesOneSum);
            }
            let term = Term::for_months(scenario.months).expect("a checked scenario");
            let rate = rates.of(term).ok_or(NotOffered::NoRate { term })?;
            (0..=scenario.months)
                .map(|month| deposit_worth(scenario.first_deposit, rate, month))
                .collect()
        }
        Pays::TheRateLess(fee) => scenario.grown(monthly(scenario.rate, fee)),
    };
    let end = *value_by_month.last().expect("the start is always there");
    let at_the_rate = *scenario
        .at_the_rate()
        .last()
        .expect("the start is always there");
    let earned = end - scenario.deposited();
    let tax = place.pays.tax().on(earned, scenario.kept_up_with_prices());
    let after_tax = end - tax;
    // Grown at the rate less the yearly cost, the deposits end at `end`.
    let yearly_cost = Decimal::ONE - yearly_factor_ending_at(scenario, end) / yearly(scenario.rate);
    Ok(Outcome {
        value_by_month,
        earned,
        tax,
        after_tax,
        kept: at_the_rate - end,
        yearly_cost: as_percent(yearly_cost),
        yearly_after_tax: as_percent(yearly_factor_ending_at(scenario, after_tax) - Decimal::ONE),
    })
}

/// Every compared place's outcome, best first, and what the money would come
/// to at the Bank of Israel's rate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// What the money would be worth after each month at the Bank of
    /// Israel's rate, with nothing kept and no tax.
    pub at_the_rate: Vec<Decimal>,
    pub places: Vec<PlaceOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOutcome {
    /// Where the place is in the list given to [`compare`].
    pub index: usize,
    /// Or why the money can't be kept there.
    pub outcome: Result<Outcome, NotOffered>,
}

/// Keeps `scenario`'s money in each of `places`, sorted by what's left after
/// tax, most first. Places that can't be used for it go last.
#[must_use]
pub fn compare(places: &[&Place], scenario: &Scenario) -> Comparison {
    let mut outcomes: Vec<PlaceOutcome> = places
        .iter()
        .enumerate()
        .map(|(index, place)| PlaceOutcome {
            index,
            outcome: simulate(place, scenario),
        })
        .collect();
    outcomes.sort_by_key(|place| {
        let left = place.outcome.as_ref().ok().map(|outcome| outcome.after_tax);
        std::cmp::Reverse(left)
    });
    Comparison {
        at_the_rate: scenario.at_the_rate(),
        places: outcomes,
    }
}

// ─────────────────────────── The arithmetic ───────────────────────────

/// 1 + `rate`: what a year multiplies money by.
fn yearly(rate: Percent) -> Decimal {
    Decimal::ONE + rate.of(Decimal::ONE)
}

/// The monthly factor that compounds, over twelve months, to growing at
/// `rate` less a yearly `fee`: (1 + rate) × (1 − fee). Worked out in
/// floating point, as `simulation`'s is.
fn monthly(rate: Percent, fee: Percent) -> Decimal {
    let year = yearly(rate) * (Decimal::ONE - fee.of(Decimal::ONE));
    let factor = year.to_f64().unwrap_or(1.0).powf(1.0 / 12.0);
    Decimal::try_from(factor).unwrap_or(Decimal::ONE)
}

/// What a deposit of `amount` at `rate` a year is worth after `month`
/// months. Part of a year earns a share of the year's interest; a whole
/// year's interest joins the deposit and earns interest too.
fn deposit_worth(amount: Decimal, rate: Percent, month: u32) -> Decimal {
    let whole_years = (0..month / 12).fold(amount, |value, _| value * yearly(rate));
    whole_years * (Decimal::ONE + rate.of(Decimal::from(month % 12)) / Decimal::from(12))
}

/// What a year multiplies by, raised to a period of `months`.
fn years_of(factor: Decimal, months: u32) -> Decimal {
    let power = factor
        .to_f64()
        .unwrap_or(1.0)
        .powf(f64::from(months) / 12.0);
    Decimal::try_from(power).unwrap_or(Decimal::ONE)
}

/// The yearly factor at which the scenario's deposits, each growing from
/// its month to the end, end at `target`: found by halving, since the end
/// grows with the factor. In floating point, like `simulation`'s yearly
/// cost.
fn yearly_factor_ending_at(scenario: &Scenario, target: Decimal) -> Decimal {
    let number = |value: Decimal| value.to_f64().unwrap_or(0.0);
    let deposits: Vec<f64> = scenario
        .deposits()
        .map(|(_, deposit)| number(deposit))
        .collect();
    let ends_with = |yearly: f64| {
        let monthly = yearly.powf(1.0 / 12.0);
        // Horner's scheme: each deposit grows for the months after it.
        deposits
            .iter()
            .fold(0.0, |value, deposit| (value + deposit) * monthly)
    };
    let target = number(target);
    // From losing everything to tripling in a year: far past any place's.
    let (mut low, mut high) = (0.0_f64, 3.0_f64);
    for _ in 0..64 {
        let middle = f64::midpoint(low, high);
        if ends_with(middle) < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    Decimal::try_from(f64::midpoint(low, high)).unwrap_or(Decimal::ONE)
}

/// A fraction as a percentage, to four places: 0.0169 is 1.69%.
fn as_percent(fraction: Decimal) -> Percent {
    Percent((fraction * Decimal::ONE_HUNDRED).round_dp(4))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario(amount: Decimal, months: u32, rate: Decimal) -> Scenario {
        Scenario {
            first_deposit: amount,
            monthly_deposit: Decimal::ZERO,
            months,
            rate: Percent(rate),
            inflation: Percent(dec!(2)),
        }
    }

    fn place(pays: Pays) -> Place {
        Place {
            name: Text::same("Test"),
            short_name: Text::same("Test"),
            description: Text::same(""),
            pays,
            caveats: vec![],
            compared_at_first: true,
        }
    }

    fn deposit(rate: Decimal) -> Place {
        place(Pays::Fixed(Rates::new([Some(Percent(rate)); Term::COUNT])))
    }

    fn fund(fee: Decimal) -> Place {
        place(Pays::TheRateLess(Percent(fee)))
    }

    /// Each group takes the terms longer than the one before it, up to its
    /// own: 12 months is "up to a year", 13 "1 to 2 years".
    #[test]
    fn a_deposit_falls_in_the_first_term_long_enough() {
        let term = |months| Term::for_months(months).unwrap();
        assert_eq!(term(1), Term::UpToAMonth);
        assert_eq!(term(2), Term::UpToThreeMonths);
        assert_eq!(term(6), Term::UpToSixMonths);
        assert_eq!(term(7), Term::UpToAYear);
        assert_eq!(term(12), Term::UpToAYear);
        assert_eq!(term(13), Term::UpToTwoYears);
        assert_eq!(term(60), Term::UpToFiveYears);
        assert_eq!(Term::for_months(61), None);
    }

    /// At 4% a year, 6 months earn 2%, and 18 months a year's 4% and then
    /// 2% on ₪104,000: ₪106,080.
    #[test]
    fn a_deposit_pays_part_of_a_year_pro_rata_and_whole_years_compound() {
        let rate = Percent(dec!(4));
        assert_eq!(deposit_worth(dec!(100000), rate, 6), dec!(102000));
        assert_eq!(deposit_worth(dec!(100000), rate, 12), dec!(104000));
        assert_eq!(deposit_worth(dec!(100000), rate, 18), dec!(106080));
    }

    /// Over a year a fund grows by (1 + rate) × (1 − fee), so its yearly
    /// cost is its fee: 100,000 × 1.03 × 0.998 = ₪102,794.
    #[test]
    fn a_funds_year_is_the_rate_less_its_fee() {
        let outcome = simulate(&fund(dec!(0.2)), &scenario(dec!(100000), 12, dec!(3))).unwrap();
        assert_eq!(
            outcome.value_by_month.last().unwrap().round_dp(2),
            dec!(102794.00)
        );
        assert_eq!(outcome.yearly_cost, Percent(dec!(0.2)));
        // 25% of the gain beyond 2% inflation: 0.25 × (2,794 − 2,000) = ₪198.50.
        assert_eq!(outcome.tax.round_dp(2), dec!(198.50));
    }

    /// A deposit's tax is 15% of all its interest, inflation or not.
    #[test]
    fn a_deposits_interest_is_taxed_whole() {
        let outcome = simulate(&deposit(dec!(4)), &scenario(dec!(100000), 12, dec!(3))).unwrap();
        assert_eq!(outcome.earned, dec!(4000));
        assert_eq!(outcome.tax, dec!(600));
        assert_eq!(outcome.after_tax, dec!(103400));
        // It pays more than the 3% rate: the bank keeps less than nothing.
        assert_eq!(outcome.kept.round_dp(2), dec!(-1000.00));
        assert!(outcome.yearly_cost.0 < Decimal::ZERO);
    }

    #[test]
    fn a_fund_pays_no_tax_on_what_only_kept_up_with_prices() {
        let outcome = simulate(&fund(dec!(0.2)), &scenario(dec!(100000), 12, dec!(1))).unwrap();
        assert_eq!(outcome.tax, Decimal::ZERO);
        assert_eq!(outcome.after_tax, *outcome.value_by_month.last().unwrap());
    }

    #[test]
    fn a_term_with_no_published_rate_isnt_offered() {
        let mut by_term = [Some(Percent(dec!(3))); Term::COUNT];
        by_term[Term::UpToTwoYears as usize] = None;
        let bank = place(Pays::Fixed(Rates::new(by_term)));
        assert_eq!(
            simulate(&bank, &scenario(dec!(1000), 18, dec!(3))),
            Err(NotOffered::NoRate {
                term: Term::UpToTwoYears
            })
        );
        assert!(simulate(&bank, &scenario(dec!(1000), 12, dec!(3))).is_ok());
    }

    #[test]
    fn the_comparison_puts_the_most_left_first_and_the_unusable_last() {
        let no_rates = place(Pays::Fixed(Rates::new([None; Term::COUNT])));
        let places = [
            &fund(dec!(0.2)),
            &no_rates,
            &deposit(dec!(4)),
            &fund(dec!(0.1)),
        ];
        let comparison = compare(&places, &scenario(dec!(100000), 12, dec!(3)));
        let order: Vec<usize> = comparison.places.iter().map(|place| place.index).collect();
        assert_eq!(order, [2, 3, 0, 1]);
        assert_eq!(
            comparison.at_the_rate.last().unwrap().round_dp(2),
            dec!(103000.00)
        );
    }

    #[test]
    fn a_scenario_needs_an_amount_and_a_period_the_figures_cover() {
        let valid = scenario(dec!(1000), 12, dec!(3));
        assert_eq!(valid.check(), Ok(()));
        let with = |change: fn(&mut Scenario)| {
            let mut changed = valid.clone();
            change(&mut changed);
            changed.check()
        };
        assert_eq!(
            with(|s| s.first_deposit = dec!(0)),
            Err(InvalidScenario::NoAmount)
        );
        // Monthly deposits alone are enough.
        assert_eq!(
            with(|s| {
                s.first_deposit = dec!(0);
                s.monthly_deposit = dec!(500);
            }),
            Ok(())
        );
        assert_eq!(
            with(|s| s.monthly_deposit = dec!(-1)),
            Err(InvalidScenario::NegativeAmount)
        );
        assert_eq!(
            with(|s| s.first_deposit = dec!(1e13)),
            Err(InvalidScenario::TooLarge)
        );
        assert_eq!(with(|s| s.months = 0), Err(InvalidScenario::Months));
        assert_eq!(with(|s| s.months = 61), Err(InvalidScenario::Months));
        assert_eq!(
            with(|s| s.rate = Percent(dec!(-100))),
            Err(InvalidScenario::RateBelowMinus100)
        );
        // The largest amount itself is fine; a monthly one too large to add
        // up isn't.
        assert_eq!(with(|s| s.first_deposit = LARGEST_AMOUNT), Ok(()));
        assert_eq!(
            with(|s| s.monthly_deposit = Decimal::MAX),
            Err(InvalidScenario::TooLarge)
        );
    }

    /// ₪1,000 at the start of each of two months, at 12% a year with no fee:
    /// the first grows two months, 1.12^(2/12) = 1.0190676, the second one,
    /// 1.12^(1/12) = 1.0094888, so they end at ₪2,028.56. Prices rose 12%
    /// too, raising each deposit the same way, so nothing is a real gain
    /// and nothing is taxed; the money earned 12% a year, all of it the
    /// rate's.
    #[test]
    fn every_monthly_deposit_grows_from_its_month() {
        let two_months = Scenario {
            first_deposit: Decimal::ZERO,
            monthly_deposit: dec!(1000),
            months: 2,
            rate: Percent(dec!(12)),
            inflation: Percent(dec!(12)),
        };
        let outcome = simulate(&fund(dec!(0)), &two_months).unwrap();
        assert_eq!(outcome.value_by_month[0], dec!(1000));
        assert_eq!(outcome.value_by_month[2].round_dp(2), dec!(2028.56));
        assert_eq!(outcome.earned.round_dp(2), dec!(28.56));
        // To a millionth of a shekel: growth and prices are each raised to a
        // part of a year in floating point.
        assert!(outcome.tax < dec!(0.000001), "₪{}", outcome.tax);
        assert_eq!(outcome.yearly_after_tax, Percent(dec!(12)));
        assert_eq!(outcome.yearly_cost, Percent(dec!(0)));
    }

    /// A deposit is one sum: with money put in every month it's not offered.
    #[test]
    fn a_deposit_takes_one_sum() {
        let monthly = Scenario {
            monthly_deposit: dec!(500),
            ..scenario(dec!(1000), 12, dec!(3))
        };
        assert_eq!(
            simulate(&deposit(dec!(4)), &monthly),
            Err(NotOffered::TakesOneSum)
        );
        assert!(simulate(&fund(dec!(0.2)), &monthly).is_ok());
    }

    /// What the user reads when the inputs are wrong, each its own words.
    #[test]
    fn each_problem_says_what_to_fix() {
        let he: Vec<&str> = [
            InvalidScenario::NoAmount,
            InvalidScenario::NegativeAmount,
            InvalidScenario::TooLarge,
            InvalidScenario::Months,
            InvalidScenario::RateBelowMinus100,
        ]
        .into_iter()
        .map(|problem| problem.text(Lang::He))
        .collect();
        assert_eq!(
            he,
            [
                "צריך להפקיד משהו, בהתחלה או כל חודש",
                "הסכומים לא יכולים להיות שליליים",
                "הסכומים גדולים מדי בשביל החישוב",
                "אפשר לחסוך לתקופה של חודש עד 60 חודשים",
                "הריבית והאינפלציה לא יכולות להיות −100% או פחות",
            ]
        );
        assert_eq!(
            InvalidScenario::Months.text(Lang::En),
            "the money can be kept for 1 to 60 months"
        );
    }

    /// The tax and when the money comes out, in the words each row shows.
    #[test]
    fn the_rules_are_named_plainly() {
        assert_eq!(Tax::OfInterest.name(Lang::He), "15% מכל הריבית");
        assert_eq!(Tax::OfRealGain.name(Lang::He), "25% מהרווח שמעבר לאינפלציה");
        assert_eq!(Liquidity::AnyDay.name(Lang::He), "בכל יום");
        assert_eq!(Liquidity::AtTheEnd.name(Lang::He), "בסוף התקופה");
        assert_eq!(Liquidity::AtTheEnd.name(Lang::En), "At the end of the term");
    }

    /// The funds first, the shorter list, then the banks' deposits.
    #[test]
    fn the_kinds_are_the_funds_then_the_deposits() {
        let kinds = kinds();
        let names: Vec<&str> = kinds.iter().map(|kind| kind.name.en.as_ref()).collect();
        assert_eq!(names, ["Money market fund", "Fixed-rate deposit"]);
    }
}
