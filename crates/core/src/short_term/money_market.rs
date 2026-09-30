//! Money market funds (קרנות כספיות): mutual funds that keep their money in
//! the Bank of Israel's short-term bills, short government bonds and bank
//! deposits. Listed as a kind, like the provident funds ([`crate::funds`]):
//! what the funds charge on average, and the cheapest and the dearest fund.
//! `policies/money-market-funds.py` works the fees out from the funds'
//! reports to the Tel Aviv Stock Exchange; `policies/sources.md` says how.

use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use super::{Kind, Pays, Place};
use crate::{Caveat, Page, Percent, TariffDate, Text, funds};

/// When the fees were last worked out.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 10 - 01)
}

/// The day the funds' fees are from.
const DATA_OF: TariffDate = TariffDate::Day(date!(2026 - 09 - 30));

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

fn source(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

// ─────────────────────────── Sources ───────────────────────────

/// The Tel Aviv Stock Exchange's list of mutual funds, with each one's fees.
#[must_use]
pub fn maya() -> Page {
    source(
        t(
            "Maya, the Tel Aviv Stock Exchange's list of mutual funds",
            "מאיה, רשימת קרנות הנאמנות של הבורסה בתל אביב",
        ),
        "https://maya.tase.co.il/he/funds/mutual-funds",
    )
}

fn dolphin() -> Page {
    source(
        t(
            "Ayalon's Dolphin fund on Maya",
            "דולפין כספית שקלית של איילון במאיה",
        ),
        "https://maya.tase.co.il/he/funds/mutual-funds/5141098",
    )
}

fn custody_rule() -> Page {
    source(
        t(
            "The Bank of Israel on custody for makam and money market funds (28 November 2012)",
            "בנק ישראל על דמי משמרת על מק״מ ועל קרנות כספיות (28 בנובמבר 2012)",
        ),
        "https://boi.org.il/publications/pressreleases/%D7%94%D7%9E%D7%A4%D7%A7%D7%97-%D7%A2%D7%9C-%D7%94%D7%91%D7%A0%D7%A7%D7%99%D7%9D-%D7%A0%D7%95%D7%A7%D7%98-%D7%91%D7%A6%D7%A2%D7%93%D7%99%D7%9D-%D7%9C%D7%94%D7%A4%D7%97%D7%AA%D7%AA-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-%D7%A2%D7%91%D7%95%D7%A8-%D7%9E%D7%A9%D7%A7%D7%99-%D7%94%D7%91%D7%99%D7%AA-%D7%95%D7%94%D7%A2%D7%A1%D7%A7%D7%99%D7%9D-%D7%94%D7%A7%D7%98%D7%A0%D7%99%D7%9D/",
    )
}

fn leumi_tariff() -> Page {
    source(
        t("Leumi's tariff (PDF)", "התעריפון של לאומי (PDF)"),
        "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf",
    )
}

fn excellence_tariff() -> Page {
    source(
        t("Excellence's tariff (PDF)", "התעריפון של אקסלנס (PDF)"),
        "https://www.xnes.co.il/media/nwogkoos/taarifon.pdf",
    )
}

fn ibi_funds() -> Page {
    source(
        t("IBI's page on funds", "עמוד הקרנות באתר IBI"),
        "https://www.ibi.co.il/solutions/zero-balance-managed-funds/",
    )
}

fn rate_came_down() -> [Page; 2] {
    [
        source(
            t(
                "Calcalist on the banks' deposit rates (13 November 2025), at a rate of 4.5%",
                "כלכליסט על הריביות בפיקדונות בבנקים (13 בנובמבר 2025), בריבית של 4.5%",
            ),
            "https://www.calcalist.co.il/market/article/bytylkzgbl",
        ),
        source(
            t(
                "ynet on the cut to 3.25% (1 September 2026)",
                "ynet על הורדת הריבית ל-3.25% (1 בספטמבר 2026)",
            ),
            "https://www.ynet.co.il/economy/article/cdo59v3t1",
        ),
    ]
}

// ─────────────────────────── The funds ───────────────────────────

/// A fund at `fee`; `ticked` if it's compared when the calculator opens.
fn fund(name: Text, description: Text, fee: Percent, ticked: bool, caveats: Vec<Caveat>) -> Place {
    Place {
        compared_at_first: ticked,
        short_name: name.clone(),
        name,
        description,
        pays: Pays::TheRateLess(fee),
        caveats,
    }
}

/// The shekel money market funds, as their managers reported them on 30
/// September 2026: 44 funds holding ₪185 billion.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the funds' data")]
pub fn kind() -> Kind {
    let maya = maya();
    let average = fund(
        t("Average fund", "קרן ממוצעת"),
        t(
            "A fund that charges what the 44 shekel money market funds charge on average, \
             each counted by the money it holds.",
            "קרן שגובה את דמי הניהול הממוצעים של 44 הקרנות הכספיות השקליות, כשכל קרן משפיעה על הממוצע לפי כמות הכסף שבה.",
        ),
        Percent(dec!(0.169)),
        true,
        vec![],
    );
    let cheapest = fund(
        t("Cheapest fund", "הקרן הזולה"),
        t(
            "The shekel money market fund that charges the least: Ayalon's Dolphin \
             (דולפין\u{a0}כספית\u{a0}שקלית), which takes no management fee, only the \
             trustee's 0.01%.",
            "הקרן הכספית השקלית שגובה הכי מעט: דולפין כספית שקלית של איילון, שלא גובה דמי ניהול, רק 0.01% דמי נאמנות.",
        ),
        Percent(dec!(0.01)),
        true,
        vec![
            Caveat::may_cost_more(
                t(
                    "A fund less than a year old: its 0% management fee may be an opening \
                     offer, and could rise.",
                    "קרן בת פחות משנה: דמי ניהול של 0% עשויים להיות מבצע פתיחה, ויכולים לעלות.",
                ),
                t(
                    "A new fund: its fee may rise",
                    "קרן חדשה: דמי הניהול עשויים לעלות",
                ),
            )
            .source(&dolphin()),
        ],
    );
    let dearest = fund(
        t("Dearest fund", "הקרן היקרה"),
        t(
            "The shekel money market fund that charges the most: Meitav's \
             (מיטב\u{a0}כספית).",
            "הקרן הכספית השקלית שגובה הכי הרבה: מיטב כספית.",
        ),
        Percent(dec!(0.26)),
        false,
        vec![],
    );

    let [before, after] = rate_came_down();
    Kind {
        name: t("Money market fund", "קרן כספית"),
        description: t(
            "A mutual fund that keeps its money in the Bank of Israel's short-term bills \
             (מק״מ), short government bonds and bank deposits, most of them some corporate \
             bonds too, so it earns about the Bank of Israel's rate, less its fee. Its units \
             are bought and sold through a bank or an investment house, any business day. The \
             app compares the kind, not each fund: the fees are close, and the list shows the \
             average, the cheapest and the dearest.",
            "קרן נאמנות שמחזיקה את הכסף במק״מ של בנק ישראל, באג״ח ממשלתיות קצרות ובפיקדונות בבנקים, ורובן גם מעט אג״ח של חברות. לכן היא מרוויחה בערך את ריבית בנק ישראל, פחות דמי הניהול. קונים ומוכרים את היחידות דרך בנק או בית השקעות, בכל יום עסקים. המחשבון משווה את סוג הקרן, ולא כל קרן בנפרד: דמי הניהול קרובים, והרשימה מראה את הממוצע, את הזולה ואת היקרה.",
        ),
        data_of: DATA_OF,
        checked: checked(),
        source: maya.clone(),
        caveats: vec![
            Caveat::published(t(
                "The fees the fund managers report to the Tel Aviv Stock Exchange (30 \
                 September 2026): the manager's fee and the trustee's, both taken from the \
                 fund's assets. The distribution fee beside them is paid by the manager, out \
                 of its own fee.",
                "דמי הניהול שמנהלי הקרנות מדווחים לבורסה (30 בספטמבר 2026): דמי הניהול של מנהל הקרן ודמי הנאמנות, ששניהם יורדים מנכסי הקרן. את עמלת ההפצה שמופיעה לצידם משלם מנהל הקרן, מתוך דמי הניהול שלו.",
            ))
            .source(&maya),
            Caveat::reading(
                t(
                    "A fund is taken to earn the Bank of Israel's rate, on average over the \
                     months, before its fee. What it holds follows the rate closely but not \
                     exactly: over the 12 months to September 2026 the largest funds returned \
                     4.1% to 4.3% after their fees, while the rate came down from 4.5% to \
                     3.25%.",
                    "ההנחה היא שקרן מרוויחה את ריבית בנק ישראל, בממוצע על פני התקופה, לפני דמי הניהול. מה שהיא מחזיקה עוקב אחרי הריבית מקרוב, אבל לא בדיוק: ב-12 החודשים עד ספטמבר 2026 הקרנות הגדולות הרוויחו 4.1% עד 4.3% אחרי דמי הניהול, בזמן שהריבית ירדה מ-4.5% ל-3.25%.",
                ),
                t("the funds' 12-month returns on Maya", "התשואות של הקרנות ל-12 חודשים במאיה"),
            )
            .sources(&[&maya, &before, &after]),
            Caveat::published(t(
                "Banks may not charge custody on a money market fund, nor on the Bank of \
                 Israel's bills: a Bank of Israel rule since 2013, which Leumi's price list, \
                 for one, repeats.",
                "בנקים לא רשאים לגבות דמי משמרת על קרן כספית, וגם לא על מק״מ: כלל של בנק ישראל מ-2013, שהתעריפון של לאומי, למשל, חוזר עליו.",
            ))
            .sources(&[&custody_rule(), &leumi_tariff()]),
            Caveat::reading(
                t(
                    "Bought through a bank, buying and selling are taken to cost nothing: \
                     banks leave mutual funds out of their trade fees, and the fund's manager \
                     pays them a distribution fee instead.",
                    "ההנחה היא שדרך בנק, קנייה ומכירה לא עולות כלום: הבנקים לא גובים עליהן את העמלות שלהם על קנייה ומכירה, ובמקום זה מנהל הקרן משלם להם עמלת הפצה.",
                ),
                t(
                    "Leumi's price list, whose trade fees leave mutual funds out",
                    "התעריפון של לאומי, שהעמלות שלו על קנייה ומכירה לא חלות על קרנות נאמנות",
                ),
            )
            .source(&leumi_tariff()),
            Caveat::not_counted(t(
                "At an investment house, a fund can cost a trade fee: Excellence charges ₪16 a \
                 trade on managed funds, while IBI trades some managers' funds free. None is \
                 counted: it depends on where the fund is bought, not on the fund.",
                "בבית השקעות, קרן יכולה לעלות עמלת קנייה ומכירה: אקסלנס גובה ₪16 לפעולה בקרנות מנוהלות, ו-IBI סוחר בקרנות של חלק מהמנהלים בחינם. העמלה לא נספרת כאן: היא תלויה במקום שבו קונים את הקרן, ולא בקרן.",
            ))
            .sources(&[&excellence_tariff(), &ibi_funds()]),
            Caveat::published(t(
                "Sold, a quarter of the gain beyond the rise in prices is tax, as for any \
                 security: a gain that only keeps up with prices isn't taxed (the Income Tax \
                 Ordinance, section 91(ב)(1)).",
                "במכירה משלמים מס רווחי הון, 25% מהרווח שמעבר לאינפלציה, כמו על כל נייר ערך: רווח שרק שומר על ערך הכסף לא ממוסה (פקודת מס הכנסה, סעיף 91(ב)(1)).",
            ))
            .source(&funds::income_tax_ordinance()),
        ],
        places: vec![average, cheapest, dearest],
    }
}
