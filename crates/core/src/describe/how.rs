//! "איך עושים את זה" (REDESIGN.md §2.8): the identity line of each product,
//! what to type in the app, and the steps. First drafts: the bond series'
//! names and the funds' numbers are research still to be done, so their
//! search slots are marked placeholders, never invented.

#![allow(
    clippy::too_many_lines,
    clippy::format_push_string,
    reason = "the words of every product"
)]

use rust_decimal::Decimal;
use time::Date;

use super::card::{both, end_date};
use super::reasons::{shekels, span};
use crate::guide::{Account, Kind};
use crate::{Lang, Product, Text, Vehicle};

/// What the words need.
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub kind: Kind,
    pub now: Decimal,
    pub monthly: Decimal,
    pub months: u32,
    pub today: Option<Date>,
    /// A cushion to fill first: its target and months.
    pub cushion: Option<(Decimal, u32)>,
    pub account: Option<Account>,
}

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// "repaid in 2029", or "repaid in 3 years".
fn repaid(ctx: &Context, lang: Lang) -> String {
    match ctx.today.and_then(|today| end_date(today, ctx.months)) {
        Some(end) => format!(
            "{}{}",
            lang.pick("repaid in ", "נפדית ב\u{2011}"),
            end.year()
        ),
        None => format!(
            "{} {}",
            lang.pick("repaid in", "נפדית בעוד"),
            span(ctx.months, lang)
        ),
    }
}

/// The identity line: who issues it, where it trades, its currency, what
/// it's linked to, when the money comes out.
#[must_use]
pub fn identity(ctx: &Context) -> Vec<Text> {
    match ctx.kind {
        Kind::MoneyMarketFund => vec![
            fixed("A mutual fund in shekels", "קרן נאמנות בשקלים"),
            fixed(
                "Lends money short-term to the state and the banks",
                "מלווה כסף לזמן קצר למדינה ולבנקים",
            ),
            fixed("Bought in the bank's app", "נקנית באפליקציה של הבנק"),
            fixed("Sold any trading day", "נמכרת בכל יום מסחר"),
            fixed(
                "Its rate moves with the Bank of Israel's",
                "הריבית משתנה עם ריבית בנק ישראל",
            ),
        ],
        Kind::GovernmentBond { linked } => vec![
            fixed("A loan to the State of Israel", "הלוואה למדינת ישראל"),
            fixed(
                "Traded on the Tel Aviv Stock Exchange",
                "נסחרת בבורסה בתל אביב",
            ),
            if linked {
                fixed(
                    "In shekels, linked to the consumer price index",
                    "בשקלים, צמודה למדד המחירים לצרכן",
                )
            } else {
                fixed("In shekels, not linked", "בשקלים, לא צמודה")
            },
            both(|lang| repaid(ctx, lang)),
        ],
        Kind::FixedDeposit | Kind::LinkedDeposit => vec![
            fixed("A deposit at a bank, in shekels", "פיקדון בבנק, בשקלים"),
            fixed(
                "A fixed rate: the sum is known in advance",
                "ריבית קבועה: הסכום ידוע מראש",
            ),
            both(|lang| match lang {
                Lang::En => format!("Locked for {}", span(ctx.months, lang)),
                Lang::He => format!("הכסף נעול ל\u{2011}{}", span(ctx.months, lang)),
            }),
        ],
        Kind::CurrentAccount => vec![],
        Kind::Way(way) => {
            let holds = fixed(
                "A fund that holds the 500 largest US companies (the S&P 500)",
                "קרן שמחזיקה את 500 החברות הגדולות בארה״ב (מדד S&P\u{a0}500)",
            );
            match (way.product, way.vehicle) {
                (_, Vehicle::StudyFund) => vec![
                    fixed("Savings through a fund company", "חיסכון דרך חברה מנהלת"),
                    fixed(
                        "Opens for the self-employed, or through an employer",
                        "נפתחת לעצמאים, או דרך מעסיק",
                    ),
                    fixed(
                        "The gain is tax-free after 6 years",
                        "הרווח פטור ממס אחרי 6 שנים",
                    ),
                ],
                (_, Vehicle::InvestmentGemel) => vec![
                    fixed("Savings through a fund company", "חיסכון דרך חברה מנהלת"),
                    fixed(
                        "Anyone can open one, up to a yearly ceiling",
                        "כל אחד יכול לפתוח, עד תקרה שנתית",
                    ),
                    fixed(
                        "Can be taken out any time; as a pension from 60, the gain is tax-free",
                        "אפשר למשוך בכל עת; כקצבה מגיל 60 הרווח פטור ממס",
                    ),
                ],
                (_, Vehicle::SavingsPolicy) => vec![
                    fixed(
                        "Savings through an insurance company",
                        "חיסכון דרך חברת ביטוח",
                    ),
                    fixed("Can be taken out any time", "אפשר למשוך בכל עת"),
                ],
                (
                    Product::IsraeliIndexFund
                    | Product::StudyFundTrack
                    | Product::InvestmentGemelTrack
                    | Product::SavingsPolicyTrack,
                    _,
                ) => vec![
                    fixed(
                        "A mutual fund that follows the S&P 500 index",
                        "קרן נאמנות שעוקבת אחרי מדד S&P\u{a0}500",
                    ),
                    fixed(
                        "Bought once a day, at the day's closing price",
                        "נקנית פעם ביום, במחיר של סוף היום",
                    ),
                    fixed(
                        "In shekels; moves with the shares and with the dollar",
                        "בשקלים, עולה ויורדת עם המניות ועם הדולר",
                    ),
                ],
                (Product::IsraeliEtf, _) => vec![
                    holds,
                    fixed(
                        "Traded on the Tel Aviv Stock Exchange, in shekels",
                        "נסחרת בבורסה בתל אביב, בשקלים",
                    ),
                    fixed("Sold any trading day", "נמכרת בכל יום מסחר"),
                ],
                (Product::UsFund, _) => vec![
                    holds,
                    fixed(
                        "Traded on a US exchange, in dollars",
                        "נסחרת בבורסה בארה״ב, בדולרים",
                    ),
                    fixed("Sold any trading day", "נמכרת בכל יום מסחר"),
                ],
                (Product::IrishFund | Product::ForeignEtfInTelAviv, _) => vec![
                    holds,
                    fixed(
                        "Registered in Ireland, traded in Europe",
                        "רשומה באירלנד, נסחרת באירופה",
                    ),
                    fixed("Sold any trading day", "נמכרת בכל יום מסחר"),
                ],
            }
        }
    }
}

/// What to type in the app, and whether the slot is still a placeholder.
#[must_use]
pub fn search(ctx: &Context) -> Option<(Text, bool)> {
    let at = |lang: Lang| match ctx.today.and_then(|today| end_date(today, ctx.months)) {
        Some(end) => end.year().to_string(),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(ctx.months, lang)),
    };
    match ctx.kind {
        Kind::MoneyMarketFund => Some((
            fixed("money market fund in shekels (כספית שקלית)", "כספית שקלית"),
            false,
        )),
        Kind::GovernmentBond { linked } => Some((
            both(|lang| match (lang, linked) {
                (Lang::En, true) => {
                    format!("ממשלתי צמוד ____ (the series repaid nearest {})", at(lang))
                }
                (Lang::He, true) => format!(
                    "ממשלתי צמוד ____ (הסדרה שנפדית הכי קרוב ל\u{2011}{})",
                    at(lang)
                ),
                (Lang::En, false) => {
                    format!("ממשלתי שקלי ____ (the series repaid nearest {})", at(lang))
                }
                (Lang::He, false) => format!(
                    "ממשלתי שקלי ____ (הסדרה שנפדית הכי קרוב ל\u{2011}{})",
                    at(lang)
                ),
            }),
            true,
        )),
        Kind::FixedDeposit | Kind::LinkedDeposit => Some((
            both(|lang| match lang {
                Lang::En => format!("fixed-rate deposit, {}", span(ctx.months, lang)),
                Lang::He => format!("פיקדון בריבית קבועה, {}", span(ctx.months, lang)),
            }),
            false,
        )),
        Kind::CurrentAccount => None,
        Kind::Way(way) => match (way.product, way.vehicle) {
            (_, Vehicle::StudyFund | Vehicle::InvestmentGemel | Vehicle::SavingsPolicy) => Some((
                fixed(
                    "the track that follows the S&P 500 index",
                    "מסלול שעוקב אחרי מדד S&P\u{a0}500",
                ),
                false,
            )),
            (Product::UsFund, _) => Some((
                fixed(
                    "for example VOO (an example, not a recommendation)",
                    "למשל VOO (דוגמה, לא המלצה)",
                ),
                false,
            )),
            (Product::IrishFund | Product::ForeignEtfInTelAviv, _) => Some((
                fixed(
                    "for example CSPX (an example, not a recommendation)",
                    "למשל CSPX (דוגמה, לא המלצה)",
                ),
                false,
            )),
            (
                Product::IsraeliIndexFund
                | Product::StudyFundTrack
                | Product::InvestmentGemelTrack
                | Product::SavingsPolicyTrack,
                _,
            ) => Some((
                fixed(
                    "S&P 500 index fund · security number: [—]",
                    "קרן מחקה S&P\u{a0}500 · מספר נייר: [—]",
                ),
                true,
            )),
            (Product::IsraeliEtf, _) => Some((
                fixed(
                    "S&P 500 ETF · security number: [—]",
                    "קרן סל S&P\u{a0}500 · מספר נייר: [—]",
                ),
                true,
            )),
        },
    }
}

fn money(ctx: &Context, lang: Lang) -> String {
    match (ctx.now.is_zero(), ctx.monthly.is_zero(), lang) {
        (false, false, Lang::En) => format!(
            "{} now, and {} every month",
            shekels(ctx.now),
            shekels(ctx.monthly)
        ),
        (false, false, Lang::He) => format!(
            "{} עכשיו, ועוד {} בכל חודש",
            shekels(ctx.now),
            shekels(ctx.monthly)
        ),
        (true, _, Lang::En) => format!("{} every month", shekels(ctx.monthly)),
        (true, _, Lang::He) => format!("{} בכל חודש", shekels(ctx.monthly)),
        (false, true, _) => shekels(ctx.now),
    }
}

/// The steps, in order.
#[must_use]
pub fn steps(ctx: &Context) -> Vec<Text> {
    let mut steps = Vec::new();
    if let (Some((target, _)), Kind::Way(_)) = (ctx.cushion, ctx.kind) {
        steps.push(both(|lang| match lang {
            Lang::En => format!(
                "First: {} a month to a money market fund in the bank's app, until {}.",
                shekels(ctx.monthly),
                shekels(target)
            ),
            Lang::He => format!(
                "קודם: {} בחודש לקרן כספית באפליקציה של הבנק, עד {}.",
                shekels(ctx.monthly),
                shekels(target)
            ),
        }));
    }
    let buy = both(|lang| match lang {
        Lang::En => format!("Buy: {}.", money(ctx, lang)),
        Lang::He => format!("קונים: {}.", money(ctx, lang)),
    });
    let open_account = fixed(
        "Open a trading account, if you don't have one.",
        "פותחים חשבון מסחר, אם אין לכם.",
    );
    match ctx.kind {
        Kind::MoneyMarketFund => {
            steps.push(fixed(
                "In the bank's app, under mutual funds, search for ״כספית שקלית״.",
                "באפליקציה של הבנק, תחת קרנות נאמנות, מחפשים ״כספית שקלית״.",
            ));
            steps.push(buy);
            steps.push(fixed(
                "Sell when you need it, any trading day. Tax: 25% of the gain above inflation.",
                "מוכרים כשצריך, בכל יום מסחר. מס: 25% מהרווח שמעבר לאינפלציה.",
            ));
        }
        Kind::GovernmentBond { .. } => {
            steps.push(open_account);
            steps.push(fixed(
                "Search for the series repaid nearest your date, and buy it.",
                "מחפשים את הסדרה שנפדית הכי קרוב לתאריך שלכם, וקונים אותה.",
            ));
            steps.push(buy);
            steps.push(fixed(
                "Hold it until it's repaid. Tax: 25% of the gain above inflation.",
                "מחזיקים עד הפדיון. מס: 25% מהרווח שמעבר לאינפלציה.",
            ));
        }
        Kind::FixedDeposit | Kind::LinkedDeposit => {
            steps.push(fixed(
                "Open it in the app or at a branch. The rate in the figures is an average, not an offer: ask what you'll get.",
                "פותחים באפליקציה או בסניף. הריבית בחישוב היא ממוצע, לא הצעה: שאלו מה תקבלו.",
            ));
            steps.push(buy);
            steps.push(fixed(
                "Don't touch it: breaking it early is by the deposit's terms. Tax: 15% of all the interest.",
                "לא נוגעים: שבירה מוקדמת היא לפי תנאי הפיקדון. מס: 15% מכל הריבית.",
            ));
        }
        Kind::CurrentAccount => {}
        Kind::Way(way) => {
            if way.vehicle == Vehicle::Brokerage {
                if ctx.account == Some(Account::YourBank) {
                    steps.push(fixed(
                        "In your bank's securities account, buy the fund.",
                        "בחשבון ניירות הערך בבנק שלכם, קונים את הקרן.",
                    ));
                } else {
                    steps.push(open_account);
                }
                steps.push(buy);
                steps.push(fixed(
                    "Don't touch it until you need the money. If the index is down then, wait.",
                    "לא נוגעים עד שצריך את הכסף. אם המדד בירידה אז, מחכים.",
                ));
                steps.push(fixed(
                    "Tax: 25% of the gain when you sell.",
                    "מס: 25% מהרווח כשמוכרים.",
                ));
            } else {
                steps.push(fixed(
                    "Check with the fund (or your employer) that you can add this amount.",
                    "בודקים מול הקרן, או מול המעסיק, שאפשר להוסיף את הסכום הזה.",
                ));
                steps.push(fixed(
                    "Choose the track that follows the S&P 500 index.",
                    "בוחרים מסלול שעוקב אחרי מדד S&P\u{a0}500.",
                ));
                steps.push(buy);
            }
        }
    }
    steps
}
