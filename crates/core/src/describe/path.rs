//! The plan and the path to it (REDESIGN.md §4.5): the plan's amount, each
//! step's choice and the line on what it hinged on, the step "איזה מדד",
//! and "למה זה ברור?".
//!
//! A step's line says what the decision hinged on, not only money: the
//! chosen option's "מתאים אם" or main fact, then what the other would have
//! given besides money. The amounts stay in the decision, under "שינוי".

#![allow(
    clippy::format_push_string,
    reason = "sentences built a clause at a time"
)]

use rust_decimal::Decimal;
use time::Date;

use super::card::{about, both, end_date, joined_to};
use super::reasons::{grouped, shekels, span};
use crate::guide::Kind;
use crate::{Lang, Percent, Text};

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A step's choice and its line.
#[derive(Debug, Clone)]
pub struct StepWords {
    pub choice: Text,
    pub hinge: Text,
}

fn step(choice: Text, hinge: Text) -> StepWords {
    StepWords { choice, hinge }
}

/// "ב‑2056", or "בעוד 30 שנה" with no date of the visitor's.
fn in_year(today: Option<Date>, months: u32, dated: bool, lang: Lang) -> String {
    match today
        .filter(|_| dated)
        .and_then(|today| end_date(today, months))
    {
        Some(end) => format!("{}{}", lang.pick("in ", "ב\u{2011}"), end.year()),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
    }
}

/// "2056", or "בעוד 30 שנה".
fn year(today: Option<Date>, months: u32, dated: bool, lang: Lang) -> String {
    match today
        .filter(|_| dated)
        .and_then(|today| end_date(today, months))
    {
        Some(end) => end.year().to_string(),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(months, lang)),
    }
}

/// A name after "ל": "לקרן סל על מדד מניות רחב".
fn to(name: &Text, lang: Lang) -> String {
    match lang {
        Lang::En => format!("to {}", lowered(&name.en)),
        Lang::He => joined_to('ל', &name.he),
    }
}

fn lowered(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

// ─────────────────────────── The labels ───────────────────────────

/// The label of the step outside the site: "איזה מדד".
#[must_use]
pub fn which_index() -> (Text, StepWords) {
    (
        fixed("Which index", "איזה מדד"),
        step(
            fixed("A broad stock index", "מדד מניות רחב"),
            fixed(
                "The figures here are by the S&P 500 index, as an example, since it has the longest history. Which index to buy, and whether to spread between several, is yours to choose. The site doesn't compare indexes.",
                "המספרים כאן לפי מדד S&P\u{a0}500, כדוגמה, כי יש לו ההיסטוריה הארוכה ביותר. איזה מדד לקנות, ואם לפזר בין כמה, אתם בוחרים. האתר לא משווה בין מדדים.",
            ),
        ),
    )
}

// ─────────────────────────── כסף לחירום ───────────────────────────

/// "יש לכם כבר כרית ביטחון": all the monthly money goes to the plan's
/// product.
#[must_use]
pub fn cushion_has(monthly: Decimal, product: &Text) -> StepWords {
    step(
        fixed("You already have a cushion", "יש לכם כבר כרית ביטחון"),
        both(|lang| match lang {
            Lang::En => format!(
                "So all {} goes {}. The cushion keeps a sudden expense from forcing you to sell just when the market is down.",
                shekels(monthly),
                to(product, lang)
            ),
            Lang::He => format!(
                "לכן כל ה\u{2011}{} הולכים {}. הכרית שומרת שהוצאה פתאומית לא תכריח אתכם למכור דווקא בירידה.",
                shekels(monthly),
                to(product, lang)
            ),
        }),
    )
}

/// "קודם כרית ביטחון": the cushion's sum first, then the plan's product; or,
/// where it takes all the time there is, every shekel to it.
#[must_use]
pub fn cushion_first(
    target: Decimal,
    monthly: Decimal,
    product: &Text,
    takes_all: bool,
) -> StepWords {
    step(
        fixed("A safety cushion first", "קודם כרית ביטחון"),
        both(|lang| match (lang, takes_all) {
            (Lang::En, false) => format!(
                "First {} for the cushion, which can be taken out the next day. Then the {} goes {}.",
                shekels(target),
                shekels(monthly),
                to(product, lang)
            ),
            (Lang::He, false) => format!(
                "קודם {} לכרית, שאפשר למשוך ממנה כבר למחרת. אחר כך ה\u{2011}{} עוברים {}.",
                shekels(target),
                shekels(monthly),
                to(product, lang)
            ),
            (Lang::En, true) => format!(
                "All {} goes to the cushion, in a money market fund, which can be taken out the next day: it takes all the time there is.",
                shekels(monthly)
            ),
            (Lang::He, true) => format!(
                "כל ה\u{2011}{} הולכים לכרית, בקרן כספית שאפשר למשוך ממנה כבר למחרת: הכרית לוקחת את כל הזמן שיש.",
                shekels(monthly)
            ),
        }),
    )
}

// ─────────────────────────── מניות או בטוח ───────────────────────────

/// Stocks, stated by the site: "הכסף מחכה 30 שנה."
#[must_use]
pub fn stocks_stated(months: u32) -> StepWords {
    step(
        fixed("Stocks", "מניות"),
        both(|lang| match lang {
            Lang::En => format!("The money waits {}.", span(months, lang)),
            Lang::He => format!("הכסף מחכה {}.", span(months, lang)),
        }),
    )
}

/// Safe, stated by the site: the line's own sentence, after its "בטוח."
#[must_use]
pub fn safe_stated(line: &Text) -> StepWords {
    let rest = |text: &str, lead: &[&str]| {
        lead.iter()
            .find_map(|lead| text.strip_prefix(lead))
            .unwrap_or(text)
            .trim_start()
            .to_owned()
    };
    step(
        fixed("Safe", "בטוח"),
        Text::owned(
            rest(&line.en, &["Safe.", "Safe:"]),
            rest(&line.he, &["בטוח.", "בטוח:"]),
        ),
    )
}

/// Safe, the visitor's: "לא, אצטרך את הכסף בדיוק אז".
#[must_use]
pub fn safe_chosen(today: Option<Date>, months: u32, dated: bool, no_date: bool) -> StepWords {
    step(
        fixed("Safe", "בטוח"),
        both(|lang| match (lang, no_date) {
            (Lang::En, false) => format!(
                "You'll need the money exactly {}, so it can't wait for the market to recover.",
                in_year(today, months, dated, lang)
            ),
            (Lang::He, false) => format!(
                "תצטרכו את הכסף בדיוק {}, אז אי אפשר לחכות שהבורסה תתאושש.",
                in_year(today, months, dated, lang)
            ),
            (Lang::En, true) => {
                "You may need the money sooner, so it can't wait for the market to recover."
                    .to_owned()
            }
            (Lang::He, true) => {
                "אולי תצטרכו את הכסף לפני כן, אז אי אפשר לחכות שהבורסה תתאושש.".to_owned()
            }
        }),
    )
}

/// Stocks, the visitor's: "כן, אפשר לחכות".
#[must_use]
pub fn stocks_chosen(today: Option<Date>, months: u32, dated: bool, no_date: bool) -> StepWords {
    step(
        fixed("Stocks", "מניות"),
        both(|lang| match (lang, no_date) {
            (Lang::En, false) => format!(
                "If the market is down {}, you can wait another year or two.",
                in_year(today, months, dated, lang)
            ),
            (Lang::He, false) => format!(
                "אם {} הבורסה תהיה בירידה, תוכלו לחכות עוד שנה\u{2011}שנתיים.",
                in_year(today, months, dated, lang)
            ),
            (Lang::En, true) => format!(
                "The money can wait at least {}, even if the market is down.",
                span(months, lang)
            ),
            (Lang::He, true) => format!(
                "הכסף יכול לחכות לפחות {}, גם אם הבורסה בירידה.",
                span(months, lang)
            ),
        }),
    )
}

/// A portfolio at a bank: the place isn't asked.
#[must_use]
pub fn existing_stated(line: &Text) -> StepWords {
    step(
        fixed("What's in the portfolio", "מה שכבר בתיק"),
        line.clone(),
    )
}

// ─────────────────────────── המוצר ───────────────────────────

/// The study fund's decision, "a way anyone can open" chosen.
#[must_use]
pub fn wrapper_anyone(short: Text) -> StepWords {
    step(
        short,
        fixed(
            "Fits when you have no study fund you can add to. In a study fund the gain is tax-free after 6 years, but it's open only to the self-employed or through an employer.",
            "מתאים כשאין לכם קרן השתלמות שאפשר להוסיף לה. בקרן השתלמות הרווח פטור ממס אחרי 6 שנים, אבל היא פתוחה רק לעצמאים או דרך מעסיק.",
        ),
    )
}

/// The study fund chosen; `other` is the way anyone can open.
#[must_use]
pub fn wrapper_study(other: &Text) -> StepWords {
    step(
        fixed("Study fund, stock track", "קרן השתלמות, במסלול מניות"),
        both(|lang| match lang {
            Lang::En => format!(
                "In your study fund the gain is tax-free after 6 years. {} was open to anyone, with no employer.",
                capitalised(&other.en)
            ),
            Lang::He => format!(
                "בקרן ההשתלמות שלכם הרווח פטור ממס אחרי 6 שנים. {} הייתה פתוחה לכל אחד, בלי מעסיק.",
                other.he
            ),
        }),
    )
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Whether a Hebrew name takes a masculine verb: a deposit (פיקדון), makam
/// (מק״מ); the rest here are feminine (קרן, אג״ח, קופה, פוליסה).
fn masculine(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit | Kind::TreasuryBill
    )
}

/// The two-option safe decision, the money market fund chosen; `other` is
/// the way whose sum is known in advance.
#[must_use]
pub fn simplest_fund(other: Kind, other_name: &Text) -> StepWords {
    step(
        fixed("The simplest: a money market fund", "הכי פשוט: קרן כספית"),
        both(|lang| match lang {
            Lang::En => format!(
                "A money market fund: the simplest, and sold any trading day. {} would have told you today how much there will be.",
                capitalised(&other_name.en)
            ),
            Lang::He => format!(
                "קרן כספית: הכי פשוט, ונמכרת בכל יום מסחר. {} {} לדעת כבר היום כמה יהיה.",
                other_name.he,
                if masculine(other) {
                    "היה נותן"
                } else {
                    "הייתה נותנת"
                }
            ),
        }),
    )
}

/// The two-option safe decision, the known sum chosen.
#[must_use]
pub fn known_in_advance(name: &Text) -> StepWords {
    step(
        both(|lang| match lang {
            Lang::En => format!("Known in advance: {}", name.en),
            Lang::He => format!("לדעת מראש: {}", name.he),
        }),
        fixed(
            "You know today how much there will be. A money market fund would have been simpler, and sold any trading day.",
            "יודעים כבר היום כמה יהיה. קרן כספית הייתה פשוטה יותר, ונמכרת בכל יום מסחר.",
        ),
    )
}

/// "איזו דרך בטוחה?": a fixed-rate deposit chosen.
#[must_use]
pub fn safe_deposit(today: Option<Date>, months: u32, dated: bool) -> StepWords {
    step(
        fixed("A deposit", "פיקדון"),
        both(|lang| match lang {
            Lang::En => format!(
                "A rate known in advance, but the money is locked until {}.",
                year(today, months, dated, lang)
            ),
            Lang::He => format!(
                "ריבית ידועה מראש, אבל הכסף נעול עד {}.",
                year(today, months, dated, lang)
            ),
        }),
    )
}

/// "איזו דרך בטוחה?": a bond chosen.
#[must_use]
pub fn safe_bond() -> StepWords {
    step(
        fixed("A bond", "אג״ח"),
        fixed(
            "You know today how much there will be. It takes a securities account, and can be sold before it's repaid too.",
            "יודעים כבר היום כמה יהיה. צריך חשבון ניירות ערך, ואפשר למכור גם לפני הפדיון.",
        ),
    )
}

/// "איזו דרך בטוחה?": the money market fund chosen.
#[must_use]
pub fn safe_fund() -> StepWords {
    step(
        fixed("A money market fund", "קרן כספית"),
        fixed(
            "The simplest, and sold any trading day. The sum at the end moves with the Bank of Israel's rate.",
            "הכי פשוט, ונמכרת בכל יום מסחר. הסכום בסוף זז עם ריבית בנק ישראל.",
        ),
    )
}

/// "איזו דרך בטוחה?" for money needed any day: the deposit at the bank.
#[must_use]
pub fn safe_any_day_deposit() -> StepWords {
    step(
        fixed(
            "A deposit you can take out any day",
            "פיקדון שאפשר למשוך בכל יום",
        ),
        fixed(
            "It stays at your bank, and can be taken out any day. Its rate changes, and is usually lower.",
            "נשאר בבנק שבו יש לכם חשבון, ואפשר למשוך ממנו בכל יום. הריבית משתנה, ובדרך כלל נמוכה יותר.",
        ),
    )
}

/// "איזו דרך בטוחה?" for money needed any day: makam.
#[must_use]
pub fn safe_bill() -> StepWords {
    step(
        fixed("Makam", "מק״מ"),
        fixed(
            "It takes a securities account. It can be sold any trading day, at that day's price, which moves a little.",
            "צריך חשבון ניירות ערך. אפשר למכור בכל יום מסחר, במחיר של אותו יום, שזז מעט.",
        ),
    )
}

/// The way, stated by the site: its name, and why.
#[must_use]
pub fn way_stated(name: &Text, why: Text) -> StepWords {
    step(name.clone(), why)
}

/// Why a safe way is stated (§4.4): the only one here, or the one that
/// leaves more in every version.
#[must_use]
pub fn safe_way_why(only: bool) -> Text {
    if only {
        fixed("It's the only safe way here.", "זו הדרך הבטוחה היחידה כאן.")
    } else {
        fixed(
            "Among the safe ways, it leaves more than the others in every version we checked (the Bank of Israel's rate, inflation).",
            "בין הדרכים הבטוחות, זו משאירה יותר מהאחרות בכל גרסה שבדקנו (ריבית בנק ישראל, אינפלציה).",
        )
    }
}

/// Why the index's way is stated: it leaves the most of the ways anyone
/// can open, or ties with another.
#[must_use]
pub fn index_way_why(tied_with: Option<&Text>) -> Text {
    match tied_with {
        Some(other) => both(|lang| match lang {
            Lang::En => format!(
                "Of the ways anyone can open, it leaves the most after fees and tax; {} leaves about the same.",
                lowered(&other.en)
            ),
            Lang::He => format!(
                "מבין הדרכים שכל אחד יכול לפתוח, היא משאירה הכי הרבה אחרי עמלות ומס; בערך אותו סכום משאירה גם {}.",
                other.he
            ),
        }),
        None => fixed(
            "Of the ways anyone can open, it leaves the most after fees and tax.",
            "מבין הדרכים שכל אחד יכול לפתוח, היא משאירה הכי הרבה אחרי עמלות ומס.",
        ),
    }
}

// ─────────────────────────── מה יש בתיק? ───────────────────────────

#[must_use]
pub fn holdings_index_funds() -> StepWords {
    step(
        fixed("Index funds or shares", "קרנות מחקות או מניות"),
        fixed(
            "They move as they are: not sold, so no tax. With managed funds, moving alone wouldn't have lowered their fees.",
            "עוברים כמו שהם: לא מוכרים, ולכן אין מס. בקרנות מנוהלות, מעבר לבד לא היה מוריד את דמי הניהול.",
        ),
    )
}

#[must_use]
pub fn holdings_managed() -> StepWords {
    step(
        fixed(
            "Managed funds, or portfolio management",
            "קרנות מנוהלות, או ניהול תיקים",
        ),
        fixed(
            "A fund's management fee is charged wherever it's held. With index funds or shares, moving would have saved on fees.",
            "דמי הניהול של קרן נגבים בכל מקום שמחזיקים אותה. בקרנות מחקות או במניות, מעבר היה חוסך בעמלות.",
        ),
    )
}

#[must_use]
pub fn holdings_dont_know() -> StepWords {
    step(
        fixed("Don't know", "לא יודעים"),
        fixed(
            "The bank's app lists each holding with its kind. Once you know, the answer shows whether moving saves.",
            "באפליקציה של הבנק רשום כל נייר וסוגו. אחרי שתדעו, התשובה תראה אם מעבר חוסך.",
        ),
    )
}

// ─────────────────────────── איפה קונים ───────────────────────────

/// A new account chosen: "חשבון חדש באינטראקטיב".
#[must_use]
pub fn new_account(names: &Text) -> StepWords {
    step(
        both(|lang| match lang {
            Lang::En => format!("A new account at {}", names.en),
            Lang::He => format!("חשבון חדש {}", names.he),
        }),
        fixed(
            "You open an account once. At the bank there's nothing to open, but the fees there are higher, and paid every year.",
            "פותחים חשבון פעם אחת. בבנק לא צריך לפתוח כלום, אבל העמלות שם גבוהות יותר, ומשולמות כל שנה.",
        ),
    )
}

/// The visitor's bank chosen.
#[must_use]
pub fn your_bank(named: bool) -> StepWords {
    step(
        if named {
            fixed("Through your bank", "דרך הבנק שלכם")
        } else {
            fixed("Through a bank", "דרך בנק")
        },
        fixed(
            "Nothing to open. In a new account the fees would have been lower, but it has to be opened once.",
            "לא צריך לפתוח כלום. בחשבון חדש העמלות היו נמוכות יותר, אבל צריך לפתוח אותו פעם אחת.",
        ),
    )
}

/// Where, stated by the site: the names, and the line's own reason.
#[must_use]
pub fn where_stated(choice: Text, line: &Text) -> StepWords {
    step(choice, line.clone())
}

// ─────────────────────────── למה זה ברור? ───────────────────────────

/// The figures "למה זה ברור?" words.
#[derive(Debug, Clone)]
pub struct WhyFigures {
    /// The stretch the history counts, months.
    pub months: u32,
    pub windows: u32,
    pub below: u32,
    pub since: i32,
    pub now: Decimal,
    pub monthly: Decimal,
    /// The stocks are the answer (or, for a safe answer, not).
    pub stocks: bool,
    /// The worst stretch left more than the best safe way.
    pub worst_beats_safe: bool,
    /// The best safe way's name.
    pub safe: Text,
    pub index_return: Percent,
}

/// What "למה זה ברור?" says: the lead, the count, the crises, the past.
#[derive(Debug, Clone)]
pub struct WhyWords {
    pub lead: Text,
    pub count: Text,
    pub crises: Text,
    pub past: Text,
}

/// "with a deposit every month, as yours": how the stretches were run.
fn as_yours(now: Decimal, monthly: Decimal, lang: Lang) -> &'static str {
    match (now.is_zero(), monthly.is_zero()) {
        (true, _) => lang.pick(
            "depositing every month, as you do",
            "עם הפקדה כל חודש כמו שלכם",
        ),
        (false, true) => lang.pick(
            "with one sum at the start, as yours",
            "עם סכום אחד בהתחלה, כמו שלכם",
        ),
        (false, false) => lang.pick(
            "with a sum at the start and a deposit every month, as yours",
            "עם סכום בהתחלה והפקדה כל חודש, כמו שלכם",
        ),
    }
}

#[must_use]
pub fn why_clear(f: &WhyFigures) -> WhyWords {
    let windows = grouped(f.windows);
    let below = grouped(f.below);
    let since = f.since;
    let lead = both(|lang| {
        let years = span(f.months, lang);
        match (lang, f.below) {
            (Lang::En, 0) => {
                format!("A loss after {years} is very unlikely: in the past it never happened.")
            }
            (Lang::He, 0) => format!("הפסד אחרי {years} הוא מאוד לא סביר: בעבר זה לא קרה אף פעם."),
            (Lang::En, _) => format!(
                "In stocks, a loss after {years} happened in the past: in {below} of {windows} stretches."
            ),
            (Lang::He, _) => format!(
                "במניות, הפסד אחרי {years} קרה בעבר: ב\u{2011}{below} מתוך {windows} תקופות."
            ),
        }
    });
    let count = both(|lang| {
        let years = span(f.months, lang);
        let yours = as_yours(f.now, f.monthly, lang);
        let mut out = match (lang, f.below) {
            (Lang::En, 0) => format!(
                "We checked every stretch of {years} since {since}, {windows} of them, {yours}. In none of them was less left at the end than was put in."
            ),
            (Lang::He, 0) => format!(
                "בדקנו כל תקופה של {years} מאז {since}, {windows} תקופות, {yours}. באף אחת מהן לא נשאר בסוף פחות ממה שהופקד."
            ),
            (Lang::En, _) => format!(
                "We checked every stretch of {years} since {since}, {windows} of them, {yours}. In {below} of them, less was left at the end than was put in."
            ),
            (Lang::He, _) => format!(
                "בדקנו כל תקופה של {years} מאז {since}, {windows} תקופות, {yours}. ב\u{2011}{below} מהן נשאר בסוף פחות ממה שהופקד."
            ),
        };
        if f.stocks && f.worst_beats_safe {
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!(
                    "Even the worst of them left more than {}.",
                    lowered(&f.safe.en)
                ),
                Lang::He => format!(
                    "גם בתקופה הגרועה מכולן נשאר יותר מאשר {}.",
                    joined_to('ב', &f.safe.he)
                ),
            });
        }
        out
    });
    let crises = both(|lang| {
        let mut out = lang
            .pick(
                "These stretches went through the crash of 1929, the Second World War, 2000 and 2008.",
                "התקופות האלה עברו דרך המשבר של 1929, מלחמת העולם השנייה, 2000 ו\u{2011}2008.",
            )
            .to_owned();
        if f.below == 0 {
            out.push(' ');
            out.push_str(lang.pick(
                "Whoever kept depositing and didn't sell along the way ended each of them with more than they put in.",
                "מי שהמשיך להפקיד ולא מכר באמצע, סיים כל אחת מהן עם יותר ממה שהפקיד.",
            ));
        }
        out
    });
    let past = both(|lang| {
        let mut out = lang
            .pick(
                "This is what happened in the past, not a promise.",
                "זה מה שקרה בעבר, לא הבטחה.",
            )
            .to_owned();
        if f.stocks && f.below == 0 {
            out.push(' ');
            out.push_str(&match lang {
                Lang::En => format!(
                    "To lose over {}, the coming years would have to be worse than anything since {since}.",
                    span(f.months, lang)
                ),
                Lang::He => format!(
                    "כדי להפסיד ב\u{2011}{}, השנים הבאות יצטרכו להיות גרועות מכל מה שקרה מאז {since}.",
                    span(f.months, lang)
                ),
            });
        }
        out.push(' ');
        out.push_str(lang.pick(
            "The data is the S&P 500 index's, after fees and tax, in today's money.",
            "הנתונים של מדד S&P\u{a0}500, אחרי עמלות ומס, בכסף של היום.",
        ));
        out
    });
    WhyWords {
        lead,
        count,
        crises,
        past,
    }
}

/// The four amounts' labels.
#[must_use]
pub fn deposited_label() -> Text {
    fixed("What you'll put in", "מה שתפקידו")
}

#[must_use]
pub fn safe_label(name: &Text) -> Text {
    both(|lang| match lang {
        Lang::En => format!("{}, the safe way", name.en),
        Lang::He => format!("{}, הדרך הבטוחה", name.he),
    })
}

#[must_use]
pub fn worst_label(since: i32) -> Text {
    both(|lang| match lang {
        Lang::En => format!("Stocks, in the worst stretch since {since}"),
        Lang::He => format!("מניות, בתקופה הגרועה ביותר מאז {since}"),
    })
}

#[must_use]
pub fn assumed_label(rate: Percent) -> Text {
    both(|lang| match lang {
        Lang::En => format!("Stocks, if the broad stock index rises {rate} a year"),
        Lang::He => format!("מניות, אם מדד המניות הרחב יעלה {rate} בשנה"),
    })
}

// ─────────────────────────── The plan's amount ───────────────────────────

/// How sure the plan's amount is, after its date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sureness {
    /// Known today: a deposit, a bond held to the end.
    Known,
    /// At today's rate: a money market fund, a deposit you can take out any
    /// day.
    Rate,
    /// If the index rises as assumed; `true` where the plan's sentence
    /// named the index ("קרן סל על מדד מניות רחב"), so "המדד" refers back.
    Index(Percent, bool),
}

/// The amount's words: "בערך ₪427,000"; "ב‑2056, אם המדד יעלה 10% בשנה";
/// "בכסף של היום, אחרי מס. תפקידו בערך ₪136,000."; for stocks, "החישוב לפי
/// מדד S&P 500, כדוגמה למדד רחב."
#[must_use]
pub fn plan_amount(
    value: Decimal,
    sureness: Sureness,
    today: Option<Date>,
    months: u32,
    dated: bool,
    deposited: Decimal,
) -> (Text, Text, Text, Option<Text>) {
    let amount = match sureness {
        Sureness::Known => both(|_| shekels(value)),
        Sureness::Rate | Sureness::Index(..) => {
            both(|lang| format!("{}{}", lang.pick("about ", "בערך "), about(value)))
        }
    };
    let when = both(|lang| {
        let at = match today
            .filter(|_| dated)
            .and_then(|today| end_date(today, months))
        {
            Some(end) => format!("{}{}", lang.pick("In ", "ב\u{2011}"), end.year()),
            None if dated => format!("{} {}", lang.pick("In", "בעוד"), span(months, lang)),
            None => format!(
                "{} {}",
                lang.pick("For example, in", "לדוגמה, בעוד"),
                span(months, lang)
            ),
        };
        let sure = match (sureness, lang) {
            (Sureness::Known, _) => lang.pick("known today", "ידוע כבר היום").to_owned(),
            (Sureness::Rate, _) => lang.pick("at today's rate", "בריבית של היום").to_owned(),
            (Sureness::Index(rate, true), Lang::En) => format!("if the index rises {rate} a year"),
            (Sureness::Index(rate, true), Lang::He) => format!("אם המדד יעלה {rate} בשנה"),
            (Sureness::Index(rate, false), Lang::En) => {
                format!("if the broad stock index rises {rate} a year")
            }
            (Sureness::Index(rate, false), Lang::He) => {
                format!("אם מדד המניות הרחב יעלה {rate} בשנה")
            }
        };
        format!("{at}, {sure}")
    });
    let basis = both(|lang| match lang {
        Lang::En => format!(
            "In today's money, after tax. You'll put in about {}.",
            about(deposited)
        ),
        Lang::He => format!("בכסף של היום, אחרי מס. תפקידו בערך {}.", about(deposited)),
    });
    let example = matches!(sureness, Sureness::Index(..)).then(|| {
        fixed(
            "Worked out by the S&P 500 index, as an example of a broad index.",
            "החישוב לפי מדד S&P\u{a0}500, כדוגמה למדד רחב.",
        )
    });
    (amount, when, basis, example)
}
