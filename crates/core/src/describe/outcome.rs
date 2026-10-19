//! The outcome chart's words (REDESIGN.md §2.6): each row's name, its short
//! tags, its note, why the rules leave an option out, and the losses note
//! under the index's bar, and the chart's own fixed words (`words`).
//!
//! A product is named by what it is before any trade name (§2.8): "אג״ח
//! צמודה של מדינת ישראל", and the index is named: "מדד S&P 500".

use super::card::both;
use super::reasons::{grouped, span};
use crate::guide::{Arena, Kind, SafeBecause};
use crate::history::DepositsBand;
use crate::{Lang, Percent, Product, Text, Vehicle};

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A row's name: what the product is.
#[must_use]
pub fn name(kind: Kind) -> Text {
    match kind {
        Kind::MoneyMarketFund => fixed("Money market fund", "קרן כספית"),
        Kind::FixedDeposit => fixed("Fixed-rate deposit", "פיקדון בריבית קבועה"),
        Kind::LinkedDeposit => fixed("CPI-linked deposit", "פיקדון צמוד למדד המחירים לצרכן"),
        Kind::GovernmentBond { linked: true } => fixed(
            "Israeli government bond, linked to the CPI",
            "אג״ח צמודה של מדינת ישראל",
        ),
        Kind::GovernmentBond { linked: false } => fixed(
            "Israeli government bond, in shekels",
            "אג״ח שקלית של מדינת ישראל",
        ),
        Kind::CurrentAccount => fixed("Current account", "עו״ש"),
        Kind::Way(way) => match (way.product, way.vehicle) {
            (_, Vehicle::InvestmentGemel) => fixed(
                "Provident fund for investment, S&P 500 track",
                "קופת גמל להשקעה, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::StudyFund) => fixed(
                "Study fund, S&P 500 track",
                "קרן השתלמות, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::SavingsPolicy) => fixed(
                "Savings policy, S&P 500 track",
                "פוליסת חיסכון, מסלול מדד S&P\u{a0}500",
            ),
            (
                Product::IsraeliIndexFund
                | Product::StudyFundTrack
                | Product::InvestmentGemelTrack
                | Product::SavingsPolicyTrack,
                _,
            ) => fixed("Index fund on the S&P 500", "קרן מחקה על מדד S&P\u{a0}500"),
            (Product::IsraeliEtf, _) => fixed(
                "ETF on the S&P 500, in Tel Aviv",
                "קרן סל על מדד S&P\u{a0}500, בתל אביב",
            ),
            (Product::UsFund, _) => fixed(
                "ETF on the S&P 500, in the US",
                "קרן סל על מדד S&P\u{a0}500, בארה״ב",
            ),
            (Product::IrishFund | Product::ForeignEtfInTelAviv, _) => fixed(
                "Irish ETF on the S&P 500",
                "קרן סל אירית על מדד S&P\u{a0}500",
            ),
        },
    }
}

/// The row's short tags: how sure, and what it asks of the saver.
#[must_use]
pub fn tags(kind: Kind, arena: Option<Arena>) -> Vec<Text> {
    let account = fixed("Needs a trading account", "צריך חשבון מסחר");
    match kind {
        Kind::MoneyMarketFund => vec![
            fixed("The rate changes", "הריבית משתנה"),
            fixed("At your bank", "בבנק שלכם"),
        ],
        Kind::FixedDeposit | Kind::LinkedDeposit => vec![
            fixed("Sum known in advance", "הסכום ידוע מראש"),
            fixed("Locked to the end", "נעול עד הסוף"),
        ],
        Kind::GovernmentBond { linked: true } => vec![
            fixed("Buying power known in advance", "כוח הקנייה ידוע מראש"),
            account,
        ],
        Kind::GovernmentBond { linked: false } => {
            vec![fixed("Sum known in advance", "הסכום ידוע מראש"), account]
        }
        Kind::CurrentAccount => vec![],
        Kind::Way(way) => {
            let mut tags = vec![fixed("Rises and falls", "עולה ויורד")];
            match way.vehicle {
                Vehicle::StudyFund => tags.push(fixed(
                    "Self-employed or through an employer",
                    "רק לעצמאים או דרך מעסיק",
                )),
                Vehicle::Brokerage if arena == Some(Arena::Safe) => {
                    tags.push(fixed("Not for a fixed date", "לא לתאריך קבוע"));
                }
                Vehicle::Brokerage => tags.push(account),
                Vehicle::InvestmentGemel | Vehicle::SavingsPolicy => {
                    tags.push(fixed("At a fund company", "בחברה מנהלת"));
                }
            }
            tags
        }
    }
}

/// Under the fund while its months fill the cushion, and what its bar spans
/// if it has one (`rate`). Said without "the first N", whose Hebrew gender
/// follows the unit.
#[must_use]
pub fn cushion_note(months: u32, rate: Option<Percent>) -> Text {
    let bar = rate.map(rate_note);
    both(|lang| {
        let bar = bar
            .as_ref()
            .map_or_else(String::new, |bar| format!(" {}", &bar[lang]));
        match lang {
            Lang::En => format!(
                "In your plan, the cushion builds up here for {}.{bar}",
                span(months, lang)
            ),
            Lang::He => format!(
                "בתוכנית שלכם, כאן נבנית הכרית, במשך {}.{bar}",
                span(months, lang)
            ),
        }
    })
}

/// Under the fund's bar: what it spans.
#[must_use]
pub fn rate_note(rate: Percent) -> Text {
    let low = Percent((rate.0 - crate::guide::RATE_STEP).max(rust_decimal::Decimal::ZERO));
    let high = Percent(rate.0 + crate::guide::RATE_STEP);
    both(|lang| match lang {
        Lang::En => format!(
            "The bar: what's left if the Bank of Israel's rate averages between {low} and {high}."
        ),
        Lang::He => {
            format!("הפס: מה יישאר אם ריבית בנק ישראל תהיה בממוצע בין {low} ל\u{2011}{high}.")
        }
    })
}

/// Why the rules leave the index out.
#[must_use]
pub fn excluded(because: SafeBecause, months: u32) -> Text {
    both(|lang| {
        let when = span(months, lang);
        match (because, lang) {
            (SafeBecause::AnyDay, Lang::En) => {
                "Not offered for money you may need on any day.".to_owned()
            }
            (SafeBecause::AnyDay, Lang::He) => "לא מוצע לכסף שאולי תצטרכו בכל יום.".to_owned(),
            (_, Lang::En) => format!(
                "Not offered for money needed in {when} on a fixed date. If the date is 5 years away or more and can wait, that changes."
            ),
            (_, Lang::He) => format!(
                "לא מוצע לכסף שצריך בעוד {when} בתאריך קבוע. אם התאריך רחוק 5 שנים ויותר ואפשר לחכות, זה משתנה."
            ),
        }
    })
}

/// The losses note under the index's bar: the past, said as the past, and
/// never rounded to "never". With deposits every month, a loss is ending
/// with less than was put in.
#[must_use]
pub fn losses(history: &DepositsBand, monthly: bool) -> Text {
    let windows = grouped(history.windows);
    let lower = grouped(history.lower);
    let since = history.from.year();
    both(|lang| {
        let years = span(history.months, lang);
        match (history.lower, monthly, lang) {
            (0, true, Lang::En) => format!(
                "In none of {windows} stretches of {years} since {since} was less left at the end than was put in (in dollars, before fees and tax)."
            ),
            (0, true, Lang::He) => format!(
                "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} לא נשאר בסוף פחות ממה שהופקד (בדולרים, לפני עמלות ומס)."
            ),
            (0, false, Lang::En) => format!(
                "In none of {windows} stretches of {years} since {since} did the S&P 500 index end at a loss (in dollars, before fees and tax)."
            ),
            (0, false, Lang::He) => format!(
                "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} מדד S&P\u{a0}500 לא נגמר בהפסד (בדולרים, לפני עמלות ומס)."
            ),
            (_, true, Lang::En) => format!(
                "In {lower} of {windows} stretches of {years} since {since}, less was left at the end than was put in (in dollars, before fees and tax)."
            ),
            (_, true, Lang::He) => format!(
                "ב\u{2011}{lower} מתוך {windows} תקופות של {years} מאז {since} נשאר בסוף פחות ממה שהופקד (בדולרים, לפני עמלות ומס)."
            ),
            (_, false, Lang::En) => format!(
                "In {lower} of {windows} stretches of {years} since {since}, the S&P 500 index ended at a loss (in dollars, before fees and tax)."
            ),
            (_, false, Lang::He) => format!(
                "ב\u{2011}{lower} מתוך {windows} תקופות של {years} מאז {since} מדד S&P\u{a0}500 נגמר בהפסד (בדולרים, לפני עמלות ומס)."
            ),
        }
    })
}

/// The chart's fixed words: its subtitle, the legend, and the labels on the
/// index's bar.
#[derive(Debug, Clone)]
pub struct Words {
    /// Under the title: what the amounts are, and that a row can be tapped.
    pub subtitle: Text,
    /// The reference lines, each followed by its amount in the legend.
    pub deposited: Text,
    pub current_account: Text,
    /// The keys of the marks, each shown only when a row draws one.
    pub known: Text,
    pub market: Text,
    /// The labels on the index's bar, each followed by its amount.
    pub worst: Text,
    pub median: Text,
    pub best: Text,
    /// After the yearly return the index's figure assumes: "10% בשנה".
    pub per_year: Text,
    /// For a screen reader, after the index's figure: this, the yearly
    /// return, "%" and `per_year`.
    pub if_it_rises: Text,
}

#[must_use]
pub fn words() -> Words {
    Words {
        subtitle: fixed(
            "In today's money, after fees and tax. Tap a row to see how it's done.",
            "בכסף של היום, אחרי עמלות ומס. הקישו על שורה כדי לראות איך עושים את זה.",
        ),
        deposited: fixed("What you put in", "מה שהפקדתם"),
        current_account: fixed(
            "If the money stays in the current account",
            "אם הכסף יישאר בעו״ש",
        ),
        known: fixed(
            "A sum known more or less in advance",
            "סכום שידוע פחות או יותר מראש",
        ),
        market: fixed(
            "The S&P 500 index: from the worst to the best stretch so far",
            "מדד S&P\u{a0}500: מהתקופה הגרועה לטובה שהיו עד היום",
        ),
        worst: fixed("Worst so far", "הגרוע שהיה"),
        median: fixed("Middle", "באמצע"),
        best: fixed("Best so far", "הטוב שהיה"),
        per_year: fixed("a year", "בשנה"),
        if_it_rises: fixed(
            "if the S&P 500 index rises by ",
            "אם מדד S&P\u{a0}500 יעלה ב\u{2011}",
        ),
    }
}
