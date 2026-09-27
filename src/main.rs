//! The kind of question the graphs answer: "I put ₪X a month into a US S&P 500
//! ETF. What does a year of fees cost me at each broker?"
//!
//! cargo run --example compare

use broker_fees::tariffs::{altshuler, leumi};
use broker_fees::*;
use money2::{Currency, Exchange as _, ExchangeRates};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn main() {
    // User input
    let monthly_deposit = ils(dec!(2000));
    let share_price = usd(dec!(500));
    // Fixed example rates ($1 = ₪3.70). For today's: `ExchangeRates::new().await`
    let rates: ExchangeRates = "Date, USD, ILS\n27 September 2026, 1.25, 4.625"
        .parse()
        .unwrap();

    println!(
        "Yearly fees in ₪ by portfolio size, depositing {} a month into a US ETF\n",
        monthly_deposit.amount
    );
    let sizes = [
        dec!(50000),
        dec!(100000),
        dec!(250000),
        dec!(500000),
        dec!(1000000),
    ];
    print!("{:<48}", "");
    for s in sizes {
        print!("{:>10}", s);
    }
    println!();

    for broker in [altshuler(), leumi()] {
        for plan in &broker.plans {
            print!("{:<48}", format!("{} – {}", broker.name, plan.name));
            for size in sizes {
                let fees = yearly_fees(plan, monthly_deposit, share_price, ils(size), &rates);
                print!("{:>10}", fees.amount.round());
            }
            println!();
        }
    }
}

/// 12 monthly conversions and purchases, plus a year of custody on the portfolio.
fn yearly_fees(
    plan: &Plan,
    monthly_deposit: money2::Money,
    share_price: money2::Money,
    portfolio: money2::Money,
    rates: &ExchangeRates,
) -> money2::Money {
    let dollars = monthly_deposit.exchange(Currency::Usd, rates);
    let conversion = plan.conversion_fee(monthly_deposit, rates);
    let purchase = plan
        .trade_fee(
            &Trade {
                security: Security::Etf,
                exchange: Exchange::Usa,
                shares: dollars.amount / share_price.amount,
                value: dollars,
            },
            rates,
        )
        .expect("every plan here has a US ETF row");
    let per_month = conversion.amount + purchase.exchange(Currency::Ils, rates).amount;

    let custody = plan.custody_per_year(
        &[Holding {
            exchange: Exchange::Usa,
            value: portfolio,
        }],
        rates,
    );
    ils(per_month * Decimal::from(12) + custody.amount)
}
