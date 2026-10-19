//! What the page draws beside an answer: each kind's money month by month,
//! in today's shekels, and the band of what the index did over stretches as
//! long as the horizon. The numbers come from the same engines as the
//! answer's alternatives; only the whole line is kept here, not just where
//! it ends.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::history::Band;
use crate::{Percent, Text};

use super::safe::price_level;
use super::{Amount, Arena, Kind, Safety};

/// One line of the alternatives chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KindSeries {
    pub kind: Kind,
    pub name: Text,
    pub safety: Safety,
    /// The kind the answer recommends: the others are drawn against it.
    pub recommended: bool,
    /// Today's shekels at the start (index 0) and after each month. The
    /// last point is what is left after tax, which is the figure shown.
    pub value_by_month: Vec<Decimal>,
    pub left: Amount,
}

/// What every stretch as long as the horizon did to the same deposits
/// (before fees and tax: it is the index itself), in today's shekels.
#[derive(Debug, Clone, PartialEq)]
pub struct BandSeries {
    pub band: Band,
    pub worst: Vec<Decimal>,
    pub best: Vec<Decimal>,
}

/// The chart's data.
#[derive(Debug, Clone, PartialEq)]
pub struct Alternatives {
    /// The months the answer's money is kept.
    pub months: u32,
    pub series: Vec<KindSeries>,
    pub band: Option<BandSeries>,
}

/// A column of the list of places or plans within a kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum ColumnId {
    /// What is left at the end, after tax: what the list is ranked by.
    LeftAfterTax,
    /// What the place or plan loses a year, as a fund's fee is stated.
    CostAYear,
    /// Fees paid over the years (the index kinds' lists only).
    Fees,
    /// Tax paid on the gain.
    Tax,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Column {
    pub id: ColumnId,
    pub name: Text,
}

/// The columns of the list within an arena, in the order shown. The safe
/// kinds' places differ by what they leave, their yearly cost and the tax;
/// the index kinds' plans also show the fees.
#[must_use]
pub fn columns_for(arena: Arena) -> Vec<Column> {
    let column = |id, en, he| Column {
        id,
        name: Text::new(en, he),
    };
    let mut columns = vec![
        column(ColumnId::LeftAfterTax, "Left after tax", "נשאר אחרי מס"),
        column(ColumnId::CostAYear, "Cost a year", "עלות בשנה"),
    ];
    // A fund's or a bank's cost is the yearly one; fees are paid to brokers.
    if arena == Arena::Index {
        columns.push(column(ColumnId::Fees, "Fees", "עמלות"));
    }
    columns.push(column(ColumnId::Tax, "Tax", "מס"));
    columns
}

/// The deposits put end to end, left as they are: today's shekels at the
/// start (the first sum alone) and after each month's deposit.
pub(super) fn deposits_by_month(
    first: Decimal,
    monthly: Decimal,
    months: u32,
    inflation: Percent,
) -> Vec<Decimal> {
    grown(first, monthly, months, inflation, Decimal::ONE)
}

/// The deposits grown at `yearly` a year, in today's shekels.
pub(super) fn grown_at(
    first: Decimal,
    monthly: Decimal,
    months: u32,
    inflation: Percent,
    yearly: Percent,
) -> Vec<Decimal> {
    let factor = (1.0 + yearly.0.to_f64().unwrap_or(0.0) / 100.0)
        .max(0.0)
        .powf(1.0 / 12.0);
    grown(
        first,
        monthly,
        months,
        inflation,
        Decimal::try_from(factor).unwrap_or(Decimal::ONE),
    )
}

fn grown(
    first: Decimal,
    monthly: Decimal,
    months: u32,
    inflation: Percent,
    factor: Decimal,
) -> Vec<Decimal> {
    let per_month = price_level(inflation, 1);
    let mut prices = Decimal::ONE;
    let mut value = first;
    let mut series = vec![value];
    for _ in 1..=months {
        prices *= per_month;
        value = (value + monthly) * factor;
        series.push(value / prices);
    }
    series
}

/// A nominal line as today's shekels, month by month, ending at `left`.
/// `waiting` is a deposit the engine already counts in the month before it
/// is made (the short term's, which starts at the first sum plus the first
/// month's deposit): it is taken off, so month 0 is the first sum alone in
/// every line, as the brokers' engine has it.
pub(super) fn deflated(
    nominal: &[Decimal],
    inflation: Percent,
    left: Decimal,
    waiting: Decimal,
) -> Vec<Decimal> {
    let last = nominal.len().saturating_sub(1);
    let per_month = price_level(inflation, 1);
    let mut prices = Decimal::ONE;
    let mut series: Vec<Decimal> = nominal
        .iter()
        .enumerate()
        .map(|(month, value)| {
            if month > 0 {
                prices *= per_month;
            }
            let value = if month < last {
                value - waiting
            } else {
                *value
            };
            value / prices
        })
        .collect();
    if let Some(last) = series.last_mut() {
        *last = left;
    }
    series
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::guide::safe::price_level;

    /// Prices rise a little each month, and a deposit the engine counts a
    /// month early is taken off every month but the last, which is `left`.
    #[test]
    fn deflating_divides_by_the_prices_of_that_month() {
        let inflation = Percent(dec!(2));
        let nominal = [dec!(100), dec!(110), dec!(120), dec!(130)];
        let series = deflated(&nominal, inflation, dec!(77), dec!(10));
        assert_eq!(series[0], dec!(90));
        assert_eq!(series[1], dec!(100) / price_level(inflation, 1));
        assert_eq!(series[2], dec!(110) / price_level(inflation, 2));
        assert_eq!(series[3], dec!(77));
    }

    /// A line growing at a set rate a year, from the first sum and a deposit
    /// each month, in today's money.
    #[test]
    fn the_line_at_a_yearly_rate_grows_by_that_rate() {
        let line = grown_at(dec!(1000), dec!(0), 12, Percent(dec!(0)), Percent(dec!(10)));
        assert_eq!(line[0], dec!(1000));
        assert!((line[12] - dec!(1100)).abs() < dec!(0.01), "{}", line[12]);
        // One month's deposit, grown by that month's share of the rate.
        let line = grown_at(dec!(0), dec!(100), 1, Percent(dec!(0)), Percent(dec!(10)));
        assert!(line[1] > dec!(100) && line[1] < dec!(101), "{}", line[1]);
    }
}
