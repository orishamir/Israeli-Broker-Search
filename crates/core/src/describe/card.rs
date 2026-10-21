//! The answer card's words (REDESIGN.md §2.2–§2.5): the chip's note, the
//! title and body, the lines' sentences, each decision's question, why it is
//! the visitor's, its options and the difference, and the plan. First drafts
//! from the mockup's copy; every number in them comes from the guide.
//!
//! Each function takes the figures it words and returns both languages.

#![allow(
    clippy::too_many_lines,
    clippy::format_push_string,
    reason = "the words of every case"
)]

use rust_decimal::Decimal;
use time::Date;

use super::reasons::{grouped, month_year, shekels, span};
use crate::guide::{
    Account, Arena, ConfidenceState, Holdings, Kind, LineKind, SafeBecause, SafeWay,
};
use crate::ways::Way;
use crate::{Lang, Percent, Product, Text, Vehicle};

/// A text built once in each language.
pub(crate) fn both(make: impl Fn(Lang) -> String) -> Text {
    Text::owned(make(Lang::En), make(Lang::He))
}

/// The words of a decision.
#[derive(Debug, Clone)]
pub struct DecisionWords {
    pub question: Text,
    pub why: Text,
    pub options: Vec<OptWords>,
    pub difference: Option<Text>,
}

/// The words of one option.
#[derive(Debug, Clone)]
pub struct OptWords {
    pub title: Text,
    pub amount: Text,
    /// A second amount, in its own unit, under the first: the pension a
    /// provident fund's sum would pay.
    pub also: Option<Text>,
    pub facts: Vec<Text>,
    pub fits_if: Option<Text>,
    pub button: Text,
}

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A figure that rests on an assumption (the index at a yearly return, a
/// money market fund at a rate), to three significant figures: the one
/// rounding rule for such figures (principle 13). ₪427,481 is ₪427,000,
/// ₪1,228,620 is ₪1,230,000, ₪19,925 is ₪19,900.
#[must_use]
pub fn three_figures(amount: Decimal) -> Decimal {
    let whole = amount.abs().round();
    let digits = whole.to_string().len();
    if digits <= 3 {
        return amount.round();
    }
    let step = Decimal::from(10_u64.pow(u32::try_from(digits - 3).unwrap_or(0)));
    (amount / step).round() * step
}

/// A headline amount, rounded to match how sure it is (principle 13):
/// "₪427,000".
#[must_use]
pub fn about(amount: Decimal) -> String {
    shekels(three_figures(amount))
}

/// "בערך ₪427,000": a figure that rests on an assumption, said as such.
fn roughly(amount: Decimal, lang: Lang) -> String {
    format!("{}{}", lang.pick("about ", "בערך "), about(amount))
}

/// Whether a kind's sum is known in advance to the shekel: a deposit, a bond
/// held to maturity. Its figures, and the gaps between them, are said to the
/// shekel; the rest to three figures (principle 13: one precision for one
/// amount, the review's N6).
fn known(kind: Option<Kind>) -> bool {
    kind.is_some_and(super::outcome::known)
}

/// A figure of `kind`, at its precision: "₪649,032", or "₪427,000".
fn figure(kind: Option<Kind>, amount: Decimal) -> String {
    if known(kind) {
        shekels(amount)
    } else {
        about(amount)
    }
}

/// The same, said with how sure it is: "₪649,032", or "בערך ₪427,000".
fn figure_said(kind: Option<Kind>, amount: Decimal, lang: Lang) -> String {
    if known(kind) {
        shekels(amount)
    } else {
        roughly(amount, lang)
    }
}

/// Two figures as a range, "בין ₪2,990 ל‑₪4,110", or one where they round
/// alike, said with how sure it is.
fn figures_between(kind: Option<Kind>, low: Decimal, high: Decimal, lang: Lang) -> String {
    let (low_text, high_text) = (figure(kind, low), figure(kind, high));
    if low_text == high_text {
        figure_said(kind, low, lang)
    } else {
        match lang {
            Lang::En => format!("{low_text}–{high_text}"),
            Lang::He => format!("בין {low_text} ל\u{2011}{high_text}"),
        }
    }
}

/// The visitor's bank among the places a sentence names: "הבנק שלכם
/// (הפועלים)". In brackets, so a list of names around it stays a list.
#[must_use]
pub fn your_bank(provider: &Text) -> Text {
    both(|lang| match lang {
        Lang::En => format!("your bank ({})", provider.en),
        Lang::He => format!("הבנק שלכם ({})", provider.he),
    })
}

// ─────────────────────────── Dates ───────────────────────────

/// The first of the month `months` after `today`'s.
pub(crate) fn end_date(today: Date, months: u32) -> Option<Date> {
    let total =
        i64::from(today.year()) * 12 + i64::from(u8::from(today.month())) - 1 + i64::from(months);
    let year = i32::try_from(total.div_euclid(12)).ok()?;
    let month = u8::try_from(total.rem_euclid(12) + 1).ok()?;
    Date::from_calendar_date(year, time::Month::try_from(month).ok()?, 1).ok()
}

/// The end the visitor's own date gives, if they gave one.
fn dated_end(today: Option<Date>, months: u32, dated: bool) -> Option<Date> {
    today
        .filter(|_| dated)
        .and_then(|today| end_date(today, months))
}

/// "October 2029"; with no date of the visitor's, the span said as an
/// example: "לדוגמה, בעוד 10 שנים" (finding 11: no invented date).
#[must_use]
pub fn date(today: Option<Date>, months: u32, dated: bool) -> Text {
    both(|lang| match dated_end(today, months, dated) {
        Some(end) => month_year(end, lang),
        None if dated => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
        None => format!(
            "{} {}",
            lang.pick("for example, in", "לדוגמה, בעוד"),
            span(months, lang)
        ),
    })
}

/// "in October 2029" / "באוקטובר 2029", or "in 3 years" / "בעוד 3 שנים".
fn at_date(today: Option<Date>, months: u32, lang: Lang) -> String {
    match today.and_then(|today| end_date(today, months)) {
        Some(end) => format!("{}{}", lang.pick("in ", "ב"), month_year(end, lang)),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
    }
}

/// The year the money is needed, or "in 3 years".
fn in_year(today: Option<Date>, months: u32, lang: Lang) -> String {
    match today.and_then(|today| end_date(today, months)) {
        Some(end) => format!("{}{}", lang.pick("in ", "ב\u{2011}"), end.year()),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
    }
}

/// "What you'll have in October 2029", or with no date of the visitor's,
/// "מה יהיה לכם, לדוגמה, בעוד 10 שנים".
#[must_use]
pub fn outcome_title(today: Option<Date>, months: u32, dated: bool) -> Text {
    both(|lang| {
        let what = lang.pick("What you'll have", "מה יהיה לכם");
        if dated {
            format!("{what} {}", at_date(today, months, lang))
        } else {
            format!(
                "{what}{} {}",
                lang.pick(", for example,", ", לדוגמה,"),
                at_date(None, months, lang)
            )
        }
    })
}

// ─────────────────────────── The chip, the title ───────────────────────────

#[must_use]
pub fn chip_note(state: ConfidenceState, arena: Option<Arena>, open: bool, existing: bool) -> Text {
    match (state, arena) {
        _ if existing => match state {
            ConfidenceState::A => fixed(
                "What it costs where it is is clear. What the portfolio holds decides what moving would change.",
                "ברור כמה זה עולה איפה שזה נמצא. מה יש בתיק קובע מה מעבר היה משנה.",
            ),
            _ => fixed(
                "Moving changes little here, whatever the portfolio holds.",
                "כאן מעבר משנה מעט, מה שלא יהיה בתיק.",
            ),
        },
        (ConfidenceState::C, _) => fixed(
            "There's no one right answer here. The decision below settles it.",
            "אין כאן תשובה אחת נכונה. ההחלטה למטה מכריעה.",
        ),
        (ConfidenceState::A, Some(Arena::Index)) if open => fixed(
            "The direction is clear. The decisions below are about the way.",
            "הכיוון ברור. ההחלטות למטה הן על הדרך.",
        ),
        (ConfidenceState::A, Some(Arena::Index)) => fixed(
            "The direction is clear, and it holds in every version we checked.",
            "הכיוון ברור, והוא לא משתנה בשום גרסה שבדקנו.",
        ),
        (ConfidenceState::A, _) => fixed(
            "In every version we checked, the answer stays the same.",
            "בכל גרסה שבדקנו, התשובה לא משתנה.",
        ),
        (ConfidenceState::B, Some(Arena::Safe)) => fixed(
            "What matters is clear: a safe way. Which one depends on what suits you.",
            "מה שחשוב ברור: דרך בטוחה. איזו מהן, תלוי במה שנוח לכם.",
        ),
        (ConfidenceState::B, _) => fixed(
            "The site's rules point to stocks, but in the worst stretches of the past a safe way left more.",
            "לפי הכללים של האתר אלה מניות, אבל בתקופות הגרועות של העבר דרך בטוחה השאירה יותר.",
        ),
    }
}

/// The figures the title and body word. No amount: the body says the
/// direction and what's left to decide, and the figures are the lines', the
/// options' and the chart's (principle 13: each number once).
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "facts about the answer, each its own"
)]
pub struct TitleFigures {
    pub state: ConfidenceState,
    pub arena: Option<Arena>,
    pub asked: bool,
    pub existing: bool,
    pub months: u32,
    /// The visitor gave the date.
    pub dated: bool,
    pub fixed_date: bool,
    /// The safe kinds compared, best first.
    pub safe_kinds: Vec<Kind>,
    pub open: bool,
    pub retirement: bool,
    /// The index's worst stretch of the past, on the figures' basis, ends
    /// below what was deposited.
    pub below_deposited: bool,
    /// The way the growth side holds the index in.
    pub lead: Option<Way>,
    /// Money that may be needed any day: no date to say.
    pub any_day: bool,
}

/// The growth side as a sentence names it: a fund on a broad stock index,
/// or the wrapper on a stock track (a provident fund is no קרן, the
/// review's N5).
fn growth_named(lead: Option<Way>, lang: Lang) -> String {
    match lead.map(|lead| lead.vehicle) {
        Some(vehicle) if vehicle != Vehicle::Brokerage => {
            let wrapper = match vehicle {
                Vehicle::InvestmentGemel => {
                    lang.pick("A provident fund for investment", "קופת גמל להשקעה")
                }
                Vehicle::StudyFund => lang.pick("A study fund", "קרן השתלמות"),
                _ => lang.pick("A savings policy", "פוליסת חיסכון"),
            };
            match lang {
                Lang::En => format!("{wrapper} on a stock track"),
                Lang::He => format!("{wrapper} במסלול מניות"),
            }
        }
        _ => lang
            .pick("A fund on a broad stock index", "קרן על מדד מניות רחב")
            .to_owned(),
    }
}

fn safe_range(low: Option<Decimal>, high: Option<Decimal>, lang: Lang) -> String {
    match (low, high) {
        // In words, not "₪a–₪b": right to left, the two would swap places.
        (Some(low), Some(high)) if about(high) != about(low) => match lang {
            Lang::En => format!("about {}–{}", about(low), about(high)),
            Lang::He => format!("בין {} ל\u{2011}{} בערך", about(low), about(high)),
        },
        (_, Some(high)) => roughly(high, lang),
        _ => String::new(),
    }
}

/// What a saver for retirement may already have (BUT-5), said once in the
/// body: not a question, since nothing on the page depends on it.
fn existing_pension(lang: Lang) -> &'static str {
    lang.pick(
        "This comes on top of the pension from a pension fund through work, if you have one: its forecast is in the fund's yearly report, and the pension clearing house (המסלקה הפנסיונית) shows all your pension savings in one place.",
        "וזה בנוסף לקצבה מקרן הפנסיה מהעבודה, אם יש לכם: התחזית שלה מופיעה בדוח השנתי של הקרן, ואת כל החיסכון הפנסיוני שלכם אפשר לראות במקום אחד, במסלקה הפנסיונית.",
    )
}

/// What the safe side holds, said by how sure each kind is (finding 6):
/// a money market fund's sum moves with the rate; a deposit's and a shekel
/// bond's are known in advance; a linked bond's buying power is.
fn safe_certainty(kinds: &[Kind], lang: Lang) -> &'static str {
    let fund = kinds.contains(&Kind::MoneyMarketFund);
    let known = kinds.iter().any(|kind| {
        matches!(
            kind,
            Kind::FixedDeposit | Kind::LinkedDeposit | Kind::GovernmentBond { .. }
        )
    });
    match (fund, known) {
        (true, false) => lang.pick(
            "a money market fund leaves a sum that moves with the Bank of Israel's rate",
            "קרן כספית משאירה סכום שזז עם ריבית בנק ישראל",
        ),
        (false, _) => lang.pick(
            "a safe way leaves a sum known in advance",
            "דרך בטוחה משאירה סכום שידוע מראש",
        ),
        (true, true) => lang.pick(
            "a safe way leaves a sum known in advance, or one that moves only with the Bank of Israel's rate",
            "דרך בטוחה משאירה סכום שידוע מראש, או כזה שזז רק עם ריבית בנק ישראל",
        ),
    }
}

#[must_use]
pub fn title_and_body(f: &TitleFigures) -> (Text, Text) {
    let when = |lang| span(f.months, lang);
    let (title, body) = if f.existing {
        (
            fixed(
                "What the portfolio costs where it is, and what moving would change.",
                "כמה עולה התיק איפה שהוא, ומה מעבר היה משנה.",
            ),
            fixed(
                "The question here isn't where to invest, but what holding what you bought costs. What the portfolio holds decides whether moving it saves anything: moved as it is, it isn't sold, and pays no tax.",
                "השאלה כאן היא לא איפה להשקיע, אלא כמה עולה להחזיק את מה שכבר קניתם. מה יש בתיק קובע אם מעבר יחסוך משהו: תיק שעובר כמו שהוא לא נמכר, ולא משלם מס.",
            ),
        )
    } else if f.asked || f.state == ConfidenceState::C {
        let title = fixed(
            "Safety or growth: that's your decision.",
            "ביטחון או צמיחה: זו החלטה שלכם.",
        );
        // Worded from the comparison that makes it a question (finding 3):
        // the worst stretches left less than a safe way, and, only where
        // they did, less than was put in.
        let body = both(|lang| {
            let start = if f.dated {
                format!("{} {}", lang.pick("In", "בעוד"), when(lang))
            } else {
                format!(
                    "{} {}",
                    lang.pick("Over at least", "לטווח של לפחות"),
                    when(lang)
                )
            };
            let safe = safe_certainty(&f.safe_kinds, lang);
            let growth = growth_named(f.lead, lang);
            match (lang, f.below_deposited) {
                (Lang::En, true) => format!(
                    "{start}, {safe}. {growth} is expected to leave more, but in the worst stretches of the past it would have left less than a safe way, and even less than was put in, in the very year the money was needed."
                ),
                (Lang::He, true) => format!(
                    "{start}, {safe}. {growth} צפויה להשאיר יותר, אבל בתקופות הגרועות של העבר היא הייתה משאירה פחות מדרך בטוחה, ואפילו פחות ממה שהופקד, דווקא בשנה שהיו צריכים את הכסף."
                ),
                (Lang::En, false) => format!(
                    "{start}, {safe}. {growth} is expected to leave more, but in the worst stretches of the past it would have left less than a safe way, in the very year the money was needed."
                ),
                (Lang::He, false) => format!(
                    "{start}, {safe}. {growth} צפויה להשאיר יותר, אבל בתקופות הגרועות של העבר היא הייתה משאירה פחות מדרך בטוחה, דווקא בשנה שהיו צריכים את הכסף."
                ),
            }
        });
        (title, body)
    } else if let Some(Arena::Index) = f.arena {
        let title = both(|lang| match (f.state, lang) {
            (ConfidenceState::A, Lang::En) => {
                format!("A broad stock index, for {}.", when(lang))
            }
            (ConfidenceState::A, Lang::He) => {
                format!("מדד מניות רחב, לטווח של {}.", when(lang))
            }
            (_, Lang::En) => format!(
                "A broad stock index for {}, by the site's rules.",
                when(lang)
            ),
            (_, Lang::He) => format!(
                "מדד מניות רחב ל\u{2011}{}, לפי הכללים של האתר.",
                when(lang)
            ),
        });
        let body = both(|lang| {
            let direction = match (f.state, lang) {
                (ConfidenceState::A, Lang::En) => {
                    "Over this long, the direction is clear: a fund on a broad stock index leaves more than the safe ways."
                }
                (ConfidenceState::A, Lang::He) => {
                    "לטווח ארוך כזה הכיוון ברור: קרן על מדד מניות רחב משאירה יותר מהדרכים הבטוחות."
                }
                (_, Lang::En) => {
                    "Over this long, the site's rules put the money in a fund on a broad stock index."
                }
                (_, Lang::He) => {
                    "לטווח ארוך כזה, הכללים של האתר שמים את הכסף בקרן על מדד מניות רחב."
                }
            };
            if f.open {
                format!(
                    "{direction} {}",
                    lang.pick(
                        "What's left is how, and only you can decide it: the decisions below.",
                        "מה שנשאר הוא איך, ורק אתם יכולים להחליט: ההחלטות למטה.",
                    )
                )
            } else {
                direction.to_owned()
            }
        });
        (title, body)
    } else if f.state == ConfidenceState::A {
        let lead = f.safe_kinds.first().copied().map(super::outcome::name);
        let title = both(|lang| match (&lead, lang) {
            (Some(lead), Lang::En) => format!("What fits here: {}.", lead.en),
            (Some(lead), Lang::He) => format!("מה שמתאים כאן: {}.", lead.he),
            (None, _) => String::new(),
        });
        // As the way's line says it (the review's N16): the only safe way
        // here, or the one that leaves more in every version.
        let only = f.safe_kinds.len() < 2;
        let body = both(|lang| {
            let clear = &super::path::safe_way_why(only)[lang];
            if f.open {
                format!(
                    "{clear} {}",
                    lang.pick("What's left is where, below.", "מה שנשאר הוא איפה, למטה.")
                )
            } else {
                clear.to_owned()
            }
        });
        (title, body)
    } else {
        let title = fixed(
            "A safe way, and you choose which.",
            "דרך בטוחה, ואתם בוחרים איזו.",
        );
        let body = both(|lang| match lang {
            _ if f.any_day => lang
                .pick(
                    "Money you may need any day: every way here can be taken out the next day, and they all end close to each other, in today's money and after tax, so the choice between them is about what suits you.",
                    "כסף שאולי תצטרכו בכל יום: מכל הדרכים כאן אפשר למשוך כבר למחרת, וכולן מסתיימות קרוב זו לזו, בכסף של היום ואחרי מס, ולכן הבחירה ביניהן היא לפי מה שנוח לכם.",
                )
                .to_owned(),
            Lang::En => format!(
                "The money is needed in {}{}. All the safe ways end close to each other, in today's money and after tax, so the choice between them is about what suits you.",
                when(lang),
                if f.fixed_date {
                    ", on a date that can't move"
                } else {
                    ""
                }
            ),
            Lang::He => format!(
                "הכסף נדרש בעוד {}{}. כל הדרכים הבטוחות מסתיימות קרוב זו לזו, בכסף של היום ואחרי מס, ולכן הבחירה ביניהן היא לפי מה שנוח לכם.",
                when(lang),
                if f.fixed_date {
                    ", בתאריך שלא זז"
                } else {
                    ""
                }
            ),
        });
        (title, body)
    };
    if !f.retirement {
        return (title, body);
    }
    let body = both(|lang| format!("{} {}", &body[lang], existing_pension(lang)));
    (title, body)
}

/// A monthly pension, from ₪1,000 up to the nearest ₪100: the payers'
/// coefficients already make it a range, so tens would claim too much.
fn pension_rounded(amount: Decimal) -> String {
    let step = if amount >= Decimal::from(1000) {
        Decimal::ONE_HUNDRED
    } else {
        Decimal::TEN
    };
    shekels((amount / step).round() * step)
}

/// The monthly pension as a range: "בין ₪5,700 ל‑₪6,300 בחודש בערך".
fn pension_range(estimate: &crate::annuity::Estimate, lang: Lang) -> String {
    let (low, high) = (
        pension_rounded(estimate.low),
        pension_rounded(estimate.high),
    );
    if low == high {
        match lang {
            Lang::En => format!("about {low} a month"),
            Lang::He => format!("בערך {low} בחודש"),
        }
    } else {
        match lang {
            Lang::En => format!("about {low} to {high} a month"),
            Lang::He => format!("בין {low} ל\u{2011}{high} בחודש בערך"),
        }
    }
}

/// The monthly pension the provident fund's sum would buy (BUT-5), on the
/// answer card: the range across the payers' published coefficients. Whose
/// they are and on what terms is in the fund's steps (`pension_terms`).
#[must_use]
pub fn annuity(estimate: &crate::annuity::Estimate, index_return: Percent) -> Text {
    both(|lang| {
        let range = pension_range(estimate, lang);
        match lang {
            Lang::En => format!(
                "As a monthly pension from the provident fund for investment: {range}, in today's money, {}.",
                if_index_rises(index_return, lang)
            ),
            Lang::He => format!(
                "כקצבה חודשית מקופת הגמל להשקעה: {range}, בכסף של היום, {}.",
                if_index_rises(index_return, lang)
            ),
        }
    })
}

/// The same pension as an option's second amount line (the review's N7):
/// the amount above it already says what it assumes of the index.
#[must_use]
pub fn annuity_also(estimate: &crate::annuity::Estimate) -> Text {
    both(|lang| {
        let range = pension_range(estimate, lang);
        match lang {
            Lang::En => format!("As a pension: {range}, in today's money"),
            Lang::He => format!("כקצבה: {range}, בכסף של היום"),
        }
    })
}

// ─────────────────────────── The lines ───────────────────────────

/// The lines' names (§4.1).
#[must_use]
pub fn label(kind: LineKind) -> Text {
    match kind {
        LineKind::Before => fixed("Emergency money", "כסף לחירום"),
        LineKind::PlaceType => fixed("Stocks or safe", "מניות או בטוח"),
        LineKind::Way => fixed("The product", "המוצר"),
        LineKind::Where => fixed("Where to buy", "איפה קונים"),
    }
}

/// "בטוח" stated (§4.4): what to do, and why. For a short or a known date,
/// and under the 5-year rule, how often stocks over the visitor's span
/// ended below what was put in, the past said as the past (`past`, the
/// count the engine works out for the span; without one, the span alone).
#[must_use]
pub fn place_safe(because: SafeBecause, months: u32, past: Option<&Past>) -> Text {
    both(|lang| {
        let when = span(months, lang);
        match (because, lang, past) {
            (SafeBecause::AnyDay, Lang::En, _) => {
                "Safe: money you may need on any day can't wait out a fall.".to_owned()
            }
            (SafeBecause::AnyDay, Lang::He, _) => {
                "בטוח: כסף שאולי תצטרכו בכל יום לא יכול לחכות שהבורסה תתאושש.".to_owned()
            }
            (
                SafeBecause::Short | SafeBecause::KnownDate | SafeBecause::Rules,
                Lang::En,
                Some(past),
            ) => {
                let span = span(past.months, lang);
                let windows = grouped(past.windows);
                let since = past.since;
                match past.below_deposited {
                    0 => format!(
                        "Safe. In stocks, in none of {windows} stretches of {span} since {since} would less have been left at the end than was put in."
                    ),
                    below => format!(
                        "Safe. In stocks, in {} of {windows} stretches of {span} since {since}, less would have been left at the end than was put in.",
                        grouped(below)
                    ),
                }
            }
            (
                SafeBecause::Short | SafeBecause::KnownDate | SafeBecause::Rules,
                Lang::He,
                Some(past),
            ) => {
                let span = span(past.months, lang);
                let windows = grouped(past.windows);
                let since = past.since;
                match past.below_deposited {
                    0 => format!(
                        "בטוח. במניות, באף אחת מ\u{2011}{windows} תקופות של {span} מאז {since} לא היה נשאר בסוף פחות ממה שהופקד."
                    ),
                    below => format!(
                        "בטוח. במניות, ב\u{2011}{} מתוך {windows} תקופות של {span} מאז {since} היה נשאר בסוף פחות ממה שהופקד.",
                        grouped(below)
                    ),
                }
            }
            (SafeBecause::Short | SafeBecause::KnownDate | SafeBecause::Rules, Lang::En, None) => {
                format!("Safe. Over {when}, stocks can be down just when the money is needed.")
            }
            (SafeBecause::Short | SafeBecause::KnownDate | SafeBecause::Rules, Lang::He, None) => {
                format!("בטוח. ב\u{2011}{when}, מניות יכולות להיות בירידה בדיוק כשצריך את הכסף.")
            }
            (SafeBecause::Chosen, Lang::En, _) => "Safe: as you chose.".to_owned(),
            (SafeBecause::Chosen, Lang::He, _) => "בטוח: כמו שבחרתם.".to_owned(),
            (SafeBecause::Cushion, Lang::En, _) => format!(
                "Safe: putting the cushion aside takes all of the {when}, so all the money goes to it, where it can be reached any day."
            ),
            (SafeBecause::Cushion, Lang::He, _) => format!(
                "בטוח: הכרית לוקחת את כל ה\u{2011}{when}, אז כל הכסף הולך אליה, למקום שאפשר להגיע אליו בכל יום."
            ),
        }
    })
}

/// A portfolio at a bank: the place isn't asked (finding 2).
#[must_use]
pub fn place_existing() -> Text {
    fixed(
        "What's already in the portfolio: the question is what holding it costs, not where to put the money.",
        "מה שכבר בתיק: השאלה היא כמה עולה להחזיק אותו, לא לאן להכניס את הכסף.",
    )
}

/// The past of the index's lead, on the basis of the figures beside it
/// (finding 3): after fees and tax, in today's money, for the visitor's
/// own deposits.
#[derive(Debug, Clone, Copy)]
pub struct Past {
    pub below_deposited: u32,
    pub below_safe: Option<u32>,
    pub windows: u32,
    pub months: u32,
    pub since: i32,
}

/// "(after fees and tax, in today's money, by the S&P 500 index)", the
/// basis every statement about the past beside a figure is on: the S&P
/// 500 is what was measured (§2.13 item 1).
fn on_basis(lang: Lang) -> &'static str {
    lang.pick(
        "(after fees and tax, in today's money, by the S&P 500 index)",
        "(אחרי עמלות ומס, בכסף של היום, לפי מדד S&P\u{a0}500)",
    )
}

/// "In 135 of 1,110 stretches of 5 years since 1926, the fund would have
/// left less than was put in", on the figures' basis; never rounded to
/// "never" (principle 16).
fn below_deposited_sentence(past: &Past, lang: Lang) -> String {
    let years = span(past.months, lang);
    let windows = grouped(past.windows);
    let since = past.since;
    match (past.below_deposited, lang) {
        (0, Lang::En) => format!(
            "In none of {windows} stretches of {years} since {since} would less have been left than was put in {}.",
            on_basis(lang)
        ),
        (0, Lang::He) => format!(
            "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} לא היה נשאר פחות ממה שהופקד {}.",
            on_basis(lang)
        ),
        (lower, Lang::En) => format!(
            "In {} of {windows} stretches of {years} since {since}, less would have been left than was put in {}.",
            grouped(lower),
            on_basis(lang)
        ),
        (lower, Lang::He) => format!(
            "ב\u{2011}{} מתוך {windows} תקופות של {years} מאז {since} היה נשאר פחות ממה שהופקד {}.",
            grouped(lower),
            on_basis(lang)
        ),
    }
}

/// The figures of the stated index line.
#[derive(Debug, Clone, Copy)]
pub struct PlaceIndexFigures {
    pub months: u32,
    pub past: Option<Past>,
    /// The best safe way.
    pub safe: Option<Kind>,
}

/// The index, stated: even its worst past stretch beat the best safe way by
/// more than the margin, said as the past and on the figures' basis. The
/// figures themselves are the chart's (each number once, principle 13).
#[must_use]
pub fn place_index(f: &PlaceIndexFigures) -> Text {
    both(|lang| {
        let when = span(f.months, lang);
        let mut out = match lang {
            Lang::En => format!("Stocks, in a broad index: {when}."),
            Lang::He => format!("מניות, במדד מניות רחב: {when}."),
        };
        if let (Some(past), Some(kind)) = (f.past, f.safe) {
            let name = super::outcome::name(kind);
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!(
                    "Even in the worst stretch of {when} since {}, stocks would have left clearly more than {} {}.",
                    past.since,
                    name.en.to_lowercase(),
                    on_basis(lang)
                ),
                Lang::He => format!(
                    "גם בתקופה הגרועה ביותר של {when} מאז {}, מניות היו משאירות בבירור יותר מ{} {}.",
                    past.since,
                    name.he,
                    on_basis(lang)
                ),
            });
            if past.below_deposited > 0 {
                out.push(' ');
                out.push_str(&below_deposited_sentence(&past, lang));
            }
        }
        out
    })
}

// ─────────────────────────── The decisions ───────────────────────────

/// A stretch said as roughly as it's known: 80 months is "about 7 years".
fn rough_months(months: u32) -> u32 {
    if months >= 24 {
        (months + 6) / 12 * 12
    } else {
        months
    }
}

/// The figures of the cushion's decision.
#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "facts about the cushion, each its own"
)]
pub struct CushionFigures {
    pub monthly: Decimal,
    pub target: Decimal,
    pub months_to_fill: u32,
    pub expenses: Decimal,
    pub stand_in: bool,
    /// What each answer leaves at the date, today's money.
    pub has: Option<Decimal>,
    pub has_not: Option<Decimal>,
    /// The cushion takes all the time there is.
    pub takes_all: bool,
    pub index_return: Percent,
}

#[must_use]
pub fn cushion_decision(f: &CushionFigures) -> DecisionWords {
    let months = rough_months(f.months_to_fill);
    let left = |value: Option<Decimal>| {
        both(move |lang| match value {
            Some(value) => format!(
                "{} {}",
                roughly(value, lang),
                if_index_rises(f.index_return, lang)
            ),
            None => String::new(),
        })
    };
    let has_not_facts = if f.takes_all {
        vec![
            both(|lang| match lang {
                Lang::En => format!(
                    "{} a month to a money market fund: the cushion takes all the time there is.",
                    shekels(f.monthly)
                ),
                Lang::He => format!(
                    "{} בחודש לקרן כספית: הכרית לוקחת את כל הזמן שיש.",
                    shekels(f.monthly)
                ),
            }),
            expenses_fact(f),
        ]
    } else {
        vec![
            both(|lang| match lang {
                Lang::En => format!(
                    "First {} a month to a money market fund, for about {}, until {} is aside; it stays there.",
                    shekels(f.monthly),
                    span(months, lang),
                    shekels(f.target)
                ),
                Lang::He => format!(
                    "קודם {} בחודש לקרן כספית, בערך {}, עד שיש בצד {}; הם נשארים שם.",
                    shekels(f.monthly),
                    span(months, lang),
                    shekels(f.target)
                ),
            }),
            expenses_fact(f),
            both(|lang| match lang {
                Lang::En => format!(
                    "Then the {} goes to a fund on a broad stock index.",
                    shekels(f.monthly)
                ),
                Lang::He => format!(
                    "אחר כך ה\u{2011}{} עוברים לקרן על מדד מניות רחב.",
                    shekels(f.monthly)
                ),
            }),
        ]
    };
    DecisionWords {
        question: fixed(
            "Do you already have 3 months of expenses set aside?",
            "יש לכם כבר כסף בצד ל\u{2011}3 חודשי הוצאות?",
        ),
        why: fixed(
            "Without a cushion, a sudden expense can force a sale of stocks in a fall. Only you know what you have aside.",
            "בלי כרית, הוצאה פתאומית יכולה להכריח למכור מניות דווקא בירידה. רק אתם יודעים מה יש לכם בצד.",
        ),
        options: vec![
            OptWords {
                title: fixed("I have a cushion", "יש לי כרית"),
                amount: left(f.has),
                facts: vec![
                    both(|lang| match lang {
                        Lang::En => format!(
                            "All {} to a fund on a broad stock index, from this month.",
                            shekels(f.monthly)
                        ),
                        Lang::He => format!(
                            "כל ה\u{2011}{} לקרן על מדד מניות רחב, מהחודש הזה.",
                            shekels(f.monthly)
                        ),
                    }),
                    fixed("The cushion stays where it is.", "הכרית נשארת איפה שהיא."),
                ],
                fits_if: None,
                also: None,
                button: fixed("I have one", "יש לי"),
            },
            OptWords {
                title: fixed("Not yet", "עוד לא"),
                amount: if f.takes_all {
                    both(|lang| match f.has_not {
                        Some(value) => format!(
                            "{} {}",
                            roughly(value, lang),
                            lang.pick("at today's rate", "בריבית של היום")
                        ),
                        None => String::new(),
                    })
                } else {
                    left(f.has_not)
                },
                facts: has_not_facts,
                fits_if: None,
                also: None,
                button: fixed("Not yet", "עוד לא"),
            },
        ],
        difference: f.has.zip(f.has_not).map(|(has, has_not)| {
            both(|lang| match lang {
                Lang::En => format!(
                    "The difference: about {} less at the date with the cushion first, in return for money ready for an emergency.",
                    about(has - has_not)
                ),
                Lang::He => format!(
                    "ההבדל: עם הכרית קודם, בערך {} פחות בסוף, בתמורה לכסף זמין לחירום.",
                    about(has - has_not)
                ),
            })
        }),
    }
}

/// Whose expenses the cushion is worked out by.
fn expenses_fact(f: &CushionFigures) -> Text {
    both(|lang| match (lang, f.stand_in) {
        (Lang::En, true) => format!(
            "By expenses of {} a month, a stand-in: enter yours.",
            shekels(f.expenses)
        ),
        (Lang::He, true) => format!(
            "לפי הוצאות של {} בחודש, הערכה: כתבו את שלכם.",
            shekels(f.expenses)
        ),
        (Lang::En, false) => format!("By your expenses of {} a month.", shekels(f.expenses)),
        (Lang::He, false) => format!("לפי ההוצאות שלכם, {} בחודש.", shekels(f.expenses)),
    })
}

/// The figures of the place type's decision.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "facts about the question, each its own"
)]
pub struct PlaceTypeFigures {
    pub no_date: bool,
    pub today: Option<Date>,
    pub months: u32,
    /// The safe side's lowest and highest, the fund's rate range included.
    pub safe_low: Option<Decimal>,
    pub safe_high: Option<Decimal>,
    /// The best safe way, as things are.
    pub safe_best: Option<Decimal>,
    pub growth: Option<Decimal>,
    pub index_return: Percent,
    /// The worst stretch, on the figures' basis, ends below the deposits.
    pub below_deposited: bool,
    pub past: Option<Past>,
    /// The way the growth option holds the index in.
    pub lead: Option<Way>,
    /// The safe kinds, best first.
    pub safe_kinds: Vec<Kind>,
    /// What the money market fund leaves at the rates tried.
    pub fund_range: Option<(Decimal, Decimal)>,
    pub retirement: bool,
    /// For retirement: the pension the growth option would pay.
    pub annuity: Option<Text>,
}

/// Where a way is bought (finding 10): the same words for each kind bought
/// in a securities account.
fn bought_in_account() -> Text {
    fixed(
        "Bought in a securities account, at a bank or an investment house.",
        "נקנית בחשבון ניירות ערך, בבנק או בבית השקעות.",
    )
}

fn if_index_rises(rate: Percent, lang: Lang) -> String {
    match lang {
        Lang::En => format!("if the broad stock index rises {rate} a year"),
        Lang::He => format!("אם מדד המניות הרחב יעלה {rate} בשנה"),
    }
}

/// The same, inside an option whose title already names the index: "המדד"
/// refers back to it (§2.8, amended 2026-10-20).
fn it_rises(rate: Percent, lang: Lang) -> String {
    match lang {
        Lang::En => format!("if the index rises {rate} a year"),
        Lang::He => format!("אם המדד יעלה {rate} בשנה"),
    }
}

/// Each safe kind with how sure its sum is (finding 6), as one fact.
fn safe_kinds_fact(kinds: &[Kind]) -> Text {
    both(|lang| {
        let parts: Vec<String> = kinds
            .iter()
            .map(|kind| {
                let name = super::outcome::name(*kind);
                let sure = match kind {
                    // Its range is the option's amount: said once.
                    Kind::MoneyMarketFund => {
                        lang.pick("the rate changes", "הריבית משתנה").to_owned()
                    }
                    Kind::GovernmentBond { linked: true } | Kind::LinkedDeposit => lang
                        .pick("buying power known in advance", "כוח הקנייה ידוע מראש")
                        .to_owned(),
                    _ => lang.pick("known in advance", "ידוע מראש").to_owned(),
                };
                let name = match lang {
                    Lang::En => name.en.to_string(),
                    Lang::He => name.he.to_string(),
                };
                format!("{name} ({sure})")
            })
            .collect();
        format!("{}{}.", lang.pick("In it: ", "בפנים: "), parts.join(", "))
    })
}

/// What holds the index in the growth option, as a sentence's subject: a
/// provident fund is "הקופה", not "הקרן" (the review's N5). Each is
/// feminine in Hebrew, so the verbs after it stay as they are.
fn holder(lead: Option<Way>, lang: Lang) -> &'static str {
    match lead.map(|lead| lead.vehicle) {
        Some(Vehicle::InvestmentGemel) => lang.pick("the provident fund", "הקופה"),
        Some(Vehicle::StudyFund) => lang.pick("the study fund", "קרן ההשתלמות"),
        Some(Vehicle::SavingsPolicy) => lang.pick("the policy", "הפוליסה"),
        _ => lang.pick("the fund", "הקרן"),
    }
}

/// The growth option's past, on the figures' basis (finding 3): how often
/// it would have left less than the best safe way, and less than was put
/// in, said with the numbers that make it a question.
fn past_fact(past: &Past, lead: Option<Way>, lang: Lang) -> String {
    let holder = holder(lead, lang);
    let years = span(past.months, lang);
    let windows = grouped(past.windows);
    let since = past.since;
    let deposited = match (past.below_deposited, lang) {
        (0, Lang::En) => " In none of them less than was put in.".to_owned(),
        (0, Lang::He) => " באף אחת מהן לא פחות ממה שהופקד.".to_owned(),
        (lower, Lang::En) => format!(" In {} of them, less than was put in too.", grouped(lower)),
        (lower, Lang::He) => format!(" ב\u{2011}{} מהן גם פחות ממה שהופקד.", grouped(lower)),
    };
    match (past.below_safe, lang) {
        (Some(0) | None, Lang::En) => format!(
            "Even in the worst stretch of {years} since {since}, {holder} would have left about as much as the best safe way, not clearly more {}.",
            on_basis(lang)
        ),
        (Some(0) | None, Lang::He) => format!(
            "בתקופה הגרועה ביותר של {years} מאז {since}, {holder} הייתה משאירה בערך כמו הטובה שבדרכים הבטוחות, לא יותר ממנה בבירור {}.",
            on_basis(lang)
        ),
        (Some(below), Lang::En) => format!(
            "In {} of {windows} stretches of {years} since {since}, {holder} would have left less than the best safe way {}.{deposited}",
            grouped(below),
            on_basis(lang)
        ),
        (Some(below), Lang::He) => format!(
            "ב\u{2011}{} מתוך {windows} תקופות של {years} מאז {since}, {holder} הייתה משאירה פחות מהטובה שבדרכים הבטוחות {}.{deposited}",
            grouped(below),
            on_basis(lang)
        ),
    }
}

#[must_use]
pub fn place_type_decision(f: &PlaceTypeFigures) -> DecisionWords {
    let question = if f.no_date {
        both(|lang| match lang {
            Lang::En => format!(
                "Can the money wait at least {}, even if the market is down?",
                span(f.months, lang)
            ),
            Lang::He => format!(
                "הכסף יכול לחכות לפחות {}, גם אם הבורסה בירידה?",
                span(f.months, lang)
            ),
        })
    } else {
        both(|lang| match lang {
            Lang::En => format!(
                "If the market is down {}, could you wait another year or two?",
                in_year(f.today, f.months, lang)
            ),
            Lang::He => format!(
                "אם {} הבורסה תהיה בירידה, תוכלו לחכות עוד שנה\u{2011}שנתיים?",
                in_year(f.today, f.months, lang)
            ),
        })
    };
    let mut safe_facts = vec![safe_kinds_fact(&f.safe_kinds)];
    if f.safe_kinds.len() > 1 {
        safe_facts.push(fixed(
            "After this choice you'll choose which safe way.",
            "אחרי הבחירה הזאת תבחרו איזו דרך בטוחה.",
        ));
    }
    if f.retirement {
        // What the safe way gives up here (the review's X6): the provident
        // fund's pension, free of tax on the gain.
        safe_facts.push(fixed(
            "Outside a provident fund: no pension free of tax on the gain; the tax is 25% of the gain above inflation.",
            "מחוץ לקופת גמל: בלי קצבה פטורה ממס על הרווח; המס הוא 25% מהרווח שמעבר לאינפלציה.",
        ));
    }
    let mut growth_facts = Vec::new();
    if let Some(past) = &f.past {
        growth_facts.push(both(|lang| past_fact(past, f.lead, lang)));
    }
    // A fund company's way is drawn on, not sold (the review's N5); a
    // study fund's money is locked for its first years.
    match f.lead.map(|lead| lead.vehicle) {
        Some(Vehicle::StudyFund) => {}
        Some(Vehicle::InvestmentGemel | Vehicle::SavingsPolicy) => growth_facts.push(fixed(
            "Can be drawn on any day. If there's a fall, you wait a year or two.",
            "אפשר למשוך בכל יום. אם יש ירידה, מחכים שנה\u{2011}שנתיים.",
        )),
        _ => growth_facts.push(fixed(
            "Can be sold any day. If there's a fall, you wait a year or two.",
            "אפשר למכור בכל יום. אם יש ירידה, מחכים שנה\u{2011}שנתיים.",
        )),
    }
    if f.lead
        .is_some_and(|lead| lead.vehicle == Vehicle::Brokerage)
    {
        growth_facts.push(bought_in_account());
    }
    let difference = f.growth.zip(f.safe_best).map(|(growth, safe)| {
        both(|lang| {
            let rises = if_index_rises(f.index_return, lang);
            let gap = about(growth - safe);
            // A fund on the index, or the wrapper the option names.
            let holder = match f.lead {
                Some(lead) if lead.vehicle != Vehicle::Brokerage => holder(Some(lead), lang).to_owned(),
                _ => lang.pick("a fund on it", "קרן על המדד").to_owned(),
            };
            match (lang, f.below_deposited) {
                (Lang::En, true) => format!("{rises}, {holder} would leave about {gap} more than the best of the safe ways. In return: in a bad stretch it can leave less than they do, and even less than was put in, in the very year you need the money."),
                (Lang::He, true) => format!("{rises}, {holder} תשאיר בערך {gap} יותר מהטובה שבדרכים הבטוחות. בתמורה: בתקופה גרועה היא יכולה להשאיר פחות ממנה, ואפילו פחות ממה שהפקדתם, דווקא בשנה שתצטרכו את הכסף."),
                (Lang::En, false) => format!("{rises}, {holder} would leave about {gap} more than the best of the safe ways. In return: in a bad stretch it can leave less than they do, in the very year you need the money."),
                (Lang::He, false) => format!("{rises}, {holder} תשאיר בערך {gap} יותר מהטובה שבדרכים הבטוחות. בתמורה: בתקופה גרועה היא יכולה להשאיר פחות ממנה, דווקא בשנה שתצטרכו את הכסף."),
            }
        })
    });
    let growth_title = match f.lead {
        // The wrapper apart from what it holds (§2.8).
        Some(lead) if lead.vehicle != Vehicle::Brokerage => both(|lang| match lang {
            Lang::En => format!("Growth: {}", super::outcome::name(Kind::Way(lead)).en),
            Lang::He => format!("צמיחה: {}", super::outcome::name(Kind::Way(lead)).he),
        }),
        _ => fixed(
            "Growth: a fund on a broad stock index",
            "צמיחה: קרן על מדד מניות רחב",
        ),
    };
    DecisionWords {
        question,
        why: if f.no_date {
            fixed(
                "Only you know when you might need it. That decides between safety and growth.",
                "רק אתם יודעים מתי אולי תצטרכו אותו. זה מה שמכריע בין ביטחון לצמיחה.",
            )
        } else {
            fixed(
                "Only you know whether the date can move. That decides between safety and growth.",
                "רק אתם יודעים אם התאריך יכול לזוז. זה מה שמכריע בין ביטחון לצמיחה.",
            )
        },
        options: vec![
            OptWords {
                title: fixed("Safe", "בטוח"),
                amount: both(|lang| safe_range(f.safe_low, f.safe_high, lang)),
                facts: safe_facts,
                fits_if: None,
                also: None,
                button: if f.no_date {
                    fixed("No, I may need it sooner", "לא, אולי אצטרך אותו קודם")
                } else {
                    fixed(
                        "No, I'll need it exactly then",
                        "לא, אצטרך את הכסף בדיוק אז",
                    )
                },
            },
            OptWords {
                title: growth_title,
                amount: both(|lang| match f.growth {
                    Some(growth) => {
                        format!(
                            "{} {}",
                            roughly(growth, lang),
                            it_rises(f.index_return, lang)
                        )
                    }
                    None => String::new(),
                }),
                facts: growth_facts,
                fits_if: None,
                also: f.annuity.clone(),
                button: fixed("Yes, it can wait", "כן, אפשר לחכות"),
            },
        ],
        difference,
    }
}

#[must_use]
/// The safe way, stated: the only one that fits (`only`), or the one that
/// leaves the most by far in every version.
pub fn safe_way_stated(kind: Kind, months: Option<u32>, only: bool) -> Text {
    let name = super::outcome::name(kind);
    // Money that may be needed any day has no term to say.
    let Some(months) = months else {
        return both(|lang| format!("{}: {}", &name[lang], &super::path::safe_way_why(only)[lang]));
    };
    both(|lang| {
        let when = span(months, lang);
        match (lang, only) {
            (Lang::En, true) => format!("{}: for {when}, the only safe way here.", name.en),
            (Lang::He, true) => format!("{}: לטווח של {when}, זו הדרך הבטוחה היחידה כאן.", name.he),
            (Lang::En, false) => format!(
                "{}: for {when}, the safe way that leaves the most, by far, in every version we checked.",
                name.en
            ),
            (Lang::He, false) => format!(
                "{}: לטווח של {when}, זו הדרך הבטוחה שמשאירה הכי הרבה, בפער, בכל גרסה שבדקנו.",
                name.he
            ),
        }
    })
}

/// One option of the safe way's decision.
#[derive(Debug, Clone, Copy)]
pub struct SafeOption {
    pub way: SafeWay,
    pub kind: Kind,
    pub left: Decimal,
    pub variant: Option<(Kind, Decimal)>,
    pub rate_range: Option<(Decimal, Decimal)>,
}

/// What a bank leaves for a product (finding 9): the visitor's, once named,
/// or the banks' range while not.
#[derive(Debug, Clone)]
pub enum AtBank {
    Named(Text, Decimal),
    Range(Decimal, Decimal),
}

#[derive(Debug, Clone)]
pub struct SafeWayFigures<'a> {
    pub options: &'a [SafeOption],
    pub today: Option<Date>,
    pub months: u32,
    pub dated: bool,
    pub rate: Percent,
    pub switch: Option<Percent>,
    pub bond_at_bank: Option<AtBank>,
    /// The order changes within the versions tried: then which leaves more
    /// depends on a rate nobody knows, and the why says so.
    pub flips: bool,
}

#[must_use]
#[allow(clippy::too_many_lines, reason = "one option for each safe way")]
pub fn safe_way_decision(f: &SafeWayFigures<'_>) -> DecisionWords {
    let has_fund = f
        .options
        .iter()
        .any(|option| option.way == SafeWay::MoneyMarketFund);
    let high = f
        .options
        .iter()
        .map(|option| option.left)
        .max()
        .unwrap_or_default();
    let low = f
        .options
        .iter()
        .map(|option| option.left)
        .min()
        .unwrap_or_default();
    // "Knowing today" is the other option's only where its sum is known.
    let other_known = f
        .options
        .iter()
        .any(|option| option.way != SafeWay::MoneyMarketFund && known(Some(option.kind)));
    let question = if has_fund && f.options.len() == 2 && other_known {
        fixed(
            "What matters more to you: the simplest, or knowing today how much there will be?",
            "מה חשוב לכם יותר: הכי פשוט, או לדעת כבר היום כמה יהיה?",
        )
    } else {
        fixed("Which safe way?", "איזו דרך בטוחה?")
    };
    // How far apart they are is the difference's, under the options, except
    // for three ways, where the difference is only the first two's.
    let apart = |lang: Lang| {
        if f.options.len() > 2 {
            let gap = if has_fund {
                about(high - low)
            } else {
                shekels(high - low)
            };
            match lang {
                Lang::En => format!(
                    "They're at most {gap} apart over {}. ",
                    span(f.months, lang)
                ),
                Lang::He => format!("ההבדל ביניהן עד {gap} על פני {}. ", span(f.months, lang)),
            }
        } else {
            String::new()
        }
    };
    let why = both(|lang| match (lang, has_fund && f.flips) {
        (Lang::En, true) => format!(
            "{}The difference in money is small, and depends on a rate nobody knows in advance. So it's about what suits you, not about which leaves more.",
            apart(lang)
        ),
        (Lang::He, true) => format!(
            "{}ההבדל בכסף קטן, ותלוי בריבית שאף אחד לא יודע מראש. לכן זה עניין של מה שנוח לכם, ולא של מה שמשאיר יותר.",
            apart(lang)
        ),
        (Lang::En, false) => format!(
            "{}What sets them apart is locking against flexibility, and knowing the sum against letting it move with the rate.",
            apart(lang)
        ),
        (Lang::He, false) => format!(
            "{}מה שמבדיל ביניהן הוא נעילה מול גמישות, וסכום ידוע מול סכום שזז עם הריבית.",
            apart(lang)
        ),
    });
    let in_year = |lang: Lang| {
        if f.dated {
            in_year(f.today, f.months, lang)
        } else {
            format!("{} {}", lang.pick("in", "בעוד"), span(f.months, lang))
        }
    };
    let options = f
        .options
        .iter()
        .map(|option| match option.way {
            SafeWay::Deposit if option.kind == Kind::AnyDayDeposit => OptWords {
                title: super::outcome::name(option.kind),
                amount: both(|lang| format!("{} {}", roughly(option.left, lang), lang.pick("at today's rate", "בריבית של היום"))),
                facts: vec![
                    fixed("It stays at your bank: nothing to open.", "נשאר בבנק שלכם: לא צריך לפתוח כלום."),
                    fixed("Can be taken out any day. The rate is variable, and the bank sets it.", "אפשר למשוך בכל יום. הריבית משתנה, והבנק קובע אותה."),
                ],
                fits_if: Some(fixed("Fits if: keeping the money at your bank matters to you.", "מתאים אם: חשוב לכם שהכסף יישאר בבנק שלכם.")),
                also: None,
                button: fixed("A deposit", "פיקדון"),
            },
            SafeWay::Bond if option.kind == Kind::TreasuryBill => OptWords {
                title: super::outcome::name(option.kind),
                amount: both(|lang| format!("{}, {}", shekels(option.left), lang.pick("known today, held to the end", "ידוע כבר היום, אם מחזיקים עד הסוף"))),
                facts: vec![
                    bought_in_account(),
                    fixed("Can be sold any trading day, at that day's price, which moves a little.", "אפשר למכור בכל יום מסחר, במחיר של אותו יום, שזז מעט."),
                ],
                fits_if: Some(fixed("Fits if: you have a securities account, or will open one.", "מתאים אם: יש לכם חשבון ניירות ערך, או שתפתחו אחד.")),
                also: None,
                button: fixed("Makam", "מק״מ"),
            },
            SafeWay::MoneyMarketFund => OptWords {
                title: if f.options.len() == 2 && other_known {
                    fixed("The simplest: a money market fund", "הכי פשוט: קרן כספית")
                } else {
                    fixed("A money market fund", "קרן כספית")
                },
                amount: both(|lang| format!("{} {}", roughly(option.left, lang), lang.pick("at today's rate", "בריבית של היום"))),
                facts: {
                    let mut facts = vec![
                        fixed("In the bank's app, or at any investment house.", "באפליקציה של הבנק, או בכל בית השקעות."),
                        fixed("Can be sold any trading day.", "אפשר למכור בכל יום מסחר."),
                    ];
                    if let Some((low, high)) = option.rate_range {
                        facts.push(both(|lang| {
                            let up = Percent(f.rate.0 + crate::guide::RATE_STEP);
                            let down = Percent((f.rate.0 - crate::guide::RATE_STEP).max(Decimal::ZERO));
                            match lang {
                                Lang::En => format!("The sum moves with the Bank of Israel's rate: {} if it averages {up}, {} if {down}.", about(high), about(low)),
                                Lang::He => format!("הסכום זז עם ריבית בנק ישראל: {} אם תהיה בממוצע {up}, ו\u{2011}{} אם {down}.", about(high), about(low)),
                            }
                        }));
                    }
                    facts
                },
                fits_if: Some(fixed("Fits if: you want as little fuss as possible.", "מתאים אם: אתם רוצים כמה שפחות התעסקות.")),
                also: None,
                button: fixed("A money market fund", "קרן כספית"),
            },
            SafeWay::Bond => {
                let name = super::outcome::name(option.kind);
                let known = match option.kind {
                    Kind::GovernmentBond { linked: true } => fixed("buying power known today", "כוח הקנייה ידוע כבר היום"),
                    _ => fixed("known today", "ידוע כבר היום"),
                };
                let mut facts = vec![
                    bought_in_account(),
                    both(|lang| match lang {
                        Lang::En => format!("Bought each month in the same series, held until it's repaid {}.", in_year(lang)),
                        Lang::He => format!("קונים את אותה סדרה, ומחזיקים עד הפדיון {}.", in_year(lang)),
                    }),
                ];
                if let Some((variant, left)) = option.variant {
                    let variant = super::outcome::name(variant);
                    facts.push(both(|lang| match lang {
                        Lang::En => format!("There's also {}: {}.", variant.en, shekels(left)),
                        Lang::He => format!("יש גם {}: {}.", variant.he, shekels(left)),
                    }));
                }
                OptWords {
                    title: both(|lang| match lang {
                        Lang::En => format!("Known in advance: {}", name.en),
                        Lang::He => format!("ידוע מראש: {}", name.he),
                    }),
                    amount: both(|lang| format!("{}, {}", shekels(option.left), &known[lang])),
                    facts,
                    fits_if: Some(fixed("Fits if: knowing today how much there will be matters to you.", "מתאים אם: חשוב לכם לדעת היום כמה יהיה בסוף.")),
                    also: None,
                    button: fixed("Know in advance", "לדעת מראש"),
                }
            }
            SafeWay::Deposit => OptWords {
                title: both(|lang| match lang {
                    Lang::En => format!("A fixed-rate deposit for {}", span(f.months, lang)),
                    Lang::He => format!("פיקדון בריבית קבועה ל\u{2011}{}", span(f.months, lang)),
                }),
                amount: both(|lang| format!("{}, {}", shekels(option.left), lang.pick("known today", "ידוע כבר היום"))),
                facts: vec![
                    fixed("Opened at a bank. At a bank other than yours, you open an account there first.", "נפתח בבנק. בבנק אחר משלכם צריך לפתוח שם חשבון."),
                    both(|lang| match lang {
                        Lang::En => format!("The money is locked until the end, {}.", in_year(lang)),
                        Lang::He => format!("הכסף נעול עד הסוף, {}.", in_year(lang)),
                    }),
                ],
                fits_if: Some(both(|lang| match lang {
                    Lang::En => format!("Fits if: you won't touch the money before the end, {}.", in_year(lang)),
                    Lang::He => format!("מתאים אם: לא תצטרכו לגעת בכסף לפני הסוף, {}.", in_year(lang)),
                })),
                also: None,
                button: fixed("A deposit", "פיקדון"),
            },
        })
        .collect();
    let lead = f.options.first();
    let fund = f
        .options
        .iter()
        .find(|option| option.way == SafeWay::MoneyMarketFund);
    // The difference names the pair it measures (the review's N13), and the
    // switch point is said against the fund only when that pair has it.
    let difference = lead.map(|lead| {
        both(|lang| {
            let mut out = String::new();
            let second = f.options.get(1);
            if let Some(second) = second {
                let (lead_name, second_name) = (
                    super::outcome::name(lead.kind),
                    super::outcome::name(second.kind),
                );
                let gap = if !known(Some(lead.kind)) || !known(Some(second.kind)) {
                    roughly(lead.left - second.left, lang)
                } else {
                    shekels(lead.left - second.left)
                };
                out = match lang {
                    Lang::En => format!("At today's rate, {} leaves {gap} more than {}.", lead_name.en, second_name.en.to_lowercase()),
                    Lang::He => format!("בריבית של היום, {} משאירה {gap} יותר מ{}.", lead_name.he, second_name.he),
                };
            }
            let pair_has_fund = lead.way == SafeWay::MoneyMarketFund
                || second.is_some_and(|second| second.way == SafeWay::MoneyMarketFund);
            if let Some(switch) = f.switch.filter(|_| pair_has_fund) {
                let fund_leads = lead.way == SafeWay::MoneyMarketFund;
                out.push(' ');
                out.push_str(&match (lang, fund_leads) {
                    (Lang::En, false) => format!("If the rate averages above about {switch}, the money market fund leaves more."),
                    (Lang::He, false) => format!("אם הריבית תהיה בממוצע מעל כ\u{2011}{switch}, הקרן הכספית משאירה יותר."),
                    (Lang::En, true) => format!("If the rate averages below about {switch}, the fund leaves less than the others."),
                    (Lang::He, true) => format!("אם הריבית תהיה בממוצע מתחת לכ\u{2011}{switch}, הקרן הכספית משאירה פחות."),
                });
            }
            if let (Some(bank), Some(fund)) = (&f.bond_at_bank, fund) {
                out.push(' ');
                out.push_str(&match bank {
                    AtBank::Named(name, left) => {
                        let less = *left < fund.left;
                        match (lang, less) {
                            (Lang::En, true) => format!("Through your bank, {}, a bond would leave {}, less than the money market fund.", name.en, shekels(*left)),
                            (Lang::He, true) => format!("דרך הבנק שלכם, {}, אג״ח הייתה משאירה {}, פחות מהקרן הכספית.", name.he, shekels(*left)),
                            (Lang::En, false) => format!("Through your bank, {}, a bond would leave {}.", name.en, shekels(*left)),
                            (Lang::He, false) => format!("דרך הבנק שלכם, {}, אג״ח הייתה משאירה {}.", name.he, shekels(*left)),
                        }
                    }
                    AtBank::Range(low, high) => {
                        let tail = if *high < fund.left {
                            lang.pick(", less than the money market fund at every bank", ", בכל בנק פחות מהקרן הכספית")
                        } else if *low < fund.left {
                            lang.pick(", at the dearer banks less than the money market fund", ", ובבנקים היקרים פחות מהקרן הכספית")
                        } else {
                            ""
                        };
                        match lang {
                            Lang::En => format!("Through a bank, a bond would leave {} to {}, by the bank{tail}.", shekels(*low), shekels(*high)),
                            Lang::He => format!("דרך בנק, אג״ח הייתה משאירה בין {} ל\u{2011}{}, לפי הבנק{tail}.", shekels(*low), shekels(*high)),
                        }
                    }
                });
            }
            out.trim().to_owned()
        })
    });
    DecisionWords {
        question,
        why,
        options,
        difference: difference.filter(|text| !text.is_empty()),
    }
}

/// The figures of the study fund's decision.
#[derive(Debug, Clone, Copy)]
pub struct WrapperFigures {
    pub lead: Way,
    pub lead_left: Option<Decimal>,
    pub study_left: Option<Decimal>,
    pub index_return: Percent,
    pub monthly: Decimal,
    /// The study fund's edge is mostly the tax it doesn't pay.
    pub mostly_tax: bool,
}

/// A way's short name, for a question or a button.
pub(crate) fn way_short(way: Way, lang: Lang) -> &'static str {
    match (way.product, way.vehicle) {
        (_, Vehicle::InvestmentGemel) => {
            lang.pick("a provident fund for investment", "קופת גמל להשקעה")
        }
        (_, Vehicle::SavingsPolicy) => lang.pick("a savings policy", "פוליסת חיסכון"),
        (_, Vehicle::StudyFund) => lang.pick("a study fund", "קרן השתלמות"),
        (Product::IsraeliIndexFund, _) => lang.pick("an index fund", "קרן מחקה"),
        _ => lang.pick("an ETF", "קרן סל"),
    }
}

#[must_use]
pub fn wrapper_decision(f: &WrapperFigures) -> DecisionWords {
    let lead_name = super::outcome::name(Kind::Way(f.lead));
    let amount = |left: Option<Decimal>| {
        both(move |lang| match left {
            Some(left) => format!("{} {}", roughly(left, lang), it_rises(f.index_return, lang)),
            None => String::new(),
        })
    };
    let opens = if f.lead.vehicle == Vehicle::Brokerage {
        fixed(
            "Anyone can buy one, in a securities account at a bank or an investment house.",
            "כל אחד יכול לקנות, בחשבון ניירות ערך בבנק או בבית השקעות.",
        )
    } else {
        fixed(
            "Anyone can open one, at a fund company.",
            "כל אחד יכול לפתוח, בחברה מנהלת.",
        )
    };
    DecisionWords {
        question: both(|lang| match lang {
            Lang::En => format!("Where will the money build up: {}, or a study fund of yours?", way_short(f.lead, lang)),
            Lang::He => format!("איפה ייצבר הכסף: {}, או קרן השתלמות שלכם?", way_short(f.lead, lang)),
        }),
        why: fixed(
            "A study fund is open only to the self-employed or through an employer. Only you know whether you have one you can add to.",
            "קרן השתלמות פתוחה רק לעצמאים או דרך מעסיק. רק אתם יודעים אם יש לכם אחת שאפשר להוסיף לה.",
        ),
        options: vec![
            OptWords {
                title: lead_name.clone(),
                amount: amount(f.lead_left),
                facts: vec![opens],
                fits_if: Some(fixed("Fits if: you have no study fund you can add to.", "מתאים אם: אין לכם קרן השתלמות שאפשר להוסיף לה.")),
                also: None,
                button: both(|lang| {
                    let short = way_short(f.lead, lang);
                    let mut chars = short.chars();
                    match (lang, chars.next()) {
                        (Lang::En, Some(first)) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        _ => short.to_owned(),
                    }
                }),
            },
            OptWords {
                title: fixed("Study fund, stock track", "קרן השתלמות, במסלול מניות"),
                amount: amount(f.study_left),
                facts: vec![
                    fixed("Only for the self-employed, or through an employer.", "רק לעצמאים, או דרך מעסיק."),
                    fixed("The gain is tax-free after 6 years.", "הרווח פטור ממס אחרי 6 שנים."),
                    fixed("Worked out for the self-employed. For employees: [₪—].", "החישוב לעצמאי. לשכירים: [₪—]."),
                ],
                fits_if: Some(both(|lang| match lang {
                    Lang::En => format!("Fits if: you have one, and can add {} a month to it.", shekels(f.monthly)),
                    Lang::He => format!("מתאים אם: יש לכם קרן כזאת, ואפשר להוסיף לה {} בחודש.", shekels(f.monthly)),
                })),
                also: None,
                button: fixed("I have a study fund", "יש לי קרן השתלמות"),
            },
        ],
        difference: f.lead_left.zip(f.study_left).map(|(lead, study)| {
            both(|lang| match (lang, f.mostly_tax) {
                (Lang::En, true) => format!("If the study fund is open to you, it leaves about {} more, mostly from the tax exemption.", about(study - lead)),
                (Lang::He, true) => format!("אם קרן ההשתלמות זמינה לכם, היא משאירה בערך {} יותר, בעיקר בגלל הפטור ממס.", about(study - lead)),
                (Lang::En, false) => format!("If the study fund is open to you, it leaves about {} more.", about(study - lead)),
                (Lang::He, false) => format!("אם קרן ההשתלמות זמינה לכם, היא משאירה בערך {} יותר.", about(study - lead)),
            })
        }),
    }
}

#[must_use]
pub fn index_way_stated(lead: Way, tied_with: Option<Way>) -> Text {
    let name = super::outcome::name(Kind::Way(lead));
    both(|lang| {
        let how = if lead.vehicle == Vehicle::Brokerage {
            lang.pick(", in a securities account", ", בחשבון ניירות ערך")
        } else {
            ""
        };
        match (tied_with, lang) {
            (Some(other), Lang::En) => format!(
                "{}{how}. About the same is left by: {}.",
                name.en,
                super::outcome::name(Kind::Way(other)).en
            ),
            (Some(other), Lang::He) => format!(
                "{}{how}. בערך אותו סכום משאירה גם: {}.",
                name.he,
                super::outcome::name(Kind::Way(other)).he
            ),
            (None, Lang::En) => format!("{}{how}.", name.en),
            (None, Lang::He) => format!("{}{how}.", name.he),
        }
    })
}

/// The figures of the account's decision.
#[derive(Debug, Clone)]
pub struct AccountFigures<'a> {
    /// What is bought: its sum known to the shekel, or not.
    pub kind: Option<Kind>,
    pub tied: &'a [Text],
    pub best: Decimal,
    pub best_cost: Option<Percent>,
    /// The visitor's bank, once named.
    pub yours: Option<&'a Text>,
    pub yours_left: Option<Decimal>,
    pub yours_cost: Option<Percent>,
    pub range: Option<(Decimal, Decimal)>,
    pub gap: Decimal,
    pub years: u32,
}

/// Names joined as "A, B or C".
fn names(names: &[Text], lang: Lang) -> String {
    let all: Vec<&str> = names.iter().map(|name| &name[lang]).collect();
    match all.as_slice() {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [init @ .., last] => format!("{} {} {last}", init.join(", "), lang.pick("or", "או")),
    }
}

/// The names after a Hebrew preposition joined to each ("באקסלנס, באלטשולר
/// או במיטב"): the ה of a name that has one goes ("בבינלאומי", "לפועלים").
/// In English, the names alone.
fn names_after(preposition: char, list: &[Text], lang: Lang) -> String {
    if lang == Lang::En {
        return names(list, lang);
    }
    let joined: Vec<Text> = list
        .iter()
        .map(|name| Text::owned(name.en.to_string(), joined_to(preposition, &name.he)))
        .collect();
    names(&joined, lang)
}

/// A Hebrew name after a one-letter preposition: "באקסלנס", and "בבינלאומי"
/// for "הבינלאומי", whose ה the preposition takes. Before Latin letters or
/// a digit it takes a non-breaking hyphen, "ב‑IBI": joined straight on, the
/// line reads backwards (the review's N5).
pub(crate) fn joined_to(preposition: char, name: &str) -> String {
    let hebrew = name
        .chars()
        .next()
        .is_some_and(|first| ('\u{5d0}'..='\u{5ea}').contains(&first));
    if hebrew {
        format!("{preposition}{}", name.strip_prefix('ה').unwrap_or(name))
    } else {
        format!("{preposition}\u{2011}{name}")
    }
}

fn cost_fact(cost: Option<Percent>) -> Option<Text> {
    cost.map(|cost| {
        let cost = Percent(cost.0.round_dp(2));
        both(|lang| match lang {
            Lang::En => format!("Fees and the fund's own cost: about {cost} a year."),
            Lang::He => format!("העמלות והעלות של הקרן: בערך {cost} בשנה."),
        })
    })
}

#[must_use]
pub fn account_decision(f: &AccountFigures<'_>) -> DecisionWords {
    // Nothing is the visitor's bank until they name it (finding 9).
    let bank_title = both(|lang| match (f.yours, lang) {
        (Some(yours), Lang::En) => format!("Through your bank: {}", yours.en),
        (Some(yours), Lang::He) => format!("דרך הבנק שלכם: {}", yours.he),
        (None, _) => lang.pick("Through a bank", "דרך בנק").to_owned(),
    });
    let bank_amount = both(|lang| match (f.yours, f.yours_left, f.range) {
        (Some(_), Some(left), _) => figure_said(f.kind, left, lang),
        (_, _, Some((low, high))) => format!(
            "{}{}",
            figures_between(f.kind, low, high, lang),
            lang.pick(", by the bank", ", לפי הבנק")
        ),
        _ => String::new(),
    });
    let mut new_facts = vec![fixed(
        "You open a new account there: a one-time chore.",
        "פותחים שם חשבון חדש: טרחה של פעם אחת.",
    )];
    new_facts.extend(cost_fact(f.best_cost));
    let mut bank_facts = vec![fixed(
        "In the bank's securities account: if you already have one, it's one more order.",
        "בחשבון ניירות הערך בבנק: אם כבר יש לכם אחד, זו עוד פקודה.",
    )];
    bank_facts.extend(cost_fact(f.yours_cost));
    DecisionWords {
        question: match f.yours {
            Some(_) => fixed(
                "Open a new account, or buy through your bank?",
                "לפתוח חשבון חדש, או לקנות דרך הבנק שלכם?",
            ),
            None => fixed(
                "Open a new account, or buy through a bank?",
                "לפתוח חשבון חדש, או לקנות דרך בנק?",
            ),
        },
        why: fixed(
            "Opening an account is a one-time chore; the fees are paid every year. Only you can say whether the chore is worth the difference.",
            "פתיחת חשבון היא טרחה של פעם אחת, והעמלות משולמות כל שנה. רק אתם יודעים אם הטרחה שווה לכם את ההפרש.",
        ),
        options: vec![
            OptWords {
                title: both(|lang| match lang {
                    Lang::En => format!("A new account at {}", names(f.tied, lang)),
                    Lang::He => format!("חשבון חדש {}", names_after('ב', f.tied, lang)),
                }),
                amount: both(|lang| figure_said(f.kind, f.best, lang)),
                facts: new_facts,
                fits_if: None,
                also: None,
                button: fixed("Open an account", "לפתוח חשבון"),
            },
            OptWords {
                title: bank_title,
                amount: bank_amount,
                facts: bank_facts,
                fits_if: None,
                also: None,
                button: fixed("Stay at the bank", "להישאר בבנק"),
            },
        ],
        difference: Some(both(|lang| {
            let years = span(f.years * 12, lang);
            // Until the bank is named, against the banks' range, both ends
            // (the review's N4): from the bank that leaves most to the one
            // that leaves least.
            let gap = match (f.yours, f.range) {
                (None, Some((low, high))) => format!(
                    "{}{}",
                    figures_between(f.kind, f.best - high, f.best - low, lang),
                    lang.pick(" by the bank", " לפי הבנק")
                ),
                _ => figure_said(f.kind, f.gap, lang),
            };
            match lang {
                Lang::En => format!("The difference: {gap}, over {years}, mostly fees."),
                Lang::He => format!("ההבדל: {gap}, לאורך {years}, רובו עמלות."),
            }
        })),
    }
}

/// The figures of the portfolio's decision.
#[derive(Debug, Clone)]
pub struct HoldingsFigures<'a> {
    /// What moving would save: against the named bank, or from the cheapest
    /// branch plan to the dearest while none is named (low, high).
    pub saves: Option<(Decimal, Decimal)>,
    pub margin: Decimal,
    pub tied: &'a [Text],
    pub months: u32,
    pub named: Option<&'a Text>,
}

/// What moving saves, in words: "בערך ₪145,000", or a range by the bank.
fn saves_words(saves: (Decimal, Decimal), lang: Lang) -> String {
    let (low, high) = saves;
    if about(low) == about(high) {
        roughly(low, lang)
    } else {
        match lang {
            Lang::En => format!(
                "about {} to {}, by the bank",
                about(low.max(Decimal::ZERO)),
                about(high)
            ),
            Lang::He => format!(
                "בין {} ל\u{2011}{} בערך, לפי הבנק",
                about(low.max(Decimal::ZERO)),
                about(high)
            ),
        }
    }
}

#[must_use]
pub fn holdings_decision(f: &HoldingsFigures<'_>) -> DecisionWords {
    DecisionWords {
        question: fixed("What's in the portfolio?", "מה יש בתיק?"),
        why: fixed(
            "Only you know what you bought. That decides whether moving saves anything.",
            "רק אתם יודעים מה קניתם. זה מה שקובע אם מעבר יחסוך משהו.",
        ),
        options: vec![
            OptWords {
                title: fixed("Index funds or shares", "קרנות מחקות או מניות"),
                amount: both(|lang| match f.saves {
                    Some(saves) if saves.1 > f.margin => match lang {
                        Lang::En => format!(
                            "Moving them to {} saves, over {}, {}",
                            names(f.tied, lang),
                            span(f.months, lang),
                            saves_words(saves, lang)
                        ),
                        Lang::He => format!(
                            "העברה {} חוסכת לאורך {} {}",
                            names_after('ל', f.tied, lang),
                            span(f.months, lang),
                            saves_words(saves, lang)
                        ),
                    },
                    _ => lang
                        .pick("Moving saves little here", "כאן מעבר לא חוסך הרבה")
                        .to_owned(),
                }),
                facts: vec![
                    fixed(
                        "Moved as they are: not sold, so no tax.",
                        "עוברים כמו שהם: לא מוכרים, ולכן אין מס.",
                    ),
                    match f.named {
                        Some(bank) => both(|lang| match lang {
                            Lang::En => format!("Against {}'s branch price list.", bank.en),
                            Lang::He => format!("לעומת תעריף הסניף של {}.", bank.he),
                        }),
                        None => fixed(
                            "Against the banks' branch price lists, from the cheapest to the dearest.",
                            "לעומת תעריפי הסניף של הבנקים, מהזול ליקר.",
                        ),
                    },
                ],
                fits_if: None,
                also: None,
                button: fixed("Index funds or shares", "קרנות מחקות או מניות"),
            },
            OptWords {
                title: fixed(
                    "Managed funds, or portfolio management",
                    "קרנות מנוהלות, או ניהול תיקים",
                ),
                amount: fixed(
                    "Their fees aren't in the calculation",
                    "דמי הניהול שלהן לא בחישוב",
                ),
                facts: vec![
                    fixed(
                        "A fund's management fee is charged wherever it's held: moving alone doesn't lower it.",
                        "דמי הניהול של קרן נגבים בכל מקום שמחזיקים אותה: מעבר לבד לא מוריד אותם.",
                    ),
                    fixed(
                        "What to check: each fund's management fee, and the portfolio manager's fee.",
                        "מה לבדוק: דמי הניהול של כל קרן, ודמי ניהול התיק.",
                    ),
                ],
                fits_if: None,
                also: None,
                button: fixed("Managed funds", "קרנות מנוהלות"),
            },
            OptWords {
                title: fixed("Don't know", "לא יודעים"),
                amount: fixed("Worth checking first", "שווה לבדוק קודם"),
                facts: vec![fixed(
                    "In the bank's app, under the securities portfolio, each holding is listed with its kind.",
                    "באפליקציה של הבנק, בתיק ניירות הערך, רשום כל נייר וסוגו.",
                )],
                fits_if: None,
                also: None,
                button: fixed("Don't know", "לא יודעים"),
            },
        ],
        difference: None,
    }
}

// ─────────────────────────── Where ───────────────────────────

/// Where a money market fund is bought (finding 10): a bank's app, the
/// visitor's once named, or any investment house.
#[must_use]
pub fn where_fund(bank: Option<&Text>) -> Text {
    both(|lang| {
        let at = match (bank, lang) {
            (Some(bank), Lang::En) => format!("In {}'s app, or at any investment house", bank.en),
            (Some(bank), Lang::He) => {
                format!("באפליקציה של {}, או בכל בית השקעות", bank.he)
            }
            (None, _) => lang
                .pick(
                    "In the bank's app, or at any investment house",
                    "באפליקציה של הבנק, או בכל בית השקעות",
                )
                .to_owned(),
        };
        format!(
            "{at}. {}",
            lang.pick(
                "A bank may not charge custody on a money market fund.",
                "בנק לא רשאי לגבות דמי משמרת על קרן כספית.",
            )
        )
    })
}

/// Where a money market fund is bought, short, as a step's choice:
/// "באפליקציה של הפועלים".
#[must_use]
pub fn where_fund_short(bank: Option<&Text>) -> Text {
    both(|lang| match (bank, lang) {
        (Some(bank), Lang::En) => format!("In {}'s app", bank.en),
        (Some(bank), Lang::He) => format!("באפליקציה של {}", bank.he),
        (None, _) => lang
            .pick("In the bank's app", "באפליקציה של הבנק")
            .to_owned(),
    })
}

/// Where a fund company's way is, short, as a step's choice.
#[must_use]
pub fn where_fund_company_short(vehicle: Vehicle) -> Text {
    match vehicle {
        Vehicle::StudyFund => fixed("In your study fund", "בקרן ההשתלמות שלכם"),
        _ => fixed("At a fund company", "בחברה מנהלת"),
    }
}

/// Names as a list: "A, B או C".
#[must_use]
pub fn names_text(list: &[Text]) -> Text {
    both(|lang| names(list, lang))
}

/// The figures of a deposit's place.
#[derive(Debug, Clone)]
pub struct DepositWhere<'a> {
    /// The banks within the providers' margin of the best.
    pub tied: &'a [Text],
    /// Their highest less their lowest.
    pub spread: Decimal,
    /// The visitor's bank, once named.
    pub yours: Option<&'a Text>,
    /// How far it is behind the best.
    pub behind: Option<Decimal>,
    pub yours_tied: bool,
}

#[must_use]
pub fn where_deposit(f: &DepositWhere<'_>) -> Text {
    both(|lang| {
        let mut out = match (f.tied.len(), lang) {
            (1, _) => format!("{}.", names(f.tied, lang)),
            (_, Lang::En) => format!(
                "A tie: {}, at most {} apart.",
                names(f.tied, lang),
                shekels(f.spread)
            ),
            (_, Lang::He) => format!(
                "תיקו: {}, ההפרש ביניהם עד {}.",
                names(f.tied, lang),
                shekels(f.spread)
            ),
        };
        out.push(' ');
        out.push_str(lang.pick(
            "The rates in the figures are averages, not offers: ask each bank what you'll get.",
            "הריביות בחישוב הן ממוצעים, לא הצעות: שאלו כל בנק מה תקבלו.",
        ));
        // From the visitor's own bank (principle 15): a deposit elsewhere
        // means an account there.
        if let (Some(yours), Some(behind)) = (f.yours, f.behind)
            && !f.yours_tied
            && behind > Decimal::ZERO
        {
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!(
                    "At your bank, {}: {} less. At another bank, you open an account there first.",
                    yours.en,
                    shekels(behind)
                ),
                Lang::He => format!(
                    "בבנק שלכם, {}: {} פחות. בבנק אחר צריך לפתוח שם חשבון.",
                    yours.he,
                    shekels(behind)
                ),
            });
        }
        out
    })
}

#[must_use]
pub fn where_fund_company(vehicle: Vehicle) -> Text {
    match vehicle {
        Vehicle::StudyFund => fixed(
            "In your study fund, on a stock track.",
            "בקרן ההשתלמות שלכם, במסלול מניות.",
        ),
        _ => fixed(
            "At a fund company, on a stock track. The figures use the companies' average fee: ask what you'll pay.",
            "בחברה מנהלת, במסלול מניות. החישוב לפי דמי הניהול הממוצעים: שאלו כמה תשלמו.",
        ),
    }
}

/// The figures of a stated איפה line.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "facts about the line, each its own"
)]
pub struct WhereFigures<'a> {
    /// What is bought: its sum known to the shekel, or not.
    pub kind: Option<Kind>,
    /// The names the line gives: the visitor's bank first when it joins.
    pub tied: &'a [Text],
    pub spread: Decimal,
    /// What the best place leaves.
    pub best: Decimal,
    /// The best less the visitor's bank (or the best bank while none is
    /// named).
    pub gap: Option<Decimal>,
    pub yours: Option<&'a Text>,
    /// The visitor's bank is among the names (`tied`).
    pub joined: bool,
    /// What the banks leave, while none is named.
    pub range: Option<(Decimal, Decimal)>,
    /// The margin between kinds: within it, the bank is no question.
    pub margin: Decimal,
    /// Through the bank it leaves no more than a money market fund.
    pub moot: bool,
}

#[must_use]
pub fn where_stated(f: &WhereFigures<'_>) -> Text {
    both(|lang| {
        let mut out = match f.tied.len() {
            1 => format!("{}.", names(f.tied, lang)),
            _ => format!(
                "{}: {}{}",
                names(f.tied, lang),
                apart(f.spread, lang).trim_end_matches('.'),
                lang.pick(", so any is fine.", ", אז כל אחד בסדר.")
            ),
        };
        if f.joined {
            return out;
        }
        let Some(gap) = f.gap else { return out };
        // Until the bank is named, against the banks' range, both ends (the
        // review's N4): from the bank that leaves most to the one that
        // leaves least.
        let gaps = f.range.map(|(low, high)| (f.best - high, f.best - low));
        let sentence = match (f.yours, gaps) {
            (Some(yours), _) if f.moot => match lang {
                Lang::En => format!(
                    "Against your bank, {}: {} more.",
                    yours.en,
                    figure_said(f.kind, gap, lang)
                ),
                Lang::He => format!(
                    "לעומת הבנק שלכם, {}: {} יותר.",
                    yours.he,
                    figure_said(f.kind, gap, lang)
                ),
            },
            (None, Some((low, high))) if f.moot => match lang {
                Lang::En => format!(
                    "Against a bank: {} more, by the bank.",
                    figures_between(f.kind, low, high, lang)
                ),
                Lang::He => format!(
                    "לעומת בנק: {} יותר, לפי הבנק.",
                    figures_between(f.kind, low, high, lang)
                ),
            },
            (None, Some((_, high))) if high <= Decimal::ONE => lang
                .pick(
                    "Through a bank it's about the same.",
                    "גם דרך בנק כמעט אותו סכום.",
                )
                .to_owned(),
            (None, Some((_, high))) if high <= f.margin => match lang {
                Lang::En => format!(
                    "Through a bank it's nearly the same: at most {} less.",
                    figure(f.kind, high)
                ),
                Lang::He => format!(
                    "גם דרך בנק כמעט אותו סכום: עד {} פחות.",
                    figure(f.kind, high)
                ),
            },
            (None, Some((low, high))) => match lang {
                Lang::En => format!(
                    "Through a bank: {} less, by the bank.",
                    figures_between(f.kind, low, high, lang)
                ),
                Lang::He => format!(
                    "דרך בנק: {} פחות, לפי הבנק.",
                    figures_between(f.kind, low, high, lang)
                ),
            },
            // Named and within the margin, the bank joined the names (or
            // is in the tie group already).
            (Some(_), _) | (None, None) => return out,
        };
        out.push(' ');
        out.push_str(&sentence);
        out
    })
}

/// "באינטראקטיב, אקסלנס או מיטב": names after "ב", for a sentence.
#[must_use]
pub fn in_names(list: &[Text]) -> Text {
    both(|lang| names_after('ב', list, lang))
}

/// A way's short name, both languages: "קרן סל".
#[must_use]
pub fn way_short_text(way: Way) -> Text {
    both(|lang| way_short(way, lang).to_owned())
}

/// "דרך בנק": the banks as one row (§4.7).
#[must_use]
pub fn through_a_bank() -> Text {
    fixed("Through a bank", "דרך בנק")
}

/// Beside it: "נקודת השוואה, מהיקר ועד מזרחי · אונליין".
#[must_use]
pub fn bank_reference_note(cheapest: Option<&Text>) -> Text {
    both(|lang| match (cheapest, lang) {
        (Some(name), Lang::En) => format!("the reference point, from the dearest to {}", name.en),
        (Some(name), Lang::He) => format!("נקודת השוואה, מהיקר ועד {}", name.he),
        (None, _) => lang.pick("the reference point", "נקודת השוואה").to_owned(),
    })
}

/// Two figures as a range, at the kind's precision: "בערך ₪322,000 עד
/// ₪339,000", or one where they round alike.
#[must_use]
pub fn range_said(kind: Option<Kind>, low: Decimal, high: Decimal) -> Text {
    both(|lang| {
        let (low_text, high_text) = (figure(kind, low), figure(kind, high));
        if low_text == high_text {
            figure_said(kind, low, lang)
        } else if known(kind) {
            match lang {
                Lang::En => format!("{low_text} to {high_text}"),
                Lang::He => format!("{low_text} עד {high_text}"),
            }
        } else {
            match lang {
                Lang::En => format!("about {low_text} to {high_text}"),
                Lang::He => format!("בערך {low_text} עד {high_text}"),
            }
        }
    })
}

/// The difference where to buy makes (§4.7): "באינטראקטיב: בין ₪88,500
/// ל‑₪105,000 יותר מאשר דרך בנק, לאורך 30 שנים.", or against the visitor's
/// bank once named. One rounding for every amount: the kind's.
#[must_use]
pub fn where_difference(
    kind: Option<Kind>,
    best: &[Text],
    gap: (Decimal, Decimal),
    yours: Option<&Text>,
    months: u32,
) -> Text {
    both(|lang| {
        let at = match lang {
            Lang::En => format!("At {}", names(best, lang)),
            Lang::He => names_after('ב', best, lang),
        };
        let more = figures_between(kind, gap.0.max(Decimal::ZERO), gap.1, lang);
        let over = span(months, lang);
        match (yours, lang) {
            (Some(bank), Lang::En) => format!(
                "{at}: {more} more than at your bank ({}), over {over}.",
                bank.en
            ),
            (Some(bank), Lang::He) => format!(
                "{at}: {more} יותר מאשר בבנק שלכם ({}), לאורך {over}.",
                bank.he
            ),
            (None, Lang::En) => format!("{at}: {more} more than through a bank, over {over}."),
            (None, Lang::He) => format!("{at}: {more} יותר מאשר דרך בנק, לאורך {over}."),
        }
    })
}

/// "איפה לקנות קרן סל על מדד מניות רחב".
#[must_use]
pub fn where_title(yours: bool, existing: bool, name: &Text) -> Text {
    if !existing {
        return both(|lang| match (yours, lang) {
            (true, Lang::En) => format!("Where to buy {}, against your bank", name.en.to_lowercase()),
            (true, Lang::He) => format!("איפה לקנות {}, לעומת הבנק שלכם", name.he),
            (false, Lang::En) => format!("Where to buy {}", name.en.to_lowercase()),
            (false, Lang::He) => format!("איפה לקנות {}", name.he),
        });
    }
    match (existing, yours) {
        (true, true) => fixed(
            "Where to move it, against your bank's branch price list",
            "לאן להעביר, לעומת תעריף הסניף של הבנק שלכם",
        ),
        (true, false) => fixed(
            "Where to move it, against the banks' branch price lists",
            "לאן להעביר, לעומת תעריפי הסניף של הבנקים",
        ),
        (false, true) => fixed(
            "Where to buy, against your bank",
            "איפה לקנות, לעומת הבנק שלכם",
        ),
        (false, false) => fixed("Where to buy", "איפה לקנות"),
    }
}

/// "At most ₪400 apart.", or the same sum where they're within a shekel.
fn apart(spread: Decimal, lang: Lang) -> String {
    if spread < Decimal::ONE {
        lang.pick("They leave the same.", "הם משאירים אותו סכום.")
            .to_owned()
    } else {
        match lang {
            Lang::En => format!("They're at most {} apart.", shekels(spread)),
            Lang::He => format!("ההפרש ביניהם עד {}.", shekels(spread)),
        }
    }
}

/// Under where to buy: the tie group's size and its spread, its highest
/// less its lowest (finding 10).
#[must_use]
pub fn where_note(tied: usize, spread: Decimal) -> Option<Text> {
    (tied > 1).then(|| {
        both(|lang| match (lang, spread < Decimal::ONE) {
            (Lang::En, true) => format!("These {tied} leave the same, so any of them is fine."),
            (Lang::He, true) => {
                format!("ה\u{2011}{tied} האלה משאירים אותו סכום, אז כל אחד מהם בסדר.")
            }
            (Lang::En, false) => format!(
                "These {tied} are at most {} apart, so any of them is fine.",
                shekels(spread)
            ),
            (Lang::He, false) => format!(
                "ההפרש בין ה\u{2011}{tied} האלה הוא עד {}, אז כל אחד מהם בסדר.",
                shekels(spread)
            ),
        })
    })
}

/// The figures of a portfolio's move.
#[derive(Debug, Clone)]
pub struct TransferFigures<'a> {
    pub tied: &'a [Text],
    pub spread: Decimal,
    pub saves: Option<(Decimal, Decimal)>,
    pub margin: Decimal,
    pub named: Option<&'a Text>,
    pub months: u32,
}

/// The move, stated (finding 2): the holdings as they are, no sale, no tax.
#[must_use]
pub fn where_transfer(f: &TransferFigures<'_>) -> Text {
    both(|lang| {
        let worth = f.saves.is_some_and(|(_, high)| high > f.margin);
        let to = match lang {
            Lang::En => names(f.tied, lang),
            Lang::He => names_after('ל', f.tied, lang),
        };
        let mut out = match (worth, lang) {
            (true, Lang::En) => format!("Move the holdings as they are to {to}: no sale, no tax."),
            (true, Lang::He) => format!("להעביר את הניירות כמו שהם {to}: בלי מכירה ובלי מס."),
            (false, Lang::En) => {
                "Moving saves little here: the portfolio can stay where it is.".to_owned()
            }
            (false, Lang::He) => "כאן מעבר לא חוסך הרבה: התיק יכול להישאר איפה שהוא.".to_owned(),
        };
        if worth && f.tied.len() > 1 {
            out.push(' ');
            out.push_str(&apart(f.spread, lang));
        }
        if let Some(saves) = f.saves.filter(|_| worth) {
            out.push(' ');
            let span = span(f.months, lang);
            // "More is left" before the sum: a range ends with "by the
            // bank", which nothing may follow (the review's N5).
            out.push_str(&match (f.named, lang) {
                (Some(bank), Lang::En) => format!(
                    "Against {}'s branch price list, over {span}, more is left: {}.",
                    bank.en,
                    saves_words(saves, lang)
                ),
                (Some(bank), Lang::He) => format!(
                    "לעומת תעריף הסניף של {}, לאורך {span}, נשאר יותר: {}.",
                    bank.he,
                    saves_words(saves, lang)
                ),
                (None, Lang::En) => format!(
                    "Against the banks' branch price lists, over {span}, more is left: {}.",
                    saves_words(saves, lang)
                ),
                (None, Lang::He) => format!(
                    "לעומת תעריפי הסניף של הבנקים, לאורך {span}, נשאר יותר: {}.",
                    saves_words(saves, lang)
                ),
            });
        }
        out
    })
}

/// Where, for managed funds: what to check before any move.
#[must_use]
pub fn where_managed() -> Text {
    fixed(
        "Before moving, check each fund's management fee and the portfolio manager's: they're charged wherever the funds are held.",
        "לפני מעבר, בודקים את דמי הניהול של כל קרן ושל מנהל התיק: הם נגבים בכל מקום שמחזיקים את הקרנות.",
    )
}

/// Where, while the visitor doesn't know what's in the portfolio.
#[must_use]
pub fn where_dont_know() -> Text {
    fixed(
        "First see in the bank's app what the portfolio holds.",
        "קודם רואים באפליקציה של הבנק מה יש בתיק.",
    )
}

// ─────────────────────────── The plan ───────────────────────────

#[derive(Debug, Clone)]
pub struct PlanFigures<'a> {
    pub kind: Kind,
    pub now: Decimal,
    pub monthly: Decimal,
    pub today: Option<Date>,
    pub months: u32,
    /// The visitor gave the date.
    pub dated: bool,
    /// The cushion to fill first: its target and months.
    pub cushion: Option<(Decimal, u32)>,
    pub tied: &'a [Text],
    pub account: Option<Account>,
    /// The visitor's bank's place, once named.
    pub yours: Option<&'a Text>,
    /// The visitor's bank, once named.
    pub bank: Option<&'a Text>,
    pub holdings: Option<Holdings>,
}

fn deposits(now: Decimal, monthly: Decimal, lang: Lang) -> String {
    match (now.is_zero(), monthly.is_zero(), lang) {
        (false, false, Lang::En) => {
            format!("{} now and {} every month", shekels(now), shekels(monthly))
        }
        (false, false, Lang::He) => {
            format!("{} עכשיו ועוד {} בכל חודש", shekels(now), shekels(monthly))
        }
        (true, _, Lang::En) => format!("{} a month", shekels(monthly)),
        (true, _, Lang::He) => format!("{} בחודש", shekels(monthly)),
        (false, true, _) => shekels(now),
    }
}

#[must_use]
pub fn plan(f: &PlanFigures<'_>) -> Text {
    both(|lang| {
        let money = deposits(f.now, f.monthly, lang);
        let name = super::outcome::name(f.kind);
        // "לקרן סל על מדד מניות רחב", "to an ETF on a broad stock index".
        let to_it = match lang {
            Lang::En => {
                let mut chars = name.en.chars();
                let lowered: String = chars
                    .next()
                    .map(|first| first.to_lowercase().chain(chars).collect())
                    .unwrap_or_default();
                format!("to {lowered}")
            }
            Lang::He => joined_to('ל', &name.he),
        };
        let app = match (f.bank, lang) {
            (Some(bank), Lang::En) => format!("in {}'s app", bank.en),
            (Some(bank), Lang::He) => format!("באפליקציה של {}", bank.he),
            (None, _) => lang
                .pick("in the bank's app", "באפליקציה של הבנק")
                .to_owned(),
        };
        // The cushion first, in the same sentence (§4.5: the plan is one).
        let pre = match (f.cushion, lang) {
            (Some((target, _)), Lang::En) => format!(
                "First {} a month to a money market fund {app}, until {} is set aside, then ",
                shekels(f.monthly),
                shekels(target)
            ),
            (Some((target, _)), Lang::He) => format!(
                "קודם {} בחודש לקרן כספית {app}, עד שיצטברו {}, ואחר כך ",
                shekels(f.monthly),
                shekels(target)
            ),
            (None, _) => String::new(),
        };
        let at = names(f.tied, lang);
        let in_at = names_after('ב', f.tied, lang);
        let to_at = names_after('ל', f.tied, lang);
        let through = |lang: Lang| {
            f.yours.or(f.bank).map_or_else(
                || lang.pick("a bank", "בנק").to_owned(),
                |yours| yours[lang].to_string(),
            )
        };
        let main = match (f.kind, f.holdings, f.account, lang) {
            (_, Some(Holdings::IndexFunds), Some(Account::NewAccount), Lang::En) => format!("Move the holdings as they are to {at}: no sale, no tax."),
            (_, Some(Holdings::IndexFunds), Some(Account::NewAccount), Lang::He) => format!("מעבירים את הניירות כמו שהם {to_at}: בלי מכירה ובלי מס."),
            (_, Some(Holdings::IndexFunds), _, Lang::En) => "Keep the portfolio where it is: moving saves little here.".to_owned(),
            (_, Some(Holdings::IndexFunds), _, Lang::He) => "משאירים את התיק איפה שהוא: כאן מעבר לא חוסך הרבה.".to_owned(),
            (_, Some(Holdings::Managed), _, Lang::En) => "Check each fund's management fee and the manager's fee before moving: moving alone doesn't lower them.".to_owned(),
            (_, Some(Holdings::Managed), _, Lang::He) => "בודקים את דמי הניהול של כל קרן ושל ניהול התיק לפני מעבר: מעבר לבד לא מוריד אותם.".to_owned(),
            (_, Some(Holdings::DontKnow), _, Lang::En) => "First see in the bank's app what the portfolio holds; then come back to this answer.".to_owned(),
            (_, Some(Holdings::DontKnow), _, Lang::He) => "קודם רואים באפליקציה של הבנק מה יש בתיק, ואז חוזרים לתשובה הזאת.".to_owned(),
            (Kind::MoneyMarketFund, _, _, _) => format!("{money} {to_it}, {app}"),
            (Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit, _, _, Lang::En) => {
                format!("{money} {to_it}, at {at}")
            }
            (Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit, _, _, Lang::He) => {
                format!("{money} {to_it}, {in_at}")
            }
            (Kind::Way(way), _, _, Lang::En) if way.vehicle == Vehicle::StudyFund => {
                format!("{money} to your study fund, on a stock track")
            }
            (Kind::Way(way), _, _, Lang::He) if way.vehicle == Vehicle::StudyFund => {
                format!("{money} לקרן ההשתלמות שלכם, במסלול מניות")
            }
            (Kind::Way(way), _, _, Lang::En) if way.vehicle != Vehicle::Brokerage => {
                format!("{money} {to_it}, at a fund company")
            }
            (Kind::Way(way), _, _, Lang::He) if way.vehicle != Vehicle::Brokerage => {
                format!("{money} {to_it}, בחברה מנהלת")
            }
            (_, _, Some(Account::YourBank), Lang::En) => {
                format!("{money} {to_it}, through {}", through(lang))
            }
            (_, _, Some(Account::YourBank), Lang::He) => {
                format!("{money} {to_it}, דרך {}", through(lang))
            }
            (_, _, Some(Account::NewAccount), Lang::En) => {
                format!("{money} {to_it}, in a new account at {at}")
            }
            (_, _, Some(Account::NewAccount), Lang::He) => {
                format!("{money} {to_it}, בחשבון חדש {in_at}")
            }
            (_, _, None, Lang::En) => {
                format!("{money} {to_it}, in a securities account at {at}")
            }
            (_, _, None, Lang::He) => format!("{money} {to_it}, בחשבון ניירות ערך {in_at}"),
        };
        if f.holdings.is_some() {
            return main;
        }
        // After "then", the money starts lower-case in English.
        match lang {
            Lang::En if !pre.is_empty() => {
                let mut chars = main.chars();
                let rest: String = chars
                    .next()
                    .map(|first| first.to_lowercase().chain(chars).collect())
                    .unwrap_or_default();
                pre + &rest
            }
            _ => pre + &main,
        }
    })
}
