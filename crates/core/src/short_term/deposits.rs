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
    date!(2026 - 10 - 18)
}

/// The month the deposits were opened in.
const DATA_OF: TariffDate = TariffDate::Month(date!(2026 - 09 - 01));

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

/// The banks' deposits, as the Bank of Israel published them for September 2026.
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
        pays: Pays::Fixed(rates![1.76 2.54 2.73 3.54 3.22 2.93 3.01]),
        caveats: vec![],
        compared_at_first: true,
    };
    let banks = vec![
        bank(
            t("Bank Hapoalim", "בנק הפועלים"),
            t("Hapoalim", "הפועלים"),
            true,
            rates![1.95 2.26 3.37 3.77 2.83 3.1 2.96],
            vec![],
        ),
        bank(
            t("Bank Leumi", "בנק לאומי"),
            t("Leumi", "לאומי"),
            true,
            rates![1.52 2.93 2.8 3.66 3.86 2.93 2.65],
            vec![],
        ),
        bank(
            t("Israel Discount Bank", "בנק דיסקונט"),
            t("Discount", "דיסקונט"),
            true,
            rates![1.97 2.32 2.42 3.51 2.43 2.45 1.72],
            vec![],
        ),
        bank(
            t("Mizrahi-Tefahot Bank", "בנק מזרחי\u{2011}טפחות"),
            t("Mizrahi", "מזרחי"),
            true,
            rates![1.63 1.85 1.41 3.3 3.35 3.02 3.1],
            open_to_anyone(
                "Mizrahi-Tefahot opens a deposit even for someone without a current account \
                 with it, as the Bank of Jerusalem does.",
                "מזרחי\u{2011}טפחות פותח פיקדון גם למי שאין לו חשבון עו״ש אצלו, כמו בנק ירושלים.",
            ),
        ),
        bank(
            t("First International Bank", "הבנק הבינלאומי"),
            t("First International", "הבינלאומי"),
            true,
            rates![1.55 2.49 2.3 3.3 3.29 2.81 2.95],
            vec![],
        ),
        bank(
            t("Bank Yahav", "בנק יהב"),
            t("Yahav", "יהב"),
            false,
            rates![0.29 0.29 1.79 2.91 3.7 2.2 2.82],
            vec![],
        ),
        bank(
            t("Mercantile Bank", "בנק מרכנתיל"),
            t("Mercantile", "מרכנתיל"),
            false,
            rates![2.84 2.99 3.44 2.9 3.18 2.57 1.38],
            vec![],
        ),
        bank(
            t("Bank Massad", "בנק מסד"),
            t("Massad", "מסד"),
            false,
            rates![1.8 2.75 3.04 2.79 3.15 1.88 -],
            vec![],
        ),
        bank(
            t("Bank of Jerusalem", "בנק ירושלים"),
            t("Jerusalem", "ירושלים"),
            false,
            rates![2.51 2.83 3.43 3.71 3.73 3.79 3.81],
            open_to_anyone(
                "The Bank of Jerusalem opens a deposit even for someone without a current \
                 account with it, as Mizrahi-Tefahot does.",
                "בנק ירושלים פותח פיקדון גם למי שאין לו חשבון עו״ש אצלו, כמו מזרחי\u{2011}טפחות.",
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
                        "וואן זירו, בנק דיגיטלי, פרסם ריבית רק לפיקדונות של 6 חודשים עד שנה: 6% בשנה, הרבה מעל שאר הבנקים. כנראה זה מבצע ללקוחות חדשים: בנובמבר 2025 הוא הציע 5.5% לשנה למצטרפים למסלולי הפרימיום שלו.",
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
            "You give a bank one sum for a term chosen in advance, at a rate set on the day \
             it's opened that doesn't change. It suits money you're sure you won't need before \
             a known date, especially when you expect the Bank of Israel to cut its rate. The \
             downside: the money is locked until the end, and taking it out early usually \
             loses the interest. And the rate depends on the bank and the term, and can be \
             well below the Bank of Israel's, especially for short terms.",
            "נותנים לבנק סכום אחד לתקופה שבוחרים מראש, בריבית שנקבעת ביום הפתיחה ולא משתנה. מתאים לכסף שבטוח לא תצטרכו עד תאריך ידוע, במיוחד כשמצפים שריבית בנק ישראל תרד. החיסרון: הכסף נעול עד הסוף, ויציאה מוקדמת מפסידה בדרך כלל את הריבית. והריבית תלויה בבנק ובתקופה, ויכולה להיות נמוכה בהרבה מריבית בנק ישראל, במיוחד בתקופות קצרות.",
        ),
        data_of: DATA_OF,
        checked: checked(),
        source: figures.clone(),
        caveats: vec![
            Caveat::published(t(
                "Each bank's rate is the average on the fixed-rate, unlinked shekel deposits \
                 households opened with it in September 2026, as it reports them to the Bank of \
                 Israel. It's what depositors got on average, not a rate on offer: yours may \
                 be higher or lower, and some customers get more than others.",
                "הריבית של כל בנק היא הממוצע על הפיקדונות בריבית קבועה, בשקלים ולא צמודים, שמשקי בית פתחו בו בספטמבר 2026, כפי שהבנק מדווח לבנק ישראל. זה מה שמפקידים קיבלו בממוצע, ולא ריבית שמוצעת לכם: אצלכם היא יכולה להיות גבוהה או נמוכה יותר, ויש לקוחות שמקבלים יותר.",
            ))
            .sources(&[&figures, &the_figures()]),
            Caveat::reading(
                t(
                    "The Bank of Israel groups deposits by term: up to a month, 1 to 3 months, \
                     and so on up to 3 to 5 years. A deposit for the whole period is taken at \
                     the rate of the group it falls in, each group taking the terms longer than \
                     the one before it: 12 months is \"6 months to a year\", 13 months \"1 to \
                     2 years\".",
                    "בנק ישראל מקבץ את הפיקדונות לפי אורך התקופה: עד חודש, חודש עד 3 חודשים, וכן הלאה עד 3 שנים עד 5 שנים. הפיקדון מקבל את הריבית של הטווח שהתקופה שלו נופלת בו. כל טווח כולל את הקצה העליון שלו: 12 חודשים נכללים ב״6 חודשים עד שנה״, ו\u{2011}13 חודשים ב״שנה עד שנתיים״.",
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
                    "כשלבנק אין ריבית לתקופה מסוימת, בנק ישראל לא פרסם אותה: הוא משמיט תקופה שבה נפתחו בבנק פחות מעשרה פיקדונות בחודש, או פחות מ\u{2011}5% מהפיקדונות של החודש. במקרה כזה הפיקדון לא מושווה.",
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The Bank of Israel publishes a month's figures about the 10th of the
    /// next: September's on 11 October 2026. Fails once the month after
    /// `DATA_OF` has been out about five days, so a stale rate can't rank the
    /// banks unnoticed: run `policies/deposit-rates.py`.
    #[test]
    fn the_rates_are_no_more_than_a_month_behind() {
        let (TariffDate::Month(month) | TariffDate::Day(month)) = DATA_OF;
        let today = time::OffsetDateTime::now_utc().date();
        let index = |date: Date| date.year() * 12 + i32::from(u8::from(date.month()));
        let behind = index(today) - index(month);
        assert!(
            behind < 2 || (behind == 2 && today.day() < 15),
            "the deposits' rates are of {month}: the Bank of Israel has published newer ones"
        );
    }
}
