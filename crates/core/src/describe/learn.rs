//! The learn cards: one page per term a saver meets in a bank's app, in an
//! ad or around them, in the shape of the plan's screen s5 (what it is, what
//! it's good for, its downside, what it costs, its tax, where you meet it, an
//! example, and what to compare it with).
//!
//! Every claim in a card rests on a page linked under it (`sources`, listed
//! with their claims in `policies/sources.md`, "Learn cards"). A number that
//! moves (a fee, a rate, a ceiling) isn't written here: the card sends the
//! reader to the app's table, which has it from the data. Only rules of law
//! are written as numbers (the 25% on a real gain, the 15% on a deposit's
//! interest), and the examples' numbers are round ones that say they are.

use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, IntoEnumIterator};

use super::Source;
use crate::{Lang, Page, Text, funds, short_term};

/// What has a card. The order of the variants is the glossary's: what you
/// can buy, then what it costs, then how money grows and is taxed, then the
/// places it's kept in: the order someone who knows nothing meets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Term {
    // What you can buy
    Etf,
    IndexFund,
    MutualFund,
    MoneyMarketFund,
    FixedDeposit,
    CpiLinkedDeposit,
    Makam,
    GovernmentBond,
    Sp500Index,
    InvestmentGemel,
    StudyFund,
    SavingsPolicy,
    SavingsGemel,
    // What it costs
    ManagementFee,
    CustodyAndHandling,
    TradeCommission,
    ConversionMarkup,
    // How money grows and is taxed
    CompoundInterest,
    Inflation,
    CapitalGainsTax,
    YieldToMaturity,
    BankOfIsraelRate,
    Pension,
    DepositCeiling,
    Cushion,
    // Where it's kept
    BrokerageAccount,
    Exchange,
}

/// A card in one language: ready to show.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct LearnCard {
    pub term: Term,
    /// The term as Israelis say it: "קרן כספית".
    pub name: String,
    /// "Money market fund", shown under the Hebrew name.
    pub english: String,
    /// What it's called in the bank's app and on its forms.
    pub as_the_bank_calls_it: String,
    /// What it's called in ads and the news.
    pub as_ads_call_it: String,
    pub what: String,
    pub good_for: String,
    pub downside: String,
    pub cost: String,
    pub tax: String,
    pub where_met: String,
    /// With round numbers, which it says are for illustration.
    pub example: String,
    /// The cards the reader may want next: "לעומת פיקדון ›".
    pub compare_with: Vec<Term>,
    pub sources: Vec<Source>,
}

/// A card with both languages, before one is picked.
struct Card {
    name: Text,
    english: &'static str,
    bank: Text,
    ads: Text,
    what: Text,
    good_for: Text,
    downside: Text,
    cost: Text,
    tax: Text,
    where_met: Text,
    example: Text,
    compare_with: &'static [Term],
    sources: Vec<Page>,
}

impl Term {
    /// Its card in `lang`.
    #[must_use]
    pub fn card(self, lang: Lang) -> LearnCard {
        let card = card(self);
        LearnCard {
            term: self,
            name: card.name[lang].to_owned(),
            english: card.english.to_owned(),
            as_the_bank_calls_it: card.bank[lang].to_owned(),
            as_ads_call_it: card.ads[lang].to_owned(),
            what: card.what[lang].to_owned(),
            good_for: card.good_for[lang].to_owned(),
            downside: card.downside[lang].to_owned(),
            cost: card.cost[lang].to_owned(),
            tax: card.tax[lang].to_owned(),
            where_met: card.where_met[lang].to_owned(),
            example: card.example[lang].to_owned(),
            compare_with: card.compare_with.to_vec(),
            sources: card
                .sources
                .iter()
                .map(|page| Source::new(page, lang))
                .collect(),
        }
    }
}

/// `term`'s card in `lang`.
#[must_use]
pub fn learn_card(term: Term, lang: Lang) -> LearnCard {
    term.card(lang)
}

/// Every card, each once, in the order of [`Term`].
#[must_use]
pub fn glossary(lang: Lang) -> Vec<LearnCard> {
    Term::iter().map(|term| term.card(lang)).collect()
}

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

fn page(en: &'static str, he: &'static str, url: &'static str) -> Page {
    Page {
        name: t(en, he),
        url: url.to_owned(),
    }
}

// ─────────────────────────── The pages the cards rest on ───────────────────────────

fn themarker_etf_and_index() -> Page {
    page(
        "TheMarker: ETFs and index funds compared",
        "דה\u{2011}מרקר: קרנות סל וקרנות מחקות",
        "https://www.supermarker.themarker.com/Investments/EtfAndIndexTrackingFunds.aspx",
    )
}

fn etf_reform() -> Page {
    page(
        "Protocol on the 2018 reform: every certificate becomes a fund",
        "פרוטוקול על הרפורמה של 2018: כל תעודות הסל הופכות לקרנות",
        "https://protocol.co.il/etfs-mutual-funds/",
    )
}

fn boi_money_fund() -> Page {
    page(
        "Bank of Israel: the money market fund",
        "בנק ישראל: הקרן הכספית",
        "https://www.boi.org.il/information/bank-paymnts/savingsandinvestmentproducts/moneyfund/",
    )
}

fn tradingil_money_fund() -> Page {
    page(
        "Tradingil on the shekel money market fund",
        "טריידינגיל על הקרן הכספית השקלית",
        "https://tradingil.co.il/קרן-כספית-שקלית/",
    )
}

fn boi_makam() -> Page {
    page(
        "Bank of Israel: makam",
        "בנק ישראל: מק״מ",
        "https://boi.org.il/information/bank-paymnts/savingsandinvestmentproducts/makam/",
    )
}

fn boi_custody_rule() -> Page {
    page(
        "Bank of Israel, 28 November 2012: no custody fee on a money market fund or makam",
        "בנק ישראל, 28 בנובמבר 2012: אין דמי משמרת על קרן כספית ועל מק״מ",
        "https://boi.org.il/publications/pressreleases/%D7%94%D7%9E%D7%A4%D7%A7%D7%97-%D7%A2%D7%9C-%D7%94%D7%91%D7%A0%D7%A7%D7%99%D7%9D-%D7%A0%D7%95%D7%A7%D7%98-%D7%91%D7%A6%D7%A2%D7%93%D7%99%D7%9D-%D7%9C%D7%94%D7%A4%D7%97%D7%AA%D7%AA-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-%D7%A2%D7%91%D7%95%D7%A8-%D7%9E%D7%A9%D7%A7%D7%99-%D7%94%D7%91%D7%99%D7%AA-%D7%95%D7%94%D7%A2%D7%A1%D7%A7%D7%99%D7%9D-%D7%94%D7%A7%D7%98%D7%A0%D7%99%D7%9D/",
    )
}

fn boi_deposits() -> Page {
    page(
        "Bank of Israel: compare the banks' deposit rates",
        "בנק ישראל: השוואת ריביות הפיקדונות בבנקים",
        "https://www.boi.org.il/information/bank-paymnts/financial-education/campaigns/boi-equator/deposit/",
    )
}

fn bizportal_tax_on_interest() -> Page {
    page(
        "Bizportal on the tax on interest, linked and unlinked",
        "ביזפורטל על המס על ריבית, צמודה ולא צמודה",
        "https://www.bizportal.co.il/general/news/article/104576",
    )
}

fn leumi_linked_savings() -> Page {
    page(
        "Leumi's page of savings rates, with its CPI-linked savings",
        "עמוד ריביות החיסכון של לאומי, כולל ״חיסכון צמוד מדד״",
        "https://www.leumi.co.il/savings/Payments-saving-month",
    )
}

fn themarker_bond_0347() -> Page {
    page(
        "TheMarker: the bond \u{201c}ממשלתי שקלי 0347\u{201d}",
        "דה\u{2011}מרקר: האג״ח ״ממשלתי שקלי 0347״",
        "https://finance.themarker.com/bond/1140193",
    )
}

pub(crate) fn boi_yield_curve() -> Page {
    page(
        "Bank of Israel: the zero-coupon yield curve",
        "בנק ישראל: עקום התשואות",
        "https://edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/BOI.STATISTICS/ZCM/1.0?format=csv&lastNObservations=1",
    )
}

fn boi_monetary_policy() -> Page {
    page(
        "Bank of Israel: monetary policy",
        "בנק ישראל: המדיניות המוניטרית",
        "https://www.boi.org.il/en/economic-roles/monetary-policy/",
    )
}

fn boi_rate_schedule() -> Page {
    page(
        "Bank of Israel: the interest rate, set eight times a year",
        "בנק ישראל: ריבית בנק ישראל, נקבעת שמונה פעמים בשנה",
        "https://www.boi.org.il/en/economic-roles/financial-markets/shir/",
    )
}

fn boi_inflation_target() -> Page {
    page(
        "Bank of Israel: the inflation target of 1\u{2013}3%",
        "בנק ישראל: יעד האינפלציה, 1\u{2013}3%",
        "https://boi.org.il/en/communication-and-publications/press-releases/6-11-24/",
    )
}

fn kolzchut_investment_gemel() -> Page {
    page(
        "Kol Zchut: a provident fund for investment",
        "כל\u{2011}זכות: קופת גמל להשקעה",
        "https://www.kolzchut.org.il/he/קופת_גמל_להשקעה",
    )
}

fn kolzchut_pension_exemption() -> Page {
    page(
        "Kol Zchut: the tax-free pension from 60",
        "כל\u{2011}זכות: הפטור ממס בקצבה מגיל 60",
        "https://www.kolzchut.org.il/he/פטור_ממס_על_רווחים_בקופת_גמל_להשקעה_המשולמת_כקצבה_לאחר_גיל_60",
    )
}

fn kolzchut_study_fund() -> Page {
    page(
        "Kol Zchut: a study fund",
        "כל\u{2011}זכות: קרן השתלמות",
        "https://www.kolzchut.org.il/he/קרן_השתלמות",
    )
}

fn kolzchut_gemel() -> Page {
    page(
        "Kol Zchut: a provident fund (kupat gemel)",
        "כל\u{2011}זכות: קופת גמל",
        "https://www.kolzchut.org.il/he/קופת_גמל",
    )
}

fn meitav_amendment_190() -> Page {
    page(
        "Meitav on Amendment 190 and the provident fund for savings",
        "מיטב על תיקון 190 ועל קופת גמל לחיסכון",
        "https://www.meitav.co.il/provident_pension/amendment_190/",
    )
}

fn analyst_savings_policy() -> Page {
    page(
        "Analyst: a savings policy and a provident fund for investment",
        "אנליסט: פוליסת חיסכון וקופת גמל להשקעה",
        "https://www.analyst.co.il/articles/savings-policy-and-investment-provident-fund/",
    )
}

fn menora_gemel_and_policy() -> Page {
    page(
        "Menora Mivtachim: the differences between a provident fund and a policy",
        "מנורה מבטחים: ההבדלים בין קופת גמל להשקעה לפוליסת חיסכון",
        "https://www.menoramivt.co.il/general/articles-fellow/gemel-invest-differnces",
    )
}

fn investor_gov_fees() -> Page {
    page(
        "SEC's investor.gov: how fees and expenses affect your portfolio",
        "רשות ניירות הערך האמריקאית: איך דמי ניהול והוצאות משפיעים על התיק",
        "https://www.investor.gov/introduction-investing/general-resources/news-alerts/alerts-bulletins/investor-bulletins/updated",
    )
}

fn spglobal_sp500() -> Page {
    page(
        "S&P Dow Jones Indices: the S&P 500",
        "S&P Dow Jones Indices: מדד ה\u{2011}S&P\u{a0}500",
        "https://www.spglobal.com/spdji/en/indices/equity/sp-500/",
    )
}

fn tase_securities() -> Page {
    page(
        "Tel Aviv Stock Exchange: securities and derivatives",
        "הבורסה בתל אביב: ניירות ערך ונגזרות",
        "https://www.tase.co.il/en/content/knowledge_center/securities_derivatives",
    )
}

fn tase_trading_days() -> Page {
    page(
        "Tel Aviv Stock Exchange: trading days and hours",
        "הבורסה בתל אביב: ימי ושעות המסחר",
        "https://www.tase.co.il/en/content/knowledge_center/trading_vacation_schedule",
    )
}

fn emergency_fund_rule() -> Page {
    page(
        "Phoenix: an emergency fund of 3 to 6 months' expenses (a rule of thumb)",
        "הפניקס: קרן חירום של 3 עד 6 חודשי הוצאות (כלל אצבע)",
        "https://www.fnx.co.il/gemel/gemel-information/financial-emergency-fund/",
    )
}

fn ibi_currency_faq() -> Page {
    page(
        "IBI's currency FAQ: no fee, a spread in each customer's appendix",
        "שאלות ותשובות של IBI על מט״ח: אין עמלה, יש מרווח בנספח של כל לקוח",
        "https://www.ibi.co.il/solutions/trading/forms-important-information/foreign-currency-faq/",
    )
}

fn altshuler_currency_page() -> Page {
    page(
        "Altshuler's currency page: the rate shown is the market's plus the markup",
        "עמוד המט״ח של אלטשולר: השער שמוצג הוא שער השוק בתוספת המרווח",
        "https://www.as-invest.co.il/trade/currency_exchange/",
    )
}

fn calcalist_etf_fees() -> Page {
    page(
        "Calcalist on the fees of Israeli and foreign ETFs (January 2025)",
        "כלכליסט על דמי הניהול של קרנות סל ישראליות וזרות (ינואר 2025)",
        "https://www.calcalist.co.il/market/article/r140625p1g",
    )
}

fn shibolet_surtax() -> Page {
    page(
        "Shibolet on the surtax (מס יסף)",
        "שיבולת על מס יסף",
        "https://www.shibolet.com/הטלת-מס-יסף-נוסף-בשיעור-של-2-על-הכנסה-ממק/",
    )
}

fn ordinance() -> Page {
    funds::income_tax_ordinance()
}

fn maya() -> Page {
    short_term::money_market::maya()
}

fn the_estate_rule() -> Page {
    page(
        "IRS: nonresidents with US assets must file an estate tax return",
        "רשות המסים האמריקאית: מי שאינם תושבים ויש להם נכסים בארה״ב חייבים בדוח מס עיזבון",
        "https://www.irs.gov/individuals/international-taxpayers/some-nonresidents-with-us-assets-must-file-estate-tax-returns",
    )
}

// ─────────────────────────── The cards ───────────────────────────

#[allow(clippy::too_many_lines, reason = "the cards' text")]
fn card(term: Term) -> Card {
    match term {
        Term::Etf => etf(),
        Term::IndexFund => index_fund(),
        Term::MutualFund => mutual_fund(),
        Term::MoneyMarketFund => money_market_fund(),
        Term::FixedDeposit => fixed_deposit(),
        Term::CpiLinkedDeposit => cpi_linked_deposit(),
        Term::Makam => makam(),
        Term::GovernmentBond => government_bond(),
        Term::Sp500Index => sp500_index(),
        Term::InvestmentGemel => investment_gemel(),
        Term::StudyFund => study_fund(),
        Term::SavingsPolicy => savings_policy(),
        Term::SavingsGemel => savings_gemel(),
        Term::ManagementFee => management_fee(),
        Term::CustodyAndHandling => custody_and_handling(),
        Term::TradeCommission => trade_commission(),
        Term::ConversionMarkup => conversion_markup(),
        Term::CompoundInterest => compound_interest(),
        Term::Inflation => inflation(),
        Term::CapitalGainsTax => capital_gains_tax(),
        Term::YieldToMaturity => yield_to_maturity(),
        Term::BankOfIsraelRate => bank_of_israel_rate(),
        Term::Pension => pension(),
        Term::DepositCeiling => deposit_ceiling(),
        Term::Cushion => cushion(),
        Term::BrokerageAccount => brokerage_account(),
        Term::Exchange => exchange(),
    }
}

fn etf() -> Card {
    Card {
        name: t("ETF", "קרן\u{a0}סל"),
        english: "ETF (exchange-traded fund)",
        bank: t(
            "A fund you buy through the securities account, by its name and number, as you buy a share.",
            "קרן שקונים דרך חשבון ניירות הערך, לפי השם והמספר שלה, כמו שקונים מניה.",
        ),
        ads: t(
            "\u{201c}ETF\u{201d}, and, until 2018, \u{201c}תעודת סל\u{201d}.",
            "״ETF״, ועד 2018 ״תעודת סל״.",
        ),
        what: t(
            "A fund that holds a basket of many securities, usually to follow an index such as the S&P 500, and trades on the stock exchange all day like a share, at the price of that moment.",
            "קרן שמחזיקה סל של ניירות ערך רבים, בדרך כלל כדי לעקוב אחרי מדד מניות, כמו מדד S&P\u{a0}500, ונסחרת בבורסה לאורך כל יום המסחר כמו מניה, במחיר של אותו רגע.",
        ),
        good_for: t(
            "Spreading the money over hundreds of companies in one purchase, and buying or selling at any moment of the trading day, in any amount.",
            "פיזור הכסף על מאות חברות בקנייה אחת, וקנייה או מכירה בכל רגע של יום המסחר, בכל סכום.",
        ),
        downside: t(
            "It follows the index, so it falls when the index falls. It also never matches the index exactly, and every purchase and sale pays the broker's commission, so buying a little every month adds up.",
            "היא עוקבת אחרי מדד המניות שלה, ולכן יורדת כשהוא יורד. היא גם לא תואמת אותו בדיוק, וכל קנייה ומכירה עולה עמלה לבנק או לבית ההשקעות, ולכן קנייה קטנה בכל חודש מצטברת.",
        ),
        cost: t(
            "A yearly management fee, taken from the fund so you never see a bill, and the broker's commission on every purchase and sale. The fees are in the app's table, by fund.",
            "דמי ניהול שנתיים, שיורדים מהקרן בלי שתראו חיוב, ועמלת קנייה ומכירה לכל פעולה. דמי הניהול מופיעים בטבלה, לפי קרן.",
        ),
        tax: t(
            "A Tel Aviv ETF, like any security, pays 25% of the real gain when you sell: the gain beyond the rise in prices.",
            "על קרן סל בבורסה בתל אביב, כמו על כל נייר ערך, משלמים 25% מהרווח הריאלי כשמוכרים: הרווח שמעבר לעליית המחירים.",
        ),
        where_met: t(
            "In the securities account of a bank or an investment house, and in articles that compare it with an index fund. Before 2018 it was a תעודת סל, a note rather than a fund.",
            "בחשבון ניירות הערך של בנק או בית השקעות, ובכתבות שמשוות אותה לקרן מחקה. עד 2018 היא הייתה תעודת סל, שטר חוב ולא קרן.",
        ),
        example: t(
            "For illustration: a fund whose management fee is 0.5% a year takes \u{20aa}50 a year from \u{20aa}10,000, with no bill.",
            "להמחשה: קרן שדמי הניהול שלה 0.5% בשנה לוקחת \u{20aa}50 בשנה מכל \u{20aa}10,000, בלי חיוב.",
        ),
        compare_with: &[Term::IndexFund, Term::MutualFund, Term::Sp500Index],
        sources: vec![themarker_etf_and_index(), etf_reform(), ordinance()],
    }
}

fn index_fund() -> Card {
    Card {
        name: t("Index fund", "קרן\u{a0}מחקה"),
        english: "Index-tracking mutual fund",
        bank: t(
            "A mutual fund, bought through the bank or the investment house with its name, for any amount.",
            "קרן נאמנות שקונים דרך הבנק או בית ההשקעות לפי שמה, בכל סכום.",
        ),
        ads: t(
            "\u{201c}קרן מחקה\u{201d} or \u{201c}קרן עוקבת מדד\u{201d}.",
            "״קרן מחקה״ או ״קרן עוקבת מדד״.",
        ),
        what: t(
            "A mutual fund that follows an index. Unlike an ETF it isn't traded on the exchange: you buy it from the fund's manager and sell it back to them, once a day, at the price set after the exchange closes.",
            "קרן נאמנות שעוקבת אחרי מדד מניות, כמו מדד S&P\u{a0}500. בניגוד לקרן סל, היא לא נסחרת בבורסה: קונים אותה ממנהל הקרן ומוכרים לו בחזרה, פעם ביום, במחיר שנקבע אחרי סגירת המסחר.",
        ),
        good_for: t(
            "The same spread over an index as an ETF, with no need to watch the price during the day, which suits a monthly purchase.",
            "אותו פיזור על מאות חברות כמו בקרן סל, בלי צורך לעקוב אחרי המחיר במהלך היום, וזה מתאים לקנייה חודשית.",
        ),
        downside: t(
            "An order is priced once a day, so you don't know the price when you give it. And it never matches the index exactly.",
            "פקודה מתומחרת פעם ביום, ולכן אתם לא יודעים את המחיר כשאתם נותנים אותה. וגם היא לא תואמת את מדד המניות שלה בדיוק.",
        ),
        cost: t(
            "A yearly management fee, taken from the fund, and the broker's commission, which many brokers price differently from an ETF's. Both are in the app's table.",
            "דמי ניהול שנתיים, שיורדים מהקרן, ועמלת קנייה ומכירה, שבהרבה בנקים ובתי השקעות מתומחרת אחרת מזו של קרן סל. שניהם מופיעים בטבלה.",
        ),
        tax: t(
            "25% of the real gain when you sell, like any security.",
            "25% מהרווח הריאלי כשמוכרים, כמו על כל נייר ערך.",
        ),
        where_met: t(
            "Under \u{201c}mutual funds\u{201d} in the bank's app and on its price list, and in articles that compare it with an ETF.",
            "בין ״קרנות נאמנות״ באפליקציה של הבנק ובתעריפון שלו, ובכתבות שמשוות אותה לקרן סל.",
        ),
        example: t(
            "For illustration: an order you give at 10:00 is bought at the price set after the close, not at 10:00's.",
            "להמחשה: פקודה שנתתם בעשר בבוקר מתבצעת במחיר שנקבע אחרי הסגירה, לא במחיר של עשר בבוקר.",
        ),
        compare_with: &[Term::Etf, Term::MutualFund, Term::Sp500Index],
        sources: vec![themarker_etf_and_index(), maya(), ordinance()],
    }
}

fn mutual_fund() -> Card {
    Card {
        name: t("Mutual fund", "קרן\u{a0}נאמנות"),
        english: "Mutual fund",
        bank: t(
            "\u{201c}Mutual funds\u{201d}: the heading under which the bank lists funds of all kinds.",
            "״קרנות נאמנות״: הכותרת שהבנק מציג תחתיה קרנות מכל הסוגים.",
        ),
        ads: t(
            "\u{201c}קרן נאמנות\u{201d}, often meaning a fund whose manager picks the shares.",
            "״קרן נאמנות״, ולא פעם הכוונה לקרן שמנהל קרן בוחר בה מניות.",
        ),
        what: t(
            "The family: a fund that pools many savers' money, which a manager invests and a trustee oversees. Index funds, ETFs and money market funds are all mutual funds. Where the manager picks the investments instead of following an index it's a managed fund (קרן מנוהלת).",
            "המשפחה: קרן שאוספת כסף מחוסכים רבים, שמנהל משקיע ונאמן מפקח עליו. קרנות מחקות, קרנות סל וקרנות כספיות הן כולן קרנות נאמנות. כשהמנהל בוחר את ההשקעות במקום לעקוב אחרי מדד מניות, כמו מדד S&P\u{a0}500, זו קרן מנוהלת.",
        ),
        good_for: t(
            "Putting the money to work without choosing securities yourself.",
            "להעמיד את הכסף לעבודה בלי לבחור בעצמכם ניירות ערך.",
        ),
        downside: t(
            "A managed fund can't promise to beat the index, and the app doesn't compare managed funds: it compares funds that follow an index.",
            "קרן מנוהלת לא יכולה להבטיח לנצח את מדד המניות שמולו היא נמדדת, והמחשבון לא משווה קרנות מנוהלות: הוא משווה קרנות שעוקבות אחרי מדד S&P\u{a0}500.",
        ),
        cost: t(
            "A yearly management fee from the fund, which differs a lot between funds: see each fund's page in the table.",
            "דמי ניהול שנתיים מהקרן, שמשתנים מאוד בין קרנות: ראו את הדף של כל קרן בטבלה.",
        ),
        tax: t(
            "25% of the real gain when you sell, like any security.",
            "25% מהרווח הריאלי כשמוכרים, כמו על כל נייר ערך.",
        ),
        where_met: t(
            "In every bank's app, in the funds' reports to the exchange, and in the news about the funds' yields.",
            "בכל אפליקציה של בנק, בדיווחים של הקרנות לבורסה, ובכתבות על תשואות הקרנות.",
        ),
        example: t(
            "For illustration: a managed fund and an index fund are both mutual funds. The difference is who chooses what's in them.",
            "להמחשה: קרן מנוהלת וקרן מחקה הן שתיהן קרנות נאמנות. ההבדל הוא מי בוחר מה בתוכן.",
        ),
        compare_with: &[Term::IndexFund, Term::Etf, Term::MoneyMarketFund],
        sources: vec![themarker_etf_and_index(), maya()],
    }
}

fn money_market_fund() -> Card {
    Card {
        name: t("Money market fund", "קרן\u{a0}כספית"),
        english: "Money market fund",
        bank: t(
            "A mutual fund with a name like \u{201c}כספית שקלית\u{201d}, for example \u{201c}דולפין כספית שקלית\u{201d}.",
            "קרן נאמנות בשם כמו ״כספית שקלית״, למשל ״דולפין כספית שקלית״.",
        ),
        ads: t(
            "Shown as an alternative to a bank deposit.",
            "מוצגת כחלופה לפיקדון בבנק.",
        ),
        what: t(
            "A fund that lends the money for short periods, to the state and the banks, so it earns about the Bank of Israel's rate. You buy and sell it like a security, and the money isn't locked.",
            "קרן שמלווה את הכסף לזמן קצר, למדינה ולבנקים, ולכן מרוויחה בערך את ריבית בנק ישראל. קונים ומוכרים אותה כמו נייר ערך, והכסף לא נעול.",
        ),
        good_for: t(
            "Money that waits: a cushion, money for an apartment within the year, or a sum you haven't decided about. You can sell on any trading day, and banks may not charge custody on it.",
            "כסף שמחכה: כרית ביטחון, כסף לדירה בשנה הקרובה, או סכום שעוד לא החלטתם מה לעשות איתו. אפשר למכור בכל יום מסחר, ובנקים לא רשאים לגבות עליה דמי משמרת.",
        ),
        downside: t(
            "Its return isn't fixed: when the Bank of Israel cuts its rate the fund earns less, and it can earn less than inflation. Money you need in five years is better in something that promises a sum, such as a bond held to maturity.",
            "התשואה שלה לא קבועה: כשבנק ישראל מוריד את הריבית הקרן מרוויחה פחות, והיא יכולה להרוויח פחות מהאינפלציה. כסף שצריך בעוד חמש שנים עדיף במשהו שמבטיח סכום, כמו אג״ח שמחזיקים עד הפדיון.",
        ),
        cost: t(
            "A management fee and a trustee's fee, both taken from the fund. In a bank there's no custody fee; an investment house may charge a commission on each purchase. The fees by fund are in the app's table.",
            "דמי ניהול ודמי נאמנות, ששניהם יורדים מהקרן. בבנק אין דמי משמרת; בבית השקעות יכולה להיות עמלה על כל קנייה. דמי הניהול לפי קרן מופיעים בטבלה.",
        ),
        tax: t(
            "25% of the real gain when you sell, the gain beyond inflation. A bank deposit's interest, by contrast, pays 15% of all of it.",
            "25% מהרווח הריאלי כשמוכרים, הרווח שמעבר לאינפלציה. על הריבית בפיקדון בבנק, לעומת זאת, משלמים 15% מכולה.",
        ),
        where_met: t(
            "In the bank's app under \u{201c}mutual funds\u{201d}, with names that say \u{201c}כספית\u{201d}, and in the news when the Bank of Israel changes its rate.",
            "באפליקציה של הבנק בין קרנות הנאמנות, בשמות שכתוב בהם ״כספית״, ובכתבות כשבנק ישראל משנה את הריבית.",
        ),
        example: t(
            "For illustration: a fund that earns 3% a year with a fee of 0.1% leaves 2.9%. On \u{20aa}30,000 for half a year that's about \u{20aa}435 before tax.",
            "להמחשה: קרן שמרוויחה 3% בשנה ודמי הניהול שלה 0.1% משאירה 2.9%. על \u{20aa}30,000 לחצי שנה זה בערך \u{20aa}435 לפני מס.",
        ),
        compare_with: &[Term::FixedDeposit, Term::Makam, Term::Cushion],
        sources: vec![
            boi_money_fund(),
            tradingil_money_fund(),
            boi_custody_rule(),
            maya(),
            ordinance(),
        ],
    }
}

fn fixed_deposit() -> Card {
    Card {
        name: t("Fixed-rate deposit", "פיקדון\u{a0}בריבית\u{a0}קבועה"),
        english: "Fixed-rate deposit",
        bank: t(
            "\u{201c}Deposit\u{201d}, with a term and a rate agreed when you open it.",
            "״פיקדון״, עם תקופה וריבית שקובעים כשפותחים אותו.",
        ),
        ads: t(
            "\u{201c}Guaranteed interest\u{201d}, and the bank's offer for new money.",
            "״ריבית מובטחת״, וההצעה של הבנק לכסף חדש.",
        ),
        what: t(
            "Money you leave with a bank for a set term, at a rate fixed when you open the deposit, so you know the sum you get at the end.",
            "כסף שמשאירים בבנק לתקופה קבועה, בריבית שנקבעת כשפותחים את הפיקדון, ולכן יודעים מראש כמה יהיה בסוף.",
        ),
        good_for: t(
            "Money for a date you know, when you want the sum known in advance and don't expect to need the money sooner.",
            "כסף לתאריך שאתם יודעים, כשרוצים לדעת את הסכום מראש ולא מצפים להזדקק לכסף קודם.",
        ),
        downside: t(
            "The money is locked to the end of the term, and taking it out sooner is only on the bank's terms: read them before you open it. It isn't linked to inflation, so in a high-inflation year it can lose value.",
            "הכסף נעול עד סוף התקופה, ומשיכה מוקדמת היא רק בתנאי הבנק: קראו אותם לפני שפותחים. הוא לא צמוד לאינפלציה, ולכן בשנה של אינפלציה גבוהה הוא יכול לאבד מערכו.",
        ),
        cost: t(
            "No fee: the cost is the rate the bank pays you compared with the Bank of Israel's. Banks' rates by term are in the app's table.",
            "אין עמלה: העלות היא הפער בין הריבית שהבנק משלם לריבית בנק ישראל. הריביות של הבנקים לפי תקופה מופיעות בטבלה.",
        ),
        tax: t(
            "15% of all the interest, whether or not there was inflation, for a deposit not linked to the index.",
            "15% מכל הריבית, בין אם הייתה אינפלציה ובין אם לא, על פיקדון שאינו צמוד למדד המחירים לצרכן.",
        ),
        where_met: t(
            "On the bank's website and app under savings and deposits, and in the Bank of Israel's comparison of the banks' rates.",
            "באתר ובאפליקציה של הבנק תחת חיסכון ופיקדונות, ובהשוואת הריביות של בנק ישראל.",
        ),
        example: t(
            "For illustration: \u{20aa}20,000 for 12 months at 4% earns \u{20aa}800, tax at 15% is \u{20aa}120, and \u{20aa}20,680 is left.",
            "להמחשה: \u{20aa}20,000 ל\u{2011}12 חודשים ב\u{2011}4% מרוויחים \u{20aa}800, המס ב\u{2011}15% הוא \u{20aa}120, ונשארים \u{20aa}20,680.",
        ),
        compare_with: &[
            Term::MoneyMarketFund,
            Term::CpiLinkedDeposit,
            Term::GovernmentBond,
        ],
        sources: vec![boi_deposits(), ordinance(), bizportal_tax_on_interest()],
    }
}

fn cpi_linked_deposit() -> Card {
    Card {
        name: t(
            "CPI-linked deposit",
            "פיקדון\u{a0}צמוד למדד\u{a0}המחירים\u{a0}לצרכן",
        ),
        english: "CPI-linked deposit",
        bank: t(
            "\u{201c}Deposit linked to the index\u{201d}, with a fixed rate on top of the linkage.",
            "״פיקדון צמוד מדד״, עם ריבית קבועה מעל ההצמדה.",
        ),
        ads: t(
            "\u{201c}Protection from inflation\u{201d}.",
            "״הגנה מפני אינפלציה״.",
        ),
        what: t(
            "A deposit whose sum rises with the consumer price index (the מדד המחירים לצרכן), plus a fixed interest rate on top, so its buying power is kept and it earns something more.",
            "פיקדון שהסכום שלו עולה עם מדד המחירים לצרכן, ועוד ריבית קבועה מעליו, כך שכוח הקנייה נשמר והוא מרוויח משהו נוסף.",
        ),
        good_for: t(
            "Money kept for years that must not lose buying power.",
            "כסף ששומרים שנים, שאסור שיאבד מכוח הקנייה שלו.",
        ),
        downside: t(
            "Locked to its term. If prices fall the sum falls with them. And it isn't a kind the app compares, because the Bank of Israel doesn't publish its rates and only one bank's published rate, without a date, was found. A government bond linked to the index is what the app compares for the same need.",
            "נעול עד סוף התקופה. אם המחירים יורדים, הסכום יורד איתם. והוא לא אחד מהסוגים שהמחשבון משווה, כי בנק ישראל לא מפרסם את הריביות שלו ורק בנק אחד פרסם ריבית, בלי תאריך. אג״ח צמודה של מדינת ישראל היא מה שהמחשבון משווה לאותו צורך.",
        ),
        cost: t(
            "No fee. The interest above the linkage differs between banks and offers, so ask the bank.",
            "אין עמלה. הריבית שמעל ההצמדה שונה בין בנקים ובין הצעות, ולכן שאלו את הבנק.",
        ),
        tax: t(
            "25% of the real gain: what the deposit earns beyond the rise in prices.",
            "25% מהרווח הריאלי: מה שהפיקדון מרוויח מעבר לעליית המחירים.",
        ),
        where_met: t(
            "On the bank's website, as \u{201c}צמוד מדד\u{201d} deposits and savings plans.",
            "באתר של הבנק, כפיקדונות וכתוכניות חיסכון ״צמוד מדד״.",
        ),
        example: t(
            "For illustration: at 1% above the index and inflation of 2% a year, the deposit grows about 3% a year, of which 2% only keeps its buying power.",
            "להמחשה: בריבית של 1% מעל מדד המחירים לצרכן ובאינפלציה של 2% בשנה, הפיקדון גדל בערך 3% בשנה, ו\u{2011}2% מהם רק שומרים על כוח הקנייה.",
        ),
        compare_with: &[Term::FixedDeposit, Term::GovernmentBond, Term::Inflation],
        sources: vec![
            bizportal_tax_on_interest(),
            leumi_linked_savings(),
            ordinance(),
        ],
    }
}

fn makam() -> Card {
    Card {
        name: t("Makam (short-term loan)", "מק״מ"),
        english: "Makam (Treasury-style bill of the Bank of Israel)",
        bank: t(
            "\u{201c}מלווה קצר מועד\u{201d}, bought as a security with its series name.",
            "״מלווה קצר מועד״, שקונים כנייר ערך לפי שם הסדרה.",
        ),
        ads: t(
            "\u{201c}מק״מ\u{201d}, in the news about money market funds, which hold it.",
            "״מק״מ״, בכתבות על קרנות כספיות, שמחזיקות בו.",
        ),
        what: t(
            "A bill the Bank of Israel issues for up to a year. It pays no interest along the way: it's sold below its face value and repaid at its face value, and the difference is your interest.",
            "שטר שבנק ישראל מנפיק לעד שנה. הוא לא משלם ריבית בדרך: הוא נמכר מתחת לערכו הנקוב ונפדה בערכו הנקוב, וההפרש הוא הריבית שלכם.",
        ),
        good_for: t(
            "A sum you know you won't need for a few months, with a return known in advance.",
            "סכום שאתם יודעים שלא תצטרכו כמה חודשים, עם תשואה שידועה מראש.",
        ),
        downside: t(
            "It isn't linked to inflation, and it's bought through a broker, who sets the commission. The app doesn't compare it: a money market fund, which holds makam, covers the need.",
            "הוא לא צמוד לאינפלציה, וקונים אותו דרך בנק או בית השקעות, שקובעים את העמלה. המחשבון לא משווה אותו: קרן כספית, שמחזיקה מק״מ, מכסה את אותו צורך.",
        ),
        cost: t(
            "A bank may not charge custody on it. The commission for buying depends on the broker.",
            "בנק לא רשאי לגבות עליו דמי משמרת. עמלת הקנייה תלויה בבנק או בבית ההשקעות.",
        ),
        tax: t(
            "15% of the interest, which is the discount, not the real gain.",
            "15% מהריבית, שהיא ההנחה במחיר, ולא מהרווח הריאלי.",
        ),
        where_met: t(
            "In a securities account, by series name, and in articles about money market funds.",
            "בחשבון ניירות ערך, לפי שם הסדרה, ובכתבות על קרנות כספיות.",
        ),
        example: t(
            "For illustration: a bill of \u{20aa}1,000 bought at \u{20aa}985 and repaid at \u{20aa}1,000 earns \u{20aa}15; the tax is 15% of it, \u{20aa}2.25.",
            "להמחשה: שטר של \u{20aa}1,000 שנקנה ב\u{2011}\u{20aa}985 ונפדה ב\u{2011}\u{20aa}1,000 מרוויח \u{20aa}15; המס הוא 15% מהם, \u{20aa}2.25.",
        ),
        compare_with: &[
            Term::MoneyMarketFund,
            Term::FixedDeposit,
            Term::GovernmentBond,
        ],
        sources: vec![boi_makam(), boi_custody_rule(), ordinance()],
    }
}

fn government_bond() -> Card {
    Card {
        name: t("Government bond", "אג״ח\u{a0}ממשלתית"),
        english: "Government bond held to maturity",
        bank: t(
            "A security with the kind and the month and year it's repaid in its name: \u{201c}ממשלתי שקלי 0347\u{201d} is repaid in March 2047.",
            "נייר ערך שהסוג והחודש והשנה של הפדיון שלו בשמו: ״ממשלתי שקלי 0347״ נפדה במרץ 2047.",
        ),
        ads: t(
            "\u{201c}אג״ח ממשלתי\u{201d}, and \u{201c}אג״ח ממשלתי צמוד\u{201d} for the linked kind.",
            "״אג״ח ממשלתי״, ו״אג״ח ממשלתי צמוד״ לסוג הצמוד.",
        ),
        what: t(
            "A loan to the state that you buy on the exchange. If you hold it until it's repaid (בפדיון) you get back its face value, and the interest on the way, so the sum at the end is known when you buy.",
            "הלוואה למדינה שקונים בבורסה. אם מחזיקים אותה עד הפדיון מקבלים בחזרה את הערך הנקוב, ואת הריבית בדרך, ולכן הסכום בסוף ידוע כבר בקנייה.",
        ),
        good_for: t(
            "Money for a date years away that you want known in advance, or want linked to inflation (a linked bond).",
            "כסף לתאריך שנים קדימה שרוצים לדעת מראש, או שרוצים צמוד לאינפלציה (אג״ח צמוד).",
        ),
        downside: t(
            "If you sell before it's repaid you get the day's price, which can be lower than you paid when rates have risen. An unlinked bond loses to inflation. And buying costs the broker's commission.",
            "אם מוכרים לפני הפדיון מקבלים את מחיר היום, שיכול להיות נמוך ממה ששילמתם כשהריבית עלתה. אג״ח לא צמוד מפסיד לאינפלציה. והקנייה עולה עמלה לבנק או לבית ההשקעות.",
        ),
        cost: t(
            "The broker's commission on a bond on the Tel Aviv exchange, by plan: see the table.",
            "עמלת הקנייה של הבנק או בית ההשקעות על אג״ח בבורסה בתל אביב, לפי מסלול: ראו בטבלה.",
        ),
        tax: t(
            "A bond not linked to the index pays 15% of the whole gain; a linked one, 25% of the real gain.",
            "על אג״ח שאינו צמוד למדד משלמים 15% מכל הרווח; על אג״ח צמוד, 25% מהרווח הריאלי.",
        ),
        where_met: t(
            "In a securities account, in the lists of the exchange, and in articles about the state's debt.",
            "בחשבון ניירות ערך, ברשימות של הבורסה, ובכתבות על חוב המדינה.",
        ),
        example: t(
            "For illustration: \u{201c}ממשלתי צמוד 0536\u{201d} is a linked bond repaid in May 2036; you know its date from its name, and its sum rises with the index.",
            "להמחשה: ״ממשלתי צמוד 0536״ הוא אג״ח צמוד שנפדה במאי 2036; את התאריך שלו יודעים מהשם, והסכום שלו עולה עם המדד.",
        ),
        compare_with: &[
            Term::YieldToMaturity,
            Term::FixedDeposit,
            Term::MoneyMarketFund,
        ],
        sources: vec![
            themarker_bond_0347(),
            bizportal_tax_on_interest(),
            boi_yield_curve(),
            ordinance(),
        ],
    }
}

fn sp500_index() -> Card {
    Card {
        name: t("S&P 500 index", "מדד\u{a0}S&P\u{a0}500"),
        english: "S&P 500 index",
        bank: t(
            "Part of the name of a fund: \u{201c}עוקב S&P 500\u{201d} or \u{201c}מחקה S&P 500\u{201d}.",
            "חלק משם של קרן: ״עוקבת S&P 500״ או ״מחקה S&P 500״.",
        ),
        ads: t(
            "\u{201c}The S&P\u{201d}, \u{201c}the American market\u{201d}.",
            "״ה\u{2011}S&P״, ״השוק האמריקאי״.",
        ),
        what: t(
            "A list of 500 of the biggest companies in the United States, weighted by their size, whose combined value is followed as one number. A fund that follows it holds a part of each.",
            "רשימה של 500 מהחברות הגדולות בארה״ב, שקובעים לפי הגודל שלהן את משקל כל אחת, ושהשווי המשותף שלהן נמדד כמספר אחד. קרן שעוקבת אחריו מחזיקה חלק מכל אחת מהן.",
        ),
        good_for: t(
            "Spreading the money over the American market in one fund, with no need to choose companies.",
            "פיזור הכסף על השוק האמריקאי בקרן אחת, בלי לבחור חברות.",
        ),
        downside: t(
            "It's only the United States, in dollars, so it moves with the exchange rate too, and it has fallen by large amounts, sometimes for years. It's a bet on one market.",
            "זה רק ארה״ב, בדולרים, ולכן הוא זז גם עם שער החליפין, והוא ירד בשיעורים גדולים, לפעמים שנים. זה הימור על שוק אחד.",
        ),
        cost: t(
            "The index costs nothing; what it costs is the fund that follows it: its management fee and what it fails to track. See the fees in the table.",
            "מדד S&P\u{a0}500 עצמו לא עולה כלום; מה שעולה הוא הקרן שעוקבת אחריו: דמי הניהול שלה וכמה שהיא לא עוקבת בדיוק. ראו בטבלה.",
        ),
        tax: t(
            "As the fund holding it: 25% of the real gain when you sell. In a fund registered in the United States there's also the American estate tax.",
            "כמו הקרן שמחזיקה אותו: 25% מהרווח הריאלי כשמוכרים. בקרן שרשומה בארה״ב יש גם מס עיזבון אמריקאי.",
        ),
        where_met: t(
            "In the names of funds, in ads for funds and in the news, as \u{201c}the S&P\u{201d}.",
            "בשמות של קרנות, בפרסומות לקרנות ובחדשות, בשם ״ה\u{2011}S&P״.",
        ),
        example: t(
            "For illustration: when the biggest companies in it rise, it rises more than when a small one does, because it's weighted by size.",
            "להמחשה: כשהחברות הגדולות בו עולות הוא עולה יותר מאשר כשחברה קטנה עולה, כי המשקל לפי גודל.",
        ),
        compare_with: &[Term::Etf, Term::IndexFund, Term::Exchange],
        sources: vec![
            spglobal_sp500(),
            crate::history::source(),
            the_estate_rule(),
            ordinance(),
        ],
    }
}

fn investment_gemel() -> Card {
    Card {
        name: t("Provident fund for investment", "קופת\u{a0}גמל\u{a0}להשקעה"),
        english: "Provident fund for investment (kupat gemel lehashkaa)",
        bank: t(
            "\u{201c}קופת גמל להשקעה\u{201d}, opened with a fund company or a bank, with an investment track such as \u{201c}עוקב מדד S&P 500\u{201d}.",
            "״קופת גמל להשקעה״, שפותחים אצל חברה מנהלת או בבנק, עם מסלול השקעה כמו ״עוקב מדד S&P 500״.",
        ),
        ads: t(
            "\u{201c}A fund that gives a tax-free pension\u{201d}.",
            "״קופה שנותנת קצבה פטורה ממס״.",
        ),
        what: t(
            "A savings account run by a fund company, with the money invested in the track you choose. Anyone can open one, and you can take the money out any time.",
            "חשבון חיסכון שחברה מנהלת מנהלת, כשהכסף מושקע במסלול שאתם בוחרים. כל אחד יכול לפתוח אחת, ואפשר למשוך את הכסף בכל עת.",
        ),
        good_for: t(
            "Long saving that you may need before 60 but would like to end as a pension: from 60, taken as a monthly pension, the gain is tax-free.",
            "חיסכון ארוך שאולי תצטרכו לפני גיל 60 אבל הייתם רוצים שיסתיים כקצבה: מגיל 60, כשמושכים כקצבה חודשית, הרווח פטור ממס.",
        ),
        downside: t(
            "A ceiling on what you can deposit each year, no tax benefit on the way in, and withdrawing everything at once, or before 60, pays 25% of the real gain. The monthly pension isn't paid by the fund itself.",
            "יש תקרה למה שמפקידים בשנה, אין הטבת מס בהפקדה, ומשיכה של הכול בבת אחת או לפני גיל 60 משלמת 25% מהרווח הריאלי. את הקצבה החודשית לא משלמת הקופה עצמה.",
        ),
        cost: t(
            "A management fee from the balance, and sometimes from each deposit, with a legal maximum. The app's table has the average, the cheapest and the dearest company.",
            "דמי ניהול מהצבירה, ולפעמים גם מכל הפקדה, עם תקרה בחוק. בטבלה מופיעים הממוצע, החברה הזולה והחברה היקרה.",
        ),
        tax: t(
            "25% of the real gain on a lump sum or a withdrawal before 60; a monthly pension from 60 is tax-free on the gain.",
            "25% מהרווח הריאלי במשיכה בבת אחת או לפני גיל 60; קצבה חודשית מגיל 60 פטורה ממס על הרווח.",
        ),
        where_met: t(
            "At fund companies and banks, in ads for \u{201c}קופת גמל להשקעה\u{201d}, and in articles about the pension exemption.",
            "אצל חברות הגמל ובבנקים, בפרסומות ל״קופת גמל להשקעה״ ובכתבות על הפטור ממס בקצבה.",
        ),
        example: t(
            "For illustration: with a gain of \u{20aa}200,000, a lump sum pays 25% of the real gain; taken as a pension from 60, the gain pays nothing.",
            "להמחשה: ברווח של \u{20aa}200,000, משיכה בבת אחת משלמת 25% מהרווח הריאלי; כשמושכים כקצבה מגיל 60, הרווח לא משלם דבר.",
        ),
        compare_with: &[Term::StudyFund, Term::SavingsPolicy, Term::BrokerageAccount],
        sources: vec![
            kolzchut_investment_gemel(),
            kolzchut_pension_exemption(),
            funds::gemel_net(),
            ordinance(),
        ],
    }
}

fn study_fund() -> Card {
    Card {
        name: t("Study fund", "קרן\u{a0}השתלמות"),
        english: "Study fund (keren hishtalmut)",
        bank: t(
            "\u{201c}קרן השתלמות\u{201d}, usually opened through the employer.",
            "״קרן השתלמות״, שבדרך כלל נפתחת דרך המעסיק.",
        ),
        ads: t(
            "A saving the employer deposits into too, with the gain tax-free.",
            "חיסכון שגם המעסיק מפקיד לו, והרווח בו פטור ממס.",
        ),
        what: t(
            "A savings fund for the medium term. An employee opens one with the employer, who deposits with them; a self-employed saver opens one alone. After six years the money can be taken out.",
            "קרן חיסכון לטווח בינוני. שכיר פותח אחת דרך המעסיק, שמפקיד איתו; עצמאי פותח אחת לבד. אחרי שש שנים אפשר למשוך את הכסף.",
        ),
        good_for: t(
            "Saving for six years or more, where the gain is tax-free up to a yearly ceiling.",
            "חיסכון לשש שנים ויותר, שבו הרווח פטור ממס עד תקרה שנתית.",
        ),
        downside: t(
            "Taking the money out sooner is taxed as income, except in cases the law lists such as studies. An employee needs the employer to open it, and the app counts only the self-employed's.",
            "משיכה מוקדמת ממוסה כהכנסה, חוץ ממקרים שהחוק מונה כמו לימודים. שכיר צריך את המעסיק כדי לפתוח אותה, והמחשבון מחשב רק את זו של עצמאי.",
        ),
        cost: t(
            "A management fee from the balance with a legal maximum. The table has the average, the cheapest and the dearest company.",
            "דמי ניהול מהצבירה, עם תקרה בחוק. בטבלה מופיעים הממוצע, החברה הזולה והחברה היקרה.",
        ),
        tax: t(
            "After six years, the gain on deposits up to the yearly ceiling is exempt; above it, 25% of the real gain.",
            "אחרי שש שנים הרווח על הפקדות עד התקרה השנתית פטור; מעליה, 25% מהרווח הריאלי.",
        ),
        where_met: t(
            "On the payslip, as a deduction for \u{201c}קרן השתלמות\u{201d}, and in ads for it.",
            "בתלוש השכר, כניכוי ל״קרן השתלמות״, ובפרסומות לה.",
        ),
        example: t(
            "For illustration: a saver deposits for six years, and at the end the gain on the deposits within the ceiling isn't taxed; on a withdrawal in year four it would be taxed as income.",
            "להמחשה: חוסך מפקיד שש שנים, ובסוף הרווח על ההפקדות שבתוך התקרה לא ממוסה; במשיכה בשנה הרביעית הוא היה ממוסה כהכנסה.",
        ),
        compare_with: &[
            Term::InvestmentGemel,
            Term::SavingsPolicy,
            Term::DepositCeiling,
        ],
        sources: vec![kolzchut_study_fund(), funds::gemel_net(), ordinance()],
    }
}

fn savings_policy() -> Card {
    Card {
        name: t("Savings policy", "פוליסת\u{a0}חיסכון"),
        english: "Savings policy (an insurance company's investment policy)",
        bank: t(
            "\u{201c}פוליסת חיסכון\u{201d}, sold by an insurance company, sometimes through the bank.",
            "״פוליסת חיסכון״, שחברת ביטוח מוכרת, לפעמים דרך הבנק.",
        ),
        ads: t(
            "\u{201c}Savings with no ceiling\u{201d}.",
            "״חיסכון בלי תקרה״.",
        ),
        what: t(
            "An insurance company's investment account: you deposit as much as you like, it's invested in a track, and you can take it out when you want.",
            "חשבון השקעה של חברת ביטוח: מפקידים כמה שרוצים, הכסף מושקע במסלול, ואפשר למשוך כשרוצים.",
        ),
        good_for: t(
            "Saving more than the ceiling of a provident fund for investment allows.",
            "חיסכון בסכום שגדול מהתקרה של קופת גמל להשקעה.",
        ),
        downside: t(
            "No tax-free pension, and moving to another insurer counts as a withdrawal and is taxed. Its fees are the insurers' and are less open than the provident funds'.",
            "אין קצבה פטורה ממס, ומעבר לחברת ביטוח אחרת נחשב משיכה וממוסה. דמי הניהול הם של חברות הביטוח, והם פחות גלויים מאלה של קופות הגמל.",
        ),
        cost: t(
            "A management fee from the balance. The table has the average, as a reading, and the most the insurer may take.",
            "דמי ניהול מהצבירה. בטבלה מופיע הממוצע, כהערכה, והמקסימום שחברת הביטוח רשאית לקחת.",
        ),
        tax: t(
            "25% of the real gain when you withdraw.",
            "25% מהרווח הריאלי במשיכה.",
        ),
        where_met: t(
            "At insurance companies and agents, and in articles that compare it with a provident fund for investment.",
            "בחברות ביטוח ואצל סוכנים, ובכתבות שמשוות אותה לקופת גמל להשקעה.",
        ),
        example: t(
            "For illustration: a saver who deposits more than a provident fund takes in a year can put the rest in a policy.",
            "להמחשה: מי שמפקיד יותר ממה שקופת גמל לוקחת בשנה יכול לשים את היתר בפוליסה.",
        ),
        compare_with: &[Term::InvestmentGemel, Term::StudyFund, Term::DepositCeiling],
        sources: vec![
            analyst_savings_policy(),
            menora_gemel_and_policy(),
            funds::gemel_net(),
        ],
    }
}

fn savings_gemel() -> Card {
    Card {
        name: t("Provident fund for savings", "קופת\u{a0}גמל\u{a0}לחיסכון"),
        english: "Provident fund for savings (kupat gemel, the original kind)",
        bank: t(
            "\u{201c}קופת גמל\u{201d} or \u{201c}קופת גמל לתגמולים\u{201d}, often opened through the employer.",
            "״קופת גמל״ או ״קופת גמל לתגמולים״, שנפתחת לא פעם דרך המעסיק.",
        ),
        ads: t(
            "\u{201c}Saving for a pension with a tax benefit\u{201d}.",
            "״חיסכון לפנסיה עם הטבת מס״.",
        ),
        what: t(
            "The older kind of provident fund, meant for a pension: the money is locked until 60, and a deposit up to a limit brings a tax benefit.",
            "הסוג הוותיק של קופת גמל, שנועד לפנסיה: הכסף נעול עד גיל 60, והפקדה עד סכום מסוים מזכה בהטבת מס.",
        ),
        good_for: t(
            "Pension saving, where the benefit on the way in is worth more than the freedom to withdraw.",
            "חיסכון לפנסיה, כשההטבה בהפקדה שווה יותר מהחופש למשוך.",
        ),
        downside: t(
            "The money is for retirement, and its rules (the benefit, the pension, a lump sum) depend on your income and your other pension savings.",
            "הכסף מיועד לפרישה, והכללים שלו (ההטבה, הקצבה, משיכה בבת אחת) תלויים בהכנסה שלכם ובשאר החיסכון הפנסיוני שלכם.",
        ),
        cost: t(
            "A management fee: ask the fund.",
            "דמי ניהול: שאלו את הקופה.",
        ),
        tax: t(
            "Depends on your income and the part of the deposit that is pension money; the app doesn't work it out.",
            "תלוי בהכנסה שלכם ובחלק מההפקדה שהוא כסף פנסיוני; המחשבון לא מחשב אותו.",
        ),
        where_met: t(
            "On the payslip, as a deduction for \u{201c}גמל\u{201d} or \u{201c}תגמולים\u{201d}, and in pension articles.",
            "בתלוש השכר, כניכוי ל״גמל״ או ל״תגמולים״, ובכתבות על פנסיה.",
        ),
        example: t(
            "Why it isn't in the app: counting it needs your income, your tax bracket and what your pension already uses, and the app asks for none of them. So it isn't compared, and says so.",
            "למה היא לא במחשבון: כדי לחשב אותה צריך את ההכנסה שלכם, את מדרגת המס ואת מה שהפנסיה שלכם כבר מנצלת, והמחשבון לא שואל על אף אחד מהם. לכן היא לא מושווית, והוא אומר את זה.",
        ),
        compare_with: &[Term::InvestmentGemel, Term::StudyFund, Term::Pension],
        sources: vec![kolzchut_gemel(), meitav_amendment_190()],
    }
}

fn management_fee() -> Card {
    Card {
        name: t("Management fee", "דמי\u{a0}ניהול"),
        english: "Management fee (expense ratio)",
        bank: t(
            "\u{201c}דמי ניהול\u{201d}, a yearly percentage of what you hold, in the fund's page.",
            "״דמי ניהול״, אחוז שנתי ממה שמחזיקים, בדף של הקרן.",
        ),
        ads: t(
            "\u{201c}Low fees\u{201d}, \u{201c}0% fee\u{201d}.",
            "״דמי ניהול נמוכים״, ״דמי ניהול 0%״.",
        ),
        what: t(
            "What the company that runs a fund or a provident fund takes each year, as a share of the money in it. It's taken from the fund itself, so you never get a bill: your balance simply grows a little less.",
            "מה שהחברה שמנהלת קרן או קופת גמל לוקחת בכל שנה, כאחוז מהכסף שבה. זה יורד מהקרן עצמה, ולכן אתם לא מקבלים חיוב: היתרה שלכם פשוט גדלה קצת פחות.",
        ),
        good_for: t(
            "Comparing two funds that hold the same thing: the cheaper one leaves more.",
            "להשוות בין שתי קרנות שמחזיקות את אותו דבר: הזולה משאירה יותר.",
        ),
        downside: t(
            "A small percentage looks harmless but it comes off every year, from a balance that would have earned interest on interest: over twenty years it's a large share of the end sum.",
            "אחוז קטן נראה לא מזיק, אבל הוא יורד בכל שנה מיתרה שהייתה מרוויחה ריבית דריבית: לאורך עשרים שנה הוא חלק גדול מהסכום בסוף.",
        ),
        cost: t(
            "It is itself the cost. Some funds also have a variable fee, which they take only above the index's return, and a fee from each deposit. All of them are in the table.",
            "היא עצמה העלות. לקרנות מסוימות יש גם דמי ניהול משתנים, שהן לוקחות רק מעל התשואה של מדד המניות שמולו הן נמדדות, ודמי ניהול מכל הפקדה. כולם מופיעים בטבלה.",
        ),
        tax: t(
            "It isn't taxed on its own; it lowers the gain, and so the tax on it.",
            "היא לא ממוסה בפני עצמה; היא מקטינה את הרווח, ולכן גם את המס עליו.",
        ),
        where_met: t(
            "In every fund's page and in articles that rank funds by fee.",
            "בדף של כל קרן ובכתבות שמדרגות קרנות לפי דמי ניהול.",
        ),
        example: t(
            "For illustration: \u{20aa}100,000 that earns 5% a year grows to about \u{20aa}265,300 in 20 years; with a fee of 1% a year, so 4%, it grows to about \u{20aa}219,100. The fee took about \u{20aa}46,200.",
            "להמחשה: \u{20aa}100,000 שמרוויחים 5% בשנה גדלים בעוד 20 שנה לכ\u{2011}\u{20aa}265,300; עם דמי ניהול של 1% בשנה, כלומר 4%, הם גדלים לכ\u{2011}\u{20aa}219,100. דמי הניהול לקחו כ\u{2011}\u{20aa}46,200.",
        ),
        compare_with: &[
            Term::CustodyAndHandling,
            Term::CompoundInterest,
            Term::TradeCommission,
        ],
        sources: vec![investor_gov_fees(), kolzchut_investment_gemel(), maya()],
    }
}

fn custody_and_handling() -> Card {
    Card {
        name: t("Custody and handling fee", "דמי\u{a0}משמרת\u{a0}וטיפול"),
        english: "Custody and handling fee (account fees)",
        bank: t(
            "\u{201c}דמי משמרת\u{201d} and \u{201c}דמי טיפול\u{201d}, in the price list under the securities account.",
            "״דמי משמרת״ ו״דמי טיפול״, בתעריפון תחת חשבון ניירות הערך.",
        ),
        ads: t(
            "\u{201c}No account fees for the first year\u{201d}.",
            "״בלי דמי טיפול בשנה הראשונה״.",
        ),
        what: t(
            "What a bank or an investment house charges for keeping your securities, even if you buy and sell nothing. Custody is a share of what you hold; handling is a fixed amount, usually monthly.",
            "מה שבנק או בית השקעות גובים על שמירת ניירות הערך שלכם, גם אם אתם לא קונים ולא מוכרים. דמי משמרת הם אחוז ממה שמחזיקים; דמי טיפול הם סכום קבוע, בדרך כלל לחודש.",
        ),
        good_for: t(
            "Nothing: it's a cost to compare. A plan with no custody and no handling fee can be better for a small account, and one with a low percentage for a large one.",
            "לכלום: זו עלות להשוות. מסלול בלי דמי משמרת וטיפול יכול להתאים יותר לחשבון קטן, ומסלול עם אחוז נמוך לחשבון גדול.",
        ),
        downside: t(
            "A share of the holdings grows with your savings, a fixed amount weighs most on a small account, and a joining offer may waive it only at first.",
            "אחוז מהתיק גדל עם החיסכון, סכום קבוע מכביד ביותר על חשבון קטן, ובמבצע הצטרפות לפעמים מוותרים עליו רק בהתחלה.",
        ),
        cost: t(
            "By plan, in the table. A bank may not charge custody on a money market fund or on makam.",
            "לפי מסלול, בטבלה. בנק לא רשאי לגבות דמי משמרת על קרן כספית ועל מק״מ.",
        ),
        tax: t(
            "Not taxed, and not deducted from the gain at the end the way a purchase's commission is: the law counts custody as an expense of its own year.",
            "לא ממוסים, ולא מנוכים מהרווח בסוף כמו עמלת קנייה: החוק רואה בדמי משמרת הוצאה של השנה שבה שולמו.",
        ),
        where_met: t(
            "In the price lists of banks and investment houses, and in the offers for new customers.",
            "בתעריפונים של בנקים ובתי השקעות, ובהצעות ללקוחות חדשים.",
        ),
        example: t(
            "For illustration: a handling fee of \u{20aa}15 a month is \u{20aa}180 a year; custody of 0.1% on \u{20aa}100,000 is \u{20aa}100 a year.",
            "להמחשה: דמי טיפול של \u{20aa}15 בחודש הם \u{20aa}180 בשנה; דמי משמרת של 0.1% על \u{20aa}100,000 הם \u{20aa}100 בשנה.",
        ),
        compare_with: &[
            Term::ManagementFee,
            Term::TradeCommission,
            Term::BrokerageAccount,
        ],
        sources: vec![boi_custody_rule(), funds::direct_expenses_regulations()],
    }
}

fn trade_commission() -> Card {
    Card {
        name: t("Trade commission", "עמלת\u{a0}קנייה\u{a0}ומכירה"),
        english: "Trade commission (brokerage fee)",
        bank: t(
            "\u{201c}עמלת פעולה\u{201d} or \u{201c}עמלת קנייה ומכירה\u{201d}, in the price list, by kind of security.",
            "״עמלת פעולה״ או ״עמלת קנייה ומכירה״, בתעריפון, לפי סוג נייר הערך.",
        ),
        ads: t(
            "Joining offers that refund or waive the commissions for a period.",
            "מבצעי הצטרפות שמחזירים או מוותרים על העמלות לתקופה.",
        ),
        what: t(
            "What the bank or investment house takes each time you buy or sell, usually a percentage of the trade with a minimum amount.",
            "מה שהבנק או בית ההשקעות לוקחים בכל קנייה או מכירה, בדרך כלל אחוז מסכום העסקה עם מינימום.",
        ),
        good_for: t(
            "Nothing: it's a cost, and it's the one you control by how often you trade.",
            "לכלום: זו עלות, והיא העלות היחידה שאתם שולטים בה לפי כמה פעמים אתם קונים.",
        ),
        downside: t(
            "The minimum weighs most on small, frequent purchases: a monthly purchase of a small sum can cost more in commission than a yearly one. Selling everything at the end is another trade.",
            "המינימום מכביד ביותר על קניות קטנות ותכופות: קנייה חודשית של סכום קטן יכולה לעלות יותר עמלות מקנייה שנתית. מכירת הכול בסוף היא עוד פעולה.",
        ),
        cost: t(
            "By plan, security and exchange, in the table. Some brokers charge less for a standing order.",
            "לפי מסלול, נייר ערך ובורסה, בטבלה. יש שגובים פחות על הוראת קבע.",
        ),
        tax: t(
            "The commission paid on a purchase counts as part of its cost, so it lowers the gain that's taxed.",
            "עמלה ששילמתם בקנייה נחשבת חלק מעלות הקנייה, ולכן היא מקטינה את הרווח שממוסים.",
        ),
        where_met: t(
            "In the price lists of banks and investment houses, and in comparison sites.",
            "בתעריפונים של בנקים ובתי השקעות, ובאתרי השוואה.",
        ),
        example: t(
            "For illustration: at 0.07% with a minimum of \u{20aa}3, a purchase of \u{20aa}2,000 costs the \u{20aa}3, because 0.07% of it is \u{20aa}1.40.",
            "להמחשה: בעמלה של 0.07% ומינימום \u{20aa}3, קנייה של \u{20aa}2,000 עולה \u{20aa}3, כי 0.07% ממנה הם \u{20aa}1.40.",
        ),
        compare_with: &[
            Term::ManagementFee,
            Term::CustodyAndHandling,
            Term::ConversionMarkup,
        ],
        sources: vec![funds::direct_expenses_regulations(), ordinance()],
    }
}

fn conversion_markup() -> Card {
    Card {
        name: t("Conversion markup", "מרווח\u{a0}המרה"),
        english: "Conversion markup (currency spread)",
        bank: t(
            "\u{201c}שער המרה\u{201d} or \u{201c}מרווח\u{201d}, in the price list or the daily exchange rates.",
            "״שער המרה״ או ״מרווח״, בתעריפון או בשערי החליפין היומיים.",
        ),
        ads: t(
            "\u{201c}No conversion fee\u{201d}, when the cost is in the rate instead.",
            "״בלי עמלת המרה״, כשהעלות נמצאת בשער.",
        ),
        what: t(
            "When you buy a security abroad, the bank converts your shekels to dollars, and back when you sell. It does so at a rate a little worse than the market's, and the difference is the markup.",
            "כשקונים נייר ערך בחו״ל, הבנק ממיר את השקלים שלכם לדולרים, וחזרה כשמוכרים. הוא עושה את זה בשער קצת גרוע מזה של השוק, וההפרש הוא המרווח.",
        ),
        good_for: t(
            "Nothing: it only appears when you invest in another currency. On the Tel Aviv exchange, in shekels, there's none.",
            "לכלום: הוא מופיע רק כשמשקיעים במטבע אחר. בבורסה בתל אביב, בשקלים, אין כזה.",
        ),
        downside: t(
            "It's charged twice, on the way in and on the way out, and most price lists publish only a maximum, so the app reads what's actually charged from other pages and says so.",
            "הוא נגבה פעמיים, בכניסה וביציאה, ברוב התעריפונים מפורסם רק מקסימום, ולכן המחשבון קורא מה נגבה בפועל מעמודים אחרים ואומר את זה.",
        ),
        cost: t(
            "By broker, in the table, with how sure the app is of it.",
            "לפי בנק או בית השקעות, בטבלה, עם כמה המחשבון בטוח בו.",
        ),
        tax: t(
            "Part of the cost of buying, so it lowers the gain that's taxed.",
            "חלק מעלות הקנייה, ולכן הוא מקטין את הרווח שממוסים.",
        ),
        where_met: t(
            "In the price list's section on foreign securities, and in the banks' published exchange rates.",
            "בפרק של התעריפון על ניירות ערך בחו״ל, ובשערי החליפין שהבנקים מפרסמים.",
        ),
        example: t(
            "For illustration: a markup of 0.5% on converting \u{20aa}10,000 costs \u{20aa}50, and the same again when you convert back.",
            "להמחשה: מרווח של 0.5% על המרה של \u{20aa}10,000 עולה \u{20aa}50, ועוד פעם כשממירים חזרה.",
        ),
        compare_with: &[
            Term::TradeCommission,
            Term::Exchange,
            Term::BrokerageAccount,
        ],
        sources: vec![
            ibi_currency_faq(),
            altshuler_currency_page(),
            funds::direct_expenses_regulations(),
        ],
    }
}

fn compound_interest() -> Card {
    Card {
        name: t("Compound interest", "ריבית\u{a0}דריבית"),
        english: "Compound interest",
        bank: t(
            "\u{201c}Interest on interest\u{201d}, when the interest joins the deposit and earns interest too.",
            "״ריבית דריבית״, כשהריבית מצטרפת לפיקדון ומרוויחה גם היא ריבית.",
        ),
        ads: t(
            "The line that curves upward in every savings calculator.",
            "הקו שמתעקם כלפי מעלה בכל מחשבון חיסכון.",
        ),
        what: t(
            "Earning a return on what you've already earned: each year's gain joins the money, so the next year's return is on a larger sum. The line curves upward the longer it runs.",
            "להרוויח תשואה על מה שכבר הרווחתם: הרווח של כל שנה מצטרף לכסף, ולכן התשואה של השנה הבאה היא על סכום גדול יותר. הקו מתעקם כלפי מעלה ככל שהזמן עובר.",
        ),
        good_for: t(
            "Explaining why starting early matters more than the size of the deposits.",
            "להסביר למה להתחיל מוקדם חשוב יותר מגובה ההפקדות.",
        ),
        downside: t(
            "It works against you as well: a fee taken each year comes off a balance that would have earned interest on interest, so its damage grows with the years.",
            "זה עובד גם נגדכם: דמי ניהול שנלקחים בכל שנה יורדים מיתרה שהייתה מרוויחה ריבית דריבית, ולכן הנזק שלהם גדל עם השנים.",
        ),
        cost: t(
            "No cost of its own; it's how fees and returns add up.",
            "אין לו עלות משלו; כך מצטברים דמי ניהול ותשואה.",
        ),
        tax: t(
            "The tax comes at the end, on the whole gain, so the gain that compounds meanwhile isn't reduced by it each year.",
            "המס בא בסוף, על כל הרווח, ולכן הרווח שצובר ריבית דריבית לא מצטמצם בגללו בכל שנה.",
        ),
        where_met: t(
            "In every savings calculator and in the graphs of this app, where the line curves.",
            "בכל מחשבון חיסכון ובגרפים של המחשבון הזה, שבהם הקו מתעקם.",
        ),
        example: t(
            "For illustration: \u{20aa}10,000 at 5% a year for 20 years becomes about \u{20aa}26,500; interest of 5% on the original sum alone would only make \u{20aa}20,000.",
            "להמחשה: \u{20aa}10,000 ב\u{2011}5% בשנה במשך 20 שנה הופכים לכ\u{2011}\u{20aa}26,500; ריבית של 5% רק על הסכום המקורי הייתה נותנת \u{20aa}20,000.",
        ),
        compare_with: &[Term::ManagementFee, Term::Inflation, Term::YieldToMaturity],
        sources: vec![investor_gov_fees()],
    }
}

fn inflation() -> Card {
    Card {
        name: t(
            "Inflation and today's money",
            "אינפלציה\u{a0}וכסף\u{a0}של\u{a0}היום",
        ),
        english: "Inflation (today's money)",
        bank: t(
            "\u{201c}Linked to the index\u{201d}, \u{201c}real return\u{201d}: the return beyond the rise in prices.",
            "״צמוד מדד״, ״תשואה ריאלית״: התשואה שמעבר לעליית המחירים.",
        ),
        ads: t(
            "\u{201c}The cost of living\u{201d}, \u{201c}the money loses value\u{201d}.",
            "״יוקר המחיה״, ״הכסף מאבד מערכו״.",
        ),
        what: t(
            "Prices rising over time, so the same sum buys less. \u{201c}Today's money\u{201d} restates a future sum in what it would buy now, which lets you compare it with what you'd pay today.",
            "עליית מחירים לאורך זמן, כך שאותו סכום קונה פחות. ״כסף של היום״ מציג סכום עתידי לפי מה שהוא היה קונה עכשיו, וכך אפשר להשוות אותו למה שהייתם משלמים היום.",
        ),
        good_for: t(
            "Judging whether an end sum is really more than you put in, and whether a place beat inflation.",
            "לשפוט אם סכום בסוף הוא באמת יותר ממה שהפקדתם, ואם מקום מסוים ניצח את האינפלציה.",
        ),
        downside: t(
            "A sum that doesn't grow loses buying power every year. The Bank of Israel aims at 1\u{2013}3% a year, and the app takes 2% unless you choose another.",
            "סכום שלא גדל מאבד כוח קנייה בכל שנה. בנק ישראל שואף ל\u{2011}1\u{2013}3% בשנה, והמחשבון לוקח 2% אלא אם תבחרו אחרת.",
        ),
        cost: t(
            "It isn't a fee; it's what your money loses just by waiting.",
            "זו לא עמלה; זה מה שהכסף מפסיד רק מהמתנה.",
        ),
        tax: t(
            "Capital gains tax is on the real gain, the gain beyond inflation, so a gain that only keeps up with prices isn't taxed.",
            "מס רווחי הון הוא על הרווח הריאלי, הרווח שמעבר לאינפלציה, ולכן רווח שרק שומר על ערך הכסף לא ממוסה.",
        ),
        where_met: t(
            "In the news about the consumer price index, and in the names of linked deposits.",
            "בחדשות על מדד המחירים לצרכן, ובשמות של פיקדונות צמודים.",
        ),
        example: t(
            "For illustration: at 2% a year, what costs \u{20aa}100 today costs about \u{20aa}149 in 20 years, so \u{20aa}10,000 then buys what about \u{20aa}6,700 buys today.",
            "להמחשה: ב\u{2011}2% בשנה, מה שעולה היום \u{20aa}100 יעלה בעוד 20 שנה כ\u{2011}\u{20aa}149, ולכן \u{20aa}10,000 אז יקנו מה ש\u{2011}\u{20aa}6,700 בערך קונים היום.",
        ),
        compare_with: &[
            Term::CpiLinkedDeposit,
            Term::CapitalGainsTax,
            Term::BankOfIsraelRate,
        ],
        sources: vec![boi_inflation_target(), ordinance()],
    }
}

fn capital_gains_tax() -> Card {
    Card {
        name: t("Capital gains tax", "מס\u{a0}רווחי\u{a0}הון"),
        english: "Capital gains tax",
        bank: t(
            "\u{201c}מס רווח הון\u{201d}, withheld by the bank when you sell, and shown in the sale's confirmation.",
            "״מס רווח הון״, שהבנק מנכה כשמוכרים ומופיע באישור המכירה.",
        ),
        ads: t(
            "\u{201c}Tax-free\u{201d}, said of the funds where it's exempt.",
            "״פטור ממס״, על קופות שבהן הוא פטור.",
        ),
        what: t(
            "The tax on the gain when you sell a security: for an individual, 25% of the real gain, which is the gain less the rise in prices. A loss pays nothing.",
            "המס על הרווח כשמוכרים נייר ערך: ליחיד, 25% מהרווח הריאלי, שהוא הרווח פחות עליית המחירים. הפסד לא משלם דבר.",
        ),
        good_for: t(
            "Knowing what's left in your hand: the app ranks by what's left after the tax.",
            "לדעת מה נשאר ביד: המחשבון מדרג לפי מה שנשאר אחרי המס.",
        ),
        downside: t(
            "A very large gain in one year pays a surtax (מס יסף) on top. And on a holding abroad the law measures the real gain against the exchange rate, not against the index of prices.",
            "רווח גדול מאוד בשנה אחת משלם גם מס יסף. ובהחזקה בחו״ל החוק מודד את הרווח הריאלי מול שער החליפין ולא מול מדד המחירים.",
        ),
        cost: t(
            "It's the tax itself; fees paid earlier lower the gain it's taken from.",
            "זה המס עצמו; עמלות ששולמו קודם מקטינות את הרווח שהוא נלקח ממנו.",
        ),
        tax: t(
            "A bank deposit's interest pays 15% of all of it. A provident fund for investment, taken as a pension from 60, pays nothing on the gain.",
            "על הריבית בפיקדון בבנק משלמים 15% מכולה. קופת גמל להשקעה שמושכים ממנה קצבה מגיל 60 לא משלמת מס על הרווח.",
        ),
        where_met: t(
            "On the confirmation of every sale, and in the law, as section 91 of the Income Tax Ordinance.",
            "באישור על כל מכירה, ובחוק, בסעיף 91 לפקודת מס הכנסה.",
        ),
        example: t(
            "For illustration: you bought for \u{20aa}10,000 and sell for \u{20aa}15,000, with prices up 10%. The cost is raised to \u{20aa}11,000, the real gain is \u{20aa}4,000, and the tax is 25% of it, \u{20aa}1,000.",
            "להמחשה: קניתם ב\u{2011}\u{20aa}10,000 ומוכרים ב\u{2011}\u{20aa}15,000, כשהמחירים עלו 10%. העלות מתעדכנת ל\u{2011}\u{20aa}11,000, הרווח הריאלי הוא \u{20aa}4,000, והמס הוא 25% ממנו, \u{20aa}1,000.",
        ),
        compare_with: &[Term::Inflation, Term::Pension, Term::FixedDeposit],
        sources: vec![
            ordinance(),
            funds::meitav_on_tax(),
            boi_inflation_target(),
            shibolet_surtax(),
        ],
    }
}

fn yield_to_maturity() -> Card {
    Card {
        name: t("Yield to maturity", "תשואה\u{a0}לפדיון"),
        english: "Yield to maturity",
        bank: t(
            "\u{201c}תשואה לפדיון\u{201d}, in the bond's page, as a yearly percentage.",
            "״תשואה לפדיון״, בדף של האג״ח, כאחוז שנתי.",
        ),
        ads: t(
            "\u{201c}The yield of the bond\u{201d}.",
            "״התשואה של האג״ח״.",
        ),
        what: t(
            "The yearly return you get if you buy a bond at today's price and hold it until it's repaid: its interest on the way, and the difference between what you paid and the face value you get back.",
            "התשואה השנתית שתקבלו אם תקנו אג״ח במחיר של היום ותחזיקו אותו עד הפדיון: הריבית שלו בדרך, וההפרש בין מה ששילמתם לערך הנקוב שתקבלו בחזרה.",
        ),
        good_for: t(
            "Comparing bonds of different prices and dates with each other and with a deposit.",
            "להשוות בין אג״ח במחירים ובתאריכים שונים, ובינם לבין פיקדון.",
        ),
        downside: t(
            "It's a promise only if you hold to the end; selling earlier gets the day's price. The bank of Israel's yield curve shows it by the years left.",
            "זו הבטחה רק אם מחזיקים עד הסוף; מכירה מוקדמת מקבלת את מחיר היום. עקום התשואות של בנק ישראל מראה אותה לפי השנים שנותרו.",
        ),
        cost: t(
            "None: it's before the broker's commission and the tax.",
            "אין: היא לפני עמלת הקנייה ולפני המס.",
        ),
        tax: t(
            "Before tax: an unlinked bond pays 15% of the whole gain, a linked one 25% of the real gain.",
            "לפני מס: על אג״ח לא צמוד משלמים 15% מכל הרווח, ועל אג״ח צמוד 25% מהרווח הריאלי.",
        ),
        where_met: t(
            "In the bond's page on the exchange and on finance sites, and in the Bank of Israel's yield curve.",
            "בדף של האג״ח בבורסה ובאתרי פיננסים, ובעקום התשואות של בנק ישראל.",
        ),
        example: t(
            "For illustration: a bond that pays no interest, bought at \u{20aa}95 and repaid at \u{20aa}100 in a year, has a yield to maturity of about 5.3%.",
            "להמחשה: אג״ח בלי ריבית, שנקנה ב\u{2011}\u{20aa}95 ונפדה ב\u{2011}\u{20aa}100 בעוד שנה, תשואתו לפדיון כ\u{2011}5.3%.",
        ),
        compare_with: &[
            Term::GovernmentBond,
            Term::FixedDeposit,
            Term::CompoundInterest,
        ],
        sources: vec![
            boi_yield_curve(),
            themarker_bond_0347(),
            bizportal_tax_on_interest(),
        ],
    }
}

fn bank_of_israel_rate() -> Card {
    Card {
        name: t("The Bank of Israel's rate", "ריבית\u{a0}בנק\u{a0}ישראל"),
        english: "Bank of Israel interest rate",
        bank: t(
            "\u{201c}ריבית בנק ישראל\u{201d}, the basis for the bank's own rates.",
            "״ריבית בנק ישראל״, הבסיס לריביות של הבנק עצמו.",
        ),
        ads: t(
            "\u{201c}The Bank of Israel cut the rate\u{201d}, in the news, and \u{201c}the rate decision\u{201d}.",
            "״בנק ישראל הוריד את הריבית״, בחדשות, ו״החלטת הריבית״.",
        ),
        what: t(
            "The rate the Monetary Committee of the Bank of Israel sets, eight times a year. Banks and money market funds earn about it, so it sets what a deposit or a fund pays.",
            "הריבית שהוועדה המוניטרית של בנק ישראל קובעת, שמונה פעמים בשנה. בנקים וקרנות כספיות מרוויחים בערך אותה, ולכן היא קובעת מה פיקדון או קרן משלמים.",
        ),
        good_for: t(
            "Knowing what money kept short term should earn, and judging whether a bank's offer is good.",
            "לדעת מה כסף לטווח קצר אמור להרוויח, ולשפוט אם ההצעה של בנק טובה.",
        ),
        downside: t(
            "It changes. A deposit locks the rate it had on the day you open it; a money market fund follows the rate down.",
            "היא משתנה. פיקדון נועל את הריבית של יום פתיחתו; קרן כספית יורדת איתה.",
        ),
        cost: t(
            "It isn't a cost; the gap between it and what your bank pays you is.",
            "היא לא עלות; הפער בינה לבין מה שהבנק משלם לכם הוא העלות.",
        ),
        tax: t(
            "Not taxed itself; what a deposit or a fund earns from it is, as each one's card says.",
            "היא לא ממוסה בעצמה; מה שפיקדון או קרן מרוויחים ממנה כן, כמו שכתוב בכרטיס של כל אחד.",
        ),
        where_met: t(
            "In the news on the days of the decision, and in the app, as the rate a fund is taken to earn.",
            "בחדשות בימי ההחלטה, ובמחשבון, כריבית שקרן כספית מרוויחה לפיה.",
        ),
        example: t(
            "For illustration: when the rate falls from 4% to 3.5%, a money market fund's return falls by about half a point, and a deposit opened before keeps its rate.",
            "להמחשה: כשהריבית יורדת מ\u{2011}4% ל\u{2011}3.5%, התשואה של קרן כספית יורדת בכחצי אחוז, ופיקדון שנפתח קודם שומר על הריבית שלו.",
        ),
        compare_with: &[Term::MoneyMarketFund, Term::FixedDeposit, Term::Inflation],
        sources: vec![boi_monetary_policy(), boi_rate_schedule(), boi_deposits()],
    }
}

fn pension() -> Card {
    Card {
        name: t("Pension (monthly annuity)", "קצבה"),
        english: "Pension (monthly annuity)",
        bank: t(
            "\u{201c}קצבה\u{201d} or \u{201c}משיכה כקצבה\u{201d}, as against \u{201c}משיכה בבת אחת\u{201d}.",
            "״קצבה״ או ״משיכה כקצבה״, לעומת ״משיכה בבת אחת״.",
        ),
        ads: t(
            "\u{201c}A tax-free pension for life\u{201d}.",
            "״קצבה פטורה ממס לכל החיים״.",
        ),
        what: t(
            "Taking the savings out as a monthly payment instead of one sum. The fund doesn't pay it: the money moves to an insurer or a pension fund, which sets the monthly amount by your age.",
            "משיכת החיסכון כתשלום חודשי במקום סכום אחד. הקופה לא משלמת אותה: הכסף עובר לחברת ביטוח או לקרן פנסיה, שקובעות את הסכום החודשי לפי הגיל.",
        ),
        good_for: t(
            "Income for life, and in a provident fund for investment from 60, a pension whose gain is tax-free.",
            "הכנסה לכל החיים, ובקופת גמל להשקעה מגיל 60, קצבה שהרווח בה פטור ממס.",
        ),
        downside: t(
            "The monthly amount is set by the insurer's or pension fund's terms at retirement, by your age, so it can't be known exactly in advance.",
            "הסכום החודשי נקבע לפי התנאים של חברת הביטוח או קרן הפנסיה בפרישה, לפי הגיל, ולכן אי אפשר לדעת אותו במדויק מראש.",
        ),
        cost: t(
            "The insurer's or pension fund's terms: ask them at retirement.",
            "התנאים של חברת הביטוח או קרן הפנסיה: שאלו אותן בפרישה.",
        ),
        tax: t(
            "From 60, a pension from a provident fund for investment is exempt on the gain; a lump sum, or a withdrawal before 60, pays 25% of the real gain.",
            "מגיל 60, קצבה מקופת גמל להשקעה פטורה ממס על הרווח; משיכה בבת אחת, או לפני גיל 60, משלמת 25% מהרווח הריאלי.",
        ),
        where_met: t(
            "In pension articles and at the fund when you ask to withdraw.",
            "בכתבות על פנסיה ובקופה כשמבקשים למשוך.",
        ),
        example: t(
            "For illustration: a balance of \u{20aa}200,000 divided by about 200 is a pension of about \u{20aa}1,000 a month, which the app shows as a reading.",
            "להמחשה: יתרה של \u{20aa}200,000 מחולקת בכ\u{2011}200 היא קצבה של כ\u{2011}\u{20aa}1,000 בחודש, וכך המחשבון מציג אותה, כהערכה.",
        ),
        compare_with: &[
            Term::InvestmentGemel,
            Term::SavingsGemel,
            Term::CapitalGainsTax,
        ],
        sources: vec![kolzchut_pension_exemption(), kolzchut_investment_gemel()],
    }
}

fn deposit_ceiling() -> Card {
    Card {
        name: t("Deposit ceiling", "תקרת\u{a0}הפקדה"),
        english: "Deposit ceiling",
        bank: t(
            "\u{201c}תקרת הפקדה שנתית\u{201d}, in the fund's terms.",
            "״תקרת הפקדה שנתית״, בתנאי הקופה.",
        ),
        ads: t(
            "\u{201c}Deposit up to the ceiling\u{201d}.",
            "״הפקידו עד התקרה״.",
        ),
        what: t(
            "The most that may be deposited in a year in a provident fund for investment, or the part of a study fund's deposits whose gain is tax-free. It's set by law, and raised each January.",
            "הסכום המקסימלי שמותר להפקיד בשנה בקופת גמל להשקעה, או החלק מהפקדות קרן השתלמות שהרווח עליו פטור ממס. היא נקבעת בחוק, ועולה בכל ינואר.",
        ),
        good_for: t(
            "Knowing how much of your saving can go into the tax-favoured funds, and how much has to go elsewhere.",
            "לדעת כמה מהחיסכון אפשר לשים בקופות עם הטבת המס, וכמה צריך לשים במקום אחר.",
        ),
        downside: t(
            "The ceiling for a provident fund covers all your funds together; a saver who deposits more is told so in the table, by the year it's passed.",
            "התקרה של קופת גמל להשקעה חלה על כל הקופות שלכם יחד; מי שמפקיד יותר רואה בטבלה הודעה על כך, לפי השנה שבה עברו אותה.",
        ),
        cost: t(
            "It isn't a cost: it's a limit. The current amounts are in the table, from the data, not here.",
            "זו לא עלות: זו מגבלה. הסכומים העדכניים מופיעים בטבלה, מתוך הנתונים, ולא כאן.",
        ),
        tax: t(
            "In a study fund, the gain on deposits above the ceiling pays 25% of the real gain.",
            "בקרן השתלמות, הרווח על הפקדות שמעל התקרה משלם 25% מהרווח הריאלי.",
        ),
        where_met: t(
            "In the fund's terms and in articles at the start of each year, when the ceilings are updated.",
            "בתנאי הקופה ובכתבות בתחילת כל שנה, כשהתקרות מתעדכנות.",
        ),
        example: t(
            "For illustration: a saver who wants to deposit more than the ceiling in a year puts the rest in a brokerage account or a savings policy.",
            "להמחשה: מי שרוצה להפקיד בשנה יותר מהתקרה שם את היתר בחשבון מסחר או בפוליסת חיסכון.",
        ),
        compare_with: &[Term::InvestmentGemel, Term::StudyFund, Term::SavingsPolicy],
        sources: vec![kolzchut_investment_gemel(), kolzchut_study_fund()],
    }
}

fn cushion() -> Card {
    Card {
        name: t("Emergency cushion", "כרית\u{a0}ביטחון"),
        english: "Emergency fund (a safety cushion)",
        bank: t(
            "Not a product: money you keep in a current account or a money market fund.",
            "זה לא מוצר: כסף שמחזיקים בעו״ש או בקרן כספית.",
        ),
        ads: t(
            "\u{201c}קרן חירום\u{201d}, an emergency fund.",
            "״קרן חירום״.",
        ),
        what: t(
            "Money set aside for what can't be planned, such as a layoff or a bill, kept where it can be taken out the same week and doesn't fall in value.",
            "כסף שמפרישים למה שאי אפשר לתכנן, כמו פיטורים או חשבון לא צפוי, ושמור במקום שאפשר למשוך ממנו באותו שבוע ושלא יורד בערכו.",
        ),
        good_for: t(
            "Not having to sell investments in a fall to pay for an emergency.",
            "לא להיאלץ למכור השקעות בירידה כדי לשלם על מקרה חירום.",
        ),
        downside: t(
            "It earns little, because it must stay safe and reachable. The rule of thumb is three to six months of expenses, which is a rule of thumb, not a law.",
            "היא מרוויחה מעט, כי היא חייבת להישאר בטוחה ונגישה. כלל האצבע הוא שלושה עד שישה חודשי הוצאות, וזה כלל אצבע ולא חוק.",
        ),
        cost: t(
            "In a money market fund, its small fee: see its card.",
            "בקרן כספית, דמי הניהול הקטנים שלה: ראו את הכרטיס שלה.",
        ),
        tax: t(
            "As the place it's kept in: see the money market fund's and the deposit's cards.",
            "כמו המקום שבו היא שמורה: ראו את הכרטיסים של הקרן הכספית ושל הפיקדון.",
        ),
        where_met: t(
            "In articles on personal finance, and in the app, when you save every month with no goal.",
            "בכתבות על כספים אישיים, ובמחשבון, כשחוסכים כל חודש בלי יעד.",
        ),
        example: t(
            "For illustration: expenses of \u{20aa}8,000 a month and a cushion of three months is \u{20aa}24,000.",
            "להמחשה: הוצאות של \u{20aa}8,000 בחודש וכרית של שלושה חודשים הן \u{20aa}24,000.",
        ),
        compare_with: &[Term::MoneyMarketFund, Term::FixedDeposit, Term::Makam],
        sources: vec![emergency_fund_rule(), boi_money_fund()],
    }
}

fn brokerage_account() -> Card {
    Card {
        name: t("Trading account", "חשבון\u{a0}מסחר"),
        english: "Brokerage (securities) account",
        bank: t(
            "\u{201c}חשבון ניירות ערך\u{201d} or \u{201c}תיק השקעות\u{201d}, opened with the bank or an investment house.",
            "״חשבון ניירות ערך״ או ״תיק השקעות״, שפותחים בבנק או בבית השקעות.",
        ),
        ads: t(
            "\u{201c}An online trading account\u{201d}, the investment houses' name for it.",
            "״חשבון מסחר אונליין״, השם שבתי ההשקעות נותנים לו.",
        ),
        what: t(
            "The account in which securities are held and traded: funds, shares, bonds. It's separate from the current account, and it's where the commissions and the custody fee are charged.",
            "החשבון שבו מחזיקים וסוחרים בניירות ערך: קרנות, מניות, אג״ח. הוא נפרד מהעו״ש, ובו נגבות העמלות ודמי המשמרת.",
        ),
        good_for: t(
            "Buying and selling any security, in any amount, with no ceiling on deposits and no lock.",
            "קנייה ומכירה של כל נייר ערך, בכל סכום, בלי תקרת הפקדה ובלי נעילה.",
        ),
        downside: t(
            "No tax benefit: the gain pays 25% of the real gain when you sell. Each bank and investment house prices it differently, which is what the app compares.",
            "אין הטבת מס: הרווח משלם 25% מהרווח הריאלי כשמוכרים. כל בנק ובית השקעות מתמחרים אחרת, וזה מה שהמחשבון משווה.",
        ),
        cost: t(
            "Trade commissions, custody and handling fees and, abroad, a conversion markup: all by plan, in the table.",
            "עמלות קנייה ומכירה, דמי משמרת וטיפול, ובחו״ל מרווח המרה: הכול לפי מסלול, בטבלה.",
        ),
        tax: t(
            "25% of the real gain on a sale; the bank withholds it.",
            "25% מהרווח הריאלי במכירה; הבנק מנכה אותו.",
        ),
        where_met: t(
            "In the bank's app and at the investment houses, which compete on their price lists and joining offers.",
            "באפליקציה של הבנק ובבתי ההשקעות, שמתחרים בתעריפונים ובמבצעי ההצטרפות שלהם.",
        ),
        example: t(
            "For illustration: two accounts that buy the same fund can cost different amounts a year, because of the commission, the custody and the handling fee.",
            "להמחשה: שני חשבונות שקונים את אותה קרן יכולים לעלות סכומים שונים בשנה, בגלל העמלה, דמי המשמרת ודמי הטיפול.",
        ),
        compare_with: &[
            Term::InvestmentGemel,
            Term::CustodyAndHandling,
            Term::Exchange,
        ],
        sources: vec![
            tase_securities(),
            ordinance(),
            funds::direct_expenses_regulations(),
        ],
    }
}

fn exchange() -> Card {
    Card {
        name: t("Stock exchange", "בורסה"),
        english: "Stock exchange (Tel Aviv, US, Europe)",
        bank: t(
            "\u{201c}Tel Aviv\u{201d}, \u{201c}the US\u{201d}, \u{201c}Europe\u{201d}, in the price list by market.",
            "״תל אביב״, ״ארה״ב״, ״אירופה״, בתעריפון לפי שוק.",
        ),
        ads: t(
            "\u{201c}Wall Street\u{201d}, \u{201c}the American market\u{201d}.",
            "״וול סטריט״, ״השוק האמריקאי״.",
        ),
        what: t(
            "The market where securities are bought and sold. On the Tel Aviv exchange you pay in shekels. On the US or European exchanges the broker converts your shekels to dollars or euros and back.",
            "השוק שבו קונים ומוכרים ניירות ערך. בבורסה בתל אביב משלמים בשקלים. בבורסות בארה״ב ובאירופה הבנק או בית ההשקעות ממיר את השקלים לדולרים או ליורו וחזרה.",
        ),
        good_for: t(
            "Choosing where to hold the same index: Tel Aviv has no conversion; abroad the funds are often cheaper, but the conversion and the estate tax come with them.",
            "לבחור איפה להחזיק את אותו מדד S&P\u{a0}500: בתל אביב אין המרה; בחו״ל הקרנות לרוב זולות יותר, אבל המרה ומס עיזבון באים איתן.",
        ),
        downside: t(
            "Abroad you pay a conversion markup twice and take the exchange rate's risk; and a fund registered in the US may expose the heirs to American estate tax, above $60,000. A fund registered in Ireland or Israel doesn't.",
            "בחו״ל משלמים מרווח המרה פעמיים ולוקחים את הסיכון של שער החליפין; וקרן שרשומה בארה״ב עלולה לחשוף את היורשים למס עיזבון אמריקאי, מעל $60,000. קרן שרשומה באירלנד או בישראל לא חשופה.",
        ),
        cost: t(
            "The commission differs by exchange: see the table.",
            "העמלה שונה בין בורסות: ראו בטבלה.",
        ),
        tax: t(
            "In Israel 25% of the real gain whichever the exchange, but on a holding abroad the law measures the gain against the exchange rate.",
            "בישראל 25% מהרווח הריאלי בכל בורסה, אבל בהחזקה בחו״ל החוק מודד את הרווח מול שער החליפין.",
        ),
        where_met: t(
            "In every price list, in the news about the market's closing, and in the names of funds.",
            "בכל תעריפון, בחדשות על סגירת המסחר, ובשמות של קרנות.",
        ),
        example: t(
            "For illustration: the same index in a Tel Aviv ETF is bought in shekels; in a US ETF it's bought in dollars after a conversion.",
            "להמחשה: אותו מדד S&P\u{a0}500 בקרן סל בתל אביב נקנה בשקלים; בקרן סל אמריקאית הוא נקנה בדולרים אחרי המרה.",
        ),
        compare_with: &[Term::ConversionMarkup, Term::Etf, Term::BrokerageAccount],
        sources: vec![
            tase_securities(),
            tase_trading_days(),
            calcalist_etf_fees(),
            the_estate_rule(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_term_has_a_full_card_in_both_languages() {
        let filled = |card: &LearnCard| {
            [
                &card.name,
                &card.english,
                &card.as_the_bank_calls_it,
                &card.as_ads_call_it,
                &card.what,
                &card.good_for,
                &card.downside,
                &card.cost,
                &card.tax,
                &card.where_met,
                &card.example,
            ]
            .iter()
            .all(|text| !text.trim().is_empty())
        };
        for term in Term::iter() {
            for lang in [Lang::En, Lang::He] {
                let card = learn_card(term, lang);
                assert_eq!(card.term, term);
                assert!(filled(&card), "{term:?} {lang:?}");
                assert!(!card.sources.is_empty(), "{term:?} has no source");
                for source in &card.sources {
                    assert!(source.url.starts_with("http"), "{term:?}: {}", source.url);
                    assert_ne!(source.name, "");
                }
            }
        }
    }

    #[test]
    fn compare_with_points_at_other_terms_once() {
        for term in Term::iter() {
            let with = card(term).compare_with;
            assert!(!with.is_empty(), "{term:?} compares with nothing");
            assert!(!with.contains(&term), "{term:?} compares with itself");
            let distinct: HashSet<_> = with.iter().collect();
            assert_eq!(distinct.len(), with.len(), "{term:?} repeats a term");
        }
    }

    #[test]
    fn the_glossary_has_each_term_once_in_order() {
        let terms: Vec<Term> = glossary(Lang::He).iter().map(|card| card.term).collect();
        assert_eq!(terms, Term::iter().collect::<Vec<_>>());
        assert_eq!(terms.len(), Term::COUNT);
        assert_eq!(terms.iter().collect::<HashSet<_>>().len(), Term::COUNT);
    }

    #[test]
    fn hebrew_names_are_hebrew() {
        // A card whose Hebrew was left as its English would compile.
        for term in Term::iter() {
            let name = learn_card(term, Lang::He).name;
            assert!(
                name.chars().any(|c| ('\u{5d0}'..='\u{5ea}').contains(&c)),
                "{term:?}"
            );
        }
    }
}
