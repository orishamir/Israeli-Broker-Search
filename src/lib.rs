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
//! To draw a graph, call them in a loop over the user's inputs (see
//! `src/main.rs`).
//!
//! # Exchange rates
//!
//! Every method takes `money2`'s [`ExchangeRates`]. They're used only where
//! two currencies meet: a dollar minimum applied to a euro trade, or shekel
//! and dollar holdings added up for custody.
//!
//! [`ExchangeRates::new`] downloads the European Central Bank's daily
//! reference rates, which include the shekel, and caches them for the day.
//! It's async, so the app needs a runtime such as Tokio. The ECB rate differs
//! from the Bank of Israel's representative rate (Sha'ar Yatzig), which tariffs
//! refer to, by a fraction of a percent: a few agorot on these fees.
//!
//! `money2` rounds every conversion to two decimal places.
//!
//! # Deliberately left out
//!
//! Buy vs sell differences, tiered prices, third-party pass-through fees,
//! dividend fees, per-security and per-account caps, T-bills, OTC stocks,
//! options and futures. None of them changes a comparison much for a
//! long-term ETF investor.

pub mod simulation;
pub mod tariffs;

use money2::{Currency, Exchange as _, ExchangeRates, Money};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// What kind of security is traded. Brokers price these differently, even on
/// the same exchange: Altshuler charges an ETF on the Tel Aviv exchange a
/// ₪3.5 minimum, but a mutual fund ₪16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Security {
    /// Exchange-traded fund, e.g. an S&P 500 tracker. In Israel, a Keren Sal.
    Etf,
    Stock,
    Bond,
    /// A fund bought from the fund manager at the day's price rather than
    /// traded continuously. In Israel, a Keren Ne'emanut.
    MutualFund,
}

/// Where the security is traded. Decides both the trade fee row and the
/// custody row: Leumi charges 0.15% a quarter to hold Tel Aviv securities but
/// 0.2% for foreign ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exchange {
    /// Tel Aviv Stock Exchange.
    Tlv,
    /// Any US exchange (NYSE, Nasdaq).
    Usa,
    /// Any European exchange.
    Europe,
}

impl Exchange {
    /// The currency securities on this exchange trade in.
    pub fn currency(self) -> Currency {
        match self {
            Exchange::Tlv => Currency::Ils,
            Exchange::Usa => Currency::Usd,
            Exchange::Europe => Currency::Eur,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Period {
    Month,
    Quarter,
    Year,
}

impl Period {
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
    pub fn covers(&self, t: &Trade) -> bool {
        self.applies_to(t.security, t.exchange)
    }

    /// True if this row applies to `security` traded on `exchange`.
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
        /// Written as the number in the tariff: `dec!(0.15)` means 0.15%, not 15%.
        percent: Decimal,
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
    pub fn apply(&self, t: &Trade, rates: &ExchangeRates) -> Money {
        let ccy = t.value.currency;
        match self {
            Price::Percent { percent, min, max } => {
                let fee = t.value.amount * percent / Decimal::ONE_HUNDRED;
                clamp(
                    Money {
                        amount: fee,
                        currency: ccy,
                    },
                    *min,
                    *max,
                    rates,
                )
            }
            Price::PerShare {
                per_share,
                min,
                max,
            } => {
                // Multiply before converting: money2 rounds conversions to
                // cents, so 1¢ a share would become €0.01 instead of €0.008.
                let fee = Money {
                    amount: per_share.amount * t.shares,
                    ..*per_share
                };
                clamp(fee.exchange(ccy, rates), *min, *max, rates)
            }
            Price::Flat(amount) => amount.exchange(ccy, rates),
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
    /// The rate as written in the tariff: `dec!(0.15)` means 0.15%.
    pub percent: Decimal,
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
    pub fn covers(&self, h: &Holding) -> bool {
        empty_or_contains(&self.exchanges, h.exchange)
    }

    /// The rate converted to a yearly percentage: 0.15% a quarter is 0.6.
    /// Handy for displaying plans side by side.
    pub fn percent_per_year(&self) -> Decimal {
        self.percent * Decimal::from(self.per.times_per_year())
    }

    /// A year of this fee in ₪, on holdings worth `value` in total (all of
    /// them covered by this row). Nothing held, nothing charged.
    pub fn cost_per_year(&self, value: Money, rates: &ExchangeRates) -> Money {
        let value = value.exchange(Currency::Ils, rates).amount;
        if value.is_zero() {
            return ils(Decimal::ZERO);
        }
        let charges = Decimal::from(self.billed.times_per_year());
        let mut per_charge = value * self.percent_per_year() / Decimal::ONE_HUNDRED / charges;
        if let Some(min) = self.min {
            per_charge = per_charge.max(min.exchange(Currency::Ils, rates).amount);
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
/// (`spread_percent`). Leumi online charges 0.16% and doesn't publish its
/// spread; Altshuler charges no fee but a spread of up to 0.7%.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionFee {
    /// The fee listed in the tariff, as a percentage of the converted amount.
    pub percent: Decimal,
    /// The fee is never less than this.
    pub min: Option<Money>,
    /// The fee is never more than this.
    pub max: Option<Money>,
    /// How far the broker's exchange rate is from the market rate, as a
    /// percentage of that rate: Altshuler's "up to 0.7%". It isn't listed as a
    /// fee, but it's often the bigger cost. Not bounded by `min` or `max`.
    /// Zero when the tariff doesn't publish it, which understates the cost.
    pub spread_percent: Decimal,
}

impl ConversionFee {
    /// What converting `amount` costs, fee plus spread, in `amount`'s
    /// currency. The same in either direction (₪ to $ or $ to ₪).
    ///
    /// The spread is approximated as costing `spread_percent` of the amount.
    /// Exactly, a rate s% worse costs s% of the amount when you sell foreign
    /// currency, but s/(1+s) when you buy it: ₪13.90 rather than ₪14 on
    /// ₪2,000 at 0.7%. That gap is far smaller than the uncertainty in the
    /// published spread itself.
    pub fn cost(&self, amount: Money, rates: &ExchangeRates) -> Money {
        let fee = Money {
            amount: amount.amount * self.percent / Decimal::ONE_HUNDRED,
            ..amount
        };
        let spread = Money {
            amount: amount.amount * self.spread_percent / Decimal::ONE_HUNDRED,
            ..amount
        };
        clamp(fee, self.min, self.max, rates) + spread
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
    /// The trade fee table. The first row that covers a trade is used.
    pub trading: Vec<TradeFee>,
    /// The custody table. The first row that covers a holding is used.
    pub custody: Vec<CustodyFee>,
    /// The cost of converting between shekels and foreign currency.
    pub conversion: ConversionFee,
    /// Assumptions and gaps in the tariff that affect this plan's numbers,
    /// e.g. "doesn't say what orders over ₪30,000 cost". Shown to the user.
    /// Ones that apply to every plan go in [`Broker::notes`].
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Plan {
    /// The trade fee row used for `security` traded on `exchange`, if any.
    pub fn trade_row(&self, security: Security, exchange: Exchange) -> Option<&TradeFee> {
        self.trading
            .iter()
            .find(|row| row.applies_to(security, exchange))
    }

    /// The custody row used for holdings on `exchange`, if any.
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
    pub fn trade_fee(&self, t: &Trade, rates: &ExchangeRates) -> Option<Money> {
        self.trade_row(t.security, t.exchange)
            .map(|row| row.price.apply(t, rates))
    }

    /// A year of custody fees in ₪, assuming `holdings` stay the same all year.
    ///
    /// Each holding goes to the first custody row that covers it; each row is
    /// then charged on the total value of its holdings. That's why a row's
    /// minimum is paid once for the whole portfolio, not once per holding.
    pub fn custody_per_year(&self, holdings: &[Holding], rates: &ExchangeRates) -> Money {
        let mut value_per_row = vec![Decimal::ZERO; self.custody.len()];
        for h in holdings {
            if let Some(i) = self.custody.iter().position(|row| row.covers(h)) {
                value_per_row[i] += h.value.exchange(Currency::Ils, rates).amount;
            }
        }
        let total = self
            .custody
            .iter()
            .zip(value_per_row)
            .map(|(row, value)| row.cost_per_year(ils(value), rates).amount)
            .sum();
        ils(total)
    }

    /// What converting `amount` costs, fee plus spread, in `amount`'s currency.
    pub fn conversion_fee(&self, amount: Money, rates: &ExchangeRates) -> Money {
        self.conversion.cost(amount, rates)
    }
}

/// A broker or bank, and every plan it offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Broker {
    /// E.g. "Bank Leumi".
    pub name: String,
    /// The date on the tariff document the numbers came from. Tariffs change
    /// several times a year, so keep it next to the numbers.
    pub tariff_date: String,
    /// Where the tariff document can be read online.
    #[serde(default)]
    pub source_url: Option<String>,
    /// Like [`Plan::notes`], for ones that apply to every plan.
    #[serde(default)]
    pub notes: Vec<String>,
    /// Every plan a customer of this broker can be on.
    pub plans: Vec<Plan>,
}

// ─────────────────────────── Inputs ───────────────────────────

/// A single buy or sell, as input to [`Plan::trade_fee`].
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
pub struct Holding {
    pub exchange: Exchange,
    /// Current value, in any currency.
    pub value: Money,
}

// ─────────────────────────── Money helpers ───────────────────────────

/// Exchange rates from what a dollar and a euro cost in shekels, for when
/// they're typed in rather than downloaded.
pub fn exchange_rates(ils_per_usd: Decimal, ils_per_eur: Decimal) -> ExchangeRates {
    // money2 reads the ECB's CSV format, where every rate is per euro. It
    // ignores the date column.
    let usd_per_eur = ils_per_eur / ils_per_usd;
    format!("Date, USD, ILS\n-, {usd_per_eur}, {ils_per_eur}")
        .parse()
        .expect("two numbers always make valid CSV")
}

/// An amount in shekels: `ils(dec!(3.5))` is ₪3.50.
pub fn ils(amount: Decimal) -> Money {
    Money {
        amount,
        currency: Currency::Ils,
    }
}

/// An amount in US dollars: `usd(dec!(24))` is $24.
pub fn usd(amount: Decimal) -> Money {
    Money {
        amount,
        currency: Currency::Usd,
    }
}

// ─────────────────────────── Helpers ───────────────────────────

/// True if `list` contains `x`, or if `list` is empty (meaning "any").
fn empty_or_contains<T: PartialEq>(list: &[T], x: T) -> bool {
    list.is_empty() || list.contains(&x)
}

/// `fee` raised to `min` and lowered to `max`, converting the bounds to `fee`'s currency.
fn clamp(fee: Money, min: Option<Money>, max: Option<Money>, rates: &ExchangeRates) -> Money {
    let mut amount = fee.amount;
    if let Some(min) = min {
        amount = amount.max(min.exchange(fee.currency, rates).amount);
    }
    if let Some(max) = max {
        amount = amount.min(max.exchange(fee.currency, rates).amount);
    }
    Money { amount, ..fee }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// €1 = $1.25 = ₪4.625, so $1 = ₪3.70. Not real rates.
    fn rates() -> ExchangeRates {
        "Date, USD, ILS\n27 September 2026, 1.25, 4.625"
            .parse()
            .unwrap()
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
                percent: dec!(0.4),
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
            percent: dec!(0.15),
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
            percent: dec!(0.15),
            per: Period::Quarter,
            billed: Period::Quarter,
            min: None,
        };
        assert_eq!(leumi.percent_per_year(), dec!(0.6));
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
            percent: dec!(0.15),
            per: Period::Year,
            billed: Period::Month,
            min: Some(ils(dec!(75))),
        };
        assert_eq!(altshuler.percent_per_year(), dec!(0.15));
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
    fn conversion_fee_and_spread() {
        // Leumi online: 0.16%, minimum $5.76, maximum $2,400. Spread not published.
        let leumi = ConversionFee {
            percent: dec!(0.16),
            min: Some(usd(dec!(5.76))),
            max: Some(usd(dec!(2400))),
            spread_percent: dec!(0),
        };
        // Altshuler: no fee, but up to 0.7% spread.
        let altshuler = ConversionFee {
            percent: dec!(0),
            min: None,
            max: None,
            spread_percent: dec!(0.7),
        };
        assert_eq!(leumi.cost(usd(dec!(1000)), &rates()), usd(dec!(5.76)));
        assert_eq!(leumi.cost(usd(dec!(10000)), &rates()), usd(dec!(16)));
        assert_eq!(altshuler.cost(usd(dec!(1000)), &rates()), usd(dec!(7)));
    }
}
