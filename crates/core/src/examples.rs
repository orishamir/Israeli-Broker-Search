//! What to type into the bank's app to find what the site describes
//! (REDESIGN.md, principle 14): the series of Israeli government bonds and
//! the Bank of Israel's bills, with the day each is repaid, and real funds
//! that hold the S&P 500 or keep money in a money market fund, each with the
//! number or ticker it's found by.
//!
//! The funds are examples, never a pick (principle 17): in Tel Aviv every
//! fund of each kind the script found, abroad the three US funds most
//! Israelis meet (VOO, IVV, SPY) and the Irish CSPX, each kind in order of
//! size, an order that says nothing about which is better. Whatever shows one says so
//! ([`NOT_A_PICK`]). A bond series is different: the one to hold to a date is
//! decided by the date ([`nearest_bond`]).
//!
//! Every name and number is fetched by `policies/examples.py`, which rewrites
//! the block between its two marker lines; `policies/sources.md` says from
//! where. Nothing here is typed by hand, so where a source didn't answer, the
//! field is `None` and the page shows a marked placeholder (principle 16).

// The exchange's numbers are names, written as it writes them, not amounts
// to group by thousands.
#![allow(
    clippy::unreadable_literal,
    reason = "the exchange's numbers, as it writes them"
)]

use std::fmt;

use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use crate::short_term::government_bonds::Linkage;
use crate::ways::Way;
use crate::{Exchange, Lang, Named, Page, Percent, Product, Text};

/// The words beside every example fund, wherever one is shown.
pub const NOT_A_PICK: Text = Text::new("an example, not a recommendation", "דוגמה, לא המלצה");

/// The day the script last fetched everything here.
#[must_use]
pub fn checked() -> Date {
    CHECKED
}

/// The number the Tel Aviv Stock Exchange gives a security or a fund (מספר
/// נייר, מספר קרן): the surest thing to search for, since every bank's app
/// finds a security by it, whatever it calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SecurityNumber(pub u32);

// Written as the bare number, as the exchange and the apps write it. (The
// workspace's `derive_more` has no "display" feature, and one impl is
// shorter than turning it on.)
impl fmt::Display for SecurityNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A page on the exchange's site about one security.
fn exchange_page(number: SecurityNumber) -> Page {
    Page {
        name: Text::new(
            "The security's page on the Tel Aviv Stock Exchange's site",
            "הדף של נייר הערך באתר הבורסה בתל אביב",
        ),
        url: format!("https://market.tase.co.il/he/market_data/security/{number}/major_data"),
    }
}

// ─────────────────────────── Bonds ───────────────────────────

/// A series of an Israeli government bond, or of the Bank of Israel's bills
/// (מק״מ), as the exchange lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BondSeries {
    /// Its name on the exchange, which is what the bank's app shows: "ממשל
    /// צמודה 0529", a trade name the site writes only in the slot for what
    /// to search for (REDESIGN.md §2.8).
    pub name: &'static str,
    pub number: SecurityNumber,
    pub linkage: Linkage,
    /// The day the last of it is repaid, as the exchange gives it. The
    /// script checks it against the month and year the name ends in, so a
    /// part payment can't pass for the end.
    pub repaid: Date,
}

impl BondSeries {
    /// Where its name, number and day come from.
    #[must_use]
    pub fn page(&self) -> Page {
        exchange_page(self.number)
    }
}

/// Every CPI-linked (ממשל צמודה) and fixed-rate shekel (ממשל שקלית) series
/// on the exchange, by the day each is repaid. The variable-rate and the
/// dollar ones are left out: the app doesn't model them.
#[must_use]
pub fn government_bonds() -> &'static [BondSeries] {
    GOVERNMENT_BONDS
}

/// Every series of the Bank of Israel's bills, by the day each is repaid.
/// They pay no interest: each is sold below the shekel it repays.
#[must_use]
pub fn makam() -> &'static [BondSeries] {
    MAKAM_SERIES
}

/// The series that is repaid last among those repaid after `after` (today)
/// and no later than `by` (the visitor's date): a bond held to maturity must
/// have paid back by the day the money is needed, and the latest such one
/// earns its yield the longest. `None` if no series is repaid in time.
///
/// `after` keeps out a series already repaid: the list is as old as
/// [`checked`].
#[must_use]
pub fn nearest_bond(linkage: Linkage, after: Date, by: Date) -> Option<&'static BondSeries> {
    latest_between(
        GOVERNMENT_BONDS
            .iter()
            .filter(|bond| bond.linkage == linkage),
        after,
        by,
    )
}

/// The bill repaid last after `after` and no later than `by`, as for
/// [`nearest_bond`].
#[must_use]
pub fn nearest_makam(after: Date, by: Date) -> Option<&'static BondSeries> {
    latest_between(MAKAM_SERIES.iter(), after, by)
}

fn latest_between(
    series: impl Iterator<Item = &'static BondSeries>,
    after: Date,
    by: Date,
) -> Option<&'static BondSeries> {
    series
        .filter(|bond| after < bond.repaid && bond.repaid <= by)
        .max_by_key(|bond| bond.repaid)
}

// ─────────────────────────── Funds ───────────────────────────

/// What a fund is found by in the bank's app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindBy {
    /// In Tel Aviv: the fund's or the security's number.
    Number(SecurityNumber),
    /// Abroad: the ticker, the same in every broker's app.
    Ticker(&'static str),
}

/// Where a fund is bought: the market whose name the identity line gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum Venue {
    /// The Tel Aviv Stock Exchange, where an index fund is listed too,
    /// though it's bought from its manager once a day.
    TelAviv,
    /// NYSE Arca, where VOO, IVV and SPY are listed.
    NyseArca,
    /// Euronext Amsterdam, where CSPX trades in euros (in London it trades
    /// in dollars, a market the app doesn't model yet).
    EuronextAmsterdam,
}

impl Named for Venue {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Venue::TelAviv => lang.pick("the Tel Aviv Stock Exchange", "הבורסה\u{a0}בתל\u{a0}אביב"),
            Venue::NyseArca => "NYSE Arca",
            Venue::EuronextAmsterdam => "Euronext Amsterdam",
        }
    }
}

impl Venue {
    /// The app's exchange, whose currency and fees apply.
    #[must_use]
    pub fn exchange(self) -> Exchange {
        match self {
            Venue::TelAviv => Exchange::Tlv,
            Venue::NyseArca => Exchange::Usa,
            Venue::EuronextAmsterdam => Exchange::Europe,
        }
    }
}

/// Where an example's fee comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum Source {
    /// Maya, the exchange's list of mutual funds: an index fund's fees.
    Maya,
    /// Funder, which lists an Israeli ETF's three fees as plain HTML (the
    /// exchange's API doesn't give them).
    Funder,
    /// The exchange's own data, for the foreign funds listed there.
    TelAvivExchange,
    Vanguard,
    IsharesUs,
    Ssga,
    IsharesEurope,
}

/// A real fund that holds the S&P 500, to search for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Example {
    /// Its name as its source writes it (in Tel Aviv, the exchange's name,
    /// stray marks and all): a trade name, shown only beside its number.
    pub name: &'static str,
    pub find_by: FindBy,
    /// The kind of product it is, which ties it to the ways that hold it.
    pub product: Product,
    pub venue: Venue,
    /// Its fixed fee a year, the trustee's included: what its source
    /// publishes, if the script could read it.
    pub fee: Option<Percent>,
    /// The most an Israeli ETF's variable fee can add a year, when it beats
    /// the index.
    pub variable_fee_up_to: Option<Percent>,
    pub source: Source,
}

impl Example {
    /// The page its name, number and fee come from.
    ///
    /// # Panics
    ///
    /// A Tel Aviv source with a ticker instead of a number, which the tests
    /// rule out for the data.
    #[must_use]
    pub fn page(&self) -> Page {
        let page = |en, he, url: String| Page {
            name: Text::new(en, he),
            url,
        };
        let number = || match self.find_by {
            FindBy::Number(number) => number,
            FindBy::Ticker(ticker) => panic!("{ticker}: a Tel Aviv source names a number"),
        };
        match self.source {
            Source::Maya => page(
                "The fund's page on Maya, the Tel Aviv Stock Exchange's list of mutual funds",
                "הדף של הקרן במאיה, רשימת קרנות הנאמנות של הבורסה בתל אביב",
                format!("https://maya.tase.co.il/he/funds/mutual-funds/{}", number()),
            ),
            Source::Funder => page(
                "The ETF's page on Funder",
                "הדף של קרן הסל בפאנדר",
                format!("https://www.funder.co.il/etf/{}", number()),
            ),
            Source::TelAvivExchange => exchange_page(number()),
            Source::Vanguard => page(
                "Vanguard S&P 500 ETF (VOO)",
                "Vanguard S&P\u{a0}500 ETF (VOO)",
                "https://investor.vanguard.com/investment-products/etfs/profile/voo".to_owned(),
            ),
            Source::IsharesUs => page(
                "iShares Core S&P 500 ETF (IVV)",
                "iShares Core S&P\u{a0}500 ETF (IVV)",
                "https://www.ishares.com/us/products/239726/ishares-core-sp-500-etf".to_owned(),
            ),
            Source::Ssga => page(
                "SPDR S&P 500 ETF Trust (SPY)",
                "SPDR S&P\u{a0}500 ETF Trust (SPY)",
                "https://www.ssga.com/us/en/intermediary/etfs/spdr-sp-500-etf-trust-spy".to_owned(),
            ),
            Source::IsharesEurope => page(
                "iShares Core S&P 500 UCITS ETF (CSPX), with its listings",
                "iShares Core S&P\u{a0}500 UCITS ETF (CSPX), עם הבורסות שבהן היא נסחרת",
                "https://www.ishares.com/uk/individual/en/products/253743/\
                 ishares-sp-500-b-ucits-etf-acc-fund"
                    .to_owned(),
            ),
        }
    }
}

/// Every example: Israeli index funds, Israeli ETFs, the Irish ETFs listed in
/// Tel Aviv, then the US ones and the Irish one bought in Europe. Within each
/// kind, largest first by what its source reports (Maya's and the exchange's
/// assets, the issuers' net assets), which is no ranking.
#[must_use]
pub fn funds() -> &'static [Example] {
    FUNDS
}

/// The examples of one kind of product, in [`funds`]' order.
pub fn of_product(product: Product) -> impl Iterator<Item = &'static Example> {
    FUNDS.iter().filter(move |fund| fund.product == product)
}

/// The examples a way can hold: its product, bought where the way buys it.
/// A fund's track has none: it's chosen inside the fund, not searched for.
pub fn for_way(way: Way) -> impl Iterator<Item = &'static Example> {
    of_product(way.product).filter(move |fund| {
        way.exchange
            .is_some_and(|exchange| fund.venue.exchange() == exchange)
    })
}

// ─────────────────────────── Money market funds ───────────────────────────

/// A shekel money market fund (קרן כספית שקלית) on Maya.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyMarketFund {
    /// Its name on Maya, as the bank's app shows it.
    pub name: &'static str,
    pub number: SecurityNumber,
    /// The manager's fee and the trustee's, a year.
    pub fee: Percent,
}

impl MoneyMarketFund {
    /// Its page on Maya.
    #[must_use]
    pub fn page(&self) -> Page {
        Page {
            name: Text::new(
                "The fund's page on Maya, the Tel Aviv Stock Exchange's list of mutual funds",
                "הדף של הקרן במאיה, רשימת קרנות הנאמנות של הבורסה בתל אביב",
            ),
            url: format!(
                "https://maya.tase.co.il/he/funds/mutual-funds/{}",
                self.number
            ),
        }
    }
}

/// Every shekel money market fund on Maya, largest first.
#[must_use]
pub fn money_market_funds() -> &'static [MoneyMarketFund] {
    MONEY_MARKET_FUNDS
}

/// A money market fund by its name on Maya: how the kind's named funds
/// (`short_term::money_market`) find their numbers.
#[must_use]
pub fn money_market_fund(name: &str) -> Option<&'static MoneyMarketFund> {
    MONEY_MARKET_FUNDS.iter().find(|fund| fund.name == name)
}

// The data, rewritten by `policies/examples.py --write` (then `cargo fmt`).
// BEGIN examples.py
const CHECKED: Date = date!(2026 - 10 - 20);

/// Every series, by the day it's repaid.
const GOVERNMENT_BONDS: &[BondSeries] = &[
    BondSeries {
        name: "ממשל שקלית 1026",
        number: SecurityNumber(1099456),
        linkage: Linkage::Unlinked,
        repaid: date!(2026 - 10 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0327",
        number: SecurityNumber(1139344),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 03 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0527",
        number: SecurityNumber(1140847),
        linkage: Linkage::CpiLinked,
        repaid: date!(2027 - 05 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0927",
        number: SecurityNumber(1203579),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 09 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0728",
        number: SecurityNumber(1226240),
        linkage: Linkage::Unlinked,
        repaid: date!(2028 - 07 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0928",
        number: SecurityNumber(1150879),
        linkage: Linkage::Unlinked,
        repaid: date!(2028 - 09 - 28),
    },
    BondSeries {
        name: "ממשל צמודה 1028",
        number: SecurityNumber(1197326),
        linkage: Linkage::CpiLinked,
        repaid: date!(2028 - 10 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0229",
        number: SecurityNumber(1194802),
        linkage: Linkage::Unlinked,
        repaid: date!(2029 - 02 - 28),
    },
    BondSeries {
        name: "ממשל צמודה 0529",
        number: SecurityNumber(1157023),
        linkage: Linkage::CpiLinked,
        repaid: date!(2029 - 05 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0829",
        number: SecurityNumber(1212893),
        linkage: Linkage::Unlinked,
        repaid: date!(2029 - 08 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0330",
        number: SecurityNumber(1160985),
        linkage: Linkage::Unlinked,
        repaid: date!(2030 - 03 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0530",
        number: SecurityNumber(1241991),
        linkage: Linkage::Unlinked,
        repaid: date!(2030 - 05 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0431",
        number: SecurityNumber(1220722),
        linkage: Linkage::CpiLinked,
        repaid: date!(2031 - 04 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0731",
        number: SecurityNumber(1233782),
        linkage: Linkage::Unlinked,
        repaid: date!(2031 - 07 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 1131",
        number: SecurityNumber(1172220),
        linkage: Linkage::CpiLinked,
        repaid: date!(2031 - 11 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0432",
        number: SecurityNumber(1180660),
        linkage: Linkage::Unlinked,
        repaid: date!(2032 - 04 - 30),
    },
    BondSeries {
        name: "ממשל צמודה 1033",
        number: SecurityNumber(1204379),
        linkage: Linkage::CpiLinked,
        repaid: date!(2033 - 10 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0335",
        number: SecurityNumber(1202332),
        linkage: Linkage::Unlinked,
        repaid: date!(2035 - 03 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 1035",
        number: SecurityNumber(1227784),
        linkage: Linkage::Unlinked,
        repaid: date!(2035 - 10 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0536",
        number: SecurityNumber(1097708),
        linkage: Linkage::CpiLinked,
        repaid: date!(2036 - 05 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0537",
        number: SecurityNumber(1166180),
        linkage: Linkage::Unlinked,
        repaid: date!(2037 - 05 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0837",
        number: SecurityNumber(1237205),
        linkage: Linkage::CpiLinked,
        repaid: date!(2037 - 08 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0841",
        number: SecurityNumber(1120583),
        linkage: Linkage::CpiLinked,
        repaid: date!(2041 - 08 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 0142",
        number: SecurityNumber(1125400),
        linkage: Linkage::Unlinked,
        repaid: date!(2042 - 01 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 0545",
        number: SecurityNumber(1134865),
        linkage: Linkage::CpiLinked,
        repaid: date!(2045 - 05 - 31),
    },
    BondSeries {
        name: "ממשל שקלית 0347",
        number: SecurityNumber(1140193),
        linkage: Linkage::Unlinked,
        repaid: date!(2047 - 03 - 31),
    },
    BondSeries {
        name: "ממשל צמודה 1151",
        number: SecurityNumber(1168301),
        linkage: Linkage::CpiLinked,
        repaid: date!(2051 - 11 - 30),
    },
    BondSeries {
        name: "ממשל שקלית 1152",
        number: SecurityNumber(1184076),
        linkage: Linkage::Unlinked,
        repaid: date!(2052 - 11 - 29),
    },
    BondSeries {
        name: "ממשל צמודה 0456",
        number: SecurityNumber(1230713),
        linkage: Linkage::CpiLinked,
        repaid: date!(2056 - 04 - 30),
    },
];

/// Every bill, by the day it's repaid.
const MAKAM_SERIES: &[BondSeries] = &[
    BondSeries {
        name: "מ.ק.מ. 1116",
        number: SecurityNumber(8261117),
        linkage: Linkage::Unlinked,
        repaid: date!(2026 - 11 - 04),
    },
    BondSeries {
        name: "מ.ק.מ. 1216",
        number: SecurityNumber(8261216),
        linkage: Linkage::Unlinked,
        repaid: date!(2026 - 12 - 02),
    },
    BondSeries {
        name: "מ.ק.מ. 117",
        number: SecurityNumber(8270118),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 01 - 06),
    },
    BondSeries {
        name: "מ.ק.מ 217",
        number: SecurityNumber(8270217),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 02 - 03),
    },
    BondSeries {
        name: "מ.ק.מ 317",
        number: SecurityNumber(8270316),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 03 - 03),
    },
    BondSeries {
        name: "מ.ק.מ. 417",
        number: SecurityNumber(8270415),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 04 - 07),
    },
    BondSeries {
        name: "מ.ק.מ. 517",
        number: SecurityNumber(8270514),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 05 - 05),
    },
    BondSeries {
        name: "מ.ק.מ. 617",
        number: SecurityNumber(8270613),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 06 - 02),
    },
    BondSeries {
        name: "מ.ק.מ 717",
        number: SecurityNumber(8270712),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 07 - 07),
    },
    BondSeries {
        name: "מ.ק.מ. 817",
        number: SecurityNumber(8270811),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 08 - 04),
    },
    BondSeries {
        name: "מ.ק.מ 927",
        number: SecurityNumber(8270928),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 09 - 08),
    },
    BondSeries {
        name: "מ.ק.מ. 1017",
        number: SecurityNumber(8271017),
        linkage: Linkage::Unlinked,
        repaid: date!(2027 - 10 - 06),
    },
];

/// Largest first, in each group, by what its source reports.
const FUNDS: &[Example] = &[
    Example {
        name: "קסם SP500 KTF",
        find_by: FindBy::Number(SecurityNumber(5124482)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.025))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "הראל מחקהSP500",
        find_by: FindBy::Number(SecurityNumber(5129275)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.025))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "MTF מח SP500",
        find_by: FindBy::Number(SecurityNumber(5122627)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.03))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "תכלית SP500 TTF",
        find_by: FindBy::Number(SecurityNumber(5113998)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.33))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "איביאי מחקה S&P 500",
        find_by: FindBy::Number(SecurityNumber(5127469)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.275))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "מור מחקה S&P500",
        find_by: FindBy::Number(SecurityNumber(5117601)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.42))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "י.ל מחקה S&P 500",
        find_by: FindBy::Number(SecurityNumber(5139282)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.02))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "א.ש מחקה S&P 500",
        find_by: FindBy::Number(SecurityNumber(5139100)),
        product: Product::IsraeliIndexFund,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.03))),
        variable_fee_up_to: None,
        source: Source::Maya,
    },
    Example {
        name: "תכלית סל (4D) S&P 500",
        find_by: FindBy::Number(SecurityNumber(1144385)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.73))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "MTF סל (S&P 500 (4D",
        find_by: FindBy::Number(SecurityNumber(1150333)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.28))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "מור סל (4D)י S&P 500",
        find_by: FindBy::Number(SecurityNumber(1165810)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.12))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "הראל סל (4D) S&P 500",
        find_by: FindBy::Number(SecurityNumber(1149020)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.608))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "קסם S&P 500 (4D) ETF",
        find_by: FindBy::Number(SecurityNumber(1146471)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.63))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "אי.בי.אי. סל (4D)י S&P 500",
        find_by: FindBy::Number(SecurityNumber(1148162)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.8085))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "ילין לפידות סל (4D)י S&P 500",
        find_by: FindBy::Number(SecurityNumber(1201656)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.02))),
        variable_fee_up_to: Some(Percent(dec!(0.3))),
        source: Source::Funder,
    },
    Example {
        name: "ATF סל (4D)י S&P 500",
        find_by: FindBy::Number(SecurityNumber(1218916)),
        product: Product::IsraeliEtf,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.008))),
        variable_fee_up_to: Some(Percent(dec!(0.2))),
        source: Source::Funder,
    },
    Example {
        name: "ISHARES CORE S&P 500 UCITS ETF",
        find_by: FindBy::Number(SecurityNumber(1159250)),
        product: Product::ForeignEtfInTelAviv,
        venue: Venue::TelAviv,
        fee: Some(Percent(dec!(0.07))),
        variable_fee_up_to: None,
        source: Source::TelAvivExchange,
    },
    Example {
        name: "INVESCO S&P 500 UCITS ETF",
        find_by: FindBy::Number(SecurityNumber(1183441)),
        product: Product::ForeignEtfInTelAviv,
        venue: Venue::TelAviv,
        fee: None,
        variable_fee_up_to: None,
        source: Source::TelAvivExchange,
    },
    Example {
        name: "Vanguard S&P 500 ETF",
        find_by: FindBy::Ticker("VOO"),
        product: Product::UsFund,
        venue: Venue::NyseArca,
        fee: Some(Percent(dec!(0.03))),
        variable_fee_up_to: None,
        source: Source::Vanguard,
    },
    Example {
        name: "iShares Core S&P 500 ETF",
        find_by: FindBy::Ticker("IVV"),
        product: Product::UsFund,
        venue: Venue::NyseArca,
        fee: Some(Percent(dec!(0.03))),
        variable_fee_up_to: None,
        source: Source::IsharesUs,
    },
    Example {
        name: "SPDR S&P 500 ETF Trust",
        find_by: FindBy::Ticker("SPY"),
        product: Product::UsFund,
        venue: Venue::NyseArca,
        fee: Some(Percent(dec!(0.0945))),
        variable_fee_up_to: None,
        source: Source::Ssga,
    },
    Example {
        name: "iShares Core S&P 500 UCITS ETF",
        find_by: FindBy::Ticker("CSPX"),
        product: Product::IrishFund,
        venue: Venue::EuronextAmsterdam,
        fee: Some(Percent(dec!(0.07))),
        variable_fee_up_to: None,
        source: Source::IsharesEurope,
    },
];

/// Largest first.
const MONEY_MARKET_FUNDS: &[MoneyMarketFund] = &[
    MoneyMarketFund {
        name: "אש כספית ניהול נזילות",
        number: SecurityNumber(5140918),
        fee: Percent(dec!(0.07)),
    },
    MoneyMarketFund {
        name: "הראל כספית מגמת ריבית",
        number: SecurityNumber(5127790),
        fee: Percent(dec!(0.19)),
    },
    MoneyMarketFund {
        name: "מיטב כספית שקלית כשרה",
        number: SecurityNumber(5136544),
        fee: Percent(dec!(0.17)),
    },
    MoneyMarketFund {
        name: "י.ל.כספית ניהול נזילות",
        number: SecurityNumber(5139258),
        fee: Percent(dec!(0.195)),
    },
    MoneyMarketFund {
        name: "קסם אקטיב כספית שקלית",
        number: SecurityNumber(5137740),
        fee: Percent(dec!(0.2)),
    },
    MoneyMarketFund {
        name: "איילון (00) כספית",
        number: SecurityNumber(5117700),
        fee: Percent(dec!(0.185)),
    },
    MoneyMarketFund {
        name: "י.ל. כספית כשרה",
        number: SecurityNumber(5141452),
        fee: Percent(dec!(0.065)),
    },
    MoneyMarketFund {
        name: "אנליסט (00) כספית שקלית",
        number: SecurityNumber(5120852),
        fee: Percent(dec!(0.179)),
    },
    MoneyMarketFund {
        name: "איביאי כספית ניהול נזילות",
        number: SecurityNumber(5137732),
        fee: Percent(dec!(0.179)),
    },
    MoneyMarketFund {
        name: "קסם אקטיב כספית פטורה",
        number: SecurityNumber(5127881),
        fee: Percent(dec!(0.25)),
    },
    MoneyMarketFund {
        name: "מגדל כספית שקלית",
        number: SecurityNumber(5102991),
        fee: Percent(dec!(0.249)),
    },
    MoneyMarketFund {
        name: "מגדל כספית שקלית כשרה",
        number: SecurityNumber(5138094),
        fee: Percent(dec!(0.154)),
    },
    MoneyMarketFund {
        name: "דולפין כספית",
        number: SecurityNumber(5138698),
        fee: Percent(dec!(0.159)),
    },
    MoneyMarketFund {
        name: "אנליסט (00) כספית",
        number: SecurityNumber(5121140),
        fee: Percent(dec!(0.225)),
    },
    MoneyMarketFund {
        name: "י.ל כספית",
        number: SecurityNumber(5137641),
        fee: Percent(dec!(0.23)),
    },
    MoneyMarketFund {
        name: "מגדל כספית",
        number: SecurityNumber(5134309),
        fee: Percent(dec!(0.249)),
    },
    MoneyMarketFund {
        name: "קסם אקטיב כספית כשרה",
        number: SecurityNumber(5142377),
        fee: Percent(dec!(0.035)),
    },
    MoneyMarketFund {
        name: "מיטב כספית",
        number: SecurityNumber(5123898),
        fee: Percent(dec!(0.26)),
    },
    MoneyMarketFund {
        name: "מגדל כספית שקלי ללא קונצ",
        number: SecurityNumber(5140785),
        fee: Percent(dec!(0.07)),
    },
    MoneyMarketFund {
        name: "מיטב כספית ניהול נזילות",
        number: SecurityNumber(5135926),
        fee: Percent(dec!(0.23)),
    },
    MoneyMarketFund {
        name: "FOREST כספית",
        number: SecurityNumber(5141353),
        fee: Percent(dec!(0.055)),
    },
    MoneyMarketFund {
        name: "הראל כספית ניהול נזילות",
        number: SecurityNumber(5129408),
        fee: Percent(dec!(0.18)),
    },
    MoneyMarketFund {
        name: "מור כספית ניהול נזילות",
        number: SecurityNumber(5139324),
        fee: Percent(dec!(0.045)),
    },
    MoneyMarketFund {
        name: "הראל כספית שקלית",
        number: SecurityNumber(5119409),
        fee: Percent(dec!(0.2)),
    },
    MoneyMarketFund {
        name: "מור כספית",
        number: SecurityNumber(5119813),
        fee: Percent(dec!(0.245)),
    },
    MoneyMarketFund {
        name: "איביאי כספית שקלית כשרה",
        number: SecurityNumber(5139522),
        fee: Percent(dec!(0.154)),
    },
    MoneyMarketFund {
        name: "איילון כספית ניהול נזילות",
        number: SecurityNumber(5136866),
        fee: Percent(dec!(0.185)),
    },
    MoneyMarketFund {
        name: "מיטב כספית שקלית ללא קונ",
        number: SecurityNumber(5137690),
        fee: Percent(dec!(0.15)),
    },
    MoneyMarketFund {
        name: "אלטשולר שחם כספית",
        number: SecurityNumber(5138763),
        fee: Percent(dec!(0.244)),
    },
    MoneyMarketFund {
        name: "הראל כספית שקלית כשרה",
        number: SecurityNumber(5137815),
        fee: Percent(dec!(0.15)),
    },
    MoneyMarketFund {
        name: "איביאי כספית חיסכון",
        number: SecurityNumber(5141197),
        fee: Percent(dec!(0.075)),
    },
    MoneyMarketFund {
        name: "איביאי כספית שקלית",
        number: SecurityNumber(5103510),
        fee: Percent(dec!(0.21)),
    },
    MoneyMarketFund {
        name: "מיטב כספית ג'מבו",
        number: SecurityNumber(5141296),
        fee: Percent(dec!(0.125)),
    },
    MoneyMarketFund {
        name: "אנליסט כספית חיסכון",
        number: SecurityNumber(5140413),
        fee: Percent(dec!(0.125)),
    },
    MoneyMarketFund {
        name: "קסם אקטיב כספית",
        number: SecurityNumber(5136874),
        fee: Percent(dec!(0.25)),
    },
    MoneyMarketFund {
        name: "איילון כספית כשרה",
        number: SecurityNumber(5142252),
        fee: Percent(dec!(0.055)),
    },
    MoneyMarketFund {
        name: "אש כספית ללא קונצרני",
        number: SecurityNumber(5105820),
        fee: Percent(dec!(0.22)),
    },
    MoneyMarketFund {
        name: "דולפין כספית שקלית",
        number: SecurityNumber(5141098),
        fee: Percent(dec!(0.01)),
    },
    MoneyMarketFund {
        name: "אנליסט כספית שקלית כשרה",
        number: SecurityNumber(5142906),
        fee: Percent(dec!(0.045)),
    },
    MoneyMarketFund {
        name: "אזימוט כספית",
        number: SecurityNumber(5138706),
        fee: Percent(dec!(0.15)),
    },
    MoneyMarketFund {
        name: "ברק כספית",
        number: SecurityNumber(5141692),
        fee: Percent(dec!(0.01)),
    },
    MoneyMarketFund {
        name: "מגדל כספית מחלקת חודשי",
        number: SecurityNumber(5137559),
        fee: Percent(dec!(0.17)),
    },
    MoneyMarketFund {
        name: "מגדל כספית חיסכון",
        number: SecurityNumber(5141304),
        fee: Percent(dec!(0.045)),
    },
    MoneyMarketFund {
        name: "פורטה כספית",
        number: SecurityNumber(5143128),
        fee: Percent(dec!(0.18)),
    },
];
// END examples.py
