//! The brokers' tariffs, entered from the documents in `policies` (see
//! `policies/sources.md` for where each number comes from). Eventually this
//! belongs in one JSON file per broker (the types already serialize); it's
//! Rust for now so the compiler checks it.
//!
//! Investment houses publish a full tariff, the most they may charge, and
//! offer new customers far less. Where that offer isn't published, a plan
//! named "Typical offer" holds what comparison sites list for joining, and
//! the full tariff wherever the offer doesn't say.
//!
//! Every plan spells out all its fields: nothing here comes from a default,
//! since a defaulted fee would read as free. Every caveat is made with the
//! constructor that says how sure it is ([`Caveat::published`],
//! [`Caveat::reading`], [`Caveat::at_most`], [`Caveat::may_cost_more`],
//! [`Caveat::not_counted`]), and names the fee it's about where it's about
//! one.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use crate::Exchange::{self, Europe, Tlv, Usa};
use crate::Security::{self, Bond, Etf, IndexFund, Stock};
use crate::{
    Broker, BrokerKind, Caveat, ConversionFee, CustodyFee, FeeKind, HandlingFee, Markup, Money,
    Page, Percent, PercentFee, Period, Plan, Price, TariffDate, Text, Track, TradeFee, Vehicle,
    ils, iso, usd,
};

/// Every broker the app knows, in the order to offer them.
#[must_use]
pub fn all() -> Vec<Broker> {
    vec![
        altshuler(),
        leumi(),
        excellence(),
        ibi(),
        interactive(),
        meitav(),
        mizrahi(),
        otsar_hahayal(),
    ]
}

/// When the numbers were last checked against the brokers' documents and
/// sites.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 09 - 29)
}

// ─────────────────────────── Helpers ───────────────────────────

/// A trade row: `price` for `securities` on `exchanges` (empty means all).
fn trade(securities: &[Security], exchanges: &[Exchange], price: Price) -> TradeFee {
    TradeFee {
        securities: securities.to_vec(),
        exchanges: exchanges.to_vec(),
        price,
    }
}

/// "`percent`%, min `min`"
fn percent(percent: Decimal, min: Option<Money>) -> Price {
    Price::Percent {
        percent: Percent(percent),
        min,
        max: None,
    }
}

/// "`per_share` per share, min `min`"
fn per_share(per_share: Money, min: Money) -> Price {
    Price::PerShare {
        per_share,
        min: Some(min),
        max: None,
    }
}

/// Custody on everything: `percent` a `per`, charged every `billed`.
fn custody(percent: Decimal, per: Period, billed: Period, min: Option<Money>) -> CustodyFee {
    CustodyFee {
        securities: vec![],
        exchanges: vec![],
        percent: Percent(percent),
        per,
        billed,
        min,
    }
}

/// A handling fee of `per_month` shekels, free for `free_months`.
fn handling(per_month: Decimal, free_months: u32) -> HandlingFee {
    HandlingFee {
        per_month: ils(per_month),
        free_months,
        less_trade_fees: false,
    }
}

/// The same, taken off by the month's trade fees: the typical offers.
fn handling_less_trade_fees(per_month: Decimal, free_months: u32) -> HandlingFee {
    HandlingFee {
        less_trade_fees: true,
        ..handling(per_month, free_months)
    }
}

fn eur(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::EUR)
}

/// A text in both languages, short enough to write beside every number.
fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// A page a number comes from, to link beside its caveat.
fn source(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

/// The exchange's calculator of its members' tariffs and actual average fees.
#[must_use]
pub fn exchange_calculator() -> Page {
    source(
        t(
            "The Tel Aviv Stock Exchange's fee calculator",
            "מחשבון העמלות של הבורסה לניירות ערך בתל אביב",
        ),
        "https://market.tase.co.il/he/market_data/trading_fees",
    )
}

/// gemeltop.co.il's comparison of the investment houses' offers.
#[must_use]
pub fn gemeltop_comparison() -> Page {
    source(
        t(
            "gemeltop.co.il's comparison of trading accounts",
            "השוואת חשבונות המסחר באתר gemeltop.co.il",
        ),
        "https://gemeltop.co.il/hashvaat-batei-hashkaot-mschar-atzmai/",
    )
}

/// A broker's offer on gemeltop.co.il, at `slug`.
fn gemeltop(slug: &str) -> Page {
    source(
        t(
            "The offer on gemeltop.co.il",
            "מבצע ההצטרפות באתר gemeltop.co.il",
        ),
        &format!("https://gemeltop.co.il/{slug}/"),
    )
}

/// tradingil.co.il's comparison of trading accounts.
#[must_use]
pub fn tradingil_comparison() -> Page {
    source(
        t(
            "tradingil.co.il's comparison of trading accounts",
            "השוואת חשבונות המסחר באתר tradingil.co.il",
        ),
        "https://tradingil.co.il/%D7%97%D7%A9%D7%91%D7%95%D7%9F-%D7%9E%D7%A1%D7%97%D7%A8-\
         %D7%A2%D7%A6%D7%9E%D7%90%D7%99/",
    )
}

/// A broker's offer on tradingil.co.il, at `slug`.
fn tradingil(slug: &str) -> Page {
    source(
        t(
            "The offer on tradingil.co.il",
            "מבצע ההצטרפות באתר tradingil.co.il",
        ),
        &format!("https://tradingil.co.il/{slug}/"),
    )
}

/// tradingil.co.il's comparison of what converting currency costs at each.
#[must_use]
pub fn tradingil_conversions() -> Page {
    source(
        t(
            "tradingil.co.il on conversion costs",
            "tradingil.co.il על עלויות המרת מט״ח",
        ),
        "https://tradingil.co.il/%D7%94%D7%9E%D7%A8%D7%AA-%D7%9E%D7%98%D7%97/",
    )
}

/// broker.co.il's comparison of Meitav Trade and IBI Trade.
#[must_use]
pub fn broker_co_il() -> Page {
    source(
        t(
            "broker.co.il's comparison of Meitav and IBI",
            "השוואת מיטב ו-IBI באתר broker.co.il",
        ),
        "https://www.broker.co.il/blog/?ContentID=66316",
    )
}

/// No conversion fee, only a markup.
fn markup_only(markup: Markup) -> ConversionFee {
    ConversionFee {
        fee: PercentFee::FREE,
        or_if_less: None,
        markup,
    }
}

/// What a typical-offer plan's caveats begin with: where its numbers come
/// from (`offer_pages`), and why they can be trusted.
fn typical_offer_caveat(broker: &Text, sites: &str, offer_pages: &[&Page]) -> Caveat {
    Caveat::reading(
        Text::owned(
            format!(
                "Not published by {}: the terms comparison sites list for joining \
                 ({sites}, September 2026). Ask for them when you open the account.",
                broker.en
            ),
            format!(
                "לא פורסם על ידי {}: התנאים שאתרי ההשוואה מציגים למצטרפים ({sites}, ספטמבר 2026). בקשו אותם בפתיחת החשבון.",
                broker.he
            ),
        ),
        t(
            "the Tel Aviv Stock Exchange's actual average fees for June 2026 match them: \
             0.07%–0.085% on Tel Aviv stocks, against full tariffs of 0.15%–0.4%",
            "העמלות הממוצעות בפועל שהבורסה בתל אביב מפרסמת ליוני 2026 תואמות אותם: 0.07%–0.085% על מניות בתל אביב, לעומת תעריפונים מלאים של 0.15%–0.4%",
        ),
    )
    .sources(offer_pages)
    .source(&exchange_calculator())
}

/// Why a typical offer prices some securities as the full tariff does.
fn not_in_the_offer(text: Text) -> Caveat {
    Caveat::at_most(text).about_fee(FeeKind::Trade)
}

/// Why a typical offer's handling fee is taken off by the month's trade fees,
/// and `support`: which site says so, with the pages.
fn handling_less_trade_fees_caveat(support: Text, sources: &[&Page]) -> Caveat {
    Caveat::reading(
        t("The month's trade fees are taken off the ₪15, so a month that pays ₪15 or more \
         in them pays nothing for the account.", "עמלות המסחר של החודש מקוזזות מה-₪15, כך שבחודש שבו שולמו ₪15 או יותר בעמלות מסחר לא משלמים על החשבון דבר."),
        support,
    )
    .about_fee(FeeKind::Account)
    .sources(sources)
}

/// Why a typical offer charges nothing for the account but its monthly fee,
/// on what the comparison sites say (`sources`: their pages, and the
/// broker's if it says so too).
fn only_the_monthly_fee_in_the_offer(text: Text, sources: &[&Page]) -> Caveat {
    Caveat::reading(
        text,
        t("the comparison sites list a holding cost wherever there is one (\u{201c}plus \
         custody\u{201d} for the banks), and the ₪15 a month is the only one they list \
         for the offer", "אתרי ההשוואה מציינים עלות החזקה בכל מקום שיש כזו (״בתוספת דמי משמרת״ אצל הבנקים), ו-₪15 לחודש הם העלות היחידה שהם מציינים למבצע"),
    )
    .about_fee(FeeKind::Account)
    .sources(sources)
}

// ─────────────────────────── Altshuler Shaham Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn altshuler() -> Broker {
    let offer = source(
        t("Altshuler's joining offer", "מבצע ההצטרפות של אלטשולר"),
        "https://www.as-invest.co.il/trade/trading_benefits/",
    );
    let rules = source(
        t("The offer's rules (PDF)", "תקנון המבצע (PDF)"),
        "https://www.as-invest.co.il/media/p1pb1i2x/\
         %D7%AA%D7%A7%D7%A0%D7%95%D7%9F-%D7%90%D7%9C%D7%98%D7%A9%D7%95%D7%9C%D7%A8-\
         %D7%A9%D7%97%D7%9D-%D7%98%D7%A8%D7%99%D7%99%D7%93.pdf",
    );
    let faq = source(
        t("Altshuler's FAQ", "שאלות ותשובות באתר אלטשולר"),
        "https://www.as-invest.co.il/trade/faq/",
    );
    let currency_page = source(
        t("Altshuler's currency page", "עמוד המט״ח באתר אלטשולר"),
        "https://www.as-invest.co.il/trade/currency_exchange/",
    );
    let full_tariff = Plan {
        name: t("Full tariff", "תעריפון מלא"),
        description: t("Altshuler's published price list. For US stocks and ETFs you choose \
                      one of three tracks when opening the account; the app uses the \
                      cheapest for your inputs.", "התעריפון שאלטשולר מפרסמת. למניות וקרנות סל בארה״ב בוחרים אחת משלוש שיטות חיוב בפתיחת החשבון; המחשבון משתמש בזולה ביותר עבורכם."),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            // ETFs and funds share the tariff's funds row, but ETFs have a lower minimum.
            trade(&[Etf], &[Tlv], percent(dec!(0.15), Some(ils(dec!(3.5))))),
            trade(
                &[IndexFund],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(16)))),
            ),
            trade(
                &[Stock, Bond],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(3.5)))),
            ),
            // The tariff's "foreign bonds and funds" row; only the US, since
            // its site says it trades on the Israeli and US exchanges only.
            trade(
                &[Bond, IndexFund],
                &[Usa],
                percent(dec!(0.3), Some(usd(dec!(24)))),
            ),
        ],
        // Tariff 4(a)(7)(a): US stocks and ETFs on one of three tracks.
        tracks: vec![
            Track {
                name: t("1¢ a share", "1¢ למניה"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(9))),
                )],
            },
            Track {
                name: t("$11 per order", "$11 לפקודה"),
                trading: vec![trade(&[Stock, Etf], &[Usa], Price::Flat(usd(dec!(11))))],
            },
            Track {
                name: t("0.15%", "0.15%"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.15), Some(usd(dec!(9)))),
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // A year's rate, as the Tel Aviv Stock Exchange lists it.
        custody: vec![custody(
            dec!(0.15),
            Period::Year,
            Period::Month,
            Some(ils(dec!(75))),
        )],
        conversion: markup_only(Markup::UpTo(Percent(dec!(0.7)))),
        // Part 9: "periodic management fees, up to ₪80 a month".
        handling: Some(handling(dec!(80), 0)),
        management: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::at_most(
                t("A \"periodic management fee\" of up to ₪80 a month is listed, without saying \
                 when it applies; the full ₪80 is assumed.", "״דמי ניהול תקופתיים״ של עד ₪80 לחודש מופיעים בתעריפון בלי לומר מתי הם חלים; המחשבון לוקח את מלוא ה-₪80."),
            )
            .about_fee(FeeKind::Account),
            Caveat::reading(
                t("The share of your holdings is 0.15% a year, charged monthly.", "דמי המשמרת הם 0.15% לשנה, ונגבים חודשית."),
                t("the row doesn't say what period the 0.15% is for; the exchange's actual \
                 averages for June 2026 (0.15% a year for foreign holdings) confirm a year", "השורה לא אומרת לאיזו תקופה מתייחסים 0.15%; הממוצעים בפועל של הבורסה ליוני 2026 (0.15% לשנה על החזקות בחו״ל) מאשרים שנה"),
            )
            .about_fee(FeeKind::Account)
            .source(&exchange_calculator()),
        ],
    };

    // The offer for new customers (trading_benefits page and its January 2026
    // rules). Anything it doesn't mention is priced as the full tariff,
    // which its fine print refers to.
    let new_customers = Plan {
        name: t("New customers", "לקוחות חדשים"),
        description: t(
            "Altshuler's offer for opening a new account with at least ₪5,000. It has no \
             end date.",
            "מבצע ההצטרפות של אלטשולר לפתיחת חשבון חדש עם ₪5,000 לפחות. אין לו תאריך סיום.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: [
            vec![
                // The offer names continuously-traded ETFs (במסלול רציף), which
                // is how Tel Aviv ETFs trade.
                trade(
                    &[Stock, Etf, IndexFund],
                    &[Tlv],
                    percent(dec!(0.07), Some(ils(dec!(2.9)))),
                ),
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(6))),
                ),
            ],
            full_tariff.trading.clone(),
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion: full_tariff.conversion.clone(),
        handling: None,
        management: None,
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            Caveat::published(
                t("The 0.07% is for index funds (קרנות\u{a0}מחקות); active and money-market \
                 funds cost nothing to trade.", "ה-0.07% חל על קרנות מחקות; קרנות אקטיביות וקרנות כספיות נסחרות ללא עמלה."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&offer),
            not_in_the_offer(
                t("The offer doesn't mention bonds, so the full tariff's prices are assumed.", "המבצע לא מזכיר אג״ח, ולכן נלקחו מחירי התעריפון המלא."),
            )
            .about(&[Bond]),
            not_in_the_offer(
                t("The offer doesn't mention funds abroad, so the full tariff's prices are \
                 assumed.", "המבצע לא מזכיר קרנות בחו״ל, ולכן נלקחו מחירי התעריפון המלא."),
            )
            .about(&[IndexFund])
            .on(&[Usa]),
            Caveat::reading(
                t("\"Management fees\" are waived with no end date: taken to cover both \
                 the share of your holdings and the monthly fee.", "המבצע מבטל ״דמי ניהול״ ללא הגבלת זמן, והבנו שזה כולל גם את דמי המשמרת וגם את דמי הניהול החודשיים."),
                t("the tariff calls both of them management fees (custody is \
                 \u{201c}דמי\u{a0}ניהול/טיפול\u{a0}פקדון\u{201d}, the monthly fee \
                 \u{201c}דמי\u{a0}ניהול\u{a0}תקופתיים\u{201d}), the offer's rules waive \
                 \u{201c}account management fees\u{201d} without distinguishing, and the \
                 exchange's actual averages for June 2026 show Altshuler's customers paying \
                 0% custody", "התעריפון קורא לשניהם דמי ניהול (דמי המשמרת הם ״דמי ניהול/טיפול פקדון״, והחודשיים ״דמי ניהול תקופתיים״), תקנון המבצע מבטל ״דמי ניהול חשבון״ בלי להבחין ביניהם, והממוצעים בפועל של הבורסה ליוני 2026 מראים שלקוחות אלטשולר משלמים 0% דמי משמרת"),
            )
            .about_fee(FeeKind::Account)
            .source(&offer)
            .source(&rules)
            .source(&exchange_calculator()),
            Caveat::published(
                t("The offer needs an active account with at least ₪5,000 in it; after a year \
                 without activity its benefits may be withdrawn.", "המבצע דורש חשבון פעיל עם ₪5,000 לפחות; אחרי שנה ללא פעילות ייתכן שההטבות יבוטלו."),
            )
            .source(&rules),
            Caveat::not_counted(t("The ₪200 gift for opening an account isn't included.", "מתנת ₪200 על פתיחת חשבון לא נכללה."))
                .source(&offer),
        ],
    };

    Broker {
        name: t("Altshuler Shaham Trade", "אלטשולר שחם טרייד"),
        short_name: t("Altshuler", "אלטשולר"),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // New customers
        compared_at_first: true,
        description: t("An investment house's trading platform, not a bank. Its full tariff \
                      has three tracks for US stocks and ETFs; new customers get a cheaper \
                      offer instead.", "פלטפורמת המסחר של בית השקעות, לא בנק. בתעריפון המלא שלו שלוש שיטות חיוב למניות וקרנות סל בארה״ב; לקוחות חדשים מקבלים במקום זאת מבצע זול יותר."),
        tariff_date: None, // not stated in the PDF
        source_url: Some(
            "https://www.as-invest.co.il/media/oqdfpafo/\
             %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%9E%D7%9C%D7%90-\
             %D7%90%D7%9C%D7%98%D7%A9%D7%95%D7%9C%D7%A8-%D7%A9%D7%97%D7%9D-\
             %D7%98%D7%A8%D7%99%D7%99%D7%93.pdf"
                .into(),
        ),
        caveats: vec![
            Caveat::reading(
                t("The tariff says the conversion markup is up to 0.7%, and 0.7% is what it \
                 charges: no fee, and the rate shown when converting includes the markup, \
                 as its site says.", "התעריפון אומר שמרווח ההמרה הוא עד 0.7%, ו-0.7% הוא מה שנגבה בפועל: ללא עמלה, והשער המוצג בהמרה כולל את המרווח, כפי שאתר החברה אומר."),
                t("tradingil.co.il's comparison of conversion costs lists Altshuler's markup \
                 as 0.7% (September 2026), and Altshuler's currency page says the shown \
                 rate is final, the market's rate plus the markup, with no fee", "השוואת עלויות ההמרה באתר tradingil.co.il מציינת את המרווח של אלטשולר כ-0.7% (ספטמבר 2026), ועמוד המט״ח של אלטשולר אומר שהשער המוצג סופי: שער השוק בתוספת המרווח, ללא עמלה"),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&tradingil_conversions())
            .source(&currency_page),
            Caveat::published(
                t("Altshuler trades on the Israeli and US exchanges only, as its site says: \
                 nothing in Europe is offered.", "אלטשולר מאפשרת מסחר רק בבורסות בישראל ובארה״ב, כפי שכתוב באתר שלה: באירופה אין מסחר."),
            )
            .about_fee(FeeKind::Trade)
            .on(&[Europe])
            .source(&faq),
        ],
        plans: vec![full_tariff, new_customers],
    }
}

// ─────────────────────────── Bank Leumi ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn leumi() -> Broker {
    let tariff_url =
        "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf";
    let tariff = source(t("Tariff (PDF)", "תעריפון (PDF)"), tariff_url);
    let rates_page = source(
        t("Leumi's exchange rates", "שערי החליפין של לאומי"),
        "https://www.bankleumi.co.il/vgnprod/shearim.asp",
    );
    let pepper_site = source(t("Pepper's site", "אתר פפר"), "https://www.pepper.co.il/");
    let pepper_package = source(
        t(
            "Pepper's package, as Leumi announced it",
            "חבילת פפר, כפי שלאומי הודיע עליה",
        ),
        "https://www.leumi.co.il/he/node/2577",
    );
    let broker_caveats = vec![
        Caveat::reading(
            t("Leumi converts at its published transfers-and-checks rate, about 0.9% from \
             the representative rate each way: a conversion markup of up to 0.9%, on top \
             of the fee. A rate agreed live in the app during trading hours may be a \
             little better.", "לאומי ממיר לפי שער ההעברות והשקים שהוא מפרסם, כ-0.9% מהשער היציג לכל כיוון: מרווח המרה של עד 0.9%, בנוסף לעמלה. שער שנסגר באפליקציה בזמן אמת בשעות המסחר עשוי להיות מעט טוב יותר."),
            t("Leumi's exchange rates for 28 September 2026: it bought dollars at ₪3.0395 \
             and sold them at ₪3.0960 against a representative ₪3.0660 (0.86% under and \
             0.98% over), and euros at ₪3.4575 and ₪3.5218 against ₪3.4877", "שערי לאומי ל-28 בספטמבר 2026: קנה דולרים ב-₪3.0395 ומכר ב-₪3.0960 לעומת שער יציג של ₪3.0660 (0.86% מתחת ו-0.98% מעל), ואירו ב-₪3.4575 ו-₪3.5218 לעומת ₪3.4877"),
        )
        .about_fee(FeeKind::Markup)
        .on(&[Usa, Europe])
        .source(&rates_page),
        Caveat::published(t("Active (non-index) mutual funds on Tel Aviv have no trade fee.", "קרנות נאמנות אקטיביות (לא מחקות) בתל אביב פטורות מעמלת קנייה ומכירה."))
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv]),
    ];
    let quarterly = |exchanges: &[Exchange], rate: Decimal| CustodyFee {
        exchanges: exchanges.to_vec(),
        ..custody(rate, Period::Quarter, Period::Quarter, None)
    };

    // Measured from the bank's published rates (the caveat above).
    let markup = Markup::UpTo(Percent(dec!(0.9)));
    // Online prices (Nispach Heh): Leumi Trade, the website and the app. The
    // branch prices in the main tables are higher.
    let online_conversion = PercentFee {
        percent: Percent(dec!(0.16)),
        min: Some(usd(dec!(5.76))),
        max: Some(usd(dec!(2400))),
    };
    let online = Plan {
        name: t("Online", "אונליין"),
        description: t(
            "Leumi's standard prices for trading yourself in Leumi Trade, on the \
                      website or in the app (the tariff's appendix on direct channels). \
                      Trading through a banker at a branch costs more.",
            "המחירים הרגילים של לאומי למסחר עצמאי בלאומי טרייד, באתר או באפליקציה (נספח הערוצים הישירים בתעריפון). מסחר דרך בנקאי בסניף עולה יותר.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            // Tariff 4(a)(1) excludes mutual funds, but its footnote puts
            // index-tracking funds (Keren Mechaka) and ETFs back in. Active
            // funds carry no trade fee, at most a distribution fee in some
            // cases (4(b)(2)).
            trade(
                &[],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.4)),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            ),
            trade(
                &[],
                &[Usa, Europe],
                Price::Percent {
                    percent: Percent(dec!(0.3)),
                    min: Some(usd(dec!(24))),
                    max: Some(usd(dec!(6750))),
                },
            ),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![
            quarterly(&[Tlv], dec!(0.15)),
            quarterly(&[Usa, Europe], dec!(0.2)),
        ],
        conversion: ConversionFee {
            fee: online_conversion,
            or_if_less: None,
            markup,
        },
        handling: None,
        management: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![],
    };

    // Customer group from Nispach Alef. Only the parts that differ from Online.
    let plus18 = Plan {
        name: t("Online, 'Leumi 18+'", "אונליין, 'לאומי 18+'"),
        description: t(
            "Online prices with the discounts of 'Leumi 18+', a customer group for young \
             customers. Check your eligibility with the bank.",
            "מחירי אונליין עם ההנחות של 'לאומי 18+', קבוצת לקוחות לצעירים. בדקו את זכאותכם מול הבנק.",
        ),
        // The group's 0.35% on bonds beats online's 0.4%, but comes with the
        // branch's minimum and maximum (₪27, ₪7,000), and benefits don't
        // stack: each fee is the better of the two. The group's rate within
        // online's bounds is never more than ₪1 off that.
        trading: [
            vec![trade(
                &[Bond],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.35)),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            )],
            online.trading.clone(),
        ]
        .concat(),
        custody: vec![
            quarterly(&[Tlv], dec!(0.075)),
            quarterly(&[Usa, Europe], dec!(0.1)),
        ],
        // 50% off the branch fee, but not below its minimum. Benefits don't
        // add up: each fee is the better of the group's and the online one
        // (the tariff's first page), and online is less below $4,500.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(7.2))),
                max: Some(usd(dec!(3000))),
            },
            or_if_less: Some(online_conversion),
            markup,
        },
        caveats: vec![
            Caveat::reading(
                t("Bonds on Tel Aviv cost the group's 0.35%, with Online's ₪26 minimum and \
                 ₪6,300 maximum.", "אג״ח בתל אביב עולות 0.35%, מחיר הקבוצה, עם המינימום (₪26) והמקסימום (₪6,300) של אונליין."),
                t("the group's price is 0.35% with the branch's bounds (at least ₪27, at most \
                 ₪7,000), and the tariff's first page gives each fee the better of the \
                 group's and the online price; the better of those two is within ₪1 of \
                 this on any order", "מחיר הקבוצה הוא 0.35% עם גבולות הסניף (מינימום ₪27, מקסימום ₪7,000), והעמוד הראשון בתעריפון נותן לכל עמלה את הטוב מבין מחיר הקבוצה ומחיר אונליין; הטוב מבין השניים נמצא בטווח של ₪1 מזה בכל פקודה"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond])
            .on(&[Tlv])
            .source(&tariff),
            Caveat::at_most(
                t("Conversion is 50% off the branch fee. Whether its $3,000 maximum is \
                 halved too isn't stated; it's assumed not.", "ההמרה בהנחה של 50% מעמלת הסניף. לא כתוב אם גם המקסימום, $3,000, יורד בחצי; המחשבון מניח שלא."),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            // The maximum only binds above $3M; halved, above $1.5M.
            .when_above(usd(dec!(1_500_000))),
        ],
        ..online.clone()
    };

    // Buying an index fund on Tel Aviv by standing order (tariff 4(a)(1), note
    // 10). Any Online customer can set one up; it's a plan here so it can be
    // compared with buying the same funds by one-off orders. The note is
    // only about buying: selling costs what it does on Online.
    let standing_order = Plan {
        name: t(
            "Online, monthly standing order",
            "אונליין, הוראת קבע חודשית",
        ),
        description: t(
            "Not a separate account: an Online customer buying Tel Aviv index funds by \
             monthly standing order, which the tariff prices lower. Everything else is \
             priced as Online.",
            "לא חשבון נפרד: לקוח אונליין שקונה קרנות מחקות בתל אביב בהוראת קבע חודשית, שהתעריפון מתמחר בזול יותר. כל השאר מתומחר כמו אונליין.",
        ),
        standing_orders: vec![trade(
            &[IndexFund],
            &[Tlv],
            Price::Percent {
                percent: Percent(dec!(0.225)),
                min: Some(ils(dec!(5))),
                max: Some(ils(dec!(6300))),
            },
        )],
        caveats: vec![
            Caveat::published(t(
                "Only buying index funds on Tel Aviv by standing order is cheaper; \
                 everything else costs the same as Online.",
                "רק קניית קרנות מחקות בתל אביב בהוראת קבע זולה יותר; כל השאר עולה כמו באונליין.",
            ))
            .about_fee(FeeKind::StandingOrder),
        ],
        ..online.clone()
    };

    let pepper = Plan {
        name: t("Pepper", "פפר"),
        description: t(
            "Leumi's digital-bank app, with its own, simpler prices: a flat fee for each \
             order.",
            "אפליקציית הבנק הדיגיטלי של לאומי, עם מחירים משלה ופשוטים יותר: עמלה קבועה לכל פקודה.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            TradeFee {
                securities: vec![IndexFund],
                ..online.trading[0].clone()
            },
            trade(&[], &[Tlv], Price::Flat(ils(dec!(4)))),
            trade(&[], &[Usa, Europe], Price::Flat(usd(dec!(4)))),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![
            online.custody[0].clone(),
            quarterly(&[Usa, Europe], dec!(0.15)),
        ],
        // 50% off the branch fee (0.2%, maximum $3,000), minimum $3. Less
        // than the online fee on any amount, so it needs no `or_if_less`.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(3))),
                max: Some(usd(dec!(1500))),
            },
            or_if_less: None,
            markup,
        },
        handling: None,
        management: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::may_cost_more(
                t("₪4 is only stated for orders up to ₪30,000; Pepper's site says larger \
                 orders are priced by the tariff, without saying which row. They're \
                 assumed to cost ₪4 too.", "המחיר ₪4 כתוב רק לפקודות של עד ₪30,000; לפי אתר פפר, פקודות גדולות יותר מתומחרות לפי התעריפון, בלי לומר לפי איזו שורה. המחשבון מניח שגם הן עולות ₪4."),
                t("₪4 is stated only for orders up to ₪30,000", "המחיר ₪4 כתוב רק לפקודות של עד ₪30,000"),
            )
            .about_fee(FeeKind::Trade)
            .on(&[Tlv])
            .when_above(ils(dec!(30_000)))
            .source(&pepper_site),
            Caveat::may_cost_more(
                t("$4 is only stated for orders up to $8,000; Pepper's site says larger \
                 orders are priced by the tariff, without saying which row. They're \
                 assumed to cost $4 too.", "המחיר $4 כתוב רק לפקודות של עד $8,000; לפי אתר פפר, פקודות גדולות יותר מתומחרות לפי התעריפון, בלי לומר לפי איזו שורה. המחשבון מניח שגם הן עולות $4."),
                t("$4 is stated only for orders up to $8,000", "המחיר $4 כתוב רק לפקודות של עד $8,000"),
            )
            .about_fee(FeeKind::Trade)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(8000)))
            .source(&pepper_site),
            Caveat::reading(
                t("Pepper's ₪4 row names stocks, T-bills and bonds; ETFs are taken to be \
                 included.", "שורת ה-₪4 של פפר מונה מניות, מק״מ ואג״ח; הבנו שגם קרנות סל נכללות."),
                t("the tariff's own \u{201c}stocks and bonds\u{201d} row counts ETFs and index \
                 funds in (part 4, footnote 4), and Pepper's row uses the same words", "שורת ״מניות ואג״ח״ של התעריפון עצמו כוללת קרנות סל וקרנות מחקות (חלק 4, הערה 4), ושורת פפר משתמשת באותן מילים"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Etf])
            .on(&[Tlv])
            .source(&tariff),
            Caveat::at_most(
                t("Pepper's ₪4 covers stocks, T-bills and bonds. Index funds on Tel Aviv \
                 aren't mentioned, so they're given the Online price.", "ה-₪4 של פפר חלים על מניות, מק״מ ואג״ח. קרנות מחקות בתל אביב לא מוזכרות, ולכן הן מתומחרות כמו באונליין."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv]),
            Caveat::published(
                t("The share of your holdings is 0.15% a quarter on everything: the tariff's Pepper rate for \
                 foreign holdings, and Online's for Tel Aviv ones. Pepper's site says the \
                 same.", "דמי המשמרת הם 0.15% לרבעון על הכול: שיעור פפר שבתעריפון להחזקות בחו״ל, ושיעור אונליין להחזקות בתל אביב. אתר פפר אומר את אותו הדבר."),
            )
            .about_fee(FeeKind::Account)
            .source(&pepper_site),
            Caveat::not_counted(
                t("Pepper's package of March 2026, for customers who move their salary to \
                 it: a year without trade fees, minimums or currency fees, and a grant of \
                 up to ₪1,500. One year only.", "חבילת פפר ממרץ 2026 ללקוחות שמעבירים אליו את המשכורת: שנה ללא עמלות מסחר, מינימומים או עמלות מט״ח, ומענק של עד ₪1,500. שנה אחת בלבד."),
            )
            .source(&pepper_package),
            Caveat::may_cost_more(
                t("Conversion is 50% off the branch fee with a $3 minimum. Its $1,500 \
                 maximum (half the branch's) is assumed.", "ההמרה בהנחה של 50% מעמלת הסניף, עם מינימום $3. המקסימום, $1,500 (חצי מזה של הסניף), הוא הערכה שלנו."),
                t("the conversion maximum is assumed halved", "הנחנו שגם מקסימום ההמרה יורד בחצי"),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            // The halved maximum only binds above $1.5M.
            .when_above(usd(dec!(1_500_000))),
        ],
    };

    Broker {
        name: t("Bank Leumi", "בנק לאומי"),
        short_name: t("Leumi", "לאומי"),
        kind: BrokerKind::Bank,
        new_customer_plan: 0, // Online
        compared_at_first: true,
        description: t(
            "One of Israel's largest banks. Its securities prices depend on how \
                      you trade (online in Leumi Trade, or at a branch), on customer groups, \
                      and on the Pepper app, so it has several plans here.",
            "מהבנקים הגדולים בישראל. מחירי ניירות הערך שלו תלויים באופן המסחר (אונליין בלאומי טרייד, או בסניף), בקבוצות לקוחות ובאפליקציית פפר, ולכן יש לו כאן כמה מסלולים.",
        ),
        tariff_date: Some(TariffDate::Day(date!(2026 - 06 - 29))),
        source_url: Some(tariff_url.into()),
        caveats: broker_caveats,
        plans: vec![online, plus18, standing_order, pepper],
    }
}

// ─────────────────────────── Excellence Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn excellence() -> Broker {
    let article = source(
        t(
            "Excellence's article on conversion costs",
            "המאמר של אקסלנס על עלויות המרה",
        ),
        "https://www.xnes.co.il/academy/trading/account-fees/",
    );
    // The joining package it advertises, and the rules of its two free years
    // (linked from its disclosure page).
    let package = source(
        t("Excellence's joining package", "חבילת ההצטרפות של אקסלנס"),
        "https://lp.xnes.co.il/bursa_xnes_trade/join-package2/",
    );
    let rules = source(
        t(
            "The two free years' rules (PDF)",
            "תקנון פטור מדמי טיפול לשנתיים (PDF)",
        ),
        "https://www.xnes.co.il/media/fw0l5mrv/management26.pdf",
    );
    let on_gemeltop = gemeltop("excellence-trade-amlot");
    let on_tradingil = tradingil(
        "%D7%91%D7%A8%D7%95%D7%A7%D7%A8-%D7%9C%D7%9E%D7%A1%D7%97%D7%A8-%D7%91%D7%99%D7%A9%D7%A8%D7%90%D7%9C",
    );
    // Row 6, US securities, priced by the trading system the customer uses.
    let us_tracks = |securities: &[Security]| {
        vec![
            Track {
                name: t("3¢ a share", "3¢ למניה"),
                trading: vec![trade(
                    securities,
                    &[Usa],
                    per_share(usd(dec!(0.03)), usd(dec!(8))),
                )],
            },
            Track {
                name: t("$11 per order", "$11 לפקודה"),
                trading: vec![trade(securities, &[Usa], Price::Flat(usd(dec!(11))))],
            },
            Track {
                name: t("0.3% plus the broker's fee", "0.3% ועוד עמלת הברוקר בחו״ל"),
                trading: vec![trade(securities, &[Usa], percent(dec!(0.3), None))],
            },
        ]
    };
    // Row 1 covers every Tel Aviv security but traditional mutual funds,
    // ETFs and index funds (קרנות מחקות) included; row 7 everything outside
    // the US, its minimum in the trade's currency.
    let tel_aviv = |securities: &[Security]| {
        trade(securities, &[Tlv], percent(dec!(0.4), Some(ils(dec!(10)))))
    };
    let outside_us = trade(&[], &[Europe], percent(dec!(0.4), Some(eur(dec!(35)))));

    let full_tariff = Plan {
        name: t("Full tariff", "תעריפון מלא"),
        description: t(
            "Excellence's published maximum prices. US trades are priced by the trading \
             system you choose; the app uses the cheapest for your inputs.",
            "מחירי המקסימום שאקסלנס מפרסמת. עסקאות בארה״ב מתומחרות לפי מערכת המסחר שבוחרים; המחשבון משתמש בזולה ביותר עבורכם.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![tel_aviv(&[]), outside_us.clone()],
        tracks: us_tracks(&[]),
        standing_orders: vec![],
        standing_order_conversion: None,
        // Row 8: 0.6% a quarter, at least ₪40 a month.
        custody: vec![custody(
            dec!(0.6),
            Period::Quarter,
            Period::Month,
            Some(ils(dec!(40))),
        )],
        // 0.1%, plus a markup the tariff leaves to the customer's agreement
        // and the site puts at 2 agorot a dollar.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup: Markup::PerDollar(ils(dec!(0.02))),
        },
        handling: Some(handling(dec!(99), 0)),
        management: None,
        fractions_on: vec![Usa],
        min_first_deposit: None,
        caveats: vec![
            Caveat::not_counted(
                t("US trades cost at most 2% of their value; the 0.3% track adds the broker's \
                 fee, which isn't included.", "עסקאות בארה״ב עולות לכל היותר 2% משוויין; שיטת החיוב של 0.3% מוסיפה את עמלת הברוקר בחו״ל, שלא נכללה."),
            )
            .about_fee(FeeKind::Trade)
            .on(&[Usa]),
            Caveat::at_most(
                t("Accounts that hold assets rather than trade actively may be charged 0.1% a \
                 quarter for custody instead; the general 0.6% is used.", "חשבונות שמחזיקים נכסים ולא סוחרים באופן פעיל עשויים לשלם במקום זאת דמי משמרת של 0.1% לרבעון; המחשבון לוקח את ה-0.6% הרגילים."),
            )
            .about_fee(FeeKind::Account),
            Caveat::published(
                t("The tariff leaves the conversion markup to each customer's agreement; \
                 Excellence's site puts it at 2 agorot a dollar, at most ₪200 on $10,000, \
                 which is used.", "התעריפון משאיר את מרווח ההמרה להסכם עם כל לקוח; אתר אקסלנס מציין 2 אגורות לדולר, לכל היותר ₪200 על $10,000, וזה המחיר שבמחשבון."),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&article),
            Caveat::at_most(
                t("The tariff's 0.1% conversion fee is charged on top of the 2 agorot; \
                 Excellence's site describes the 2 agorot as the whole cost, so the fee \
                 may not apply.", "עמלת ההמרה 0.1% שבתעריפון נגבית בנוסף ל-2 האגורות; אתר אקסלנס מתאר את 2 האגורות כעלות כולה, כך שייתכן שהעמלה לא חלה."),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&article),
        ],
    };

    let typical_offer = Plan {
        name: t("Typical offer", "מבצע הצטרפות"),
        description: t(
            "What new customers are usually offered, as comparison sites list it; where the \
             offer says nothing, the full tariff's price is used.",
            "מה שמוצע בדרך כלל ללקוחות חדשים, לפי אתרי ההשוואה; במה שההצעה לא מזכירה, נלקח המחיר מהתעריפון המלא.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            trade(
                &[Stock, Etf, IndexFund],
                &[Tlv],
                percent(dec!(0.07), Some(ils(dec!(3)))),
            ),
            trade(&[Bond], &[Tlv], percent(dec!(0.06), Some(ils(dec!(3))))),
            trade(
                &[Stock, Etf],
                &[Usa],
                per_share(usd(dec!(0.01)), usd(dec!(6))),
            ),
            outside_us,
        ],
        // The offer doesn't price US bonds and funds: the full tariff's tracks.
        tracks: us_tracks(&[Bond, IndexFund]),
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        // "2 agorot a dollar", as its site states the cost of converting.
        conversion: markup_only(Markup::PerDollar(ils(dec!(0.02)))),
        handling: Some(handling_less_trade_fees(dec!(15), 24)),
        management: None,
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(10000))),
        caveats: vec![
            typical_offer_caveat(
                &t("Excellence", "אקסלנס"),
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::reading(
                t("Index funds cost the offer's 0.07% and bonds 0.06%, at least ₪3 each; \
                 gemeltop.co.il lists only stocks and ETFs.", "קרנות מחקות עולות במבצע 0.07% ואג״ח 0.06%, מינימום ₪3 לכל אחת; gemeltop.co.il מציין רק מניות וקרנות סל."),
                t("tradingil.co.il lists index funds (קרן\u{a0}מחקה) with stocks and ETFs at \
                 0.07%, and bonds at 0.06%, for its joining offer (September 2026)", "tradingil.co.il מציין קרנות מחקות עם מניות וקרנות סל ב-0.07%, ואג״ח ב-0.06%, במבצע ההצטרפות שלו (ספטמבר 2026)"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond, IndexFund])
            .on(&[Tlv])
            .source(&on_tradingil),
            Caveat::published(
                t("Converting costs 2 agorot a dollar and nothing else: Excellence's article \
                 puts it at most ₪200 on $10,000, and its joining package says there's no \
                 currency fee.", "המרה עולה 2 אגורות לדולר ולא יותר: המאמר של אקסלנס מציין לכל היותר ₪200 על $10,000, וחבילת ההצטרפות שלה מציינת שאין עמלת מט״ח."),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&article)
            .source(&package),
            handling_less_trade_fees_caveat(
                t("tradingil.co.il says the handling fee \u{201c}can be offset\u{201d} by trade \
                 fees in its joining offer (September 2026)", "tradingil.co.il אומר שדמי הטיפול ״ניתנים לקיזוז״ מול עמלות מסחר במבצע ההצטרפות שלו (ספטמבר 2026)"),
                &[&on_tradingil],
            ),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .on(&[Europe]),
            Caveat::published(t("Some of its trading systems charge at least $5 instead of $6.", "חלק ממערכות המסחר שלה גובות מינימום $5 במקום $6."))
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa])
                .source(&on_tradingil),
            // Its package page says "no custody" outright, and its rules define
            // the two free years as the monthly fee's: published, not a
            // reading of the comparison sites.
            Caveat::published(
                t("Free for the first two years, then ₪15 a month, as Excellence's joining \
                 package and its rules state; nothing is charged as a share of what you \
                 hold.", "חינם בשנתיים הראשונות, ואז ₪15 לחודש, כפי שחבילת ההצטרפות של אקסלנס והתקנון שלה מציינים; אין דמי משמרת."),
            )
            .about_fee(FeeKind::Account)
            .source(&package)
            .source(&rules),
            Caveat::not_counted(
                t("Some sign-up links offer three free years and a refund of commissions; not \
                 included.", "חלק מקישורי ההצטרפות מציעים שלוש שנים חינם והחזר עמלות; לא נכלל."),
            )
            .about_fee(FeeKind::Account)
            .source(&on_gemeltop)
            .source(&on_tradingil),
        ],
    };

    Broker {
        name: t("Excellence Trade", "אקסלנס טרייד"),
        short_name: t("Excellence", "אקסלנס"),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // Typical offer
        compared_at_first: true,
        description: t(
            "The trading arm of the Phoenix investment house. It publishes only a \
                      full tariff of maximum prices; new customers are usually offered far \
                      less.",
            "זרוע המסחר של בית ההשקעות הפניקס. היא מפרסמת רק תעריפון מלא של מחירי מקסימום; ללקוחות חדשים מוצע בדרך כלל הרבה פחות.",
        ),
        tariff_date: None, // not stated in the PDF
        source_url: Some("https://www.xnes.co.il/media/nwogkoos/taarifon.pdf".into()),
        caveats: vec![],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── IBI ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn ibi() -> Broker {
    let fund_page = source(
        t("IBI's page on funds", "עמוד הקרנות באתר IBI"),
        "https://www.ibi.co.il/solutions/zero-balance-managed-funds/",
    );
    let currency_faq = source(
        t("IBI's currency FAQ", "שאלות ותשובות על מט״ח באתר IBI"),
        "https://www.ibi.co.il/solutions/trading/forms-important-information/foreign-currency-faq/",
    );
    let on_gemeltop = gemeltop("ibi-trade-amlot");
    let on_tradingil = tradingil(
        "%D7%97%D7%A9%D7%91%D7%95%D7%9F-%D7%9E%D7%A1%D7%97%D7%A8-%D7%A2%D7%A6%D7%9E%D7%90%D7%99-ibi",
    );
    // Everything but Tel Aviv stocks and ETFs, as the full tariff prices it.
    let abroad = vec![
        // Foreign funds: "0.275% plus the broker's cost".
        trade(&[IndexFund], &[Usa, Europe], percent(dec!(0.275), None)),
        // Minimums "in the trade's currency".
        trade(&[Bond], &[Usa], percent(dec!(0.225), Some(usd(dec!(20))))),
        trade(
            &[Bond],
            &[Europe],
            percent(dec!(0.225), Some(eur(dec!(20)))),
        ),
        trade(
            &[Stock, Etf],
            &[Europe],
            percent(dec!(0.3), Some(eur(dec!(30)))),
        ),
    ];
    let tel_aviv_funds = trade(
        &[IndexFund],
        &[Tlv],
        percent(dec!(0.08), Some(ils(dec!(5)))),
    );
    let conversion = markup_only(Markup::UpTo(Percent(dec!(0.7))));

    let full_tariff = Plan {
        name: t("Full tariff", "תעריפון מלא"),
        description: t(
            "IBI's published maximum prices. For US stocks and ETFs you choose one of four \
             tracks; the app uses the cheapest for your inputs.",
            "מחירי המקסימום ש-IBI מפרסמת. למניות וקרנות סל בארה״ב בוחרים אחת מארבע שיטות חיוב; המחשבון משתמש בזולה ביותר עבורכם.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: [
            vec![
                trade(
                    &[Stock, Etf, Bond],
                    &[Tlv],
                    percent(dec!(0.15), Some(ils(dec!(3.5)))),
                ),
                tel_aviv_funds.clone(),
            ],
            abroad.clone(),
        ]
        .concat(),
        // US stocks and ETFs, on one of four tracks.
        tracks: vec![
            Track {
                name: t("1¢ a share", "1¢ למניה"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(10))),
                )],
            },
            Track {
                name: t("$14 per order", "$14 לפקודה"),
                trading: vec![trade(&[Stock, Etf], &[Usa], Price::Flat(usd(dec!(14))))],
            },
            Track {
                name: t("0.15%", "0.15%"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.15), Some(usd(dec!(10)))),
                )],
            },
            Track {
                name: t("0.15% + 1¢ a share", "0.15% + 1¢ למניה"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    Price::PercentPlusPerShare {
                        percent: Percent(dec!(0.15)),
                        per_share: usd(dec!(0.01)),
                        min: Some(usd(dec!(6))),
                        max: None,
                    },
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // "0.1% of the account's value a quarter", but its site promises its
        // trading customers no custody "on any fund in the system, index
        // funds included".
        custody: vec![
            CustodyFee {
                securities: vec![IndexFund],
                exchanges: vec![Tlv],
                ..custody(dec!(0), Period::Quarter, Period::Quarter, None)
            },
            custody(dec!(0.1), Period::Quarter, Period::Quarter, None),
        ],
        conversion: conversion.clone(),
        handling: Some(handling(dec!(50), 0)),
        management: None,
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(15000))),
        caveats: vec![
            Caveat::at_most(t(
                "The tariff's prices are maximums (\"up to\"); the ₪50 a month is taken \
                 in full.",
                "מחירי התעריפון הם מקסימום (״עד״); דמי הטיפול נלקחו במלוא ה-₪50.",
            ))
            .about_fee(FeeKind::Account),
            Caveat::published(t(
                "Nothing as a share of what you hold, as IBI's site promises for every fund; \
                 the tariff itself would charge 0.1% a quarter.",
                "ללא דמי משמרת, כפי שאתר IBI מבטיח על כל קרן; התעריפון עצמו היה גובה 0.1% לרבעון.",
            ))
            .about_fee(FeeKind::Account)
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&fund_page),
        ],
    };

    let typical_offer = Plan {
        name: t("Typical offer", "מבצע הצטרפות"),
        description: t(
            "What new customers are usually offered, as comparison sites list it; where the \
             offer says nothing, the full tariff's price is used.",
            "מה שמוצע בדרך כלל ללקוחות חדשים, לפי אתרי ההשוואה; במה שההצעה לא מזכירה, נלקח המחיר מהתעריפון המלא.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: [
            vec![
                trade(
                    &[Stock, Etf, Bond],
                    &[Tlv],
                    percent(dec!(0.08), Some(ils(dec!(2.35)))),
                ),
                tel_aviv_funds,
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(7.5))),
                ),
            ],
            abroad,
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion,
        handling: Some(handling_less_trade_fees(dec!(15), 0)),
        management: None,
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(15000))),
        caveats: vec![
            typical_offer_caveat(
                &t("IBI", "IBI"),
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::reading(
                t("Bonds cost the offer's 0.08% (at least ₪2.35), and index funds 0.08% (at \
                 least ₪5), which is also the full tariff's price for them; gemeltop.co.il \
                 lists only stocks and ETFs.", "אג״ח עולות במבצע 0.08% (מינימום ₪2.35), וקרנות מחקות 0.08% (מינימום ₪5), שזה גם מחיר התעריפון המלא שלהן; gemeltop.co.il מציין רק מניות וקרנות סל."),
                t("tradingil.co.il lists both for its joining offer (September 2026)", "tradingil.co.il מציין את שניהם במבצע ההצטרפות שלו (ספטמבר 2026)"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond, IndexFund])
            .on(&[Tlv])
            .source(&on_tradingil),
            Caveat::reading(
                t("1¢ a share, at least $7.50: gemeltop.co.il states only the minimum.", "1¢ למניה, מינימום $7.50: gemeltop.co.il מציין רק את המינימום."),
                t("the tariff's first US track is 1¢ a share with a minimum, and \
                 tradingil.co.il spells out 1¢ a share with a $7.50 minimum for its joining \
                 offer (September 2026)", "שיטת החיוב הראשונה לארה״ב בתעריפון היא 1¢ למניה עם מינימום, ו-tradingil.co.il מפרט 1¢ למניה עם מינימום $7.50 במבצע ההצטרפות שלו (ספטמבר 2026)"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Stock, Etf])
            .on(&[Usa])
            .source(&on_gemeltop)
            .source(&on_tradingil),
            handling_less_trade_fees_caveat(
                t("tradingil.co.il and broker.co.il say so for IBI's joining offer (2026)", "tradingil.co.il ו-broker.co.il אומרים זאת על מבצע ההצטרפות של IBI (2026)"),
                &[&on_tradingil, &broker_co_il()],
            ),
            Caveat::at_most(
                t("tradingil.co.il's joining offer adds two free years of the handling fee; \
                 gemeltop.co.il's doesn't, so it's charged from the first month.", "מבצע ההצטרפות באתר tradingil.co.il מוסיף שנתיים חינם מדמי הטיפול; זה שבאתר gemeltop.co.il לא, ולכן הם נגבים מהחודש הראשון."),
            )
            .about_fee(FeeKind::Account)
            .source(&on_tradingil)
            .source(&on_gemeltop),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .on(&[Europe]),
            only_the_monthly_fee_in_the_offer(
                t("The account costs only the ₪15 a month the comparison sites list, nothing as \
                 a share of what you hold; IBI's site says so for every fund.", "החשבון עולה רק את ה-₪15 לחודש שאתרי ההשוואה מציינים, בלי דמי משמרת; אתר IBI מציין שאין דמי משמרת על אף קרן."),
                &[&on_gemeltop, &on_tradingil, &fund_page],
            ),
            Caveat::not_counted(t("The ₪300 gift for opening an account isn't included.", "מתנת ₪300 על פתיחת חשבון לא נכללה."))
                .source(&on_gemeltop),
        ],
    };

    Broker {
        name: t("IBI", "IBI"),
        short_name: t("IBI", "IBI"),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // Typical offer
        compared_at_first: true,
        description: t("An investment house's trading platform (IBI TRADE, with the IBI SMART \
                      app). It publishes only a full tariff of maximum prices; new customers \
                      are usually offered far less. Opening an account takes at least \
                      ₪15,000.", "פלטפורמת המסחר של בית השקעות (IBI TRADE, עם אפליקציית IBI SMART). היא מפרסמת רק תעריפון מלא של מחירי מקסימום; ללקוחות חדשים מוצע בדרך כלל הרבה פחות. פתיחת חשבון דורשת ₪15,000 לפחות."),
        tariff_date: Some(TariffDate::Day(date!(2026 - 07 - 01))),
        source_url: Some("https://campaign.ibi.co.il/PDF/TRADE/TAARIFON_IBI.pdf".into()),
        caveats: vec![
            Caveat::reading(
                t("The conversion markup is 0.7%: IBI's site says it charges no fee, only a \
                 spread written in each customer's fee appendix, and works its example at \
                 0.7%.", "מרווח ההמרה הוא 0.7%: אתר IBI אומר שאין עמלה, רק מרווח שנכתב בנספח העמלות של כל לקוח, ומחשב את הדוגמה שלו לפי 0.7%."),
                t("IBI's currency FAQ (2026), and tradingil.co.il's comparison of conversion \
                 costs (September 2026), which lists 0.7% below $15,000", "השאלות והתשובות על מט״ח באתר IBI (2026), והשוואת עלויות ההמרה באתר tradingil.co.il (ספטמבר 2026), שמציינת 0.7% מתחת ל-$15,000"),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&currency_faq)
            .source(&tradingil_conversions()),
            Caveat::at_most(
                t("Conversions of $15,000 or more get a 0.5% markup instead of 0.7%, per \
                 tradingil.co.il; 0.7% is used for all.", "המרות של $15,000 ומעלה מקבלות מרווח של 0.5% במקום 0.7%, לפי tradingil.co.il; המחשבון לוקח 0.7% לכולן."),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(15_000)))
            .source(&tradingil_conversions()),
            Caveat::published(
                t("Managed (active) funds of 12 fund managers cost nothing to trade; index \
                 funds aren't among them.", "קרנות מנוהלות (אקטיביות) של 12 מנהלי קרנות נסחרות ללא עמלה; קרנות מחקות אינן ביניהן."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&fund_page),
            Caveat::not_counted(
                t("Foreign funds cost 0.275% plus the foreign broker's cost, which isn't \
                 included.", "קרנות זרות עולות 0.275% בתוספת עלות הברוקר הזר, שלא נכללה."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Usa, Europe]),
        ],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── Interactive Israel ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn interactive() -> Broker {
    let tariff_url = "https://www.inter-il.com/wp-content/uploads/2026/09/\
                      %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-\
                      23.09.2026-1.pdf";
    let tariff = source(t("Tariff (PDF)", "תעריפון (PDF)"), tariff_url);
    let commission_page = source(
        t(
            "Interactive's commission page",
            "עמוד העמלות באתר אינטראקטיב",
        ),
        "https://www.inter-il.com/commission/",
    );
    let tel_aviv_page = source(
        t(
            "Interactive's Tel Aviv page",
            "עמוד הבורסה בתל אביב באתר אינטראקטיב",
        ),
        "https://www.inter-il.com/israeli-stock-market/",
    );
    let ibkr = source(
        t(
            "Interactive Brokers on spot currencies",
            "אינטראקטיב ברוקרס על המרות מט״ח",
        ),
        "https://www.interactivebrokers.com/en/pricing/commissions-spot-currencies.php",
    );
    let us_shares = trade(
        &[Stock, Etf],
        &[Usa],
        per_share(usd(dec!(0.01)), usd(dec!(2.5))),
    );
    // Converting shekels: 0.002%, at least ₪10, which covers up to ₪500,000,
    // at the live market rate, as its site says.
    let conversion = ConversionFee {
        fee: PercentFee {
            percent: Percent(dec!(0.002)),
            min: Some(ils(dec!(10))),
            max: None,
        },
        or_if_less: None,
        markup: Markup::MarketRate,
    };
    let standard = Plan {
        name: t("Standard", "רגיל"),
        description: t(
            "Interactive Israel's only price list. Opening an account takes no minimum \
             deposit.",
            "התעריפון היחיד של אינטראקטיב ישראל. פתיחת חשבון לא דורשת הפקדת מינימום.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            us_shares.clone(),
            trade(
                &[Stock, Etf],
                &[Europe],
                percent(dec!(0.15), Some(eur(dec!(2.5)))),
            ),
            trade(&[Bond], &[Usa], percent(dec!(0.2), Some(usd(dec!(10))))),
            trade(&[Bond], &[Europe], percent(dec!(0.2), Some(eur(dec!(10))))),
        ],
        tracks: vec![],
        // Its automatic investment plan buys at the usual price and converts
        // for free, at the same unpublished spread.
        standing_orders: vec![us_shares],
        standing_order_conversion: Some(ConversionFee {
            fee: PercentFee::FREE,
            ..conversion.clone()
        }),
        custody: vec![],
        conversion,
        handling: None,
        management: None,
        fractions_on: vec![Usa],
        min_first_deposit: None,
        caveats: vec![
            Caveat::reading(
                t("Its automatic investment plan converts for free; taken as buying every \
                 month, for US stocks and ETFs.", "תוכנית ההשקעה האוטומטית שלה ממירה בחינם; הבנו אותה כקנייה כל חודש, של מניות וקרנות סל בארה״ב."),
                t("the tariff waives the conversion fee for conversions within a fixed \
                 automatic investment plan, which is what buying by standing order is", "התעריפון פוטר מעמלת המרה המרות במסגרת תוכנית השקעה אוטומטית קבועה, שזה בדיוק קנייה בהוראת קבע"),
            )
            .about_fee(FeeKind::StandingOrder)
            .about(&[Stock, Etf])
            .on(&[Usa])
            .source(&tariff),
            Caveat::published(
                t("Fractions of a share are charged as whole shares, as its tariff says.", "שברי מניה מחויבים כמניות שלמות, כפי שהתעריפון אומר."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Stock, Etf])
            .on(&[Usa]),
            Caveat::reading(
                t("Bonds are charged 0.2% of their face value; the trade's value is used \
                 instead.", "העמלה על אג״ח היא 0.2% מהערך הנקוב (הסכום שהאג״ח מחזירה בפירעון); המחשבון מחשב אותה משווי העסקה."),
                t("bonds trade within a few percent of their face value, so the fee is nearly \
                 the same", "אג״ח נסחרות בטווח של אחוזים בודדים מהערך הנקוב, כך שהעמלה כמעט זהה"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond])
            .source(&tariff),
            Caveat::not_counted(
                t("Its tariff's price for mutual funds is unclear, so it isn't included.", "מחיר קרנות הנאמנות בתעריפון לא ברור, ולכן לא נכלל."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund]),
            Caveat::published(t("Tel Aviv securities are traded for institutional clients only.", "ניירות ערך בתל אביב נסחרים ללקוחות מוסדיים בלבד."))
                .about_fee(FeeKind::Trade)
                .on(&[Tlv])
                .source(&tel_aviv_page),
        ],
    };

    Broker {
        name: t("Interactive Israel", "אינטראקטיב ישראל"),
        short_name: t("Interactive", "אינטראקטיב"),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 0, // Standard
        compared_at_first: true,
        description: t("The Israeli introducing broker for Interactive Brokers, run by MEXEM \
                      (regulated in Cyprus, not by the Israel Securities Authority). US and \
                      European exchanges; Tel Aviv only for institutional clients.", "הנציג הישראלי (״ברוקר מציג״) של אינטראקטיב ברוקרס, שמופעל על ידי MEXEM (בפיקוח הרגולטור בקפריסין, לא של רשות ניירות ערך). מסחר בבורסות ארה״ב ואירופה; בתל אביב רק ללקוחות מוסדיים."),
        tariff_date: Some(TariffDate::Day(date!(2026 - 09 - 23))),
        source_url: Some(tariff_url.into()),
        caveats: vec![
            Caveat::published(
                t("Converting is at the live market rate (שער\u{a0}רציף), as its site says, for \
                 up to ₪500,000: the fee is the whole cost the broker adds.", "ההמרה היא לפי שער השוק בזמן אמת (שער רציף), כפי שכתוב באתר, עד ₪500,000: העמלה היא כל מה שאינטראקטיב מוסיפה."),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&commission_page),
            Caveat::not_counted(
                t("The currency market's own bid-ask spread, a few hundredths of a percent: \
                 Interactive Brokers, where the account is held, passes market quotes through \
                 and charges the fee instead of a markup. Its automatic conversions may move \
                 the rate by up to 0.03% instead.", "הפער בין שער הקנייה לשער המכירה בשוק המט״ח עצמו, כמה מאיות האחוז: אינטראקטיב ברוקרס, שבה מתנהל החשבון, מעבירה את שערי השוק כפי שהם, וגובה עמלה במקום מרווח. בהמרות האוטומטיות שלה השער עשוי לזוז בעד 0.03%, במקום העמלה."),
            )
            .on(&[Usa, Europe])
            .source(&ibkr),
            Caveat::reading(
                t("Converting back to shekels is taken to cost the same as converting them: \
                 0.002%, at least ₪10.", "הבנו שההמרה חזרה לשקלים עולה כמו המרת השקלים: 0.002%, מינימום ₪10."),
                t("the tariff prices conversions by the currency traded, and its only note \
                 is on converting shekels; nothing prices the way back differently", "התעריפון מתמחר המרות לפי המטבע הנסחר, וההערה היחידה בו היא על המרת שקלים; שום דבר בו לא קובע מחיר אחר להמרה חזרה"),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&tariff),
        ],
        plans: vec![standard],
    }
}

// ─────────────────────────── Meitav Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn meitav() -> Broker {
    let site = source(
        t("Meitav Trade's site", "אתר מיטב טרייד"),
        "https://www.meitav.co.il/trade/independent_trading/",
    );
    let on_gemeltop = gemeltop("meitav-trade-amlot");
    let on_tradingil = tradingil("%D7%9E%D7%99%D7%98%D7%91-%D7%98%D7%A8%D7%99%D7%99%D7%93");
    // Row 1: every Tel Aviv security but TA options and futures.
    let tel_aviv = |securities: &[Security]| {
        trade(securities, &[Tlv], percent(dec!(0.3), Some(ils(dec!(10)))))
    };
    let abroad = vec![
        trade(
            &[Bond],
            &[Usa, Europe],
            percent(dec!(0.3), Some(usd(dec!(25)))),
        ),
        // Foreign funds, not counting clearing and correspondent fees.
        trade(
            &[IndexFund],
            &[Usa, Europe],
            percent(dec!(0.2), Some(usd(dec!(20)))),
        ),
        // "The world's other exchanges: 0.25%, min 25 in the trade's currency".
        trade(
            &[Stock, Etf],
            &[Europe],
            percent(dec!(0.25), Some(eur(dec!(25)))),
        ),
    ];
    let markup = Markup::UpTo(Percent(dec!(0.7)));

    let full_tariff = Plan {
        name: t("Full tariff", "תעריפון מלא"),
        description: t(
            "Meitav Trade's published maximum prices. For US stocks you choose one of two \
             tracks; the app uses the cheapest for your inputs.",
            "מחירי המקסימום שמיטב טרייד מפרסמת. למניות בארה״ב בוחרים אחת משתי שיטות חיוב; המחשבון משתמש בזולה ביותר עבורכם.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: [vec![tel_aviv(&[])], abroad.clone()].concat(),
        tracks: vec![
            Track {
                name: t("1¢ a share", "1¢ למניה"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(12))),
                )],
            },
            Track {
                name: t("0.7%", "0.7%"),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.7), Some(usd(dec!(12)))),
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // "0.15%, at least ₪270 a quarter": a quarter's rate, as the Tel Aviv
        // Stock Exchange lists it (0.6% a year).
        custody: vec![custody(
            dec!(0.15),
            Period::Quarter,
            Period::Quarter,
            Some(ils(dec!(270))),
        )],
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.13)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup,
        },
        handling: Some(handling(dec!(90), 0)),
        management: None,
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            Caveat::reading(
                t("The share of your holdings is 0.15% a quarter (0.6% a year).", "דמי המשמרת הם 0.15% לרבעון (0.6% לשנה)."),
                t("the row doesn't say what period the 0.15% is for; the exchange's actual \
                 averages for June 2026 (0.6% a year) confirm a quarter", "השורה לא אומרת לאיזו תקופה מתייחסים 0.15%; הממוצעים בפועל של הבורסה ליוני 2026 (0.6% לשנה) מאשרים רבעון"),
            )
            .about_fee(FeeKind::Account)
            .source(&exchange_calculator()),
            Caveat::not_counted(
                t("Foreign funds add clearing and correspondent fees, which aren't included.", "בקרנות זרות יש גם עמלות סליקה ועמלות של בנק מתווך בחו״ל (קורספונדנט), שלא נכללו."),
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Usa, Europe]),
            Caveat::at_most(
                t("The monthly fee is listed as up to ₪90; the full ₪90 is assumed.", "דמי הטיפול מופיעים כ״עד ₪90 לחודש״; המחשבון לוקח את מלוא ה-₪90."),
            )
            .about_fee(FeeKind::Account),
        ],
    };

    let typical_offer = Plan {
        name: t("Typical offer", "מבצע הצטרפות"),
        description: t(
            "What new customers are usually offered, as comparison sites list it; where the \
             offer says nothing, the full tariff's price is used.",
            "מה שמוצע בדרך כלל ללקוחות חדשים, לפי אתרי ההשוואה; במה שההצעה לא מזכירה, נלקח המחיר מהתעריפון המלא.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: [
            vec![
                trade(&[Etf], &[Tlv], percent(dec!(0.07), Some(ils(dec!(4.65))))),
                trade(
                    &[Stock, Bond],
                    &[Tlv],
                    percent(dec!(0.08), Some(ils(dec!(4.65)))),
                ),
                tel_aviv(&[IndexFund]),
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(5))),
                ),
            ],
            abroad,
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        // Its site: no conversion fee, and no custody.
        custody: vec![],
        conversion: markup_only(markup),
        handling: Some(handling_less_trade_fees(dec!(15), 24)),
        management: None,
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            typical_offer_caveat(
                &t("Meitav", "מיטב"),
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::published(t("Nothing as a share of what you hold, and no conversion fee, as Meitav's site states.", "ללא דמי משמרת וללא עמלת המרה, כפי שאתר מיטב מציין."))
                .about_fee(FeeKind::Account)
                .source(&site),
            Caveat::reading(
                t("Bonds cost the offer's 0.08%, at least ₪4.65; gemeltop.co.il lists only \
                 ETFs and stocks.", "אג״ח עולות במבצע 0.08%, מינימום ₪4.65; gemeltop.co.il מציין רק קרנות סל ומניות."),
                t("tradingil.co.il lists bonds at 0.08% for its joining offer (September 2026)", "tradingil.co.il מציין אג״ח ב-0.08% במבצע ההצטרפות שלו (ספטמבר 2026)"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond])
            .on(&[Tlv])
            .source(&on_tradingil),
            not_in_the_offer(
                t("The offer doesn't mention index funds, so the full tariff's price is used. \
                 Managed (active) and money-market funds cost nothing to trade in it.", "המבצע לא מזכיר קרנות מחקות, ולכן נלקח מחיר התעריפון המלא. קרנות מנוהלות (אקטיביות) וקרנות כספיות נסחרות בו ללא עמלה."),
            )
            .about(&[IndexFund])
            .on(&[Tlv]),
            handling_less_trade_fees_caveat(
                t("broker.co.il says so for Meitav's joining offer (2026)", "broker.co.il אומר זאת על מבצע ההצטרפות של מיטב (2026)"),
                &[&broker_co_il()],
            ),
            Caveat::published(t("Some of its trading systems charge at least $7.50 instead of $5.", "חלק ממערכות המסחר שלה גובות מינימום $7.50 במקום $5."))
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa])
                .source(&on_tradingil),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer(t("Not in the offer, so the full tariff's prices are used.", "לא במבצע, ולכן נלקחו מחירי התעריפון המלא."))
                .on(&[Europe]),
            Caveat::not_counted(t("The ₪100 gift for opening an account isn't included.", "מתנת ₪100 על פתיחת חשבון לא נכללה."))
                .source(&on_gemeltop),
        ],
    };

    Broker {
        name: t("Meitav Trade", "מיטב טרייד"),
        short_name: t("Meitav", "מיטב"),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // Typical offer
        compared_at_first: true,
        description: t("The largest exchange member that isn't a bank, part of the Meitav \
                      investment house. It publishes only a full tariff of maximum prices; \
                      new customers are usually offered far less.", "חבר הבורסה הגדול ביותר שאינו בנק, חלק מבית ההשקעות מיטב. היא מפרסמת רק תעריפון מלא של מחירי מקסימום; ללקוחות חדשים מוצע בדרך כלל הרבה פחות."),
        tariff_date: Some(TariffDate::Month(date!(2025 - 01 - 01))),
        source_url: Some("https://www.meitav.co.il/media/z2hkkhku/taarifon.pdf".into()),
        caveats: vec![
            Caveat::reading(
                t("The tariff says the conversion markup is up to 0.7%; 2.1 agorot a dollar \
                 is charged, about 0.7% at ₪3 a dollar, so the full 0.7% is used.", "התעריפון אומר שמרווח ההמרה הוא עד 0.7%; בפועל נגבות 2.1 אגורות לדולר, כ-0.7% בשער של ₪3 לדולר, ולכן המחשבון לוקח את מלוא ה-0.7%."),
                t("tradingil.co.il's comparison of conversion costs, September 2026", "השוואת עלויות ההמרה באתר tradingil.co.il, ספטמבר 2026"),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&tradingil_conversions()),
        ],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── Mizrahi-Tefahot Bank ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn mizrahi() -> Broker {
    // The tariff is published in parts; securities are part 4, and the online
    // prices are appendix E (direct channels). Only the parts used are linked.
    let tariff_url = "https://www.mizrahi-tefahot.co.il/media/smallbusiness4.pdf";
    let tariff = source(t("Tariff (PDF)", "תעריפון (PDF)"), tariff_url);
    let direct_channels = source(
        t(
            "Tariff, appendix E: direct channels (PDF)",
            "תעריפון, נספח ה׳: ערוצים ישירים (PDF)",
        ),
        "https://www.mizrahi-tefahot.co.il/media/arutzyashir.pdf",
    );
    let groups = source(
        t(
            "Tariff, appendix A: customer groups (PDF)",
            "תעריפון, נספח א׳: הטבות לקבוצות אוכלוסייה (PDF)",
        ),
        "https://www.mizrahi-tefahot.co.il/media/specials.pdf",
    );
    let rates_page = source(
        t(
            "Mizrahi-Tefahot's exchange rates",
            "שערי החליפין של מזרחי-טפחות",
        ),
        "https://www.mizrahi-tefahot.co.il/brokerage/foreignexchange/",
    );
    let broker_caveats = vec![
        Caveat::reading(
            t("Mizrahi-Tefahot converts at its published transfers-and-checks rate, about \
             0.8% from the representative rate each way: a conversion markup of up to \
             0.8%, on top of the fee.", "מזרחי-טפחות ממיר לפי שער ההעברות וההמחאות שהוא מפרסם, כ-0.8% מהשער היציג לכל כיוון: מרווח המרה של עד 0.8%, בנוסף לעמלה."),
            t("Mizrahi-Tefahot's exchange rates for 29 September 2026: it bought dollars \
             at ₪3.0462 and sold them at ₪3.0953 against a representative ₪3.0720 (0.84% \
             under and 0.76% over), and euros at ₪3.4539 and ₪3.5098 against ₪3.4857 \
             (0.91% under and 0.69% over)", "שערי מזרחי-טפחות ל-29 בספטמבר 2026: קנה דולרים ב-₪3.0462 ומכר ב-₪3.0953 לעומת שער יציג של ₪3.0720 (0.84% מתחת ו-0.76% מעל), ואירו ב-₪3.4539 ו-₪3.5098 לעומת ₪3.4857 (0.91% מתחת ו-0.69% מעל)"),
        )
        .about_fee(FeeKind::Markup)
        .on(&[Usa, Europe])
        .source(&rates_page),
        Caveat::published(t(
            "ETFs and index funds on Tel Aviv are priced as stocks and bonds: the row \
             leaves mutual funds out but counts index funds and listed funds back in \
             (part 4, note 4).",
            "קרנות סל וקרנות מחקות בתל אביב מתומחרות כמו מניות ואג״ח: השורה מוציאה קרנות נאמנות, אבל מחזירה פנימה קרנות מחקות וקרנות שיחידותיהן רשומות בבורסה (חלק 4, הערה 4).",
        ))
        .about_fee(FeeKind::Trade)
        .about(&[Etf, IndexFund])
        .on(&[Tlv])
        .source(&tariff),
        Caveat::not_counted(t(
            "Trading abroad also passes on the broker's, custodian's and any other cost \
             of completing the order there, as actually charged (part 4, note 7).",
            "במסחר בחו״ל נגבות בנוסף הוצאות ברוקר, קסטודיאן וכל הוצאה אחרת שנדרשת להשלמת הפקודה שם, לפי החיוב בפועל (חלק 4, הערה 7).",
        ))
        .about_fee(FeeKind::Trade)
        .on(&[Usa, Europe])
        .source(&tariff),
    ];
    let quarterly = |exchanges: &[Exchange], rate: Decimal| CustodyFee {
        exchanges: exchanges.to_vec(),
        ..custody(rate, Period::Quarter, Period::Quarter, None)
    };

    // Measured from the bank's published rates (the caveat above).
    let markup = Markup::UpTo(Percent(dec!(0.8)));
    // Appendix E: the website's prices. The branch prices in part 4 are
    // higher (0.64% and 0.89%), and the "PC service" differs a little.
    let online_conversion = PercentFee {
        percent: Percent(dec!(0.133)),
        min: Some(usd(dec!(5.4))),
        max: Some(usd(dec!(2250))),
    };
    let online = Plan {
        name: t("Online", "אונליין"),
        description: t(
            "Mizrahi-Tefahot's prices for trading yourself on its website or in its \
             capital-market app (the tariff's appendix on direct channels). Trading through \
             a banker at a branch costs more.",
            "המחירים של מזרחי-טפחות למסחר עצמאי באתר או באפליקציית שוק ההון (נספח הערוצים הישירים בתעריפון). מסחר דרך בנקאי בסניף עולה יותר.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            trade(
                &[],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.52)),
                    min: Some(ils(dec!(45))),
                    max: Some(ils(dec!(9675))),
                },
            ),
            trade(
                &[],
                &[Usa, Europe],
                Price::Percent {
                    percent: Percent(dec!(0.3)),
                    min: Some(usd(dec!(25))),
                    max: Some(usd(dec!(6750))),
                },
            ),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        // Part 4, rows 4.5.1 and 4.5.2: the same online and at a branch.
        custody: vec![
            quarterly(&[Tlv], dec!(0.175)),
            quarterly(&[Usa, Europe], dec!(0.195)),
        ],
        conversion: ConversionFee {
            fee: online_conversion,
            or_if_less: None,
            markup,
        },
        handling: None,
        management: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::published(t(
                "The share of your holdings is the tariff's, online or not: 0.175% a \
                 quarter on Tel Aviv holdings and 0.195% on foreign ones, charged every \
                 quarter for the days held.",
                "דמי המשמרת הם של התעריפון, באונליין או לא: 0.175% לרבעון על החזקות בתל אביב ו-0.195% על החזקות בחו״ל, ונגבים כל רבעון לפי ימי ההחזקה.",
            ))
            .about_fee(FeeKind::Account)
            .source(&tariff)
            .source(&direct_channels),
        ],
    };

    // Appendix A: the same benefits for four groups. The 0.4% is "not less
    // than the minimum fee", read as the tariff's ₪50; whether Online's ₪45
    // applies too isn't said.
    let group_conversion = PercentFee {
        percent: Percent(dec!(0.095)),
        min: Some(usd(dec!(6))),
        max: Some(usd(dec!(2500))),
    };
    let young = Plan {
        name: t("Online, young customers", "אונליין, לקוחות צעירים"),
        description: t(
            "Online prices with the tariff's discounts for its young customer groups: ages \
             14 to 21½, soldiers and national-service volunteers, discharged soldiers \
             (three years, or to age 30) and students (four years, or to 35). Check your \
             eligibility with the bank.",
            "מחירי אונליין עם ההנחות שבתעריפון לקבוצות הלקוחות הצעירים: גילאי 14 עד 21½, חיילים ובנות שירות לאומי, חיילים משוחררים (שלוש שנים, או עד גיל 30) וסטודנטים (ארבע שנים, או עד גיל 35). בדקו את זכאותכם מול הבנק.",
        ),
        trading: vec![
            trade(
                &[],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.4)),
                    min: Some(ils(dec!(50))),
                    max: Some(ils(dec!(10_750))),
                },
            ),
            online.trading[1].clone(),
        ],
        // Half the branch fee, not below its minimum; or Online's, where
        // that's less (under about $4,500).
        conversion: ConversionFee {
            fee: group_conversion,
            or_if_less: Some(online_conversion),
            markup: Markup::UpTo(Percent(dec!(0.5))),
        },
        caveats: vec![
            Caveat::at_most(t(
                "Tel Aviv costs the groups' 0.4% with the tariff's ₪50 minimum. Whether \
                 Online's ₪45 minimum applies as well isn't stated; on orders under ₪9,600 \
                 that would be up to ₪5 less.",
                "בתל אביב המחיר הוא 0.4%, מחיר הקבוצות, עם המינימום שבתעריפון, ₪50. לא כתוב אם המינימום של אונליין, ₪45, חל גם כאן; בפקודות של פחות מ-₪9,600 זה היה חוסך עד ₪5.",
            ))
            .about_fee(FeeKind::Trade)
            .on(&[Tlv])
            .source(&groups),
            Caveat::at_most(t(
                "Conversion is half the branch fee, not below its $6 minimum, or Online's \
                 fee where that's less. Whether the half applies to Online's fee and to \
                 the $2,500 maximum too isn't stated; it's assumed not.",
                "ההמרה עולה חצי מעמלת הסניף, ולא פחות מהמינימום שלה, $6, או את עמלת אונליין כשהיא נמוכה יותר. לא כתוב אם ההנחה של חצי חלה גם על עמלת אונליין ועל המקסימום, $2,500; המחשבון מניח שלא.",
            ))
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&groups),
            Caveat::reading(
                t("The groups' exchange-rate benefit of 3 pro mille takes the markup down \
                 from 0.8% to 0.5%.", "הטבת השער של הקבוצות, 3 פרומיל, מורידה את מרווח ההמרה מ-0.8% ל-0.5%."),
                t("appendix A gives these groups \u{201c}a rate benefit of 3 pro mille on \
                 buying and converting foreign currency\u{201d}, and the markup measured \
                 from the bank's published rates is 0.8%", "נספח א׳ נותן לקבוצות האלה ״הטבת שער ברכישה ובהמרת מט״ח״ של 3 פרומיל, והמרווח שנמדד משערי הבנק שפורסמו הוא 0.8%"),
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&groups)
            .source(&rates_page),
            Caveat::published(t(
                "The groups' securities and conversion discounts don't apply to an \
                 account a portfolio manager holds power of attorney over.",
                "ההנחות של הקבוצות בניירות ערך ובהמרה לא חלות על חשבון עם ייפוי כוח למנהל תיקים.",
            ))
            .source(&groups),
        ],
        ..online.clone()
    };

    Broker {
        name: t("Mizrahi-Tefahot Bank", "בנק מזרחי-טפחות"),
        short_name: t("Mizrahi", "מזרחי"),
        kind: BrokerKind::Bank,
        new_customer_plan: 0, // Online
        compared_at_first: true,
        description: t(
            "One of Israel's largest banks. Its securities prices depend on how you \
             trade (on its website and app, or through a banker) and on customer \
             groups, so it has two plans here. Its tariff is published in parts: \
             securities (2 April 2024), direct channels (12 August 2025), customer \
             groups (13 January 2026) and foreign currency.",
            "מהבנקים הגדולים בישראל. מחירי ניירות הערך שלו תלויים באופן המסחר (באתר ובאפליקציה, או דרך בנקאי) ובקבוצות לקוחות, ולכן יש לו כאן שני מסלולים. התעריפון שלו מפורסם בחלקים: ניירות ערך (2 באפריל 2024), ערוצים ישירים (12 באוגוסט 2025), קבוצות אוכלוסייה (13 בינואר 2026) ומטבע חוץ.",
        ),
        tariff_date: Some(TariffDate::Day(date!(2024 - 04 - 02))),
        source_url: Some(tariff_url.into()),
        caveats: broker_caveats,
        plans: vec![online, young],
    }
}

// ─────────────────────────── Bank Otsar Hahayal ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn otsar_hahayal() -> Broker {
    // Otsar Hahayal is a brand of the First International Bank, which merged
    // it in; the tariff on its site is the First International's for
    // individuals and small businesses, and the rates page is the parent's.
    let tariff_url = "https://www.bankotsar.co.il/media/dhneratu/%D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%91%D7%99%D7%A0%D7%9C%D7%90%D7%95%D7%9E%D7%99-%D7%99%D7%97%D7%99%D7%93%D7%99%D7%9D-16092026.pdf";
    let tariff = source(t("Tariff (PDF)", "תעריפון (PDF)"), tariff_url);
    let rates_page = source(
        t(
            "The First International Bank's exchange rates",
            "שערי החליפין של הבנק הבינלאומי",
        ),
        "https://apps.fibi.co.il/Matach/matach.aspx",
    );
    let club_page = source(
        t(
            "The Otzar Habitachon club's securities benefits",
            "הטבות מועדון ״אוצר הביטחון״ בניירות ערך",
        ),
        "https://www.bankotsar.co.il/private/account/armedforces/defense/securitiesbenefits/",
    );
    let top_trade_page = source(
        t("Top Trade's page", "עמוד טופ טרייד"),
        "https://www.bankotsar.co.il/private/capitalmarket/toptrade/",
    );
    let conscripts_page = source(
        t("Top Trade for conscripts", "טופ טרייד חיילים"),
        "https://www.bankotsar.co.il/private/account/types/soldiers/",
    );
    let broker_caveats = vec![
        Caveat::reading(
            t("The bank converts at the First International's published transfers-and-checks \
             rate, about 0.9% from its mid-rate each way: a conversion markup of up to \
             0.9%, on top of the fee.", "הבנק ממיר לפי שער ההעברות וההמחאות שהבינלאומי מפרסם, כ-0.9% משער האמצע שלו לכל כיוון: מרווח המרה של עד 0.9%, בנוסף לעמלה."),
            t("the First International's exchange rates for 29 September 2026: it bought \
             dollars at ₪3.0493 and sold them at ₪3.1047 around a mid-rate of ₪3.0770 \
             (0.9% each way; the representative rate was ₪3.0720), and euros at ₪3.4636 \
             and ₪3.5266 around ₪3.4951", "שערי הבינלאומי ל-29 בספטמבר 2026: קנה דולרים ב-₪3.0493 ומכר ב-₪3.1047 סביב שער אמצע של ₪3.0770 (0.9% לכל כיוון; השער היציג היה ₪3.0720), ואירו ב-₪3.4636 ו-₪3.5266 סביב ₪3.4951"),
        )
        .about_fee(FeeKind::Markup)
        .on(&[Usa, Europe])
        .source(&rates_page),
        Caveat::published(t(
            "ETFs and index funds on Tel Aviv are priced as stocks and bonds: the row \
             leaves mutual funds out but counts index funds and listed funds back in \
             (part 4, notes 3 and 14).",
            "קרנות סל וקרנות מחקות בתל אביב מתומחרות כמו מניות ואג״ח: השורה מוציאה קרנות נאמנות, אבל מחזירה פנימה קרנות מחקות וקרנות שיחידותיהן רשומות בבורסה (חלק 4, הערות 3 ו-14).",
        ))
        .about_fee(FeeKind::Trade)
        .about(&[Etf, IndexFund])
        .on(&[Tlv])
        .source(&tariff),
        Caveat::not_counted(t(
            "Third-party costs are passed on on top of every trade, as actually charged \
             (part 11): the exchange's own trading and clearing fees in Tel Aviv, and \
             abroad the broker's (up to 6¢ a share in the US), the custodian's and the \
             SEC's.",
            "הוצאות צד שלישי נגבות בנוסף על כל עסקה, לפי החיוב בפועל (חלק 11): עמלות המסחר והסליקה של הבורסה עצמה בתל אביב, ובחו״ל הוצאות הברוקר (עד 6 סנט למניה בארה״ב), הקסטודיאן וה-SEC.",
        ))
        .about_fee(FeeKind::Trade)
        .source(&tariff),
    ];
    let quarterly = |exchanges: &[Exchange], rate: Decimal| CustodyFee {
        exchanges: exchanges.to_vec(),
        ..custody(rate, Period::Quarter, Period::Quarter, None)
    };

    // Measured from the parent bank's published rates (the caveat above).
    let markup = Markup::UpTo(Percent(dec!(0.9)));
    // Appendix E (direct channels): the website's and the app's prices. The
    // branch prices in part 4 are higher (0.64% and 0.85%).
    let online_conversion = PercentFee {
        percent: Percent(dec!(0.0875)),
        min: Some(usd(dec!(5))),
        max: Some(usd(dec!(2475))),
    };
    let online_tlv = Price::Percent {
        percent: Percent(dec!(0.3)),
        min: Some(ils(dec!(39))),
        max: Some(ils(dec!(6900))),
    };
    let online_abroad = Price::Percent {
        percent: Percent(dec!(0.25)),
        min: Some(usd(dec!(39))),
        max: Some(usd(dec!(6900))),
    };
    let online = Plan {
        name: t("Online", "אונליין"),
        description: t(
            "The bank's prices for trading yourself on its website or in its app (the \
             tariff's appendix on direct channels). Trading through a banker costs more.",
            "המחירים של הבנק למסחר עצמאי באתר או באפליקציה (נספח הערוצים הישירים בתעריפון). מסחר דרך בנקאי עולה יותר.",
        ),
        vehicle: Vehicle::Brokerage,
        trading: vec![
            trade(&[], &[Tlv], online_tlv.clone()),
            trade(&[], &[Usa, Europe], online_abroad.clone()),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        // Part 4, rows 4.5.1 and 4.5.2: the same online and at a branch.
        custody: vec![
            quarterly(&[Tlv], dec!(0.195)),
            quarterly(&[Usa, Europe], dec!(0.199)),
        ],
        conversion: ConversionFee {
            fee: online_conversion,
            or_if_less: None,
            markup,
        },
        handling: None,
        management: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::published(t(
                "The share of your holdings is the tariff's, online or not: 0.195% a \
                 quarter on Tel Aviv holdings and 0.199% on foreign ones, charged on the \
                 15th of the first month of each quarter for the days held.",
                "דמי המשמרת הם של התעריפון, באונליין או לא: 0.195% לרבעון על החזקות בתל אביב ו-0.199% על החזקות בחו״ל, ונגבים ב-15 בחודש הראשון של כל רבעון לפי ימי ההחזקה.",
            ))
            .about_fee(FeeKind::Account)
            .source(&tariff),
        ],
    };

    // Part 4, note 13 on the Tel Aviv row and note 10 on the foreign one:
    // buying an index fund or an ETF by standing order has a lower minimum.
    // Only the minimum is stated, so the rate is assumed to be Online's.
    let standing_order = Plan {
        // Shorter than Leumi's "monthly standing order": with the bank's
        // longer name, that didn't fit on a phone's fee breakdown.
        name: t("Online, standing order", "אונליין, הוראת קבע"),
        description: t(
            "Not a separate account: an Online customer buying Tel Aviv index funds or \
             foreign ETFs by monthly standing order, for which the tariff lowers the \
             minimum fee. Everything else is priced as Online.",
            "לא חשבון נפרד: לקוח אונליין שקונה קרנות מחקות בתל אביב או קרנות סל בחו״ל בהוראת קבע חודשית, שהתעריפון מוריד לה את העמלה המינימלית. כל השאר מתומחר כמו אונליין.",
        ),
        standing_orders: vec![
            trade(
                &[IndexFund],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.3)),
                    min: Some(ils(dec!(7))),
                    max: Some(ils(dec!(6900))),
                },
            ),
            trade(
                &[Etf],
                &[Usa, Europe],
                Price::Percent {
                    percent: Percent(dec!(0.25)),
                    min: Some(usd(dec!(4.5))),
                    max: Some(usd(dec!(6900))),
                },
            ),
        ],
        caveats: vec![
            Caveat::may_cost_more(
                t("The tariff lowers only the standing order's minimum (₪7 for index funds, \
                 $4.50 for ETFs) without saying which rate goes with it. Online's is \
                 assumed (0.3% and 0.25%); through a banker it would be 0.64% and 0.85%.", "התעריפון מוריד רק את המינימום של הוראת הקבע (₪7 לקרנות מחקות, $4.50 לקרנות סל), בלי לומר איזה שיעור הולך איתו. המחשבון לוקח את השיעור של אונליין (0.3% ו-0.25%); דרך בנקאי השיעור היה 0.64% ו-0.85%."),
                t("the standing order's rate isn't stated", "שיעור העמלה בהוראת קבע לא כתוב"),
            )
            .about_fee(FeeKind::StandingOrder)
            .source(&tariff),
            Caveat::at_most(t(
                "The lower minimum names index funds on Tel Aviv and ETFs abroad. ETFs on \
                 Tel Aviv by standing order aren't mentioned, so they're priced as Online.",
                "המינימום המופחת מונה קרנות מחקות בתל אביב וקרנות סל בחו״ל. קרנות סל בתל אביב בהוראת קבע לא מוזכרות, ולכן תומחרו כמו אונליין.",
            ))
            .about_fee(FeeKind::StandingOrder)
            .about(&[Etf])
            .on(&[Tlv]),
            Caveat::published(t(
                "Only buying by standing order is cheaper; selling, and buying anything \
                 else, costs what it does on Online.",
                "רק קנייה בהוראת קבע זולה יותר; מכירה, וקנייה של כל השאר, עולות כמו באונליין.",
            ))
            .about_fee(FeeKind::StandingOrder),
        ],
        ..online.clone()
    };

    // The bank's club for the security forces' family (its site). Its
    // abroad rate is above Online's, and its terms allow no double benefits.
    let club = Plan {
        name: t("Otzar Habitachon club", "מועדון ״אוצר הביטחון״"),
        description: t(
            "The bank's club for the security forces' family: customers whose salary or \
             pension comes from the IDF, the Ministry of Defence or another body in its \
             joining form, or who show an employee card. Until 30 November 2030, under the \
             bank's agreement with the Ministry of Defence.",
            "המועדון של הבנק למשפחת כוחות הביטחון: לקוחות שמשכורתם או גמלתם מגיעה מצה״ל, ממשרד הביטחון או מגוף אחר שבטופס ההצטרפות, או שמציגים תעודת עובד. עד 30 בנובמבר 2030, לפי ההסכם בין הבנק למשרד הביטחון.",
        ),
        trading: vec![
            trade(&[], &[Tlv], percent(dec!(0.175), Some(ils(dec!(10))))),
            trade(&[], &[Usa, Europe], percent(dec!(0.3), Some(usd(dec!(18))))),
        ],
        custody: vec![],
        caveats: vec![
            Caveat::published(t(
                "Custody (דמי\u{a0}משמרת) on Tel Aviv and foreign holdings is waived.",
                "דמי משמרת על החזקות בתל אביב ובחו״ל: פטור.",
            ))
            .about_fee(FeeKind::Account)
            .source(&club_page),
            Caveat::published(t(
                "The 0.175% is for orders placed on the website or in the app; through a \
                 banker or the call centre it's 0.2%.",
                "0.175% הם לפקודות שניתנות באתר או באפליקציה; דרך בנקאי או המוקד הטלפוני זה 0.2%.",
            ))
            .about_fee(FeeKind::Trade)
            .on(&[Tlv])
            .source(&club_page),
            Caveat::at_most(t(
                "Abroad, the club's 0.3% (at least $18) is used throughout. On orders above \
                 $15,600 Online's 0.25% would be less, but the club's terms allow no \
                 double benefits.",
                "בחו״ל המחשבון לוקח את מחיר המועדון, 0.3% (מינימום $18), בכל הפקודות. בפקודות של יותר מ-$15,600 המחיר של אונליין, 0.25%, היה זול יותר, אבל תנאי המועדון לא מאפשרים כפל הטבות.",
            ))
            .about_fee(FeeKind::Trade)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(15_600)))
            .source(&club_page),
            Caveat::reading(
                t("Conversion isn't in the club's terms, so Online's price is used: 0.0875% \
                 (at least $5), plus the markup.", "ההמרה לא מופיעה בתנאי המועדון, ולכן נלקח המחיר של אונליין: 0.0875% (מינימום $5), ועוד המרווח."),
                t("the direct-channel prices are the tariff's own, for every customer who \
                 uses the website or app, not a club benefit that its \u{201c}no double \
                 benefits\u{201d} rule would cancel", "מחירי הערוצים הישירים הם של התעריפון עצמו, לכל לקוח שמשתמש באתר או באפליקציה, ולא הטבת מועדון שהכלל ״אין כפל הטבות״ היה מבטל"),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&club_page)
            .source(&tariff),
        ],
        ..online.clone()
    };

    // The bank's plan for the young (its site): flat prices and no custody,
    // for a portfolio of up to ₪200,000.
    let top_trade = Plan {
        name: t("Top Trade", "טופ טרייד"),
        description: t(
            "The bank's plan for customers aged 18 to 30 who trade themselves, without \
             advice, with a portfolio of up to ₪200,000. The bank may change or end it at \
             any time.",
            "המסלול של הבנק ללקוחות בגילאי 18 עד 30 שסוחרים עצמאית, בלי ייעוץ, עם תיק של עד ₪200,000. הבנק רשאי לשנות או להפסיק אותו בכל עת.",
        ),
        trading: vec![
            trade(&[], &[Tlv], Price::Flat(ils(dec!(5)))),
            trade(&[], &[Usa, Europe], Price::Flat(usd(dec!(5)))),
        ],
        custody: vec![],
        caveats: vec![
            Caveat::may_cost_more(
                t("Top Trade's prices are for a portfolio of up to ₪200,000; what a larger \
                 one pays isn't stated. They're assumed to go on.", "מחירי טופ טרייד כתובים לתיק של עד ₪200,000, ולא כתוב כמה משלם תיק גדול יותר. המחשבון מניח שאותם מחירים ממשיכים."),
                t("the prices are stated only for portfolios up to ₪200,000", "המחירים כתובים רק לתיקים של עד ₪200,000"),
            )
            .about_fee(FeeKind::Trade)
            .source(&top_trade_page),
            Caveat::reading(
                t("Top Trade's rows name stocks and bonds on Tel Aviv, and stocks, funds and \
                 bonds abroad; ETFs and index funds are taken to be included.", "השורות של טופ טרייד מונות מניות ואג״ח בתל אביב, ומניות, קרנות ואג״ח בחו״ל; הבנו שגם קרנות סל וקרנות מחקות נכללות."),
                t("the tariff's own \u{201c}stocks and bonds\u{201d} row counts index funds \
                 and listed funds in (part 4, notes 3 and 14), and Top Trade's row uses the \
                 same words", "שורת ״מניות ואג״ח״ של התעריפון עצמו כוללת קרנות מחקות וקרנות רשומות (חלק 4, הערות 3 ו-14), ושורת טופ טרייד משתמשת באותן מילים"),
            )
            .about_fee(FeeKind::Trade)
            .about(&[Etf, IndexFund])
            .source(&top_trade_page)
            .source(&tariff),
            Caveat::reading(
                t("Conversion isn't on Top Trade's page, so Online's price is used: 0.0875% \
                 (at least $5), plus the markup.", "ההמרה לא מופיעה בעמוד טופ טרייד, ולכן נלקח המחיר של אונליין: 0.0875% (מינימום $5), ועוד המרווח."),
                t("Top Trade is trading on the website and in the app, whose conversion \
                 price is the tariff's direct-channel one", "טופ טרייד הוא מסחר באתר ובאפליקציה, שמחיר ההמרה בהם הוא מחיר הערוצים הישירים שבתעריפון"),
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&top_trade_page)
            .source(&tariff),
            Caveat::published(t(
                "No custody (דמי\u{a0}משמרת) on Tel Aviv or foreign holdings, and nothing \
                 for an order that isn't executed.",
                "אין דמי משמרת על החזקות בתל אביב או בחו״ל, ולא משלמים על פקודה שלא בוצעה.",
            ))
            .about_fee(FeeKind::Account)
            .source(&top_trade_page),
            Caveat::not_counted(t(
                "Conscripts and national-service volunteers get Top Trade with every fee \
                 waived (trading, custody and conversion), for a portfolio of up to \
                 ₪300,000 and up to 150 orders, until 31 December 2026.",
                "חיילים בשירות חובה ובני שירות לאומי מקבלים את טופ טרייד עם פטור מכל העמלות (מסחר, דמי משמרת והמרה), לתיק של עד ₪300,000 ועד 150 פעולות, עד 31 בדצמבר 2026.",
            ))
            .source(&conscripts_page),
        ],
        ..online.clone()
    };

    Broker {
        name: t("Bank Otsar Hahayal", "בנק אוצר החייל"),
        short_name: t("Otsar Hahayal", "אוצר החייל"),
        kind: BrokerKind::Bank,
        new_customer_plan: 0, // Online
        compared_at_first: true,
        description: t(
            "The security forces' bank, part of the First International Bank \
             (הבינלאומי), which merged it in and keeps it as a brand: the tariff is the \
             First International's. Its securities prices depend on how you trade (on \
             the website and app, or through a banker), on its club for the security \
             forces' family, and on Top Trade for the young, so it has several plans \
             here.",
            "הבנק של כוחות הביטחון, חלק מהבנק הבינלאומי, שמיזג אותו לתוכו ושומר עליו כמותג: התעריפון הוא של הבינלאומי. מחירי ניירות הערך שלו תלויים באופן המסחר (באתר ובאפליקציה, או דרך בנקאי), במועדון שלו למשפחת כוחות הביטחון, ובטופ טרייד לצעירים, ולכן יש לו כאן כמה מסלולים.",
        ),
        tariff_date: Some(TariffDate::Day(date!(2026 - 09 - 16))),
        source_url: Some(tariff_url.into()),
        caveats: broker_caveats,
        plans: vec![online, standing_order, club, top_trade],
    }
}
