//! A model of Israeli brokers' fees, for comparing what a given investing
//! pattern costs at each broker.
//!
//! # Shape of the data
//!
//! ```text
//! Broker                          e.g. "Bank Leumi"
//! └── Plan                        one price list a customer can be on, e.g. "Online", "Pepper"
//!     ├── vehicle:         Vehicle          a brokerage account, a provident fund, a savings policy
//!     ├── trading:         Vec<TradeFee>    fee per buy or sell (first matching row wins)
//!     ├── standing_orders: Vec<TradeFee>    cheaper fees for buying by standing order, if any
//!     ├── custody:         Vec<CustodyFee>  fee for holding securities (first matching row wins)
//!     ├── conversion:      ConversionFee    fee for changing ₪ into foreign currency and back
//!     └── management:      ManagementFee    a manager's share of deposits and of the balance, if any
//! ```
//!
//! The fee types are separate because each is charged on something
//! different: a trade fee on one trade, custody on everything you hold over
//! time, conversion on an amount of money. Each type has only the fields that
//! make sense for it, so for example a conversion can't be priced per share.
//!
//! Fees belong to the plan; the tax on the gain and the limits on deposits
//! belong to its [`Vehicle`], whoever runs the money ([`vehicles`]).
//!
//! # Usage
//!
//! Pick a [`Plan`] and ask it one of three questions:
//! [`Plan::trade_fee`], [`Plan::custody_per_year`], [`Plan::conversion_fee`].
//! [`simulation`] runs these over the years for a whole investing plan.
//!
//! # Exchange rates
//!
//! Every method takes [`ExchangeRates`]. They're used only where two
//! currencies meet: a dollar minimum applied to a euro trade, or shekel and
//! dollar holdings added up for custody.
//!
//! # Deliberately left out
//!
//! Buy vs sell differences, tiered prices, third-party pass-through fees,
//! dividend fees, per-security and per-account caps, T-bills, OTC stocks,
//! options and futures. None of them changes a comparison much for a
//! long-term ETF investor.

pub mod describe;
pub mod examples;
pub mod funds;
pub mod money;
mod percent;
pub mod simulation;
pub mod tariffs;
pub mod vehicles;
pub mod yours;

pub use describe::FeeKind;
pub use money::{Currency, ExchangeRates, Money, ils, iso, usd};
pub use percent::Percent;
pub use vehicles::{Vehicle, Withdrawal};
// For `Security::iter()` and `Exchange::iter()`.
use std::borrow::Cow;
use std::ops::Index;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
pub use strum::IntoEnumIterator;
use time::Date;

// ─────────────────────────── Languages ───────────────────────────

/// The language the app is shown in. Every text a user sees comes in both,
/// as a [`Text`] or through [`Lang::pick`], so nothing falls back to English
/// unnoticed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Lang {
    #[default]
    En,
    He,
}

impl Lang {
    /// `en` or `he`, whichever this is: for a fixed text, where building a
    /// [`Text`] every time would be a waste.
    #[must_use]
    pub const fn pick<'a>(self, en: &'a str, he: &'a str) -> &'a str {
        match self {
            Lang::En => en,
            Lang::He => he,
        }
    }
}

/// A text in both languages. Both are required, so a missing translation is
/// a compile error rather than an English fallback. Read one with the
/// language as an index: `text[lang]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Text {
    pub en: Cow<'static, str>,
    pub he: Cow<'static, str>,
}

impl Text {
    #[must_use]
    pub const fn new(en: &'static str, he: &'static str) -> Self {
        Text {
            en: Cow::Borrowed(en),
            he: Cow::Borrowed(he),
        }
    }

    /// A text built at run time, `format!`ed in each language.
    #[must_use]
    pub fn owned(en: String, he: String) -> Self {
        Text {
            en: Cow::Owned(en),
            he: Cow::Owned(he),
        }
    }

    /// The same in both languages: a name the user typed, a number.
    #[must_use]
    pub fn same(text: &str) -> Self {
        Text::owned(text.to_owned(), text.to_owned())
    }

    /// Empty in both languages.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.en.is_empty() && self.he.is_empty()
    }
}

impl Index<Lang> for Text {
    type Output = str;

    fn index(&self, lang: Lang) -> &str {
        match lang {
            Lang::En => &self.en,
            Lang::He => &self.he,
        }
    }
}

/// Something with a name in each language: "ETF", "קרן\u{a0}סל".
pub trait Named {
    fn name(self, lang: Lang) -> &'static str;
}

/// What kind of security is traded. Brokers price these differently, even on
/// the same exchange: Altshuler charges an ETF on the Tel Aviv exchange a
/// ₪3.5 minimum, but an index fund ₪16.
///
/// In the order to offer them: `Security::iter()`. Its name in each language
/// is [`Named::name`]: "ETF", "Index fund".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Security {
    /// Exchange-traded fund, e.g. an S&P 500 tracker. In Israel, a Keren Sal.
    Etf,
    /// A mutual fund that tracks an index, bought from the fund manager at
    /// the day's price rather than traded continuously. In Israel, a Keren
    /// Mechaka. Every tariff prices it apart from managed (active) funds,
    /// which mostly cost nothing to trade and aren't compared. It was
    /// "Mutual fund" until September 2026, and saved plans from then say so.
    #[serde(alias = "MutualFund")]
    IndexFund,
    Bond,
    Stock,
}

impl Named for Security {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Security::Etf => lang.pick("ETF", "קרן סל"),
            Security::IndexFund => lang.pick("Index fund", "קרן מחקה"),
            Security::Bond => lang.pick("Bond", "אג\u{5f4}ח"),
            Security::Stock => lang.pick("Stock", "מניה"),
        }
    }
}

/// Where the security is traded. Decides both the trade fee row and the
/// custody row: Leumi charges 0.15% a quarter to hold Tel Aviv securities but
/// 0.2% for foreign ones.
///
/// In the order to offer them: `Exchange::iter()`. Its name in each language
/// is [`Named::name`]: "Tel Aviv", "USA".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Exchange {
    /// Tel Aviv Stock Exchange.
    Tlv,
    /// Any US exchange (NYSE, Nasdaq).
    Usa,
    /// Any European exchange.
    Europe,
}

impl Named for Exchange {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Exchange::Tlv => lang.pick("Tel Aviv", "תל אביב"),
            Exchange::Usa => lang.pick("USA", "ארה\u{5f4}ב"),
            Exchange::Europe => lang.pick("Europe", "אירופה"),
        }
    }
}

impl Exchange {
    /// The currency securities on this exchange trade in.
    #[must_use]
    pub fn currency(self) -> &'static Currency {
        match self {
            Exchange::Tlv => iso::ILS,
            Exchange::Usa => iso::USD,
            Exchange::Europe => iso::EUR,
        }
    }
}

/// In the order to offer them: `Period::iter()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Period {
    Month,
    Quarter,
    Year,
}

impl Period {
    #[must_use]
    pub fn times_per_year(self) -> u32 {
        match self {
            Period::Month => 12,
            Period::Quarter => 4,
            Period::Year => 1,
        }
    }
}

// ─────────────────────────── Trade fees ───────────────────────────

/// One row of a plan's trade fee table: what a single buy or sell costs, for
/// the securities and exchanges the row covers. Hebrew tariffs call this
/// Amlat Kniya/Mechira.
///
/// A plan checks its rows in order and uses the first one that covers the
/// trade, so list specific rows (ETFs on Tel Aviv) before general ones
/// (anything on Tel Aviv).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeFee {
    /// The securities this row covers. Empty means every security.
    #[serde(default)]
    pub securities: Vec<Security>,
    /// The exchanges this row covers. Empty means every exchange.
    #[serde(default)]
    pub exchanges: Vec<Exchange>,
    /// What each trade covered by this row costs.
    pub price: Price,
}

impl TradeFee {
    /// True if this row applies to `t`.
    #[must_use]
    pub fn covers(&self, t: &Trade) -> bool {
        self.applies_to(t.security, t.exchange)
    }

    /// True if this row applies to `security` traded on `exchange`.
    #[must_use]
    pub fn applies_to(&self, security: Security, exchange: Exchange) -> bool {
        empty_or_contains(&self.securities, security)
            && empty_or_contains(&self.exchanges, exchange)
    }
}

/// The price of one trade, as a tariff states it.
///
/// Percentage and per-share prices can have a minimum and a maximum. A flat
/// price can't: on a fixed amount they would mean nothing, so
/// [`Price::Flat`] has no room for them.
///
/// The bounds can be in a different currency from the trade (Leumi's minimum
/// for foreign trades is in dollars); [`Price::apply`] converts them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Price {
    /// A percentage of the trade's value. Leumi online, Tel Aviv: 0.4%,
    /// minimum ₪26, maximum ₪6,300.
    Percent {
        percent: Percent,
        /// The fee is never less than this. Usually what dominates small
        /// trades: Leumi's $24 minimum is 4.4% of a $540 purchase.
        min: Option<Money>,
        /// The fee is never more than this. Only matters for very large trades.
        max: Option<Money>,
    },
    /// A fixed amount per share. Altshuler's US option: 1¢ a share, minimum $9.
    PerShare {
        /// Charged for every share traded.
        per_share: Money,
        /// The fee is never less than this.
        min: Option<Money>,
        /// The fee is never more than this.
        max: Option<Money>,
    },
    /// The same amount whatever the trade's size. Altshuler's US option: $11.
    Flat(Money),
    /// A percentage of the trade's value plus an amount per share. IBI's
    /// fourth US track: 0.15% plus 1¢ a share, minimum $6.
    PercentPlusPerShare {
        percent: Percent,
        per_share: Money,
        /// The fee is never less than this.
        min: Option<Money>,
        /// The fee is never more than this.
        max: Option<Money>,
    },
}

impl Price {
    /// The fee for trade `t`, in the trade's currency.
    #[must_use]
    pub fn apply(&self, t: &Trade, rates: &ExchangeRates) -> Money {
        let currency = t.value.currency();
        match self {
            Price::Percent { percent, min, max } => {
                let fee = percent.of(*t.value.amount());
                clamp(Money::from_decimal(fee, currency), *min, *max, rates)
            }
            Price::PerShare {
                per_share,
                min,
                max,
            } => {
                let fee = Money::from_decimal(per_share.amount() * t.shares, per_share.currency());
                clamp(rates.convert(fee, currency), *min, *max, rates)
            }
            Price::Flat(amount) => rates.convert(*amount, currency),
            Price::PercentPlusPerShare {
                percent,
                per_share,
                min,
                max,
            } => {
                let per_shares =
                    Money::from_decimal(per_share.amount() * t.shares, per_share.currency());
                let fee =
                    percent.of(*t.value.amount()) + rates.convert(per_shares, currency).amount();
                clamp(Money::from_decimal(fee, currency), *min, *max, rates)
            }
        }
    }
}

// ─────────────────────────── Custody fees ───────────────────────────

/// One row of a plan's custody table: the fee for holding securities, as a
/// percentage of their value (Dmei Nihul Pikadon / Dmei Mishmeret).
///
/// Like trade fees, the first row that covers a holding is used. Holdings no
/// row covers are free: IBI charges nothing for holding funds, so its plans
/// put a free row for them before the one for everything else.
///
/// Enter the rate exactly as the tariff states it, with the period it's
/// quoted per: Leumi's "0.15% a quarter, charged quarterly" is `percent`
/// 0.15, `per` and `billed` [`Period::Quarter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyFee {
    /// The securities this row covers. Empty means every security.
    #[serde(default)]
    pub securities: Vec<Security>,
    /// The exchanges this row covers. Empty means every exchange.
    #[serde(default)]
    pub exchanges: Vec<Exchange>,
    pub percent: Percent,
    /// The period `percent` is quoted per. Leumi's 0.15% is per quarter.
    pub per: Period,
    /// How often the fee is charged. This can differ from `per`: Altshuler
    /// quotes a yearly rate but charges monthly. It matters because `min`
    /// applies to each charge.
    pub billed: Period,
    /// The least charged each billing period, for everything this row covers
    /// together. Altshuler's ₪75 a month means a ₪100,000 portfolio pays 0.9%
    /// a year instead of 0.15%.
    pub min: Option<Money>,
}

impl CustodyFee {
    /// True if this row applies to `h`.
    #[must_use]
    pub fn covers(&self, h: &Holding) -> bool {
        self.applies_to(h.security, h.exchange)
    }

    /// True if this row applies to holding `security` on `exchange`.
    #[must_use]
    pub fn applies_to(&self, security: Security, exchange: Exchange) -> bool {
        empty_or_contains(&self.securities, security)
            && empty_or_contains(&self.exchanges, exchange)
    }

    /// True if it never charges anything: IBI's row for Tel Aviv index funds.
    #[must_use]
    pub fn is_free(&self) -> bool {
        self.percent.is_zero() && self.min.is_none_or(|min| min.is_zero())
    }

    /// The rate converted to a yearly percentage: 0.15% a quarter is 0.6%.
    /// Handy for displaying plans side by side.
    #[must_use]
    pub fn percent_per_year(&self) -> Percent {
        Percent(self.percent.0 * Decimal::from(self.per.times_per_year()))
    }

    /// A year of this fee in ₪, on holdings worth `value` in total (all of
    /// them covered by this row). Nothing held, nothing charged.
    #[must_use]
    pub fn cost_per_year(&self, value: Money, rates: &ExchangeRates) -> Money {
        let value = *rates.convert(value, iso::ILS).amount();
        if value.is_zero() {
            return ils(Decimal::ZERO);
        }
        let charges = Decimal::from(self.billed.times_per_year());
        let mut per_charge = self.percent_per_year().of(value) / charges;
        if let Some(min) = self.min {
            per_charge = per_charge.max(*rates.convert(min, iso::ILS).amount());
        }
        ils(per_charge * charges)
    }
}

// ─────────────────────────── Conversion fees ───────────────────────────

/// A fee that's a percentage of an amount, within a minimum and a maximum:
/// "0.16%, min $5.76, max $2,400". The bounds can be in a different currency
/// from the amount; [`PercentFee::of`] converts them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PercentFee {
    pub percent: Percent,
    /// The fee is never less than this.
    pub min: Option<Money>,
    /// The fee is never more than this.
    pub max: Option<Money>,
}

impl PercentFee {
    /// Never charges anything.
    pub const FREE: PercentFee = PercentFee {
        percent: Percent(Decimal::ZERO),
        min: None,
        max: None,
    };

    /// The fee on `amount`, in `amount`'s currency.
    #[must_use]
    pub fn of(&self, amount: Money, rates: &ExchangeRates) -> Money {
        let fee = Money::from_decimal(self.percent.of(*amount.amount()), amount.currency());
        clamp(fee, self.min, self.max, rates)
    }

    /// True if it never charges anything.
    #[must_use]
    pub fn is_free(&self) -> bool {
        self.percent.is_zero() && self.min.is_none_or(|min| min.is_zero())
    }
}

/// What it costs to convert shekels to foreign currency or back (Amlat
/// Hamara / Chalifin). Paid every time you buy a foreign security with shekels.
///
/// There are two costs, and a plan can have either or both: an explicit fee
/// and a worse-than-market exchange rate (`markup`). Leumi online charges
/// 0.16% and doesn't publish its markup; Altshuler charges no fee but a
/// markup of up to 0.7%.
///
/// No `Default`, on purpose: a default would mean "free", and a plan that
/// forgot to say what converting costs would then look free, the same as one
/// whose tariff waives it. Say [`ConversionFee::FREE`] where that's meant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionFee {
    /// The fee listed in the tariff, as a percentage of the converted amount.
    #[serde(flatten)]
    pub fee: PercentFee,
    /// A second fee the customer also gets; each conversion pays whichever
    /// is less. Leumi's 18+ group pays half the branch fee but at least its
    /// $7.20 minimum, while its online fee is lower on small amounts, and the
    /// tariff gives each fee the better benefit, never both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub or_if_less: Option<PercentFee>,
    /// How much worse than the market's the broker's exchange rate is. It
    /// isn't listed as a fee, but it's often the bigger cost. Not bounded by
    /// the fee's minimum or maximum.
    pub markup: Markup,
}

/// The Currency Conversion Markup (Hebrew: מרווח המרה): how much worse than
/// the market's a broker's exchange rate is, as a percentage of that rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Markup {
    /// "Up to 0.7%", as tariffs state it. The full amount is assumed.
    UpTo(Percent),
    /// An amount in shekels for every dollar converted: Excellence's 2
    /// agorot. As a percentage it follows the exchange rate.
    PerDollar(Money),
    /// The broker converts at the currency market's live rate and charges
    /// only its fee, as Interactive Israel's site says (שער\u{a0}רציף):
    /// costs nothing beyond the market's own bid-ask spread, which isn't
    /// counted. Every plan with one carries a "published" caveat about it.
    MarketRate,
    /// The tariff doesn't say. Counted as nothing, which understates the cost:
    /// every plan with one carries a "may cost more" caveat about it.
    NotPublished,
}

impl Markup {
    /// Converts at the market rate. Only the no-fee baseline and the user's
    /// own plans say so: no listed broker publishes that it has no markup.
    pub const NONE: Markup = Markup::UpTo(Percent(Decimal::ZERO));

    /// What the markup costs on converting `amount`, in `amount`'s currency.
    #[must_use]
    pub fn cost(self, amount: Money, rates: &ExchangeRates) -> Decimal {
        match self {
            Markup::UpTo(percent) => percent.of(*amount.amount()),
            Markup::PerDollar(per_dollar) => {
                let dollars = *rates.convert(amount, iso::USD).amount();
                let cost =
                    Money::from_decimal(dollars * per_dollar.amount(), per_dollar.currency());
                *rates.convert(cost, amount.currency()).amount()
            }
            Markup::MarketRate | Markup::NotPublished => Decimal::ZERO,
        }
    }
}

impl ConversionFee {
    /// Converting costs nothing, at the market rate.
    pub const FREE: ConversionFee = ConversionFee {
        fee: PercentFee::FREE,
        or_if_less: None,
        markup: Markup::NONE,
    };

    /// What converting `amount` costs, fee plus markup, in `amount`'s
    /// currency. The same in either direction (₪ to $ or $ to ₪).
    ///
    /// The markup is approximated as costing its percentage of the amount.
    /// Exactly, a rate m% worse costs m% of the amount when you sell foreign
    /// currency, but m/(1+m) when you buy it: ₪13.90 rather than ₪14 on
    /// ₪2,000 at 0.7%. That gap is far smaller than the uncertainty in the
    /// published markup itself.
    #[must_use]
    pub fn cost(&self, amount: Money, rates: &ExchangeRates) -> Money {
        let mut fee = *self.fee.of(amount, rates).amount();
        if let Some(other) = self.or_if_less {
            fee = fee.min(*other.of(amount, rates).amount());
        }
        let markup = self.markup.cost(amount, rates);
        Money::from_decimal(fee + markup, amount.currency())
    }
}

// ─────────────────────────── Brokers and plans ───────────────────────────

/// One complete price list a customer can be on.
///
/// A broker often has several: Leumi has different prices for customer
/// groups (18+, students) and for its Pepper app, and investment houses
/// have their full tariff and the offer new customers get.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    /// Shown in comparisons, e.g. "Online" or "Full tariff".
    pub name: Text,
    /// What the plan is, in plain words, for someone who hasn't read the
    /// tariff: who it's for, on what terms, and how it differs from the
    /// broker's other plans. Not its prices: the plan's details show those
    /// under it, only the ones that apply to what the user buys.
    pub description: Text,
    /// What kind of account it is, which decides the tax on the gain and
    /// how much may be deposited.
    #[serde(default = "Vehicle::of_older_saves")]
    pub vehicle: Vehicle,
    /// The trade fee table. The first row that covers a trade is used.
    pub trading: Vec<TradeFee>,
    /// Price options chosen when opening the account, for some trades (see
    /// [`Track`]). Empty if there's no choice.
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// Cheaper fees for buying by standing order (Hora'at Keva): the same
    /// amount bought automatically every month. A table like `trading`, but
    /// only for those purchases: a one-time deposit and selling pay
    /// `trading`, and so does buying less often than every month.
    #[serde(default)]
    pub standing_orders: Vec<TradeFee>,
    /// What converting costs for purchases by standing order, if it differs
    /// from `conversion`: Interactive Israel's automatic plan converts for
    /// free.
    #[serde(default)]
    pub standing_order_conversion: Option<ConversionFee>,
    /// The custody table. The first row that covers a holding is used.
    pub custody: Vec<CustodyFee>,
    /// The cost of converting between shekels and foreign currency.
    pub conversion: ConversionFee,
    /// A monthly fee for keeping the account, if the plan has one.
    #[serde(default)]
    pub handling: Option<HandlingFee>,
    /// What a manager takes for running the money, if someone does: a
    /// provident fund's or a savings policy's only fee.
    #[serde(default)]
    pub management: Option<ManagementFee>,
    /// The exchanges where the broker sells fractions of a share. Empty
    /// means none: there, only whole shares are bought.
    #[serde(default)]
    pub fractions_on: Vec<Exchange>,
    /// The least the account can be opened with (in ₪), if the plan has a
    /// minimum.
    #[serde(default)]
    pub min_first_deposit: Option<Money>,
    /// Assumptions and gaps in the tariff that affect this plan's numbers.
    /// Ones that apply to every plan go in [`Broker::caveats`].
    #[serde(default)]
    pub caveats: Vec<Caveat>,
}

/// A price option chosen when opening the account, for some trades:
/// Altshuler offers three for US stocks and ETFs, a price per share, per
/// order or as a percentage. A plan with tracks costs what its cheapest
/// track does for the user's investing ([`simulation::compare`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// "1¢ a share"
    pub name: Text,
    /// Trade rows used before the plan's own, for what they cover.
    pub trading: Vec<TradeFee>,
}

/// A fee for keeping the account, charged every month (Dmei Tipul): "₪15 a
/// month, free for the first 2 years". Some brokers take the month's trade
/// fees off it, so an account that trades enough pays nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HandlingFee {
    /// In ₪.
    pub per_month: Money,
    /// How many months after opening the account it isn't charged.
    #[serde(default)]
    pub free_months: u32,
    /// True if the month's trade fees are taken off it.
    #[serde(default)]
    pub less_trade_fees: bool,
}

impl HandlingFee {
    /// What `month` (0 is the first) costs, in ₪, when its trades paid
    /// `trade_fees_ils` in fees.
    #[must_use]
    pub fn for_month(&self, month: u32, trade_fees_ils: Decimal) -> Decimal {
        if month < self.free_months {
            return Decimal::ZERO;
        }
        let taken_off = if self.less_trade_fees {
            trade_fees_ils
        } else {
            Decimal::ZERO
        };
        (*self.per_month.amount() - taken_off).max(Decimal::ZERO)
    }
}

/// What a manager takes for running the money (Dmei Nihul), the way
/// provident funds and savings policies charge: a share of each deposit on
/// its way in, and a share of what has built up, every year. Inside a fund
/// nothing else is paid: no trade fees and no custody.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ManagementFee {
    /// Taken from each deposit before it's invested.
    pub of_deposits: Percent,
    /// A year's charge on the balance, taken a twelfth every month.
    pub of_balance: Percent,
}

impl ManagementFee {
    /// What's taken from `deposit` on its way in.
    #[must_use]
    pub fn on_deposit(&self, deposit: Decimal) -> Decimal {
        self.of_deposits.of(deposit)
    }

    /// A month's charge on `balance`.
    #[must_use]
    pub fn for_month(&self, balance: Decimal) -> Decimal {
        self.of_balance.of(balance) / Decimal::from(12)
    }
}

/// How sure the app is of a number, and why: what a caveat says about it,
/// from surest to least. Every caveat has one, so a reader knows whether
/// they're looking at the tariff's word, a reading of it, a stand-in or a
/// known gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Basis {
    /// The tariff or the broker's site says so. Shown so the user knows,
    /// not because anything is uncertain.
    Published,
    /// The tariff is unclear or silent, and this is how it was read, with
    /// what supports the reading: "the exchange's June 2026 averages".
    Reading { support: Text },
    /// Nothing to go on: a stand-in value, and which way it errs.
    Assumed { errs: Errs },
    /// A real cost the model leaves out, and why.
    NotCounted,
}

/// Which way a stand-in value errs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Errs {
    /// On the expensive side: the plan can only be cheaper than shown.
    AtMost,
    /// On the cheap side: the plan may cost more than shown, so the
    /// comparison flags it, with a few words on what: "conversion markup
    /// not published".
    MayCostMore { summary: Text },
}

/// A page a number comes from, to link to: "IBI's currency FAQ".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Page {
    pub name: Text,
    pub url: String,
}

/// Something a tariff leaves unclear, or that affects the numbers, and how
/// it was read: "₪4 is only stated for orders up to ₪30,000". Made with the
/// constructor that names its [`Basis`], so none is without one. Shown only
/// to users it matters to: it says which securities and exchanges, and from
/// what order size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caveat {
    pub text: Text,
    pub basis: Basis,
    /// The pages it rests on, linked beside it. A reading always has one:
    /// the tariff itself, the broker's site or a comparison site.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Page>,
    /// The fee it's about, if one: marked beside that fee's price.
    #[serde(default)]
    pub fee: Option<FeeKind>,
    /// Empty means every security.
    #[serde(default)]
    pub securities: Vec<Security>,
    /// Empty means every exchange.
    #[serde(default)]
    pub exchanges: Vec<Exchange>,
    /// Matters only to orders, or conversions, above this much, in the
    /// trade's currency: Pepper's $4 is stated for orders up to $8,000.
    #[serde(default)]
    pub above: Option<Money>,
}

impl Caveat {
    fn with(text: Text, basis: Basis) -> Self {
        Caveat {
            text,
            basis,
            sources: vec![],
            fee: None,
            securities: vec![],
            exchanges: vec![],
            above: None,
        }
    }

    /// What the tariff or the broker's site says.
    #[must_use]
    pub fn published(text: Text) -> Self {
        Caveat::with(text, Basis::Published)
    }

    /// How an unclear or silent row was read, and `support`: what backs
    /// the reading.
    #[must_use]
    pub fn reading(text: Text, support: Text) -> Self {
        Caveat::with(text, Basis::Reading { support })
    }

    /// A stand-in on the expensive side: the full price or maximum.
    #[must_use]
    pub fn at_most(text: Text) -> Self {
        Caveat::with(text, Basis::Assumed { errs: Errs::AtMost })
    }

    /// A stand-in on the cheap side. `summary` flags it in the comparison:
    /// "conversion markup not published".
    #[must_use]
    pub fn may_cost_more(text: Text, summary: Text) -> Self {
        Caveat::with(
            text,
            Basis::Assumed {
                errs: Errs::MayCostMore { summary },
            },
        )
    }

    /// A real cost the model leaves out.
    #[must_use]
    pub fn not_counted(text: Text) -> Self {
        Caveat::with(text, Basis::NotCounted)
    }

    /// Resting on `source`, a page to link beside it.
    #[must_use]
    pub fn source(mut self, source: &Page) -> Self {
        self.sources.push(source.clone());
        self
    }

    /// Resting on each of `sources`.
    #[must_use]
    pub fn sources(mut self, sources: &[&Page]) -> Self {
        self.sources
            .extend(sources.iter().map(|&source| source.clone()));
        self
    }

    /// About one fee: marked beside its price.
    #[must_use]
    pub fn about_fee(self, fee: FeeKind) -> Self {
        Caveat {
            fee: Some(fee),
            ..self
        }
    }

    /// Only for orders, or conversions, above `amount` (in the trade's
    /// currency).
    #[must_use]
    pub fn when_above(self, amount: Money) -> Self {
        Caveat {
            above: Some(amount),
            ..self
        }
    }

    /// The summary the comparison flags, if it may cost more.
    #[must_use]
    pub fn may_cost_more_summary(&self) -> Option<&Text> {
        match &self.basis {
            Basis::Assumed {
                errs: Errs::MayCostMore { summary },
            } => Some(summary),
            _ => None,
        }
    }

    /// Only for securities traded on these exchanges.
    #[must_use]
    pub fn on(self, exchanges: &[Exchange]) -> Self {
        Caveat {
            exchanges: exchanges.to_vec(),
            ..self
        }
    }

    /// Only for these securities.
    #[must_use]
    pub fn about(self, securities: &[Security]) -> Self {
        Caveat {
            securities: securities.to_vec(),
            ..self
        }
    }

    /// True if it's about `security` on `exchange`, whatever the amounts.
    #[must_use]
    pub fn applies_to(&self, security: Security, exchange: Exchange) -> bool {
        empty_or_contains(&self.securities, security)
            && empty_or_contains(&self.exchanges, exchange)
    }

    /// True if it matters to `buying`: it's about the security and exchange,
    /// and the biggest order reaches its amount, if it has one. An unknown
    /// biggest order counts as reaching it.
    #[must_use]
    pub fn matters_for(&self, buying: Buying, rates: &ExchangeRates) -> bool {
        self.applies_to(buying.security, buying.exchange)
            && match (self.above, buying.largest_trade) {
                (Some(above), Some(largest)) => {
                    rates.convert(largest, above.currency()).amount() >= above.amount()
                }
                _ => true,
            }
    }
}

impl Plan {
    /// The trade fee row used for `security` traded on `exchange`, if any.
    #[must_use]
    pub fn trade_row(&self, security: Security, exchange: Exchange) -> Option<&TradeFee> {
        self.trading
            .iter()
            .find(|row| row.applies_to(security, exchange))
    }

    /// What a manager leaves of `deposit`: all of it where there's none.
    #[must_use]
    pub fn deposit_less_fee(&self, deposit: Decimal) -> Decimal {
        deposit
            - self
                .management
                .map_or(Decimal::ZERO, |fee| fee.on_deposit(deposit))
    }

    /// The plan as it is on track `index`: that track's trade rows before
    /// its own, and no tracks left to choose.
    #[must_use]
    pub fn on_track(&self, index: usize) -> Plan {
        let Some(track) = self.tracks.get(index) else {
            return self.clone();
        };
        Plan {
            trading: [track.trading.clone(), self.trading.clone()].concat(),
            tracks: vec![],
            ..self.clone()
        }
    }

    /// The track, if any, whose rows price `security` on `exchange`: the
    /// tracks only matter for what they cover.
    #[must_use]
    pub fn track_for(
        &self,
        index: usize,
        security: Security,
        exchange: Exchange,
    ) -> Option<&Track> {
        self.tracks.get(index).filter(|track| {
            track
                .trading
                .iter()
                .any(|row| row.applies_to(security, exchange))
        })
    }

    /// True if fractions of a share are sold on `exchange`.
    #[must_use]
    pub fn sells_fractions_on(&self, exchange: Exchange) -> bool {
        self.fractions_on.contains(&exchange)
    }

    /// The standing order row used for buying `security` on `exchange`, if
    /// the plan has a cheaper price for buying it by standing order.
    #[must_use]
    pub fn standing_order_row(&self, security: Security, exchange: Exchange) -> Option<&TradeFee> {
        self.standing_orders
            .iter()
            .find(|row| row.applies_to(security, exchange))
    }

    /// The custody row used for holding `security` on `exchange`, if any.
    #[must_use]
    pub fn custody_row(&self, security: Security, exchange: Exchange) -> Option<&CustodyFee> {
        self.custody
            .iter()
            .find(|row| row.applies_to(security, exchange))
    }

    /// The fee on one trade, in the trade's currency.
    ///
    /// Returns `None` if no row covers the trade, meaning the broker doesn't
    /// list a price for it (Altshuler has no row for European ETFs). That is
    /// different from a free trade, which is a row with a zero price.
    #[must_use]
    pub fn trade_fee(&self, t: &Trade, rates: &ExchangeRates) -> Option<Money> {
        self.trade_row(t.security, t.exchange)
            .map(|row| row.price.apply(t, rates))
    }

    /// A year of custody fees in ₪, assuming `holdings` stay the same all year.
    ///
    /// Each holding goes to the first custody row that covers it; each row is
    /// then charged on the total value of its holdings. That's why a row's
    /// minimum is paid once for the whole portfolio, not once per holding.
    #[must_use]
    pub fn custody_per_year(&self, holdings: &[Holding], rates: &ExchangeRates) -> Money {
        let mut value_per_row = vec![Decimal::ZERO; self.custody.len()];
        for h in holdings {
            if let Some(i) = self.custody.iter().position(|row| row.covers(h)) {
                value_per_row[i] += *rates.convert(h.value, iso::ILS).amount();
            }
        }
        let total = self
            .custody
            .iter()
            .zip(value_per_row)
            .map(|(row, value)| *row.cost_per_year(ils(value), rates).amount())
            .sum();
        ils(total)
    }

    /// What converting `amount` costs, fee plus markup, in `amount`'s currency.
    #[must_use]
    pub fn conversion_fee(&self, amount: Money, rates: &ExchangeRates) -> Money {
        self.conversion.cost(amount, rates)
    }
}

/// The date a tariff document gives itself: a day, or only a month, as
/// Meitav's "version 01/2025".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TariffDate {
    Day(Date),
    /// Only the month and year count.
    Month(Date),
}

/// A bank or an investment house. It decides what a new customer usually
/// gets: a bank's online prices, an investment house's joining offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum BrokerKind {
    /// Publishes what it charges, and charges less for trading online than
    /// at a branch.
    Bank,
    /// Publishes only a full tariff, the most it may charge, and offers new
    /// customers less.
    InvestmentHouse,
    /// Not one company but a kind of fund, as a whole: a provident fund for
    /// investment, a savings policy. Its fee is agreed person by person, so
    /// its plans are what savers pay: on average, at the cheapest and the
    /// dearest company, and the most that's allowed.
    Funds,
}

/// A broker or bank, and every plan it offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Broker {
    /// E.g. "Bank Leumi".
    pub name: Text,
    /// Its name beside a plan's, where plan names repeat: "Leumi" in "Leumi ·
    /// Online".
    pub short_name: Text,
    /// A bank or an investment house, which decides what "usual" means.
    pub kind: BrokerKind,
    /// Which of `plans` a new customer usually gets: the one compared at
    /// first.
    #[serde(default)]
    pub new_customer_plan: usize,
    /// Whether that plan is ticked when the app opens. Everyone compares
    /// the brokers; of the funds, only the kind most like a broker's
    /// account.
    #[serde(default = "yes")]
    pub compared_at_first: bool,
    /// What kind of broker it is and how its plans relate, in plain words.
    pub description: Text,
    /// The date on the tariff document the numbers came from, if it has one.
    /// Tariffs change several times a year, so keep it next to the numbers.
    pub tariff_date: Option<TariffDate>,
    /// Where the tariff document can be read online.
    #[serde(default)]
    pub source_url: Option<String>,
    /// Like [`Plan::caveats`], for ones that apply to every plan.
    #[serde(default)]
    pub caveats: Vec<Caveat>,
    /// Every plan a customer of this broker can be on.
    pub plans: Vec<Plan>,
}

// ─────────────────────────── Inputs ───────────────────────────

/// A single buy or sell, as input to [`Plan::trade_fee`].
#[derive(Debug, Clone, Copy)]
pub struct Trade {
    pub security: Security,
    pub exchange: Exchange,
    /// How many shares. Only per-share prices use it; fractions are fine.
    pub shares: Decimal,
    /// The total value of the trade, in the currency it trades in.
    pub value: Money,
}

/// Something you hold, as input to [`Plan::custody_per_year`]. Custody depends
/// only on what it is, where it's traded and what it's worth.
#[derive(Debug, Clone, Copy)]
pub struct Holding {
    pub security: Security,
    pub exchange: Exchange,
    /// Current value, in any currency.
    pub value: Money,
}

/// What the user buys, for picking the fees and caveats that matter to them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Buying {
    pub security: Security,
    pub exchange: Exchange,
    /// The biggest single order or conversion, in the exchange's currency,
    /// if known: some caveats matter only above an amount.
    pub largest_trade: Option<Money>,
}

impl Buying {
    /// `security` on `exchange`, whatever the amounts.
    #[must_use]
    pub fn any_amount(security: Security, exchange: Exchange) -> Self {
        Buying {
            security,
            exchange,
            largest_trade: None,
        }
    }
}

// ─────────────────────────── Helpers ───────────────────────────

/// Everything the app lists to compare, in the order to offer it: the
/// brokers, then the funds. Other code refers to them, and to their plans,
/// by position in this list.
#[must_use]
pub fn listed() -> Vec<Broker> {
    let mut listed = tariffs::all();
    listed.extend(funds::all());
    listed
}

/// What a broker saved before `compared_at_first` was: compared.
fn yes() -> bool {
    true
}

/// True if `list` contains `x`, or if `list` is empty (meaning "any").
fn empty_or_contains<T: PartialEq + Copy>(list: &[T], x: T) -> bool {
    list.is_empty() || list.contains(&x)
}

/// `fee` raised to `min` and lowered to `max`, converting the bounds to `fee`'s currency.
fn clamp(fee: Money, min: Option<Money>, max: Option<Money>, rates: &ExchangeRates) -> Money {
    let in_fee_currency = |bound| *rates.convert(bound, fee.currency()).amount();
    let mut amount = *fee.amount();
    if let Some(min) = min {
        amount = amount.max(in_fee_currency(min));
    }
    if let Some(max) = max {
        amount = amount.min(in_fee_currency(max));
    }
    Money::from_decimal(amount, fee.currency())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// €1 = $1.25 = ₪4.625, so $1 = ₪3.70. Not real rates.
    fn rates() -> ExchangeRates {
        ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
    }

    fn tase_stock(value: Decimal) -> Trade {
        Trade {
            security: Security::Stock,
            exchange: Exchange::Tlv,
            shares: dec!(1),
            value: ils(value),
        }
    }

    #[test]
    fn trade_fee_row_covers_and_prices() {
        // Leumi's online rate for anything on the Tel Aviv exchange.
        let row = TradeFee {
            securities: vec![], // empty: every security
            exchanges: vec![Exchange::Tlv],
            price: Price::Percent {
                percent: Percent(dec!(0.4)),
                min: Some(ils(dec!(26))),
                max: Some(ils(dec!(6300))),
            },
        };
        let trade = tase_stock(dec!(1000));
        assert!(row.covers(&trade));
        assert!(!row.covers(&Trade {
            exchange: Exchange::Usa,
            ..trade
        }));
        assert_eq!(
            row.price.apply(&tase_stock(dec!(1000)), &rates()),
            ils(dec!(26))
        );
    }

    #[test]
    fn percent_price_with_minimum() {
        // "0.15%, minimum ₪3.5"
        let price = Price::Percent {
            percent: Percent(dec!(0.15)),
            min: Some(ils(dec!(3.5))),
            max: None,
        };
        assert_eq!(
            price.apply(&tase_stock(dec!(1000)), &rates()),
            ils(dec!(3.5))
        );
        assert_eq!(
            price.apply(&tase_stock(dec!(10000)), &rates()),
            ils(dec!(15))
        );
    }

    #[test]
    fn custody_quoted_per_quarter() {
        // Leumi's "0.15% a quarter, charged quarterly".
        let leumi = CustodyFee {
            securities: vec![],
            exchanges: vec![Exchange::Tlv],
            percent: Percent(dec!(0.15)),
            per: Period::Quarter,
            billed: Period::Quarter,
            min: None,
        };
        assert_eq!(leumi.percent_per_year(), Percent(dec!(0.6)));
        assert_eq!(
            leumi.cost_per_year(ils(dec!(200000)), &rates()),
            ils(dec!(1200))
        );
    }

    #[test]
    fn custody_minimum_per_billing_period() {
        // Altshuler's "0.15% a year, charged monthly, at least ₪75 a month".
        let altshuler = CustodyFee {
            securities: vec![],
            exchanges: vec![], // empty: holdings on any exchange
            percent: Percent(dec!(0.15)),
            per: Period::Year,
            billed: Period::Month,
            min: Some(ils(dec!(75))),
        };
        assert_eq!(altshuler.percent_per_year(), Percent(dec!(0.15)));
        assert_eq!(
            altshuler.cost_per_year(ils(dec!(100000)), &rates()),
            ils(dec!(900))
        );
        assert_eq!(
            altshuler.cost_per_year(ils(dec!(1000000)), &rates()),
            ils(dec!(1500))
        );
        assert_eq!(
            altshuler.cost_per_year(ils(dec!(0)), &rates()),
            ils(dec!(0))
        );
    }

    #[test]
    fn conversion_fee_and_markup() {
        // Leumi online: 0.16%, minimum $5.76, maximum $2,400. Markup not published.
        let online = PercentFee {
            percent: Percent(dec!(0.16)),
            min: Some(usd(dec!(5.76))),
            max: Some(usd(dec!(2400))),
        };
        let leumi = ConversionFee {
            fee: online,
            or_if_less: None,
            markup: Markup::NotPublished,
        };
        // Altshuler: no fee, but a markup of up to 0.7%.
        let altshuler = ConversionFee {
            markup: Markup::UpTo(Percent(dec!(0.7))),
            ..ConversionFee::FREE
        };
        assert_eq!(leumi.cost(usd(dec!(1000)), &rates()), usd(dec!(5.76)));
        assert_eq!(leumi.cost(usd(dec!(10000)), &rates()), usd(dec!(16)));
        assert_eq!(altshuler.cost(usd(dec!(1000)), &rates()), usd(dec!(7)));

        // 0.1%, minimum $7.20, or the online fee if that's less.
        let either = ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(7.2))),
                max: None,
            },
            or_if_less: Some(online),
            markup: Markup::NotPublished,
        };
        assert_eq!(either.cost(usd(dec!(1000)), &rates()), usd(dec!(5.76)));
        assert_eq!(either.cost(usd(dec!(10000)), &rates()), usd(dec!(10)));
    }

    #[test]
    fn a_minimum_alone_is_not_free() {
        let fee = PercentFee {
            percent: Percent(dec!(0)),
            min: Some(usd(dec!(5))),
            max: None,
        };
        assert!(!fee.is_free());
        assert_eq!(fee.of(usd(dec!(100)), &rates()), usd(dec!(5)));
        assert!(PercentFee::default().is_free());
    }

    #[test]
    fn a_plan_on_a_track_has_that_tracks_rows_first_and_no_choice_left() {
        let altshuler = tariffs::altshuler().plans.remove(0);
        let on_track = altshuler.on_track(1);
        assert_eq!(on_track.tracks, []);
        assert_eq!(
            on_track.trading.len(),
            altshuler.tracks[1].trading.len() + altshuler.trading.len()
        );
        assert_eq!(on_track.trading[0], altshuler.tracks[1].trading[0]);
        // Past its tracks, the plan is left as it is.
        assert_eq!(altshuler.on_track(9), altshuler);
    }
}
