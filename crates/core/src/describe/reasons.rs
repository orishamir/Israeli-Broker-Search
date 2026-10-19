//! PLACEHOLDER texts of the guide: the reasons, why the alternatives are not
//! what fits, and the steps. Short, but real enough to read. The words
//! package replaces the bodies, never the signatures: `reason`, `why_not` and
//! `steps_for` are what `guide` calls.

use rust_decimal::Decimal;

use crate::describe::format_money;
use crate::guide::{Amount, Circumstance, Context, Kind, Recommendation, Step};
use crate::{Lang, Named, ils};

/// ₪ to the shekel: "₪51,000".
fn shekels(amount: Decimal) -> String {
    format_money(ils(amount.round_dp(0)))
}

/// A number of months as the saver thinks of it: "5 years", "18 months".
fn span(months: u32, lang: Lang) -> String {
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

/// The reason a circumstance gives for recommending `kind`.
#[must_use]
#[allow(clippy::too_many_lines, reason = "one arm for each circumstance")]
pub fn reason(circumstance: &Circumstance, kind: Kind, ctx: &Context, lang: Lang) -> String {
    let kind_name = kind.name(lang);
    match circumstance {
        Circumstance::NeedsAnyDay => lang
            .pick(
                "You may need this money on any day, so it has to be somewhere it can be \
                 reached the next day.",
                "ייתכן שתצטרכו את הכסף בכל יום, ולכן הוא צריך להיות במקום שאפשר למשוך ממנו ביום המחרת.",
            )
            .to_owned(),
        Circumstance::ShortHorizon { months } => match lang {
            Lang::En => format!(
                "{} is too short for the market: a fall can take longer than that to recover.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "{} זה קצר מדי לבורסה: ירידה יכולה לקחת יותר זמן להתאושש.",
                span(*months, lang)
            ),
        },
        Circumstance::KnownDate { months } => match lang {
            Lang::En => format!(
                "You know the date, {} from now, so what you want is to know in advance what \
                 you will have.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "התאריך ידוע, בעוד {}, ולכן מה שחשוב הוא לדעת מראש כמה יהיה בסוף.",
                span(*months, lang)
            ),
        },
        Circumstance::FixedDate { months } => match lang {
            Lang::En => format!(
                "The date is fixed, {} from now: stocks could be down exactly then.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "התאריך קבוע, בעוד {}: אפשר שהמניות יהיו בירידה בדיוק אז.",
                span(*months, lang)
            ),
        },
        Circumstance::FlexibleDate { months } => match lang {
            Lang::En => format!(
                "The date can move, {} from now. From 5 years with a date that can move, stocks \
                 become reasonable: if they are down then, you wait.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "התאריך גמיש, בעוד {}. מחמש שנים, עם תאריך גמיש, המניות כבר סבירות: אם הן בירידה אז, מחכים.",
                span(*months, lang)
            ),
        },
        Circumstance::LongHorizon { months } => match lang {
            Lang::En => format!(
                "{} is long enough for the market's ups and downs to even out, as they have \
                 before.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "{} זה מספיק זמן כדי שעליות וירידות של הבורסה יתאזנו, כמו שקרה בעבר.",
                span(*months, lang)
            ),
        },
        Circumstance::NoDate => lang
            .pick(
                "There is no date yet, so the money should be somewhere that grows and can be \
                 opened any time.",
                "עוד אין תאריך, ולכן הכסף צריך להיות במקום שמצמיח אותו ואפשר לפתוח אותו בכל עת.",
            )
            .to_owned(),
        Circumstance::OneSum => lang
            .pick(
                "It is one sum, which is what a deposit takes.",
                "זה סכום אחד, וזה מה שפיקדון מקבל.",
            )
            .to_owned(),
        Circumstance::MonthlyDeposits => lang
            .pick(
                "Money comes in every month, and a fixed-rate deposit takes one sum only.",
                "הכסף נכנס כל חודש, ופיקדון בריבית קבועה מקבל סכום אחד בלבד.",
            )
            .to_owned(),
        Circumstance::EarnsBankRate { rate } => match lang {
            Lang::En => format!(
                "A {kind_name} earns about the Bank of Israel's rate, {rate}, less its fee."
            ),
            Lang::He => {
                format!("{kind_name} מרוויחה בערך את ריבית בנק ישראל, {rate}, פחות דמי הניהול שלה.")
            }
        },
        Circumstance::RateNotPromised { rate } => match lang {
            Lang::En => format!(
                "A money market fund's rate follows the Bank of Israel's ({rate} now) and is \
                 not promised for so long."
            ),
            Lang::He => format!(
                "הריבית בקרן כספית עוקבת אחרי ריבית בנק ישראל ({rate} עכשיו) ואינה מובטחת לזמן כזה."
            ),
        },
        Circumstance::DepositLocked { months } => match lang {
            Lang::En => format!(
                "A deposit is locked for its whole term, {}.",
                span(*months, lang)
            ),
            Lang::He => format!("פיקדון נעול לכל התקופה שלו, {}.", span(*months, lang)),
        },
        Circumstance::SumKnownInAdvance => lang
            .pick(
                "What you will have at the end is known the day you buy.",
                "כמה יהיה בסוף ידוע כבר ביום הקנייה.",
            )
            .to_owned(),
        Circumstance::RealValueKnownInAdvance => lang
            .pick(
                "What it will buy is known in advance, but the sum in shekels follows the index.",
                "מה שהוא יקנה ידוע מראש, אבל הסכום בשקלים עוקב אחרי המדד.",
            )
            .to_owned(),
        Circumstance::BondCanBeSold => lang
            .pick(
                "A bond can be sold before it is repaid, at the day's price.",
                "אפשר למכור אג״ח לפני הפירעון, במחיר של אותו יום.",
            )
            .to_owned(),
        Circumstance::IfItsDownYouWait { months } => match lang {
            Lang::En => format!(
                "If the market is down when the {} are over, you wait.",
                span(*months, lang)
            ),
            Lang::He => format!(
                "אם הבורסה תהיה בירידה כש{} יסתיימו, תחכו.",
                span(*months, lang)
            ),
        },
        Circumstance::HistoryOfTheHorizon { years } => match (&ctx.band, lang) {
            (Some(band), Lang::En) => format!(
                "In {} of {} stretches of {} years since {}, the S&P 500 ended lower (in dollars).",
                band.lower, band.windows, years.normalize(), band.from.year()
            ),
            (Some(band), Lang::He) => format!(
                "ב\u{2011}{} מתוך {} תקופות של {} שנים מאז {}, ה\u{2011}S&P\u{a0}500 נגמר בהפסד (בדולרים).",
                band.lower, band.windows, years.normalize(), band.from.year()
            ),
            (None, _) => lang
                .pick(
                    "The history holds no stretch this long.",
                    "בהיסטוריה אין תקופה באורך כזה.",
                )
                .to_owned(),
        },
        Circumstance::WhatSafetyCosts => match lang {
            Lang::En => format!(
                "Safety over this long costs the difference: {} instead of {}, in today's money.",
                shekels(ctx.current_account.today),
                shekels(ctx.left.today)
            ),
            Lang::He => format!(
                "ביטחון לאורך זמן כזה עולה את ההפרש: {} במקום {}, בכסף של היום.",
                shekels(ctx.current_account.today),
                shekels(ctx.left.today)
            ),
        },
        Circumstance::CheapestAfterTax { cost_a_year } => match (cost_a_year, lang) {
            (Some(cost), Lang::En) => format!(
                "It leaves the most after fees and tax; fees and the fund's own cost take about \
                 {cost} a year."
            ),
            (Some(cost), Lang::He) => format!(
                "זה מה שמשאיר הכי הרבה אחרי עמלות ומס; העמלות והעלות של הקרן עצמה לוקחות בערך {cost} בשנה."
            ),
            (None, _) => lang
                .pick(
                    "It leaves the most after fees and tax.",
                    "זה מה שמשאיר הכי הרבה אחרי עמלות ומס.",
                )
                .to_owned(),
        },
        Circumstance::PricesRise { inflation } => match lang {
            Lang::En => format!(
                "Prices rise about {inflation} a year, so money left in a current account buys \
                 less each year."
            ),
            Lang::He => format!(
                "המחירים עולים בערך {inflation} בשנה, ולכן כסף בעו״ש קונה פחות משנה לשנה."
            ),
        },
        Circumstance::Retirement { age, retire_at } => match lang {
            Lang::En => format!("You are {age} and retire at {retire_at}."),
            Lang::He => format!("אתם בני {age} ופורשים בגיל {retire_at}."),
        },
        Circumstance::TaxFreePension { age_at_the_end } => match lang {
            Lang::En => format!(
                "At {age_at_the_end} a provident fund for investment can be taken as a pension, \
                 tax-free; in a brokerage account the gain pays 25%."
            ),
            Lang::He => format!(
                "בגיל {age_at_the_end} אפשר למשוך קופת גמל להשקעה כקצבה, בלי מס; בחשבון מסחר הרווח משלם 25%."
            ),
        },
        Circumstance::PensionNeedsSixty { age_at_the_end } => match lang {
            Lang::En => format!(
                "The fund's pension is tax-free only from 60, and you will be {age_at_the_end}."
            ),
            Lang::He => format!(
                "הקצבה מהקופה פטורה ממס רק מגיל 60, ואתם תהיו בני {age_at_the_end}."
            ),
        },
        Circumstance::DepositCeiling { ceiling } => match lang {
            Lang::En => format!(
                "A provident fund for investment takes up to {} a year.",
                shekels(*ceiling)
            ),
            Lang::He => format!("בקופת גמל להשקעה אפשר להפקיד עד {} בשנה.", shekels(*ceiling)),
        },
        Circumstance::StudyFundNeedsEligibility => lang
            .pick(
                "A study fund opens for the self-employed, or through an employer.",
                "קרן השתלמות נפתחת לעצמאים, או דרך מעסיק.",
            )
            .to_owned(),
        Circumstance::NoCushion {
            monthly_expenses,
            target,
        } => match lang {
            Lang::En => format!(
                "There is no cushion yet: three months of {} is {}, which comes first, so a \
                 sudden expense doesn't force you to sell an investment in a fall.",
                shekels(*monthly_expenses),
                shekels(*target)
            ),
            Lang::He => format!(
                "עוד אין כרית ביטחון: שלושה חודשים של {} זה {}, וזה בא קודם, כדי שהוצאה פתאומית לא תכריח אתכם למכור השקעה בירידות.",
                shekels(*monthly_expenses),
                shekels(*target)
            ),
        },
        Circumstance::HasCushion => lang
            .pick(
                "You already have a cushion, so this money can grow.",
                "כבר יש לכם כרית ביטחון, ולכן הכסף הזה יכול לצמוח.",
            )
            .to_owned(),
        Circumstance::ExistingPortfolio => lang
            .pick(
                "Your bank's branch prices are compared with the rest, to see what moving would change.",
                "המחירים של הסניף בבנק שלכם מושווים לשאר, כדי לראות מה ישתנה אם תעברו.",
            )
            .to_owned(),
        Circumstance::KnowsWhatToBuy => lang
            .pick(
                "You know what you want, so every way of holding it is compared.",
                "אתם יודעים מה אתם רוצים, ולכן כל הדרכים להחזיק אותו מושוות.",
            )
            .to_owned(),
        Circumstance::ChosenByYou => lang
            .pick("You chose this yourself.", "בחרתם בזה בעצמכם.")
            .to_owned(),
    }
}

/// Why `kind`, which was not chosen, is not what fits: `circumstance` is the
/// one of the saver's that decided it, `left` what it would have come to (in
/// its own terms, when it can be worked out), `ctx` the answer beside it.
#[must_use]
pub fn why_not(
    kind: Kind,
    circumstance: &Circumstance,
    left: Option<Amount>,
    ctx: &Context,
    lang: Lang,
) -> String {
    let kind_name = kind.name(lang);
    let reason = match (kind, circumstance) {
        (Kind::CurrentAccount, _) => lang
            .pick(
                "Left as it is, it earns nothing and prices eat into it.",
                "כשהוא נשאר כמו שהוא, הוא לא מרוויח כלום והמחירים אוכלים אותו.",
            )
            .to_owned(),
        (Kind::Way(_), Circumstance::FixedDate { months } | Circumstance::KnownDate { months }) => {
            match lang {
                Lang::En => format!(
                    "The date is fixed, {} away, and the market can be down then.",
                    span(*months, lang)
                ),
                Lang::He => format!(
                    "התאריך קבוע, בעוד {}, והבורסה יכולה להיות בירידה אז.",
                    span(*months, lang)
                ),
            }
        }
        (Kind::Way(_), _) => lang
            .pick(
                "Its value can be down on the day you need it.",
                "הערך שלה יכול להיות נמוך ביום שתצטרכו אותה.",
            )
            .to_owned(),
        (Kind::MoneyMarketFund, Circumstance::RateNotPromised { rate }) => match lang {
            Lang::En => {
                format!("Its rate follows the Bank of Israel's ({rate} now) and is not promised.")
            }
            Lang::He => {
                format!("הריבית בה עוקבת אחרי ריבית בנק ישראל ({rate} עכשיו) ואינה מובטחת.")
            }
        },
        (Kind::FixedDeposit, Circumstance::MonthlyDeposits) => lang
            .pick(
                "A deposit takes one sum, and money comes in every month.",
                "פיקדון מקבל סכום אחד, והכסף נכנס כל חודש.",
            )
            .to_owned(),
        (Kind::FixedDeposit, Circumstance::NeedsAnyDay) => lang
            .pick(
                "A deposit is locked until its term ends.",
                "פיקדון נעול עד סוף התקופה שלו.",
            )
            .to_owned(),
        _ => match lang {
            Lang::En => format!("{kind_name} leaves less after tax for money like this."),
            Lang::He => format!("{kind_name} משאירה פחות אחרי מס לכסף כזה."),
        },
    };
    let _ = (left, ctx);
    reason
}

/// How to do what was chosen, step by step.
#[must_use]
pub fn steps_for(recommendation: &Recommendation, lang: Lang) -> Vec<Step> {
    let kind_name = recommendation.kind.name(lang);
    let provider = &recommendation.choice.provider[lang];
    let plan = &recommendation.choice.plan[lang];
    let step = |title: &str, text: String| Step {
        title: title.to_owned(),
        text,
    };
    vec![
        step(
            lang.pick("Open an account", "פותחים חשבון"),
            match lang {
                Lang::En => format!("Open it with {provider}, on the plan {plan}."),
                Lang::He => format!("פותחים ב{provider}, במסלול {plan}."),
            },
        ),
        step(
            lang.pick("Buy", "קונים"),
            match lang {
                Lang::En => format!("Ask for a {kind_name} in the bank's app or at the branch."),
                Lang::He => format!("מבקשים {kind_name} באפליקציה של הבנק או בסניף."),
            },
        ),
        step(
            lang.pick("Keep going", "ממשיכים"),
            lang.pick(
                "Deposit on the same day each month, and leave it alone until the date.",
                "מפקידים באותו יום בכל חודש, ולא נוגעים עד התאריך.",
            )
            .to_owned(),
        ),
    ]
}
