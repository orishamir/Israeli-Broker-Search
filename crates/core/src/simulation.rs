//! Simulates an investing plan month by month: deposits, conversions,
//! purchases, growth and custody, then selling everything at the end.
//!
//! Money moves as it does at a broker:
//! - Deposits arrive at the start of each month, the first deposit together
//!   with the first monthly one, and wait as shekels until a purchase.
//! - A purchase converts the waiting shekels for a foreign security, then
//!   buys with them and with converted money left over from before. ETFs and
//!   stocks abroad are bought in whole shares ([`whole_shares`]), unless the
//!   broker sells fractions there, which can leave some over. A purchase
//!   that the fee would swallow, or too small for a share, isn't made: the
//!   money waits for the next one.
//! - Custody is charged every month, a twelfth of a year's fee on what's
//!   held, and so is a plan's handling fee; both are paid from the shekels.
//!   What they don't cover is owed, and the next deposits repay it first.
//! - A plan with tracks is run on each, and costs what the cheapest does.
//! - At the end everything is sold and converted back to shekels, except
//!   what would cost more to sell than it's worth.
//!
//! Simplifications, all small next to the fees themselves:
//! - The expected return is in the security's own currency. Exchange rates
//!   stay the same for the whole period.
//! - Waiting money earns nothing, and owing costs nothing.
//! - Taxes are ignored.

use crate::money::{Currency, ExchangeRates, Money, iso};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::{
    ConversionFee, Exchange, Holding, IntoEnumIterator, Percent, Plan, Price, Security, Trade,
    TradeFee,
};

/// What the user invests in, and how.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub security: Security,
    pub exchange: Exchange,
    /// ₪
    pub first_deposit: Decimal,
    /// ₪
    pub monthly_deposit: Decimal,
    /// A year's growth, in the security's own currency.
    pub yearly_return: Percent,
    pub years: u32,
    /// Deposits wait as uninvested cash until the next purchase, which happens
    /// every this many months. Buying less often means paying fewer minimum fees.
    pub buy_every_months: u32,
    /// Today's price of one share, in the exchange's currency. It decides how
    /// many whole shares a purchase buys ([`whole_shares`]) and what plans
    /// charging per share cost; it grows with the expected return.
    pub share_price: Decimal,
}

/// The most, in ₪, that the deposits may grow to: far more than any real
/// portfolio, and far enough below the largest `Decimal` (about 8 × 10²⁸)
/// that fees and conversions on it can't overflow.
const LARGEST_VALUE: f64 = 1e15;

/// Why a scenario can't be simulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidScenario {
    #[error("the yearly return can't be below −100%")]
    ReturnBelowMinus100,
    /// Beyond [`LARGEST_VALUE`], where the decimal arithmetic could overflow.
    #[error("the deposits would grow too large to calculate")]
    TooLarge,
}

impl Scenario {
    /// Every shekel put in over the years, in ₪.
    #[must_use]
    pub fn deposited(&self) -> Decimal {
        self.first_deposit + self.monthly_deposit * Decimal::from(12 * self.years)
    }

    /// Checks for what [`simulate`] can't handle: losing more than
    /// everything, and amounts too large for its decimal arithmetic.
    pub fn check(&self) -> Result<(), InvalidScenario> {
        if self.yearly_return.0 < -Decimal::ONE_HUNDRED {
            return Err(InvalidScenario::ReturnBelowMinus100);
        }
        // Estimated in floating point, which can't overflow: every deposit,
        // grown for the whole period.
        let number = |value: Decimal| value.to_f64().unwrap_or(f64::INFINITY);
        let years = f64::from(self.years);
        let deposited = number(self.first_deposit) + number(self.monthly_deposit) * 12.0 * years;
        let growth = (1.0 + number(self.yearly_return.0) / 100.0)
            .max(1.0)
            .powf(years);
        // Written so that NaN fails it too.
        if deposited * growth <= LARGEST_VALUE {
            Ok(())
        } else {
            Err(InvalidScenario::TooLarge)
        }
    }
}

/// Whether `security` on `exchange` is bought in whole shares, unless the
/// broker sells fractions there ([`Plan::fractions_on`]). It matters for ETFs
/// and stocks abroad: one share can cost more than a monthly deposit. Mutual
/// funds are bought by amount, and bonds and Tel Aviv shares are simplified
/// to that too; Tel Aviv units cost little next to a deposit.
#[must_use]
pub fn whole_shares(security: Security, exchange: Exchange) -> bool {
    matches!(security, Security::Etf | Security::Stock) && exchange != Exchange::Tlv
}

/// Whether the share price changes what buying `security` on `exchange`
/// costs on any of `plans`: one of them buys it in whole shares or charges
/// per share. With no plans, whether shares are usually bought whole.
#[must_use]
pub fn share_price_matters(security: Security, exchange: Exchange, plans: &[&Plan]) -> bool {
    if plans.is_empty() {
        return whole_shares(security, exchange);
    }
    plans.iter().any(|plan| {
        let rows = plan
            .tracks
            .iter()
            .flat_map(|track| &track.trading)
            .chain(&plan.trading)
            .chain(&plan.standing_orders);
        let per_share = rows
            .filter(|row| row.applies_to(security, exchange))
            .any(|row| {
                matches!(
                    row.price,
                    Price::PerShare { .. } | Price::PercentPlusPerShare { .. }
                )
            });
        let whole = whole_shares(security, exchange) && !plan.sells_fractions_on(exchange);
        whole || per_share
    })
}

/// The result of following a [`Scenario`] on one plan. All amounts in ₪.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// What the investment is worth at the start (index 0) and at the end of
    /// each month, including money waiting to be invested, less fees owed.
    pub value_by_month: Vec<Decimal>,
    /// Worth at the end, without selling.
    pub held: Decimal,
    /// What you'd get by selling everything at the end and, for foreign
    /// securities, converting back to shekels. Negative if fees are owed
    /// beyond what's held.
    pub after_selling: Decimal,
    /// Every fee paid, including selling at the end.
    pub fees: Fees,
    /// The fees paid during each year (index 0 is the first year), not
    /// including selling at the end.
    pub fees_by_year: Vec<Fees>,
    /// The plan's track used, the cheapest, if its tracks price this trade.
    pub track: Option<usize>,
    /// The biggest single order, in the exchange's currency: a purchase, or
    /// selling everything at the end. Some caveats matter only above an
    /// amount.
    pub largest_trade: Decimal,
}

/// Fees in ₪, by what they were charged for. Adding two adds each kind, so
/// they can be totalled with `+`, `+=` and `.sum()`.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sum,
)]
pub struct Fees {
    /// Buying securities.
    pub purchases: Decimal,
    /// Changing shekels into the security's currency.
    pub conversions: Decimal,
    /// Holding the securities.
    pub custody: Decimal,
    /// Keeping the account: a monthly handling fee.
    pub handling: Decimal,
    /// Selling everything at the end and converting back to shekels.
    pub selling: Decimal,
}

impl Fees {
    #[must_use]
    pub fn total(&self) -> Decimal {
        self.purchases + self.conversions + self.custody + self.handling + self.selling
    }
}

impl Outcome {
    /// What's left after selling, compared with `no_fees`: how much the fees
    /// cost in the end, lost growth included.
    #[must_use]
    pub fn lost_to_fees(&self, no_fees: &Outcome) -> Decimal {
        no_fees.after_selling - self.after_selling
    }

    /// How much less this investment is worth than `no_fees`, at the start
    /// and at the end of each month. The last is after selling, like
    /// [`Outcome::lost_to_fees`], so a chart of it ends where the table does.
    pub fn lost_by_month<'a>(&'a self, no_fees: &'a Outcome) -> impl Iterator<Item = Decimal> + 'a {
        no_fees
            .values_then_sold()
            .zip(self.values_then_sold())
            .map(|(no_fees, this)| no_fees - this)
    }

    /// [`Outcome::value_by_month`], with the last value after selling.
    fn values_then_sold(&self) -> impl Iterator<Item = Decimal> + '_ {
        let before_the_end = &self.value_by_month[..self.value_by_month.len() - 1];
        before_the_end
            .iter()
            .copied()
            .chain(std::iter::once(self.after_selling))
    }

    /// The fees paid by the end of each year (index 0 is the first year).
    /// The last year includes selling at the end.
    #[must_use]
    pub fn fees_up_to_each_year(&self) -> Vec<Fees> {
        let mut up_to: Vec<Fees> = self
            .fees_by_year
            .iter()
            .scan(Fees::default(), |total, year| {
                *total += *year;
                Some(*total)
            })
            .collect();
        if let Some(last) = up_to.last_mut() {
            last.selling = self.fees.selling;
        }
        up_to
    }
}

/// Runs `scenario` on `plan`, on its cheapest track if it has tracks.
/// Returns `None` if the plan has no price for the security on that
/// exchange. Check the scenario first ([`Scenario::check`]): absurd amounts
/// can overflow.
#[must_use]
pub fn simulate(plan: &Plan, scenario: &Scenario, rates: &ExchangeRates) -> Option<Outcome> {
    if plan.tracks.is_empty() {
        return simulate_prices(plan, scenario, rates);
    }
    // The first of the cheapest: tracks that don't price this trade cost
    // the same, and give the first track, reported as none.
    let (index, mut outcome) = (0..plan.tracks.len())
        .filter_map(|index| {
            Some((
                index,
                simulate_prices(&plan.on_track(index), scenario, rates)?,
            ))
        })
        .min_by_key(|(_, outcome)| std::cmp::Reverse(outcome.after_selling))?;
    outcome.track = plan
        .track_for(index, scenario.security, scenario.exchange)
        .map(|_| index);
    Some(outcome)
}

/// Runs `scenario` on `plan`'s own prices, without choosing a track.
fn simulate_prices(plan: &Plan, scenario: &Scenario, rates: &ExchangeRates) -> Option<Outcome> {
    let price = &plan.trade_row(scenario.security, scenario.exchange)?.price;
    // A standing order buys every month; buying less often takes one-off orders.
    let standing_order = plan
        .standing_order_row(scenario.security, scenario.exchange)
        .filter(|_| scenario.buy_every_months == 1)
        .map(|row| &row.price);
    let currency = scenario.exchange.currency();
    let monthly_growth = monthly_growth(scenario.yearly_return);

    let mut cash_ils = Decimal::ZERO; // negative while fees are owed
    let mut left = Decimal::ZERO; // converted and not yet spent, in `currency`
    let mut invested = Decimal::ZERO; // in `currency`
    let mut share_price = scenario.share_price;
    let mut value_by_month = vec![scenario.first_deposit];
    let mut fees_by_year = Vec::new();
    let mut fees_this_year = Fees::default();
    let mut largest_trade = Decimal::ZERO;

    for month in 0..scenario.years * 12 {
        let one_time = if month == 0 {
            scenario.first_deposit
        } else {
            Decimal::ZERO
        };
        cash_ils += scenario.monthly_deposit + one_time;
        let mut trade_fees_this_month = Decimal::ZERO;

        if month % scenario.buy_every_months.max(1) == 0 && cash_ils > Decimal::ZERO {
            // A standing order buys only the monthly deposits; the one-time
            // deposit is a one-off order.
            let conversion = &plan.conversion;
            let orders = match standing_order {
                Some(standing_order) => {
                    let by_standing_order = plan
                        .standing_order_conversion
                        .as_ref()
                        .unwrap_or(conversion);
                    vec![
                        (one_time, price, conversion),
                        (cash_ils - one_time, standing_order, by_standing_order),
                    ]
                }
                None => vec![(cash_ils, price, conversion)],
            };
            for (amount, price, conversion) in orders {
                if amount <= Decimal::ZERO {
                    continue;
                }
                let order = Order {
                    price,
                    conversion,
                    cash_ils: amount,
                    left,
                    share_price,
                };
                let Some(purchase) = buy(plan, scenario, &order, rates) else {
                    continue;
                };
                cash_ils -= amount;
                left = purchase.left;
                invested += purchase.bought;
                largest_trade = largest_trade.max(purchase.bought);
                trade_fees_this_month += purchase.fees.purchases;
                fees_this_year += purchase.fees;
            }
        }

        invested *= monthly_growth;
        share_price *= monthly_growth;

        let holding = Holding {
            security: scenario.security,
            exchange: scenario.exchange,
            value: Money::from_decimal(invested, currency),
        };
        let custody_ils = *plan.custody_per_year(&[holding], rates).amount() / Decimal::from(12);
        fees_this_year.custody += custody_ils;
        cash_ils -= custody_ils;

        let handling_ils = plan.handling.map_or(Decimal::ZERO, |fee| {
            fee.for_month(month, trade_fees_this_month)
        });
        fees_this_year.handling += handling_ils;
        cash_ils -= handling_ils;

        value_by_month.push(in_ils(invested + left, currency, rates) + cash_ils);
        if (month + 1) % 12 == 0 {
            fees_by_year.push(std::mem::take(&mut fees_this_year));
        }
    }

    let held = in_ils(invested + left, currency, rates) + cash_ils;
    let sale = sell(plan, price, scenario, invested, left, share_price, rates);
    let fees = Fees {
        selling: sale.fees_ils,
        ..fees_by_year.iter().copied().sum()
    };
    Some(Outcome {
        value_by_month,
        held,
        after_selling: sale.proceeds_ils + cash_ils,
        fees,
        fees_by_year,
        track: None,
        largest_trade: largest_trade.max(invested),
    })
}

/// Every compared plan's outcome, best first, and what the same investing
/// would come to with no fees at all.
#[derive(Debug, Clone)]
pub struct Comparison {
    /// With no fees, buying every month.
    pub no_fees: Outcome,
    pub plans: Vec<PlanOutcome>,
}

#[derive(Debug, Clone)]
pub struct PlanOutcome {
    /// Where the plan is in the list given to [`compare`].
    pub index: usize,
    /// `None` if the plan doesn't offer the security on that exchange.
    pub outcome: Option<Outcome>,
}

/// Runs `scenario` on each of `plans`, sorted by what's left after selling,
/// most first. Plans that don't offer the security go last.
#[must_use]
pub fn compare(plans: &[&Plan], scenario: &Scenario, rates: &ExchangeRates) -> Comparison {
    // With no fees there's no reason to wait, so the no-fee plan buys every
    // month, and the growth lost while money waits counts as lost to fees.
    let buying_monthly = Scenario {
        buy_every_months: 1,
        ..scenario.clone()
    };
    let no_fees =
        simulate(&free_plan(), &buying_monthly, rates).expect("the free plan covers every trade");
    let mut plans: Vec<PlanOutcome> = plans
        .iter()
        .enumerate()
        .map(|(index, plan)| PlanOutcome {
            index,
            outcome: simulate(plan, scenario, rates),
        })
        .collect();
    plans.sort_by_key(|plan| std::cmp::Reverse(plan.outcome.as_ref().map(|o| o.after_selling)));
    Comparison { no_fees, plans }
}

/// A plan that charges nothing, to measure what fees cost in lost growth.
/// It buys fractions of a share, so every shekel is invested at once: the
/// growth lost while money waits for a whole share counts as lost too.
#[must_use]
pub fn free_plan() -> Plan {
    Plan {
        name: "No fees".into(),
        description: String::new(),
        trading: vec![TradeFee {
            securities: vec![],
            exchanges: vec![],
            price: Price::Flat(crate::ils(Decimal::ZERO)),
        }],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion: ConversionFee::FREE,
        handling: None,
        fractions_on: Exchange::iter().collect(),
        min_first_deposit: None,
        caveats: vec![],
    }
}

struct Purchase {
    /// Value of the securities bought, in the exchange's currency.
    bought: Decimal,
    /// Converted money not spent, in the exchange's currency.
    left: Decimal,
    /// Only `purchases` and `conversions` are set.
    fees: Fees,
}

struct Sale {
    proceeds_ils: Decimal,
    /// The sell fee plus converting back to shekels.
    fees_ils: Decimal,
}

/// A purchase to make: the money for it, and what it costs.
struct Order<'a> {
    price: &'a Price,
    /// What converting its shekels costs.
    conversion: &'a ConversionFee,
    /// Shekels to buy with.
    cash_ils: Decimal,
    /// Converted money left over from earlier purchases, in the exchange's
    /// currency.
    left: Decimal,
    /// Today's price of one share, in the exchange's currency.
    share_price: Decimal,
}

/// Makes `order`: converts its shekels for a foreign security, and buys with
/// them and with the money left over from before. `None` if nothing would be
/// bought, because the money doesn't cover a share or the fee would take it
/// all: then no order is placed, and the money waits.
fn buy(plan: &Plan, scenario: &Scenario, order: &Order, rates: &ExchangeRates) -> Option<Purchase> {
    let currency = scenario.exchange.currency();
    let (price, share_price) = (order.price, order.share_price);
    let mut fees = Fees::default();
    let mut available = order.cash_ils;
    if currency != iso::ILS {
        let cash = crate::ils(order.cash_ils);
        fees.conversions = *order.conversion.cost(cash, rates).amount();
        available = in_currency(order.cash_ils - fees.conversions, currency, rates);
    }
    available += order.left;

    let whole = whole_shares(scenario.security, scenario.exchange)
        && !plan.sells_fractions_on(scenario.exchange);
    let (bought, trade_fee) = if whole && share_price > Decimal::ZERO {
        let shares = affordable_shares(price, scenario, available, share_price, rates);
        let value = shares * share_price;
        (value, trade_fee(price, scenario, value, shares, rates))
    } else {
        // Charged on all the money, including the part that pays the fee:
        // more than exact by the fee's percentage of itself (0.4% of 0.4%).
        let shares = charged_shares(plan, scenario, shares_worth(available, share_price));
        let fee = trade_fee(price, scenario, available, shares, rates);
        (available - fee, fee)
    };
    if bought <= Decimal::ZERO {
        return None;
    }
    fees.purchases = in_ils(trade_fee, currency, rates);
    Some(Purchase {
        bought,
        left: available - bought - trade_fee,
        fees,
    })
}

/// The shares a trade of `shares` is charged for. Where a broker sells
/// fractions of shares that are otherwise bought whole, a fraction is charged
/// as a whole share, as Interactive Israel's tariff says.
fn charged_shares(plan: &Plan, scenario: &Scenario, shares: Decimal) -> Decimal {
    let (security, exchange) = (scenario.security, scenario.exchange);
    if whole_shares(security, exchange) && plan.sells_fractions_on(exchange) {
        shares.ceil()
    } else {
        shares
    }
}

/// The most whole shares that `money` buys at `share_price` (more than 0),
/// the trade's fee included.
fn affordable_shares(
    price: &Price,
    scenario: &Scenario,
    money: Decimal,
    share_price: Decimal,
    rates: &ExchangeRates,
) -> Decimal {
    let cost = |shares: Decimal| {
        let value = shares * share_price;
        value + trade_fee(price, scenario, value, shares, rates)
    };
    // A fee never falls as a trade grows, so halve the range between no
    // shares and as many as the money buys without a fee, keeping `low`
    // affordable.
    let (mut low, mut high) = (Decimal::ZERO, (money / share_price).floor());
    while low < high {
        let middle = ((low + high + Decimal::ONE) / Decimal::TWO).floor();
        if cost(middle) <= money {
            low = middle;
        } else {
            high = middle - Decimal::ONE;
        }
    }
    low
}

/// Sells `invested` and converts it, with the money `left` over, back to
/// shekels, each in the exchange's currency. What would cost more to sell or
/// convert than it's worth is kept instead, and counts as nothing.
fn sell(
    plan: &Plan,
    price: &Price,
    scenario: &Scenario,
    invested: Decimal,
    left: Decimal,
    share_price: Decimal,
    rates: &ExchangeRates,
) -> Sale {
    let currency = scenario.exchange.currency();
    let mut proceeds = left;
    let mut fees = Decimal::ZERO;
    let shares = charged_shares(plan, scenario, shares_worth(invested, share_price));
    let sell_fee = trade_fee(price, scenario, invested, shares, rates);
    if sell_fee < invested {
        proceeds += invested - sell_fee;
        fees += sell_fee;
    }
    if currency != iso::ILS && proceeds > Decimal::ZERO {
        let conversion = *plan
            .conversion_fee(Money::from_decimal(proceeds, currency), rates)
            .amount();
        if conversion < proceeds {
            proceeds -= conversion;
            fees += conversion;
        } else {
            proceeds = Decimal::ZERO;
        }
    }
    Sale {
        proceeds_ils: in_ils(proceeds, currency, rates),
        fees_ils: in_ils(fees, currency, rates),
    }
}

/// What `price` charges for trading `shares` worth `value`, in the
/// exchange's currency.
fn trade_fee(
    price: &Price,
    scenario: &Scenario,
    value: Decimal,
    shares: Decimal,
    rates: &ExchangeRates,
) -> Decimal {
    let trade = Trade {
        security: scenario.security,
        exchange: scenario.exchange,
        shares,
        value: Money::from_decimal(value, scenario.exchange.currency()),
    };
    *price.apply(&trade, rates).amount()
}

/// How many shares, fractions included, `value` is worth.
fn shares_worth(value: Decimal, share_price: Decimal) -> Decimal {
    // No share price (zero) means no shares, rather than dividing by zero.
    value.checked_div(share_price).unwrap_or_default()
}

/// The monthly factor that compounds to `yearly` over a year: 10% a year is
/// about 0.797% a month, not 10/12.
fn monthly_growth(yearly: Percent) -> Decimal {
    let factor: f64 = (Decimal::ONE + yearly.of(Decimal::ONE))
        .try_into()
        .unwrap_or(1.0);
    Decimal::try_from(factor.powf(1.0 / 12.0)).unwrap_or(Decimal::ONE)
}

fn in_ils(amount: Decimal, currency: &'static Currency, rates: &ExchangeRates) -> Decimal {
    *rates
        .convert(Money::from_decimal(amount, currency), iso::ILS)
        .amount()
}

fn in_currency(amount_ils: Decimal, currency: &'static Currency, rates: &ExchangeRates) -> Decimal {
    *rates.convert(crate::ils(amount_ils), currency).amount()
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    use crate::{HandlingFee, PercentFee};

    fn rates() -> ExchangeRates {
        ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
    }

    fn scenario() -> Scenario {
        Scenario {
            security: Security::Etf,
            exchange: Exchange::Tlv,
            first_deposit: dec!(10000),
            monthly_deposit: dec!(1000),
            yearly_return: Percent(dec!(0)),
            years: 2,
            buy_every_months: 1,
            share_price: dec!(100),
        }
    }

    #[test]
    fn no_fees_no_growth_keeps_every_deposit() {
        let outcome = simulate(&free_plan(), &scenario(), &rates()).unwrap();
        let by_year: Vec<_> = outcome.value_by_month.iter().copied().step_by(12).collect();
        assert_eq!(by_year, [dec!(10000), dec!(22000), dec!(34000)]);
        assert_eq!(outcome.value_by_month.len(), 2 * 12 + 1);
        assert_eq!(outcome.after_selling, dec!(34000));
        assert_eq!(outcome.fees, Fees::default());
    }

    #[test]
    fn growth_compounds_to_the_yearly_rate() {
        let s = Scenario {
            monthly_deposit: dec!(0),
            yearly_return: Percent(dec!(10)),
            ..scenario()
        };
        let outcome = simulate(&free_plan(), &s, &rates()).unwrap();
        // ₪10,000 at 10% for two years is ₪12,100.
        assert_eq!(outcome.held.round(), dec!(12100));
    }

    #[test]
    fn buying_less_often_pays_fewer_minimums() {
        let plan = crate::tariffs::leumi().plans.remove(0); // Online: ₪26 minimum on Tel Aviv
        let monthly = simulate(&plan, &scenario(), &rates()).unwrap();
        let quarterly = Scenario {
            buy_every_months: 3,
            ..scenario()
        };
        let quarterly = simulate(&plan, &quarterly, &rates()).unwrap();
        assert!(quarterly.fees.purchases < monthly.fees.purchases);
    }

    #[test]
    fn fees_are_split_by_type_and_year() {
        let plan = crate::tariffs::leumi().plans.remove(0); // Online
        let outcome = simulate(&plan, &scenario(), &rates()).unwrap();
        // Tel Aviv: no conversions. The first purchase (₪11,000) pays 0.4% = ₪44,
        // the other 11 of the year the ₪26 minimum.
        assert_eq!(outcome.fees.conversions, dec!(0));
        assert_eq!(outcome.fees_by_year.len(), 2);
        assert_eq!(
            outcome.fees_by_year[0].purchases,
            dec!(44) + dec!(11) * dec!(26)
        );
        assert!(outcome.fees.custody > dec!(0));
        assert!(outcome.fees.selling > dec!(0));

        let yearly_total: Fees = outcome.fees_by_year.iter().copied().sum();
        // Selling at the end is the only fee not in any year.
        assert_eq!(
            yearly_total.total() + outcome.fees.selling,
            outcome.fees.total()
        );
        let up_to = outcome.fees_up_to_each_year();
        assert_eq!(up_to.last(), Some(&outcome.fees));
        assert_eq!(up_to[0].selling, dec!(0));
    }

    #[test]
    fn compare_puts_the_best_first_and_unoffered_last() {
        let (altshuler, leumi) = (crate::tariffs::altshuler(), crate::tariffs::leumi());
        let s = Scenario {
            exchange: Exchange::Europe, // Altshuler doesn't offer it
            ..scenario()
        };
        let plans = [&altshuler.plans[0], &leumi.plans[0], &leumi.plans[3]];
        let comparison = compare(&plans, &s, &rates());
        let order: Vec<usize> = comparison.plans.iter().map(|p| p.index).collect();
        assert_eq!(order, [2, 1, 0]); // Pepper, Online, then Altshuler
        assert!(comparison.plans[2].outcome.is_none());
        assert_eq!(comparison.no_fees.fees, Fees::default());
    }

    #[test]
    fn plan_without_a_price_is_none() {
        let altshuler = crate::tariffs::altshuler().plans.remove(0);
        let s = Scenario {
            exchange: Exchange::Europe,
            ..scenario()
        };
        assert!(simulate(&altshuler, &s, &rates()).is_none());
    }

    /// A plan with a trade price for anything, and nothing else to pay. It
    /// buys whole shares, like most brokers.
    fn priced(price: Price) -> Plan {
        Plan {
            trading: vec![TradeFee {
                securities: vec![],
                exchanges: vec![],
                price,
            }],
            fractions_on: vec![],
            ..free_plan()
        }
    }

    /// "`percent`%, at least ₪`min`"
    fn percent(percent: Decimal, min: Decimal) -> Price {
        Price::Percent {
            percent: Percent(percent),
            min: Some(crate::ils(min)),
            max: None,
        }
    }

    #[test]
    fn a_standing_order_buys_only_the_monthly_deposits() {
        // Like Leumi's: 0.4% (at least ₪26), or 0.225% (at least ₪5) by standing order.
        let plan = Plan {
            standing_orders: priced(percent(dec!(0.225), dec!(5))).trading,
            ..priced(percent(dec!(0.4), dec!(26)))
        };
        let s = Scenario {
            first_deposit: dec!(100000),
            monthly_deposit: dec!(2000),
            years: 1,
            ..scenario()
        };
        let outcome = simulate(&plan, &s, &rates()).unwrap();
        // The one-time deposit is a one-off order: 0.4% of ₪100,000 = ₪400.
        // Each ₪2,000 goes by standing order: 0.225% = ₪4.50 → ₪5, 12 times.
        assert_eq!(outcome.fees.purchases, dec!(400) + dec!(12) * dec!(5));
        // Selling isn't buying: ₪99,600 + 12 × ₪1,995 = ₪123,540, at 0.4%.
        assert_eq!(outcome.fees.selling, dec!(494.16));

        // Buying every other month, there's no standing order: all pay 0.4%.
        let every_other = Scenario {
            buy_every_months: 2,
            ..s
        };
        let without = priced(percent(dec!(0.4), dec!(26)));
        assert_eq!(
            simulate(&plan, &every_other, &rates()).unwrap().fees,
            simulate(&without, &every_other, &rates()).unwrap().fees
        );
    }

    #[test]
    fn whole_shares_wait_until_one_is_affordable() {
        // A $500 share at $1 a trade, bought with ₪1,000 (about $270) a month.
        let plan = priced(Price::Flat(crate::usd(dec!(1))));
        let s = Scenario {
            exchange: Exchange::Usa,
            first_deposit: dec!(0),
            monthly_deposit: dec!(1000),
            years: 1,
            share_price: dec!(500),
            ..scenario()
        };
        let outcome = simulate(&plan, &s, &rates()).unwrap();
        // A share is affordable every other month: 6 trades, not 12.
        assert_eq!(outcome.fees.purchases, dec!(6) * dec!(3.7));
        // The dollars left over still count: only the fees are missing.
        let unaccounted = s.deposited() - outcome.after_selling - outcome.fees.total();
        assert_eq!(unaccounted.round_dp(10), dec!(0));
    }

    #[test]
    fn a_purchase_the_fee_would_swallow_isnt_made() {
        // ₪50 is $13.51, less than a $24 minimum: the money waits instead.
        let plan = priced(Price::Percent {
            percent: Percent(dec!(0.3)),
            min: Some(crate::usd(dec!(24))),
            max: None,
        });
        let s = Scenario {
            security: Security::Bond,
            exchange: Exchange::Usa,
            first_deposit: dec!(50),
            monthly_deposit: dec!(0),
            ..scenario()
        };
        let outcome = simulate(&plan, &s, &rates()).unwrap();
        assert_eq!(outcome.fees, Fees::default());
        assert_eq!(outcome.after_selling, dec!(50));
    }

    #[test]
    fn fees_beyond_the_money_are_owed() {
        // Altshuler's custody is at least ₪75 a month; this deposits ₪50.
        let altshuler = Plan {
            handling: None,
            ..crate::tariffs::altshuler().plans.remove(0)
        };
        let s = Scenario {
            first_deposit: dec!(0),
            monthly_deposit: dec!(50),
            years: 10,
            ..scenario()
        };
        let outcome = simulate(&altshuler, &s, &rates()).unwrap();
        assert!(outcome.value_by_month.iter().any(|value| *value < dec!(0)));
        // ₪6,000 deposited, ₪9,007 of fees: ₪3.50 to buy, 120 × ₪75 for
        // custody, ₪3.50 to sell. What's missing is exactly the fees.
        assert_eq!(outcome.fees.total(), dec!(9007));
        assert_eq!(outcome.after_selling, dec!(6000) - dec!(9007));
    }

    #[test]
    fn lost_to_fees_is_measured_against_buying_monthly() {
        let plan = crate::tariffs::leumi().plans.remove(0);
        let buying_every = |months| Scenario {
            yearly_return: Percent(dec!(10)),
            buy_every_months: months,
            ..scenario()
        };
        let monthly = compare(&[&plan], &buying_every(1), &rates());
        let yearly = compare(&[&plan], &buying_every(12), &rates());
        // The same no-fee plan for both, so their losses can be compared.
        assert_eq!(monthly.no_fees.after_selling, yearly.no_fees.after_selling);
        // A chart of the loss ends after selling, as the table does.
        let outcome = yearly.plans[0].outcome.as_ref().unwrap();
        assert_eq!(
            outcome.lost_by_month(&yearly.no_fees).last(),
            Some(outcome.lost_to_fees(&yearly.no_fees))
        );
    }

    #[test]
    fn no_plan_ends_with_more_than_no_fees() {
        // Buying fractions of a share, as Interactive does, invests sooner
        // than waiting for whole shares; the no-fee plan buys fractions too.
        let plans: Vec<Plan> = crate::tariffs::all()
            .into_iter()
            .flat_map(|broker| broker.plans)
            .collect();
        let plans: Vec<&Plan> = plans.iter().collect();
        let (etf, bond, fund) = (Security::Etf, Security::Bond, Security::MutualFund);
        let (tlv, usa, europe) = (Exchange::Tlv, Exchange::Usa, Exchange::Europe);
        for (security, exchange) in [
            (etf, usa),
            (etf, europe),
            (etf, tlv),
            (bond, usa),
            (fund, tlv),
        ] {
            for (share_price, buy_every_months) in [(dec!(5), 1), (dec!(500), 1), (dec!(500), 3)] {
                // The app's defaults: over 20 years, investing sooner adds
                // up to more than a cheap plan's fees.
                let s = Scenario {
                    security,
                    exchange,
                    share_price,
                    buy_every_months,
                    monthly_deposit: dec!(2000),
                    yearly_return: Percent(dec!(10)),
                    years: 20,
                    ..scenario()
                };
                let comparison = compare(&plans, &s, &rates());
                for compared in &comparison.plans {
                    if let Some(outcome) = &compared.outcome {
                        let lost = outcome.lost_to_fees(&comparison.no_fees);
                        let name = &plans[compared.index].name;
                        assert!(lost > dec!(0), "{name}, {security} on {exchange}: {lost}");
                    }
                }
            }
        }
    }

    #[test]
    fn checks_catch_what_cant_be_simulated() {
        let with_return = |percent| Scenario {
            yearly_return: Percent(percent),
            ..scenario()
        };
        assert_eq!(with_return(dec!(-100)).check(), Ok(()));
        assert_eq!(
            with_return(dec!(-150)).check(),
            Err(InvalidScenario::ReturnBelowMinus100)
        );
        let huge = Scenario {
            years: 50,
            ..with_return(dec!(500))
        };
        assert_eq!(huge.check(), Err(InvalidScenario::TooLarge));
    }

    #[test]
    fn the_handling_fee_starts_after_its_free_months() {
        // ₪15 a month after two free years, like Meitav's typical offer.
        let plan = Plan {
            handling: Some(HandlingFee {
                per_month: crate::ils(dec!(15)),
                free_months: 24,
                less_trade_fees: false,
            }),
            ..free_plan()
        };
        let s = Scenario {
            years: 3,
            ..scenario()
        };
        let outcome = simulate(&plan, &s, &rates()).unwrap();
        let by_year: Vec<Decimal> = outcome
            .fees_by_year
            .iter()
            .map(|fees| fees.handling)
            .collect();
        assert_eq!(by_year, [dec!(0), dec!(0), dec!(12) * dec!(15)]);
        assert_eq!(outcome.after_selling, s.deposited() - dec!(180));
    }

    #[test]
    fn a_months_trade_fees_can_count_toward_the_handling_fee() {
        // ₪15 a month, less what that month's trades paid: ₪4 a purchase.
        let plan = Plan {
            handling: Some(HandlingFee {
                per_month: crate::ils(dec!(15)),
                free_months: 0,
                less_trade_fees: true,
            }),
            ..priced(Price::Flat(crate::ils(dec!(4))))
        };
        let s = Scenario {
            first_deposit: dec!(0),
            years: 1,
            ..scenario()
        };
        let monthly = simulate(&plan, &s, &rates()).unwrap();
        assert_eq!(monthly.fees.purchases, dec!(12) * dec!(4));
        assert_eq!(monthly.fees.handling, dec!(12) * dec!(11));
        // Buying every quarter, the 8 months without a purchase pay it all.
        let quarterly = Scenario {
            buy_every_months: 3,
            ..s
        };
        let quarterly = simulate(&plan, &quarterly, &rates()).unwrap();
        assert_eq!(
            quarterly.fees.handling,
            dec!(4) * dec!(11) + dec!(8) * dec!(15)
        );
    }

    #[test]
    fn fractions_are_charged_as_whole_shares() {
        // $1 a share, buying with ₪555 ($150) in $100 shares.
        let per_share = priced(Price::PerShare {
            per_share: crate::usd(dec!(1)),
            min: None,
            max: None,
        });
        let s = Scenario {
            exchange: Exchange::Usa,
            first_deposit: dec!(555),
            monthly_deposit: dec!(0),
            years: 1,
            ..scenario()
        };
        // Whole shares only: one share for $1, and $49 waits.
        let whole = simulate(&per_share, &s, &rates()).unwrap();
        assert_eq!(whole.fees.purchases, dec!(3.7));
        // With fractions all of it is bought, but 1.5 shares pay for 2: $2.
        let fractions = Plan {
            fractions_on: vec![Exchange::Usa],
            ..per_share
        };
        let outcome = simulate(&fractions, &s, &rates()).unwrap();
        assert_eq!(outcome.fees.purchases, dec!(7.4));
        // Selling the 1.48 shares bought pays for 2 as well.
        assert_eq!(outcome.fees.selling, dec!(7.4));
    }

    #[test]
    fn a_standing_order_can_convert_for_less() {
        // 1% to convert, but nothing when buying by standing order, like
        // Interactive's automatic investment plan.
        let plan = Plan {
            standing_orders: free_plan().trading,
            standing_order_conversion: Some(ConversionFee::FREE),
            conversion: ConversionFee {
                fee: PercentFee {
                    percent: Percent(dec!(1)),
                    min: None,
                    max: None,
                },
                ..ConversionFee::FREE
            },
            ..free_plan()
        };
        let s = Scenario {
            security: Security::MutualFund,
            exchange: Exchange::Usa,
            first_deposit: dec!(3700),
            monthly_deposit: dec!(370),
            years: 1,
            ..scenario()
        };
        // Only the one-time deposit pays: 1% of ₪3,700.
        let monthly = simulate(&plan, &s, &rates()).unwrap();
        assert_eq!(monthly.fees.conversions, dec!(37));
        // Buying every other month, there's no standing order: 1% of ₪4,070
        // first, then of ₪740 five times. December's deposit isn't bought.
        let every_other = Scenario {
            buy_every_months: 2,
            ..s
        };
        let every_other = simulate(&plan, &every_other, &rates()).unwrap();
        assert_eq!(
            every_other.fees.conversions,
            dec!(40.7) + dec!(5) * dec!(7.4)
        );
    }

    #[test]
    fn custody_can_depend_on_the_security() {
        // IBI's full tariff charges 0.1% a quarter, but nothing on Tel Aviv
        // index funds.
        let ibi = crate::tariffs::ibi().plans.remove(0);
        let custody = |security| {
            let s = Scenario {
                security,
                years: 1,
                ..scenario()
            };
            simulate(&ibi, &s, &rates()).unwrap().fees.custody
        };
        assert_eq!(custody(Security::MutualFund), dec!(0));
        assert!(custody(Security::Etf) > dec!(0));
    }

    #[test]
    fn the_share_price_matters_for_whole_shares_and_per_share_prices() {
        let leumi = crate::tariffs::leumi();
        let online = [&leumi.plans[0]];
        assert!(share_price_matters(
            Security::Etf,
            Exchange::Europe,
            &online
        ));
        assert!(!share_price_matters(
            Security::MutualFund,
            Exchange::Usa,
            &online
        ));
        assert!(!share_price_matters(Security::Etf, Exchange::Tlv, &online));
        let per_share = priced(Price::PerShare {
            per_share: crate::ils(dec!(0.01)),
            min: None,
            max: None,
        });
        assert!(share_price_matters(
            Security::Etf,
            Exchange::Tlv,
            &[&per_share]
        ));
    }
}
