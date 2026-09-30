//! Ready-made investing patterns, to try the comparison with one click: the
//! ways Israelis commonly invest for the long term. Each sets every input
//! at once.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::simulation::Scenario;
use crate::{Exchange, Percent, Security, Text, Withdrawal};

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
        inflation: Percent(dec!(0)),
        age: 30,
        withdrawal: Withdrawal::LumpSum,
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
                "₪200,000 בבת אחת לקרן סל בארה״ב, ולא יותר, ל-10 שנים ב-10% בשנה: למשל ירושה או בונוס. יש רק קנייה אחת, ולכן מה שחשוב הוא כמה גובים על החזקת התיק.",
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
                "₪100,000 להתחלה, ואז ₪1,000 בחודש לאג״ח בתל אביב, ל-5 שנים ב-4% בשנה: כסף שתצטרכו בקרוב, שבו גם עמלה קטנה אוכלת חלק גדול מהריבית.",
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

/// A short-term pattern: a name for its button, the pattern in words, and
/// the amounts and months it sets. The rate the saver expects stays theirs.
#[derive(Debug, Clone)]
pub struct ShortTermExample {
    pub name: Text,
    pub explanation: Text,
    /// ₪, put in at the start.
    pub first_deposit: Decimal,
    /// ₪, put in every month.
    pub monthly_deposit: Decimal,
    pub months: u32,
}

/// The short-term examples, in the order to offer them.
#[must_use]
pub fn short_term() -> Vec<ShortTermExample> {
    vec![
        ShortTermExample {
            name: Text::new("₪100,000 for a year", "₪100,000 לשנה"),
            explanation: Text::new(
                "₪100,000 kept for a year, until it's needed: money from a sale, say, or \
                 for a payment that's coming.",
                "₪100,000 שנשמרים לשנה, עד שיהיה בהם צורך: למשל כסף ממכירה, או לתשלום שמתקרב.",
            ),
            first_deposit: dec!(100_000),
            monthly_deposit: dec!(0),
            months: 12,
        },
        ShortTermExample {
            name: Text::new("An emergency fund", "כרית ביטחון"),
            explanation: Text::new(
                "₪30,000 put aside for a rainy day, over six months: money you may need any \
                 day, so when it can come out matters as much as what it earns.",
                "₪30,000 בצד למקרה חירום, לחצי שנה: כסף שאולי תצטרכו בכל יום, ולכן חשוב מתי אפשר למשוך אותו, ולא רק כמה הוא מרוויח.",
            ),
            first_deposit: dec!(30_000),
            monthly_deposit: dec!(0),
            months: 6,
        },
        ShortTermExample {
            name: Text::new("Saving for a flat, 3 years", "חיסכון לדירה, 3 שנים"),
            explanation: Text::new(
                "₪5,000 a month for three years, towards a flat's first payment. A deposit \
                 takes one sum, so only the funds take money every month.",
                "₪5,000 בחודש במשך שלוש שנים, לקראת התשלום הראשון על דירה. פיקדון מקבל סכום אחד, ולכן רק הקרנות מקבלות הפקדה חודשית.",
            ),
            first_deposit: dec!(0),
            monthly_deposit: dec!(5000),
            months: 36,
        },
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

    /// Each short-term example sets a scenario that can be worked out.
    #[test]
    fn every_short_term_example_can_be_worked_out() {
        let examples = short_term();
        let names: Vec<&str> = examples.iter().map(|example| &*example.name.en).collect();
        assert_eq!(
            names,
            [
                "₪100,000 for a year",
                "An emergency fund",
                "Saving for a flat, 3 years"
            ]
        );
        for example in &examples {
            let scenario = crate::short_term::Scenario {
                first_deposit: example.first_deposit,
                monthly_deposit: example.monthly_deposit,
                months: example.months,
                rate: crate::short_term::TODAYS_RATE,
                inflation: crate::simulation::USUAL_INFLATION,
            };
            scenario.check().unwrap();
            assert!(!example.explanation.he.is_empty());
        }
    }
}
