//! Ready-made investing patterns, to try the comparison with one click: the
//! ways Israelis commonly invest for the long term. Each sets every input
//! at once.

use rust_decimal_macros::dec;

use crate::simulation::Scenario;
use crate::{Exchange, Percent, Security, Text};

/// One pattern: a short name for its button, the pattern in words, and the
/// inputs it sets.
#[derive(Debug, Clone)]
pub struct Example {
    /// "Monthly, Tel Aviv"
    pub name: Text,
    /// "₪2,000 a month into an index fund on Tel Aviv, for 20 years…"
    pub explanation: Text,
    pub scenario: Scenario,
}

/// The examples, in the order to offer them.
#[must_use]
pub fn all() -> Vec<Example> {
    let example = |name: Text, explanation: Text, scenario| Example {
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
            Text::new("Monthly, Tel Aviv", "חודשי, תל אביב"),
            Text::new(
                "₪2,000 a month into an index fund on Tel Aviv, for 20 years, expecting 10% a \
                 year (the S&P 500's long-run average): steady saving from a salary, in shekels.",
                "₪2,000 בחודש לקרן מחקה בתל אביב, ל-20 שנה, בציפייה ל-10% בשנה (הממוצע ארוך הטווח של S&P 500): חיסכון קבוע מהמשכורת, בשקלים.",
            ),
            Scenario {
                security: Security::IndexFund,
                exchange: Exchange::Tlv,
                monthly_deposit: dec!(2000),
                ..base.clone()
            },
        ),
        example(
            Text::new("Monthly, US ETF", "חודשי, קרן סל בארה\u{5f4}ב"),
            Text::new(
                "₪10,000 to start, then ₪3,000 a month into an ETF in the USA, for 20 years at \
                 10% a year: the same saving in dollars, where converting the shekels costs too.",
                "₪10,000 להתחלה, ואז ₪3,000 בחודש לקרן סל בארה״ב, ל-20 שנה ב-10% בשנה: אותו חיסכון בדולרים, שבו גם המרת השקלים עולה כסף.",
            ),
            Scenario {
                first_deposit: dec!(10000),
                monthly_deposit: dec!(3000),
                ..base.clone()
            },
        ),
        example(
            Text::new("One lump sum", "סכום חד-פעמי"),
            Text::new(
                "₪200,000 at once into an ETF in the USA, and nothing more, for 10 years at 10% \
                 a year: an inheritance or a bonus. One purchase, so what's charged for holding \
                 matters most.",
                "₪200,000 בבת אחת לקרן סל בארה״ב, ולא יותר, ל-10 שנים ב-10% בשנה: ירושה או בונוס. קנייה אחת, ולכן מה שנגבה על ההחזקה חשוב ביותר.",
            ),
            Scenario {
                first_deposit: dec!(200_000),
                years: 10,
                ..base.clone()
            },
        ),
        example(
            Text::new("Bonds, 5 years", "אג\u{5f4}ח, 5 שנים"),
            Text::new(
                "₪100,000 to start, then ₪1,000 a month into bonds on Tel Aviv, for 5 years at \
                 4% a year: money that's needed before long, where a small fee is a large share \
                 of the interest.",
                "₪100,000 להתחלה, ואז ₪1,000 בחודש לאג״ח בתל אביב, ל-5 שנים ב-4% בשנה: כסף שיידרש בקרוב, שבו עמלה קטנה היא חלק גדול מהריבית.",
            ),
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
        let names: Vec<&str> = examples.iter().map(|example| &*example.name.en).collect();
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
            assert!(!example.explanation.is_empty());
            assert!(!example.name.he.is_empty());
        }
    }
}
