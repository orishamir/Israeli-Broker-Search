//! Ready-made investing patterns, to try the comparison with one click:
//! the reasons Israelis invest for the long term, each showing a different
//! fee or rule as the one that matters. Each sets every basic input at once.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::simulation::Scenario;
use crate::{Exchange, Percent, Security, Text, Withdrawal};

/// One pattern: a short name for its button, the pattern in words, and the
/// inputs it sets, how the money is taken out and the saver's age included.
#[derive(Debug, Clone)]
pub struct Example {
    /// "Saving from your salary"
    pub name: Text,
    /// "₪1,500 a month from your salary, from 30 to 60…"
    pub explanation: Text,
    pub scenario: Scenario,
}

/// The examples, in the order to offer them.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the examples' data")]
pub fn all() -> Vec<Example> {
    let example = |name: Text, explanation: Text, scenario| Example {
        name,
        explanation,
        scenario,
    };
    // What the rest are variations on: the app's own defaults, buying every
    // month and selling everything at once at the end.
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
            Text::new("Saving from your salary", "חיסכון מהמשכורת"),
            Text::new(
                "₪1,500 a month from your salary, from 30 to 60, into an index fund that follows \
                 the S&P 500, on Tel Aviv, expecting 10% a year (the index's long-run average). \
                 Over 30 years, custody, charged every year on everything held, matters far more \
                 than the fee on each purchase.",
                "₪1,500 בחודש מהמשכורת, מגיל 30 ועד 60, לקרן מחקה על מדד S&P 500 בתל אביב, בציפייה ל-10% בשנה (הממוצע ארוך הטווח של המדד). לאורך 30 שנה, דמי המשמרת, שנגבים כל שנה מכל התיק, חשובים הרבה יותר מהעמלה על כל קנייה.",
            ),
            Scenario {
                security: Security::IndexFund,
                exchange: Exchange::Tlv,
                monthly_deposit: dec!(1500),
                years: 30,
                ..base.clone()
            },
        ),
        example(
            Text::new("Saving for a child", "חיסכון לילד"),
            Text::new(
                "₪300 a month from a child's birth until they're 18, into an index fund on Tel \
                 Aviv: for their studies, a flat or a wedding. With purchases this small, a \
                 minimum fee on each, or a fixed monthly fee, takes a large share of every \
                 deposit.",
                "₪300 בחודש מהלידה ועד גיל 18, לקרן מחקה בתל אביב: ללימודים, לדירה או לחתונה. בקניות קטנות כאלה, עמלת מינימום על כל קנייה, או דמי טיפול קבועים בכל חודש, לוקחים חלק גדול מכל הפקדה.",
            ),
            Scenario {
                security: Security::IndexFund,
                exchange: Exchange::Tlv,
                monthly_deposit: dec!(300),
                years: 18,
                ..base.clone()
            },
        ),
        example(
            Text::new("An inheritance", "ירושה"),
            Text::new(
                "₪500,000 at once, from an inheritance or a flat that was sold, into an ETF that \
                 follows the S&P 500, in the USA, for 15 years. There's one purchase, so what's \
                 charged for holding matters most. A provident fund for investment can't take so \
                 much in one year.",
                "₪500,000 בבת אחת, מירושה או ממכירת דירה, לקרן סל על מדד S&P 500 בארה״ב, ל-15 שנה. יש רק קנייה אחת, ולכן מה שחשוב הוא כמה גובים על החזקת התיק. לקופת גמל להשקעה אי אפשר להפקיד סכום כזה בשנה אחת.",
            ),
            Scenario {
                first_deposit: dec!(500_000),
                years: 15,
                age: 50,
                ..base.clone()
            },
        ),
        example(
            Text::new("Saving for retirement", "חיסכון לפרישה"),
            Text::new(
                "₪3,000 a month from 45 until 65, on top of the pension from work, into an ETF \
                 that follows the S&P 500, in the USA. A provident fund for investment pays the \
                 money out as a monthly pension free of tax, which can make up for its management \
                 fee.",
                "₪3,000 בחודש מגיל 45 ועד 65, בנוסף לפנסיה מהעבודה, לקרן סל על מדד S&P 500 בארה״ב. קופת גמל להשקעה משלמת את הכסף כקצבה חודשית פטורה ממס, וזה יכול לפצות על דמי הניהול שלה.",
            ),
            Scenario {
                monthly_deposit: dec!(3000),
                years: 20,
                age: 45,
                withdrawal: Withdrawal::Pension,
                ..base.clone()
            },
        ),
        example(
            Text::new("After retiring", "אחרי הפרישה"),
            Text::new(
                "₪400,000 in bonds on Tel Aviv, from 67, for 10 years at 4% a year: money kept \
                 with little risk after retiring. With a return this low, even custody of a \
                 fraction of a percent a year takes a large share of the interest.",
                "₪400,000 באג״ח בתל אביב, מגיל 67 ול-10 שנים, ב-4% בשנה: כסף שרוצים לשמור בלי סיכון גדול אחרי הפרישה. כשהתשואה נמוכה כל כך, גם דמי משמרת של חלקי אחוז בשנה לוקחים חלק גדול מהריבית.",
            ),
            Scenario {
                security: Security::Bond,
                exchange: Exchange::Tlv,
                first_deposit: dec!(400_000),
                yearly_return: Percent(dec!(4)),
                years: 10,
                age: 67,
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
    use crate::vehicles::{GainsTax, Vehicle};

    #[test]
    fn every_example_can_be_simulated_and_differs_in_name() {
        let examples = all();
        let names: Vec<&str> = examples.iter().map(|example| &*example.name.en).collect();
        assert_eq!(
            names,
            [
                "Saving from your salary",
                "Saving for a child",
                "An inheritance",
                "Saving for retirement",
                "After retiring"
            ]
        );
        for example in &examples {
            example.scenario.check().unwrap();
            assert!(!example.explanation.is_empty());
            assert!(!example.name.he.is_empty());
        }
    }

    /// What two tips say of a provident fund for investment holds for their
    /// inputs: the saver for retirement is old enough at the end for its
    /// tax-free pension, and the inheritance is more than it takes in a year.
    #[test]
    fn the_tips_about_the_provident_fund_hold() {
        let scenario = |name: &str| {
            all()
                .into_iter()
                .find(|example| example.name.en == name)
                .unwrap()
                .scenario
        };
        let rules = Vehicle::InvestmentGemel.rules();
        let retirement = scenario("Saving for retirement");
        assert_eq!(
            rules.tax_at(retirement.withdrawal, retirement.age + retirement.years),
            GainsTax::Exempt
        );
        let inheritance = scenario("An inheritance");
        assert!(inheritance.first_deposit > rules.deposit_ceiling.unwrap());
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
