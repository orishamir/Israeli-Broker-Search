//! How to do what was chosen, step by step, in the words people meet in the
//! bank's app. Every rule stated here rests on a page in
//! `policies/sources.md`; what could not be sourced is left out (how a bank
//! charges for breaking a deposit, whether tax is withheld for you, when a
//! sale's money arrives).

use std::sync::OnceLock;

use rust_decimal::Decimal;

use super::answer::also_possible_line;
use super::reasons::{shekels, span};
use crate::guide::{Circumstance, Kind, Recommendation, Step};
use crate::short_term::deposits;
use crate::{Broker, BrokerKind, Exchange, Lang, Named, Product, Text, Vehicle};

/// The tariffs and the funds, built once: a plan's own description says who
/// can join it and on what terms (an offer's condition).
fn listed() -> &'static [Broker] {
    static LISTED: OnceLock<Vec<Broker>> = OnceLock::new();
    LISTED.get_or_init(crate::listed)
}

fn same(a: &Text, b: &Text) -> bool {
    a.en == b.en || a.he == b.he
}

/// What the plan the answer names is, in its own words.
fn plan_description(provider: &Text, plan: &Text, lang: Lang) -> Option<String> {
    let broker = listed()
        .iter()
        .find(|broker| same(&broker.name, provider) || same(&broker.short_name, provider))?;
    // A plan with a price option chosen at opening is named "Plan · option".
    let found = broker
        .plans
        .iter()
        .find(|candidate| plan.en.starts_with(&*candidate.name.en))?;
    Some(found.description[lang].to_string())
}

/// The cheapest company of a kind of fund, in the words of its plan.
fn cheapest_company(vehicle: Vehicle, lang: Lang) -> Option<String> {
    let broker = listed().iter().find(|broker| {
        broker.kind == BrokerKind::Funds
            && broker
                .plans
                .first()
                .is_some_and(|plan| plan.vehicle == vehicle)
    })?;
    let at = broker.spread?.cheapest?;
    Some(broker.plans.get(at)?.description[lang].to_string())
}

fn month_year() -> String {
    let date = deposits::checked();
    format!("{:02}/{}", date.month() as u8, date.year())
}

/// A step from its two languages.
struct Writer {
    lang: Lang,
    steps: Vec<Step>,
}

impl Writer {
    fn add(&mut self, title: (&str, &str), text: (String, String)) {
        let lang = self.lang;
        self.steps.push(Step {
            title: lang.pick(title.0, title.1).to_owned(),
            text: match lang {
                Lang::En => text.0,
                Lang::He => text.1,
            },
        });
    }
}

/// How to do what was chosen, step by step.
#[must_use]
#[allow(clippy::too_many_lines, reason = "one arm for each kind")]
pub fn steps_for(recommendation: &Recommendation, lang: Lang) -> Vec<Step> {
    let rec = recommendation;
    let monthly = rec.answers.monthly > Decimal::ZERO;
    let waits = rec
        .reasons
        .iter()
        .any(|reason| matches!(reason.from, Circumstance::IfItsDownYouWait { .. }));
    let provider = &rec.choice.provider;
    let plan = &rec.choice.plan;
    let named = |text: &Text| text[lang].to_string();
    let offer = plan_description(provider, plan, lang).unwrap_or_default();
    let offer = offer.trim().to_owned();
    let mut w = Writer {
        lang,
        steps: Vec::new(),
    };
    match rec.kind {
        Kind::Way(way) => match way.vehicle {
            Vehicle::Brokerage => {
                w.add(
                    ("Open a trading account", "פותחים חשבון מסחר"),
                    (
                        format!(
                            "Open it with {}, on the plan {}. {offer}",
                            named(provider),
                            named(plan)
                        ),
                        format!(
                            "פותחים ב{}, במסלול {}. {offer}",
                            named(provider),
                            named(plan)
                        ),
                    ),
                );
                let what = match way.product {
                    Product::IsraeliIndexFund => (
                        "Buy an index fund (קרן מחקה) that follows a broad stock index, listed \
                         in Tel Aviv. It is a kind of mutual fund; search for it by the index's \
                         name or by its security number on the exchange."
                            .to_owned(),
                        "קונים קרן מחקה על מדד מניות רחב בבורסה בתל אביב. היא סוג של קרן נאמנות; מחפשים אותה לפי שם המדד או לפי מספר הנייר שלה בבורסה."
                            .to_owned(),
                    ),
                    Product::IsraeliEtf => (
                        "Buy an ETF (קרן סל) that follows a broad stock index and trades in Tel Aviv. \
                         It trades all day like a share."
                            .to_owned(),
                        "קונים קרן סל על מדד מניות רחב שנסחרת בבורסה בתל אביב. היא נסחרת כל היום כמו מניה."
                            .to_owned(),
                    ),
                    Product::UsFund => (
                        "Buy an ETF (קרן סל) on the US exchange that follows a broad stock index. \
                         It is bought in dollars, so the money is converted, which costs \
                         a little (the table shows how much)."
                            .to_owned(),
                        "קונים קרן סל בבורסה בארה״ב שעוקבת אחרי מדד מניות רחב. היא נקנית בדולרים, ולכן הכסף מומר, וזה עולה קצת (הטבלה מראה כמה)."
                            .to_owned(),
                    ),
                    _ => (
                        "Buy an ETF (קרן סל) on a European exchange that follows a broad stock \
                         index. It is bought in dollars or euros, so the money is \
                         converted, which costs a little (the table shows how much)."
                            .to_owned(),
                        "קונים קרן סל בבורסה באירופה שעוקבת אחרי מדד מניות רחב. היא נקנית במטבע חוץ, ולכן הכסף מומר, וזה עולה קצת (הטבלה מראה כמה)."
                            .to_owned(),
                    ),
                };
                w.add(("Buy", "קונים"), what);
                if monthly {
                    w.add(
                        ("Buy every month", "קונים בכל חודש"),
                        (
                            "Set a standing order (הוראת קבע) to buy the same amount each \
                             month. The calculator counts your monthly purchases that way; at \
                             some plans a standing order is cheaper than an ordinary order."
                                .to_owned(),
                            "מגדירים הוראת קבע לקנייה של אותו סכום בכל חודש. המחשבון סופר את הקניות החודשיות שלכם כך; בחלק מהמסלולים הוראת קבע זולה מהוראה רגילה."
                                .to_owned(),
                        ),
                    );
                }
                w.add(
                    ("Hold and sell", "מחזיקים ומוכרים"),
                    if waits {
                        (
                            "Leave it alone until the date. If the market is down then, wait \
                             a year or two before selling."
                                .to_owned(),
                            "לא נוגעים עד התאריך. אם הבורסה תהיה אז בירידה, מחכים שנה–שנתיים לפני שמוכרים."
                                .to_owned(),
                        )
                    } else {
                        (
                            "Leave it alone, and sell only when you need the money."
                                .to_owned(),
                            "לא נוגעים, ומוכרים רק כשצריך את הכסף.".to_owned(),
                        )
                    },
                );
                w.add(
                    ("Tax", "מס"),
                    if way.exchange == Some(Exchange::Tlv) {
                        (
                            "The real gain (above the rise in prices) pays 25% when you sell."
                                .to_owned(),
                            "הרווח הריאלי (מעבר לעליית המחירים) משלם 25% במכירה.".to_owned(),
                        )
                    } else {
                        (
                            "The gain pays 25% when you sell. For a fund bought abroad the law \
                             measures the gain against the exchange rate, not against prices."
                                .to_owned(),
                            "הרווח משלם 25% במכירה. בקרן שנקנית בחו״ל החוק מודד את הרווח מול שער החליפין ולא מול המחירים."
                                .to_owned(),
                        )
                    },
                );
            }
            Vehicle::InvestmentGemel => {
                let company = cheapest_company(Vehicle::InvestmentGemel, lang).unwrap_or_default();
                w.add(
                    (
                        "Open a provident fund for investment",
                        "פותחים קופת גמל להשקעה",
                    ),
                    (
                        format!(
                            "Usually you can open one online, with the company you choose. Fees \
                             differ from company to company. {company}"
                        ),
                        format!(
                            "בדרך כלל אפשר לפתוח אונליין, בחברה שבוחרים. דמי הניהול שונים בין חברה לחברה. {company}"
                        ),
                    ),
                );
                w.add(
                    ("Say what it is", "מה אומרים"),
                    (
                        "On the companies' sites and in ads it is called \"provident fund for \
                         investment\" (גמל להשקעה). It is not the provident fund for savings \
                         (גמל לחיסכון), which is a different product."
                            .to_owned(),
                        "באתרי החברות ובפרסומות זה נקרא ״גמל להשקעה״. זה לא גמל לחיסכון, שהוא מוצר אחר."
                            .to_owned(),
                    ),
                );
                w.add(
                    ("Choose the track", "בוחרים מסלול"),
                    (
                        "Choose the investment track (מסלול השקעה) that follows a broad stock index \
                         (מסלול עוקב מדד)."
                            .to_owned(),
                        "בוחרים מסלול השקעה שעוקב אחרי מדד מניות רחב (מסלול עוקב מדד).".to_owned(),
                    ),
                );
                if monthly {
                    w.add(
                        ("Deposit every month", "מפקידים בכל חודש"),
                        (
                            "Set a standing order (הוראת קבע) for the monthly amount, under \
                             the ceiling for deposits in a year."
                                .to_owned(),
                            "מגדירים הוראת קבע לסכום החודשי, מתחת לתקרת ההפקדה בשנה.".to_owned(),
                        ),
                    );
                }
                let early = rec
                    .reasons
                    .iter()
                    .any(|reason| matches!(reason.from, Circumstance::PensionNeedsSixty { .. }));
                w.add(
                    ("At 60", "בגיל 60"),
                    if early {
                        (
                            "A pension is tax-free only from 60, and you will not be 60 when you \
                             retire: money taken before then pays 25% of the real gain."
                                .to_owned(),
                            "קצבה פטורה ממס רק מגיל 60, ואתם לא תהיו בני 60 בפרישה: כסף שנמשך לפני כן משלם 25% מהרווח הריאלי."
                                .to_owned(),
                        )
                    } else {
                        (
                            "From 60 you can ask for a monthly pension (קצבה); the gain is then \
                             tax-free. The money moves to a body that pays pensions, which \
                             sets the monthly amount."
                                .to_owned(),
                            "מגיל 60 אפשר לבקש קצבה חודשית, ואז הרווח פטור ממס. הכסף עובר לגוף שמשלם קצבאות, והוא קובע את הסכום החודשי."
                                .to_owned(),
                        )
                    },
                );
                w.add(
                    ("Taking it out early", "משיכה מוקדמת"),
                    (
                        "You can take the money out at any time, in one sum (משיכה בבת אחת); \
                         the real gain then pays 25%."
                            .to_owned(),
                        "אפשר למשוך את הכסף בכל עת, בבת אחת, ואז הרווח הריאלי משלם 25%.".to_owned(),
                    ),
                );
            }
            Vehicle::StudyFund => {
                w.add(
                    ("Who can open one", "מי יכול לפתוח"),
                    (
                        "A self-employed saver can open a study fund (קרן השתלמות) with any \
                         company. An employee needs the employer to open it and deposit, \
                         usually 7.5% of the salary from the employer and 2.5% from the employee."
                            .to_owned(),
                        "עצמאי יכול לפתוח קרן השתלמות בכל חברה. שכיר צריך שהמעסיק יפתח אותה ויפקיד, בדרך כלל 7.5% מהשכר מהמעסיק ו\u{2011}2.5% מהעובד."
                            .to_owned(),
                    ),
                );
                w.add(
                    ("Choose the track", "בוחרים מסלול"),
                    (
                        "Choose the track that follows a broad stock index (מסלול עוקב מדד)."
                            .to_owned(),
                        "בוחרים את המסלול שעוקב אחרי מדד מניות רחב (מסלול עוקב מדד).".to_owned(),
                    ),
                );
                w.add(
                    ("Tax-free", "בלי מס"),
                    (
                        "The gain is tax-free if the year's deposits are within the ceiling \
                         for a tax-free gain and the money is taken out after 6 years."
                            .to_owned(),
                        "הרווח פטור ממס אם ההפקדות בשנה בתוך התקרה לרווח פטור והכסף נמשך אחרי 6 שנים."
                            .to_owned(),
                    ),
                );
                w.add(
                    ("Taking it out early", "משיכה מוקדמת"),
                    (
                        "Taken out sooner, the gain is taxed as income, and not at 25%.".to_owned(),
                        "משיכה מוקדמת יותר מחויבת במס כהכנסה, ולא ב\u{2011}25%.".to_owned(),
                    ),
                );
            }
            Vehicle::SavingsPolicy => {
                w.add(
                    ("Open a savings policy", "פותחים פוליסת חיסכון"),
                    (
                        "A savings policy (פוליסת חיסכון) is opened with an insurance company. \
                         There is no ceiling on deposits."
                            .to_owned(),
                        "פוליסת חיסכון נפתחת בחברת ביטוח. אין תקרה להפקדות.".to_owned(),
                    ),
                );
                w.add(
                    ("Choose the track", "בוחרים מסלול"),
                    (
                        "Choose the track that follows a broad stock index (מסלול עוקב מדד)."
                            .to_owned(),
                        "בוחרים את המסלול שעוקב אחרי מדד מניות רחב (מסלול עוקב מדד).".to_owned(),
                    ),
                );
                w.add(
                    ("Tax, and moving", "מס והעברה"),
                    (
                        "The real gain pays 25% when you take the money out, and there is no \
                         tax-free pension. Moving to another insurer counts as a withdrawal, \
                         and is taxed."
                            .to_owned(),
                        "הרווח הריאלי משלם 25% במשיכה, ואין קצבה פטורה ממס. מעבר לחברת ביטוח אחרת נחשב למשיכה, וחייב במס."
                            .to_owned(),
                    ),
                );
            }
        },
        Kind::MoneyMarketFund => {
            w.add(
                ("Find the fund", "מוצאים את הקרן"),
                (
                    "In the app of your bank or investment house, look under mutual funds for a \
                     shekel money market fund (קרן כספית שקלית). Each fund has a security \
                     number on the exchange, and its page on Maya shows its fees."
                        .to_owned(),
                    "באפליקציה של הבנק או בית ההשקעות מחפשים, תחת קרנות נאמנות, קרן כספית שקלית. לכל קרן יש מספר נייר בבורסה, ובדף שלה ב\u{2011}Maya מופיעים דמי הניהול שלה."
                        .to_owned(),
                ),
            );
            w.add(
                ("Check the price", "בודקים מחיר"),
                (
                    "A bank may not charge custody on a money market fund (a Bank of Israel \
                     rule), and Leumi's price list has no trade fee for mutual funds. An \
                     investment house may charge a fee for each purchase: check its price list \
                     first. The calculator does not count it."
                        .to_owned(),
                    "בנק לא רשאי לגבות דמי משמרת על קרן כספית (כלל של בנק ישראל), ובמחירון של לאומי אין עמלת קנייה לקרנות נאמנות. בית השקעות יכול לגבות עמלה על כל קנייה: בדקו במחירון שלו לפני. המחשבון לא סופר אותה."
                        .to_owned(),
                ),
            );
            if monthly {
                w.add(
                    ("Add every month", "מוסיפים בכל חודש"),
                    (
                        "Buy more units each month with the money that comes in.".to_owned(),
                        "קונים עוד יחידות בכל חודש בכסף שנכנס.".to_owned(),
                    ),
                );
            }
            w.add(
                ("Sell when you need it", "מוכרים כשצריך"),
                (
                    "It can be sold on any trading day. The real gain pays 25% when you sell."
                        .to_owned(),
                    "אפשר למכור אותה בכל יום מסחר. הרווח הריאלי משלם 25% במכירה.".to_owned(),
                ),
            );
        }
        Kind::FixedDeposit | Kind::LinkedDeposit => {
            let linked = rec.kind == Kind::LinkedDeposit;
            let dated = month_year();
            w.add(
                ("Open the deposit", "פותחים פיקדון"),
                (
                    format!(
                        "Open it in the bank's app or branch. The rate shown is the average the \
                         bank gave households for this term ({dated}), not an offer: ask for \
                         the rate you will get. Only Bank of Jerusalem and Mizrahi-Tefahot open \
                         a deposit for someone with no current account with them (Calcalist, \
                         November 2025)."
                    ),
                    format!(
                        "פותחים באפליקציה או בסניף של הבנק. הריבית שמוצגת היא הממוצע שהבנק נתן ללקוחות בתקופה הזאת ({dated}), לא הצעה: שאלו מה הריבית שתקבלו. רק בנק ירושלים ומזרחי\u{2011}טפחות פותחים פיקדון למי שאין לו חשבון עו״ש אצלם (כלכליסט, נובמבר 2025)."
                    ),
                ),
            );
            w.add(
                ("Choose the term", "בוחרים תקופה"),
                (
                    format!(
                        "Choose a term that ends on your date: {}. A fixed rate means the \
                         rate does not change on the way.",
                        span(rec.months, lang)
                    ),
                    format!(
                        "בוחרים תקופה שמסתיימת בתאריך שלכם: {}. ריבית קבועה אומרת שהיא לא משתנה בדרך.",
                        span(rec.months, lang)
                    ),
                ),
            );
            w.add(
                ("Leave it", "לא נוגעים"),
                (
                    "The money is locked for the term. Breaking the deposit early is on the \
                     deposit's own terms: read them before you sign."
                        .to_owned(),
                    "הכסף נעול לתקופה. שבירה מוקדמת היא לפי התנאים של הפיקדון עצמו: קראו אותם לפני שחותמים."
                        .to_owned(),
                ),
            );
            w.add(
                ("Tax", "מס"),
                if linked {
                    (
                        "The real gain on a linked deposit pays 25%.".to_owned(),
                        "הרווח הריאלי בפיקדון צמוד מדד משלם 25%.".to_owned(),
                    )
                } else {
                    (
                        "The interest pays 15%, all of it, whatever the inflation was.".to_owned(),
                        "הריבית משלמת 15%, כולה, בלי קשר לאינפלציה.".to_owned(),
                    )
                },
            );
        }
        Kind::GovernmentBond { linked } => {
            w.add(
                ("Open a trading account", "פותחים חשבון מסחר"),
                (
                    format!(
                        "Open it with {}, on the plan {}. {offer}",
                        named(provider),
                        named(plan)
                    ),
                    format!(
                        "פותחים ב{}, במסלול {}. {offer}",
                        named(provider),
                        named(plan)
                    ),
                ),
            );
            w.add(
                ("Buy", "קונים"),
                if linked {
                    (
                        "Buy a government bond linked to the index that is repaid when your \
                         wait ends. A series is named for its kind and the month and year it \
                         is repaid, for example \"ממשלתי צמוד 0536\" (repaid in May 2036): an \
                         example only, so look for a series repaid near your date on the \
                         exchange's list."
                            .to_owned(),
                        "קונים אג״ח ממשלתית צמודה למדד שנפדית כשההמתנה מסתיימת. סדרה נקראת לפי הסוג והחודש והשנה של הפדיון, למשל ״ממשלתי צמוד 0536״ (נפדית במאי 2036): רק דוגמה, אז מחפשים ברשימה של הבורסה סדרה שנפדית קרוב לתאריך שלכם."
                            .to_owned(),
                    )
                } else {
                    (
                        "Buy a shekel government bond that is repaid when your wait ends. A \
                         series is named for its kind and the month and year it is repaid, for \
                         example \"ממשלתי שקלי 0347\" (repaid in March 2047): an example only, \
                         so look for a series repaid near your date on the exchange's list."
                            .to_owned(),
                        "קונים אג״ח ממשלתית שקלית שנפדית כשההמתנה מסתיימת. סדרה נקראת לפי הסוג והחודש והשנה של הפדיון, למשל ״ממשלתי שקלי 0347״ (נפדית במרץ 2047): רק דוגמה, אז מחפשים ברשימה של הבורסה סדרה שנפדית קרוב לתאריך שלכם."
                            .to_owned(),
                    )
                },
            );
            if monthly {
                w.add(
                    ("Every month", "בכל חודש"),
                    (
                        "Each month's money buys a bond repaid at the same end date.".to_owned(),
                        "הכסף של כל חודש קונה אג״ח שנפדית באותו תאריך סיום.".to_owned(),
                    ),
                );
            }
            w.add(
                ("Hold to the repayment", "מחזיקים עד הפדיון"),
                (
                    "Hold it until it is repaid. You can sell earlier, at the day's price, \
                     which may be above or below what you paid. The calculator takes the \
                     repayment as free; several price lists charge the trade fee for a \
                     repayment too, so check yours."
                        .to_owned(),
                    "מחזיקים עד שהיא נפדית. אפשר למכור קודם, במחיר של אותו יום, שיכול להיות מעל או מתחת למה ששילמתם. המחשבון מניח שהפדיון חינם; בכמה מחירונים גובים עליו את עמלת המסחר, אז בדקו במחירון שלכם."
                        .to_owned(),
                ),
            );
            w.add(
                ("Tax", "מס"),
                if linked {
                    (
                        "A linked bond pays 25% of the real gain.".to_owned(),
                        "אג״ח צמודה משלמת 25% מהרווח הריאלי.".to_owned(),
                    )
                } else {
                    (
                        "An unlinked bond pays 15% of the whole gain.".to_owned(),
                        "אג״ח לא צמודה משלמת 15% מכל הרווח.".to_owned(),
                    )
                },
            );
        }
        Kind::CurrentAccount => {
            w.add(
                ("Nothing to do", "אין מה לעשות"),
                (
                    "The money stays where it is. It earns nothing, and prices eat into it."
                        .to_owned(),
                    "הכסף נשאר איפה שהוא. הוא לא מרוויח כלום, והמחירים אוכלים אותו.".to_owned(),
                ),
            );
        }
    }
    if let Some(plan) = &rec.two_parts {
        let then = plan.then.name(lang);
        w.add(
            ("When the cushion is full", "כשהכרית מלאה"),
            (
                format!(
                    "At {} (after about {}), stop adding to the cushion, keep it where it is, \
                     and open {then} for the monthly money.",
                    shekels(plan.target),
                    span(plan.months_to_fill, lang)
                ),
                format!(
                    "כשיש {} (אחרי בערך {}) מפסיקים להוסיף לכרית, משאירים אותה במקומה, ופותחים {then} לכסף החודשי.",
                    shekels(plan.target),
                    span(plan.months_to_fill, lang)
                ),
            ),
        );
    }
    // A plan that leaves more but may cost more is told here too, with how
    // sure its price is, so it is not found out only after opening.
    if let Some(also) = &rec.also_possible {
        let en = also_possible_line(also, Lang::En);
        let he = also_possible_line(also, Lang::He);
        w.add(
            ("If you prefer the other plan", "אם מעדיפים את המסלול האחר"),
            (
                format!(
                    "{en} Before opening, you can ask them whether that condition applies to you."
                ),
                format!("{he} לפני שפותחים אפשר לשאול אותם אם התנאי הזה חל עליכם."),
            ),
        );
    }
    w.steps
}
