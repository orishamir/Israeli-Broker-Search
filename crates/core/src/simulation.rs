//! Simulates an investing plan month by month: deposits, conversions,
//! purchases, growth and custody, then selling everything at the end.
//!
//! Simplifications, all small next to the fees themselves:
//! - Deposits arrive at the start of each month; the first deposit arrives
//!   together with the first monthly one.
//! - The expected return is in the security's own currency. Exchange rates
//!   stay the same for the whole period.
//! - Custody is charged monthly as a twelfth of a year's fee on the current
//!   value, and is taken out of the investment.
//! - Taxes are ignored.

use crate::money::{Currency, ExchangeRates, Money, iso};
use rust_decimal::Decimal;

use crate::{
    Broker, ConversionFee, Exchange, Holding, Percent, Plan, Price, Security, Trade, TradeFee,
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
    /// Today's price of one share, in the exchange's currency. Only matters for
    /// plans that charge per share; it grows with the expected return.
    pub share_price: Decimal,
}

impl Scenario {
    /// Every shekel put in over the years, in ₪.
    #[must_use]
    pub fn deposited(&self) -> Decimal {
        self.first_deposit + self.monthly_deposit * Decimal::from(12 * self.years)
    }
}

/// The result of following a [`Scenario`] on one plan. All amounts in ₪.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// What the investment is worth at the start (index 0) and at the end of
    /// each month, including cash waiting to be invested.
    pub value_by_month: Vec<Decimal>,
    /// Worth at the end, without selling.
    pub held: Decimal,
    /// What you'd get by selling everything at the end and, for foreign
    /// securities, converting back to shekels.
    pub after_selling: Decimal,
    /// Every fee paid, including selling at the end.
    pub fees: Fees,
    /// The fees paid during each year (index 0 is the first year), not
    /// including selling at the end.
    pub fees_by_year: Vec<Fees>,
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
    /// Selling everything at the end and converting back to shekels.
    pub selling: Decimal,
}

impl Fees {
    #[must_use]
    pub fn total(&self) -> Decimal {
        self.purchases + self.conversions + self.custody + self.selling
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
    /// and at the end of each month.
    pub fn lost_by_month<'a>(&'a self, no_fees: &'a Outcome) -> impl Iterator<Item = Decimal> + 'a {
        no_fees
            .value_by_month
            .iter()
            .zip(&self.value_by_month)
            .map(|(no_fees, this)| no_fees - this)
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

/// Runs `scenario` on `plan`. Returns `None` if the plan has no price for
/// the security on that exchange.
#[must_use]
pub fn simulate(plan: &Plan, scenario: &Scenario, rates: &ExchangeRates) -> Option<Outcome> {
    let currency = scenario.exchange.currency();
    let monthly_growth = monthly_growth(scenario.yearly_return);

    let mut cash_ils = Decimal::ZERO;
    let mut invested = Decimal::ZERO; // in `currency`
    let mut share_price = scenario.share_price;
    let mut value_by_month = vec![scenario.first_deposit];
    let mut fees_by_year = Vec::new();
    let mut fees_this_year = Fees::default();

    for month in 0..scenario.years * 12 {
        cash_ils += scenario.monthly_deposit;
        if month == 0 {
            cash_ils += scenario.first_deposit;
        }

        if month % scenario.buy_every_months.max(1) == 0 && cash_ils > Decimal::ZERO {
            let purchase = buy(plan, scenario, cash_ils, share_price, rates)?;
            invested += purchase.bought;
            fees_this_year += purchase.fees;
            cash_ils = Decimal::ZERO;
        }

        invested *= monthly_growth;
        share_price *= monthly_growth;

        let holding = Holding {
            exchange: scenario.exchange,
            value: Money::from_decimal(invested, currency),
        };
        let custody_ils = *plan.custody_per_year(&[holding], rates).amount() / Decimal::from(12);
        fees_this_year.custody += custody_ils;
        invested -= in_currency(custody_ils, currency, rates);

        value_by_month.push(in_ils(invested, currency, rates) + cash_ils);
        if (month + 1) % 12 == 0 {
            fees_by_year.push(std::mem::take(&mut fees_this_year));
        }
    }

    let held = in_ils(invested, currency, rates) + cash_ils;
    let sale = sell(plan, scenario, invested, share_price, rates)?;
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
    })
}

/// Every compared plan's outcome, best first, and what the same investing
/// would come to with no fees at all.
#[derive(Debug, Clone)]
pub struct Comparison {
    pub no_fees: Outcome,
    pub plans: Vec<PlanOutcome>,
}

#[derive(Debug, Clone)]
pub struct PlanOutcome {
    /// Where the plan is in the brokers list.
    pub broker: usize,
    pub plan: usize,
    /// `None` if the plan doesn't offer the security on that exchange.
    pub outcome: Option<Outcome>,
}

/// Runs `scenario` on each of `plans` (broker and plan positions in
/// `brokers`), sorted by what's left after selling, most first. Plans that
/// don't offer the security go last.
#[must_use]
pub fn compare(
    brokers: &[Broker],
    plans: &[(usize, usize)],
    scenario: &Scenario,
    rates: &ExchangeRates,
) -> Comparison {
    let no_fees =
        simulate(&free_plan(), scenario, rates).expect("the free plan covers every trade");
    let mut plans: Vec<PlanOutcome> = plans
        .iter()
        .map(|&(broker, plan)| PlanOutcome {
            broker,
            plan,
            outcome: simulate(&brokers[broker].plans[plan], scenario, rates),
        })
        .collect();
    plans.sort_by_key(|plan| std::cmp::Reverse(plan.outcome.as_ref().map(|o| o.after_selling)));
    Comparison { no_fees, plans }
}

/// A plan that charges nothing, to measure what fees cost in lost growth.
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
        custody: vec![],
        conversion: ConversionFee::default(),
        min_first_deposit: None,
        caveats: vec![],
    }
}

struct Purchase {
    /// Value of the securities bought, in the exchange's currency.
    bought: Decimal,
    /// Only `purchases` and `conversions` are set.
    fees: Fees,
}

struct Sale {
    proceeds_ils: Decimal,
    /// The sell fee plus converting back to shekels.
    fees_ils: Decimal,
}

/// Converts `cash_ils` to the exchange's currency if needed, and buys with
/// all of it, fees included.
fn buy(
    plan: &Plan,
    scenario: &Scenario,
    cash_ils: Decimal,
    share_price: Decimal,
    rates: &ExchangeRates,
) -> Option<Purchase> {
    let currency = scenario.exchange.currency();
    let mut fees = Fees::default();
    let mut available = crate::ils(cash_ils);

    if currency != iso::ILS {
        fees.conversions = *plan.conversion_fee(available, rates).amount();
        available = rates.convert(crate::ils(cash_ils - fees.conversions), currency);
    }

    let trade = trade(scenario, *available.amount(), share_price);
    let trade_fee = plan.trade_fee(&trade, rates)?;
    fees.purchases = in_ils(*trade_fee.amount(), currency, rates);

    Some(Purchase {
        // A fee larger than the purchase would make this negative; a real
        // broker would refuse the order instead.
        bought: (available.amount() - trade_fee.amount()).max(Decimal::ZERO),
        fees,
    })
}

/// Sells `invested` (in the exchange's currency) and converts the proceeds to ₪.
fn sell(
    plan: &Plan,
    scenario: &Scenario,
    invested: Decimal,
    share_price: Decimal,
    rates: &ExchangeRates,
) -> Option<Sale> {
    if invested <= Decimal::ZERO {
        return Some(Sale {
            proceeds_ils: Decimal::ZERO,
            fees_ils: Decimal::ZERO,
        });
    }
    let currency = scenario.exchange.currency();
    let trade_fee = plan.trade_fee(&trade(scenario, invested, share_price), rates)?;
    let proceeds = invested - trade_fee.amount();
    let conversion = if currency == iso::ILS {
        Decimal::ZERO
    } else {
        *plan
            .conversion_fee(Money::from_decimal(proceeds, currency), rates)
            .amount()
    };
    Some(Sale {
        proceeds_ils: in_ils(proceeds - conversion, currency, rates),
        fees_ils: in_ils(trade_fee.amount() + conversion, currency, rates),
    })
}

fn trade(scenario: &Scenario, value: Decimal, share_price: Decimal) -> Trade {
    Trade {
        security: scenario.security,
        exchange: scenario.exchange,
        // No share price (zero) means no shares, rather than dividing by zero.
        shares: value.checked_div(share_price).unwrap_or_default(),
        value: Money::from_decimal(value, scenario.exchange.currency()),
    }
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
        let brokers = [crate::tariffs::altshuler(), crate::tariffs::leumi()];
        let s = Scenario {
            exchange: Exchange::Europe, // Altshuler doesn't offer it
            ..scenario()
        };
        let comparison = compare(&brokers, &[(0, 0), (1, 0), (1, 3)], &s, &rates());
        let order: Vec<(usize, usize)> = comparison
            .plans
            .iter()
            .map(|p| (p.broker, p.plan))
            .collect();
        assert_eq!(order, [(1, 3), (1, 0), (0, 0)]); // Pepper, Online, then Altshuler
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
}
