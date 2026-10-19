//! "איך עושים את זה" (REDESIGN.md §2.8, §2.9): the identity line of each
//! product, what to type in the app, the steps, and the words of where to
//! buy that the places themselves don't say.
//!
//! The identity line is always in one order: what it is (and who issues
//! it) · where it trades · its currency · what it's linked to · when the
//! money comes out. What to search for is its own slot, under the line.
//! The names and numbers in it are `examples.rs`' (fetched, never typed):
//! a bond's series is the one its date decides, a fund's name is an example
//! to search for and never a pick (principle 17), and where no series fits,
//! the slot is a marked placeholder that says why.

#![allow(
    clippy::too_many_lines,
    clippy::format_push_string,
    reason = "the words of every product"
)]

use rust_decimal::Decimal;
use time::Date;

use super::card::{about, both, end_date};
use super::reasons::{month_year, shekels, span};
use crate::examples::{self, Example, FindBy, MoneyMarketFund};
use crate::guide::{Account, AlsoPossible, Flexibility, Kind, UnlockLine};
use crate::short_term::government_bonds::Linkage;
use crate::{Exchange, Lang, Page, Product, Text, Vehicle};

/// What the words need.
#[derive(Debug, Clone)]
pub struct Context {
    pub kind: Kind,
    pub now: Decimal,
    pub monthly: Decimal,
    pub months: u32,
    pub today: Option<Date>,
    /// The visitor gave a date (not "I don't know", not retirement): "until
    /// 2031" is theirs to read only then.
    pub dated: bool,
    /// Whether the date can move, if the visitor said.
    pub flexibility: Option<Flexibility>,
    /// A cushion to fill first: its target and months.
    pub cushion: Option<(Decimal, u32)>,
    pub account: Option<Account>,
    /// The visitor's bank, once named: nothing is "yours" before.
    pub bank: Option<Text>,
    /// A portfolio already at a bank, moved as it is.
    pub existing: bool,
}

fn fixed(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// The date the money is needed, when the visitor gave one.
fn end(ctx: &Context) -> Option<Date> {
    ctx.today
        .filter(|_| ctx.dated)
        .and_then(|today| end_date(today, ctx.months))
}

/// "2031", or "in 5 years".
fn year(ctx: &Context, lang: Lang) -> String {
    match end(ctx) {
        Some(end) => end.year().to_string(),
        None => format!("{} {}", lang.pick("in", "בעוד"), span(ctx.months, lang)),
    }
}

/// "ב‑2031" / "in 2031", or "בעוד 5 שנים".
fn in_year(ctx: &Context, lang: Lang) -> String {
    match end(ctx) {
        Some(end) => format!("{}{}", lang.pick("in ", "ב\u{2011}"), end.year()),
        None => year(ctx, lang),
    }
}

/// What an Israeli fund does with the S&P 500: it follows it, often through
/// contracts rather than the shares themselves.
fn follows(what: &'static str, he_what: &'static str) -> Text {
    both(|lang| match lang {
        Lang::En => format!("{what} that follows the S&P 500 index (the 500 largest US companies)"),
        Lang::He => {
            format!("{he_what} שעוקבת אחרי מדד S&P\u{a0}500 (500 החברות הגדולות בארה״ב)")
        }
    })
}

/// The S&P 500 as the identity line says what a fund holds.
fn holds(what: &'static str, he_what: &'static str) -> Text {
    both(|lang| match lang {
        Lang::En => format!("{what} that holds the 500 largest US companies (the S&P 500 index)"),
        Lang::He => {
            format!("{he_what} שמחזיקה את 500 החברות הגדולות בארה״ב (מדד S&P\u{a0}500)")
        }
    })
}

fn any_trading_day() -> Text {
    fixed("Sold any trading day", "נמכרת בכל יום מסחר")
}

/// A fund company's track: the wrapper, then what it holds.
fn track(wrapper_en: &'static str, wrapper_he: &'static str) -> Text {
    both(|lang| match lang {
        Lang::En => format!("{wrapper_en}, on the track that follows the S&P 500 index"),
        Lang::He => format!("{wrapper_he}, במסלול שעוקב אחרי מדד S&P\u{a0}500"),
    })
}

/// The identity line (§2.8), its parts in order.
#[must_use]
pub fn identity(ctx: &Context) -> Vec<Text> {
    match ctx.kind {
        Kind::MoneyMarketFund => vec![
            fixed(
                "A mutual fund that lends short-term to the state and the banks",
                "קרן נאמנות שמלווה כסף לזמן קצר למדינה ולבנקים",
            ),
            fixed(
                "Bought in a bank's app, or at an investment house",
                "נקנית באפליקציה של בנק, או בבית השקעות",
            ),
            fixed("In shekels", "בשקלים"),
            fixed(
                "Its rate moves with the Bank of Israel's",
                "הריבית זזה עם ריבית בנק ישראל",
            ),
            any_trading_day(),
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
                fixed(
                    "In shekels, not linked: the sum is known in advance",
                    "בשקלים, לא צמודה: הסכום ידוע מראש",
                )
            },
            both(|lang| match lang {
                Lang::En => format!("Repaid {}: held until then", in_year(ctx, lang)),
                Lang::He => format!("נפדית {}: מחזיקים עד אז", in_year(ctx, lang)),
            }),
        ],
        Kind::FixedDeposit | Kind::LinkedDeposit => vec![
            fixed("A deposit at a bank", "פיקדון בבנק"),
            fixed(
                "Opened in the bank's app or at a branch",
                "נפתח באפליקציה של הבנק או בסניף",
            ),
            if ctx.kind == Kind::LinkedDeposit {
                fixed(
                    "In shekels, linked to the consumer price index, at a fixed rate above it",
                    "בשקלים, צמוד למדד המחירים לצרכן, בריבית קבועה מעליו",
                )
            } else {
                fixed(
                    "In shekels, at a fixed rate: the sum is known in advance",
                    "בשקלים, בריבית קבועה: הסכום ידוע מראש",
                )
            },
            both(|lang| match (end(ctx), lang) {
                (Some(end), Lang::En) => format!("Locked until {}", end.year()),
                (Some(end), Lang::He) => format!("הכסף נעול עד {}", end.year()),
                (None, Lang::En) => format!("Locked for {}", span(ctx.months, lang)),
                (None, Lang::He) => format!("הכסף נעול ל\u{2011}{}", span(ctx.months, lang)),
            }),
        ],
        Kind::CurrentAccount => vec![],
        Kind::Way(way) => match (way.vehicle, way.product) {
            (Vehicle::StudyFund, _) => vec![
                track("A study fund", "קרן השתלמות"),
                fixed(
                    "At a fund company: for the self-employed, or through an employer",
                    "בחברה מנהלת: לעצמאים, או דרך מעסיק",
                ),
                fixed("In shekels", "בשקלים"),
                fixed(
                    "Can be taken out after 6 years, and the gain is then tax-free",
                    "אפשר למשוך אחרי 6 שנים, והרווח אז פטור ממס",
                ),
            ],
            (Vehicle::InvestmentGemel, _) => vec![
                track("A provident fund for investment", "קופת גמל להשקעה"),
                fixed(
                    "At a fund company: anyone can open one, up to a yearly ceiling",
                    "בחברה מנהלת: כל אחד יכול לפתוח, עד תקרה שנתית",
                ),
                fixed("In shekels", "בשקלים"),
                fixed(
                    "Can be taken out any time; as a pension from 60, the gain is tax-free",
                    "אפשר למשוך בכל עת; כקצבה מגיל 60 הרווח פטור ממס",
                ),
            ],
            (Vehicle::SavingsPolicy, _) => vec![
                track("A savings policy", "פוליסת חיסכון"),
                fixed(
                    "At an insurance company: anyone can open one",
                    "בחברת ביטוח: כל אחד יכול לפתוח",
                ),
                fixed("In shekels", "בשקלים"),
                fixed("Can be taken out any time", "אפשר למשוך בכל עת"),
            ],
            (
                Vehicle::Brokerage,
                Product::IsraeliIndexFund
                | Product::StudyFundTrack
                | Product::InvestmentGemelTrack
                | Product::SavingsPolicyTrack,
            ) => vec![
                follows(
                    "An Israeli mutual fund (קרן מחקה)",
                    "קרן נאמנות ישראלית (קרן מחקה)",
                ),
                fixed(
                    "Bought and sold through a bank or an investment house once a day, at the day's closing price",
                    "נקנית ונמכרת דרך בנק או בית השקעות פעם ביום, במחיר של סוף היום",
                ),
                fixed(
                    "In shekels; moves with the shares and with the dollar",
                    "בשקלים; עולה ויורדת עם המניות ועם הדולר",
                ),
                fixed("Sold any trading day", "אפשר למכור בכל יום מסחר"),
            ],
            (Vehicle::Brokerage, Product::IsraeliEtf) => vec![
                follows("An Israeli ETF (קרן סל)", "קרן סל ישראלית"),
                fixed(
                    "Traded on the Tel Aviv Stock Exchange",
                    "נסחרת בבורסה בתל אביב",
                ),
                fixed(
                    "In shekels; moves with the shares and with the dollar",
                    "בשקלים; עולה ויורדת עם המניות ועם הדולר",
                ),
                any_trading_day(),
            ],
            (Vehicle::Brokerage, Product::ForeignEtfInTelAviv) => vec![
                holds("An Irish ETF", "קרן סל אירית"),
                fixed(
                    "Also traded on the Tel Aviv Stock Exchange",
                    "נסחרת גם בבורסה בתל אביב",
                ),
                fixed(
                    "Bought in shekels; moves with the shares and with the dollar",
                    "נקנית בשקלים; עולה ויורדת עם המניות ועם הדולר",
                ),
                any_trading_day(),
            ],
            (Vehicle::Brokerage, Product::UsFund) => vec![
                holds("A US ETF", "קרן סל אמריקאית"),
                fixed("Traded on a US exchange", "נסחרת בבורסה בארה״ב"),
                fixed(
                    "In dollars: the shekels are converted",
                    "בדולרים: השקלים מומרים",
                ),
                any_trading_day(),
            ],
            (Vehicle::Brokerage, Product::IrishFund) => vec![
                holds("An Irish ETF", "קרן סל אירית"),
                if way.exchange == Some(Exchange::Europe) {
                    fixed("Traded on a European exchange", "נסחרת בבורסה באירופה")
                } else {
                    fixed("Traded abroad", "נסחרת בחו״ל")
                },
                fixed(
                    "In a foreign currency: the shekels are converted",
                    "במטבע חוץ: השקלים מומרים",
                ),
                any_trading_day(),
            ],
        },
    }
}

/// The marked placeholder of a slot the research hasn't filled (principle
/// 16): the page shows it as such.
const TO_COME: &str = "____";

/// What to type in the app: the text, whether it is still a placeholder,
/// and the pages its names and numbers come from.
#[derive(Debug, Clone)]
pub struct Slot {
    pub text: Text,
    pub placeholder: bool,
    pub sources: Vec<Page>,
}

/// The shekel money market funds that charge least, within the providers'
/// margin of each other for the visitor's money: a tie, whose names the
/// slot gives (principle 13), never "the cheapest".
#[derive(Debug, Clone)]
pub struct FundTie {
    /// Least first.
    pub funds: Vec<&'static MoneyMarketFund>,
    /// What the first leaves above the last of them, in today's ₪.
    pub gap: Decimal,
}

/// "1234567 · ממשל צמודה 0529": the number first, which every app finds,
/// then the name the exchange gives it.
fn numbered(number: impl std::fmt::Display, name: &str, lang: Lang) -> String {
    format!("{number}{}{name}", lang.pick(" · ", "\u{a0}· "))
}

/// The examples of a way, at most two, said once as examples (principle
/// 17): "for example 5124482 · קסם SP500 KTF or 5129275 · …".
fn examples_of(way: crate::ways::Way, lead_en: &str, lead_he: &str) -> Option<Slot> {
    let shown: Vec<&Example> = examples::for_way(way).take(2).collect();
    if shown.is_empty() {
        return None;
    }
    let text = both(|lang| {
        let names: Vec<String> = shown
            .iter()
            .map(|example| match example.find_by {
                FindBy::Number(number) => numbered(number, example.name, lang),
                FindBy::Ticker(ticker) => numbered(ticker, example.name, lang),
            })
            .collect();
        let or = lang.pick(" or ", " או ");
        let lead = lang.pick(lead_en, lead_he);
        let lead = if lead.is_empty() {
            String::new()
        } else {
            format!("{lead}{}", lang.pick(" · ", "\u{a0}· "))
        };
        format!(
            "{lead}{} {} ({})",
            lang.pick("for example", "למשל"),
            names.join(or),
            &examples::NOT_A_PICK[lang]
        )
    });
    Some(Slot {
        text,
        placeholder: false,
        sources: shown.iter().map(|example| example.page()).collect(),
    })
}

/// What to type in the app. A bond's series is the one repaid last by the
/// visitor's date (`examples::nearest_bond`); a fund's are examples; the
/// money market funds' are the ones that charge least, as a tie.
#[must_use]
pub fn search(ctx: &Context, cheapest_funds: Option<&FundTie>) -> Option<Slot> {
    let plain = |text: Text| Slot {
        text,
        placeholder: false,
        sources: Vec::new(),
    };
    match ctx.kind {
        Kind::MoneyMarketFund => Some(money_market(cheapest_funds)),
        Kind::GovernmentBond { linked } => Some(bond(ctx, linked)),
        Kind::FixedDeposit => Some(plain(both(|lang| match lang {
            Lang::En => format!("fixed-rate deposit, {}", span(ctx.months, lang)),
            Lang::He => format!("פיקדון בריבית קבועה, {}", span(ctx.months, lang)),
        }))),
        Kind::LinkedDeposit => Some(plain(both(|lang| match lang {
            Lang::En => format!(
                "CPI-linked deposit (״צמוד מדד״), {}",
                span(ctx.months, lang)
            ),
            Lang::He => format!("״פיקדון צמוד מדד״, {}", span(ctx.months, lang)),
        }))),
        Kind::CurrentAccount => None,
        Kind::Way(way) => match (way.vehicle, way.product) {
            (Vehicle::StudyFund, _) => Some(plain(fixed(
                "the S&P 500 track, in your study fund",
                "מסלול מדד S&P\u{a0}500, בקרן ההשתלמות שלכם",
            ))),
            (Vehicle::InvestmentGemel, _) => Some(plain(fixed(
                "the S&P 500 track, at any fund company that runs a provident fund for investment (their fees are compared on Gemel Net)",
                "מסלול מדד S&P\u{a0}500, בכל חברה מנהלת של קופת גמל להשקעה (את דמי הניהול שלהן משווים בגמל נט)",
            ))),
            (Vehicle::SavingsPolicy, _) => Some(plain(fixed(
                "the S&P 500 track, at an insurance company that sells savings policies",
                "מסלול מדד S&P\u{a0}500, בחברת ביטוח שמוכרת פוליסות חיסכון",
            ))),
            (Vehicle::Brokerage, Product::UsFund | Product::IrishFund) => examples_of(way, "", ""),
            (Vehicle::Brokerage, Product::ForeignEtfInTelAviv) => {
                examples_of(way, "S&P 500, in Tel Aviv", "S&P\u{a0}500, בתל אביב")
            }
            (Vehicle::Brokerage, Product::IsraeliEtf) => examples_of(
                way,
                "S&P 500, among ETFs (קרנות סל)",
                "S&P\u{a0}500, בקרנות הסל",
            ),
            (Vehicle::Brokerage, _) => examples_of(
                way,
                "S&P 500, among index funds (קרנות מחקות)",
                "S&P\u{a0}500, בקרנות המחקות",
            ),
        },
    }
}

/// "כספית שקלית", and the funds that charge least, as a tie.
fn money_market(tie: Option<&FundTie>) -> Slot {
    let kind = fixed("money market fund in shekels (כספית שקלית)", "כספית שקלית");
    let Some(tie) = tie.filter(|tie| !tie.funds.is_empty()) else {
        return Slot {
            text: kind,
            placeholder: false,
            sources: Vec::new(),
        };
    };
    let highest = tie
        .funds
        .iter()
        .map(|fund| fund.fee)
        .max()
        .unwrap_or_default();
    let lowest = tie
        .funds
        .iter()
        .map(|fund| fund.fee)
        .min()
        .unwrap_or_default();
    let all = tie.funds.len() == examples::money_market_funds().len();
    let text = both(|lang| {
        let names: Vec<String> = tie
            .funds
            .iter()
            .take(3)
            .map(|fund| numbered(fund.number, fund.name, lang))
            .collect();
        let shown = names.join(lang.pick(", ", ", "));
        let count = tie.funds.len();
        let rest = count.saturating_sub(names.len());
        let more = match (rest, lang) {
            (0, _) => String::new(),
            (1, Lang::En) => " and one more".to_owned(),
            (1, Lang::He) => " ועוד אחת".to_owned(),
            (_, Lang::En) => format!(" and {rest} more"),
            (_, Lang::He) => format!(" ועוד {rest}"),
        };
        let gap = shekels(tie.gap.max(Decimal::ONE));
        let charge = match (lowest == highest, lang) {
            (true, Lang::En) => format!("charge {highest} a year"),
            (true, Lang::He) => format!("גובות {highest} בשנה"),
            (false, Lang::En) => format!("charge {lowest} to {highest} a year"),
            (false, Lang::He) => format!("גובות בין {lowest} ל\u{2011}{highest} בשנה"),
        };
        match (count, all, lang) {
            (1, _, Lang::En) => format!(
                "{} · the one that charges least: {shown} ({highest} a year)",
                kind.en
            ),
            (1, _, Lang::He) => format!(
                "{}\u{a0}· הקרן שגובה הכי מעט: {shown} ({highest} בשנה)",
                kind.he
            ),
            // Every fund within the margin: nothing to name.
            (_, true, Lang::En) => format!(
                "{} · on your money, all {count} funds end at most {gap} apart: any of them is fine",
                kind.en
            ),
            (_, true, Lang::He) => format!(
                "{}\u{a0}· בכסף שלכם, כל {count} הקרנות מסתיימות בהפרש של עד {gap}: כל אחת מהן בסדר",
                kind.he
            ),
            (_, false, Lang::En) => format!(
                "{} · {count} funds {charge}, at most {gap} apart on your money, a tie: {shown}{more}",
                kind.en
            ),
            (_, false, Lang::He) => format!(
                "{}\u{a0}· {count} קרנות {charge}, וההבדל ביניהן בכסף שלכם עד {gap}, כלומר תיקו: {shown}{more}",
                kind.he
            ),
        }
    });
    Slot {
        text,
        placeholder: false,
        sources: if all {
            vec![crate::short_term::money_market::maya()]
        } else {
            tie.funds.iter().take(3).map(|fund| fund.page()).collect()
        },
    }
}

/// The series repaid last by the visitor's date, or a marked placeholder
/// that says why there is none.
fn bond(ctx: &Context, linked: bool) -> Slot {
    let linkage = if linked {
        Linkage::CpiLinked
    } else {
        Linkage::Unlinked
    };
    let by = ctx
        .today
        .and_then(|today| end_date(today, ctx.months).map(|by| (today, by)));
    let series = by.and_then(|(today, by)| examples::nearest_bond(linkage, today, by));
    if let Some(series) = series {
        // The latest series can be repaid well before a far date: then the
        // money waits from then, and the slot says when.
        let early = by.is_some_and(|(_, by)| (by - series.repaid).whole_days() > 365);
        return Slot {
            text: both(|lang| {
                let found = numbered(series.number, series.name, lang);
                match (early, lang) {
                    (false, _) => found,
                    (true, Lang::En) => format!(
                        "{found} (repaid in {}: no series is repaid nearer your date)",
                        month_year(series.repaid, lang)
                    ),
                    (true, Lang::He) => format!(
                        "{found} (נפדית ב{}: אין סדרה שנפדית קרוב יותר לתאריך שלכם)",
                        month_year(series.repaid, lang)
                    ),
                }
            }),
            placeholder: false,
            sources: vec![series.page()],
        };
    }
    let name = if linked {
        "ממשל צמודה"
    } else {
        "ממשל שקלית"
    };
    let text = both(|lang| match (by, lang) {
        (Some((_, by)), Lang::En) => format!(
            "{name} {TO_COME} (no series is repaid by {}: the nearest is repaid after your date)",
            month_year(by, lang)
        ),
        (Some((_, by)), Lang::He) => format!(
            "{name} {TO_COME} (אין סדרה שנפדית עד {}: הקרובה נפדית אחרי התאריך שלכם)",
            month_year(by, lang)
        ),
        (None, Lang::En) => format!("{name} {TO_COME} (the series repaid nearest your date)"),
        (None, Lang::He) => format!("{name} {TO_COME} (הסדרה שנפדית הכי קרוב לתאריך שלכם)"),
    });
    Slot {
        text,
        placeholder: true,
        sources: Vec::new(),
    }
}
/// "₪100,000 now, and ₪2,000 every month" as the buying steps say it: one
/// step for the first sum, one for the months.
fn buying(ctx: &Context, now: Text, every_month: Text) -> Vec<Text> {
    let mut steps = Vec::new();
    if !ctx.now.is_zero() {
        steps.push(now);
    }
    if !ctx.monthly.is_zero() {
        steps.push(every_month);
    }
    steps
}

/// Where a security is bought (finding 10): the bank's securities
/// account, the visitor's once named; a new account where that's the
/// option; or either, said the same way for every kind.
fn account_step(ctx: &Context) -> Text {
    match (ctx.account, &ctx.bank) {
        (Some(Account::YourBank), Some(bank)) => both(|lang| match lang {
            Lang::En => format!(
                "In {}'s app, in its securities account: if you have none there, it opens in the app.",
                bank.en
            ),
            Lang::He => format!(
                "באפליקציה של {}, בחשבון ניירות הערך: אם אין לכם שם, פותחים אותו באפליקציה.",
                bank.he
            ),
        }),
        (Some(Account::YourBank), None) => fixed(
            "In your bank's app, in its securities account: if you have none there, it opens in the app.",
            "באפליקציה של הבנק, בחשבון ניירות הערך: אם אין לכם שם, פותחים אותו באפליקציה.",
        ),
        (Some(Account::NewAccount), _) => fixed(
            "Open an account at one of the places below, if you have none there: a one-time chore. If you do, it's one more order.",
            "פותחים חשבון באחד המקומות שלמטה, אם אין לכם שם: טרחה של פעם אחת. אם יש, זו עוד פקודה.",
        ),
        (None, _) => fixed(
            "In a securities account, at a bank or an investment house. If you already have one, it's one more order.",
            "בחשבון ניירות ערך, בבנק או בבית השקעות. אם כבר יש לכם אחד, זו עוד פקודה.",
        ),
    }
}

/// The steps of a portfolio moved as it is (finding 2): no sale, no tax.
fn transfer_steps() -> Vec<Text> {
    vec![
        fixed(
            "Open an account at one of the places below: a one-time chore.",
            "פותחים חשבון באחד המקומות שלמטה: טרחה של פעם אחת.",
        ),
        fixed(
            "In the new account, ask to transfer the securities from the bank: they move as they are.",
            "בחשבון החדש מבקשים להעביר אליו את ניירות הערך מהבנק: הם עוברים כמו שהם.",
        ),
        fixed(
            "Don't sell: with no sale there's no tax, and the money stays invested the whole time.",
            "לא מוכרים: בלי מכירה אין מס, והכסף נשאר מושקע כל הזמן.",
        ),
        fixed(
            "Ask the bank whether it charges for transferring securities, and the new place whether it pays that back.",
            "שואלים את הבנק אם הוא גובה עמלה על העברת ניירות ערך, ואת המקום החדש אם הוא מחזיר אותה.",
        ),
    ]
}

/// The money each month or at once, as a fund company's step says it.
fn adds(ctx: &Context, lang: Lang) -> String {
    if ctx.monthly.is_zero() {
        shekels(ctx.now)
    } else {
        match lang {
            Lang::En => format!("{} a month", shekels(ctx.monthly)),
            Lang::He => format!("{} בחודש", shekels(ctx.monthly)),
        }
    }
}

/// The steps of a fund company's way, each kind its own (the review's L14).
fn fund_steps(ctx: &Context, vehicle: Vehicle) -> Vec<Text> {
    let track = fixed(
        "Choose the track that follows the S&P 500 index.",
        "בוחרים את מסלול מדד S&P\u{a0}500.",
    );
    match vehicle {
        Vehicle::InvestmentGemel => vec![
            fixed(
                "Open a provident fund for investment at a fund company, on its site or by phone: anyone can, with no employer.",
                "פותחים קופת גמל להשקעה בחברה מנהלת, באתר שלה או בטלפון: כל אחד יכול, בלי מעסיק.",
            ),
            track,
            both(|lang| match lang {
                Lang::En => format!(
                    "Deposit {}, by standing order, up to the yearly ceiling.",
                    adds(ctx, lang)
                ),
                Lang::He => format!("מפקידים {}, בהוראת קבע, עד התקרה השנתית.", adds(ctx, lang)),
            }),
            fixed(
                "Taking it out: as a monthly pension from 60, the gain is tax-free; all at once, 25% of the gain above inflation.",
                "במשיכה: כקצבה חודשית מגיל 60 הרווח פטור ממס; בבת אחת, 25% מהרווח שמעבר לאינפלציה.",
            ),
        ],
        Vehicle::SavingsPolicy => vec![
            fixed(
                "Open a savings policy at an insurance company: anyone can.",
                "פותחים פוליסת חיסכון בחברת ביטוח: כל אחד יכול.",
            ),
            track,
            both(|lang| match lang {
                Lang::En => format!("Deposit {}.", adds(ctx, lang)),
                Lang::He => format!("מפקידים {}.", adds(ctx, lang)),
            }),
            fixed(
                "Taking it out: 25% of the gain above inflation.",
                "במשיכה: 25% מהרווח שמעבר לאינפלציה.",
            ),
        ],
        _ => vec![
            both(|lang| match lang {
                Lang::En => format!(
                    "Check with the fund (or your employer) that you can add {}.",
                    adds(ctx, lang)
                ),
                Lang::He => format!(
                    "בודקים מול הקרן, או מול המעסיק, שאפשר להוסיף {}.",
                    adds(ctx, lang)
                ),
            }),
            track,
        ],
    }
}

/// The tax when a fund bought on an exchange is sold: against inflation in
/// shekels, against the exchange rate abroad (as the engine counts it).
fn index_tax(product: Product) -> Text {
    match product {
        Product::UsFund | Product::IrishFund => fixed(
            "Tax: 25% of the gain when you sell, measured against the exchange rate.",
            "מס: 25% מהרווח כשמוכרים, נמדד מול שער המטבע.",
        ),
        _ => fixed(
            "Tax: 25% of the gain above inflation, when you sell.",
            "מס: 25% מהרווח שמעבר לאינפלציה, כשמוכרים.",
        ),
    }
}

/// The steps, in order.
#[must_use]
pub fn steps(ctx: &Context) -> Vec<Text> {
    let now = shekels(ctx.now);
    let monthly = shekels(ctx.monthly);
    let mut steps = Vec::new();
    if let (Some((target, _)), Kind::Way(_)) = (ctx.cushion, ctx.kind) {
        steps.push(both(|lang| match lang {
            Lang::En => format!(
                "First: {monthly} a month to a money market fund in the bank's app, until {} is aside; it stays there.",
                shekels(target)
            ),
            Lang::He => format!(
                "קודם: {monthly} בחודש לקרן כספית באפליקציה של הבנק, עד שיש בצד {}; הם נשארים שם.",
                shekels(target)
            ),
        }));
    }
    match ctx.kind {
        Kind::MoneyMarketFund => {
            steps.push(fixed(
                "In the bank's app or at an investment house, under mutual funds, search for ״כספית שקלית״.",
                "באפליקציה של הבנק או בבית השקעות, תחת קרנות נאמנות, מחפשים ״כספית שקלית״.",
            ));
            steps.push(both(|lang| {
                match (ctx.now.is_zero(), ctx.monthly.is_zero(), lang) {
                    (false, false, Lang::En) => {
                        format!("Buy {now} now, and {monthly} more every month.")
                    }
                    (false, false, Lang::He) => {
                        format!("קונים ב\u{2011}{now} עכשיו, ועוד {monthly} בכל חודש.")
                    }
                    (true, _, Lang::En) => format!("Buy {monthly} every month."),
                    (true, _, Lang::He) => format!("קונים ב\u{2011}{monthly} בכל חודש."),
                    (false, true, Lang::En) => format!("Buy {now}."),
                    (false, true, Lang::He) => format!("קונים ב\u{2011}{now}."),
                }
            }));
            steps.push(fixed(
                "Sell when you need it, any trading day. Tax: 25% of the gain above inflation.",
                "מוכרים כשצריך, בכל יום מסחר. מס: 25% מהרווח שמעבר לאינפלציה.",
            ));
        }
        Kind::GovernmentBond { linked } => {
            steps.push(account_step(ctx));
            let repaid = |lang: Lang| in_year(ctx, lang);
            steps.extend(buying(
                ctx,
                both(|lang| match lang {
                    Lang::En => format!(
                        "Search for the series repaid {}, and buy {now} of it.",
                        repaid(lang)
                    ),
                    Lang::He => format!(
                        "מחפשים את הסדרה שנפדית {}, וקונים ב\u{2011}{now}.",
                        repaid(lang)
                    ),
                }),
                both(|lang| match (ctx.now.is_zero(), lang) {
                    (false, Lang::En) => {
                        format!("Every month, buy {monthly} more of the same series.")
                    }
                    (false, Lang::He) => {
                        format!("בכל חודש קונים את אותה סדרה ב\u{2011}{monthly}.")
                    }
                    (true, Lang::En) => format!(
                        "Every month, buy {monthly} of the series repaid {}.",
                        repaid(lang)
                    ),
                    (true, Lang::He) => format!(
                        "בכל חודש קונים ב\u{2011}{monthly} את הסדרה שנפדית {}.",
                        repaid(lang)
                    ),
                }),
            ));
            steps.push(if linked {
                fixed(
                    "Hold it until it's repaid. Tax: 25% of the gain above inflation.",
                    "מחזיקים עד הפדיון. מס: 25% מהרווח שמעבר לאינפלציה.",
                )
            } else {
                fixed(
                    "Hold it until it's repaid. Tax: 15% of the whole gain.",
                    "מחזיקים עד הפדיון. מס: 15% מכל הרווח.",
                )
            });
        }
        Kind::FixedDeposit | Kind::LinkedDeposit => {
            let dated = deposits_month();
            steps.push(both(|lang| match (ctx.kind, lang) {
                (Kind::LinkedDeposit, Lang::En) => format!(
                    "In the bank's app or at a branch, choose a deposit for {}, linked to the consumer price index, and deposit {now}.",
                    span(ctx.months, lang)
                ),
                (Kind::LinkedDeposit, Lang::He) => format!(
                    "באפליקציה של הבנק או בסניף, בוחרים פיקדון ל\u{2011}{} צמוד למדד המחירים לצרכן, ומפקידים {now}.",
                    span(ctx.months, lang)
                ),
                (_, Lang::En) => format!(
                    "In the bank's app or at a branch, choose a deposit for {} at a fixed rate, and deposit {now}.",
                    span(ctx.months, lang)
                ),
                (_, Lang::He) => format!(
                    "באפליקציה של הבנק או בסניף, בוחרים פיקדון ל\u{2011}{} בריבית קבועה, ומפקידים {now}.",
                    span(ctx.months, lang)
                ),
            }));
            steps.push(both(|lang| match lang {
                Lang::En => format!(
                    "The rate in the figures is the average the bank gave in {dated}, not an offer: ask what you'll get before you deposit."
                ),
                Lang::He => format!(
                    "הריבית בחישוב היא הממוצע שהבנק נתן ב\u{2011}{dated}, לא הצעה: שאלו מה תקבלו לפני שמפקידים."
                ),
            }));
            steps.push(both(|lang| match (end(ctx), lang) {
                (Some(end), Lang::En) => format!(
                    "Don't touch it until {}: breaking it early is by the deposit's terms.",
                    end.year()
                ),
                (Some(end), Lang::He) => format!(
                    "לא נוגעים עד {}: שבירה מוקדמת היא לפי תנאי הפיקדון.",
                    end.year()
                ),
                (None, _) => lang
                    .pick(
                        "Don't touch it: breaking it early is by the deposit's terms.",
                        "לא נוגעים: שבירה מוקדמת היא לפי תנאי הפיקדון.",
                    )
                    .to_owned(),
            }));
            steps.push(if ctx.kind == Kind::LinkedDeposit {
                fixed(
                    "Tax: 25% of the gain above inflation.",
                    "מס: 25% מהרווח שמעבר לאינפלציה.",
                )
            } else {
                fixed("Tax: 15% of all the interest.", "מס: 15% מכל הריבית.")
            });
        }
        Kind::CurrentAccount => {}
        Kind::Way(way) if ctx.existing && way.vehicle == Vehicle::Brokerage => {
            steps.extend(transfer_steps());
        }
        Kind::Way(way) => {
            if way.vehicle == Vehicle::Brokerage {
                steps.push(account_step(ctx));
                if matches!(way.product, Product::UsFund | Product::IrishFund) {
                    steps.push(fixed(
                        "Move shekels to the account and convert them; the figures count the conversion.",
                        "מעבירים שקלים לחשבון וממירים אותם; החישוב כולל את ההמרה.",
                    ));
                }
                steps.extend(buying(
                    ctx,
                    both(|lang| match lang {
                        Lang::En => format!("Buy the fund for {now}."),
                        Lang::He => format!("קונים את הקרן ב\u{2011}{now}."),
                    }),
                    both(|lang| match lang {
                        Lang::En => format!("Every month, buy {monthly} more of it."),
                        Lang::He => format!("בכל חודש קונים ממנה עוד ב\u{2011}{monthly}."),
                    }),
                ));
                steps.push(both(|lang| {
                    match (end(ctx), ctx.flexibility, lang) {
                        (Some(_), Some(Flexibility::CanWait), Lang::En) => format!(
                            "Don't touch it until {}. If the S&P 500 is down then, wait a year or two.",
                            year(ctx, lang)
                        ),
                        (Some(_), Some(Flexibility::CanWait), Lang::He) => format!(
                            "לא נוגעים עד {}. אם מדד S&P\u{a0}500 יהיה אז בירידה, מחכים שנה\u{2011}שנתיים.",
                            year(ctx, lang)
                        ),
                        (Some(_), _, Lang::En) => format!("Don't touch it until {}.", year(ctx, lang)),
                        (Some(_), _, Lang::He) => format!("לא נוגעים עד {}.", year(ctx, lang)),
                        (None, _, _) => lang
                            .pick(
                                "Don't touch it until you need the money.",
                                "לא נוגעים עד שצריך את הכסף.",
                            )
                            .to_owned(),
                    }
                }));
                steps.push(index_tax(way.product));
            } else {
                steps.extend(fund_steps(ctx, way.vehicle));
            }
        }
    }
    steps
}

/// The month of the deposits' rates, "09/2026".
fn deposits_month() -> String {
    let date = crate::short_term::deposits::checked();
    format!("{:02}/{}", u8::from(date.month()), date.year())
}

// ─────────────────────────── Where to buy ───────────────────────────

/// A bigger first deposit that would open another place, said as the
/// difference it makes (principle 13), not as two sums to compare.
#[must_use]
pub fn unlock(unlock: &UnlockLine) -> Text {
    let more = unlock.after_tax.today - unlock.instead_of.today;
    both(|lang| {
        let who = format!(
            "{}{}{}",
            &unlock.plan.provider[lang],
            lang.pick(" · ", "\u{a0}· "),
            &unlock.plan.plan[lang]
        );
        let now = if unlock.first_deposit_now.is_zero() {
            lang.pick("you have none now", "עכשיו אין לכם").to_owned()
        } else {
            match lang {
                Lang::En => format!("now {}", shekels(unlock.first_deposit_now)),
                Lang::He => format!("עכשיו {}", shekels(unlock.first_deposit_now)),
            }
        };
        match lang {
            Lang::En => format!(
                "A first deposit of {} ({now}) would also open {who}: with that deposit it would leave about {} more than the places above.",
                shekels(unlock.first_deposit),
                about(more)
            ),
            Lang::He => format!(
                "הפקדה ראשונה של {} ({now}) הייתה פותחת גם את {who}: עם הפקדה כזאת שם היה נשאר בערך {} יותר מאשר במקומות שלמעלה.",
                shekels(unlock.first_deposit),
                about(more)
            ),
        }
    })
}

/// The warning of a place that leaves more but may cost more, said under
/// its row: "⚠ Altshuler · offer may cost more: …".
#[must_use]
pub fn may_cost_more(also: &AlsoPossible) -> Text {
    both(|lang| {
        let who = format!(
            "{}{}{}",
            &also.choice.provider[lang],
            lang.pick(" · ", "\u{a0}· "),
            &also.choice.plan[lang]
        );
        match lang {
            Lang::En => format!(
                "\u{26a0} {who}: {}",
                also.warning
                    .en
                    .replacen("May cost more", "may cost more", 1)
            ),
            Lang::He => format!("\u{26a0} {who}: {}", &also.warning[lang]),
        }
    })
}

/// The warning on the row itself, where the row is that place: the
/// warning's own words.
#[must_use]
pub fn row_warning(also: &AlsoPossible) -> Text {
    also.warning.clone()
}
