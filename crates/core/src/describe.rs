//! The tariffs in plain words, for showing to users: "0.3%, min $24, max
//! $6,750", "0.15% a quarter (0.6% a year)". Kept in the core so every UI
//! says the same thing, in the language it's shown in: every function that
//! makes words takes a [`Lang`], and every fixed text is written in both
//! languages, with [`Lang::pick`] or a [`Text`].

use std::fmt::Write as _;

use rust_decimal::Decimal;
use rusty_money::{Formatter as MoneyFormatter, Params};
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use crate::simulation::{NotOffered, Outcome, Scenario, USUAL_INFLATION};
use crate::{
    Basis, Broker, BrokerKind, Buying, Caveat, ConversionFee, CustodyFee, Errs, Exchange,
    ExchangeRates, HandlingFee, IntoEnumIterator, Lang, ManagementFee, Markup, Money, Named, Page,
    PercentFee, Period, Plan, Price, Security, TariffDate, Text, TradeFee, Vehicle, Withdrawal,
    funds, ils, short_term, tariffs,
};

/// A price in words, and whether it's nothing, so that every view dims the
/// same ones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct PriceText {
    /// "0.15%, min ₪3.5", "none", "not offered"
    pub text: String,
    /// True when nothing is charged, or no price is given.
    pub nothing: bool,
    /// For "not offered", the most general mismatch that's true, for its
    /// "?": "Nothing in Europe is offered", "No index funds are offered
    /// anywhere" (see [`Plan::not_offered_reason`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl PriceText {
    /// Why there's no price: "not offered".
    #[must_use]
    pub fn nothing(why: &str) -> Self {
        PriceText {
            text: why.to_owned(),
            nothing: true,
            reason: None,
        }
    }

    /// Nothing is charged.
    #[must_use]
    pub fn none(lang: Lang) -> Self {
        PriceText::nothing(lang.pick("none", "אין"))
    }

    /// No price is given, and `reason` says how general the gap is.
    #[must_use]
    pub fn not_offered(reason: String, lang: Lang) -> Self {
        PriceText {
            reason: Some(reason),
            ..PriceText::nothing(lang.pick("not offered", "לא מוצע"))
        }
    }
}

/// A fee shown as a price: its words in a language, and whether they say
/// it's nothing ("none").
pub trait Priced {
    fn is_nothing(&self) -> bool {
        false
    }

    fn text(&self, lang: Lang) -> String;

    fn price_text(&self, lang: Lang) -> PriceText {
        PriceText {
            text: self.text(lang),
            nothing: self.is_nothing(),
            reason: None,
        }
    }
}

/// A page a number rests on, as a link: the name in one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Source {
    pub name: String,
    pub url: String,
}

impl Source {
    #[must_use]
    pub fn new(page: &Page, lang: Lang) -> Self {
        Source {
            name: page.name[lang].to_owned(),
            url: page.url.clone(),
        }
    }
}

/// A choice explained for someone who doesn't know the term, with the Hebrew
/// names that Israeli brokers and sites use for it.
pub trait Explained: Named + Copy {
    fn explanation(self, lang: Lang) -> &'static str;
    fn hebrew_names(self) -> &'static [&'static str];

    /// The English name the page shows under the Hebrew one, where it helps:
    /// "ETF" is what many look up, while "USA" says nothing "ארה״ב" doesn't.
    fn english_name(self) -> Option<&'static str> {
        Some(self.name(Lang::En))
    }
}

impl Explained for Security {
    fn explanation(self, lang: Lang) -> &'static str {
        match self {
            Security::Etf => lang.pick(
                "A basket of many companies in one security, such as the 500 biggest in the \
                 US. You buy and sell it on the stock exchange like a share: at any moment of \
                 the trading day, at whatever price it's going for right then. Brokers charge \
                 it as they charge a share. Called תעודת\u{a0}סל until 2018.",
                "קרן שמחזיקה, בנייר ערך אחד, סל של חברות רבות, למשל 500 החברות הגדולות בארה״ב. קונים ומוכרים אותה בבורסה כמו מניה, בכל רגע של יום המסחר ובמחיר של אותו רגע, והעמלות עליה הן כמו על מניה. עד 2018 נקראה תעודת סל.",
            ),
            Security::IndexFund => lang.pick(
                "The same kind of basket, but not traded on the exchange. You buy it from the \
                 fund company through your broker, and sell it back the same way, at one price \
                 a day, set after the exchange closes; any amount will do, even ₪100. Brokers \
                 charge it as a fund, which at most of them is a different price from a \
                 share: on Altshuler's full tariff an ETF costs at least ₪3.5 a trade and an \
                 index fund at least ₪16, for the same S&P 500. Managed (active) and \
                 money-market funds aren't compared here.",
                "אותו סוג של סל, אבל היא לא נסחרת בבורסה: קונים אותה מחברת הקרן דרך הבנק או בית ההשקעות, ומוכרים לה אותה בחזרה באותה דרך, במחיר אחד ביום שנקבע אחרי סגירת המסחר. אפשר לקנות בכל סכום, אפילו ₪100. העמלות עליה הן של קרן ולא של מניה, וברוב המקומות הן שונות: בתעריפון המלא של אלטשולר, עסקה בקרן סל עולה לפחות ₪3.5 ועסקה בקרן מחקה לפחות ₪16, על אותו S&P 500. קרנות מנוהלות (אקטיביות) וקרנות כספיות לא מושוות כאן.",
            ),
            Security::Bond => lang.pick(
                "A loan you give to a government or a company: it pays you interest, say 4% a \
                 year, and you can sell it on the exchange before it's repaid. Short-term \
                 government bills (מק״מ) are priced separately and aren't compared.",
                "הלוואה שאתם נותנים לממשלה או לחברה: היא משלמת לכם ריבית, נניח 4% בשנה, ואפשר למכור את האג״ח בבורסה לפני שההלוואה נפרעת. מק״מ (מלווה קצר מועד של בנק ישראל) מתומחר בנפרד, ולא מושווה כאן.",
            ),
            Security::Stock => lang.pick(
                "A share in one company, such as Teva or Apple.",
                "חלק בחברה אחת, כמו טבע או אפל.",
            ),
        }
    }

    fn hebrew_names(self) -> &'static [&'static str] {
        match self {
            Security::Etf => &[
                "קרן סל",
                "קרן סל מחקה מדד",
                "קרן סל במסלול רציף",
                "תעודת סל",
            ],
            Security::IndexFund => &["קרן מחקה", "קרן נאמנות מחקה"],
            // Both spellings of the full name are common.
            Security::Bond => &["אג\"ח", "איגרת חוב", "אגרת חוב"],
            Security::Stock => &["מניה"],
        }
    }
}

impl Explained for Exchange {
    fn explanation(self, lang: Lang) -> &'static str {
        match self {
            Exchange::Tlv => lang.pick(
                "The Tel Aviv Stock Exchange. Prices are in shekels, so nothing is converted. \
                 An S&P 500 ETF is sold here too, as a קרן\u{a0}סל: an Israeli one, or a \
                 foreign one listed here (קרן\u{a0}זרה).",
                "הבורסה לניירות ערך בתל אביב. המחירים בשקלים, ולכן אין צורך בהמרה. גם קרנות סל על S&P 500 נסחרות כאן: ישראליות, וגם זרות שרשומות כאן למסחר.",
            ),
            Exchange::Usa => lang.pick(
                "NYSE or Nasdaq. Prices are in dollars, so your shekels are converted, which \
                 some brokers charge for. The same S&P 500 ETF bought here, such as VOO, needs \
                 shekels turned into dollars, and back when you sell.",
                "הבורסות NYSE ונאסד״ק. המחירים בדולרים, ולכן השקלים שלכם מומרים לדולרים בקנייה, ובחזרה לשקלים במכירה, ויש שגובים על כך עמלה. כאן נסחרות, למשל, קרנות סל על S&P 500 כמו VOO.",
            ),
            Exchange::Europe => lang.pick(
                "A European exchange, such as Xetra or Euronext. Prices are in euros, so your \
                 shekels are converted, which some brokers charge for. An Irish-based S&P 500 \
                 ETF listed in Amsterdam, for example. London, where Irish ETFs such as CSPX \
                 trade in dollars, isn't priced here yet.",
                "בורסה אירופית, כמו Xetra או Euronext. המחירים באירו, ולכן השקלים שלכם מומרים לאירו, ויש שגובים על כך עמלה. כאן נסחרת, למשל, קרן סל אירית על S&P 500 באמסטרדם. בורסת לונדון, שבה קרנות איריות כמו CSPX נסחרות בדולרים, עדיין לא נכללת במחשבון.",
            ),
        }
    }

    fn hebrew_names(self) -> &'static [&'static str] {
        match self {
            Exchange::Tlv => &["הבורסה לניירות ערך בתל אביב"],
            Exchange::Usa | Exchange::Europe => &[],
        }
    }

    // Places, which everyone knows by their Hebrew names.
    fn english_name(self) -> Option<&'static str> {
        None
    }
}

/// "month", as in "0.15% a month": the period's name on its own.
impl Named for Period {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Period::Month => lang.pick("month", "חודש"),
            Period::Quarter => lang.pick("quarter", "רבעון"),
            Period::Year => lang.pick("year", "שנה"),
        }
    }
}

impl Period {
    /// "a month", as in "0.15% a month".
    #[must_use]
    pub fn each(self, lang: Lang) -> &'static str {
        match self {
            Period::Month => lang.pick("a month", "לחודש"),
            Period::Quarter => lang.pick("a quarter", "לרבעון"),
            Period::Year => lang.pick("a year", "לשנה"),
        }
    }

    /// "monthly", as in "charged monthly".
    #[must_use]
    pub fn adverb(self, lang: Lang) -> &'static str {
        match self {
            Period::Month => lang.pick("monthly", "חודשית"),
            Period::Quarter => lang.pick("quarterly", "רבעונית"),
            Period::Year => lang.pick("yearly", "שנתית"),
        }
    }
}

/// "ETFs", "index funds": what isn't offered.
fn plural(security: Security, lang: Lang) -> &'static str {
    match security {
        Security::Etf => lang.pick("ETFs", "קרנות סל"),
        Security::IndexFund => lang.pick("index funds", "קרנות מחקות"),
        Security::Bond => lang.pick("bonds", "אג\u{5f4}ח"),
        Security::Stock => lang.pick("stocks", "מניות"),
    }
}

/// "in the USA", "in Tel Aviv": where something is bought.
fn place(exchange: Exchange, lang: Lang) -> &'static str {
    match exchange {
        Exchange::Tlv => lang.pick("in Tel Aviv", "בתל אביב"),
        Exchange::Usa => lang.pick("in the USA", "בארה\u{5f4}ב"),
        Exchange::Europe => lang.pick("in Europe", "באירופה"),
    }
}

/// "an ETF bought in the USA", "an index fund bought in Tel Aviv".
#[must_use]
pub fn purchase(security: Security, exchange: Exchange, lang: Lang) -> String {
    let security = match security {
        Security::Etf => lang.pick("an ETF", "קרן סל"),
        Security::IndexFund => lang.pick("an index fund", "קרן מחקה"),
        Security::Bond => lang.pick("a bond", "אג\u{5f4}ח"),
        Security::Stock => lang.pick("a stock", "מניה"),
    };
    let bought = lang.pick("bought", "שנקנית");
    format!("{security} {bought} {}", place(exchange, lang))
}

/// The kinds of fee a plan charges, to name and explain them. Its name is
/// [`Named::name`]: "Buy or sell", "Conversion markup".
/// In the order the editor shows them: `FeeKind::iter()`. A standing order,
/// the second conversion fee and the markup are shown under the fee they're
/// part of ([`FeeKind::label`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum FeeKind {
    Trade,
    Track,
    StandingOrder,
    /// Custody and the monthly handling fee, as one cost of keeping the
    /// account: a share of the holdings, a fixed amount a month, or both.
    Account,
    Conversion,
    SecondConversion,
    Markup,
    /// A fund's or a policy's yearly share of what has built up.
    Management,
    /// Its share of each deposit.
    DepositFee,
}

impl Named for FeeKind {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            FeeKind::Trade => lang.pick("Buy or sell", "קנייה או מכירה"),
            FeeKind::Track => lang.pick("Track", "שיטת חיוב"),
            FeeKind::StandingOrder => lang.pick("Standing order", "הוראת קבע"),
            FeeKind::Account => lang.pick("Keeping the account", "דמי משמרת וטיפול"),
            FeeKind::Conversion => lang.pick("Conversion", "המרת מט\u{5f4}ח"),
            FeeKind::SecondConversion => lang.pick("Second conversion fee", "עמלת המרה שנייה"),
            FeeKind::Markup => lang.pick("Conversion markup", "מרווח המרה"),
            FeeKind::Management => lang.pick("Management fee", "דמי ניהול מהצבירה"),
            FeeKind::DepositFee => lang.pick("Fee on deposits", "דמי ניהול מהפקדה"),
        }
    }
}

impl FeeKind {
    /// Its name under the fee it's part of: "markup", "by standing order".
    #[must_use]
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            FeeKind::Trade => lang.pick("buy or sell", "קנייה או מכירה"),
            FeeKind::Track => lang.pick("track picked for you", "שיטת החיוב שנבחרה עבורכם"),
            FeeKind::StandingOrder => lang.pick("by standing order", "בהוראת קבע"),
            FeeKind::Account => lang.pick("keeping the account", "משמרת וטיפול"),
            FeeKind::Conversion => lang.pick("conversion", "המרה"),
            FeeKind::SecondConversion => lang.pick("or, if less", "או, אם נמוך יותר"),
            FeeKind::Markup => lang.pick("markup", "מרווח"),
            FeeKind::Management => lang.pick("management fee", "דמי ניהול מהצבירה"),
            FeeKind::DepositFee => lang.pick("from deposits", "מההפקדות"),
        }
    }
}

impl Explained for FeeKind {
    fn explanation(self, lang: Lang) -> &'static str {
        match self {
            FeeKind::Trade => lang.pick(
                "Charged on every purchase and sale, including selling everything at the end. \
                 Usually a share of the trade with a minimum: at 0.07% with a ₪3 minimum, a \
                 ₪2,000 purchase is charged the ₪3, since 0.07% of it is only ₪1.40.",
                "נגבית על כל קנייה ומכירה, כולל מכירת הכול בסוף. בדרך כלל זה אחוז מסכום העסקה, עם מינימום: בעמלה של 0.07% ומינימום ₪3, על קנייה של ₪2,000 משלמים ₪3, כי 0.07% ממנה הם רק ₪1.40.",
            ),
            FeeKind::Track => lang.pick(
                "Some brokers let you choose, when you open the account, how trades abroad \
                 are priced: per share, per order or as a share of the trade. Altshuler, for \
                 example: 1¢ a share, $11 an order, or 0.15% of the trade. The app picks the \
                 cheapest for your inputs; ask for it when you open the account.",
                "יש בנקים ובתי השקעות שנותנים לבחור, בפתיחת החשבון, איך לחייב עסקאות בחו״ל: לפי מספר המניות, סכום קבוע לכל פקודה, או אחוז מהעסקה. באלטשולר, למשל: 1¢ למניה, $11 לפקודה, או 0.15% מהעסקה. המחשבון בוחר את השיטה הזולה ביותר עבורכם; בקשו אותה כשאתם פותחים את החשבון.",
            ),
            FeeKind::StandingOrder => lang.pick(
                "An instruction to buy the same amount every month, automatically. Some \
                 brokers charge less for these purchases: Leumi charges 0.225% instead of \
                 0.4% for an index fund bought this way. A one-time deposit and selling cost \
                 the usual fee.",
                "הוראה לקנות אוטומטית את אותו סכום בכל חודש. יש שגובים על קניות כאלה פחות: לאומי, למשל, גובה 0.225% במקום 0.4% על קרן מחקה שנקנית כך. על הפקדה חד-פעמית ועל מכירה משלמים את העמלה הרגילה.",
            ),
            FeeKind::Account => lang.pick(
                "What you pay for keeping the account, whether or not you trade. Some charge \
                 a share of what you hold, which grows with your savings: 0.6% a year on \
                 ₪100,000 is ₪600, every year. Others charge a fixed amount: ₪15 a month is \
                 ₪180 a year, often free for the first years, and some take that month's \
                 trade fees off it. The full price lists charge both.",
                "מה שמשלמים על עצם החזקת החשבון, גם בלי לקנות או למכור. יש שגובים דמי משמרת: אחוז משווי התיק, שגדל ככל שהחיסכון גדל (0.6% בשנה על תיק של ₪100,000 הם ₪600, בכל שנה). יש שגובים דמי טיפול: סכום קבוע, למשל ₪15 לחודש, שהם ₪180 בשנה. דמי הטיפול לרוב בחינם בשנים הראשונות, ויש שמקזזים מהם את עמלות המסחר של אותו חודש. בתעריפונים המלאים גובים את שניהם.",
            ),
            FeeKind::Conversion => lang.pick(
                "Charged for converting your shekels to the security's currency, and back \
                 when you sell.",
                "נגבית על המרת השקלים שלכם למטבע של נייר הערך, ועל ההמרה חזרה לשקלים במכירה.",
            ),
            FeeKind::SecondConversion => lang.pick(
                "A second price for converting that you also get, such as a customer \
                 group's next to the online one. Discounts don't add up, so each conversion \
                 costs whichever is less.",
                "מחיר נוסף להמרה, שגם לו אתם זכאים, למשל מחיר של קבוצת לקוחות לצד מחיר האונליין. הנחות לא מצטברות, ולכן כל המרה עולה לפי הזול מבין השניים.",
            ),
            FeeKind::Markup => lang.pick(
                "The Currency Conversion Markup: the broker converts at a rate worse than \
                 the market's by this much. It isn't listed as a fee, but it costs the same: \
                 at a market rate of ₪3.50 a dollar, a 0.7% markup means paying ₪3.52, which \
                 is ₪14 on ₪2,000.",
                "הבנק או בית ההשקעות ממיר לפי שער גרוע משער השוק, בשיעור הזה. זה לא מופיע כעמלה, אבל עולה כסף בדיוק כמו עמלה: בשער שוק של ₪3.50 לדולר, מרווח של 0.7% פירושו שמשלמים ₪3.52 לדולר, כלומר ₪14 על כל ₪2,000.",
            ),
            FeeKind::Management => lang.pick(
                "What the company running a fund or a policy takes for investing your money: \
                 a share of everything that has built up, every year. It comes out of the \
                 balance a little each month, so there's never a bill: 0.6% a year on \
                 ₪100,000 is ₪600 that year, and more as the savings grow.",
                "מה שהחברה שמנהלת את הקופה או הפוליסה גובה על השקעת הכסף: אחוז מכל מה שנצבר, בכל שנה. הסכום יורד מהחיסכון מעט בכל חודש, כך שלא מקבלים חשבון: 0.6% בשנה על ₪100,000 הם ₪600 באותה שנה, ויותר ככל שהחיסכון גדל.",
            ),
            FeeKind::DepositFee => lang.pick(
                "A share of every deposit, taken before it's invested: at 4%, ₪1,920 of a \
                 ₪2,000 deposit is invested. Few funds take one.",
                "אחוז מכל הפקדה, שנגבה לפני שהיא מושקעת: ב-4%, מושקעים ₪1,920 מתוך הפקדה של ₪2,000. מעט קופות גובות אותם.",
            ),
        }
    }

    fn hebrew_names(self) -> &'static [&'static str] {
        match self {
            FeeKind::Trade => &["עמלת קנייה/מכירה"],
            FeeKind::Track => &["מסלול עמלות"],
            FeeKind::StandingOrder => &["הוראת קבע"],
            // The share of the holdings, then the monthly fee as the tariffs
            // name it: IBI and Meitav, Altshuler; then both, as the law and
            // Altshuler's offer name them.
            FeeKind::Account => &[
                "דמי משמרת",
                "דמי טיפול",
                "דמי שימוש",
                "דמי ניהול תקופתיים",
                "דמי ניהול חשבון",
            ],
            FeeKind::Conversion => &["עמלת המרת מט\"ח"],
            FeeKind::SecondConversion => &[],
            FeeKind::Markup => &["מרווח המרה"],
            FeeKind::Management => &["דמי ניהול מהצבירה", "דמי ניהול מיתרה צבורה"],
            FeeKind::DepositFee => &["דמי ניהול מהפקדה", "דמי ניהול מהפקדות"],
        }
    }
}

/// A caveat's kind, as a plan's details label it: its [`Basis`] as the user
/// sees it, most serious first. Its label is [`Named::name`]: "Our reading".
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, strum::EnumIter,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum CaveatKind {
    MayCostMore,
    AtMost,
    Reading,
    NotCounted,
    Published,
}

impl From<&Basis> for CaveatKind {
    fn from(basis: &Basis) -> Self {
        match basis {
            Basis::Published => CaveatKind::Published,
            Basis::Reading { .. } => CaveatKind::Reading,
            Basis::Assumed { errs: Errs::AtMost } => CaveatKind::AtMost,
            Basis::Assumed {
                errs: Errs::MayCostMore { .. },
            } => CaveatKind::MayCostMore,
            Basis::NotCounted => CaveatKind::NotCounted,
        }
    }
}

impl Named for CaveatKind {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            CaveatKind::MayCostMore => {
                lang.pick("Assumed, may cost more", "הערכה, עשוי לעלות יותר")
            }
            CaveatKind::AtMost => lang.pick("Assumed, at most", "הערכה, לכל היותר"),
            CaveatKind::Reading => lang.pick("Our reading", "הפרשנות שלנו"),
            CaveatKind::NotCounted => lang.pick("Not counted", "לא נכלל"),
            CaveatKind::Published => lang.pick("As published", "כפי שפורסם"),
        }
    }
}

impl Explained for CaveatKind {
    fn explanation(self, lang: Lang) -> &'static str {
        match self {
            CaveatKind::MayCostMore => lang.pick(
                "Nothing is published to go on, so a stand-in was used that errs on the \
                 cheap side: the plan may cost more than shown. A bank that doesn't publish \
                 its conversion markup, for example, is counted as converting at the market \
                 rate. If you know the real number, change these fees with ✎.",
                "המחיר לא פורסם, ולכן המחשבון משתמש בהערכה זולה: המסלול עשוי לעלות יותר ממה שמוצג. למשל, בנק שלא מפרסם את מרווח ההמרה שלו נספר כאילו הוא ממיר לפי שער השוק. אם אתם יודעים את המספר האמיתי, שנו את העמלות עם ✎.",
            ),
            CaveatKind::AtMost => lang.pick(
                "Nothing is published to go on, so the tariff's full price or maximum is \
                 used: the plan can only be cheaper than shown. An offer that doesn't \
                 mention bonds, for example, gets the full tariff's bond price. Ask the \
                 broker what it really charges.",
                "המחיר לא פורסם, ולכן המחשבון משתמש במחיר המלא או במקסימום שבתעריפון: המסלול יכול רק לצאת זול יותר ממה שמוצג. למשל, מבצע הצטרפות שלא מזכיר אג״ח מקבל את מחיר האג״ח שבתעריפון המלא. שאלו את הבנק או בית ההשקעות כמה הוא גובה באמת.",
            ),
            CaveatKind::Reading => lang.pick(
                "The tariff is unclear or silent here. This is how it was read, and what \
                 supports the reading: a custody rate with no period stated was read as \
                 yearly, for example, because the exchange's data on what customers pay \
                 says so.",
                "התעריפון לא ברור כאן, או לא מתייחס למקרה. כך הבנו אותו, ועל סמך מה: למשל, שיעור דמי משמרת שלא נאמר לאיזו תקופה הוא הובן כשנתי, כי נתוני הבורסה על מה שהלקוחות משלמים בפועל מראים זאת.",
            ),
            CaveatKind::NotCounted => lang.pick(
                "A real cost the app leaves out, and why: too small or too rare to change a \
                 long-term comparison, or a one-off, such as a ₪200 gift for opening the \
                 account.",
                "עלות אמיתית שהמחשבון לא סופר, והסיבה: היא קטנה או נדירה מכדי לשנות השוואה לטווח ארוך, או חד-פעמית, כמו מתנה של ₪200 על פתיחת חשבון.",
            ),
            CaveatKind::Published => lang.pick(
                "What the tariff or the broker's site says. Nothing is uncertain; it's worth \
                 knowing before you choose.",
                "כך כתוב בתעריפון או באתר של הבנק או בית ההשקעות. אין כאן ספק, אבל כדאי לדעת לפני שבוחרים.",
            ),
        }
    }

    fn hebrew_names(self) -> &'static [&'static str] {
        &[]
    }
}

/// A caveat in words, for a plan's details.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct CaveatText {
    pub text: String,
    pub kind: CaveatKind,
    /// The kind's label, for one shown outside its group: "Our reading".
    pub label: String,
    /// For a reading, what supports it.
    pub support: Option<String>,
    /// The pages it rests on, to link beside it.
    pub sources: Vec<Source>,
    /// What it's about, for one shown among caveats about other choices:
    /// "Index fund on Tel Aviv", "Orders above $8,000".
    pub covers: String,
}

impl CaveatText {
    fn new(caveat: &Caveat, lang: Lang) -> Self {
        let kind = CaveatKind::from(&caveat.basis);
        CaveatText {
            text: caveat.text[lang].to_owned(),
            kind,
            label: kind.name(lang).to_owned(),
            support: match &caveat.basis {
                Basis::Reading { support } => Some(support[lang].to_owned()),
                _ => None,
            },
            sources: links(&caveat.sources, lang),
            covers: caveat.coverage(lang),
        }
    }
}

/// `pages` as links, in one language.
fn links(pages: &[Page], lang: Lang) -> Vec<Source> {
    pages.iter().map(|page| Source::new(page, lang)).collect()
}

/// The caveats of one kind, under its label.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct CaveatGroup {
    pub kind: CaveatKind,
    /// "Assumed, may cost more"
    pub label: String,
    /// What the label means, for its "?".
    pub explanation: String,
    pub caveats: Vec<CaveatText>,
}

/// `caveats` sorted for a plan's details: the ones that matter to `buying`,
/// grouped by kind, most serious first, and the rest, each saying what it's
/// about.
fn sort_caveats(
    caveats: &[&Caveat],
    buying: Buying,
    rates: &ExchangeRates,
    lang: Lang,
) -> (Vec<CaveatGroup>, Vec<CaveatText>) {
    let (matter, others): (Vec<&Caveat>, Vec<&Caveat>) = caveats
        .iter()
        .partition(|caveat| caveat.matters_for(buying, rates));
    let groups = caveat_groups(&matter, lang);
    let others = others
        .into_iter()
        .map(|caveat| CaveatText::new(caveat, lang))
        .collect();
    (groups, others)
}

/// `caveats` grouped by kind, most serious first.
#[must_use]
pub fn caveat_groups(caveats: &[&Caveat], lang: Lang) -> Vec<CaveatGroup> {
    CaveatKind::iter()
        .filter_map(|kind| {
            let texts: Vec<CaveatText> = caveats
                .iter()
                .filter(|caveat| CaveatKind::from(&caveat.basis) == kind)
                .map(|&caveat| CaveatText::new(caveat, lang))
                .collect();
            (!texts.is_empty()).then(|| CaveatGroup {
                kind,
                label: kind.name(lang).to_owned(),
                explanation: kind.explanation(lang).to_owned(),
                caveats: texts,
            })
        })
        .collect()
}

/// One fee, named and explained, and what a plan charges for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeeLine {
    pub kind: FeeKind,
    pub name: String,
    /// Its name when it's shown under another fee: "markup".
    pub label: String,
    pub explanation: String,
    pub hebrew_names: Vec<String>,
    /// "0.15%, min ₪3.5", or that it's nothing: "not offered", "none".
    pub price: PriceText,
    /// Shown under it, as part of it: a standing order's price under buying,
    /// the second fee and the markup under conversion.
    pub parts: Vec<FeeLine>,
    /// Beside the price, if a caveat is about this fee.
    pub mark: Option<Mark>,
}

/// A mark beside a fee's price: how sure the number is, in a word or two,
/// and the caveats that say why.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Mark {
    /// The most serious kind among the caveats.
    pub kind: CaveatKind,
    /// "may cost more"
    pub text: String,
    pub caveats: Vec<CaveatText>,
}

impl CaveatKind {
    /// The kind beside a price, in a word or two: "may cost more".
    #[must_use]
    pub fn mark_text(self, lang: Lang) -> &'static str {
        match self {
            CaveatKind::MayCostMore => lang.pick("may cost more", "עשוי לעלות יותר"),
            CaveatKind::AtMost => lang.pick("at most", "לכל היותר"),
            CaveatKind::Reading => lang.pick("our reading", "הפרשנות שלנו"),
            CaveatKind::NotCounted => lang.pick("not counted", "לא נכלל"),
            CaveatKind::Published => lang.pick("as published", "כפי שפורסם"),
        }
    }
}

impl FeeLine {
    fn new(kind: FeeKind, price: PriceText, lang: Lang) -> Self {
        FeeLine {
            kind,
            name: kind.name(lang).to_owned(),
            label: kind.label(lang).to_owned(),
            explanation: kind.explanation(lang).to_owned(),
            hebrew_names: kind
                .hebrew_names()
                .iter()
                .map(|&name| name.to_owned())
                .collect(),
            price,
            parts: vec![],
            mark: None,
        }
    }

    /// Puts each of `caveats` beside the fee it's about, in this line or
    /// its parts, marking the line with the most serious kind among them.
    fn attach(&mut self, caveats: &[&Caveat], lang: Lang) {
        let about_this: Vec<CaveatText> = caveats
            .iter()
            .filter(|caveat| caveat.fee == Some(self.kind))
            .map(|&caveat| CaveatText::new(caveat, lang))
            .collect();
        self.mark = about_this
            .iter()
            .map(|caveat| caveat.kind)
            .min()
            .map(|kind| Mark {
                kind,
                text: kind.mark_text(lang).to_owned(),
                caveats: about_this,
            });
        for part in &mut self.parts {
            part.attach(caveats, lang);
        }
    }

    /// A price that's never nothing: a track's name, a standing order's.
    fn part(kind: FeeKind, price: String, lang: Lang) -> Self {
        FeeLine::new(
            kind,
            PriceText {
                text: price,
                nothing: false,
                reason: None,
            },
            lang,
        )
    }
}

/// What a plan charges for one security on one exchange, in words, and the
/// caveats that apply to it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeesFor {
    /// Conversion only abroad: nothing is converted on Tel Aviv. Its parts
    /// include the markup, the other half of what converting costs.
    pub fees: Vec<FeeLine>,
    /// The caveats that matter to the purchase, grouped by kind, most
    /// serious first.
    pub caveats: Vec<CaveatGroup>,
    /// The rest: about other securities, exchanges or amounts, each saying
    /// which.
    pub others: Vec<CaveatText>,
}

impl Plan {
    /// Every page its caveats rest on, each once, in the order first named,
    /// as links.
    #[must_use]
    pub fn sources(&self, lang: Lang) -> Vec<Source> {
        links(
            &each_once(self.caveats.iter().flat_map(|caveat| &caveat.sources)),
            lang,
        )
    }

    /// Why `security` on `exchange` has no price, in the most general terms
    /// that are true: "Nothing in Europe is offered" when nothing there is,
    /// "No index funds are offered anywhere" when the security isn't, and
    /// otherwise "No index funds are offered in Europe".
    #[must_use]
    pub fn not_offered_reason(&self, security: Security, exchange: Exchange, lang: Lang) -> String {
        let rows = || {
            self.trading
                .iter()
                .chain(self.tracks.iter().flat_map(|track| &track.trading))
        };
        let anything_there =
            rows().any(|row| Security::iter().any(|each| row.applies_to(each, exchange)));
        let anywhere =
            rows().any(|row| Exchange::iter().any(|each| row.applies_to(security, each)));
        let (securities, place) = (plural(security, lang), place(exchange, lang));
        match (anything_there, anywhere, lang) {
            (false, _, Lang::En) => format!("Nothing {place} is offered"),
            (false, _, Lang::He) => format!("שום דבר לא מוצע {place}"),
            (true, false, Lang::En) => format!("No {securities} are offered anywhere"),
            (true, false, Lang::He) => format!("{securities} לא מוצעות בשום בורסה"),
            (true, true, Lang::En) => format!("No {securities} are offered {place}"),
            (true, true, Lang::He) => format!("{securities} לא מוצעות {place}"),
        }
    }

    /// Why the plan can't be used for `scenario`: that it doesn't sell the
    /// security there ([`Plan::not_offered_reason`]), how far a year's
    /// deposits are over its ceiling, or how long its money is locked.
    #[must_use]
    pub fn why_not_offered(&self, reason: NotOffered, scenario: &Scenario, lang: Lang) -> String {
        let year = match reason {
            NotOffered::NoPrice => {
                return self.not_offered_reason(scenario.security, scenario.exchange, lang);
            }
            NotOffered::Locked { years } => {
                return match lang {
                    Lang::En => format!(
                        "Its money can be taken out on these terms only {years} years after \
                         the first deposit. Sooner, it's taxed as income"
                    ),
                    Lang::He => format!(
                        "אפשר למשוך את הכסף בתנאים האלה רק {years} שנים אחרי ההפקדה הראשונה. לפני כן הוא ממוסה כהכנסה"
                    ),
                };
            }
            NotOffered::OverTheCeiling { year } => year,
        };
        let whole = |amount: Decimal| format_money(ils(amount.round()));
        let ceiling = self
            .vehicle
            .rules()
            .deposit_ceiling_in(year, scenario.inflation);
        let ceiling = whole(ceiling.unwrap_or_default());
        let deposits = scenario.deposits_by_year();
        let deposits = whole(deposits.get(year as usize).copied().unwrap_or_default());
        match (year, lang) {
            (0, Lang::En) => format!(
                "No more than {ceiling} can be deposited in a year, and your first year's \
                 deposits come to {deposits}"
            ),
            (0, Lang::He) => format!(
                "אי אפשר להפקיד יותר מ-{ceiling} בשנה, וההפקדות שלכם בשנה הראשונה מגיעות ל-{deposits}"
            ),
            (_, Lang::En) => format!(
                "No more than {ceiling} can be deposited in year {} (the ceiling, raised with \
                 prices), and your deposits come to {deposits}",
                year + 1
            ),
            (_, Lang::He) => format!(
                "אי אפשר להפקיד יותר מ-{ceiling} בשנה ה-{} (התקרה, מעודכנת לפי האינפלציה), וההפקדות שלכם מגיעות ל-{deposits}",
                year + 1
            ),
        }
    }

    /// What the plan charges for `buying`, in words, with the caveats that
    /// apply. A fund or a policy charges its manager's fee and nothing else;
    /// a broker's plan its trade, account and conversion fees.
    #[must_use]
    pub fn describe_fees_for(
        &self,
        buying: Buying,
        track: Option<usize>,
        broker_caveats: &[Caveat],
        rates: &ExchangeRates,
        lang: Lang,
    ) -> FeesFor {
        let mut fees = if self.vehicle.invests_for_you() {
            vec![]
        } else {
            self.tariff_lines(buying, track, lang)
        };
        if let Some(fee) = self.management {
            fees.extend(fee.lines(lang));
        }
        let caveats: Vec<&Caveat> = broker_caveats.iter().chain(&self.caveats).collect();
        let (groups, others) = sort_caveats(&caveats, buying, rates, lang);
        let mattering: Vec<&Caveat> = caveats
            .iter()
            .copied()
            .filter(|caveat| caveat.matters_for(buying, rates))
            .collect();
        for fee in &mut fees {
            fee.attach(&mattering, lang);
        }
        FeesFor {
            fees,
            caveats: groups,
            others,
        }
    }

    /// A broker's fees for `buying`: the trade, keeping the account and,
    /// abroad, the conversion.
    #[allow(
        clippy::too_many_lines,
        reason = "one fee after another, each in words"
    )]
    fn tariff_lines(&self, buying: Buying, track: Option<usize>, lang: Lang) -> Vec<FeeLine> {
        let Buying {
            security, exchange, ..
        } = buying;
        let row_for = |rows: &[TradeFee]| {
            rows.iter()
                .find(|row| row.applies_to(security, exchange))
                .map(|row| row.price.text(lang))
        };
        // The tracks that price this trade, with their prices.
        let tracks: Vec<(usize, String)> = self
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(index, each)| Some((index, row_for(&each.trading)?)))
            .collect();
        let picked = tracks.iter().find(|(index, _)| Some(*index) == track);
        let price = match (picked, tracks.is_empty()) {
            (Some((_, price)), _) => Some(price.clone()),
            (None, true) => row_for(&self.trading),
            (None, false) => Some(
                lang.pick("one of the tracks below", "אחת משיטות החיוב שלמטה")
                    .to_owned(),
            ),
        };
        let mut trade = FeeLine::new(
            FeeKind::Trade,
            price.clone().map_or_else(
                || PriceText::not_offered(self.not_offered_reason(security, exchange, lang), lang),
                |text| PriceText {
                    text,
                    nothing: false,
                    reason: None,
                },
            ),
            lang,
        );
        if !tracks.is_empty() {
            let others: Vec<&str> = tracks
                .iter()
                .filter(|(index, _)| Some(*index) != track)
                .map(|(_, price)| price.as_str())
                .collect();
            let mut part = FeeLine::part(FeeKind::Track, others.join("; "), lang);
            match picked {
                Some(&(index, _)) if others.is_empty() => {
                    self.tracks[index].name[lang].clone_into(&mut part.price.text);
                }
                Some(&(index, _)) => {
                    let name = &self.tracks[index].name[lang];
                    let others = &part.price.text;
                    part.price.text = match lang {
                        Lang::En => format!("{name} (others: {others})"),
                        Lang::He => format!("{name} (אחרות: {others})"),
                    };
                }
                None => lang
                    .pick("tracks", "שיטות חיוב")
                    .clone_into(&mut part.label),
            }
            trade.parts.push(part);
        }
        // A standing order at the usual price isn't worth a line.
        if let Some(by_standing_order) = row_for(&self.standing_orders)
            && Some(&by_standing_order) != price.as_ref()
        {
            trade.parts.push(FeeLine::part(
                FeeKind::StandingOrder,
                by_standing_order,
                lang,
            ));
        }
        let account = FeeLine::new(
            FeeKind::Account,
            account_price_text(
                self.custody_row(security, exchange),
                self.handling.as_ref(),
                lang,
            ),
            lang,
        );
        let mut fees = vec![trade, account];
        if exchange != Exchange::Tlv {
            let conversion = &self.conversion;
            let mut line = FeeLine::new(FeeKind::Conversion, conversion.price_text(lang), lang);
            if let Some(second) = conversion.or_if_less {
                line.parts.push(FeeLine::new(
                    FeeKind::SecondConversion,
                    second.price_text(lang),
                    lang,
                ));
            }
            if let Some(by_standing_order) = &self.standing_order_conversion {
                line.parts.push(FeeLine::new(
                    FeeKind::StandingOrder,
                    by_standing_order.price_text(lang),
                    lang,
                ));
            }
            let markup = conversion.markup.price_text(lang);
            line.parts.push(FeeLine::new(FeeKind::Markup, markup, lang));
            fees.push(line);
        }
        fees
    }

    /// Why the plan may cost more than shown for `buying`, if a caveat says
    /// so: "May cost more: conversion markup not published". Flagged in the
    /// comparison, since such a plan may rank better than it should.
    #[must_use]
    pub fn may_cost_more(
        &self,
        buying: Buying,
        broker_caveats: &[Caveat],
        rates: &ExchangeRates,
        lang: Lang,
    ) -> Option<String> {
        let mut summaries: Vec<&str> = broker_caveats
            .iter()
            .chain(&self.caveats)
            .filter(|caveat| caveat.matters_for(buying, rates))
            .filter_map(Caveat::may_cost_more_summary)
            .map(|summary| &summary[lang])
            .collect();
        summaries.dedup();
        (!summaries.is_empty()).then(|| {
            let summaries = summaries.join("; ");
            match lang {
                Lang::En => format!("May cost more: {summaries}"),
                Lang::He => format!("עשוי לעלות יותר: {summaries}"),
            }
        })
    }

    /// "US track: 1¢ a share, the cheapest for you", if `track` is one of the
    /// plan's tracks and prices `security` on `exchange`.
    #[must_use]
    pub fn track_note(
        &self,
        security: Security,
        exchange: Exchange,
        track: Option<usize>,
        lang: Lang,
    ) -> Option<String> {
        let track = self.track_for(track?, security, exchange)?;
        let name = &track.name[lang];
        Some(match (exchange, lang) {
            (Exchange::Tlv, Lang::En) => format!("Tel Aviv track: {name}, the cheapest for you"),
            (Exchange::Usa, Lang::En) => format!("US track: {name}, the cheapest for you"),
            (Exchange::Europe, Lang::En) => {
                format!("European track: {name}, the cheapest for you")
            }
            (exchange, Lang::He) => format!(
                "שיטת חיוב {}: {name}, הזולה ביותר עבורכם",
                place(exchange, Lang::He)
            ),
        })
    }

    /// "Needs a one-time deposit of at least ₪5,000", if `first_deposit` is less
    /// than the plan's minimum.
    #[must_use]
    pub fn first_deposit_warning(&self, first_deposit: Money, lang: Lang) -> Option<String> {
        self.min_first_deposit
            .filter(|min| first_deposit.amount() < min.amount())
            .map(|min| {
                let min = format_money(min);
                match lang {
                    Lang::En => format!("Needs a one-time deposit of at least {min}"),
                    Lang::He => format!("דורש הפקדה חד-פעמית של לפחות {min}"),
                }
            })
    }

    /// Why the plan's standing order price isn't used, if it has one for
    /// `security` on `exchange` and purchases are less often than monthly.
    #[must_use]
    pub fn standing_order_note(
        &self,
        security: Security,
        exchange: Exchange,
        buy_every_months: u32,
        lang: Lang,
    ) -> Option<&'static str> {
        (buy_every_months != 1 && self.standing_order_row(security, exchange).is_some()).then(
            || {
                lang.pick(
                    "A standing order buys every month, so it isn't used here",
                    "הוראת קבע קונה כל חודש, ולכן היא לא רלוונטית כאן",
                )
            },
        )
    }
}

impl Vehicle {
    /// How its gains are taxed, in a line to show under a fund's name: "No
    /// tax on gains after 6 years, on up to ₪20,566 deposited a year". The
    /// one thing a saver must know about it, so it's said outright and not
    /// only among the caveats. `None` for a broker's account, which the
    /// others are measured against.
    #[must_use]
    pub fn tax_rule(self, lang: Lang) -> Option<String> {
        let rules = self.rules();
        Some(match self {
            Vehicle::Brokerage => return None,
            Vehicle::SavingsPolicy => lang
                .pick("Taxed like a broker", "ממוסה כמו חשבון מסחר")
                .to_owned(),
            Vehicle::InvestmentGemel => {
                let age = rules.pension?.from_age;
                match lang {
                    Lang::En => {
                        format!("No tax as a pension from {age}; otherwise taxed like a broker")
                    }
                    Lang::He => format!("אין מס בקצבה מגיל {age}; אחרת ממוסה כמו חשבון מסחר"),
                }
            }
            Vehicle::StudyFund => {
                let years = rules.open_after_years?;
                let amount = format_money(ils(rules.tax_free_deposits?));
                match lang {
                    Lang::En => format!(
                        "No tax on gains after {years} years, on up to {amount} deposited a year"
                    ),
                    Lang::He => {
                        format!("אין מס על הרווחים אחרי {years} שנים, על עד {amount} שהופקדו בשנה")
                    }
                }
            }
        })
    }
}

impl Plan {
    /// What became of the tax for `scenario`, in a line under the plan in
    /// the table: "No tax: your deposits are within ₪20,566 a year". Beside
    /// its tax, so the number doesn't have to be worked out from the
    /// caveats. `None` for a broker's plan, and when nothing is sold: then
    /// nobody is taxed yet.
    #[must_use]
    pub fn tax_note(&self, scenario: &Scenario, lang: Lang) -> Option<String> {
        if !scenario.sell_at_end {
            return None;
        }
        let rules = self.vehicle.rules();
        Some(match self.vehicle {
            Vehicle::Brokerage => return None,
            Vehicle::SavingsPolicy => return self.vehicle.tax_rule(lang),
            Vehicle::StudyFund => {
                let amount = format_money(ils(rules.tax_free_deposits?));
                let shares = scenario.taxed_shares(self.vehicle)?;
                let within = shares.iter().all(Decimal::is_zero);
                match (within, lang) {
                    (true, Lang::En) => {
                        format!("No tax: your deposits are within {amount} a year")
                    }
                    (true, Lang::He) => format!("אין מס: ההפקדות שלכם בתוך {amount} בשנה"),
                    (false, Lang::En) => {
                        format!("Taxed only on what you deposit over {amount} a year")
                    }
                    (false, Lang::He) => format!("המס הוא רק על מה שמופקד מעל {amount} בשנה"),
                }
            }
            Vehicle::InvestmentGemel => {
                let age = rules.pension?.from_age;
                let then = scenario.age.saturating_add(scenario.years);
                match (scenario.withdrawal, then >= age, lang) {
                    (Withdrawal::Pension, true, Lang::En) => {
                        format!("No tax: taken as a pension from {age}")
                    }
                    (Withdrawal::Pension, true, Lang::He) => {
                        format!("אין מס: נמשכת כקצבה מגיל {age}")
                    }
                    (Withdrawal::Pension, false, Lang::En) => format!(
                        "Taxed like a broker: the pension opens at {age}, and you'd be {then}"
                    ),
                    (Withdrawal::Pension, false, Lang::He) => format!(
                        "ממוסה כמו חשבון מסחר: הקצבה נפתחת בגיל {age}, ואתם תהיו בני {then}"
                    ),
                    (Withdrawal::LumpSum, _, Lang::En) => {
                        format!("Taxed like a broker; no tax as a pension from {age}")
                    }
                    (Withdrawal::LumpSum, _, Lang::He) => {
                        format!("ממוסה כמו חשבון מסחר; אין מס בקצבה מגיל {age}")
                    }
                }
            }
        })
    }
}

impl Outcome {
    /// "Its fees are more than you deposit", when they are: then its numbers
    /// go below zero, as the fees become a debt.
    #[must_use]
    pub fn warning(&self, deposited: Decimal, lang: Lang) -> Option<&'static str> {
        (self.fees.total() > deposited).then(|| {
            lang.pick(
                "Its fees are more than you deposit",
                "העמלות שלו גבוהות מסך ההפקדות",
            )
        })
    }
}

impl Broker {
    /// What "usual" means beside the plan a new customer gets: a bank's
    /// online prices, an investment house's joining offer, or the one plan
    /// a broker has.
    #[must_use]
    pub fn usual_plan_text(&self, lang: Lang) -> String {
        let name = &self.name[lang];
        let rest = lang.pick(
            "The usual plans are compared at first; tick others to add them.",
            "בהתחלה מושווים המסלולים הרגילים; סמנו אחרים כדי להוסיף אותם.",
        );
        match (self.kind, self.plans.len(), lang) {
            (BrokerKind::Funds, _, Lang::En) => format!(
                "What savers pay on average: the fee is agreed person by person, so there's \
                 no price list to show. {rest}"
            ),
            (BrokerKind::Funds, _, Lang::He) => format!(
                "מה שחוסכים משלמים בממוצע: דמי הניהול נקבעים לכל חוסך בנפרד, ולכן אין תעריפון להציג. {rest}"
            ),
            (_, 1, Lang::En) => {
                format!("The one plan {name} offers: its published price list. {rest}")
            }
            (_, 1, Lang::He) => {
                format!("המסלול היחיד ש{name} מציעה: התעריפון שהיא מפרסמת. {rest}")
            }
            (BrokerKind::Bank, _, Lang::En) => format!(
                "The plan a new customer of {name} usually gets: its prices for trading online \
                 by yourself, rather than a customer group's or the branch's. {rest}"
            ),
            (BrokerKind::Bank, _, Lang::He) => format!(
                "המסלול שלקוח חדש של {name} מקבל בדרך כלל: מחירי המסחר העצמאי באונליין, ולא של קבוצת לקוחות או של הסניף. {rest}"
            ),
            (BrokerKind::InvestmentHouse, _, Lang::En) => format!(
                "The plan a new customer of {name} usually gets: its joining offer \
                 (מבצע\u{a0}הצטרפות), rather than the full tariff it publishes, which is the \
                 most it may charge. {rest}"
            ),
            (BrokerKind::InvestmentHouse, _, Lang::He) => format!(
                "המסלול שלקוח חדש של {name} מקבל בדרך כלל: מבצע ההצטרפות שלו, ולא התעריפון המלא שהוא מפרסם, שהוא המקסימום שמותר לו לגבות. {rest}"
            ),
        }
    }

    /// "Tariff of 29/06/2026", "Tariff of 01/2025", or that the tariff isn't
    /// dated.
    #[must_use]
    pub fn tariff_date_text(&self, lang: Lang) -> String {
        let date = match self.tariff_date {
            Some(TariffDate::Day(date)) => date.format(format_description!("[day]/[month]/[year]")),
            Some(TariffDate::Month(date)) => date.format(format_description!("[month]/[year]")),
            None => {
                return lang
                    .pick("Tariff date not stated", "תאריך התעריפון לא צוין")
                    .to_owned();
            }
        };
        let date = date.expect("a fixed format");
        match (self.kind, lang) {
            (BrokerKind::Funds, Lang::En) => format!("Fees paid, by the data of {date}"),
            (BrokerKind::Funds, Lang::He) => format!("דמי הניהול שנגבו, לפי נתוני {date}"),
            (_, Lang::En) => format!("Tariff of {date}"),
            (_, Lang::He) => format!("תעריפון מ-{date}"),
        }
    }

    /// When its numbers were last checked: "Checked 29/09/2026".
    #[must_use]
    pub fn checked_on_text(&self, lang: Lang) -> String {
        match self.kind {
            BrokerKind::Funds => checked_text(funds::checked(), lang),
            BrokerKind::Bank | BrokerKind::InvestmentHouse => Broker::checked_text(lang),
        }
    }
}

/// "Checked 29/09/2026"
fn checked_text(date: time::Date, lang: Lang) -> String {
    let date = date
        .format(format_description!("[day]/[month]/[year]"))
        .expect("a fixed format");
    match lang {
        Lang::En => format!("Checked {date}"),
        Lang::He => format!("נבדק ב-{date}"),
    }
}

impl TradeFee {
    /// What the row covers: "ETF on Tel Aviv", "Anything on USA, Europe".
    #[must_use]
    pub fn coverage(&self, lang: Lang) -> String {
        on(&self.securities, &self.exchanges, lang)
    }
}

/// "ETF on Tel Aviv", "Anything on USA, Europe": what a row or a caveat
/// covers, from its lists (empty means all).
fn on(securities: &[Security], exchanges: &[Exchange], lang: Lang) -> String {
    let securities = list_or(securities, lang.pick("Anything", "הכול"), lang);
    let exchanges = list_or(exchanges, lang.pick("any exchange", "כל בורסה"), lang);
    match lang {
        Lang::En => format!("{securities} on {exchanges}"),
        Lang::He => format!("{securities} ב{exchanges}"),
    }
}

impl Price {
    /// "0.3%, min $24, max $6,750", "$0.01 per share, min $9", "$4 per order".
    #[must_use]
    pub fn text(&self, lang: Lang) -> String {
        let per_share = lang.pick("per share", "למניה");
        match self {
            Price::Percent { percent, min, max } => {
                format!("{percent}{}", bounds(*min, *max, lang))
            }
            Price::PerShare {
                per_share: amount,
                min,
                max,
            } => format!(
                "{} {per_share}{}",
                format_money(*amount),
                bounds(*min, *max, lang)
            ),
            Price::Flat(amount) => format!(
                "{} {}",
                format_money(*amount),
                lang.pick("per order", "לפקודה")
            ),
            Price::PercentPlusPerShare {
                percent,
                per_share: amount,
                min,
                max,
            } => format!(
                "{percent} + {} {per_share}{}",
                format_money(*amount),
                bounds(*min, *max, lang)
            ),
        }
    }
}

impl Priced for Price {
    fn text(&self, lang: Lang) -> String {
        Price::text(self, lang)
    }
}

impl CustodyFee {
    /// What the row covers: "Tel Aviv", "Any exchange", "Index fund on Tel
    /// Aviv".
    #[must_use]
    pub fn coverage(&self, lang: Lang) -> String {
        if self.securities.is_empty() {
            list_or(&self.exchanges, lang.pick("Any exchange", "כל בורסה"), lang)
        } else {
            on(&self.securities, &self.exchanges, lang)
        }
    }
}

/// What keeping the account costs, in words: a share of the holdings, a
/// fixed amount a month, both ("…, plus …"), or "none".
fn account_price_text(
    custody: Option<&CustodyFee>,
    handling: Option<&HandlingFee>,
    lang: Lang,
) -> PriceText {
    let custody = custody.filter(|row| !row.is_nothing());
    match (custody, handling) {
        (None, None) => PriceText::none(lang),
        (Some(custody), None) => custody.price_text(lang),
        (None, Some(handling)) => handling.price_text(lang),
        (Some(custody), Some(handling)) => PriceText {
            text: format!(
                "{}, {} {}",
                custody.text(lang),
                lang.pick("plus", "ועוד"),
                handling.text(lang)
            ),
            nothing: false,
            reason: None,
        },
    }
}

impl ManagementFee {
    /// "0.62% of the balance a year", or "none".
    #[must_use]
    pub fn balance_price_text(&self, lang: Lang) -> PriceText {
        if self.of_balance.is_zero() {
            return PriceText::none(lang);
        }
        let text = match lang {
            Lang::En => format!("{} of the balance a year", self.of_balance),
            Lang::He => format!("{} מהצבירה בשנה", self.of_balance),
        };
        PriceText {
            text,
            nothing: false,
            reason: None,
        }
    }

    /// "4% of each deposit", or "none".
    #[must_use]
    pub fn deposit_price_text(&self, lang: Lang) -> PriceText {
        if self.of_deposits.is_zero() {
            return PriceText::none(lang);
        }
        let text = match lang {
            Lang::En => format!("{} of each deposit", self.of_deposits),
            Lang::He => format!("{} מכל הפקדה", self.of_deposits),
        };
        PriceText {
            text,
            nothing: false,
            reason: None,
        }
    }

    /// The fee on the balance and the fee on deposits, each a fee of its
    /// own: neither is part of the other.
    fn lines(&self, lang: Lang) -> [FeeLine; 2] {
        [
            FeeLine::new(FeeKind::Management, self.balance_price_text(lang), lang),
            FeeLine::new(FeeKind::DepositFee, self.deposit_price_text(lang), lang),
        ]
    }
}

/// "0.62% of the balance a year", "1.05% of the balance a year, plus 4% of
/// each deposit", or "none".
impl Priced for ManagementFee {
    fn is_nothing(&self) -> bool {
        self.of_balance.is_zero() && self.of_deposits.is_zero()
    }

    fn text(&self, lang: Lang) -> String {
        let (balance, deposits) = (self.balance_price_text(lang), self.deposit_price_text(lang));
        match (balance.nothing, deposits.nothing) {
            (_, true) => balance.text,
            (true, false) => deposits.text,
            (false, false) => format!(
                "{}, {} {}",
                balance.text,
                lang.pick("plus", "ועוד"),
                deposits.text
            ),
        }
    }
}

/// "free for the first 2 years, then ₪15 a month, less that month's trade
/// fees".
impl Priced for HandlingFee {
    fn text(&self, lang: Lang) -> String {
        let monthly = format!(
            "{} {}",
            format_money(self.per_month),
            Period::Month.each(lang)
        );
        let mut text = match (self.free_months, lang) {
            (0, _) => monthly,
            (12, Lang::En) => format!("free for the first year, then {monthly}"),
            (12, Lang::He) => format!("חינם בשנה הראשונה, ואז {monthly}"),
            (months, Lang::En) if months % 12 == 0 => {
                format!("free for the first {} years, then {monthly}", months / 12)
            }
            (months, Lang::He) if months % 12 == 0 => {
                format!("חינם ב-{} השנים הראשונות, ואז {monthly}", months / 12)
            }
            (months, Lang::En) => format!("free for the first {months} months, then {monthly}"),
            (months, Lang::He) => format!("חינם ב-{months} החודשים הראשונים, ואז {monthly}"),
        };
        if self.less_trade_fees {
            text.push_str(lang.pick(
                ", less that month's trade fees",
                ", בקיזוז עמלות המסחר של אותו חודש",
            ));
        }
        text
    }
}

/// "0.15% a quarter (0.6% a year)", "0.15% a year, charged monthly, min ₪75 a
/// month", or "none" if it never charges anything.
impl Priced for CustodyFee {
    fn is_nothing(&self) -> bool {
        self.is_free()
    }

    fn text(&self, lang: Lang) -> String {
        if self.is_free() {
            return lang.pick("none", "אין").to_owned();
        }
        let mut text = format!("{} {}", self.percent, self.per.each(lang));
        if self.per != Period::Year {
            write!(
                text,
                " ({} {})",
                self.percent_per_year(),
                Period::Year.each(lang)
            )
            .expect("a String");
        }
        if self.billed != self.per {
            write!(
                text,
                ", {} {}",
                lang.pick("charged", "נגבה"),
                self.billed.adverb(lang)
            )
            .expect("a String");
        }
        if let Some(min) = self.min {
            write!(
                text,
                ", {} {} {}",
                lang.pick("min", "מינימום"),
                format_money(min),
                self.billed.each(lang)
            )
            .expect("a String");
        }
        text
    }
}

/// "0.16%, min $5.76, max $2,400", or "none" if it never charges anything.
impl Priced for PercentFee {
    fn is_nothing(&self) -> bool {
        self.is_free()
    }

    fn text(&self, lang: Lang) -> String {
        if self.is_free() {
            return lang.pick("none", "אין").to_owned();
        }
        format!("{}{}", self.percent, bounds(self.min, self.max, lang))
    }
}

/// The listed fee alone, without the second fee or the markup.
impl Priced for ConversionFee {
    fn is_nothing(&self) -> bool {
        self.fee.is_free()
    }

    fn text(&self, lang: Lang) -> String {
        self.fee.text(lang)
    }
}

/// "up to 0.7%", "none", "not published"
impl Priced for Markup {
    /// No markup, or none that's published.
    fn is_nothing(&self) -> bool {
        match self {
            Markup::UpTo(percent) => percent.is_zero(),
            Markup::PerDollar(_) => false,
            Markup::MarketRate | Markup::NotPublished => true,
        }
    }

    fn text(&self, lang: Lang) -> String {
        match self {
            Markup::UpTo(percent) if percent.is_zero() => lang.pick("none", "אין").to_owned(),
            Markup::UpTo(percent) => match lang {
                Lang::En => format!("up to {percent}"),
                Lang::He => format!("עד {percent}"),
            },
            Markup::PerDollar(amount) => {
                format!(
                    "{} {}",
                    format_money(*amount),
                    lang.pick("per dollar", "לדולר")
                )
            }
            Markup::MarketRate => lang.pick("at the market rate", "לפי שער השוק").to_owned(),
            Markup::NotPublished => lang.pick("not published", "לא פורסם").to_owned(),
        }
    }
}

/// An amount as a tariff writes it: "$6,750", "$5.76", "₪3.5".
///
/// A function rather than a `Display` impl: `Money` and `Display` are both
/// defined in other crates, and Rust's orphan rule only allows implementing a
/// trait for a type if one of them is ours.
#[must_use]
pub fn format_money(money: Money) -> String {
    // Normalized, so whole amounts have no ".00".
    let money = Money::from_decimal(money.amount().normalize(), money.currency());
    let params = Params {
        symbol: Some(money.currency().symbol),
        ..Params::default()
    };
    MoneyFormatter::money(&money, params)
}

/// ", min $24, max $6,750": a price's bounds, if it has any.
fn bounds(min: Option<Money>, max: Option<Money>, lang: Lang) -> String {
    let mut text = String::new();
    if let Some(min) = min {
        write!(
            text,
            ", {} {}",
            lang.pick("min", "מינימום"),
            format_money(min)
        )
        .expect("a String");
    }
    if let Some(max) = max {
        write!(
            text,
            ", {} {}",
            lang.pick("max", "מקסימום"),
            format_money(max)
        )
        .expect("a String");
    }
    text
}

/// "except Index fund on Tel Aviv (its own row above)", for a row of your
/// plan that more specific rows take part of; `covers` is what each of them
/// covers.
#[must_use]
pub fn except(covers: &[String], lang: Lang) -> Option<String> {
    let (last, rest) = covers.split_last()?;
    if rest.is_empty() {
        return Some(match lang {
            Lang::En => format!("except {last} (its own row above)"),
            Lang::He => format!("מלבד {last} (שורה משלה למעלה)"),
        });
    }
    let rest = rest.join(", ");
    Some(match lang {
        Lang::En => format!("except {rest} and {last} (their own rows above)"),
        Lang::He => format!("מלבד {rest} ו{last} (שורות משלהן למעלה)"),
    })
}

impl Broker {
    /// What plan `plan` charges for `buying`, with the broker's caveats
    /// first.
    #[must_use]
    pub fn describe_fees_for(
        &self,
        plan: &Plan,
        buying: Buying,
        track: Option<usize>,
        rates: &ExchangeRates,
        lang: Lang,
    ) -> FeesFor {
        plan.describe_fees_for(buying, track, &self.caveats, rates, lang)
    }

    /// Its tariff document, then every page its caveats and its plans' rest
    /// on, each once, as links.
    #[must_use]
    pub fn sources(&self, lang: Lang) -> Vec<Source> {
        self.sources_of(self.plans.iter().flat_map(|plan| &plan.caveats), lang)
    }

    /// The pages behind `plan`'s numbers: the tariff document, then what the
    /// broker-wide caveats and the plan's own rest on, each once.
    #[must_use]
    pub fn sources_for(&self, plan: &Plan, lang: Lang) -> Vec<Source> {
        self.sources_of(plan.caveats.iter(), lang)
    }

    fn sources_of<'a>(
        &'a self,
        caveats: impl Iterator<Item = &'a Caveat>,
        lang: Lang,
    ) -> Vec<Source> {
        let name = match self.kind {
            BrokerKind::Funds => {
                Text::new("The Capital Market Authority's data", "נתוני רשות שוק ההון")
            }
            BrokerKind::Bank | BrokerKind::InvestmentHouse => {
                Text::new("Tariff (PDF)", "תעריפון (PDF)")
            }
        };
        let tariff = self.source_url.iter().map(|url| Page {
            name: name.clone(),
            url: url.clone(),
        });
        let mut pages = tariff.collect::<Vec<Page>>();
        let caveats = self.caveats.iter().chain(caveats);
        pages.extend(caveats.flat_map(|caveat| caveat.sources.iter().cloned()));
        links(&each_once(pages.iter()), lang)
    }

    /// The broker-wide caveats that matter to `buying`, grouped by kind.
    #[must_use]
    pub fn caveats_for(
        &self,
        buying: Buying,
        rates: &ExchangeRates,
        lang: Lang,
    ) -> Vec<CaveatGroup> {
        let caveats: Vec<&Caveat> = self.caveats.iter().collect();
        sort_caveats(&caveats, buying, rates, lang).0
    }

    /// Why `plan` may cost more than shown for `buying`, if a caveat of the
    /// broker's or the plan's says so (see [`Plan::may_cost_more`]).
    #[must_use]
    pub fn may_cost_more(
        &self,
        plan: &Plan,
        buying: Buying,
        rates: &ExchangeRates,
        lang: Lang,
    ) -> Option<String> {
        plan.may_cost_more(buying, &self.caveats, rates, lang)
    }

    /// "Checked 29/09/2026": when the numbers were last checked against the
    /// brokers' documents and sites.
    #[must_use]
    pub fn checked_text(lang: Lang) -> String {
        checked_text(tariffs::checked(), lang)
    }
}

impl Caveat {
    /// "Index fund on Tel Aviv", "Everything", "Stock, ETF on USA, orders
    /// above $8,000"
    #[must_use]
    pub fn coverage(&self, lang: Lang) -> String {
        match (
            self.securities.is_empty() && self.exchanges.is_empty(),
            self.above,
        ) {
            (true, None) => lang.pick("Everything", "הכול").to_owned(),
            (true, Some(above)) => {
                let above = format_money(above);
                match lang {
                    Lang::En => format!("Orders above {above}"),
                    Lang::He => format!("פקודות מעל {above}"),
                }
            }
            (false, above) => {
                let mut text = on(&self.securities, &self.exchanges, lang);
                if let Some(above) = above {
                    let above = format_money(above);
                    match lang {
                        Lang::En => write!(text, ", orders above {above}"),
                        Lang::He => write!(text, ", פקודות מעל {above}"),
                    }
                    .expect("a String");
                }
                text
            }
        }
    }
}

// ─────────────────────────── About the numbers ───────────────────────────

/// How the numbers are made, what isn't counted and where the numbers come
/// from, for the page that explains the app. Plain text, section by section.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct About {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Section {
    pub title: String,
    pub paragraphs: Vec<String>,
    /// A list after the paragraphs, if the section has one.
    pub items: Vec<Item>,
    /// Pages to link, in groups, if the section is the sources.
    pub sources: Vec<SourceGroup>,
}

/// The pages behind one broker's numbers, or one kind of source: "Bank
/// Leumi · Tariff of 29/06/2026".
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct SourceGroup {
    pub title: String,
    pub sources: Vec<Source>,
}

/// `pages`, each URL once, in the order first seen.
fn each_once<'a>(pages: impl Iterator<Item = &'a Page>) -> Vec<Page> {
    let mut seen: Vec<Page> = vec![];
    for page in pages {
        if !seen.iter().any(|known| known.url == page.url) {
            seen.push(page.clone());
        }
    }
    seen
}

/// One item of a section's list, linked if it has somewhere to go.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Item {
    pub text: String,
    pub url: Option<String>,
}

fn item(lang: Lang, en: &str, he: &str) -> Item {
    Item {
        text: lang.pick(en, he).to_owned(),
        url: None,
    }
}

/// The page that explains the numbers: what the app does with your inputs,
/// how sure it is of each price, what it leaves out and why, and its sources.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the page's text")]
pub fn about(lang: Lang) -> About {
    let paragraph = |en: &str, he: &str| lang.pick(en, he).to_owned();
    let method = Section {
        title: paragraph("How the numbers are made", "איך המספרים מחושבים"),
        paragraphs: vec![
            paragraph(
                "Each plan is run month by month on your deposits. Money arrives at the start \
                 of the month and waits as shekels until the next purchase, which converts it \
                 (abroad) and buys with it, whole shares only where the broker sells no \
                 fractions. What keeping the account costs, a share of the holdings or a \
                 fixed amount, is paid every month out of the shekels. At the end everything \
                 is sold and converted back and the tax on the gain is paid, and what's left \
                 is what the table ranks by; or, if you choose to keep holding, the table \
                 ranks by what's held, and nothing is paid for selling or in tax.",
                "המחשבון מחשב כל מסלול חודש אחר חודש, על ההפקדות שלכם. כל הפקדה מגיעה בתחילת החודש ומחכה בשקלים עד הקנייה הבאה. בקנייה הכסף מומר (אם קונים בחו״ל) ומושקע; אם הבנק או בית ההשקעות לא מוכר שברי מניה, נקנות רק מניות שלמות. דמי המשמרת והטיפול (אחוז מהתיק או סכום קבוע) משולמים כל חודש מהשקלים. בסוף הכול נמכר, מומר חזרה לשקלים, ומשולם מס על הרווח; הסכום שנשאר קובע את הדירוג בטבלה. אם בוחרים להמשיך להחזיק, הטבלה מדרגת לפי שווי התיק, בלי עמלת מכירה ובלי מס.",
            ),
            paragraph(
                "A provident fund for investment, a study fund or a savings policy is run the \
                 same way, with a manager in the broker's place: its fee comes off each deposit and off \
                 the balance every month, every deposit is invested as it arrives, and \
                 nothing is paid for trades or for converting. It's taken to earn what your \
                 security does, before fees.",
                "קופת גמל להשקעה, קרן השתלמות ופוליסת חיסכון מחושבות באותה דרך, עם חברה מנהלת במקום הבנק או בית ההשקעות: כל הפקדה מושקעת מיד כשהיא מגיעה, דמי הניהול יורדים מההפקדות ומהצבירה בכל חודש, ואין עמלות על קנייה, מכירה או המרה. המחשבון מניח שהקופה מרוויחה, לפני דמי ניהול, בדיוק מה שנייר הערך שבחרתם מרוויח.",
            ),
            match lang {
                Lang::En => format!(
                    "Tax is a quarter of the real gain: what selling brings, less what the \
                     holdings cost, the cost raised by how much prices rose since each \
                     purchase ({USUAL_INFLATION} a year, unless you set another inflation \
                     under \u{201c}More options\u{201d}). What was paid to buy counts as cost, \
                     fees included; of what was paid to keep the account, only the last \
                     year's comes off, the law allowing it only in the year of a sale. A provident \
                     fund for investment taken as a monthly pension from the age of 60 pays \
                     no tax on the gain, and a study fund pays none on the gains of what was \
                     deposited within its yearly tax-free amount. What's lost to fees is \
                     measured before tax, and the tax is each plan's own."
                ),
                Lang::He => format!(
                    "המס הוא מס רווחי הון: 25% מהרווח. את מה ששילמתם בכל קנייה מתרגמים קודם לערך הכסף ביום המכירה, לפי האינפלציה מאז אותה קנייה ({USUAL_INFLATION} בשנה, אלא אם קבעתם אינפלציה אחרת ב״אפשרויות נוספות״), כך שרווח שרק שומר על ערך הכסף לא ממוסה. עמלות הקנייה נחשבות חלק ממחיר הקנייה. מדמי המשמרת והטיפול מוכר רק מה ששולם בשנה האחרונה, כי החוק מתיר לקזז אותם רק בשנה שבה מוכרים. קופת גמל להשקעה שנמשכת כקצבה חודשית מגיל 60 פטורה ממס על הרווח, ובקרן השתלמות פטורים הרווחים על מה שהופקד עד התקרה השנתית. מה שאבד לעמלות נמדד לפני מס; המס מחושב לכל מסלול בנפרד."
                ),
            },
            paragraph(
                "The return is the security's own, in its own currency; today's exchange rates \
                 stay as they are, and money waiting for a purchase earns nothing. A plan with \
                 several price tracks costs what its cheapest does for your inputs, and the \
                 track is named.",
                "התשואה היא של נייר הערך עצמו, במטבע שלו. המחשבון מניח ששערי החליפין נשארים כמו היום, וכסף שמחכה לקנייה לא מרוויח דבר. במסלול עם כמה שיטות חיוב, המחשבון בוחר את הזולה ביותר עבורכם ומציין אותה בשם המסלול.",
            ),
            paragraph(
                "What's lost to fees is measured against the same deposits with no fees at all, \
                 bought every month. A plan's yearly cost states that loss the way a fund's \
                 management fee is stated: the yearly charge on your holdings that would cost \
                 you the same. Buying every three months rather than monthly leaves money \
                 waiting, and that counts too. The chart by deposit, under \u{201c}More \
                 options\u{201d}, runs the plans you compare on other deposits than yours, from \
                 ₪100 to ₪32,000 a month (or, with no monthly deposit, from ₪1,000 to \
                 ₪4,600,000 at once), to show where their ranking by fees flips: a plan with \
                 minimum fees is dear for small deposits and cheap for large ones.",
                "״אבד לעמלות״ הוא ההפרש מול אותן הפקדות בלי עמלות בכלל, בקנייה כל חודש. ״עמלות בשנה״ מציג את אותו הפסד כמו שמציגים דמי ניהול של קרן: האחוז מהתיק שהייתם צריכים לשלם כל שנה כדי להפסיד אותו סכום. קנייה כל שלושה חודשים במקום כל חודש משאירה כסף שמחכה בלי תשואה, וגם זה נספר. הגרף ״לפי הפקדה״, ב״אפשרויות נוספות״, מחשב את המסלולים שאתם משווים גם בהפקדות אחרות משלכם, מ-₪100 עד ₪32,000 בחודש (ובלי הפקדה חודשית, הפקדה חד-פעמית מ-₪1,000 עד ₪4,600,000), כדי להראות איפה מתחלף המסלול הזול בעמלות: מסלול עם עמלת מינימום יקר בהפקדות קטנות וזול בגדולות.",
            ),
            paragraph(
                "Under \u{201c}More options\u{201d}, deposits can grow each year as a salary \
                 does and be bought every few months rather than monthly, the inflation can be \
                 changed, and every amount can be shown in today's shekels: each is divided by how much prices will have risen by then. Showing \
                 them so changes no ranking, only how the numbers read.",
                "ב״אפשרויות נוספות״ אפשר לתת להפקדות לגדול כל שנה, כמו משכורת, לקנות פעם בכמה חודשים במקום כל חודש, לשנות את האינפלציה, ולהציג כל סכום בשקלים של היום, כלומר מתורגם לערך הכסף היום לפי האינפלציה. הצגה כזאת לא משנה אף דירוג, רק את אופן ההצגה של הסכומים.",
            ),
            paragraph(
                "Banks publish what they charge. Investment houses publish only a full tariff, \
                 the most they may charge, and offer new customers far less by phone. Their \
                 \u{201c}Typical offer\u{201d} plan is what comparison sites list for joining, \
                 and wherever the offer is silent the full tariff's price is used: a plan is \
                 never shown cheaper than its documents allow. A fund's fee is agreed person \
                 by person, so the app shows what savers pay on average, from the funds' \
                 reports to the Capital Market Authority.",
                "בנקים מפרסמים כמה הם גובים. בתי השקעות מפרסמים רק תעריפון מלא, כלומר המקסימום שמותר להם לגבות, ובפועל מציעים ללקוחות חדשים בטלפון הרבה פחות. המסלול ״מבצע הצטרפות״ שלהם הוא מה שאתרי ההשוואה מציגים למצטרפים, ובכל מה שהמבצע לא מזכיר נלקח מחיר התעריפון המלא: מסלול אף פעם לא מוצג זול יותר ממה שהמסמכים שלו מראים. דמי הניהול של קופה נקבעים לכל חוסך בנפרד, ולכן המחשבון מציג מה שחוסכים משלמים בממוצע, לפי מה שהקופות מדווחות לרשות שוק ההון.",
            ),
            paragraph(
                "Every plan's details say how sure each number is. As published: the tariff or \
                 the broker's site says so. Our reading: the tariff is unclear or silent, and \
                 this is how it was read, with what supports it. Assumed: a stand-in, either at \
                 most (the full price, so the plan can only be cheaper) or may cost more (the \
                 cheap side, which the comparison flags). Not counted: a real cost left out, and \
                 why.",
                "בפרטי כל מסלול מצוין עד כמה כל מספר בטוח. ״כפי שפורסם״: כך כתוב בתעריפון או באתר הבנק או בית ההשקעות. ״הפרשנות שלנו״: התעריפון לא ברור או לא מתייחס למקרה, וכך הבנו אותו, ועל סמך מה. ״הערכה״: המחיר לא פורסם, והמחשבון מעריך אותו: ״לכל היותר״ (המחיר המלא, כך שהמסלול יכול רק לצאת זול יותר) או ״עשוי לעלות יותר״ (הערכה זולה, שההשוואה מסמנת). ״לא נכלל״: עלות אמיתית שלא נספרה, והסיבה.",
            ),
        ],
        items: vec![],
        sources: vec![],
    };
    let left_out = Section {
        title: paragraph("What isn't counted", "מה לא נכלל"),
        paragraphs: vec![paragraph(
            "Costs every broker has that the app leaves out, and why. Each plan's own gaps \
             are in its details, under \u{201c}Not counted\u{201d}.",
            "עלויות שקיימות בכל הבנקים ובתי ההשקעות, והמחשבון לא סופר אותן, והסיבה. מה שחסר רק במסלול מסוים מופיע בפרטים שלו, תחת ״לא נכלל״.",
        )],
        sources: vec![],
        items: vec![
            item(
                lang,
                "Tax on dividends along the way: a quarter of each payment at a broker, for \
                 a security that pays them out, and less inside a fund or an ETF that keeps \
                 them. The return is taken as total return, so it isn't counted.",
                "מס על דיבידנדים במהלך השנים: 25% מכל דיבידנד בחשבון מסחר, כשנייר הערך מחלק דיבידנדים, ופחות בתוך קופה או בקרן סל שצוברת אותם. המחשבון מניח תשואה כוללת (כולל הדיבידנדים), ולכן המס הזה לא נספר.",
            ),
            item(
                lang,
                "Other rules of the tax on the gain: a shekel bond that isn't linked to \
                 prices pays 15% of its whole gain, not a quarter of the real one; a \
                 security in foreign currency is measured against the exchange rate, not \
                 against prices; and a gain that takes a year's income past ₪721,560 pays a \
                 surtax on the part above.",
                "כללי מס אחרים על הרווח: באג״ח שקלית לא צמודה המס הוא 15% מכל הרווח, ולא 25% מהרווח שמעבר לאינפלציה; בנייר ערך במטבע חוץ הרווח נמדד לפי שער החליפין, ולא לפי האינפלציה; ומי שהכנסתו השנתית, כולל הרווח, עוברת ₪721,560 משלם מס יסף על החלק שמעבר.",
            ),
            item(
                lang,
                "A provident fund for savings (קופת\u{a0}גמל\u{a0}לחיסכון): a pension \
                 product. It's worth what its tax benefits on deposits are worth to you, and \
                 the money comes out as a pension taxed by your income; the app knows \
                 neither. Large sums deposited near the age of 60 go by other rules \
                 (תיקון\u{a0}190), which aren't counted either.",
                "קופת גמל לחיסכון: מוצר פנסיוני. הכדאיות שלה תלויה בהטבות המס שתקבלו על ההפקדות, והכסף יוצא ממנה כקצבה שממוסה לפי ההכנסה שלכם; את שני אלה המחשבון לא יכול לדעת. לסכומים גדולים שמופקדים לקראת גיל 60 יש כללים אחרים (תיקון 190), שגם הם לא נספרים.",
            ),
            item(
                lang,
                "Dividends, and the fees some brokers take on them (Altshuler and Meitav \
                 0.3% of the payment): the return is taken as total return, dividends \
                 reinvested, and 0.3% of a dividend is a few thousandths of a percent a \
                 year.",
                "דיבידנדים, והעמלה שיש שגובים עליהם (באלטשולר ובמיטב 0.3% מהדיבידנד): המחשבון מניח תשואה כוללת, עם דיבידנדים שמושקעים מחדש, ו-0.3% מדיבידנד הם אלפיות אחוז בשנה.",
            ),
            item(
                lang,
                "Third-party fees on US trades (SEC, FINRA, exchange fees): fractions of a \
                 cent a share, passed on by every broker alike.",
                "עמלות של צד שלישי על עסקאות בארה״ב (SEC, FINRA, עמלות הבורסה): שברירי סנט למניה, שכל הבנקים ובתי ההשקעות מגלגלים על הלקוח באותה מידה.",
            ),
            item(
                lang,
                "Real-time quotes and advanced trading systems: optional extras.",
                "שערים בזמן אמת ומערכות מסחר מתקדמות: תוספות בתשלום, לא חובה.",
            ),
            item(
                lang,
                "Joining gifts and refunds (₪100–₪300, or a refund of early commissions): \
                 one-off, and named in each plan's details.",
                "מתנות הצטרפות והחזרים (₪100–₪300, או החזר של העמלות הראשונות): חד-פעמיים, ומצוינים בפרטי כל מסלול.",
            ),
            item(
                lang,
                "Interest on credit or on idle cash: the app keeps no debt, and money \
                 waiting for a purchase earns nothing.",
                "ריבית על אשראי או על מזומן בחשבון: המחשבון לא לוקח הלוואות, וכסף שמחכה לקנייה לא מרוויח דבר.",
            ),
            item(
                lang,
                "Moving securities to another broker (₪20–₪35 a security, $10–$40 abroad) \
                 and cancelled orders: once, if ever.",
                "העברת ניירות ערך לבנק או לבית השקעות אחר (₪20–₪35 לכל נייר ערך, $10–$40 בחו״ל) ופקודות שבוטלו: פעם אחת, אם בכלל.",
            ),
        ],
    };
    let mut groups: Vec<SourceGroup> = crate::listed()
        .iter()
        .map(|broker| SourceGroup {
            title: format!("{} · {}", &broker.name[lang], broker.tariff_date_text(lang)),
            sources: broker.sources(lang),
        })
        .collect();
    groups.push(SourceGroup {
        title: paragraph(
            "Comparison sites, for the typical offers and the investment houses' \
             conversion markups (September 2026)",
            "אתרי השוואה, למבצעי ההצטרפות ולמרווחי ההמרה של בתי ההשקעות (ספטמבר 2026)",
        ),
        sources: links(
            &[
                tariffs::gemeltop_comparison(),
                tariffs::tradingil_comparison(),
                tariffs::tradingil_conversions(),
                tariffs::broker_co_il(),
            ],
            lang,
        ),
    });
    groups.push(SourceGroup {
        title: paragraph(
            "The Tel Aviv Stock Exchange: its members' tariffs and actual average fees \
             (June 2026), which confirm the offers and settled unclear rows",
            "הבורסה לניירות ערך בתל אביב: תעריפוני החברים והעמלות הממוצעות בפועל (יוני 2026), שמאשרים את המבצעים והכריעו שורות לא ברורות",
        ),
        sources: links(&[tariffs::exchange_calculator()], lang),
    });
    groups.push(SourceGroup {
        title: paragraph(
            "The tax on the gain: the law, and a broker's account of it",
            "המס על הרווח: החוק, והסבר של בית השקעות",
        ),
        sources: links(
            &[funds::income_tax_ordinance(), funds::meitav_on_tax()],
            lang,
        ),
    });
    let sources = Section {
        title: paragraph("Sources", "מקורות"),
        paragraphs: vec![match lang {
            Lang::En => format!(
                "{} against each broker's tariff document and site. Tariffs change several \
                 times a year: each plan's details link to its document, and each caveat to \
                 the page it rests on.",
                Broker::checked_text(lang)
            ),
            Lang::He => format!(
                "{} מול התעריפון והאתר של כל בנק ובית השקעות. תעריפונים משתנים כמה פעמים בשנה: בפרטי כל מסלול יש קישור לתעריפון שלו, ובכל הסתייגות קישור לעמוד שעליו היא מבוססת.",
                Broker::checked_text(lang)
            ),
        }],
        items: vec![],
        sources: groups,
    };
    About {
        sections: vec![method, left_out, sources],
    }
}

/// "ETF, Stock", or `any` when the list is empty (a tariff row that doesn't
/// limit it).
fn list_or<T: Named + Copy>(items: &[T], any: &str, lang: Lang) -> String {
    if items.is_empty() {
        any.to_owned()
    } else {
        items
            .iter()
            .map(|&item| item.name(lang))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lang, Percent, ils, iso, tariffs, usd};
    use rust_decimal_macros::dec;

    #[test]
    fn amounts_as_tariffs_write_them() {
        assert_eq!(format_money(usd(dec!(6750.00))), "$6,750");
        assert_eq!(format_money(usd(dec!(5.76))), "$5.76");
        assert_eq!(format_money(ils(dec!(3.5))), "₪3.5");
        assert_eq!(
            format_money(Money::from_decimal(dec!(2.5), iso::EUR)),
            "€2.5"
        );
    }

    fn rates() -> ExchangeRates {
        ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
    }

    fn buying(security: Security, exchange: Exchange) -> Buying {
        Buying::any_amount(security, exchange)
    }

    fn pepper() -> (Broker, Plan) {
        let leumi = tariffs::leumi();
        let pepper = leumi
            .plans
            .iter()
            .find(|plan| plan.name.en == "Pepper")
            .unwrap()
            .clone();
        (leumi, pepper)
    }

    #[test]
    fn leumi_online_for_a_us_etf() {
        let leumi = tariffs::leumi();
        let fees = leumi.describe_fees_for(
            &leumi.plans[0],
            buying(Security::Etf, Exchange::Usa),
            None,
            &rates(),
            Lang::En,
        );
        let lines: Vec<_> = fees
            .fees
            .iter()
            .map(|fee| (fee.name.as_str(), fee.price.text.as_str()))
            .collect();
        assert_eq!(
            lines,
            [
                ("Buy or sell", "0.3%, min $24, max $6,750"),
                ("Keeping the account", "0.2% a quarter (0.8% a year)"),
                ("Conversion", "0.16%, min $5.76, max $2,400"),
            ]
        );
        let parts: Vec<_> = fees.fees[2]
            .parts
            .iter()
            .map(|part| (part.label.as_str(), part.price.text.as_str()))
            .collect();
        assert_eq!(parts, [("markup", "up to 0.9%")]);
        assert_eq!(fees.fees[1].hebrew_names[0], "דמי משמרת");

        // The markup, measured from the bank's published rates, is marked
        // beside its price as a reading, and its caveat heads the list, under
        // the label that says what it means.
        let mark = fees.fees[2].parts[0].mark.as_ref().unwrap();
        assert_eq!(mark.kind, CaveatKind::Reading);
        assert_eq!(mark.text, "our reading");
        assert_eq!(mark.caveats.len(), 1);
        assert!(mark.caveats[0].support.is_some());
        assert_eq!(fees.fees[0].mark, None);
        assert_eq!(fees.caveats[0].kind, CaveatKind::Reading);
        assert_eq!(fees.caveats[0].label, "Our reading");
        assert!(fees.caveats[0].explanation.contains("unclear or silent"));
        assert_eq!(fees.caveats[0].caveats[0].text, mark.caveats[0].text);
    }

    #[test]
    fn second_prices_are_parts_of_their_fee() {
        let leumi = tariffs::leumi();
        let plan = |name| {
            leumi
                .plans
                .iter()
                .find(|plan| plan.name.en == name)
                .unwrap()
        };
        let parts = |fee: &FeeLine| -> Vec<(String, String)> {
            let each = |part: &FeeLine| (part.label.clone(), part.price.text.clone());
            fee.parts.iter().map(each).collect()
        };
        let describe = |plan: &Plan, security, exchange| {
            plan.describe_fees_for(buying(security, exchange), None, &[], &rates(), Lang::En)
        };

        let standing_order = plan("Online, monthly standing order");
        let fund = describe(standing_order, Security::IndexFund, Exchange::Tlv);
        assert_eq!(fund.fees[0].price.text, "0.4%, min ₪26, max ₪6,300");
        assert_eq!(
            parts(&fund.fees[0]),
            [(
                "by standing order".into(),
                "0.225%, min ₪5, max ₪6,300".into()
            )]
        );
        let etf = describe(standing_order, Security::Etf, Exchange::Tlv);
        assert_eq!(etf.fees[0].parts, []);

        let plus18 = describe(plan("Online, 'Leumi 18+'"), Security::Etf, Exchange::Usa);
        assert_eq!(plus18.fees[2].price.text, "0.1%, min $7.2, max $3,000");
        assert_eq!(
            parts(&plus18.fees[2]),
            [
                ("or, if less".into(), "0.16%, min $5.76, max $2,400".into()),
                ("markup".into(), "up to 0.9%".into()),
            ]
        );
    }

    #[test]
    fn a_standing_order_needs_monthly_purchases() {
        let leumi = tariffs::leumi();
        let plan = &leumi.plans[2];
        let note =
            |security, every| plan.standing_order_note(security, Exchange::Tlv, every, Lang::En);
        assert_eq!(note(Security::IndexFund, 1), None);
        assert!(note(Security::IndexFund, 3).is_some());
        assert_eq!(note(Security::Etf, 3), None); // it has no standing order for ETFs
    }

    #[test]
    fn caveats_follow_what_is_bought_and_how_much() {
        let (leumi, pepper) = pepper();
        let caveats = |security, exchange, largest_trade| -> Vec<String> {
            let buying = Buying {
                security,
                exchange,
                largest_trade,
            };
            leumi
                .describe_fees_for(&pepper, buying, None, &rates(), Lang::En)
                .caveats
                .iter()
                .flat_map(|group| group.caveats.iter().map(|caveat| caveat.text.clone()))
                .collect()
        };

        // With the biggest order unknown, a caveat about large orders shows.
        let abroad = caveats(Security::Etf, Exchange::Usa, None);
        assert!(abroad.iter().any(|caveat| caveat.contains("markup")));
        assert!(abroad.iter().any(|caveat| caveat.starts_with("$4")));
        assert!(!abroad.iter().any(|caveat| caveat.starts_with("₪4")));
        // $4 is stated up to $8,000: €7,000 is more than that, $7,000 isn't.
        let small = caveats(Security::Etf, Exchange::Usa, Some(usd(dec!(7000))));
        assert!(!small.iter().any(|caveat| caveat.starts_with("$4")));
        let euros = Money::from_decimal(dec!(7000), iso::EUR);
        let big = caveats(Security::Etf, Exchange::Europe, Some(euros));
        assert!(big.iter().any(|caveat| caveat.starts_with("$4")));

        let tel_aviv_etf = caveats(Security::Etf, Exchange::Tlv, None);
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.contains("onversion"))
        );
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.to_lowercase().contains("index fund"))
        );

        let tel_aviv_fund = caveats(Security::IndexFund, Exchange::Tlv, None);
        assert!(
            tel_aviv_fund
                .iter()
                .any(|caveat| caveat.to_lowercase().contains("index fund"))
        );
    }

    #[test]
    fn the_rest_say_what_they_are_about() {
        let (leumi, pepper) = pepper();
        let buying = Buying {
            security: Security::Etf,
            exchange: Exchange::Usa,
            largest_trade: Some(usd(dec!(1000))),
        };
        let fees = leumi.describe_fees_for(&pepper, buying, None, &rates(), Lang::En);
        let covers: Vec<&str> = fees
            .others
            .iter()
            .map(|caveat| caveat.covers.as_str())
            .collect();
        assert!(covers.contains(&"Anything on Tel Aviv, orders above ₪30,000"));
        assert!(covers.contains(&"Anything on USA, Europe, orders above $8,000"));
        assert!(covers.contains(&"Index fund on Tel Aviv"));
        // What matters: the markup's reading, then Pepper's package and its
        // custody note.
        let kinds: Vec<CaveatKind> = fees.caveats.iter().map(|group| group.kind).collect();
        assert_eq!(
            kinds,
            [
                CaveatKind::Reading,
                CaveatKind::NotCounted,
                CaveatKind::Published
            ]
        );
        assert_eq!(
            fees.caveats[0].caveats.len(),
            1,
            "only the markup is a reading: {:?}",
            fees.caveats
        );
    }

    #[test]
    fn what_may_cost_more_is_flagged() {
        let (leumi, pepper) = pepper();
        let flag = |exchange, largest_trade| {
            let buying = Buying {
                security: Security::Etf,
                exchange,
                largest_trade,
            };
            leumi.may_cost_more(&pepper, buying, &rates(), Lang::En)
        };
        // The markup is a reading of Leumi's published rates, not a stand-in.
        assert_eq!(flag(Exchange::Usa, Some(usd(dec!(7000)))), None);
        assert_eq!(
            flag(Exchange::Usa, Some(usd(dec!(9000)))).as_deref(),
            Some("May cost more: $4 is stated only for orders up to $8,000")
        );
        assert_eq!(flag(Exchange::Tlv, Some(ils(dec!(1000)))), None);

        // A markup stated as a maximum can only overstate: nothing to flag.
        let altshuler = tariffs::altshuler();
        let offer = &altshuler.plans[1];
        let buying = buying(Security::Etf, Exchange::Usa);
        assert_eq!(
            altshuler.may_cost_more(offer, buying, &rates(), Lang::En),
            None
        );
    }

    #[test]
    fn interactive_converts_at_the_market_rate() {
        let interactive = tariffs::interactive();
        let plan = &interactive.plans[0];
        let buying = buying(Security::Etf, Exchange::Usa);
        let fees = interactive.describe_fees_for(plan, buying, None, &rates(), Lang::En);
        // Under conversion, after the standing order's part.
        let markup = fees.fees[2]
            .parts
            .iter()
            .find(|part| part.kind == FeeKind::Markup)
            .unwrap();
        assert_eq!(markup.price, PriceText::nothing("at the market rate"));
        assert_eq!(markup.mark.as_ref().unwrap().kind, CaveatKind::Published);
        assert_eq!(
            interactive.may_cost_more(plan, buying, &rates(), Lang::En),
            None
        );
        let kinds: Vec<CaveatKind> = fees.caveats.iter().map(|group| group.kind).collect();
        assert_eq!(
            kinds,
            [
                CaveatKind::Reading,
                CaveatKind::NotCounted,
                CaveatKind::Published
            ]
        );
    }

    #[test]
    fn altshuler_custody_and_markup() {
        let plan = &tariffs::altshuler().plans[0];
        // Nothing is converted on Tel Aviv, so no conversion lines.
        let fees = plan
            .describe_fees_for(
                buying(Security::Etf, Exchange::Tlv),
                None,
                &[],
                &rates(),
                Lang::En,
            )
            .fees;
        // Custody and the monthly fee are one line: what keeping the account
        // costs.
        assert_eq!(fees.len(), 2);
        assert_eq!(
            fees[1].price.text,
            "0.15% a year, charged monthly, min ₪75 a month, plus ₪80 a month"
        );
        assert_eq!(plan.conversion.text(Lang::En), "none");
        assert_eq!(plan.conversion.markup.text(Lang::En), "up to 0.7%");
        // The custody period was read, with support, and the monthly fee is a
        // maximum: the line carries both, marked by the more serious.
        let account = fees[1].mark.as_ref().unwrap();
        assert_eq!(account.kind, CaveatKind::AtMost);
        let reading = account
            .caveats
            .iter()
            .find(|caveat| caveat.kind == CaveatKind::Reading)
            .unwrap();
        assert!(reading.support.as_deref().unwrap().contains("June 2026"));
    }

    #[test]
    fn prices_of_nothing_say_so() {
        let plan = &tariffs::altshuler().plans[0];
        let fees = plan
            .describe_fees_for(
                buying(Security::Etf, Exchange::Usa),
                None,
                &[],
                &rates(),
                Lang::En,
            )
            .fees;
        let conversion = fees.last().unwrap();
        assert_eq!(
            conversion.price,
            PriceText::nothing("none"),
            "no conversion fee, like no custody"
        );
        assert!(
            !conversion.parts[0].price.nothing,
            "the markup is up to 0.7%"
        );

        // IBI's index funds on Tel Aviv have a custody row of 0%.
        let free = &tariffs::ibi().plans[0].custody[0];
        assert_eq!(free.price_text(Lang::En), PriceText::nothing("none"));
    }

    #[test]
    fn not_offered_says_the_most_general_mismatch() {
        let reason =
            |plan: &Plan, security, exchange| plan.not_offered_reason(security, exchange, Lang::En);
        let interactive = tariffs::interactive();
        let standard = &interactive.plans[0];
        assert_eq!(
            reason(standard, Security::Bond, Exchange::Tlv),
            "Nothing in Tel Aviv is offered"
        );
        assert_eq!(
            reason(standard, Security::IndexFund, Exchange::Usa),
            "No index funds are offered anywhere"
        );
        // Index funds in the USA only: Europe is the gap.
        let mut with_us_funds = standard.clone();
        with_us_funds.trading.push(TradeFee {
            securities: vec![Security::IndexFund],
            exchanges: vec![Exchange::Usa],
            price: Price::Flat(usd(dec!(1))),
        });
        assert_eq!(
            reason(&with_us_funds, Security::IndexFund, Exchange::Europe),
            "No index funds are offered in Europe"
        );
    }

    #[test]
    fn not_offered_coverage_and_dates() {
        let plan = &tariffs::altshuler().plans[0];
        let trade = &plan
            .describe_fees_for(
                buying(Security::Etf, Exchange::Europe),
                None,
                &[],
                &rates(),
                Lang::En,
            )
            .fees[0];
        assert!(trade.price.nothing);
        assert_eq!(trade.price.text, "not offered");
        assert_eq!(
            trade.price.reason.as_deref(),
            Some("Nothing in Europe is offered")
        );
        assert_eq!(plan.trading[0].coverage(Lang::En), "ETF on Tel Aviv");
        assert_eq!(
            purchase(Security::Etf, Exchange::Usa, Lang::En),
            "an ETF bought in the USA"
        );
        assert_eq!(
            purchase(Security::IndexFund, Exchange::Tlv, Lang::En),
            "an index fund bought in Tel Aviv"
        );
        assert_eq!(
            tariffs::leumi().tariff_date_text(Lang::En),
            "Tariff of 29/06/2026"
        );
        assert_eq!(
            tariffs::altshuler().tariff_date_text(Lang::En),
            "Tariff date not stated"
        );
        assert_eq!(Broker::checked_text(Lang::En), "Checked 29/09/2026");
    }

    #[test]
    fn what_usual_means_follows_the_kind_of_broker() {
        let text = |broker: Broker| broker.usual_plan_text(Lang::En);
        let leumi = text(tariffs::leumi());
        assert!(leumi.starts_with("The plan a new customer of Bank Leumi"));
        assert!(leumi.contains("online"), "{leumi}");
        let ibi = text(tariffs::ibi());
        assert!(ibi.contains("joining offer"), "{ibi}");
        assert!(ibi.contains("full tariff"), "{ibi}");
        // One plan: nothing to choose between.
        let interactive = text(tariffs::interactive());
        assert!(interactive.starts_with("The one plan Interactive Israel offers"));
        for broker in crate::listed() {
            assert!(
                broker
                    .usual_plan_text(Lang::En)
                    .ends_with("tick others to add them.")
            );
            assert!(
                broker
                    .usual_plan_text(Lang::He)
                    .ends_with("כדי להוסיף אותם.")
            );
        }
    }

    #[test]
    fn the_about_page_has_its_three_sections() {
        let about = about(Lang::En);
        let titles: Vec<&str> = about
            .sections
            .iter()
            .map(|section| section.title.as_str())
            .collect();
        assert_eq!(
            titles,
            ["How the numbers are made", "What isn't counted", "Sources"]
        );
        let sources = &about.sections[2];
        assert!(sources.paragraphs[0].starts_with("Checked 29/09/2026"));
        // A group per broker and per kind of fund, its tariff or the
        // regulator's data first, then the comparison sites, the exchange
        // and the law.
        assert_eq!(sources.sources.len(), crate::listed().len() + 3);
        for (group, broker) in sources.sources.iter().zip(crate::listed()) {
            assert!(group.title.starts_with(&*broker.name.en), "{}", group.title);
            let first = match broker.kind {
                BrokerKind::Funds => "The Capital Market Authority's data",
                BrokerKind::Bank | BrokerKind::InvestmentHouse => "Tariff (PDF)",
            };
            assert_eq!(group.sources[0].name, first);
            assert!(
                group.sources.len() > 1,
                "{}: only the tariff",
                broker.name.en
            );
        }
        assert_eq!(sources.items, Vec::new());
    }

    /// A fund charges its manager's fee and nothing else, whatever is
    /// bought: no trade, account or conversion lines.
    #[test]
    fn a_fund_charges_only_its_managers_fee() {
        let fund = funds::investment_gemel();
        let lines = |plan: usize, lang| {
            let buying = buying(Security::Etf, Exchange::Usa);
            fund.describe_fees_for(&fund.plans[plan], buying, None, &rates(), lang)
                .fees
        };
        let average = lines(0, Lang::En);
        // The fee on deposits on a line of its own, its name beside the
        // other's, not under it as a part.
        let kinds: Vec<_> = average.iter().map(|line| line.kind).collect();
        assert_eq!(kinds, [FeeKind::Management, FeeKind::DepositFee]);
        assert!(average.iter().all(|line| line.parts.is_empty()));
        assert_eq!(average[0].price.text, "0.62% of the balance a year");
        assert_eq!(average[1].price, PriceText::nothing("none"));
        // Each says where its number comes from.
        let mark = average[0].mark.as_ref().unwrap();
        assert_eq!(mark.kind, CaveatKind::Published);
        assert!(average[1].mark.is_some());
        assert_eq!(lines(0, Lang::He)[0].price.text, "0.62% מהצבירה בשנה");

        let most = lines(3, Lang::En);
        assert_eq!(most[0].price.text, "1.05% of the balance a year");
        assert_eq!(most[1].price.text, "4% of each deposit");
        assert_eq!(lines(3, Lang::He)[1].price.text, "4% מכל הפקדה");
        let fee = fund.plans[3].management.unwrap();
        assert_eq!(
            fee.text(Lang::En),
            "1.05% of the balance a year, plus 4% of each deposit"
        );
        let only_deposits = ManagementFee {
            of_balance: crate::Percent(dec!(0)),
            ..fee
        };
        assert_eq!(only_deposits.text(Lang::En), "4% of each deposit");
        assert!(!only_deposits.is_nothing());
        let nothing = ManagementFee {
            of_deposits: crate::Percent(dec!(0)),
            ..only_deposits
        };
        assert!(nothing.is_nothing());
        assert_eq!(nothing.price_text(Lang::En), PriceText::nothing("none"));
    }

    /// A plan the deposits are too much for says the ceiling and what the
    /// deposits come to, in the year they pass it.
    #[test]
    fn a_plan_over_its_ceiling_says_by_how_much() {
        use crate::Withdrawal;
        let fund = funds::investment_gemel().plans.remove(0);
        let at_once = Scenario {
            security: Security::Etf,
            exchange: Exchange::Europe,
            first_deposit: dec!(100000),
            monthly_deposit: dec!(0),
            deposit_growth: crate::Percent(dec!(0)),
            yearly_return: crate::Percent(dec!(5)),
            years: 3,
            buy_every_months: 1,
            share_price: dec!(100),
            sell_at_end: true,
            inflation: crate::Percent(dec!(0)),
            age: 30,
            withdrawal: Withdrawal::LumpSum,
        };
        let over = |year| NotOffered::OverTheCeiling { year };
        assert_eq!(
            fund.why_not_offered(over(0), &at_once, Lang::En),
            "No more than ₪83,641 can be deposited in a year, and your first year's deposits \
             come to ₪100,000"
        );
        assert_eq!(
            fund.why_not_offered(over(0), &at_once, Lang::He),
            "אי אפשר להפקיד יותר מ-₪83,641 בשנה, וההפקדות שלכם בשנה הראשונה מגיעות ל-₪100,000"
        );
        // ₪6,900 a month is ₪82,800 a year; 3% more the next year is ₪85,284,
        // and at 2% inflation the ceiling is 83,641 × 1.02 = ₪85,313.82 by
        // then, which rounds to ₪85,314.
        let growing = Scenario {
            first_deposit: dec!(0),
            monthly_deposit: dec!(6900),
            deposit_growth: crate::Percent(dec!(3)),
            inflation: crate::Percent(dec!(2)),
            ..at_once.clone()
        };
        assert_eq!(
            fund.why_not_offered(over(1), &growing, Lang::En),
            "No more than ₪85,314 can be deposited in year 2 (the ceiling, raised with prices), \
             and your deposits come to ₪85,284"
        );
        assert!(
            fund.why_not_offered(over(1), &growing, Lang::He)
                .contains("בשנה ה-2")
        );
        // A study fund kept for less than its six years.
        let study = funds::study_fund().plans.remove(0);
        let locked = NotOffered::Locked { years: 6 };
        assert_eq!(
            study.why_not_offered(locked, &at_once, Lang::En),
            "Its money can be taken out on these terms only 6 years after the first deposit. \
             Sooner, it's taxed as income"
        );
        assert!(
            study
                .why_not_offered(locked, &at_once, Lang::He)
                .starts_with("אפשר למשוך את הכסף בתנאים האלה רק 6 שנים")
        );
        // A broker that doesn't sell the security says so, as before.
        let altshuler = tariffs::altshuler().plans.remove(0);
        assert_eq!(
            altshuler.why_not_offered(NotOffered::NoPrice, &at_once, Lang::En),
            altshuler.not_offered_reason(Security::Etf, Exchange::Europe, Lang::En)
        );
    }

    /// Each fund's tax rule is one line, said under its name; a broker's
    /// account has none, being what the others are measured against.
    #[test]
    fn a_funds_tax_rule_is_said_in_a_line() {
        let rule = |vehicle: Vehicle, lang| vehicle.tax_rule(lang);
        assert_eq!(rule(Vehicle::Brokerage, Lang::En), None);
        assert_eq!(
            rule(Vehicle::StudyFund, Lang::En).as_deref(),
            Some("No tax on gains after 6 years, on up to ₪20,566 deposited a year")
        );
        assert_eq!(
            rule(Vehicle::StudyFund, Lang::He).as_deref(),
            Some("אין מס על הרווחים אחרי 6 שנים, על עד ₪20,566 שהופקדו בשנה")
        );
        assert_eq!(
            rule(Vehicle::InvestmentGemel, Lang::En).as_deref(),
            Some("No tax as a pension from 60; otherwise taxed like a broker")
        );
        assert_eq!(
            rule(Vehicle::InvestmentGemel, Lang::He).as_deref(),
            Some("אין מס בקצבה מגיל 60; אחרת ממוסה כמו חשבון מסחר")
        );
        assert_eq!(
            rule(Vehicle::SavingsPolicy, Lang::En).as_deref(),
            Some("Taxed like a broker")
        );
        assert_eq!(
            rule(Vehicle::SavingsPolicy, Lang::He).as_deref(),
            Some("ממוסה כמו חשבון מסחר")
        );
    }

    /// A fund's row says what became of its tax for the deposits and the
    /// way out at hand; a broker's row, and any row when nothing is sold,
    /// says nothing.
    #[test]
    fn a_funds_row_says_what_became_of_its_tax() {
        use crate::Percent;
        let monthly = |monthly_deposit, age, withdrawal| Scenario {
            security: Security::Etf,
            exchange: Exchange::Usa,
            first_deposit: dec!(0),
            monthly_deposit,
            deposit_growth: Percent(dec!(0)),
            yearly_return: Percent(dec!(10)),
            years: 20,
            buy_every_months: 1,
            share_price: dec!(500),
            sell_at_end: true,
            inflation: Percent(dec!(2)),
            age,
            withdrawal,
        };
        let at_once = |monthly_deposit| monthly(monthly_deposit, 30, Withdrawal::LumpSum);
        let note = |plan: &Plan, scenario: &Scenario| plan.tax_note(scenario, Lang::En);
        let of = |broker: Broker| broker.plans.into_iter().next().unwrap();

        // ₪1,700 a month is ₪20,400 a year: within. ₪2,000 a month isn't.
        let study = of(funds::study_fund());
        assert_eq!(
            note(&study, &at_once(dec!(1700))).as_deref(),
            Some("No tax: your deposits are within ₪20,566 a year")
        );
        assert_eq!(
            note(&study, &at_once(dec!(2000))).as_deref(),
            Some("Taxed only on what you deposit over ₪20,566 a year")
        );
        assert_eq!(
            study.tax_note(&at_once(dec!(1700)), Lang::He).as_deref(),
            Some("אין מס: ההפקדות שלכם בתוך ₪20,566 בשנה")
        );
        assert_eq!(
            study.tax_note(&at_once(dec!(2000)), Lang::He).as_deref(),
            Some("המס הוא רק על מה שמופקד מעל ₪20,566 בשנה")
        );

        let gemel = of(funds::investment_gemel());
        assert_eq!(
            note(&gemel, &at_once(dec!(2000))).as_deref(),
            Some("Taxed like a broker; no tax as a pension from 60")
        );
        assert_eq!(
            note(&gemel, &monthly(dec!(2000), 40, Withdrawal::Pension)).as_deref(),
            Some("No tax: taken as a pension from 60")
        );
        assert_eq!(
            note(&gemel, &monthly(dec!(2000), 39, Withdrawal::Pension)).as_deref(),
            Some("Taxed like a broker: the pension opens at 60, and you'd be 59")
        );
        assert_eq!(
            gemel
                .tax_note(&monthly(dec!(2000), 39, Withdrawal::Pension), Lang::He)
                .as_deref(),
            Some("ממוסה כמו חשבון מסחר: הקצבה נפתחת בגיל 60, ואתם תהיו בני 59")
        );
        assert_eq!(
            note(&of(funds::savings_policy()), &at_once(dec!(2000))).as_deref(),
            Some("Taxed like a broker")
        );
        assert_eq!(note(&of(tariffs::leumi()), &at_once(dec!(2000))), None);
        // Kept, not sold: no tax to explain.
        let kept = Scenario {
            sell_at_end: false,
            ..at_once(dec!(2000))
        };
        assert_eq!(note(&study, &kept), None);
        assert_eq!(note(&gemel, &kept), None);
    }

    /// A kind of fund says whose fees it shows and from when, where a
    /// broker names its tariff.
    #[test]
    fn a_fund_says_whose_fees_it_shows() {
        let fund = funds::investment_gemel();
        assert!(
            fund.usual_plan_text(Lang::En)
                .starts_with("What savers pay on average")
        );
        assert_eq!(
            fund.tariff_date_text(Lang::En),
            "Fees paid, by the data of 08/2026"
        );
        assert_eq!(
            fund.tariff_date_text(Lang::He),
            "דמי הניהול שנגבו, לפי נתוני 08/2026"
        );
        assert_eq!(fund.checked_on_text(Lang::En), "Checked 30/09/2026");
        assert_eq!(
            tariffs::leumi().checked_on_text(Lang::En),
            Broker::checked_text(Lang::En)
        );
        let sources = fund.sources(Lang::En);
        assert_eq!(sources[0].name, "The Capital Market Authority's data");
        assert_eq!(sources[0].url, "https://gemelnet.cma.gov.il/");
    }

    /// Hebrew letters, with the spaces, quotes and slashes the names use.
    fn is_hebrew(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| ('\u{5d0}'..='\u{5ea}').contains(&c) || " \"'/,-\u{5f4}".contains(c))
    }

    #[test]
    fn every_choice_is_explained_and_its_hebrew_names_are_hebrew() {
        fn check<T: Explained>(choices: impl Iterator<Item = T>, needs_hebrew: bool) {
            for choice in choices {
                let name = choice.name(Lang::En);
                for lang in Lang::iter() {
                    assert!(!choice.explanation(lang).is_empty(), "{name}");
                    assert!(!choice.name(lang).is_empty(), "{name}");
                }
                // The Hebrew name is Hebrew, unless it's a name like "IBI".
                assert!(
                    is_hebrew(choice.name(Lang::He)) || choice.name(Lang::He) == name,
                    "{name}"
                );
                assert!(
                    choice.hebrew_names().iter().all(|name| is_hebrew(name)),
                    "{name}"
                );
                assert!(!needs_hebrew || !choice.hebrew_names().is_empty(), "{name}");
            }
        }
        check(Security::iter(), true);
        check(Exchange::iter(), false);
        check(FeeKind::iter(), false);
        check(CaveatKind::iter(), false);
        for kind in FeeKind::iter() {
            assert_ne!(kind.label(Lang::En), "");
            assert!(is_hebrew(kind.label(Lang::He)), "{}", kind.label(Lang::He));
        }
        assert_eq!(Exchange::Tlv.hebrew_names().len(), 1);
    }

    #[test]
    fn a_price_is_nothing_only_when_nothing_is_charged() {
        let min_only = PercentFee {
            percent: Percent(Decimal::ZERO),
            min: Some(usd(dec!(5))),
            max: None,
        };
        assert!(!min_only.is_nothing());
        assert!(PercentFee::FREE.is_nothing());
        assert!(ConversionFee::FREE.is_nothing());
        assert!(
            !ConversionFee {
                fee: min_only,
                ..ConversionFee::FREE
            }
            .is_nothing()
        );
        let free_custody = &tariffs::ibi().plans[0].custody[0];
        assert!(free_custody.is_nothing());
        assert!(
            !CustodyFee {
                min: Some(ils(dec!(75))),
                ..free_custody.clone()
            }
            .is_nothing()
        );
        assert!(Markup::NONE.is_nothing());
        assert!(Markup::MarketRate.is_nothing());
        assert!(Markup::NotPublished.is_nothing());
        assert!(!Markup::UpTo(Percent(dec!(0.7))).is_nothing());
        assert!(!Markup::PerDollar(ils(dec!(0.02))).is_nothing());
        // A trade price and a handling fee are always something.
        assert!(!Price::Flat(ils(dec!(4))).price_text(Lang::En).nothing);
        assert!(
            !tariffs::altshuler().plans[0]
                .handling
                .unwrap()
                .price_text(Lang::En)
                .nothing
        );
    }

    #[test]
    fn sources_are_the_tariff_then_each_page_once() {
        let (leumi, pepper) = pepper();
        let sources = leumi.sources_for(&pepper, Lang::En);
        assert_eq!(sources[0].name, "Tariff (PDF)");
        let urls: std::collections::HashSet<&str> =
            sources.iter().map(|source| source.url.as_str()).collect();
        assert_eq!(urls.len(), sources.len(), "a page listed twice");
        // Every page the broker's and the plan's caveats rest on is there.
        let caveat_sources = leumi
            .caveats
            .iter()
            .chain(&pepper.caveats)
            .flat_map(|caveat| &caveat.sources);
        let mut counted = 0;
        for source in caveat_sources {
            assert!(urls.contains(source.url.as_str()), "{}", source.name.en);
            counted += 1;
        }
        assert!(counted > 0, "the test needs caveats with sources");
        // The plan's own: each page its caveats rest on, once.
        let own = pepper.sources(Lang::En);
        let distinct: std::collections::HashSet<&str> = pepper
            .caveats
            .iter()
            .flat_map(|caveat| &caveat.sources)
            .map(|source| source.url.as_str())
            .collect();
        assert_ne!(distinct.len(), 0, "Pepper's caveats rest on pages");
        assert_eq!(own.len(), distinct.len());
        assert!(own.iter().all(|source| urls.contains(source.url.as_str())));
        assert!(leumi.sources(Lang::En).len() >= sources.len());
        // The broker-wide caveats that matter abroad: the markup's reading.
        let groups = leumi.caveats_for(buying(Security::Etf, Exchange::Usa), &rates(), Lang::En);
        assert!(
            groups
                .iter()
                .flat_map(|group| &group.caveats)
                .any(|caveat| caveat.kind == CaveatKind::Reading && caveat.text.contains("markup")),
            "{groups:?}"
        );
    }

    #[test]
    fn the_track_part_names_the_pick_and_lists_the_others() {
        let ibi = tariffs::ibi();
        let full = &ibi.plans[0];
        let describe = |plan: &Plan, exchange, track| {
            plan.describe_fees_for(
                buying(Security::Etf, exchange),
                track,
                &[],
                &rates(),
                Lang::En,
            )
        };
        let part = |fees: &FeesFor| {
            fees.fees[0]
                .parts
                .iter()
                .find(|part| part.kind == FeeKind::Track)
                .cloned()
        };
        // Picked: the trade costs what the track charges, and the part names
        // the track, then what the others charge.
        let picked = describe(full, Exchange::Usa, Some(3));
        assert_eq!(picked.fees[0].price.text, "0.15% + $0.01 per share, min $6");
        let part_picked = part(&picked).unwrap();
        assert_eq!(part_picked.label, "track picked for you");
        assert_eq!(
            part_picked.price.text,
            "0.15% + 1¢ a share (others: $0.01 per share, min $10; $14 per order; 0.15%, min $10)"
        );
        // None picked: one of the tracks, all listed.
        let none = describe(full, Exchange::Usa, None);
        assert_eq!(none.fees[0].price.text, "one of the tracks below");
        let part_none = part(&none).unwrap();
        assert_eq!(part_none.label, "tracks");
        assert_eq!(
            part_none.price.text,
            "$0.01 per share, min $10; $14 per order; 0.15%, min $10; 0.15% + $0.01 per share, min $6"
        );
        // On Tel Aviv the tracks price nothing, so there's no part.
        assert_eq!(part(&describe(full, Exchange::Tlv, Some(3))), None);
        // With one track there are no others to list: just its name.
        let mut single = full.clone();
        single.tracks.truncate(1);
        let part_single = part(&describe(&single, Exchange::Usa, Some(0))).unwrap();
        assert_eq!(part_single.price.text, single.tracks[0].name.en);
    }

    #[test]
    fn the_fees_warning_needs_fees_beyond_the_deposits() {
        use crate::simulation::{Fees, Outcome};
        let outcome = Outcome {
            value_by_month: vec![],
            held: Decimal::ZERO,
            after_selling: Decimal::ZERO,
            fees: Fees {
                purchases: dec!(300),
                ..Fees::default()
            },
            fees_by_year: vec![],
            track: None,
            largest_trade: Decimal::ZERO,
            yearly_cost: Percent::default(),
            tax: Decimal::ZERO,
            after_tax: Decimal::ZERO,
        };
        assert_eq!(
            outcome.warning(dec!(200), Lang::En),
            Some("Its fees are more than you deposit")
        );
        assert_eq!(
            outcome.warning(dec!(300), Lang::En),
            None,
            "as much isn't more"
        );
        assert_eq!(outcome.warning(dec!(400), Lang::En), None);
    }

    #[test]
    fn a_handling_fee_counts_its_free_months_in_years_where_it_can() {
        let fee = |free_months, less_trade_fees| HandlingFee {
            per_month: ils(dec!(15)),
            free_months,
            less_trade_fees,
        };
        assert_eq!(fee(0, false).text(Lang::En), "₪15 a month");
        assert_eq!(
            fee(12, false).text(Lang::En),
            "free for the first year, then ₪15 a month"
        );
        assert_eq!(
            fee(24, false).text(Lang::En),
            "free for the first 2 years, then ₪15 a month"
        );
        assert_eq!(
            fee(18, false).text(Lang::En),
            "free for the first 18 months, then ₪15 a month"
        );
        assert_eq!(
            fee(6, true).text(Lang::En),
            "free for the first 6 months, then ₪15 a month, less that month's trade fees"
        );
    }

    #[test]
    fn the_track_note_names_the_track_where_it_prices_the_trade() {
        let full = &tariffs::altshuler().plans[0];
        assert_eq!(
            full.track_note(Security::Etf, Exchange::Usa, Some(1), Lang::En)
                .as_deref(),
            Some("US track: $11 per order, the cheapest for you")
        );
        // Not on Tel Aviv, where the tracks price nothing; not without a track.
        assert_eq!(
            full.track_note(Security::Etf, Exchange::Tlv, Some(1), Lang::En),
            None
        );
        assert_eq!(
            full.track_note(Security::Etf, Exchange::Usa, None, Lang::En),
            None
        );
    }
}

// ─────────────────────────── The short term ───────────────────────────

impl short_term::Kind {
    /// "Rates given, by the data of 08/2026", "Fees, by the data of
    /// 30/09/2026".
    #[must_use]
    pub fn data_of_text(&self, lang: Lang) -> String {
        let date = match self.data_of {
            TariffDate::Day(date) => date.format(format_description!("[day]/[month]/[year]")),
            TariffDate::Month(date) => date.format(format_description!("[month]/[year]")),
        }
        .expect("a fixed format");
        let deposits = matches!(
            self.places.first().map(|place| place.pays),
            Some(short_term::Pays::Fixed(_))
        );
        match (deposits, lang) {
            (true, Lang::En) => format!("Rates given, by the data of {date}"),
            (true, Lang::He) => format!("הריביות שניתנו, לפי נתוני {date}"),
            (false, Lang::En) => format!("Fees, by the data of {date}"),
            (false, Lang::He) => format!("דמי הניהול, לפי נתוני {date}"),
        }
    }

    /// "Checked 01/10/2026"
    #[must_use]
    pub fn checked_on_text(&self, lang: Lang) -> String {
        checked_text(self.checked, lang)
    }

    /// Where its figures come from, then every page its and its places'
    /// caveats rest on, each once, in the order first named.
    #[must_use]
    pub fn sources(&self, lang: Lang) -> Vec<Source> {
        let caveats = self
            .caveats
            .iter()
            .chain(self.places.iter().flat_map(|place| &place.caveats));
        links(
            &each_once(
                std::iter::once(&self.source).chain(caveats.flat_map(|caveat| &caveat.sources)),
            ),
            lang,
        )
    }
}

impl short_term::Place {
    /// Every page its numbers rest on, each once: its kind's source, then
    /// the pages its own caveats and its kind's rest on.
    #[must_use]
    pub fn sources(&self, kind: &short_term::Kind, lang: Lang) -> Vec<Source> {
        let caveats = self.caveats.iter().chain(&kind.caveats);
        links(
            &each_once(
                std::iter::once(&kind.source).chain(caveats.flat_map(|caveat| &caveat.sources)),
            ),
            lang,
        )
    }

    /// What a fund pays, in a line: "The Bank of Israel's rate, less 0.169% a
    /// year". None for a deposit, whose rate depends on the term.
    #[must_use]
    pub fn pays_text(&self, lang: Lang) -> Option<String> {
        match self.pays {
            short_term::Pays::Fixed(_) => None,
            short_term::Pays::TheRateLess(fee) => Some(match lang {
                Lang::En => format!("The Bank of Israel's rate, less {fee} a year"),
                Lang::He => format!("ריבית בנק ישראל, פחות {fee} בשנה"),
            }),
        }
    }
}

impl short_term::NotOffered {
    /// Why the money can't be kept there, in full.
    #[must_use]
    pub fn reason(self, lang: Lang) -> String {
        match (self, lang) {
            (short_term::NotOffered::TakesOneSum, Lang::En) => "A fixed-rate deposit takes one sum, \
                 not money every month: saving monthly at a bank is a savings plan \
                 (תוכנית\u{a0}חיסכון), whose rates the Bank of Israel doesn't publish."
                .to_owned(),
            (short_term::NotOffered::TakesOneSum, Lang::He) => "פיקדון בריבית קבועה מקבל סכום אחד, ולא הפקדה חודשית: חיסכון חודשי בבנק הוא תוכנית חיסכון, ובנק ישראל לא מפרסם את הריביות שלה.".to_owned(),
            (short_term::NotOffered::NoRate { term }, Lang::En) => format!(
                "The Bank of Israel published no rate at this bank for deposits of {}: too \
                 few were opened in the month.",
                term.name(lang).to_lowercase()
            ),
            (short_term::NotOffered::NoRate { term }, Lang::He) => format!(
                "בנק ישראל לא פרסם ריבית בבנק הזה לפיקדונות של {}: נפתחו בו מעט מדי פיקדונות כאלה בחודש.",
                term.name(lang)
            ),
        }
    }

    /// The same, in a few words for a table's row.
    #[must_use]
    pub fn short(self, lang: Lang) -> &'static str {
        match self {
            short_term::NotOffered::TakesOneSum => lang.pick("Takes one sum", "מקבל סכום אחד"),
            short_term::NotOffered::NoRate { .. } => {
                lang.pick("No rate for this term", "אין ריבית לתקופה הזו")
            }
        }
    }
}

/// The short-term calculator's page on its numbers: how they're made, what
/// isn't counted, and where they come from.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the page's text")]
pub fn about_short_term(lang: Lang) -> About {
    let paragraph = |en: &str, he: &str| lang.pick(en, he).to_owned();
    let method = Section {
        title: paragraph("How the numbers are made", "איך המספרים מחושבים"),
        paragraphs: vec![
            paragraph(
                "Each place is followed month by month, and money you put in arrives at the \
                 start of a month. A money market fund earns the Bank of Israel's rate you \
                 expect, on average over the months, less its fee, which it takes from its \
                 assets; each deposit grows from its month. A fixed-rate deposit pays its \
                 bank's rate for the term your months fall in, as the Bank of Israel \
                 published it: part of a year earns that part of the year's interest, and a \
                 whole year's interest joins the deposit and earns interest too.",
                "המחשבון עוקב אחרי כל אפיק חודש אחר חודש, וכל הפקדה מגיעה בתחילת חודש. קרן כספית מרוויחה את ריבית בנק ישראל שאתם מצפים לה, בממוצע על פני התקופה, פחות דמי הניהול שהיא לוקחת מהנכסים שלה; כל הפקדה צומחת מהחודש שבו הופקדה. פיקדון בריבית קבועה משלם את הריבית של הבנק לתקופה שבה נופלים החודשים שבחרתם, כפי שבנק ישראל פרסם אותה: חלק משנה מקבל את החלק היחסי מהריבית של השנה, והריבית של שנה שלמה מצטרפת לפיקדון ומקבלת גם היא ריבית.",
            ),
            match lang {
                Lang::En => format!(
                    "A deposit's interest is taxed at 15%, all of it, as any interest on money \
                     not linked to the price index. A fund pays tax when it's sold: 25% of the \
                     gain, with what each deposit cost first raised by how much prices rose \
                     since it went in ({USUAL_INFLATION} a year, unless you set another under \
                     \u{201c}More options\u{201d}), so a gain that only keeps up with prices \
                     isn't taxed."
                ),
                Lang::He => format!(
                    "על ריבית בפיקדון משלמים מס של 15%, על כולה, כמו על כל ריבית על כסף שאינו צמוד למדד. על קרן כספית משלמים מס כשמוכרים אותה: 25% מהרווח, כשאת מה שהפקדתם מתרגמים קודם לערך הכסף ביום המכירה, לפי האינפלציה מאז כל הפקדה ({USUAL_INFLATION} בשנה, אלא אם קבעתם אינפלציה אחרת ב״אפשרויות נוספות״), כך שרווח שרק שומר על ערך הכסף לא ממוסה."
                ),
            },
            paragraph(
                "What a place keeps is measured against the same deposits at the Bank of \
                 Israel's rate, with nothing kept and no tax: a fund's fee, or how much less \
                 than the rate a bank pays. Its yearly cost states that the way a fund's fee \
                 is stated; a bank that pays more than the rate has a cost below zero.",
                "מה שאפיק שומר לעצמו נמדד מול אותן הפקדות בריבית בנק ישראל, בלי שום עלות ובלי מס: דמי הניהול של קרן, או כמה פחות מהריבית הבנק משלם. העלות השנתית מבטאת את זה כמו דמי ניהול של קרן; לבנק שמשלם יותר מהריבית יש עלות מתחת לאפס.",
            ),
            paragraph(
                "A fund's units can be sold any business day. A deposit locks the money until \
                 its term ends, and takes one sum: with money put in every month, no bank is \
                 compared.",
                "את יחידות הקרן אפשר למכור בכל יום עסקים. פיקדון נועל את הכסף עד סוף התקופה, ומקבל סכום אחד: כשמפקידים כל חודש, הבנקים לא מושווים.",
            ),
        ],
        items: vec![],
        sources: vec![],
    };
    let left_out = Section {
        title: paragraph("What isn't counted", "מה לא נכלל"),
        paragraphs: vec![paragraph(
            "Left out, so no place looks better than it is, or because it depends on you \
             rather than on the place:",
            "נשאר בחוץ, כדי שאף אפיק לא ייראה טוב ממה שהוא, או כי זה תלוי בכם ולא באפיק:",
        )],
        items: vec![
            item(
                lang,
                "Deposits at a variable rate, and the daily deposit; a bank's savings plan, its \
                 way to save every month, whose rates aren't published.",
                "פיקדונות בריבית משתנה, והפיקדון היומי; תוכנית חיסכון בבנק, הדרך שלו לחסוך כל חודש, שהריביות שלה לא מתפרסמות.",
            ),
            item(
                lang,
                "Taking a deposit out before its term ends, and renewing a shorter deposit on \
                 the way.",
                "משיכת פיקדון לפני סוף התקופה, וחידוש של פיקדון קצר יותר בדרך.",
            ),
            item(
                lang,
                "Linked and foreign-currency deposits and funds; makam, a fund's cash track \
                 (מסלול\u{a0}כספי) and fixed-term money market funds.",
                "פיקדונות וקרנות צמודים למדד או במטבע חוץ; מק״מ, מסלול כספי בקופת גמל, וקרנות כספיות מתחדשות.",
            ),
            item(
                lang,
                "What a fund really earns, a little more or less than the rate.",
                "כמה קרן מרוויחה בפועל, קצת יותר או פחות מהריבית.",
            ),
            item(
                lang,
                "A trade fee on a fund at an investment house.",
                "עמלת קנייה ומכירה על קרן בבית השקעות.",
            ),
            item(
                lang,
                "A lower tax rate on a low income, and the surtax on a high one.",
                "שיעור מס נמוך יותר בהכנסה נמוכה, ומס יסף בהכנסה גבוהה.",
            ),
        ],
        sources: vec![],
    };
    let mut groups: Vec<SourceGroup> = short_term::kinds()
        .iter()
        .map(|kind| SourceGroup {
            title: format!("{} · {}", &kind.name[lang], kind.data_of_text(lang)),
            sources: kind.sources(lang),
        })
        .collect();
    groups.push(SourceGroup {
        title: paragraph("The Bank of Israel's rate", "ריבית בנק ישראל"),
        sources: links(&short_term::todays_rate_sources(), lang),
    });
    let checked = checked_text(short_term::deposits::checked(), lang);
    let sources = Section {
        title: paragraph("Sources", "מקורות"),
        paragraphs: vec![match lang {
            Lang::En => format!(
                "{checked} against the Bank of Israel's figures and the funds' reports to \
                 the Tel Aviv Stock Exchange. The banks' rates change every month and a fund's \
                 fee at most once a year: each place's details link to what its numbers rest \
                 on."
            ),
            Lang::He => format!(
                "{checked} מול הנתונים של בנק ישראל והדיווחים של הקרנות לבורסה בתל אביב. הריביות בבנקים משתנות כל חודש, ודמי הניהול של קרן לכל היותר פעם בשנה: בפרטי כל אפיק יש קישור למה שהמספרים שלו מבוססים עליו."
            ),
        }],
        items: vec![],
        sources: groups,
    };
    About {
        sections: vec![method, left_out, sources],
    }
}

#[cfg(test)]
mod short_term_tests {
    use super::*;
    use crate::Percent;
    use rust_decimal_macros::dec;

    /// The same three sections as the long term's page, so the header's
    /// links open either; its sources grouped by kind, each kind's own page
    /// first, then the rate's.
    #[test]
    fn the_short_terms_page_has_the_same_sections_and_its_own_sources() {
        let long = about(Lang::He);
        let short = about_short_term(Lang::He);
        let titles = |page: &About| {
            page.sections
                .iter()
                .map(|section| section.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(&short), titles(&long));
        let groups: Vec<&str> = short.sections[2]
            .sources
            .iter()
            .map(|group| group.title.as_str())
            .collect();
        assert_eq!(
            groups,
            [
                "קרן כספית · דמי הניהול, לפי נתוני 30/09/2026",
                "פיקדון בריבית קבועה · הריביות שניתנו, לפי נתוני 08/2026",
                "ריבית בנק ישראל",
            ]
        );
        for kind in short_term::kinds() {
            let sources = kind.sources(Lang::En);
            assert_eq!(sources[0].url, kind.source.url);
            let urls: Vec<&str> = sources.iter().map(|source| source.url.as_str()).collect();
            let mut once = urls.clone();
            once.dedup();
            assert_eq!(urls.len(), once.len(), "each page once");
        }
        assert!(short.sections[1].items.len() >= 5);
    }

    #[test]
    fn a_place_not_offered_says_why() {
        let one_sum = short_term::NotOffered::TakesOneSum;
        assert_eq!(one_sum.short(Lang::He), "מקבל סכום אחד");
        assert!(one_sum.reason(Lang::He).contains("תוכנית חיסכון"));
        let no_rate = short_term::NotOffered::NoRate {
            term: short_term::Term::UpToFiveYears,
        };
        assert_eq!(no_rate.short(Lang::He), "אין ריבית לתקופה הזו");
        assert!(no_rate.reason(Lang::He).contains("3 שנים עד 5 שנים"));
        assert!(no_rate.reason(Lang::En).contains("3 to 5 years"));
    }

    #[test]
    fn a_fund_says_what_it_pays_and_a_deposit_leaves_it_to_its_terms() {
        let fund = short_term::Place {
            pays: short_term::Pays::TheRateLess(Percent(dec!(0.169))),
            ..short_term::your_deposit(Percent(dec!(3)))
        };
        assert_eq!(
            fund.pays_text(Lang::He).unwrap(),
            "ריבית בנק ישראל, פחות 0.169% בשנה"
        );
        assert_eq!(
            short_term::your_deposit(Percent(dec!(3))).pays_text(Lang::He),
            None
        );
    }

    /// Grouped by how sure they are, the ones that may cost more first.
    #[test]
    fn caveats_group_by_how_sure_they_are() {
        let published = Caveat::published(Text::same("a"));
        let may_cost_more = Caveat::may_cost_more(Text::same("b"), Text::same("B"));
        let groups = caveat_groups(&[&published, &may_cost_more], Lang::En);
        let kinds: Vec<CaveatKind> = groups.iter().map(|group| group.kind).collect();
        assert_eq!(kinds, [CaveatKind::MayCostMore, CaveatKind::Published]);
    }
}
