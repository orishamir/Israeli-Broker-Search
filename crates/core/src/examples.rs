//! Ready-made investing patterns, to try the comparison with one click: the
//! ways Israelis commonly invest for the long term. Each sets every input
//! at once.

use rust_decimal_macros::dec;

use crate::simulation::Scenario;
use crate::{Exchange, Percent, Security};

/// One pattern: a short name for its button, the pattern in words, and the
/// inputs it sets.
#[derive(Debug, Clone)]
pub struct Example {
    /// "Monthly, Tel Aviv"
    pub name: &'static str,
    /// "₪2,000 a month into an index fund on Tel Aviv, for 20 years…"
    pub explanation: &'static str,
    pub scenario: Scenario,
}

/// The examples, in the order to offer them.
#[must_use]
pub fn all() -> Vec<Example> {
    let example = |name, explanation, scenario| Example {
        name,
        explanation,
        scenario,
    };
    // What the rest are variations on: the app's own defaults, buying every
    // month and selling at the end.
    let base = Scenario {
        security: Security::Etf,
        exchange: Exchange::Usa,
        first_deposit: dec!(0),
        monthly_deposit: dec!(0),
        deposit_growth: Percent(dec!(0)),
        yearly_return: Percent(dec!(10)),
        years: 20,
        buy_every_months: 1,
        share_price: dec!(500),
        sell_at_end: true,
    };
    vec![
        example(
            "Monthly, Tel Aviv",
            "₪2,000 a month into an index fund on Tel Aviv, for 20 years, expecting 10% a year \
             (the S&P 500's long-run average): steady saving from a salary, in shekels.",
            Scenario {
                security: Security::IndexFund,
                exchange: Exchange::Tlv,
                monthly_deposit: dec!(2000),
                ..base.clone()
            },
        ),
        example(
            "Monthly, US ETF",
            "₪10,000 to start, then ₪3,000 a month into an ETF in the USA, for 20 years at 10% a \
             year: the same saving in dollars, where converting the shekels costs too.",
            Scenario {
                first_deposit: dec!(10000),
                monthly_deposit: dec!(3000),
                ..base.clone()
            },
        ),
        example(
            "One lump sum",
            "₪200,000 at once into an ETF in the USA, and nothing more, for 10 years at 10% a \
             year: an inheritance or a bonus. One purchase, so what's charged for holding \
             matters most.",
            Scenario {
                first_deposit: dec!(200_000),
                years: 10,
                ..base.clone()
            },
        ),
        example(
            "Bonds, 5 years",
            "₪100,000 to start, then ₪1,000 a month into bonds on Tel Aviv, for 5 years at 4% a \
             year: money that's needed before long, where a small fee is a large share of \
             the interest.",
            Scenario {
                security: Security::Bond,
                exchange: Exchange::Tlv,
                first_deposit: dec!(100_000),
                monthly_deposit: dec!(1000),
                yearly_return: Percent(dec!(4)),
                years: 5,
                ..base
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_example_can_be_simulated_and_differs_in_name() {
        let examples = all();
        let names: Vec<&str> = examples.iter().map(|example| example.name).collect();
        assert_eq!(
            names,
            [
                "Monthly, Tel Aviv",
                "Monthly, US ETF",
                "One lump sum",
                "Bonds, 5 years"
            ]
        );
        for example in &examples {
            example.scenario.check().unwrap();
            assert_ne!(example.explanation, "");
        }
    }
}
