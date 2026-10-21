//! The outcome chart's words (REDESIGN.md §2.6, amended by §4.6): each row's
//! name, its facts as plain text, its note, why the rules leave an option
//! out, the losses note under the index's bar, the labels of the bar's
//! marks and of the reference lines, the caption, and the chart's own fixed
//! words (`words`).
//!
//! A product is named by what it is before any trade name (§2.8): "אג״ח
//! צמודה של מדינת ישראל". What is bought on the stock side is "מדד מניות
//! רחב" (principle 17, §2.13 item 1): the S&P 500 is named only where it is
//! what was measured, as an example.

use rust_decimal::Decimal;

use super::card::{about, both, three_figures};
use super::reasons::{grouped, shekels, span};
use crate::guide::{Arena, Kind, SafeBecause};
use crate::{Exchange, Lang, Percent, Product, Text, Vehicle};

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A row's name: what the product is. A fund on an exchange is named by
/// what it holds, a broad stock index; where it trades is among its facts.
#[must_use]
pub fn name(kind: Kind) -> Text {
    match kind {
        Kind::MoneyMarketFund => fixed("Money market fund", "קרן כספית"),
        Kind::FixedDeposit => fixed("Fixed-rate deposit", "פיקדון בריבית קבועה"),
        Kind::LinkedDeposit => fixed("CPI-linked deposit", "פיקדון צמוד למדד המחירים לצרכן"),
        Kind::AnyDayDeposit => fixed(
            "Deposit you can take out any day",
            "פיקדון שאפשר למשוך בכל יום",
        ),
        Kind::TreasuryBill => fixed(
            "Makam, a Bank of Israel bill",
            "מק״מ, מלווה קצר מועד של בנק ישראל",
        ),
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
                "Provident fund for investment, stock track",
                "קופת גמל להשקעה, במסלול מניות",
            ),
            (_, Vehicle::StudyFund) => {
                fixed("Study fund, stock track", "קרן השתלמות, במסלול מניות")
            }
            (_, Vehicle::SavingsPolicy) => {
                fixed("Savings policy, stock track", "פוליסת חיסכון, במסלול מניות")
            }
            (
                Product::IsraeliIndexFund
                | Product::StudyFundTrack
                | Product::InvestmentGemelTrack
                | Product::SavingsPolicyTrack,
                _,
            ) => fixed(
                "Index fund on a broad stock index",
                "קרן מחקה על מדד מניות רחב",
            ),
            (Product::IsraeliEtf | Product::UsFund, _) => {
                fixed("ETF on a broad stock index", "קרן סל על מדד מניות רחב")
            }
            (Product::IrishFund | Product::ForeignEtfInTelAviv, _) => fixed(
                "Irish ETF on a broad stock index",
                "קרן סל אירית על מדד מניות רחב",
            ),
        },
    }
}

/// Where a fund on an exchange trades, as its facts and a full name say it.
fn trades_in(kind: Kind) -> Option<Text> {
    let Kind::Way(way) = kind else { return None };
    if way.vehicle != Vehicle::Brokerage {
        return None;
    }
    match way.product {
        Product::IsraeliEtf | Product::ForeignEtfInTelAviv => {
            Some(fixed("In Tel Aviv", "בתל אביב"))
        }
        Product::UsFund => Some(fixed("In the US", "בארה״ב")),
        Product::IrishFund if way.exchange == Some(Exchange::Europe) => {
            Some(fixed("In Europe", "באירופה"))
        }
        Product::IrishFund => Some(fixed("Abroad", "בחו״ל")),
        _ => None,
    }
}

/// The name with where it trades, where two funds of one kind could be
/// named in one sentence: "קרן סל על מדד מניות רחב, בארה״ב".
#[must_use]
pub fn full_name(kind: Kind) -> Text {
    let name = name(kind);
    match trades_in(kind) {
        Some(place) => both(|lang| match lang {
            Lang::En => format!("{}, {}", name.en, place.en.to_lowercase()),
            Lang::He => format!("{}, {}", name.he, place.he),
        }),
        None => name,
    }
}

/// The row's facts, each a short phrase: how sure it is, and what it asks
/// of the saver. Where each is bought is said the same way for every kind a
/// securities account holds (finding 10): the difference between the places
/// is the ₪.
#[must_use]
pub fn tags(kind: Kind, arena: Option<Arena>) -> Vec<Text> {
    let account = fixed("At a bank or an investment house", "בבנק או בבית השקעות");
    let rises_and_falls = fixed("Its value rises and falls", "הערך עולה ויורד");
    match kind {
        Kind::MoneyMarketFund => vec![fixed("The rate changes", "הריבית משתנה"), account],
        Kind::FixedDeposit | Kind::LinkedDeposit => vec![
            fixed("Sum known in advance", "הסכום ידוע מראש"),
            fixed("Locked to the end", "נעול עד הסוף"),
        ],
        Kind::AnyDayDeposit => vec![
            fixed("The rate changes", "הריבית משתנה"),
            fixed(
                "Stays at the bank you bank with",
                "נשאר בבנק שבו יש לכם חשבון",
            ),
        ],
        Kind::TreasuryBill => vec![
            fixed("Takes a securities account", "צריך חשבון ניירות ערך"),
            fixed(
                "Sold before repayment at that day's price",
                "מכירה לפני הפדיון במחיר של אותו יום",
            ),
        ],
        Kind::GovernmentBond { linked: true } => vec![
            fixed("Buying power known in advance", "כוח הקנייה ידוע מראש"),
            account,
        ],
        Kind::GovernmentBond { linked: false } => {
            vec![fixed("Sum known in advance", "הסכום ידוע מראש"), account]
        }
        Kind::CurrentAccount => vec![],
        Kind::Way(way) => match way.vehicle {
            Vehicle::StudyFund => vec![fixed(
                "Self-employed or through an employer",
                "רק לעצמאים או דרך מעסיק",
            )],
            Vehicle::Brokerage if arena == Some(Arena::Safe) => {
                vec![
                    rises_and_falls,
                    fixed("Not for a fixed date", "לא לתאריך קבוע"),
                ]
            }
            Vehicle::Brokerage => match trades_in(kind) {
                Some(place) => vec![place, rises_and_falls],
                None => vec![rises_and_falls, account],
            },
            Vehicle::InvestmentGemel | Vehicle::SavingsPolicy => {
                vec![rises_and_falls, fixed("At a fund company", "בחברה מנהלת")]
            }
        },
    }
}

/// The row's facts as one line of plain text (§4.6): "הריבית משתנה, בבנק
/// או בבית השקעות". After a full stop (makam's), the next starts a phrase.
#[must_use]
pub fn facts(kind: Kind, arena: Option<Arena>) -> Text {
    let tags = tags(kind, arena);
    both(|lang| {
        let mut out = String::new();
        for (at, tag) in tags.iter().enumerate() {
            let tag = &tag[lang];
            if at == 0 {
                out.push_str(tag);
                continue;
            }
            match lang {
                // "In the US, its value rises and falls": lowered after a comma.
                Lang::En => {
                    let mut chars = tag.chars();
                    out.push_str(", ");
                    if let Some(first) = chars.next() {
                        out.extend(first.to_lowercase());
                        out.push_str(chars.as_str());
                    }
                }
                Lang::He if kind == Kind::TreasuryBill => {
                    out.push_str(". ");
                    out.push_str(tag);
                }
                Lang::He => {
                    out.push_str(", ");
                    out.push_str(tag);
                }
            }
        }
        out
    })
}

/// Whether a kind's sum is known in advance to the shekel: its figure is
/// said to the shekel, the rest to three figures (principle 13).
#[must_use]
pub fn known(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::FixedDeposit
            | Kind::LinkedDeposit
            | Kind::GovernmentBond { .. }
            | Kind::TreasuryBill
            | Kind::CurrentAccount
    )
}

/// A row's figure in words, rounded to how sure it is (principle 13): a sum
/// known in advance to the shekel, one resting on a rate or a return to
/// three figures, said as such.
#[must_use]
pub fn amount(kind: Kind, today: Decimal) -> Text {
    if known(kind) {
        both(|_| shekels(today))
    } else {
        both(|lang| format!("{}{}", lang.pick("about ", "בערך "), about(today)))
    }
}

/// A figure on the chart, short (§4.6): "₪281 אלף", "₪1.23 מיליון", to three
/// figures like every amount that rests on an assumption.
#[must_use]
pub fn compact(today: Decimal) -> Text {
    let rounded = three_figures(today);
    let million = Decimal::from(1_000_000);
    let thousand = Decimal::from(1000);
    both(|lang| {
        if rounded.abs() >= million {
            let millions = (rounded / million).normalize();
            match lang {
                Lang::En => format!("₪{millions}m"),
                Lang::He => format!("₪{millions}\u{a0}מיליון"),
            }
        } else if rounded.abs() >= thousand {
            let thousands = (rounded / thousand).normalize();
            match lang {
                Lang::En => format!("₪{thousands}k"),
                Lang::He => format!("₪{thousands}\u{a0}אלף"),
            }
        } else {
            shekels(rounded)
        }
    })
}

/// Under an index row whose plan fills the cushion first (finding 1): the
/// figure is the plan's two parts.
#[must_use]
pub fn path_note(target: Decimal, months: u32) -> Text {
    both(|lang| match lang {
        Lang::En => format!(
            "In your plan: the first {} go to a money market fund for about {}, and stay there; the figure counts both parts.",
            shekels(target),
            span(rough(months), lang)
        ),
        Lang::He => format!(
            "בתוכנית שלכם: {} הראשונים הולכים לקרן כספית, בערך {}, ונשארים שם; הסכום כולל את שני החלקים.",
            shekels(target),
            span(rough(months), lang)
        ),
    })
}

/// A stretch said as roughly as it's known: 80 months is "about 7 years".
fn rough(months: u32) -> u32 {
    if months >= 24 {
        (months + 6) / 12 * 12
    } else {
        months
    }
}

/// Under a bar of a kind whose rate changes: what it spans.
#[must_use]
pub fn rate_note(rate: Percent) -> Text {
    let low = Percent((rate.0 - crate::guide::RATE_STEP).max(Decimal::ZERO));
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

/// Why the rules leave the index out. The chart draws no row for it
/// (§4.6); "how to do it" says it for a row the visitor asks about.
#[must_use]
pub fn excluded(because: SafeBecause, months: u32) -> Text {
    both(|lang| {
        let when = span(months, lang);
        match (because, lang) {
            (SafeBecause::AnyDay, Lang::En) => {
                "Not offered for money you may need on any day.".to_owned()
            }
            (SafeBecause::AnyDay, Lang::He) => "לא מוצע לכסף שאולי תצטרכו בכל יום.".to_owned(),
            (SafeBecause::Cushion, Lang::En) => {
                "With the cushion first, no money reaches it before the date.".to_owned()
            }
            (SafeBecause::Cushion, Lang::He) => {
                "עם הכרית קודם, לא מגיע אליה כסף לפני התאריך.".to_owned()
            }
            (_, Lang::En) => format!(
                "Not offered for money needed in {when} on a fixed date. If the date is 5 years away or more and can wait, that changes."
            ),
            (_, Lang::He) => format!(
                "לא מוצע לכסף שצריך בעוד {when} בתאריך קבוע. אם התאריך רחוק 5 שנים ויותר ואפשר לחכות, זה משתנה."
            ),
        }
    })
}

/// The past of one row's deposits, on the figures' basis.
#[derive(Debug, Clone, Copy)]
pub struct PastCount {
    /// The stretches that would have left less than was deposited.
    pub below: u32,
    pub windows: u32,
    pub months: u32,
    pub since: i32,
}

/// The losses note under the index's bar, on the bar's own basis (finding
/// 3): after fees and tax, in today's money, for the visitor's deposits.
/// The past said as the past, and never rounded to "never".
#[must_use]
pub fn losses(past: &PastCount) -> Text {
    let windows = grouped(past.windows);
    let lower = grouped(past.below);
    let since = past.since;
    both(|lang| {
        let years = span(past.months, lang);
        match (past.below, lang) {
            (0, Lang::En) => format!(
                "In none of {windows} stretches of {years} since {since} would less have been left than was put in (after fees and tax, in today's money)."
            ),
            (0, Lang::He) => format!(
                "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} לא היה נשאר בסוף פחות ממה שהופקד (אחרי עמלות ומס, בכסף של היום)."
            ),
            (_, Lang::En) => format!(
                "In {lower} of {windows} stretches of {years} since {since}, less would have been left than was put in (after fees and tax, in today's money)."
            ),
            (_, Lang::He) => format!(
                "ב\u{2011}{lower} מתוך {windows} תקופות של {years} מאז {since} היה נשאר בסוף פחות ממה שהופקד (אחרי עמלות ומס, בכסף של היום)."
            ),
        }
    })
}

/// The labels of the marks on a stock row's band (§4.6), each followed by
/// its amount: "הגרועה בעבר", "10% בשנה", "חציון העבר", "הטובה".
#[must_use]
pub fn worst_mark() -> Text {
    fixed("Worst so far", "הגרועה בעבר")
}

#[must_use]
pub fn assumed_mark(rate: Percent) -> Text {
    both(|lang| match lang {
        Lang::En => format!("{rate} a year"),
        Lang::He => format!("{rate} בשנה"),
    })
}

#[must_use]
pub fn median_mark() -> Text {
    fixed("Past median", "חציון העבר")
}

#[must_use]
pub fn best_mark() -> Text {
    fixed("Best", "הטובה")
}

/// The reference lines drawn on the chart (§4.6), each followed by its
/// amount: "הפקדתם", "בעו״ש".
#[must_use]
pub fn deposited_line() -> Text {
    fixed("You put in", "הפקדתם")
}

#[must_use]
pub fn current_account_line() -> Text {
    fixed("In the current account", "בעו״ש")
}

/// Under the chart's title (§4.6): what the amounts are, what the stock
/// rows were measured on, and that a row can be tapped. Without a stock
/// row, nothing about stocks.
#[must_use]
pub fn caption(stocks: bool) -> Text {
    if stocks {
        fixed(
            "In today's money, after fees and tax. Stocks by the S&P 500 index, as an example. Tap a row to see how it's done.",
            "בכסף של היום, אחרי עמלות ומס. המניות לפי מדד S&P\u{a0}500, לדוגמה. הקישו על שורה כדי לראות איך עושים את זה.",
        )
    } else {
        fixed(
            "In today's money, after fees and tax. Tap a row to see how it's done.",
            "בכסף של היום, אחרי עמלות ומס. הקישו על שורה כדי לראות איך עושים את זה.",
        )
    }
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
        subtitle: caption(true),
        deposited: deposited_line(),
        current_account: current_account_line(),
        known: fixed(
            "A sum known more or less in advance",
            "סכום שידוע פחות או יותר מראש",
        ),
        market: fixed(
            "Stocks, by the S&P 500 index as an example: from the worst to the best stretch so far",
            "מניות, לפי מדד S&P\u{a0}500 לדוגמה: מהתקופה הגרועה לטובה שהיו עד היום",
        ),
        worst: worst_mark(),
        median: median_mark(),
        best: best_mark(),
        per_year: fixed("a year", "בשנה"),
        if_it_rises: fixed(
            "if the broad stock index rises by ",
            "אם מדד המניות הרחב יעלה ב\u{2011}",
        ),
    }
}
