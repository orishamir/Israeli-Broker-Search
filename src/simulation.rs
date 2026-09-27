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

use money2::{Currency, Exchange as _, ExchangeRates, Money};
use rust_decimal::Decimal;

use crate::{ConversionFee, Exchange, Holding, Plan, Price, Security, Trade, TradeFee};

/// What the user invests in, and how.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub security: Security,
    pub exchange: Exchange,
    /// ₪
    pub first_deposit: Decimal,
    /// ₪
    pub monthly_deposit: Decimal,
    /// 10 means 10% a year.
    pub yearly_return_percent: Decimal,
    pub years: u32,
    /// Deposits wait as uninvested cash until the next purchase, which happens
    /// every this many months. Buying less often means paying fewer minimum fees.
    pub buy_every_months: u32,
    /// Today's price of one share, in the exchange's currency. Only matters for
    /// plans that charge per share; it grows with the expected return.
    pub share_price: Decimal,
}

/// The result of following a [`Scenario`] on one plan. All amounts in ₪.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// What the investment is worth at the start (index 0) and at the end of
    /// each year, including cash waiting to be invested.
    pub value_by_year: Vec<Decimal>,
    /// Worth at the end, without selling.
    pub held: Decimal,
    /// What you'd get by selling everything at the end and, for foreign
    /// securities, converting back to shekels.
    pub after_selling: Decimal,
    /// Every fee paid while investing, not including selling at the end.
    pub fees_paid: Decimal,
}

/// Runs `scenario` on `plan`. Returns `None` if the plan has no price for
/// the security on that exchange.
pub fn simulate(plan: &Plan, scenario: &Scenario, rates: &ExchangeRates) -> Option<Outcome> {
    let currency = scenario.exchange.currency();
    let monthly_growth = monthly_growth(scenario.yearly_return_percent);

    let mut cash_ils = Decimal::ZERO;
    let mut invested = Decimal::ZERO; // in `currency`
    let mut share_price = scenario.share_price;
    let mut fees_paid = Decimal::ZERO;
    let mut value_by_year = vec![scenario.first_deposit];

    for month in 0..scenario.years * 12 {
        cash_ils += scenario.monthly_deposit;
        if month == 0 {
            cash_ils += scenario.first_deposit;
        }

        if month % scenario.buy_every_months.max(1) == 0 && cash_ils > Decimal::ZERO {
            let purchase = buy(plan, scenario, cash_ils, share_price, rates)?;
            invested += purchase.bought;
            fees_paid += purchase.fees_ils;
            cash_ils = Decimal::ZERO;
        }

        invested *= monthly_growth;
        share_price *= monthly_growth;

        let holding = Holding {
            exchange: scenario.exchange,
            value: money(invested, currency),
        };
        let custody_ils = plan.custody_per_year(&[holding], rates).amount / Decimal::from(12);
        fees_paid += custody_ils;
        invested -= in_currency(custody_ils, currency, rates);

        if (month + 1) % 12 == 0 {
            value_by_year.push(in_ils(invested, currency, rates) + cash_ils);
        }
    }

    let held = in_ils(invested, currency, rates) + cash_ils;
    let after_selling = sell(plan, scenario, invested, share_price, rates)? + cash_ils;
    Some(Outcome {
        value_by_year,
        held,
        after_selling,
        fees_paid,
    })
}

/// A plan that charges nothing, to measure what fees cost in lost growth.
pub fn free_plan() -> Plan {
    Plan {
        name: "No fees".into(),
        trading: vec![TradeFee {
            securities: vec![],
            exchanges: vec![],
            price: Price::Flat(crate::ils(Decimal::ZERO)),
        }],
        custody: vec![],
        conversion: ConversionFee {
            percent: Decimal::ZERO,
            min: None,
            max: None,
            spread_percent: Decimal::ZERO,
        },
    }
}

struct Purchase {
    /// Value of the securities bought, in the exchange's currency.
    bought: Decimal,
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
    let mut fees_ils = Decimal::ZERO;
    let mut available = crate::ils(cash_ils);

    if currency != Currency::Ils {
        let conversion = plan.conversion_fee(available, rates);
        fees_ils += conversion.amount;
        available = crate::ils(cash_ils - conversion.amount).exchange(currency, rates);
    }

    let trade = trade(scenario, available.amount, share_price);
    let trade_fee = plan.trade_fee(&trade, rates)?;
    fees_ils += in_ils(trade_fee.amount, currency, rates);

    Some(Purchase {
        // A fee larger than the purchase would make this negative; a real
        // broker would refuse the order instead.
        bought: (available.amount - trade_fee.amount).max(Decimal::ZERO),
        fees_ils,
    })
}

/// Sells `invested` (in the exchange's currency) and returns the proceeds in ₪.
fn sell(
    plan: &Plan,
    scenario: &Scenario,
    invested: Decimal,
    share_price: Decimal,
    rates: &ExchangeRates,
) -> Option<Decimal> {
    if invested <= Decimal::ZERO {
        return Some(Decimal::ZERO);
    }
    let currency = scenario.exchange.currency();
    let trade_fee = plan.trade_fee(&trade(scenario, invested, share_price), rates)?;
    let proceeds = money(invested - trade_fee.amount, currency);
    if currency == Currency::Ils {
        return Some(proceeds.amount);
    }
    let conversion = plan.conversion_fee(proceeds, rates);
    Some(in_ils(proceeds.amount - conversion.amount, currency, rates))
}

fn trade(scenario: &Scenario, value: Decimal, share_price: Decimal) -> Trade {
    Trade {
        security: scenario.security,
        exchange: scenario.exchange,
        // No share price (zero) means no shares, rather than dividing by zero.
        shares: value.checked_div(share_price).unwrap_or_default(),
        value: money(value, scenario.exchange.currency()),
    }
}

/// The monthly factor that compounds to `yearly_percent` over a year: 10%
/// a year is about 0.797% a month, not 10/12.
fn monthly_growth(yearly_percent: Decimal) -> Decimal {
    let yearly: f64 = (Decimal::ONE + yearly_percent / Decimal::ONE_HUNDRED)
        .try_into()
        .unwrap_or(1.0);
    Decimal::try_from(yearly.powf(1.0 / 12.0)).unwrap_or(Decimal::ONE)
}

fn money(amount: Decimal, currency: Currency) -> Money {
    Money { amount, currency }
}

fn in_ils(amount: Decimal, currency: Currency, rates: &ExchangeRates) -> Decimal {
    money(amount, currency)
        .exchange(Currency::Ils, rates)
        .amount
}

fn in_currency(amount_ils: Decimal, currency: Currency, rates: &ExchangeRates) -> Decimal {
    crate::ils(amount_ils).exchange(currency, rates).amount
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchange_rates;
    use rust_decimal_macros::dec;

    fn rates() -> ExchangeRates {
        exchange_rates(dec!(3.7), dec!(4.625))
    }

    fn scenario() -> Scenario {
        Scenario {
            security: Security::Etf,
            exchange: Exchange::Tlv,
            first_deposit: dec!(10000),
            monthly_deposit: dec!(1000),
            yearly_return_percent: dec!(0),
            years: 2,
            buy_every_months: 1,
            share_price: dec!(100),
        }
    }

    #[test]
    fn no_fees_no_growth_keeps_every_deposit() {
        let outcome = simulate(&free_plan(), &scenario(), &rates()).unwrap();
        assert_eq!(
            outcome.value_by_year,
            [dec!(10000), dec!(22000), dec!(34000)]
        );
        assert_eq!(outcome.after_selling, dec!(34000));
        assert_eq!(outcome.fees_paid, dec!(0));
    }

    #[test]
    fn growth_compounds_to_the_yearly_rate() {
        let s = Scenario {
            monthly_deposit: dec!(0),
            yearly_return_percent: dec!(10),
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
        assert!(quarterly.fees_paid < monthly.fees_paid);
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
