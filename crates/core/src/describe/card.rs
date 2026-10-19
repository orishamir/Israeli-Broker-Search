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
use crate::history::DepositsBand;
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
    pub facts: Vec<Text>,
    pub fits_if: Option<Text>,
    pub button: Text,
}

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A headline amount, rounded to match how sure it is (principle 13).
#[must_use]
pub fn about(amount: Decimal) -> String {
    let step = if amount.abs() >= Decimal::from(100_000) {
        Decimal::from(1000)
    } else if amount.abs() >= Decimal::from(10_000) {
        Decimal::from(100)
    } else {
        Decimal::from(10)
    };
    shekels((amount / step).round() * step)
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

/// "October 2029", or "in 3 years" without a date.
#[must_use]
pub fn date(today: Option<Date>, months: u32) -> Text {
    both(
        |lang| match today.and_then(|today| end_date(today, months)) {
            Some(end) => month_year(end, lang),
            None => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
        },
    )
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

/// "What you'll have in October 2029".
#[must_use]
pub fn outcome_title(today: Option<Date>, months: u32) -> Text {
    both(|lang| {
        format!(
            "{} {}",
            lang.pick("What you'll have", "מה יהיה לכם"),
            at_date(today, months, lang)
        )
    })
}

// ─────────────────────────── The chip, the title ───────────────────────────

#[must_use]
pub fn chip_note(state: ConfidenceState, arena: Option<Arena>, open: bool) -> Text {
    match (state, arena) {
        (ConfidenceState::C, _) => fixed(
            "There's no one right answer here. The decision below settles it.",
            "אין כאן תשובה אחת נכונה. ההחלטה למטה מכריעה.",
        ),
        (ConfidenceState::A, Some(Arena::Index)) if open => fixed(
            "The direction is clear. The decisions below are about the way.",
            "הכיוון ברור. ההחלטות למטה הן על הדרך.",
        ),
        (ConfidenceState::A, _) => fixed(
            "It comes out ahead in every version we checked.",
            "זה יוצא קדימה בכל גרסה שבדקנו.",
        ),
        (ConfidenceState::B, Some(Arena::Safe)) => fixed(
            "What matters is clear: not stocks. The rest depends on what suits you.",
            "מה שחשוב ברור: לא מניות. השאר תלוי במה שנוח לכם.",
        ),
        (ConfidenceState::B, _) => fixed(
            "The site's rules point to the index, but in the worst stretches of the past a safe way left more.",
            "לפי הכללים של האתר זה מדד מניות, אבל בתקופות הגרועות של העבר דרך בטוחה השאירה יותר.",
        ),
    }
}

/// The figures the title and body word.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "facts about the answer, each its own"
)]
pub struct TitleFigures {
    pub state: ConfidenceState,
    pub arena: Option<Arena>,
    pub asked: bool,
    pub today: Option<Date>,
    pub months: u32,
    pub fixed_date: bool,
    pub index_left: Option<Decimal>,
    pub index_return: Percent,
    pub fund: Option<Decimal>,
    pub current_account: Decimal,
    pub deposited: Decimal,
    pub safe_low: Option<Decimal>,
    pub safe_high: Option<Decimal>,
    pub safe_lead: Option<Kind>,
    pub open: bool,
    pub retirement: bool,
}

fn safe_range(low: Option<Decimal>, high: Option<Decimal>, lang: Lang) -> String {
    match (low, high) {
        (Some(low), Some(high)) if high - low >= Decimal::ONE => {
            format!(
                "{}{}–{}",
                lang.pick("about ", "בערך "),
                shekels(low),
                shekels(high)
            )
        }
        (_, Some(high)) => format!("{}{}", lang.pick("about ", "בערך "), about(high)),
        _ => String::new(),
    }
}

#[must_use]
pub fn title_and_body(f: &TitleFigures) -> (Text, Text) {
    let when = |lang| span(f.months, lang);
    if f.asked || f.state == ConfidenceState::C {
        let title = fixed(
            "Safety or growth: that's your decision.",
            "ביטחון או צמיחה: זו החלטה שלכם.",
        );
        let body = both(|lang| {
            let safe = safe_range(f.safe_low, f.safe_high, lang);
            match lang {
                Lang::En => format!(
                    "In {}, a safe way would leave {safe}, known almost in advance. A fund on the S&P 500 index can leave much more, but also less than you put in, in the very year you need the money.",
                    when(lang)
                ),
                Lang::He => format!(
                    "בעוד {}, דרך בטוחה תשאיר {safe}, ידוע כמעט מראש. קרן על מדד S&P\u{a0}500 יכולה להשאיר הרבה יותר, אבל גם פחות ממה ששמתם, דווקא בשנה שתצטרכו את הכסף.",
                    when(lang)
                ),
            }
        });
        return (title, body);
    }
    if let Some(Arena::Index) = f.arena {
        let title = both(|lang| match (f.state, lang) {
            (ConfidenceState::A, Lang::En) => format!("A stock index, for {}.", when(lang)),
            (ConfidenceState::A, Lang::He) => format!("מדד מניות, לטווח של {}.", when(lang)),
            (_, Lang::En) => format!("A stock index for {}, by the site's rules.", when(lang)),
            (_, Lang::He) => format!("מדד מניות ל\u{2011}{}, לפי הכללים של האתר.", when(lang)),
        });
        let body = both(|lang| {
            let mut out = String::new();
            if let Some(left) = f.index_left {
                let fund = f.fund.map_or_default(about);
                let cash = about(f.current_account);
                out = match lang {
                    Lang::En => format!(
                        "If the index rises {} a year, about {} would be left, against {fund} in a money market fund and {cash} in a current account.",
                        f.index_return,
                        about(left)
                    ),
                    Lang::He => format!(
                        "אם המדד יעלה {} בשנה, יישאר בערך {}, לעומת {fund} בקרן כספית ו\u{2011}{cash} בעו״ש.",
                        f.index_return,
                        about(left)
                    ),
                };
            }
            if f.open {
                out.push(' ');
                out.push_str(lang.pick(
                    "A few decisions on the way are left, and only you can make them.",
                    "נשארו כמה החלטות על הדרך, ורק אתם יכולים לקבל אותן.",
                ));
            }
            if f.retirement {
                out.push(' ');
                out.push_str(lang.pick(
                    "This comes on top of the pension from work, if you have one: its forecast is in your pension fund's yearly report.",
                    "וזה בנוסף לקצבה מהפנסיה מהעבודה, אם יש לכם: את התחזית שלה תמצאו בדוח השנתי של קרן הפנסיה.",
                ));
            }
            out.trim().to_owned()
        });
        (title, body)
    } else {
        let spread = f
            .safe_high
            .zip(f.safe_low)
            .map_or_default(|(high, low)| high - low);
        if f.state == ConfidenceState::A {
            let lead = f.safe_lead.map(super::outcome::name);
            let title = both(|lang| match (&lead, lang) {
                (Some(lead), Lang::En) => format!("What fits here: {}.", lead.en),
                (Some(lead), Lang::He) => format!("מה שמתאים כאן: {}.", lead.he),
                (None, _) => String::new(),
            });
            let body = both(|lang| match lang {
                Lang::En => format!(
                    "In every version we checked (the Bank of Israel's rate, inflation) it leaves more than the other safe ways: {} in {}, in today's money and after tax.",
                    f.safe_high.map_or_default(about),
                    when(lang)
                ),
                Lang::He => format!(
                    "בכל גרסה שבדקנו (ריבית בנק ישראל, אינפלציה) הוא משאיר יותר משאר הדרכים הבטוחות: {} בעוד {}, בכסף של היום ואחרי מס.",
                    f.safe_high.map_or_default(about),
                    when(lang)
                ),
            });
            return (title, body);
        }
        let title = fixed(
            "Not stocks. Among the safe ways, you choose.",
            "לא מניות. ובין הדרכים הבטוחות, אתם בוחרים.",
        );
        let body = both(|lang| match lang {
            Lang::En => format!(
                "The money is needed in {}{}. All the safe ways end within {} of each other, in today's money and after tax, so the choice between them is about what suits you.",
                when(lang),
                if f.fixed_date {
                    ", on a date that can't move"
                } else {
                    ""
                },
                shekels(spread)
            ),
            Lang::He => format!(
                "הכסף נדרש בעוד {}{}. כל הדרכים הבטוחות מסתיימות בטווח של {} זו מזו, בכסף של היום ואחרי מס, ולכן הבחירה ביניהן היא לפי מה שנוח לכם.",
                when(lang),
                if f.fixed_date {
                    ", בתאריך שלא זז"
                } else {
                    ""
                },
                shekels(spread)
            ),
        });
        (title, body)
    }
}

/// The monthly pension the sum could buy: the conversion coefficients
/// aren't sourced yet, so the figures are marked placeholders.
#[must_use]
pub fn annuity_placeholder() -> Text {
    fixed(
        "As a monthly pension: about [₪—]–[₪—] a month, by the large companies' conversion coefficients.",
        "כקצבה חודשית: בערך [₪—]–[₪—] בחודש, לפי מקדמי ההמרה של החברות הגדולות.",
    )
}

// ─────────────────────────── The lines ───────────────────────────

#[must_use]
pub fn label(kind: LineKind) -> Text {
    match kind {
        LineKind::Before => fixed("First", "קודם"),
        LineKind::PlaceType => fixed("Kind of place", "סוג המקום"),
        LineKind::Way => fixed("The way", "הדרך"),
        LineKind::Where => fixed("Where", "איפה"),
    }
}

#[must_use]
pub fn place_safe(because: SafeBecause, months: u32) -> Text {
    both(|lang| {
        let when = span(months, lang);
        match (because, lang) {
            (SafeBecause::AnyDay, Lang::En) => {
                "Safe: money you may need on any day can't wait out a fall.".to_owned()
            }
            (SafeBecause::AnyDay, Lang::He) => {
                "בטוח: כסף שאולי תצטרכו בכל יום לא יכול לחכות שהבורסה תתאושש.".to_owned()
            }
            (SafeBecause::Short | SafeBecause::KnownDate, Lang::En) => format!(
                "Safe, not stocks: {when}. Even if the date can move a little, {when} is too short for stocks under the site's rules."
            ),
            (SafeBecause::Short | SafeBecause::KnownDate, Lang::He) => format!(
                "בטוח, לא מניות: {when}. גם אם התאריך יכול לזוז קצת, {when} קצר מדי למניות לפי הכללים של האתר."
            ),
            (SafeBecause::Rules, Lang::En) => format!(
                "Safe, not stocks: under the site's rules, stocks start from 5 years with a date that can move, and {when} is less."
            ),
            (SafeBecause::Rules, Lang::He) => format!(
                "בטוח, לא מניות: לפי הכללים של האתר, מניות מתחילות מ\u{2011}5 שנים עם תאריך שיכול לזוז, ו\u{2011}{when} זה פחות."
            ),
            (SafeBecause::Chosen, Lang::En) => "Safe: as you chose.".to_owned(),
            (SafeBecause::Chosen, Lang::He) => "בטוח: כמו שבחרתם.".to_owned(),
        }
    })
}

/// "In 135 of 1,110 stretches of 5 years since 1926…", for the saver's own
/// deposits.
pub(crate) fn losses_sentence(history: &DepositsBand, monthly: bool, lang: Lang) -> String {
    let years = span(history.months, lang);
    let since = history.from.year();
    let windows = grouped(history.windows);
    let lower = grouped(history.lower);
    let what = |lang: Lang| {
        if monthly {
            lang.pick(
                "the deposits ended below what was put in",
                "הכסף שהופקד נגמר מתחת למה ששמו",
            )
        } else {
            lang.pick("the index ended at a loss", "המדד נגמר בהפסד")
        }
    };
    match (history.lower, lang) {
        (0, Lang::En) => format!(
            "In none of {windows} stretches of {years} since {since} did {} (in dollars, before fees and tax).",
            if monthly {
                "the deposits end below what was put in"
            } else {
                "the index end at a loss"
            }
        ),
        (0, Lang::He) => format!(
            "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} {} (בדולרים, לפני עמלות ומס).",
            if monthly {
                "הכסף שהופקד לא נגמר מתחת למה ששמו"
            } else {
                "המדד לא נגמר בהפסד"
            }
        ),
        (_, Lang::En) => format!(
            "In {lower} of {windows} stretches of {years} since {since}, {} (in dollars, before fees and tax).",
            what(lang)
        ),
        (_, Lang::He) => format!(
            "ב\u{2011}{lower} מתוך {windows} תקופות של {years} מאז {since}, {} (בדולרים, לפני עמלות ומס).",
            what(lang)
        ),
    }
}

/// The figures of the stated index line.
#[derive(Debug, Clone, Copy)]
pub struct PlaceIndexFigures {
    pub months: u32,
    pub history: Option<DepositsBand>,
    /// The index at its worst past stretch, after fees and tax, today's money.
    pub worst: Option<Decimal>,
    /// The best safe way, and what it leaves.
    pub safe: Option<(Kind, Decimal)>,
    pub deposited: Decimal,
}

/// The index, stated: even its worst past stretch beat the best safe way,
/// said in ₪ and as the past (many see stocks as betting).
#[must_use]
pub fn place_index(f: &PlaceIndexFigures) -> Text {
    both(|lang| {
        let when = span(f.months, lang);
        let head = match lang {
            Lang::En => format!("A stock index: {when}."),
            Lang::He => format!("מדד מניות: {when}."),
        };
        let mut out = head;
        if let (Some(worst), Some((kind, safe)), Some(history)) = (f.worst, f.safe, &f.history) {
            let since = history.from.year();
            let name = super::outcome::name(kind);
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!(
                    "Even in the worst stretch of {when} since {since}, the index would have left about {}, more than {} in {} (after fees and tax, in today's money).",
                    about(worst),
                    about(safe),
                    name.en.to_lowercase()
                ),
                Lang::He => format!(
                    "גם בתקופה הגרועה ביותר של {when} מאז {since}, המדד היה משאיר בערך {}, יותר מ\u{2011}{} ב{} (אחרי עמלות ומס, בכסף של היום).",
                    about(worst),
                    about(safe),
                    name.he
                ),
            });
            if worst < f.deposited {
                out.push(' ');
                out.push_str(&losses_sentence(history, false, lang));
            }
        } else if let Some(history) = &f.history {
            out.push(' ');
            out.push_str(&losses_sentence(history, false, lang));
        }
        out
    })
}

// ─────────────────────────── The decisions ───────────────────────────

#[must_use]
pub fn cushion_decision(
    monthly: Decimal,
    target: Decimal,
    months_to_fill: Decimal,
    expenses: Decimal,
    stand_in: bool,
) -> DecisionWords {
    let months = u32::try_from(months_to_fill.mantissa()).unwrap_or(0);
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
                amount: both(|lang| match lang {
                    Lang::En => format!("All {} to the index, from this month", shekels(monthly)),
                    Lang::He => format!("כל ה\u{2011}{} למדד, מהחודש הזה", shekels(monthly)),
                }),
                facts: vec![fixed(
                    "The cushion stays where it is.",
                    "הכרית נשארת איפה שהיא.",
                )],
                fits_if: None,
                button: fixed("I have one", "יש לי"),
            },
            OptWords {
                title: fixed("Not yet", "עוד לא"),
                amount: both(|lang| match lang {
                    Lang::En => format!("First {} for the cushion", shekels(target)),
                    Lang::He => format!("קודם {} לכרית", shekels(target)),
                }),
                facts: vec![
                    both(|lang| match lang {
                        Lang::En => format!(
                            "{} a month to a money market fund, for about {}.",
                            shekels(monthly),
                            span(months, lang)
                        ),
                        Lang::He => format!(
                            "{} בחודש לקרן כספית, בערך {}.",
                            shekels(monthly),
                            span(months, lang)
                        ),
                    }),
                    both(|lang| match (lang, stand_in) {
                        (Lang::En, true) => {
                            format!("By expenses of {} a month: enter yours.", shekels(expenses))
                        }
                        (Lang::He, true) => {
                            format!("לפי הוצאות של {} בחודש: כתבו את שלכם.", shekels(expenses))
                        }
                        (Lang::En, false) => {
                            format!("By your expenses of {} a month.", shekels(expenses))
                        }
                        (Lang::He, false) => {
                            format!("לפי ההוצאות שלכם, {} בחודש.", shekels(expenses))
                        }
                    }),
                    both(|lang| match lang {
                        Lang::En => format!("Then the {} goes to the index.", shekels(monthly)),
                        Lang::He => format!("אחר כך ה\u{2011}{} עוברים למדד.", shekels(monthly)),
                    }),
                ],
                fits_if: None,
                button: fixed("Not yet", "עוד לא"),
            },
        ],
        difference: Some(both(|lang| match lang {
            Lang::En => format!(
                "The difference: about {} less in the index, for money ready for an emergency.",
                span(months, lang)
            ),
            Lang::He => format!(
                "ההבדל: בערך {} פחות במדד, בתמורה לכסף זמין לחירום.",
                span(months, lang)
            ),
        })),
    }
}

/// The figures of the place type's decision.
#[derive(Debug, Clone)]
pub struct PlaceTypeFigures {
    pub no_date: bool,
    pub today: Option<Date>,
    pub months: u32,
    pub safe_low: Option<Decimal>,
    pub safe_high: Option<Decimal>,
    pub growth: Option<Decimal>,
    pub index_return: Percent,
    pub deposited: Decimal,
    pub worst: Option<Decimal>,
    pub history: Option<DepositsBand>,
    pub brokerage: bool,
    pub safe_kinds: Vec<SafeWay>,
}

fn account_fact() -> Text {
    fixed(
        "Needs a trading account. If you already have one, it's one more order.",
        "צריך חשבון מסחר. אם כבר יש לכם אחד, זו עוד פקודה.",
    )
}

fn if_index_rises(rate: Percent, lang: Lang) -> String {
    match lang {
        Lang::En => format!("if the index rises {rate} a year"),
        Lang::He => format!("אם המדד יעלה {rate} בשנה"),
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
    let kinds = both(|lang| {
        let names: Vec<&str> = f
            .safe_kinds
            .iter()
            .map(|way| match way {
                SafeWay::MoneyMarketFund => lang.pick("a money market fund", "קרן כספית"),
                SafeWay::Bond => lang.pick("an Israeli government bond", "אג״ח של מדינת ישראל"),
                SafeWay::Deposit => lang.pick("a deposit", "פיקדון"),
            })
            .collect();
        match lang {
            Lang::En => format!("Known almost in advance: {}.", names.join(", ")),
            Lang::He => format!("ידוע כמעט מראש: {}.", names.join(", ")),
        }
    });
    let mut safe_facts = vec![kinds];
    if f.safe_kinds.len() > 1 {
        safe_facts.push(fixed(
            "After this choice you'll choose which safe way.",
            "אחרי הבחירה הזאת תבחרו איזו דרך בטוחה.",
        ));
    }
    let mut growth_facts = Vec::new();
    if let Some(history) = &f.history {
        growth_facts.push(both(|lang| losses_sentence(history, false, lang)));
    }
    growth_facts.push(fixed(
        "Can be sold any day. If there's a fall, you wait a year or two.",
        "אפשר למכור בכל יום. אם יש ירידה, מחכים שנה\u{2011}שנתיים.",
    ));
    if f.brokerage {
        growth_facts.push(account_fact());
    }
    let difference = f.growth.zip(f.safe_high).map(|(growth, safe)| {
        both(|lang| {
            let rises = if_index_rises(f.index_return, lang);
            let gap = about(growth - safe);
            let below = f.worst.is_some_and(|worst| worst < f.deposited);
            let deposited = shekels(f.deposited);
            match (lang, below) {
                (Lang::En, true) => format!("{rises}, it would leave about {gap} more than the best safe way. In return: it can be below {deposited} in the very year you need the money."),
                (Lang::He, true) => format!("{rises}, הוא ישאיר בערך {gap} יותר מהדרך הבטוחה הטובה. בתמורה: הוא יכול להיות מתחת ל\u{2011}{deposited} דווקא בשנה שתצטרכו את הכסף."),
                (Lang::En, false) => format!("{rises}, it would leave about {gap} more than the best safe way. In return: it can be down in the very year you need the money."),
                (Lang::He, false) => format!("{rises}, הוא ישאיר בערך {gap} יותר מהדרך הבטוחה הטובה. בתמורה: הוא יכול להיות בירידה דווקא בשנה שתצטרכו את הכסף."),
            }
        })
    });
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
                title: fixed(
                    "Growth: a fund on the S&P 500 index",
                    "צמיחה: קרן על מדד S&P\u{a0}500",
                ),
                amount: both(|lang| match f.growth {
                    Some(growth) => format!(
                        "{} {}",
                        shekels(growth),
                        if_index_rises(f.index_return, lang)
                    ),
                    None => String::new(),
                }),
                facts: growth_facts,
                fits_if: None,
                button: fixed("Yes, it can wait", "כן, אפשר לחכות"),
            },
        ],
        difference,
    }
}

#[must_use]
pub fn safe_way_stated(way: SafeWay, kind: Kind, months: u32) -> Text {
    let name = super::outcome::name(kind);
    both(|lang| {
        let reason = match way {
            SafeWay::MoneyMarketFund => lang.pick(
                "it's the only safe way here, or leaves the most by far",
                "היא הדרך הבטוחה היחידה כאן, או שהיא משאירה הכי הרבה בפער",
            ),
            SafeWay::Bond | SafeWay::Deposit => lang.pick(
                "it leaves the most by far in every version we checked",
                "הוא משאיר הכי הרבה, בפער, בכל גרסה שבדקנו",
            ),
        };
        match lang {
            Lang::En => format!("{}: {reason}, over {}.", name.en, span(months, lang)),
            Lang::He => format!("{}: {reason}, לאורך {}.", name.he, span(months, lang)),
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

#[derive(Debug, Clone)]
pub struct SafeWayFigures<'a> {
    pub options: &'a [SafeOption],
    pub today: Option<Date>,
    pub months: u32,
    pub rate: Percent,
    pub switch: Option<Percent>,
    pub bond_at_your_bank: Option<Decimal>,
    /// The order flips between versions (rather than a tie as things are).
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
    let question = if has_fund && f.options.len() == 2 {
        fixed(
            "What matters more to you: the simplest, or knowing today how much there will be?",
            "מה חשוב לכם יותר: הכי פשוט, או לדעת כבר היום כמה יהיה?",
        )
    } else {
        fixed("Which safe way?", "איזו דרך בטוחה?")
    };
    let why = both(|lang| match (lang, has_fund && f.flips) {
        (Lang::En, true) => format!(
            "The difference in money is small (up to {}), and depends on a rate nobody knows in advance. So it's about what suits you, not about which leaves more.",
            shekels(high - low)
        ),
        (Lang::He, true) => format!(
            "ההבדל בכסף קטן (עד {}), ותלוי בריבית שאף אחד לא יודע מראש. לכן זה עניין של מה שנוח לכם, ולא של מי משאירה יותר.",
            shekels(high - low)
        ),
        (Lang::En, false) => format!(
            "They're at most {} apart over {}. What sets them apart is locking against flexibility, and simple against a new account.",
            shekels(high - low),
            span(f.months, lang)
        ),
        (Lang::He, false) => format!(
            "ההבדל ביניהן עד {} על פני {}. מה שמבדיל הוא נעילה מול גמישות, ופשטות מול חשבון חדש.",
            shekels(high - low),
            span(f.months, lang)
        ),
    });
    let options = f
        .options
        .iter()
        .map(|option| match option.way {
            SafeWay::MoneyMarketFund => OptWords {
                title: if f.options.len() == 2 {
                    fixed("The simplest: a money market fund", "הכי פשוט: קרן כספית")
                } else {
                    fixed("A money market fund", "קרן כספית")
                },
                amount: both(|lang| format!("{} {}", shekels(option.left), lang.pick("at today's rate", "בריבית של היום"))),
                facts: {
                    let mut facts = vec![
                        fixed("In your bank's app, with no new account.", "באפליקציה של הבנק שלכם, בלי חשבון חדש."),
                        fixed("Can be sold any trading day.", "אפשר למכור בכל יום מסחר."),
                    ];
                    if let Some((low, high)) = option.rate_range {
                        facts.push(both(|lang| {
                            let up = Percent(f.rate.0 + crate::guide::RATE_STEP);
                            let down = Percent((f.rate.0 - crate::guide::RATE_STEP).max(Decimal::ZERO));
                            match lang {
                                Lang::En => format!("The sum moves with the Bank of Israel's rate: {} if it averages {up}, {} if {down}.", shekels(high), shekels(low)),
                                Lang::He => format!("הסכום זז עם ריבית בנק ישראל: {} אם תהיה בממוצע {up}, ו\u{2011}{} אם {down}.", shekels(high), shekels(low)),
                            }
                        }));
                    }
                    facts
                },
                fits_if: Some(fixed("Fits if: you want as little fuss as possible.", "מתאים אם: אתם רוצים כמה שפחות התעסקות.")),
                button: fixed("A money market fund", "קרן כספית"),
            },
            SafeWay::Bond => {
                let name = super::outcome::name(option.kind);
                let known = match option.kind {
                    Kind::GovernmentBond { linked: true } => fixed("buying power known today", "כוח הקנייה ידוע כבר היום"),
                    _ => fixed("known today", "ידוע כבר היום"),
                };
                let mut facts = vec![
                    account_fact(),
                    both(|lang| match lang {
                        Lang::En => format!("Bought each month in the same series, held until it's repaid {}.", in_year(f.today, f.months, lang)),
                        Lang::He => format!("קונים את אותה סדרה, ומחזיקים עד הפדיון {}.", in_year(f.today, f.months, lang)),
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
                    button: fixed("Know in advance, even with an account", "לדעת מראש, גם אם צריך חשבון"),
                }
            }
            SafeWay::Deposit => OptWords {
                title: both(|lang| match lang {
                    Lang::En => format!("A fixed-rate deposit for {}", span(f.months, lang)),
                    Lang::He => format!("פיקדון בריבית קבועה ל\u{2011}{}", span(f.months, lang)),
                }),
                amount: both(|lang| format!("{}, {}", shekels(option.left), lang.pick("known today", "ידוע כבר היום"))),
                facts: vec![
                    fixed("At a bank.", "בבנק."),
                    both(|lang| match lang {
                        Lang::En => format!("The money is locked until the end, {}.", in_year(f.today, f.months, lang)),
                        Lang::He => format!("הכסף נעול עד הסוף, {}.", in_year(f.today, f.months, lang)),
                    }),
                ],
                fits_if: Some(both(|lang| match lang {
                    Lang::En => format!("Fits if: you won't touch the money before the end, {}.", in_year(f.today, f.months, lang)),
                    Lang::He => format!("מתאים אם: לא תצטרכו לגעת בכסף לפני הסוף, {}.", in_year(f.today, f.months, lang)),
                })),
                button: fixed("A deposit", "פיקדון"),
            },
        })
        .collect();
    let lead = f.options.first();
    let fund = f
        .options
        .iter()
        .find(|option| option.way == SafeWay::MoneyMarketFund);
    let difference = lead.map(|lead| {
        both(|lang| {
            let mut out = String::new();
            if let Some(second) = f.options.get(1) {
                let name = super::outcome::name(lead.kind);
                out = match lang {
                    Lang::En => format!("At today's rate, {} leaves {} more.", name.en, shekels(lead.left - second.left)),
                    Lang::He => format!("בריבית של היום, {} משאיר {} יותר.", name.he, shekels(lead.left - second.left)),
                };
            }
            if let Some(switch) = f.switch {
                let fund_leads = lead.way == SafeWay::MoneyMarketFund;
                out.push(' ');
                out.push_str(&match (lang, fund_leads) {
                    (Lang::En, false) => format!("If the rate averages above about {switch}, the money market fund leaves more."),
                    (Lang::He, false) => format!("אם הריבית תהיה בממוצע מעל כ\u{2011}{switch}, הקרן הכספית משאירה יותר."),
                    (Lang::En, true) => format!("If the rate averages below about {switch}, the fund leaves less than the others."),
                    (Lang::He, true) => format!("אם הריבית תהיה בממוצע מתחת לכ\u{2011}{switch}, הקרן הכספית משאירה פחות."),
                });
            }
            if let (Some(bank), Some(fund)) = (f.bank_at_your_bank_bond(), fund) {
                out.push(' ');
                let less = bank < fund.left;
                out.push_str(&match (lang, less) {
                    (Lang::En, true) => format!("Without a new account: a bond through your bank would leave {}, less than the money market fund.", shekels(bank)),
                    (Lang::He, true) => format!("ובלי חשבון חדש: אג״ח דרך הבנק שלכם הייתה משאירה {}, פחות מהקרן הכספית.", shekels(bank)),
                    (Lang::En, false) => format!("Without a new account: a bond through your bank would leave {}.", shekels(bank)),
                    (Lang::He, false) => format!("ובלי חשבון חדש: אג״ח דרך הבנק שלכם הייתה משאירה {}.", shekels(bank)),
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

impl SafeWayFigures<'_> {
    fn bank_at_your_bank_bond(&self) -> Option<Decimal> {
        self.bond_at_your_bank
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
fn way_short(way: Way, lang: Lang) -> &'static str {
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
            Some(left) => format!("{} {}", shekels(left), if_index_rises(f.index_return, lang)),
            None => String::new(),
        })
    };
    let opens = if f.lead.vehicle == Vehicle::Brokerage {
        fixed(
            "Anyone can open one, through a trading account.",
            "כל אחד יכול לפתוח, דרך חשבון מסחר.",
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
                title: fixed("Study fund, S&P 500 track", "קרן השתלמות, מסלול מדד S&P\u{a0}500"),
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
            lang.pick(", through a trading account", ", דרך חשבון מסחר")
        } else {
            ""
        };
        match (tied_with, lang) {
            (Some(other), Lang::En) => format!(
                "{}{how}. {} leaves about the same.",
                name.en,
                super::outcome::name(Kind::Way(other)).en
            ),
            (Some(other), Lang::He) => format!(
                "{}{how}. {} משאירה בערך אותו דבר.",
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
    pub tied: &'a [Text],
    pub best: Decimal,
    pub best_cost: Option<Percent>,
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
    let bank_title = both(|lang| match (f.yours, lang) {
        (Some(yours), Lang::En) => format!("Through your bank: {}", yours.en),
        (Some(yours), Lang::He) => format!("דרך הבנק שלכם: {}", yours.he),
        (None, _) => lang.pick("Through a bank", "דרך בנק").to_owned(),
    });
    let bank_amount = both(|lang| match (f.yours, f.yours_left, f.range) {
        (Some(_), Some(left), _) => shekels(left),
        (_, _, Some((low, high))) => format!(
            "{}{}–{}",
            lang.pick("through a bank: ", "דרך בנק: "),
            shekels(low),
            shekels(high)
        ),
        _ => String::new(),
    });
    let mut new_facts = vec![fixed(
        "You open a new account: a one-time chore.",
        "פותחים חשבון חדש: טרחה של פעם אחת.",
    )];
    new_facts.extend(cost_fact(f.best_cost));
    let mut bank_facts = vec![fixed("No new account.", "בלי חשבון חדש.")];
    bank_facts.extend(cost_fact(f.yours_cost));
    DecisionWords {
        question: fixed(
            "Open a new account, or buy through your bank?",
            "לפתוח חשבון חדש, או לקנות דרך הבנק שלכם?",
        ),
        why: fixed(
            "Opening an account is a one-time chore; the fees are paid every year. It's your call, but here the difference is large.",
            "פתיחת חשבון היא טרחה של פעם אחת. העמלות משולמות כל שנה. ההחלטה שלכם, אבל כאן ההבדל גדול.",
        ),
        options: vec![
            OptWords {
                title: both(|lang| match lang {
                    Lang::En => format!("A new account at {}", names(f.tied, lang)),
                    Lang::He => format!("חשבון חדש ב{}", names(f.tied, lang)),
                }),
                amount: both(|_| shekels(f.best)),
                facts: new_facts,
                fits_if: None,
                button: fixed("Open an account", "לפתוח חשבון"),
            },
            OptWords {
                title: bank_title,
                amount: bank_amount,
                facts: bank_facts,
                fits_if: None,
                button: fixed("Stay at the bank", "להישאר בבנק"),
            },
        ],
        difference: Some(both(|lang| {
            let years = span(f.years * 12, lang);
            match (lang, f.yours.is_some()) {
                (Lang::En, true) => format!(
                    "The difference: {} over {years}, mostly fees.",
                    shekels(f.gap)
                ),
                (Lang::He, true) => format!("ההבדל: {} לאורך {years}, רובו עמלות.", shekels(f.gap)),
                (Lang::En, false) => format!(
                    "The difference: at least {} over {years} against the cheapest bank, mostly fees.",
                    shekels(f.gap)
                ),
                (Lang::He, false) => format!(
                    "ההבדל: לפחות {} לאורך {years} לעומת הבנק הזול, רובו עמלות.",
                    shekels(f.gap)
                ),
            }
        })),
    }
}

#[must_use]
pub fn holdings_decision(
    saves: Option<Decimal>,
    margin: Decimal,
    tied: &[Text],
    months: u32,
) -> DecisionWords {
    DecisionWords {
        question: fixed("What's in the portfolio?", "מה יש בתיק?"),
        why: fixed(
            "Only you know what you bought. That decides whether moving saves anything.",
            "רק אתם יודעים מה קניתם. זה מה שקובע אם מעבר יחסוך משהו.",
        ),
        options: vec![
            OptWords {
                title: fixed("Index funds or shares", "קרנות מחקות או מניות"),
                amount: both(|lang| match saves {
                    Some(saves) if saves > margin => match lang {
                        Lang::En => format!(
                            "Moving to {} saves about {} over {}",
                            names(tied, lang),
                            about(saves),
                            span(months, lang)
                        ),
                        Lang::He => format!(
                            "מעבר ל{} חוסך בערך {} לאורך {}",
                            names(tied, lang),
                            about(saves),
                            span(months, lang)
                        ),
                    },
                    _ => lang
                        .pick("Moving saves little here", "כאן מעבר לא חוסך הרבה")
                        .to_owned(),
                }),
                facts: vec![fixed(
                    "Move the holdings as they are; don't sell them.",
                    "מעבירים את הניירות כמו שהם, לא מוכרים.",
                )],
                fits_if: None,
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
                button: fixed("Don't know", "לא יודעים"),
            },
        ],
        difference: None,
    }
}

// ─────────────────────────── Where ───────────────────────────

#[must_use]
pub fn where_fund() -> Text {
    fixed(
        "In your bank's app. A bank may not charge custody on a money market fund.",
        "באפליקציה של הבנק שלכם. בנק לא רשאי לגבות דמי משמרת על קרן כספית.",
    )
}

#[must_use]
pub fn where_deposit(best: &Text, left: Option<Decimal>, yours: Option<Decimal>) -> Text {
    both(|lang| {
        let mut out = match lang {
            Lang::En => format!(
                "{}. The rate in the figures is an average, not an offer: ask what you'll get.",
                best.en
            ),
            Lang::He => format!(
                "{}. הריבית בחישוב היא ממוצע, לא הצעה: שאלו מה תקבלו.",
                best.he
            ),
        };
        if let (Some(left), Some(yours)) = (left, yours)
            && left > yours
        {
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!("At your bank: {} less.", shekels(left - yours)),
                Lang::He => format!("בבנק שלכם: {} פחות.", shekels(left - yours)),
            });
        }
        out
    })
}

#[must_use]
pub fn where_fund_company(vehicle: Vehicle) -> Text {
    match vehicle {
        Vehicle::StudyFund => fixed(
            "In your study fund, on the track that follows the S&P 500 index.",
            "בקרן ההשתלמות שלכם, במסלול שעוקב אחרי מדד S&P\u{a0}500.",
        ),
        _ => fixed(
            "At a fund company, on the S&P 500 track. The figures use the companies' average fee: ask what you'll pay.",
            "בחברה מנהלת, במסלול מדד S&P\u{a0}500. החישוב לפי דמי הניהול הממוצעים: שאלו כמה תשלמו.",
        ),
    }
}

#[must_use]
pub fn where_stated(
    tied: &[Text],
    spread: Decimal,
    gap: Option<Decimal>,
    yours: Option<&Text>,
    range: Option<(Decimal, Decimal)>,
    moot: bool,
) -> Text {
    both(|lang| {
        let mut out = match (tied.len(), lang) {
            (1, _) => format!("{}.", names(tied, lang)),
            (_, Lang::En) => format!(
                "{}: at most {} apart, so any is fine.",
                names(tied, lang),
                shekels(spread)
            ),
            (_, Lang::He) => format!(
                "{}: ההפרש ביניהם עד {}, אז כל אחד בסדר.",
                names(tied, lang),
                shekels(spread)
            ),
        };
        let against = match (yours, range) {
            (Some(_), _) => lang.pick("your bank", "הבנק שלכם"),
            (None, Some(_)) => lang.pick("a bank", "בנק"),
            (None, None) => "",
        };
        if let Some(gap) = gap
            && !against.is_empty()
        {
            out.push(' ');
            out.push_str(&if moot || gap > Decimal::ZERO {
                match lang {
                    Lang::En => format!("Against {against}: about {} more.", about(gap)),
                    Lang::He => format!("לעומת {against}: בערך {} יותר.", about(gap)),
                }
            } else {
                match lang {
                    Lang::En => format!("Through {against} it's about the same."),
                    Lang::He => format!("דרך {against} זה בערך אותו דבר."),
                }
            });
        }
        out
    })
}

#[must_use]
pub fn where_title(yours: bool) -> Text {
    if yours {
        fixed(
            "Where to buy, against your bank",
            "איפה לקנות, לעומת הבנק שלכם",
        )
    } else {
        fixed("Where to buy", "איפה לקנות")
    }
}

#[must_use]
pub fn where_note(tied: usize, spread: Option<Decimal>) -> Option<Text> {
    let spread = spread?;
    (tied > 1).then(|| {
        both(|lang| match lang {
            Lang::En => format!(
                "These {tied} are less than {} apart, so any of them is fine.",
                shekels(spread.max(Decimal::ONE))
            ),
            Lang::He => format!(
                "ה\u{2011}{tied} האלה נבדלים זה מזה בפחות מ\u{2011}{}, אז כל אחד מהם בסדר.",
                shekels(spread.max(Decimal::ONE))
            ),
        })
    })
}

// ─────────────────────────── The plan ───────────────────────────

#[derive(Debug, Clone)]
pub struct PlanFigures<'a> {
    pub kind: Kind,
    pub now: Decimal,
    pub monthly: Decimal,
    pub today: Option<Date>,
    pub months: u32,
    /// The cushion to fill first: its target and months.
    pub cushion: Option<(Decimal, u32)>,
    pub tied: &'a [Text],
    pub account: Option<Account>,
    pub yours: Option<&'a Text>,
    pub gap: Option<Decimal>,
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
        let until = at_date(f.today, f.months, lang);
        let name = super::outcome::name(f.kind);
        let name = &name[lang];
        let pre = match (f.cushion, lang) {
            (Some((target, months)), Lang::En) => format!(
                "First {} a month to a money market fund in the bank's app, until {} is set aside (about {}). Then: ",
                shekels(f.monthly),
                shekels(target),
                span(months, lang)
            ),
            (Some((target, months)), Lang::He) => format!(
                "קודם {} בחודש לקרן כספית באפליקציה של הבנק, עד שיצטברו {} (בערך {}). אחר כך: ",
                shekels(f.monthly),
                shekels(target),
                span(months, lang)
            ),
            (None, _) => String::new(),
        };
        let at = names(f.tied, lang);
        let main = match (f.kind, f.holdings, f.account, lang) {
            (_, Some(Holdings::IndexFunds), _, Lang::En) => format!("Move the portfolio as it is to {at}, without selling, and keep {name} there."),
            (_, Some(Holdings::IndexFunds), _, Lang::He) => format!("מעבירים את התיק כמו שהוא ל{at}, בלי למכור, ומחזיקים שם {name}."),
            (_, Some(Holdings::Managed), _, Lang::En) => "Check each fund's management fee and the manager's fee before moving: moving alone doesn't lower them.".to_owned(),
            (_, Some(Holdings::Managed), _, Lang::He) => "בודקים את דמי הניהול של כל קרן ושל ניהול התיק לפני מעבר: מעבר לבד לא מוריד אותם.".to_owned(),
            (_, Some(Holdings::DontKnow), _, Lang::En) => "First see in the bank's app what the portfolio holds; then come back to this answer.".to_owned(),
            (_, Some(Holdings::DontKnow), _, Lang::He) => "קודם רואים באפליקציה של הבנק מה יש בתיק, ואז חוזרים לתשובה הזאת.".to_owned(),
            (Kind::MoneyMarketFund, _, _, Lang::En) => format!("{name} in your bank's app: {money}, until {until}."),
            (Kind::MoneyMarketFund, _, _, Lang::He) => format!("{name} באפליקציה של הבנק שלכם: {money}, עד {until}."),
            (Kind::FixedDeposit, _, _, Lang::En) => format!("{name} at {at}: {money}, {until}. Ask the bank what rate you'll get."),
            (Kind::FixedDeposit, _, _, Lang::He) => format!("{name} ב{at}: {money}, {until}. שאלו את הבנק מה הריבית שתקבלו."),
            (Kind::Way(way), _, _, Lang::En) if way.vehicle == Vehicle::StudyFund => format!("{money} to your study fund, on the track that follows the S&P 500 index."),
            (Kind::Way(way), _, _, Lang::He) if way.vehicle == Vehicle::StudyFund => format!("{money} לקרן ההשתלמות שלכם, במסלול שעוקב אחרי מדד S&P\u{a0}500."),
            (Kind::Way(way), _, _, Lang::En) if way.vehicle != Vehicle::Brokerage => format!("{money} to {name}, at a fund company."),
            (Kind::Way(way), _, _, Lang::He) if way.vehicle != Vehicle::Brokerage => format!("{money} ל{name}, בחברה מנהלת."),
            (_, _, Some(Account::YourBank), Lang::En) => {
                let bank = f.yours.map_or_else(|| "your bank".to_owned(), |yours| yours.en.to_string());
                let less = f.gap.map_or_default(|gap| format!(" It leaves about {} less than a new account, because of fees.", about(gap)));
                format!("{money} to {name}, through {bank}.{less}")
            }
            (_, _, Some(Account::YourBank), Lang::He) => {
                let bank = f.yours.map_or_else(|| "הבנק שלכם".to_owned(), |yours| yours.he.to_string());
                let less = f.gap.map_or_default(|gap| format!(" זה משאיר בערך {} פחות מחשבון חדש, בגלל עמלות.", about(gap)));
                format!("{money} ל{name}, דרך {bank}.{less}")
            }
            (_, _, _, Lang::En) => format!("{name}, through a trading account at {at}: {money}."),
            (_, _, _, Lang::He) => format!("{name}, דרך חשבון מסחר ב{at}: {money}."),
        };
        pre + &main
    })
}
