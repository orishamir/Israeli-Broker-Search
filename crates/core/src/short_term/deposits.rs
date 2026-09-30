//! The banks' fixed-rate deposits: each bank's average yearly rate on the
//! unlinked shekel deposits households opened with it at a fixed rate in a
//! month, by term, as the Bank of Israel publishes them (קו המשווה).
//! `policies/deposit-rates.py` prints them from its figures, rounded to two
//! places; `policies/sources.md` says how they were read.

use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use super::{Kind, Pays, Place, Rates};
use crate::{Caveat, Page, Percent, TariffDate, Text};

/// When the rates were last taken from the Bank of Israel's figures.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 10 - 01)
}

/// The month the deposits were opened in.
const DATA_OF: TariffDate = TariffDate::Month(date!(2026 - 08 - 01));

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

fn source(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

/// A bank's rate for each term, the shortest first, as a list of numbers
/// with `-` where the Bank of Israel published none: `rates![2.12 - 3.5 …]`
/// reads like the table `policies/deposit-rates.py` prints. A macro, since
/// it matches `-` as a token of its own, which a function can't take.
macro_rules! rates {
    (@one -) => {
        None
    };
    (@one $rate:literal) => {
        Some(Percent(dec!($rate)))
    };
    ($($rate:tt)*) => {
        Rates::new([$(rates!(@one $rate)),*])
    };
}

// ─────────────────────────── Sources ───────────────────────────

/// The Bank of Israel's page comparing the banks' deposit rates.
#[must_use]
pub fn kav_hamashve() -> Page {
    source(
        t(
            "The Bank of Israel's comparison of deposit rates (קו המשווה)",
            "השוואת הריביות על פיקדונות של בנק ישראל (קו המשווה)",
        ),
        "https://www.boi.org.il/information/bank-paymnts/financial-education/campaigns/boi-equator/deposit/",
    )
}

/// The figures behind it.
fn the_figures() -> Page {
    source(
        t(
            "The Bank of Israel's figures, by bank and term (Excel)",
            "הנתונים של בנק ישראל, לפי בנק ותקופה (אקסל)",
        ),
        "https://www.boi.org.il/boi_files/Pikuah/g060a.xls",
    )
}

fn the_cut() -> Page {
    source(
        t(
            "ynet on the cut to 3.25% (1 September 2026)",
            "ynet על הורדת הריבית ל-3.25% (1 בספטמבר 2026)",
        ),
        "https://www.ynet.co.il/economy/article/cdo59v3t1",
    )
}

fn calcalist() -> Page {
    source(
        t(
            "Calcalist on the banks' deposit rates (13 November 2025)",
            "כלכליסט על הריביות בפיקדונות בבנקים (13 בנובמבר 2025)",
        ),
        "https://www.calcalist.co.il/market/article/bytylkzgbl",
    )
}

/// The law's own words on the tax on interest.
fn section_125c() -> Page {
    source(
        t(
            "The Income Tax Ordinance, section 125ג (Hebrew Wikisource)",
            "פקודת מס הכנסה, סעיף 125ג (ויקיטקסט)",
        ),
        "https://he.wikisource.org/wiki/%D7%A4%D7%A7%D7%95%D7%93%D7%AA_%D7%9E%D7%A1_%D7%94%D7%9B%D7%A0%D7%A1%D7%94",
    )
}

// ─────────────────────────── The banks ───────────────────────────

/// A bank's deposits; `big` for the five big banks, ticked at first.
fn bank(name: Text, short_name: Text, big: bool, pays: Rates, caveats: Vec<Caveat>) -> Place {
    Place {
        compared_at_first: big,
        description: t(
            "A deposit at a fixed rate: the rate is set when it's opened and holds for the \
             whole term, and the money comes out at the end.",
            "פיקדון בריבית קבועה: הריבית נקבעת ביום הפתיחה ונשארת לכל התקופה, והכסף יוצא בסופה.",
        ),
        name,
        short_name,
        pays: Pays::Fixed(pays),
        caveats,
    }
}

/// Opens a deposit for anyone, customer or not (Calcalist, November 2025).
fn open_to_anyone(en: &'static str, he: &'static str) -> Vec<Caveat> {
    vec![Caveat::published(t(en, he)).source(&calcalist())]
}

/// The banks' deposits, as the Bank of Israel published them for August 2026.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the banks' data")]
pub fn kind() -> Kind {
    let figures = kav_hamashve();
    let average = Place {
        name: t("Banks' average", "ממוצע הבנקים"),
        short_name: t("Banks' average", "ממוצע הבנקים"),
        description: t(
            "What households got on average for fixed-rate deposits, at all the banks together.",
            "מה שמשקי בית קיבלו בממוצע על פיקדונות בריבית קבועה, בכל הבנקים יחד.",
        ),
        pays: Pays::Fixed(rates![1.84 2.67 2.93 3.6 3.35 2.82 3.12]),
        caveats: vec![],
        compared_at_first: true,
    };
    let banks = vec![
        bank(
            t("Bank Hapoalim", "בנק הפועלים"),
            t("Hapoalim", "הפועלים"),
            true,
            rates![2.12 2.54 3.53 3.76 2.92 3.04 3.11],
            vec![],
        ),
        bank(
            t("Bank Leumi", "בנק לאומי"),
            t("Leumi", "לאומי"),
            true,
            rates![1.78 3.22 2.85 3.75 3.62 2.62 2.55],
            vec![],
        ),
        bank(
            t("Israel Discount Bank", "בנק דיסקונט"),
            t("Discount", "דיסקונט"),
            true,
            rates![1.83 2.38 2.55 3.49 2.55 1.73 1.67],
            vec![],
        ),
        bank(
            t("Mizrahi-Tefahot Bank", "בנק מזרחי-טפחות"),
            t("Mizrahi", "מזרחי"),
            true,
            rates![1.74 1.87 1.47 3.46 3.5 3.44 3.12],
            open_to_anyone(
                "Mizrahi-Tefahot opens a deposit even for someone without a current account \
                 with it, as the Bank of Jerusalem does.",
                "מזרחי-טפחות פותח פיקדון גם למי שאין לו חשבון עו״ש אצלו, כמו בנק ירושלים.",
            ),
        ),
        bank(
            t("First International Bank", "הבנק הבינלאומי"),
            t("First International", "הבינלאומי"),
            true,
            rates![1.44 2.13 3.35 3.42 3.58 2.84 3.09],
            vec![],
        ),
        bank(
            t("Bank Yahav", "בנק יהב"),
            t("Yahav", "יהב"),
            false,
            rates![0.3 0.61 1.43 3.1 3.82 2.2 3.11],
            vec![],
        ),
        bank(
            t("Mercantile Bank", "בנק מרכנתיל"),
            t("Mercantile", "מרכנתיל"),
            false,
            rates![2.65 3.22 3.17 3.27 3.42 2.62 1.14],
            vec![],
        ),
        bank(
            t("Bank Massad", "בנק מסד"),
            t("Massad", "מסד"),
            false,
            rates![2.13 2.56 2.97 2.95 3.07 2.1 -],
            vec![],
        ),
        bank(
            t("Bank of Jerusalem", "בנק ירושלים"),
            t("Jerusalem", "ירושלים"),
            false,
            rates![2.69 3.06 3.69 3.87 3.84 - 3.95],
            open_to_anyone(
                "The Bank of Jerusalem opens a deposit even for someone without a current \
                 account with it, as Mizrahi-Tefahot does.",
                "בנק ירושלים פותח פיקדון גם למי שאין לו חשבון עו״ש אצלו, כמו מזרחי-טפחות.",
            ),
        ),
        bank(
            t("One Zero", "וואן זירו"),
            t("One Zero", "וואן זירו"),
            false,
            rates![- - - 6 - - -],
            vec![
                Caveat::may_cost_more(
                    t(
                        "One Zero, a digital bank, published a rate only for deposits of 6 \
                         months to a year: 6% a year, far above the other banks. It's likely \
                         an offer for new customers: in November 2025 it offered 5.5% for a \
                         year to those joining its premium tracks.",
                        "וואן זירו, בנק דיגיטלי, פרסמה ריבית רק לפיקדונות של 6 חודשים עד שנה: 6% בשנה, הרבה מעל שאר הבנקים. כנראה זה מבצע ללקוחות חדשים: בנובמבר 2025 היא הציעה 5.5% לשנה למצטרפים למסלולי הפרימיום שלה.",
                    ),
                    t("Likely an offer for new customers", "כנראה מבצע ללקוחות חדשים"),
                )
                .sources(&[&the_figures(), &calcalist()]),
            ],
        ),
    ];

    Kind {
        name: t("Fixed-rate deposit", "פיקדון בריבית קבועה"),
        description: t(
            "A deposit at a bank, at a fixed rate (פיקדון\u{a0}בריבית\u{a0}קבועה): the bank \
             sets the rate when it's opened, for the whole term, and the money is locked \
             until the term ends. The Bank of Israel publishes each bank's average rate every \
             month, by how long the deposits are for.",
            "פיקדון בבנק בריבית קבועה: הבנק קובע את הריבית ביום הפתיחה, לכל התקופה, והכסף נעול עד שהתקופה נגמרת. בנק ישראל מפרסם כל חודש את הריבית הממוצעת בכל בנק, לפי אורך התקופה.",
        ),
        data_of: DATA_OF,
        checked: checked(),
        source: figures.clone(),
        caveats: vec![
            Caveat::published(t(
                "Each bank's rate is the average on the fixed-rate, unlinked shekel deposits \
                 households opened with it in August 2026, as it reports them to the Bank of \
                 Israel. It's what depositors got on average, not a rate on offer: yours may \
                 be higher or lower, and some customers get more than others.",
                "הריבית של כל בנק היא הממוצע על הפיקדונות בריבית קבועה, בשקלים ולא צמודים, שמשקי בית פתחו בו באוגוסט 2026, כפי שהבנק מדווח לבנק ישראל. זה מה שמפקידים קיבלו בממוצע, ולא ריבית שמוצעת לכם: אצלכם היא יכולה להיות גבוהה או נמוכה יותר, ויש לקוחות שמקבלים יותר.",
            ))
            .sources(&[&figures, &the_figures()]),
            Caveat::may_cost_more(
                t(
                    "These are August's rates, when the Bank of Israel's rate was 3.5%. It has \
                     been 3.25% since 1 September, so a deposit opened now may pay less.",
                    "אלה הריביות של אוגוסט, כשריבית בנק ישראל הייתה 3.5%. מ-1 בספטמבר היא 3.25%, ולכן פיקדון שנפתח היום עשוי לשלם פחות.",
                ),
                t("August's rates, before a cut", "ריביות אוגוסט, לפני הורדת ריבית"),
            )
            .sources(&[&the_figures(), &the_cut()]),
            Caveat::reading(
                t(
                    "The Bank of Israel groups deposits by term: up to a month, 1 to 3 months, \
                     and so on up to 3 to 5 years. A deposit for the whole period is taken at \
                     the rate of the group it falls in, each group taking the terms longer than \
                     the one before it: 12 months is \"6 months to a year\", 13 months \"1 to \
                     2 years\".",
                    "בנק ישראל מקבץ את הפיקדונות לפי אורך התקופה: עד חודש, חודש עד 3 חודשים, וכן הלאה עד 3 שנים עד 5 שנים. פיקדון לכל התקופה נלקח בריבית של הקבוצה שהוא נופל בה, וכל קבוצה כוללת את התקופות שארוכות מאלה של הקבוצה שלפניה: 12 חודשים הם ״6 חודשים עד שנה״, ו-13 חודשים הם ״שנה עד שנתיים״.",
                ),
                t(
                    "the terms' names in the Bank of Israel's figures",
                    "שמות התקופות בנתוני בנק ישראל",
                ),
            )
            .source(&the_figures()),
            Caveat::reading(
                t(
                    "The rates are for a year: part of a year earns that part of the year's \
                     interest, and a whole year's interest joins the deposit and earns \
                     interest too.",
                    "הריביות הן לשנה: חלק משנה מקבל את החלק היחסי מהריבית של השנה, והריבית של שנה שלמה מצטרפת לפיקדון ומקבלת גם היא ריבית.",
                ),
                t(
                    "the Bank of Israel's figures, which give yearly rates",
                    "נתוני בנק ישראל, שמביאים ריבית שנתית",
                ),
            )
            .source(&the_figures()),
            Caveat::reading(
                t(
                    "Where a bank has no rate for a term, the Bank of Israel published none: \
                     it leaves out a bank's term with fewer than ten deposits in the month, or \
                     under 5% of the month's deposits. The deposit isn't compared then.",
                    "כשלבנק אין ריבית לתקופה מסוימת, בנק ישראל לא פרסם אותה: הוא משמיט תקופה שבה נפתחו בבנק פחות מעשרה פיקדונות בחודש, או פחות מ-5% מהפיקדונות של החודש. במקרה כזה הפיקדון לא מושווה.",
                ),
                t(
                    "the Bank of Israel's note on its figures",
                    "ההערה של בנק ישראל על הנתונים",
                ),
            )
            .source(&figures),
            Caveat::published(t(
                "The interest is taxed at 15%, all of it, as interest on anything not linked to \
                 the price index: the bank withholds the tax when it pays (the Income Tax \
                 Ordinance, section 125ג(ג)(1)).",
                "על הריבית משלמים מס של 15%, על כולה, כמו על כל ריבית על כסף שאינו צמוד למדד: הבנק מנכה את המס כשהוא משלם (פקודת מס הכנסה, סעיף 125ג(ג)(1)).",
            ))
            .source(&section_125c()),
        ],
        places: std::iter::once(average).chain(banks).collect(),
    }
}
