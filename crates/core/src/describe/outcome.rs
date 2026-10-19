//! The outcome chart's words (REDESIGN.md §2.6): each row's name, its short
//! tags, its note, why the rules leave an option out, and the losses note
//! under the index's bar. First drafts.
//!
//! A product is named by what it is before any trade name (§2.8): "אג״ח
//! צמודה של מדינת ישראל", and the index is named: "מדד S&P 500".

use super::card::{both, losses_sentence};
use super::reasons::span;
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
        Kind::LinkedDeposit => fixed("CPI-linked deposit", "פיקדון צמוד מדד המחירים לצרכן"),
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

/// Under the fund while its months fill the cushion.
#[must_use]
pub fn cushion_note(months: u32) -> Text {
    both(|lang| match lang {
        Lang::En => format!(
            "In your plan: the first {}, for the cushion.",
            span(months, lang)
        ),
        Lang::He => format!(
            "בתוכנית שלכם: ה\u{2011}{} הראשונים, לכרית.",
            span(months, lang)
        ),
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

/// The losses note under the index's bar: the past, said as the past.
#[must_use]
pub fn losses(history: &DepositsBand, monthly: bool) -> Text {
    both(|lang| losses_sentence(history, monthly, lang))
}
