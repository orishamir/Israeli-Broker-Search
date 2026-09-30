//! Funds and policies: a manager invests the money, for a share of it. The
//! app lists each kind as a whole rather than every company, since a
//! saver's fee is agreed person by person and the companies' averages are
//! close: its plans are what savers pay on average, at the cheapest and the
//! dearest company, and the most that's allowed.
//!
//! The fees are the Capital Market Authority's, as the funds report them
//! (`policies/gemel-net.py` works them out from its open data);
//! `policies/sources.md` says how they were read. The tax and the ceiling on
//! deposits are the vehicle's ([`crate::vehicles`]).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use crate::{
    Broker, BrokerKind, Caveat, ConversionFee, Exchange, FeeKind, IntoEnumIterator, ManagementFee,
    Page, Percent, Plan, Price, TariffDate, Text, TradeFee, Vehicle, ils, vehicles,
};

/// Every kind of fund the app knows, in the order to offer them.
#[must_use]
pub fn all() -> Vec<Broker> {
    vec![investment_gemel(), study_fund(), savings_policy()]
}

/// When the fees were last worked out from the regulator's data.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 09 - 30)
}

/// The month of the regulator's data the fees are from.
const DATA_OF: TariffDate = TariffDate::Month(date!(2026 - 08 - 01));

// ─────────────────────────── Helpers ───────────────────────────

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

fn source(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

/// A manager's fee: `of_balance` percent a year, `of_deposits` percent of
/// each deposit.
fn fee(of_balance: Decimal, of_deposits: Decimal) -> ManagementFee {
    ManagementFee {
        of_deposits: Percent(of_deposits),
        of_balance: Percent(of_balance),
    }
}

/// A plan of a fund or a policy: the manager's fee is all there is to pay.
/// Nothing is charged for the trades inside it or for holding, nothing is
/// converted by the saver, and every shekel is invested.
fn managed(
    name: Text,
    description: Text,
    vehicle: Vehicle,
    management: ManagementFee,
    caveats: Vec<Caveat>,
) -> Plan {
    Plan {
        name,
        description,
        vehicle,
        trading: vec![TradeFee {
            securities: vec![],
            exchanges: vec![],
            price: Price::Flat(ils(Decimal::ZERO)),
        }],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion: ConversionFee::FREE,
        handling: None,
        management: Some(management),
        fractions_on: Exchange::iter().collect(),
        min_first_deposit: None,
        caveats,
    }
}

// ─────────────────────────── Sources ───────────────────────────

/// The regulator's site comparing provident funds, from the funds' own
/// monthly reports.
#[must_use]
pub fn gemel_net() -> Page {
    source(
        t(
            "Gemel Net, the Capital Market Authority's comparison of provident funds",
            "גמל נט, השוואת קופות הגמל של רשות שוק ההון",
        ),
        "https://gemelnet.cma.gov.il/",
    )
}

/// The same for the insurers' policies.
#[must_use]
pub fn bituach_net() -> Page {
    source(
        t(
            "Bituach Net, the Capital Market Authority's comparison of insurers' policies",
            "ביטוח נט, השוואת פוליסות חברות הביטוח של רשות שוק ההון",
        ),
        "https://bituachnet.cma.gov.il/",
    )
}

fn kol_zchut_fund() -> Page {
    source(
        t(
            "Kol Zchut on the provident fund for investment",
            "כל-זכות על קופת גמל להשקעה",
        ),
        "https://www.kolzchut.org.il/he/%D7%A7%D7%95%D7%A4%D7%AA_%D7%92%D7%9E%D7%9C_%D7%9C%D7%94%D7%A9%D7%A7%D7%A2%D7%94",
    )
}

fn kol_zchut_fees() -> Page {
    source(
        t(
            "Kol Zchut on the fund's management fees",
            "כל-זכות על דמי הניהול בקופת גמל להשקעה",
        ),
        "https://www.kolzchut.org.il/he/%D7%93%D7%9E%D7%99_%D7%A0%D7%99%D7%94%D7%95%D7%9C_%D7%91%D7%A7%D7%95%D7%A4%D7%AA_%D7%92%D7%9E%D7%9C_%D7%9C%D7%94%D7%A9%D7%A7%D7%A2%D7%94",
    )
}

fn kol_zchut_study_fund() -> Page {
    source(
        t("Kol Zchut on the study fund", "כל-זכות על קרן השתלמות"),
        "https://www.kolzchut.org.il/he/%D7%A7%D7%A8%D7%9F_%D7%94%D7%A9%D7%AA%D7%9C%D7%9E%D7%95%D7%AA",
    )
}

fn kol_zchut_study_fund_self_employed() -> Page {
    source(
        t(
            "Kol Zchut on the study fund of the self-employed",
            "כל-זכות על קרן השתלמות לעובד עצמאי",
        ),
        "https://www.kolzchut.org.il/he/%D7%A7%D7%A8%D7%9F_%D7%94%D7%A9%D7%AA%D7%9C%D7%9E%D7%95%D7%AA_%D7%9C%D7%A2%D7%95%D7%91%D7%93_%D7%A2%D7%A6%D7%9E%D7%90%D7%99",
    )
}

fn management_fee_regulations() -> Page {
    source(
        t(
            "The regulations on provident funds' management fees (Hebrew Wikisource)",
            "תקנות דמי הניהול של קופות גמל (ויקיטקסט)",
        ),
        "https://he.wikisource.org/wiki/%D7%AA%D7%A7%D7%A0%D7%95%D7%AA_%D7%94%D7%A4%D7%99%D7%A7%D7%95%D7%97_%D7%A2%D7%9C_%D7%A9%D7%99%D7%A8%D7%95%D7%AA%D7%99%D7%9D_%D7%A4%D7%99%D7%A0%D7%A0%D7%A1%D7%99%D7%99%D7%9D_(%D7%A7%D7%95%D7%A4%D7%95%D7%AA_%D7%92%D7%9E%D7%9C)_(%D7%93%D7%9E%D7%99_%D7%A0%D7%99%D7%94%D7%95%D7%9C)",
    )
}

fn kol_zchut_pension() -> Page {
    source(
        t(
            "Kol Zchut on the tax-free pension from 60",
            "כל-זכות על הפטור ממס בקצבה לאחר גיל 60",
        ),
        "https://www.kolzchut.org.il/he/%D7%A4%D7%98%D7%95%D7%A8_%D7%9E%D7%9E%D7%A1_%D7%A2%D7%9C_%D7%A8%D7%95%D7%95%D7%97%D7%99%D7%9D_%D7%91%D7%A7%D7%95%D7%A4%D7%AA_%D7%92%D7%9E%D7%9C_%D7%9C%D7%94%D7%A9%D7%A7%D7%A2%D7%94_%D7%94%D7%9E%D7%A9%D7%95%D7%9C%D7%9E%D7%AA_%D7%9B%D7%A7%D7%A6%D7%91%D7%94_%D7%9C%D7%90%D7%97%D7%A8_%D7%92%D7%99%D7%9C_60",
    )
}

/// The law's own words on the tax: sections 88 and 91.
#[must_use]
pub fn income_tax_ordinance() -> Page {
    source(
        t(
            "The Income Tax Ordinance, sections 88 and 91 (Hebrew Wikisource)",
            "פקודת מס הכנסה, סעיפים 88 ו-91 (ויקיטקסט)",
        ),
        "https://he.wikisource.org/wiki/%D7%A4%D7%A7%D7%95%D7%93%D7%AA_%D7%9E%D7%A1_%D7%94%D7%9B%D7%A0%D7%A1%D7%94",
    )
}

/// A broker's plain account of the tax on securities.
#[must_use]
pub fn meitav_on_tax() -> Page {
    source(
        t(
            "Meitav Trade's guide to tax on securities",
            "המדריך של מיטב טרייד למיסוי ניירות ערך",
        ),
        "https://www.meitav.co.il/trade/capital_market_guide/capital_market_tax/",
    )
}

fn direct_expenses_regulations() -> Page {
    source(
        t(
            "The regulations on a provident fund's direct expenses (Hebrew Wikisource)",
            "תקנות ההוצאות הישירות של קופות גמל (ויקיטקסט)",
        ),
        "https://he.wikisource.org/wiki/%D7%AA%D7%A7%D7%A0%D7%95%D7%AA_%D7%94%D7%A4%D7%99%D7%A7%D7%95%D7%97_%D7%A2%D7%9C_%D7%A9%D7%99%D7%A8%D7%95%D7%AA%D7%99%D7%9D_%D7%A4%D7%99%D7%A0%D7%A0%D7%A1%D7%99%D7%99%D7%9D_(%D7%A7%D7%95%D7%A4%D7%95%D7%AA_%D7%92%D7%9E%D7%9C)_(%D7%94%D7%95%D7%A6%D7%90%D7%95%D7%AA_%D7%99%D7%A9%D7%99%D7%A8%D7%95%D7%AA_%D7%91%D7%A9%D7%9C_%D7%91%D7%99%D7%A6%D7%95%D7%A2_%D7%A2%D7%A1%D7%A7%D7%90%D7%95%D7%AA)",
    )
}

fn globes_on_the_cap() -> Page {
    source(
        t(
            "Globes on the proposed limit to the exemption",
            "גלובס על ההצעה להגביל את הפטור",
        ),
        "https://www.globes.co.il/news/article.aspx?did=1001547467",
    )
}

fn bizportal_on_the_cap() -> Page {
    source(
        t(
            "Bizportal on the proposed limit to the exemption",
            "ביזפורטל על ההצעה להגביל את הפטור",
        ),
        "https://www.bizportal.co.il/longtermsavings/news/article/20034793",
    )
}

fn bizportal_comparison() -> Page {
    source(
        t(
            "Bizportal's comparison of savings products (2026)",
            "השוואת מוצרי החיסכון של ביזפורטל (2026)",
        ),
        "https://www.bizportal.co.il/longtermsavings/news/article/20038012",
    )
}

fn analyst_comparison() -> Page {
    source(
        t(
            "Analyst's comparison of a savings policy and a provident fund for investment",
            "ההשוואה של אנליסט בין פוליסת חיסכון לקופת גמל להשקעה",
        ),
        "https://www.analyst.co.il/articles/savings-policy-and-investment-provident-fund/",
    )
}

fn menora_comparison() -> Page {
    source(
        t(
            "Menora Mivtachim's comparison of a savings policy and a provident fund for investment",
            "ההשוואה של מנורה מבטחים בין פוליסת חיסכון לקופת גמל להשקעה",
        ),
        "https://www.menoramivt.co.il/general/articles-fellow/gemel-invest-differnces",
    )
}

// ─────────────────────────── Caveats the kinds share ───────────────────────────

/// A fund's own investing costs, which no fee here includes.
fn investing_costs_arent_counted() -> Caveat {
    Caveat::not_counted(t(
        "A fund's own investing costs (הוצאות\u{a0}ישירות) come out of its return, on top of \
         the management fee: trading commissions, and outside managers' fees of up to 0.25% \
         of its assets a year. An ETF's own yearly fee at a broker isn't counted either.",
        "עלויות ההשקעה של הקופה עצמה (הוצאות ישירות) יורדות מהתשואה, בנוסף לדמי הניהול: עמלות מסחר, ודמי ניהול חיצוניים של עד 0.25% מנכסי הקופה בשנה. גם דמי הניהול השנתיים של קרן סל בחשבון מסחר לא נספרים.",
    ))
    .source(&direct_expenses_regulations())
}

/// The return is the user's own assumption, for a fund as for a security.
fn earns_what_the_security_does(what: Text) -> Caveat {
    let Text { en, he } = what;
    Caveat::not_counted(Text::owned(
        format!(
            "{en} is taken to earn what the security you chose earns, before fees. An \
             investment track that follows the same index (מסלול\u{a0}עוקב\u{a0}מדד) does. A \
             managed track earns more or less than that, and the app can't know which."
        ),
        format!(
            "ההנחה היא ש{he} מרוויחה, לפני דמי ניהול, מה שנייר הערך שבחרתם מרוויח. מסלול השקעה שעוקב אחרי אותו מדד אכן מרוויח כך. מסלול מנוהל מרוויח יותר או פחות, והאפליקציה לא יכולה לדעת."
        ),
    ))
}

// ─────────────────────────── Provident fund for investment ───────────────────────────

/// Kupat Gemel Le'hashkaa. Fees: Gemel Net's report for August 2026, the
/// funds anyone can join (123 tracks of 11 companies that report a fee),
/// each weighted by its assets.
#[must_use]
#[allow(clippy::too_many_lines, reason = "a fund's data")]
pub fn investment_gemel() -> Broker {
    let gemel_net = gemel_net();
    let from_the_reports = |en: &'static str, he: &'static str| {
        vec![
            Caveat::published(t(en, he))
                .about_fee(FeeKind::Management)
                .source(&gemel_net),
            Caveat::published(t(
                "Almost no fund takes a share of deposits: 0.001% of them on average, which \
                 is counted as none.",
                "כמעט אף קופה לא גובה דמי ניהול מהפקדות: 0.001% מהן בממוצע, שנספרים כאפס.",
            ))
            .about_fee(FeeKind::DepositFee)
            .source(&gemel_net),
        ]
    };

    let average = managed(
        t("Average fee", "דמי ניהול ממוצעים"),
        t(
            "What savers in a provident fund for investment pay on average, in the eleven \
             companies anyone can join, each fund counted by the money it holds. Your own fee \
             is what you agree with the company, which may give a discount: to compare with \
             yours, change a copy of this one.",
            "מה שחוסכים בקופת גמל להשקעה משלמים בממוצע, ב-11 החברות שכל אחד יכול להצטרף אליהן, כשכל קופה נספרת לפי היקף הכסף שבה. דמי הניהול שלכם הם מה שתסכמו עם החברה, שרשאית לתת הנחה: כדי להשוות לשלכם, שנו עותק של השורה הזאת.",
        ),
        Vehicle::InvestmentGemel,
        fee(dec!(0.62), dec!(0)),
        from_the_reports(
            "What savers paid on average, as each fund reports it to the Capital Market \
             Authority (data of August 2026). The figure is for a whole year, and last changed \
             in January 2026.",
            "מה שהחוסכים שילמו בממוצע, כפי שכל קופה מדווחת לרשות שוק ההון (נתוני אוגוסט 2026). הנתון הוא לשנה שלמה, והשתנה לאחרונה בינואר 2026.",
        ),
    );
    let cheapest = managed(
        t("Cheapest company", "החברה הזולה"),
        t(
            "The company whose savers pay the least on average: Harel, in the data of August \
             2026.",
            "החברה שהחוסכים שלה משלמים הכי מעט בממוצע: הראל, לפי נתוני אוגוסט 2026.",
        ),
        Vehicle::InvestmentGemel,
        fee(dec!(0.55), dec!(0)),
        from_the_reports(
            "What Harel's savers paid on average, over all its investment tracks, as it \
             reports to the Capital Market Authority (data of August 2026).",
            "מה שהחוסכים של הראל שילמו בממוצע, בכל מסלולי ההשקעה שלה, כפי שהיא מדווחת לרשות שוק ההון (נתוני אוגוסט 2026).",
        ),
    );
    let dearest = managed(
        t("Dearest company", "החברה היקרה"),
        t(
            "The company whose savers pay the most on average: Mor, in the data of August \
             2026.",
            "החברה שהחוסכים שלה משלמים הכי הרבה בממוצע: מור, לפי נתוני אוגוסט 2026.",
        ),
        Vehicle::InvestmentGemel,
        fee(dec!(0.72), dec!(0)),
        from_the_reports(
            "What Mor's savers paid on average, over all its investment tracks, as it reports \
             to the Capital Market Authority (data of August 2026).",
            "מה שהחוסכים של מור שילמו בממוצע, בכל מסלולי ההשקעה שלה, כפי שהיא מדווחת לרשות שוק ההון (נתוני אוגוסט 2026).",
        ),
    );
    let legal_maximum = managed(
        t("Legal maximum", "המקסימום המותר"),
        t(
            "The most the regulations let a fund take, from each deposit and from the \
             balance. A company may agree to less, and then can't raise the fee for two \
             years.",
            "המקסימום שהתקנות מתירות לקופה לגבות, מכל הפקדה ומהצבירה. חברה רשאית להסכים לפחות, ואז אסור לה להעלות את דמי הניהול במשך שנתיים.",
        ),
        Vehicle::InvestmentGemel,
        vehicles::LARGEST_GEMEL_FEE,
        vec![
            Caveat::published(t(
                "The ceiling in the management-fee regulations, section 2(a). A fund may not \
                 take more.",
                "התקרה שבתקנות דמי הניהול, סעיף 2(א). אסור לקופה לגבות יותר.",
            ))
            .about_fee(FeeKind::Management)
            .source(&kol_zchut_fees()),
        ],
    );

    Broker {
        name: t("Provident fund for investment", "קופת גמל להשקעה"),
        short_name: t("Provident fund", "גמל להשקעה"),
        kind: BrokerKind::Funds,
        new_customer_plan: 0, // Average fee
        compared_at_first: true,
        description: t(
            "A fund that a management company invests for you \
             (קופת\u{a0}גמל\u{a0}להשקעה): you choose an investment track and deposit, and it \
             does the buying and selling. Anyone can open one, and the money can be taken out \
             at any time. The app compares the kind of fund, not each company: the fee is \
             agreed person by person, and the companies' averages are close to each other.",
            "קופה שחברה מנהלת משקיעה עבורכם: אתם בוחרים מסלול השקעה ומפקידים, והיא קונה ומוכרת. כל אחד יכול לפתוח קופה, ואפשר למשוך את הכסף בכל עת. האפליקציה משווה את סוג הקופה ולא כל חברה בנפרד: דמי הניהול נקבעים לכל חוסך בנפרד, והממוצעים של החברות קרובים זה לזה.",
        ),
        tariff_date: Some(DATA_OF),
        source_url: Some(gemel_net.url.clone()),
        caveats: vec![
            Caveat::published(t(
                "Taken out at once, a quarter of the gain beyond the rise in prices is tax, as \
                 at a broker. From the age of 60 the money can be taken as a monthly pension \
                 instead, and then the gain isn't taxed and neither is the pension: the money \
                 moves to a fund that pays pensions, which sets the monthly amount.",
                "במשיכה בבת אחת, רבע מהרווח שמעבר לעליית המחירים הוא מס, כמו בחשבון מסחר. מגיל 60 אפשר לקבל את הכסף כקצבה חודשית, ואז הרווח פטור ממס וגם הקצבה: הכסף עובר לקופה שמשלמת קצבאות, והיא קובעת את הסכום החודשי.",
            ))
            .source(&kol_zchut_pension()),
            Caveat::published(t(
                "No more than ₪83,641 can be deposited in a calendar year (2026), in all of \
                 one person's funds together. The ceiling follows the price index every \
                 January.",
                "אי אפשר להפקיד יותר מ-₪83,641 בשנה קלנדרית (2026), בכל הקופות של אותו אדם יחד. התקרה מתעדכנת לפי המדד בכל ינואר.",
            ))
            .source(&kol_zchut_fund()),
            earns_what_the_security_does(t("The fund", "הקופה")),
            investing_costs_arent_counted(),
            Caveat::not_counted(t(
                "Moving between investment tracks, or to another company's fund, isn't taxed. \
                 At a broker, selling one security to buy another is. The app never switches, \
                 so this isn't counted.",
                "מעבר בין מסלולי השקעה, או לקופה של חברה אחרת, לא מחויב במס. בחשבון מסחר, מכירת נייר ערך כדי לקנות אחר כן מחויבת. האפליקציה אף פעם לא מחליפה, ולכן זה לא נספר.",
            ))
            .source(&analyst_comparison()),
            Caveat::not_counted(t(
                "In June 2026 a Finance Ministry committee recommended limiting the tax-free \
                 gain to ₪200,000 a saver. It isn't law, and the app counts the whole gain as \
                 tax-free.",
                "ביוני 2026 המליצה ועדה במשרד האוצר להגביל את הרווח הפטור ממס ל-₪200,000 לחוסך. זה לא חוק, והאפליקציה מחשיבה את כל הרווח כפטור.",
            ))
            .sources(&[&globes_on_the_cap(), &bizportal_on_the_cap()]),
        ],
        plans: vec![average, cheapest, dearest, legal_maximum],
    }
}

// ─────────────────────────── Study fund ───────────────────────────

/// Keren Hishtalmut, counted as a self-employed saver's. Fees: Gemel Net's
/// report for August 2026, the funds anyone can join, without the
/// self-managed tracks (131 tracks of 11 companies), each weighted by its
/// assets.
#[must_use]
#[allow(clippy::too_many_lines, reason = "a fund's data")]
pub fn study_fund() -> Broker {
    let gemel_net = gemel_net();
    let regulations = management_fee_regulations();
    let from_the_reports = |en: &'static str, he: &'static str| {
        vec![
            Caveat::published(t(en, he))
                .about_fee(FeeKind::Management)
                .source(&gemel_net),
            Caveat::published(t(
                "A study fund may take its fee from the balance only.",
                "קרן השתלמות רשאית לגבות דמי ניהול מהצבירה בלבד.",
            ))
            .about_fee(FeeKind::DepositFee)
            .source(&regulations),
        ]
    };

    let average = managed(
        t("Average fee", "דמי ניהול ממוצעים"),
        t(
            "What savers in a study fund pay on average, in the eleven companies anyone can \
             join, each fund counted by the money it holds. Your own fee is what you agree \
             with the company, which may give a discount: to compare with yours, change a \
             copy of this one.",
            "מה שחוסכים בקרן השתלמות משלמים בממוצע, ב-11 החברות שכל אחד יכול להצטרף אליהן, כשכל קרן נספרת לפי היקף הכסף שבה. דמי הניהול שלכם הם מה שתסכמו עם החברה, שרשאית לתת הנחה: כדי להשוות לשלכם, שנו עותק של השורה הזאת.",
        ),
        Vehicle::StudyFund,
        fee(dec!(0.61), dec!(0)),
        from_the_reports(
            "What savers paid on average, as each fund reports it to the Capital Market \
             Authority (data of August 2026). The figure is for a whole year, and last changed \
             in January 2026. Self-managed funds (ניהול\u{a0}אישי) aren't counted.",
            "מה שהחוסכים שילמו בממוצע, כפי שכל קרן מדווחת לרשות שוק ההון (נתוני אוגוסט 2026). הנתון הוא לשנה שלמה, והשתנה לאחרונה בינואר 2026. קרנות בניהול אישי לא נספרות.",
        ),
    );
    let cheapest = managed(
        t("Cheapest company", "החברה הזולה"),
        t(
            "The company whose savers pay the least on average: Migdal, in the data of August \
             2026.",
            "החברה שהחוסכים שלה משלמים הכי מעט בממוצע: מגדל, לפי נתוני אוגוסט 2026.",
        ),
        Vehicle::StudyFund,
        fee(dec!(0.53), dec!(0)),
        from_the_reports(
            "What Migdal's savers paid on average, over all its investment tracks, as it \
             reports to the Capital Market Authority (data of August 2026).",
            "מה שהחוסכים של מגדל שילמו בממוצע, בכל מסלולי ההשקעה שלה, כפי שהיא מדווחת לרשות שוק ההון (נתוני אוגוסט 2026).",
        ),
    );
    let dearest = managed(
        t("Dearest company", "החברה היקרה"),
        t(
            "The company whose savers pay the most on average: Mor, in the data of August \
             2026.",
            "החברה שהחוסכים שלה משלמים הכי הרבה בממוצע: מור, לפי נתוני אוגוסט 2026.",
        ),
        Vehicle::StudyFund,
        fee(dec!(0.70), dec!(0)),
        from_the_reports(
            "What Mor's savers paid on average, over all its investment tracks, as it reports \
             to the Capital Market Authority (data of August 2026).",
            "מה שהחוסכים של מור שילמו בממוצע, בכל מסלולי ההשקעה שלה, כפי שהיא מדווחת לרשות שוק ההון (נתוני אוגוסט 2026).",
        ),
    );
    let legal_maximum = managed(
        t("Legal maximum", "המקסימום המותר"),
        t(
            "The most the regulations let a study fund take from the balance. A company may \
             agree to less.",
            "המקסימום שהתקנות מתירות לקרן השתלמות לגבות מהצבירה. חברה רשאית להסכים לפחות.",
        ),
        Vehicle::StudyFund,
        vehicles::LARGEST_STUDY_FUND_FEE,
        vec![
            Caveat::published(t(
                "The ceiling in the management-fee regulations: a study fund may take up to \
                 this share of the balance a year, and nothing from deposits.",
                "התקרה שבתקנות דמי הניהול: קרן השתלמות רשאית לגבות עד השיעור הזה מהצבירה בשנה, ולא לגבות דבר מההפקדות.",
            ))
            .about_fee(FeeKind::Management)
            .source(&regulations),
        ],
    );

    Broker {
        name: t("Study fund", "קרן השתלמות"),
        short_name: t("Study fund", "קרן השתלמות"),
        kind: BrokerKind::Funds,
        new_customer_plan: 0, // Average fee
        compared_at_first: false,
        description: t(
            "A fund that a management company invests for you (קרן\u{a0}השתלמות), and the one \
             way to save for a few years whose gains aren't taxed. The self-employed can open \
             one and deposit as they like. An employee's needs the employer, who pays most of \
             it, so it isn't a choice against a broker: the app counts a self-employed \
             saver's.",
            "קרן שחברה מנהלת משקיעה עבורכם, ואפיק החיסכון היחיד לכמה שנים שהרווחים בו פטורים ממס. עצמאים יכולים לפתוח קרן ולהפקיד כרצונם. קרן של שכיר תלויה במעסיק, שמפקיד את רוב הכסף, ולכן היא לא חלופה לחשבון מסחר: האפליקציה מחשבת קרן של חוסך עצמאי.",
        ),
        tariff_date: Some(DATA_OF),
        source_url: Some(gemel_net.url.clone()),
        caveats: vec![
            Caveat::published(t(
                "Six years after the first deposit the money can be taken out for anything. \
                 The gains on up to ₪20,566 deposited a year (2026) aren't taxed; the gains on \
                 what's deposited beyond that are, by a quarter of what they are beyond the \
                 rise in prices, as at a broker.",
                "שש שנים אחרי ההפקדה הראשונה אפשר למשוך את הכסף לכל מטרה. הרווחים על עד ₪20,566 שהופקדו בשנה (2026) פטורים ממס; הרווחים על מה שהופקד מעבר לכך חייבים במס, רבע ממה שמעבר לעליית המחירים, כמו בחשבון מסחר.",
            ))
            .source(&kol_zchut_study_fund_self_employed()),
            Caveat::not_counted(t(
                "The self-employed may deduct deposits of up to 4.5% of their income, and up \
                 to ₪13,203 a year (2026), from the income they're taxed on. That benefit isn't \
                 counted, so the fund may be worth more than shown.",
                "עצמאים רשאים לנכות מההכנסה החייבת במס הפקדות של עד 4.5% מהכנסתם, ועד ₪13,203 בשנה (2026). ההטבה הזאת לא נספרת, ולכן הקרן עשויה להיות שווה יותר ממה שמוצג.",
            ))
            .source(&kol_zchut_study_fund_self_employed()),
            Caveat::not_counted(t(
                "An employee can't open one alone. The employer deposits up to 7.5% of the \
                 salary and the employee up to 2.5%, to ₪18,854 a year together (2026). The \
                 employer's share is money a broker's account wouldn't get, and isn't counted.",
                "שכיר לא יכול לפתוח קרן לבד. המעסיק מפקיד עד 7.5% מהשכר והעובד עד 2.5%, עד ₪18,854 בשנה יחד (2026). חלק המעסיק הוא כסף שחשבון מסחר לא היה מקבל, והוא לא נספר.",
            ))
            .source(&kol_zchut_study_fund()),
            earns_what_the_security_does(t("The fund", "הקרן")),
            investing_costs_arent_counted(),
        ],
        plans: vec![average, cheapest, dearest, legal_maximum],
    }
}

// ─────────────────────────── Savings policy ───────────────────────────

/// Polisat Hisachon. The regulator reports the insurers' investment
/// policies together, savings policies and managers' insurance alike, so
/// the average is a reading: Bituach Net's report for August 2026, policies
/// sold since 2004 (168 tracks of 8 insurers), weighted by assets.
#[must_use]
#[allow(clippy::too_many_lines, reason = "a fund's data")]
pub fn savings_policy() -> Broker {
    let bituach_net = bituach_net();
    let menora = menora_comparison();
    let bizportal = bizportal_comparison();

    let average = managed(
        t("Average fee", "דמי ניהול ממוצעים"),
        t(
            "What the insurers' investment policies take on average. Savings policies aren't \
             reported apart from the insurers' other policies, so this is the nearest \
             published figure: see the note beside the fee.",
            "מה שפוליסות ההשקעה של חברות הביטוח גובות בממוצע. פוליסות חיסכון לא מדווחות בנפרד משאר הפוליסות של חברות הביטוח, ולכן זה הנתון המפורסם הקרוב ביותר: ראו את ההערה ליד דמי הניהול.",
        ),
        Vehicle::SavingsPolicy,
        fee(dec!(0.94), dec!(0)),
        vec![
            Caveat::reading(
                t(
                    "The average of all the insurers' investment policies sold since 2004, by \
                     the Capital Market Authority's data of August 2026. They include \
                     managers' insurance (ביטוח\u{a0}מנהלים), a pension product, and savings \
                     policies aren't reported apart.",
                    "הממוצע של כל פוליסות ההשקעה של חברות הביטוח שנמכרו מאז 2004, לפי נתוני רשות שוק ההון לאוגוסט 2026. הן כוללות ביטוח מנהלים, שהוא מוצר פנסיוני, ופוליסות חיסכון לא מדווחות בנפרד.",
                ),
                t(
                    "a savings policy is invested in the same tracks, and no figure of its \
                     own is published",
                    "פוליסת חיסכון מושקעת באותם מסלולי השקעה, ולא מתפרסם נתון משלה",
                ),
            )
            .about_fee(FeeKind::Management)
            .source(&bituach_net),
            Caveat::reading(
                t(
                    "Those policies also take 1.77% of deposits on average, where they report \
                     it. None is counted for a savings policy.",
                    "הפוליסות האלה גובות גם 1.77% מההפקדות בממוצע, היכן שזה מדווח. בפוליסת חיסכון זה לא נספר.",
                ),
                t(
                    "Menora Mivtachim describes the fee on the balance as a savings policy's \
                     only cost",
                    "מנורה מבטחים מתארת את דמי הניהול מהצבירה כעלות היחידה של פוליסת חיסכון",
                ),
            )
            .about_fee(FeeKind::DepositFee)
            .sources(&[&menora, &bituach_net]),
        ],
    );
    let highest = managed(
        t("Highest fee", "דמי הניהול המרביים"),
        t(
            "The most a savings policy takes from the balance, as the insurers state it. \
             Menora Mivtachim adds that most savers pay far less.",
            "המקסימום שפוליסת חיסכון גובה מהצבירה, כפי שחברות הביטוח מציגות אותו. מנורה מבטחים מוסיפה שרוב החוסכים משלמים הרבה פחות.",
        ),
        Vehicle::SavingsPolicy,
        fee(dec!(2), dec!(0)),
        vec![
            Caveat::reading(
                t(
                    "2% of the balance a year is given as the most a savings policy takes. \
                     The regulation it rests on wasn't checked.",
                    "2% מהצבירה בשנה מוצגים כמקסימום שפוליסת חיסכון גובה. התקנה שזה נשען עליה לא נבדקה.",
                ),
                t(
                    "Menora Mivtachim's comparison and Bizportal's both give it",
                    "כך בהשוואה של מנורה מבטחים וגם בזו של ביזפורטל",
                ),
            )
            .about_fee(FeeKind::Management)
            .sources(&[&menora, &bizportal]),
        ],
    );

    Broker {
        name: t("Savings policy", "פוליסת חיסכון"),
        short_name: t("Savings policy", "פוליסת חיסכון"),
        kind: BrokerKind::Funds,
        new_customer_plan: 0, // Average fee
        compared_at_first: false,
        description: t(
            "An insurer invests the money for you in an investment track you choose \
             (פוליסת\u{a0}חיסכון), much like a provident fund for investment. Despite the \
             name, it isn't insurance. There's no ceiling on deposits and no pension at the \
             end, and moving to another insurer means taking the money out and paying the \
             tax.",
            "חברת ביטוח משקיעה עבורכם את הכסף במסלול השקעה שתבחרו, בדומה לקופת גמל להשקעה. למרות השם, אין בה ביטוח. אין תקרת הפקדה ואין קצבה בסוף, ומעבר לחברת ביטוח אחרת מחייב למשוך את הכסף ולשלם את המס.",
        ),
        tariff_date: Some(DATA_OF),
        source_url: Some(bituach_net.url.clone()),
        caveats: vec![
            Caveat::published(t(
                "A quarter of the gain beyond the rise in prices is tax when the money is \
                 taken out, as at a broker, at any age and in any way: there's no tax-free \
                 pension.",
                "רבע מהרווח שמעבר לעליית המחירים הוא מס כשמושכים את הכסף, כמו בחשבון מסחר, בכל גיל ובכל צורת משיכה: אין קצבה פטורה ממס.",
            ))
            .source(&bizportal),
            earns_what_the_security_does(t("The policy", "הפוליסה")),
            Caveat::not_counted(t(
                "Moving between investment tracks isn't taxed. At a broker, selling one \
                 security to buy another is. The app never switches, so this isn't counted.",
                "מעבר בין מסלולי השקעה לא מחויב במס. בחשבון מסחר, מכירת נייר ערך כדי לקנות אחר כן מחויבת. האפליקציה אף פעם לא מחליפה, ולכן זה לא נספר.",
            ))
            .source(&analyst_comparison()),
        ],
        plans: vec![average, highest],
    }
}
