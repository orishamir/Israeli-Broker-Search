//! The words of the guide's answer: why this (the reasons, each from one of
//! the saver's circumstances, with its numbers filled in), why not the
//! others, the history sentence, the safety tags, the unlock line and the
//! cushion's two parts. How to do it step by step is in `steps.rs`.
//!
//! Register: "what fits money like this", never "our recommendation". A
//! reason says only what the guide really computed: where a rule decides
//! (the pool of kinds a row compares) the words say the rule, and where the
//! kind was ranked, they say it was ranked.

use rust_decimal::Decimal;
use time::Date;

use crate::describe::format_money;
use crate::guide::{
    Amount, Arena, Bank, Circumstance, Context, CushionPlan, Horizon,
    INDEX_IF_FLEXIBLE_FROM_MONTHS, Kind, SAFE_ONLY_UP_TO_MONTHS, Safety, Situation, UnlockLine,
};
use crate::history::Band;
use crate::{Lang, Named, Vehicle, ils};

pub use super::answer::Span;
pub use super::steps::steps_for;

/// ₪ to the shekel: "₪51,000".
pub(super) fn shekels(amount: Decimal) -> String {
    format_money(ils(amount.round_dp(0)))
}

/// A whole number with thousands separators: "1,086".
fn grouped(number: u32) -> String {
    let digits = number.to_string();
    let mut out = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

fn pick(lang: Lang, en: &str, he: &str) -> String {
    lang.pick(en, he).to_owned()
}

/// One text from its two languages, built eagerly (both are cheap).
fn say(lang: Lang, en: String, he: String) -> String {
    match lang {
        Lang::En => en,
        Lang::He => he,
    }
}

/// A number of months as the saver thinks of it: "5 years", "18 months".
pub(super) fn span(months: u32, lang: Lang) -> String {
    if months >= 12 && months.is_multiple_of(12) {
        let years = months / 12;
        match lang {
            Lang::En => format!("{years} {}", if years == 1 { "year" } else { "years" }),
            Lang::He => match years {
                1 => "שנה".to_owned(),
                2 => "שנתיים".to_owned(),
                _ => format!("{years}\u{a0}שנים"),
            },
        }
    } else {
        match lang {
            Lang::En => format!("{months} {}", if months == 1 { "month" } else { "months" }),
            Lang::He => match months {
                1 => "חודש".to_owned(),
                2 => "חודשיים".to_owned(),
                _ => format!("{months}\u{a0}חודשים"),
            },
        }
    }
}

/// The stretch of the history, in years, which may be fractional.
fn years_phrase(years: Decimal, lang: Lang) -> String {
    if years.fract().is_zero() {
        let whole = u32::try_from(years.trunc().mantissa()).unwrap_or(0);
        span(whole * 12, lang)
    } else {
        match lang {
            Lang::En => format!("{} years", years.round_dp(1).normalize()),
            Lang::He => format!("{}\u{a0}שנים", years.round_dp(1).normalize()),
        }
    }
}

fn month_year(date: Date, lang: Lang) -> String {
    let months = [
        ("January", "ינואר"),
        ("February", "פברואר"),
        ("March", "מרץ"),
        ("April", "אפריל"),
        ("May", "מאי"),
        ("June", "יוני"),
        ("July", "יולי"),
        ("August", "אוגוסט"),
        ("September", "ספטמבר"),
        ("October", "אוקטובר"),
        ("November", "נובמבר"),
        ("December", "דצמבר"),
    ];
    let (en, he) = months[usize::from(date.month() as u8) - 1];
    format!("{} {}", lang.pick(en, he), date.year())
}

/// The bank the saver already has a portfolio in, in words.
fn bank_name(situation: Option<Situation>, lang: Lang) -> String {
    match situation {
        Some(Situation::ExistingPortfolio {
            bank: Some(Bank::Leumi),
        }) => lang.pick("Bank Leumi", "בנק לאומי").to_owned(),
        Some(Situation::ExistingPortfolio {
            bank: Some(Bank::MizrahiTefahot),
        }) => lang.pick("Mizrahi-Tefahot", "מזרחי\u{a0}טפחות").to_owned(),
        Some(Situation::ExistingPortfolio {
            bank: Some(Bank::OtsarHahayal),
        }) => lang.pick("Otsar Hahayal", "אוצר החייל").to_owned(),
        _ => lang.pick("your bank", "הבנק שלכם").to_owned(),
    }
}

// ─────────────────────────── The history ───────────────────────────

/// What the S&P 500 did over stretches as long as the saver's horizon, in
/// one sentence: "in 52 of 1,086 stretches of 5 years since 1926 it ended
/// lower". The numbers are the band's.
#[must_use]
pub fn history_sentence(band: &Band, lang: Lang) -> String {
    let years = years_phrase(band.years, lang);
    let since = band.from.year();
    let windows = grouped(band.windows);
    let lower = grouped(band.lower);
    match (band.lower, lang) {
        (0, Lang::En) => format!(
            "In none of {windows} stretches of {years} since {since} did the S&P 500 end lower \
             (in dollars, before fees and tax)."
        ),
        (0, Lang::He) => format!(
            "באף אחת מ\u{2011}{windows} תקופות של {years} מאז {since} ה\u{2011}S&P\u{a0}500 לא נגמר בהפסד (בדולרים, לפני עמלות ומס)."
        ),
        (1, Lang::En) => format!(
            "In 1 of {windows} stretches of {years} since {since}, the S&P 500 ended lower (in \
             dollars, before fees and tax)."
        ),
        (1, Lang::He) => format!(
            "בתקופה אחת מתוך {windows} תקופות של {years} מאז {since}, ה\u{2011}S&P\u{a0}500 נגמר בהפסד (בדולרים, לפני עמלות ומס)."
        ),
        (_, Lang::En) => format!(
            "In {lower} of {windows} stretches of {years} since {since}, the S&P 500 ended lower \
             (in dollars, before fees and tax)."
        ),
        (_, Lang::He) => format!(
            "ב\u{2011}{lower} מתוך {windows} תקופות של {years} מאז {since}, ה\u{2011}S&P\u{a0}500 נגמר בהפסד (בדולרים, לפני עמלות ומס)."
        ),
    }
}

/// What the history sentence doesn't say by itself: the stretches overlap,
/// and the data ends on a date.
#[must_use]
pub fn history_caveat(band: &Band, lang: Lang) -> String {
    let until = month_year(band.to, lang);
    say(
        lang,
        format!(
            "The stretches overlap (each starts a month after the last), so they are far fewer \
             separate experiences than their number. The data runs to {until}, and the past is \
             not a promise. A fund in shekels also moves with the dollar's exchange rate, which the band leaves out."
        ),
        format!(
            "התקופות חופפות (כל אחת מתחילה חודש אחרי קודמתה), ולכן הן הרבה פחות ניסיונות נפרדים ממספרן. הנתונים מגיעים עד {until}, והעבר הוא לא הבטחה. קרן בשקלים זזה גם עם שער הדולר, שהטווח לא כולל."
        ),
    )
}

// ─────────────────────────── Safety ───────────────────────────

/// The tag beside a kind: how sure its end sum is.
#[must_use]
pub fn safety_tag(safety: Safety, lang: Lang) -> &'static str {
    match safety {
        Safety::SumKnown => lang.pick("Sum known in advance", "סכום ידוע מראש"),
        Safety::RateChanges => lang.pick("The rate changes", "הריבית משתנה"),
        Safety::RealValueKnown => {
            lang.pick("Buying power known in advance", "כוח הקנייה ידוע מראש")
        }
        Safety::CanFall => lang.pick("Can fall on the day", "יכול לרדת בדיוק אז"),
        Safety::NotLowerBefore => lang.pick("Never ended lower before", "בעבר לא נגמר בהפסד"),
    }
}

/// A sentence for the tag, for the tip beside it.
#[must_use]
pub fn safety_note(safety: Safety, lang: Lang) -> &'static str {
    match safety {
        Safety::SumKnown => lang.pick(
            "How much you get on the end date is set the day you buy.",
            "כמה תקבלו בתאריך הסיום נקבע כבר ביום הקנייה.",
        ),
        Safety::RateChanges => lang.pick(
            "It can't lose value, but its rate follows the Bank of Israel's, so nobody can \
             promise how much it comes to.",
            "אין בה סיכון לשוק, אבל הריבית שלה עוקבת אחרי ריבית בנק ישראל, ולכן אף אחד לא יכול להבטיח כמה יהיה בסוף.",
        ),
        Safety::RealValueKnown => lang.pick(
            "It follows the price index: what it will buy is known in advance, the sum in \
             shekels is not.",
            "היא צמודה למדד המחירים: מה שהיא תקנה ידוע מראש, והסכום בשקלים לא.",
        ),
        Safety::CanFall => lang.pick(
            "The market decides, and it can be down on the day you need the money.",
            "הבורסה מחליטה, והיא יכולה להיות בירידה בדיוק ביום שתצטרכו את הכסף.",
        ),
        Safety::NotLowerBefore => lang.pick(
            "The market decides. No stretch of this length in the history ended lower, but that \
             is the past, not a promise: it can still be down on the day you need the money.",
            "הבורסה מחליטה. אף תקופה באורך כזה בהיסטוריה לא נגמרה בהפסד, אבל זה העבר ולא הבטחה: היא עדיין יכולה להיות בירידה בדיוק ביום שתצטרכו את הכסף.",
        ),
    }
}

/// What safety costs: a safe kind against the index, over the same years,
/// both in today's money.
#[must_use]
pub fn what_safety_costs(safe: Amount, index: Amount, months: u32, lang: Lang) -> String {
    let years = span(months, lang);
    let (safe_text, index_text) = (shekels(safe.today), shekels(index.today));
    if safe.today >= index.today {
        return say(
            lang,
            format!(
                "Over {years} the safe kind would have left {safe_text}, no less than the \
                 index's {index_text}: here safety costs nothing, but the index's figure is an \
                 assumption and the safe kind's is not."
            ),
            format!(
                "ב\u{2011}{years} המקום הבטוח משאיר {safe_text}, לא פחות מ\u{2011}{index_text} של המדד: כאן הביטחון לא עולה כלום, אבל הסכום של המדד הוא הנחה והסכום של המקום הבטוח לא."
            ),
        );
    }
    let price = shekels(index.today - safe.today);
    say(
        lang,
        format!(
            "Safety over {years} costs about {price}: {safe_text} instead of {index_text}, in \
             today's money."
        ),
        format!(
            "הביטחון ל\u{2011}{years} עולה בערך {price}: {safe_text} במקום {index_text}, בכסף של היום."
        ),
    )
}

// ─────────────────────────── The reasons ───────────────────────────

/// The reason a circumstance gives for recommending `kind`.
#[must_use]
#[allow(clippy::too_many_lines, reason = "one arm for each circumstance")]
pub fn reason(circumstance: &Circumstance, kind: Kind, ctx: &Context, lang: Lang) -> String {
    let both = |en: &str, he: &str| pick(lang, en, he);
    match circumstance {
        Circumstance::NeedsAnyDay => both(
            "You may need this money on any day, so it has to be somewhere you can sell from on \
             any trading day, with no term to wait out. Only a money market fund is like that.",
            "ייתכן שתצטרכו את הכסף בכל יום, ולכן הוא צריך להיות במקום שאפשר למכור ממנו בכל יום מסחר, בלי תקופה שצריך לחכות לסופה. רק קרן כספית היא כזאת.",
        ),
        Circumstance::ShortHorizon { months } if *months > SAFE_ONLY_UP_TO_MONTHS => both(
            "No way of investing in the index can be opened with this first deposit, so the \
             money is shown where it can be reached at any time.",
            "אף דרך להשקיע במדד לא נפתחת עם ההפקדה הראשונה הזאת, ולכן הכסף מוצג במקום שאפשר להגיע אליו בכל עת.",
        ),
        Circumstance::ShortHorizon { months } => say(
            lang,
            format!(
                "{} is too short for the market: a fall can last longer than that, and then you \
                 would sell at a loss.",
                span(*months, lang)
            ),
            format!(
                "{} זה קצר מדי לבורסה: ירידה יכולה להימשך יותר מזה, ואז הייתם מוכרים בהפסד.",
                span(*months, lang)
            ),
        ),
        Circumstance::KnownDate { months } => say(
            lang,
            format!(
                "You have a date, {} from now. Within three years the market can easily be down \
                 on that very day, so only places that don't depend on the market are compared.",
                span(*months, lang)
            ),
            format!(
                "יש לכם תאריך, בעוד {}. עד שלוש שנים הבורסה יכולה בקלות להיות בירידה בדיוק באותו יום, ולכן משווים רק מקומות שלא תלויים בבורסה.",
                span(*months, lang)
            ),
        ),
        Circumstance::FixedDate { months } => say(
            lang,
            format!(
                "The date is fixed, {} from now: if the market is down exactly then, you can't \
                 wait. So only what has its sum known in advance is compared.",
                span(*months, lang)
            ),
            format!(
                "התאריך קבוע, בעוד {}: אם הבורסה תהיה בירידה בדיוק אז, לא תוכלו לחכות. לכן משווים רק מה שהסכום שלו ידוע מראש.",
                span(*months, lang)
            ),
        ),
        Circumstance::FlexibleDate { months } if *months >= INDEX_IF_FLEXIBLE_FROM_MONTHS => say(
            lang,
            format!(
                "The date can move, {} from now. From 5 years with a date that can move, \
                     stocks become reasonable: if the market is down then, you wait a year or \
                     two.",
                span(*months, lang)
            ),
            format!(
                "התאריך גמיש, בעוד {}. מחמש שנים, עם תאריך שאפשר להזיז, המניות כבר סבירות: אם הבורסה תהיה בירידה אז, מחכים שנה–שנתיים.",
                span(*months, lang)
            ),
        ),
        Circumstance::FlexibleDate { months } => say(
            lang,
            format!(
                "The date can move, {} from now, but under 5 years even a date that can move \
                 isn't enough for stocks. So only what has its sum known in advance is compared.",
                span(*months, lang)
            ),
            format!(
                "התאריך גמיש, בעוד {}, אבל עד חמש שנים גם תאריך גמיש לא מספיק למניות. לכן משווים רק מה שהסכום שלו ידוע מראש.",
                span(*months, lang)
            ),
        ),
        Circumstance::LongHorizon { months } if *months >= 84 => say(
            lang,
            format!(
                "{} is a long time: enough for the market to recover from most of the falls it \
                 has had.",
                span(*months, lang)
            ),
            format!(
                "{} זה טווח ארוך: מספיק זמן כדי שהבורסה תתאושש מרוב הירידות שהיו בה.",
                span(*months, lang)
            ),
        ),
        Circumstance::LongHorizon { months } => say(
            lang,
            format!(
                "The index kinds are worked out for {}. The market can still be down on the \
                 day, which the history below shows.",
                span(*months, lang)
            ),
            format!(
                "הדרכים שעוקבות אחרי המדד מחושבות ל\u{2011}{}. הבורסה עדיין יכולה להיות בירידה ביום הנכון, וההיסטוריה למטה מראה כמה.",
                span(*months, lang)
            ),
        ),
        Circumstance::NoDate => {
            if kind.arena() == Some(Arena::Index) {
                both(
                    "There is no date yet, so the money needs a place that can be opened at any \
                     time and meanwhile grows.",
                    "עוד אין תאריך, ולכן הכסף צריך מקום שאפשר למשוך ממנו בכל עת ושבינתיים מצמיח אותו.",
                )
            } else {
                both(
                    "There is no date yet, so the money stays where it can be sold on any \
                     trading day.",
                    "עוד אין תאריך, ולכן הכסף נשאר במקום שאפשר למכור ממנו בכל יום מסחר.",
                )
            }
        }
        Circumstance::OneSum => match kind {
            Kind::FixedDeposit => both(
                "It is one sum, which is exactly what a fixed-rate deposit takes.",
                "זה סכום אחד, וזה בדיוק מה שפיקדון בריבית קבועה מקבל.",
            ),
            _ => both(
                "It is one sum, not deposits every month, so you buy once and wait.",
                "זה סכום אחד ולא הפקדות חודשיות, אז קונים פעם אחת ומחכים.",
            ),
        },
        Circumstance::MonthlyDeposits => both(
            "Money comes in every month, and a fixed-rate deposit takes one sum only; this can \
             be bought a little more each month.",
            "הכסף נכנס כל חודש, ופיקדון בריבית קבועה מקבל סכום אחד בלבד; את זה אפשר לקנות קצת יותר בכל חודש.",
        ),
        Circumstance::EarnsBankRate { rate } => match kind {
            Kind::MoneyMarketFund => say(
                lang,
                format!(
                    "A money market fund earns about the Bank of Israel's rate, {rate} as you \
                     expect it, less the fund's fee."
                ),
                format!(
                    "קרן כספית מרוויחה בערך את ריבית בנק ישראל, {rate} כפי שציפיתם לה, פחות דמי הניהול של הקרן."
                ),
            ),
            _ => say(
                lang,
                format!("It pays about the Bank of Israel's rate, {rate}."),
                format!("היא משלמת בערך את ריבית בנק ישראל, {rate}."),
            ),
        },
        Circumstance::RateNotPromised { rate } => say(
            lang,
            format!(
                "A money market fund's rate follows the Bank of Israel's ({rate} now) and \
                 nobody promises it for so long."
            ),
            format!(
                "הריבית בקרן כספית עוקבת אחרי ריבית בנק ישראל ({rate} עכשיו), ואף אחד לא מבטיח אותה לזמן כזה."
            ),
        ),
        Circumstance::DepositLocked { months } => say(
            lang,
            format!(
                "The money is locked for the whole term, {}. Breaking a deposit early is on \
                 the deposit's own terms: read them before you sign.",
                span(*months, lang)
            ),
            format!(
                "הכסף ננעל לכל התקופה, {}. שבירה מוקדמת של פיקדון היא לפי התנאים שלו: קראו אותם לפני החתימה.",
                span(*months, lang)
            ),
        ),
        Circumstance::SumKnownInAdvance => match kind {
            Kind::FixedDeposit => both(
                "A fixed rate means what you get on the end date is known the day you open it.",
                "ריבית קבועה אומרת שכמה תקבלו בתאריך הסיום ידוע כבר ביום הפתיחה.",
            ),
            _ => both(
                "A government bond held to its repayment gives back a sum known the day you \
                 buy it.",
                "אג״ח ממשלתית שמחזיקים עד הפדיון מחזירה סכום שידוע כבר ביום הקנייה.",
            ),
        },
        Circumstance::RealValueKnownInAdvance => both(
            "A bond linked to the price index keeps its buying power: what it will buy is known \
             in advance, though the sum in shekels follows the index.",
            "אג״ח צמודה למדד שומרת על כוח הקנייה: מה שהיא תקנה ידוע מראש, אף על פי שהסכום בשקלים עוקב אחרי המדד.",
        ),
        Circumstance::BondCanBeSold => both(
            "A bond can be sold before it is repaid, at the day's price, which may be above or \
             below what you paid.",
            "אפשר למכור אג״ח לפני הפירעון, במחיר של אותו יום, שיכול להיות מעל או מתחת למה ששילמתם.",
        ),
        Circumstance::IfItsDownYouWait { months } => say(
            lang,
            format!(
                "If the market is down in {}, you don't sell: you wait a year or two until it \
                 recovers. That is what a date that can move allows.",
                span(*months, lang)
            ),
            format!(
                "אם הבורסה תהיה בירידה בעוד {}, לא מוכרים: מחכים שנה–שנתיים עד שהיא מתאוששת. זה מה שתאריך שאפשר להזיז מאפשר.",
                span(*months, lang)
            ),
        ),
        Circumstance::HistoryOfTheHorizon { .. } => match &ctx.band {
            Some(band) => history_sentence(band, lang),
            None => both(
                "The history holds no stretch this long.",
                "בהיסטוריה אין תקופה באורך כזה.",
            ),
        },
        Circumstance::WhatSafetyCosts => say(
            lang,
            format!(
                "Safe places earn about the Bank of Israel's rate ({}); the index is assumed to \
                 earn {} a year. Over {} that gap compounds: the safe kinds below show it in \
                 shekels.",
                ctx.rate,
                ctx.index_return,
                span(ctx.months, lang)
            ),
            format!(
                "מקומות בטוחים מרוויחים בערך את ריבית בנק ישראל ({}); המדד מחושב כמרוויח {} בשנה. ב\u{2011}{} הפער הזה מצטבר: המקומות הבטוחים למטה מראים אותו בשקלים.",
                ctx.rate,
                ctx.index_return,
                span(ctx.months, lang)
            ),
        ),
        Circumstance::CheapestAfterTax { cost_a_year } => {
            let scope = if matches!(ctx.answers.horizon, Horizon::Unknown) {
                lang.pick(
                    "of the ways that can be opened at any time",
                    "מהדרכים שאפשר למשוך מהן בכל עת",
                )
            } else {
                lang.pick("of the ways anyone can open", "מהדרכים שכל אחד יכול לפתוח")
            };
            match cost_a_year {
                Some(cost) => say(
                    lang,
                    format!(
                        "{scope}, this one leaves the most after fees and tax. Fees, and the \
                         fund's own cost, take about {cost} a year."
                    ),
                    format!(
                        "{scope}, כאן נשאר הכי הרבה אחרי עמלות ומס. העמלות והעלות של הקרן עצמה לוקחות בערך {cost} בשנה."
                    ),
                ),
                None => say(
                    lang,
                    format!("{scope}, this one leaves the most after fees and tax."),
                    format!("{scope}, כאן נשאר הכי הרבה אחרי עמלות ומס."),
                ),
            }
        }
        Circumstance::PricesRise { inflation } => say(
            lang,
            format!(
                "Prices rise about {inflation} a year, so money left in a current account buys \
                 less every year."
            ),
            format!("המחירים עולים בערך {inflation} בשנה, ולכן כסף בעו״ש קונה פחות משנה לשנה."),
        ),
        Circumstance::Retirement { age, retire_at } => say(
            lang,
            format!(
                "You are {age} and retire at {retire_at}: the money stays {}, long enough for \
                 the market.",
                span(ctx.months, lang)
            ),
            format!(
                "אתם בני {age} ופורשים בגיל {retire_at}: הכסף נשאר {}, מספיק זמן לבורסה.",
                span(ctx.months, lang)
            ),
        ),
        Circumstance::TaxFreePension { age_at_the_end } => say(
            lang,
            format!(
                "At {age_at_the_end} the fund can be taken as a monthly pension, and then its \
                 gain is tax-free. In a brokerage account the real gain pays 25%, and that is \
                 worth more than the fund's fee."
            ),
            format!(
                "בגיל {age_at_the_end} אפשר לקבל את הקופה כקצבה חודשית, ואז הרווח בה פטור ממס. בחשבון מסחר הרווח הריאלי משלם 25%, וזה שווה יותר מדמי הניהול של הקופה."
            ),
        ),
        Circumstance::PensionNeedsSixty { age_at_the_end } => say(
            lang,
            format!(
                "A pension from a provident fund is tax-free only from 60, and you will be \
                 {age_at_the_end}: a withdrawal before then pays 25% of the real gain, as in a \
                 brokerage account."
            ),
            format!(
                "קצבה מקופת גמל להשקעה פטורה ממס רק מגיל 60, ואתם תהיו בני {age_at_the_end}: משיכה לפני כן משלמת 25% מהרווח הריאלי, כמו בחשבון מסחר."
            ),
        ),
        Circumstance::DepositCeiling { ceiling } => say(
            lang,
            format!(
                "Your deposits fit under the fund's ceiling, {} a year in all of a person's \
                 funds together, and the money can be taken out at any time, with 25% tax on \
                 the real gain.",
                shekels(*ceiling)
            ),
            format!(
                "ההפקדות שלכם נכנסות מתחת לתקרה של הקופה, {} בשנה בכל הקופות של אותו אדם יחד, ואפשר למשוך את הכסף בכל עת, במס של 25% מהרווח הריאלי.",
                shekels(*ceiling)
            ),
        ),
        Circumstance::StudyFundNeedsEligibility => both(
            "A study fund opens for the self-employed, or through an employer.",
            "קרן השתלמות נפתחת לעצמאים, או דרך מעסיק.",
        ),
        Circumstance::NoCushion {
            monthly_expenses,
            target,
        } => say(
            lang,
            format!(
                "There is no cushion yet. Three months of {} is {}, a rule of thumb (3 to 6 \
                 months), and it comes first: without money at hand every sudden expense ends \
                 in selling an investment, maybe in a fall.",
                shekels(*monthly_expenses),
                shekels(*target)
            ),
            format!(
                "עוד אין כרית ביטחון. שלושה חודשים של {} זה {}, כלל אצבע (3–6 חודשים), וזה בא קודם: בלי כסף זמין כל הוצאה פתאומית נגמרת במכירה של השקעה, אולי בירידה.",
                shekels(*monthly_expenses),
                shekels(*target)
            ),
        ),
        Circumstance::HasCushion => both(
            "You already have a cushion, so this money doesn't have to wait for an emergency: \
             it can grow.",
            "כבר יש לכם כרית ביטחון, ולכן את הכסף הזה לא צריך לשמור לשעת חירום: הוא יכול לצמוח.",
        ),
        Circumstance::ExistingPortfolio => {
            let bank = bank_name(ctx.answers.situation, lang);
            say(
                lang,
                format!(
                    "You already have a portfolio at {bank}, so its branch price list is \
                     compared with the rest, to show what moving would change."
                ),
                format!(
                    "כבר יש לכם תיק ב{bank}, ולכן המחירון של הסניף שלו מושווה לשאר, כדי לראות מה ישתנה אם תעברו."
                ),
            )
        }
        Circumstance::KnowsWhatToBuy => both(
            "You know what you want to buy, so every way of holding it is compared side by side.",
            "אתם יודעים מה אתם רוצים לקנות, ולכן כל הדרכים להחזיק אותו מושוות זו לצד זו.",
        ),
        Circumstance::ChosenByYou => both(
            "You chose this yourself, so it is shown even where the rules would have chosen \
             otherwise.",
            "בחרתם בזה בעצמכם, ולכן זה מוצג גם במקום שבו הכללים היו בוחרים אחרת.",
        ),
    }
}

// ─────────────────────────── Why not ───────────────────────────

/// "₪a, against ₪b" when `left` is less than what was chosen leaves.
fn less_than_chosen(left: Option<Amount>, ctx: &Context, lang: Lang) -> Option<String> {
    let left = left?;
    (left.today < ctx.left.today).then(|| {
        say(
            lang,
            format!(
                "After tax it leaves {}, against {}.",
                shekels(left.today),
                shekels(ctx.left.today)
            ),
            format!(
                "אחרי מס נשאר בה {}, לעומת {}.",
                shekels(left.today),
                shekels(ctx.left.today)
            ),
        )
    })
}

/// Words joined by a space, skipping the missing.
fn join(parts: impl IntoIterator<Item = Option<String>>) -> String {
    parts.into_iter().flatten().collect::<Vec<_>>().join(" ")
}

/// Why `kind`, which was not chosen, is not what fits: `circumstance` is the
/// one of the saver's that decided it, `left` what it would have come to (in
/// its own terms, when it can be worked out), `ctx` the answer beside it.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "one arm for each kind and circumstance"
)]
pub fn why_not(
    kind: Kind,
    circumstance: &Circumstance,
    left: Option<Amount>,
    ctx: &Context,
    lang: Lang,
) -> String {
    let some = |text: String| Some(text);
    let both = |en: &str, he: &str| Some(pick(lang, en, he));
    let worse = less_than_chosen(left, ctx, lang);
    match (kind, circumstance) {
        (Kind::CurrentAccount, _) => {
            let inflation = ctx.inflation;
            let today = left.map_or(ctx.current_account.today, |amount| amount.today);
            say(
                lang,
                format!(
                    "Left in a current account it earns nothing and prices rise about \
                     {inflation} a year: what you put in is worth {} in today's money.",
                    shekels(today)
                ),
                format!(
                    "כשהכסף נשאר בעו״ש הוא לא מרוויח כלום והמחירים עולים בערך {inflation} בשנה: מה שהפקדתם שווה {} בכסף של היום.",
                    shekels(today)
                ),
            )
        }
        // The stock fund beside a safe answer.
        (Kind::Way(_), circumstance) if ctx.arena == Arena::Safe => {
            let rule = match circumstance {
                Circumstance::NeedsAnyDay => both(
                    "You may need the money on any day, and the market can be down on that day.",
                    "ייתכן שתצטרכו את הכסף בכל יום, והבורסה יכולה להיות בירידה באותו יום.",
                ),
                Circumstance::FixedDate { months } | Circumstance::KnownDate { months } => {
                    some(say(
                        lang,
                        format!(
                            "The date is fixed, {} away, and the market can be down exactly \
                             then.",
                            span(*months, lang)
                        ),
                        format!(
                            "התאריך קבוע, בעוד {}, והבורסה יכולה להיות בירידה בדיוק אז.",
                            span(*months, lang)
                        ),
                    ))
                }
                Circumstance::FlexibleDate { months } => some(say(
                    lang,
                    format!(
                        "Even with a date that can move, {} is under the 5 years from which \
                         stocks become reasonable.",
                        span(*months, lang)
                    ),
                    format!(
                        "גם עם תאריך שאפשר להזיז, {} זה פחות מחמש השנים שמהן המניות סבירות.",
                        span(*months, lang)
                    ),
                )),
                Circumstance::ShortHorizon { months } => some(say(
                    lang,
                    format!(
                        "{} is too short: a fall can last longer than that.",
                        span(*months, lang)
                    ),
                    format!(
                        "{} זה קצר מדי: ירידה יכולה להימשך יותר מזה.",
                        span(*months, lang)
                    ),
                )),
                Circumstance::NoDate => both(
                    "You don't know when you will need the money, and the market can be down \
                     on that day.",
                    "אתם לא יודעים מתי תצטרכו את הכסף, והבורסה יכולה להיות בירידה באותו יום.",
                ),
                _ => both(
                    "Its value can be down on the day you need it.",
                    "הערך שלה יכול להיות נמוך ביום שתצטרכו אותה.",
                ),
            };
            let history = match circumstance {
                Circumstance::FixedDate { .. }
                | Circumstance::KnownDate { .. }
                | Circumstance::FlexibleDate { .. } => {
                    ctx.band.as_ref().map(|band| history_sentence(band, lang))
                }
                _ => None,
            };
            let assumption = left.map(|_| {
                say(
                    lang,
                    format!(
                        "The figure drawn assumes {} a year, which nobody promises.",
                        ctx.index_return
                    ),
                    format!(
                        "הסכום שמצויר מניח {} בשנה, ואף אחד לא מבטיח אותו.",
                        ctx.index_return
                    ),
                )
            });
            join([rule, history, assumption])
        }
        // What the index kinds beside an index answer say.
        (Kind::Way(_), Circumstance::StudyFundNeedsEligibility) => {
            join([
                both(
                    "A study fund opens only for the self-employed, or through an employer, \
                     and its tax-free gain needs 6 years.",
                    "קרן השתלמות נפתחת רק לעצמאים, או דרך מעסיק, והרווח בה פטור ממס רק אחרי 6 שנים.",
                ),
                left.map(|_| {
                    pick(lang,
                        "The figure counts a self-employed saver's fund.",
                        "הסכום מחשב קרן של עצמאי.",
                    )
                }),
            ])
        }
        (Kind::Way(way), Circumstance::TaxFreePension { age_at_the_end }) => match way.vehicle {
            Vehicle::Brokerage => join([
                both(
                    "In a brokerage account the real gain pays 25% when you sell, and there is \
                     no pension to avoid it.",
                    "בחשבון מסחר הרווח הריאלי משלם 25% במכירה, ואין קצבה שפוטרת ממנו.",
                ),
                worse,
            ]),
            Vehicle::SavingsPolicy => join([
                both(
                    "A savings policy has no tax-free pension: its gain pays 25% when you take \
                     it out.",
                    "בפוליסת חיסכון אין קצבה פטורה ממס: הרווח בה משלם 25% במשיכה.",
                ),
                worse,
            ]),
            Vehicle::InvestmentGemel if *age_at_the_end < 60 => join([
                some(say(
                    lang,
                    format!(
                        "The fund's pension is tax-free only from 60, and you will be \
                         {age_at_the_end}: a withdrawal before then pays 25% of the real gain."
                    ),
                    format!(
                        "הקצבה מהקופה פטורה ממס רק מגיל 60, ואתם תהיו בני {age_at_the_end}: משיכה לפני כן משלמת 25% מהרווח הריאלי."
                    ),
                )),
                worse,
            ]),
            Vehicle::InvestmentGemel | Vehicle::StudyFund => join([
                both(
                    "Its pension is tax-free, but the fund's fees and the ceiling on deposits \
                     leave it behind what was chosen.",
                    "הקצבה בה פטורה ממס, אבל דמי הניהול של הקופה והתקרה להפקדות משאירים אותה מאחורי מה שנבחר.",
                ),
                worse,
            ]),
        },
        // The safe kinds beside an index answer.
        (kind, Circumstance::WhatSafetyCosts) => {
            let cost = left.map(|safe| what_safety_costs(safe, ctx.left, ctx.months, lang));
            let promise = match kind {
                Kind::MoneyMarketFund => both(
                    "Its rate follows the Bank of Israel's, so even its sum isn't promised.",
                    "הריבית בה עוקבת אחרי ריבית בנק ישראל, ולכן גם הסכום שלה לא מובטח.",
                ),
                Kind::FixedDeposit => both(
                    "The deposit locks the money and takes one sum.",
                    "הפיקדון נועל את הכסף ומקבל סכום אחד.",
                ),
                Kind::GovernmentBond { linked: true } => both(
                    "Its sum in shekels follows the index.",
                    "הסכום שלה בשקלים עוקב אחרי המדד.",
                ),
                _ => both(
                    "Its sum is known in advance.",
                    "הסכום שלה ידוע מראש.",
                ),
            };
            join([cost, promise])
        }
        (Kind::MoneyMarketFund, Circumstance::RateNotPromised { rate }) => join([
            some(say(
                lang,
                format!(
                    "Its rate follows the Bank of Israel's ({rate} now) and nobody promises it \
                     for so long."
                ),
                format!("הריבית בה עוקבת אחרי ריבית בנק ישראל ({rate} עכשיו), ואף אחד לא מבטיח אותה לזמן כזה."),
            )),
            left.map(|amount| {
                say(
                    lang,
                    format!(
                        "At today's rate it would leave {}, but it can come to less.",
                        shekels(amount.today)
                    ),
                    format!(
                        "בריבית של היום נשאר בה {}, אבל זה יכול לצאת פחות.",
                        shekels(amount.today)
                    ),
                )
            }),
        ]),
        (Kind::FixedDeposit, Circumstance::MonthlyDeposits) => both(
            "A deposit takes one sum, and money comes in every month.",
            "פיקדון מקבל סכום אחד, והכסף נכנס כל חודש.",
        )
        .unwrap_or_default(),
        (Kind::FixedDeposit, Circumstance::NeedsAnyDay | Circumstance::DepositLocked { .. }) => {
            both(
                "A deposit is locked until its term ends, and you may need the money before.",
                "פיקדון נעול עד סוף התקופה שלו, ואולי תצטרכו את הכסף לפני כן.",
            )
            .unwrap_or_default()
        }
        (Kind::FixedDeposit, _) => join([
            worse,
            both(
                "The money is locked until the term ends.",
                "הכסף נעול עד סוף התקופה.",
            ),
        ]),
        (Kind::GovernmentBond { .. }, Circumstance::NoDate) => both(
            "A bond has a repayment date and you don't know when you will need the money; it \
             can be sold before, at the day's price.",
            "לאג״ח יש תאריך פדיון ואתם לא יודעים מתי תצטרכו את הכסף; אפשר למכור לפני, במחיר של אותו יום.",
        )
        .unwrap_or_default(),
        (Kind::GovernmentBond { linked: true }, _) => join([
            both(
                "A linked bond keeps its buying power, but its sum in shekels follows the \
                 index, so it isn't known in advance.",
                "אג״ח צמודה שומרת על כוח הקנייה, אבל הסכום שלה בשקלים עוקב אחרי המדד ולכן לא ידוע מראש.",
            ),
            worse,
        ]),
        (Kind::GovernmentBond { linked: false }, _) => join([
            worse,
            both(
                "It needs a trading account, whose fees come off the yield.",
                "היא דורשת חשבון מסחר, ועמלותיו יורדות מהתשואה.",
            ),
        ]),
        (Kind::MoneyMarketFund, _) => join([
            both(
                "Its rate follows the Bank of Israel's, so what it comes to isn't known in \
                 advance.",
                "הריבית בה עוקבת אחרי ריבית בנק ישראל, ולכן לא ידוע מראש כמה יהיה בסוף.",
            ),
            worse,
        ]),
        (Kind::Way(_) | Kind::LinkedDeposit, _) => join([
            worse,
            both(
                "Another way of holding the money, which leaves less here.",
                "דרך אחרת להחזיק את הכסף, שמשאירה כאן פחות.",
            ),
        ]),
    }
}

// ─────────────────────────── The unlock line and the cushion ───────────────────────────

/// What a bigger first deposit would open, as a line under the answer.
#[must_use]
pub fn unlock_text(unlock: &UnlockLine, lang: Lang) -> String {
    let provider = &unlock.plan.provider[lang];
    let plan = &unlock.plan.plan[lang];
    let (now_en, now_he) = if unlock.first_deposit_now.is_zero() {
        ("you have none now".to_owned(), "עכשיו אין לכם".to_owned())
    } else {
        (
            format!("now {}", shekels(unlock.first_deposit_now)),
            format!("עכשיו {}", shekels(unlock.first_deposit_now)),
        )
    };
    say(
        lang,
        format!(
            "With a first deposit of {} ({now_en}) you could open {provider}, {plan}: it would leave {} \
             against {} for the kind chosen above with the same first deposit, in today's money. \
             Both include the extra first deposit, so they are higher than the amount above.",
            shekels(unlock.first_deposit),
            shekels(unlock.after_tax.today),
            shekels(unlock.instead_of.today)
        ),
        format!(
            "עם הפקדה ראשונה של {} ({now_he}) אפשר לפתוח את {provider}, {plan}: שם נשאר {} לעומת {} בסוג שנבחר למעלה עם אותה הפקדה ראשונה, בכסף של היום. שני הסכומים כוללים את ההפקדה הראשונה הנוספת, ולכן הם גבוהים מהסכום למעלה.",
            shekels(unlock.first_deposit),
            shekels(unlock.after_tax.today),
            shekels(unlock.instead_of.today)
        ),
    )
}

/// The cushion's two parts: first the cushion, then where the monthly money
/// goes. `monthly` is what the saver puts in each month. The thing named in
/// each part is a bold span.
#[must_use]
pub fn cushion_text(plan: &CushionPlan, monthly: Decimal, lang: Lang) -> (Vec<Span>, Vec<Span>) {
    let first = vec![
        Span::bold(pick(
            lang,
            "First, a cushion in a money market fund",
            "קודם, כרית ביטחון בקרן כספית",
        )),
        Span::plain(say(
            lang,
            format!(
                ": {} a month until {} gathers (three months of expenses), about {}.",
                shekels(monthly),
                shekels(plan.target),
                span(plan.months_to_fill, lang)
            ),
            format!(
                ": {} בחודש עד שיצטברו {} (שלושה חודשי הוצאות), בערך {}.",
                shekels(monthly),
                shekels(plan.target),
                span(plan.months_to_fill, lang)
            ),
        )),
    ];
    let then = plan.then.name(lang);
    let second = vec![
        Span::plain(pick(lang, "Then ", "אחר כך ")),
        Span::bold(then),
        Span::plain(say(
            lang,
            format!(
                ", for the long run: of the places that can be opened at any time, it leaves \
                 the most after tax over {}.",
                span(plan.then_years * 12, lang)
            ),
            format!(
                ", לטווח ארוך: ממקומות שאפשר למשוך מהם בכל עת, זה מה שמשאיר הכי הרבה אחרי מס ב\u{2011}{}.",
                span(plan.then_years * 12, lang)
            ),
        )),
    ];
    (first, second)
}
