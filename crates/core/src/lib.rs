//! A model of Israeli brokers' fees, for comparing what a given investing
//! pattern costs at each broker.
//!
//! # Shape of the data
//!
//! ```text
//! Broker                      e.g. "Bank Leumi"
//! └── Plan                    one price list a customer can be on, e.g. "Online", "Pepper"
//!     ├── trading:    Vec<TradeFee>     fee per buy or sell (first matching row wins)
//!     ├── custody:    Vec<CustodyFee>   fee for holding securities (first matching row wins)
//!     └── conversion: ConversionFee     fee for changing ₪ into foreign currency and back
//! ```
//!
//! The three fee types are separate because each is charged on something
//! different: a trade fee on one trade, custody on everything you hold over
//! time, conversion on an amount of money. Each type has only the fields that
//! make sense for it, so for example a conversion can't be priced per share.
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
pub mod money;
mod percent;
pub mod simulation;
pub mod tariffs;
pub mod yours;

pub use money::{Currency, ExchangeRates, Money, ils, iso, usd};
pub use percent::Percent;
// For `Security::iter()` and `Exchange::iter()`.
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
pub use strum::IntoEnumIterator;
use time::Date;

/// What kind of security is traded. Brokers price these differently, even on
/// the same exchange: Altshuler charges an ETF on the Tel Aviv exchange a
/// ₪3.5 minimum, but a mutual fund ₪16.
///
/// In the order to offer them: `Security::iter()`. Its `Display` is its name:
/// "ETF", "Mutual fund".
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter, strum::Display,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Security {
    /// Exchange-traded fund, e.g. an S&P 500 tracker. In Israel, a Keren Sal.
    #[strum(to_string = "ETF")]
    Etf,
    /// A fund bought from the fund manager at the day's price rather than
    /// traded continuously. In Israel, a Keren Ne'emanut.
    #[strum(to_string = "Mutual fund")]
    MutualFund,
    Bond,
    Stock,
}

/// Where the security is traded. Decides both the trade fee row and the
/// custody row: Leumi charges 0.15% a quarter to hold Tel Aviv securities but
/// 0.2% for foreign ones.
///
/// In the order to offer them: `Exchange::iter()`. Its `Display` is its name:
/// "Tel Aviv", "USA".
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter, strum::Display,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Exchange {
    /// Tel Aviv Stock Exchange.
    #[strum(to_string = "Tel Aviv")]
    Tlv,
    /// Any US exchange (NYSE, Nasdaq).
    #[strum(to_string = "USA")]
    Usa,
    /// Any European exchange.
    Europe,
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
        }
    }
}

// ─────────────────────────── Custody fees ───────────────────────────

/// One row of a plan's custody table: the fee for holding securities, as a
/// percentage of their value (Dmei Nihul Pikadon / Dmei Mishmeret).
///
/// Like trade fees, the first row that covers a holding is used. Holdings no
/// row covers are free.
///
/// Enter the rate exactly as the tariff states it, with the period it's
/// quoted per: Leumi's "0.15% a quarter, charged quarterly" is `percent`
/// 0.15, `per` and `billed` [`Period::Quarter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyFee {
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
        empty_or_contains(&self.exchanges, h.exchange)
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

/// What it costs to convert shekels to foreign currency or back (Amlat
/// Hamara / Chalifin). Paid every time you buy a foreign security with shekels.
///
/// There are two costs, and a plan can have either or both: an explicit fee
/// (`percent`, `min`, `max`) and a worse-than-market exchange rate
/// (`markup`). Leumi online charges 0.16% and doesn't publish its markup;
/// Altshuler charges no fee but a markup of up to 0.7%.
/// The default is converting for free.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConversionFee {
    /// The fee listed in the tariff, as a percentage of the converted amount.
    pub percent: Percent,
    /// The fee is never less than this.
    pub min: Option<Money>,
    /// The fee is never more than this.
    pub max: Option<Money>,
    /// How much worse than the market's the broker's exchange rate is. It
    /// isn't listed as a fee, but it's often the bigger cost. Not bounded by
    /// `min` or `max`.
    pub markup: Markup,
}

/// The Currency Conversion Markup (Hebrew: מרווח המרה): how much worse than
/// the market's a broker's exchange rate is, as a percentage of that rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Markup {
    /// "Up to 0.7%", as tariffs state it. The full amount is assumed.
    UpTo(Percent),
    /// The tariff doesn't say. Counted as nothing, which understates the cost.
    NotPublished,
}

impl Markup {
    /// The markup assumed in calculations.
    #[must_use]
    pub fn assumed(self) -> Percent {
        match self {
            Markup::UpTo(percent) => percent,
            Markup::NotPublished => Percent(Decimal::ZERO),
        }
    }
}

impl Default for Markup {
    fn default() -> Self {
        Markup::UpTo(Percent(Decimal::ZERO))
    }
}

impl ConversionFee {
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
        let fee = Money::from_decimal(self.percent.of(*amount.amount()), amount.currency());
        let fee = clamp(fee, self.min, self.max, rates);
        let markup = self.markup.assumed().of(*amount.amount());
        Money::from_decimal(fee.amount() + markup, amount.currency())
    }
}

// ─────────────────────────── Brokers and plans ───────────────────────────

/// One complete price list a customer can be on.
///
/// A broker often has several. Altshuler lets you choose among three US
/// trade fee options, so it has three plans. Leumi has different prices for
/// customer groups (18+, students) and for its Pepper app, each of which is a
/// plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    /// Shown in comparisons, e.g. "Online" or "US $11 flat".
    pub name: String,
    /// What the plan is, in plain words, for someone who hasn't read the
    /// tariff: who it's for and how it differs from the broker's other plans.
    #[serde(default)]
    pub description: String,
    /// The trade fee table. The first row that covers a trade is used.
    pub trading: Vec<TradeFee>,
    /// The custody table. The first row that covers a holding is used.
    pub custody: Vec<CustodyFee>,
    /// The cost of converting between shekels and foreign currency.
    pub conversion: ConversionFee,
    /// The least the account can be opened with (in ₪), if the plan has a
    /// minimum.
    #[serde(default)]
    pub min_first_deposit: Option<Money>,
    /// Assumptions and gaps in the tariff that affect this plan's numbers.
    /// Ones that apply to every plan go in [`Broker::caveats`].
    #[serde(default)]
    pub caveats: Vec<Caveat>,
}

/// Something a tariff leaves unclear or that affects the numbers, and how it
/// was read: "₪4 is only stated for orders up to ₪30,000". Shown only to users
/// buying what it's about, so it says which securities and exchanges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caveat {
    pub text: String,
    /// Empty means every security.
    #[serde(default)]
    pub securities: Vec<Security>,
    /// Empty means every exchange.
    #[serde(default)]
    pub exchanges: Vec<Exchange>,
}

impl Caveat {
    /// A caveat about everything; narrow it with [`Caveat::on`] and
    /// [`Caveat::about`].
    #[must_use]
    pub fn new(text: &str) -> Self {
        Caveat {
            text: text.to_owned(),
            securities: vec![],
            exchanges: vec![],
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

    /// True if it matters to someone buying `security` on `exchange`.
    #[must_use]
    pub fn applies_to(&self, security: Security, exchange: Exchange) -> bool {
        empty_or_contains(&self.securities, security)
            && empty_or_contains(&self.exchanges, exchange)
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

    /// The custody row used for holdings on `exchange`, if any.
    #[must_use]
    pub fn custody_row(&self, exchange: Exchange) -> Option<&CustodyFee> {
        self.custody
            .iter()
            .find(|row| empty_or_contains(&row.exchanges, exchange))
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

/// A broker or bank, and every plan it offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Broker {
    /// E.g. "Bank Leumi".
    pub name: String,
    /// What kind of broker it is and how its plans relate, in plain words.
    #[serde(default)]
    pub description: String,
    /// The date on the tariff document the numbers came from, if it has one.
    /// Tariffs change several times a year, so keep it next to the numbers.
    pub tariff_date: Option<Date>,
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
/// only on where it's traded and what it's worth.
#[derive(Debug, Clone, Copy)]
pub struct Holding {
    pub exchange: Exchange,
    /// Current value, in any currency.
    pub value: Money,
}

// ─────────────────────────── Helpers ───────────────────────────

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
        let leumi = ConversionFee {
            percent: Percent(dec!(0.16)),
            min: Some(usd(dec!(5.76))),
            max: Some(usd(dec!(2400))),
            markup: Markup::NotPublished,
        };
        // Altshuler: no fee, but a markup of up to 0.7%.
        let altshuler = ConversionFee {
            percent: Percent(dec!(0)),
            min: None,
            max: None,
            markup: Markup::UpTo(Percent(dec!(0.7))),
        };
        assert_eq!(leumi.cost(usd(dec!(1000)), &rates()), usd(dec!(5.76)));
        assert_eq!(leumi.cost(usd(dec!(10000)), &rates()), usd(dec!(16)));
        assert_eq!(altshuler.cost(usd(dec!(1000)), &rates()), usd(dec!(7)));
    }
}
