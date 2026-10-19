//! What the S&P 500 did over every stretch of years since 1926: the worst,
//! the best and the middle of the yearly returns, and in how many stretches
//! the money ended lower than it started. The guide shows it beside a goal
//! ("if it's down then, you wait") so a saver sees what a year count has
//! meant before, and says it is the past.
//!
//! Everything is in nominal dollars, before any fund's fee or tax: a fund
//! bought in shekels also moves with the exchange rate. The numbers are
//! worked out by `policies/sp500-history.py` from Robert Shiller's monthly
//! data (`sp500.rs` is its output); `policies/sources.md` has the method.

mod sp500;
mod sp500_months;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use time::Date;

use crate::{Page, Percent, Text};

/// One whole number of years, as the script writes it.
struct Row {
    years: u32,
    worst: Decimal,
    p10: Decimal,
    median: Decimal,
    p90: Decimal,
    best: Decimal,
    /// Stretches that ended lower than they started.
    lower: u32,
    windows: u32,
}

/// The shortest and longest stretch the history answers for. Under a year,
/// a yearly rate of a few months' swing means nothing; over 30 years the
/// data holds a few separate experiences, so it says nothing it can stand
/// behind.
pub const SHORTEST_YEARS: u32 = 1;
pub const LONGEST_YEARS: u32 = 30;

/// What every stretch of `years` years since the history's start did, as a
/// yearly rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    pub years: Decimal,
    pub worst: Percent,
    pub p10: Percent,
    pub median: Percent,
    pub p90: Percent,
    pub best: Percent,
    /// The share of stretches that ended lower than they started, 0 to 1.
    pub share_lower: Decimal,
    /// How many stretches ended lower, and how many there were: "in 52 of
    /// 1,086". They overlap (each starts a month after the last), so they
    /// are far fewer separate experiences than their number.
    pub lower: u32,
    pub windows: u32,
    /// The first and last month of the data the stretches are cut from.
    pub from: Date,
    pub to: Date,
}

/// The band for a stretch of `years` years, or `None` outside 1 to 30
/// years. Between whole years the rates and the share are interpolated in a
/// straight line; the number of stretches is exact, and the number that
/// ended lower follows from the share.
#[must_use]
pub fn band(years: Decimal) -> Option<Band> {
    if years < Decimal::from(SHORTEST_YEARS) || years > Decimal::from(LONGEST_YEARS) {
        return None;
    }
    let rows = sp500::rows();
    let whole = years.floor();
    let whole_years = whole.to_u32()?;
    let low = rows.iter().find(|row| row.years == whole_years)?;
    // At 30 years exactly there is no row above; the fraction is 0 then.
    let high = rows
        .iter()
        .find(|row| row.years == whole_years + 1)
        .unwrap_or(low);
    let fraction = years - whole;
    // `a` at the row below, `b` at the row above.
    let mix = |a: Decimal, b: Decimal| a + (b - a) * fraction;

    let months = (years * Decimal::from(12)).round().to_u32()?;
    // A stretch of n months ends in each month from the n-th on.
    let windows = sp500::MONTHS - months;
    let share_lower = mix(share(low), share(high));
    let lower = if fraction.is_zero() {
        low.lower
    } else {
        (share_lower * Decimal::from(windows)).round().to_u32()?
    };
    Some(Band {
        years,
        worst: Percent(mix(low.worst, high.worst)),
        p10: Percent(mix(low.p10, high.p10)),
        median: Percent(mix(low.median, high.median)),
        p90: Percent(mix(low.p90, high.p90)),
        best: Percent(mix(low.best, high.best)),
        share_lower,
        lower,
        windows,
        from: sp500::FROM,
        to: sp500::TO,
    })
}

/// What every stretch of the history did to one saver's own deposits: a sum
/// at the start and the same amount at the start of every month, each
/// meeting the index at its own price. The yearly rates are money-weighted:
/// the one flat rate that would end with the same money for these deposits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositsBand {
    pub months: u32,
    pub worst: Percent,
    pub median: Percent,
    pub best: Percent,
    /// How many stretches ended with less than was put in, and how many
    /// there were.
    pub lower: u32,
    pub windows: u32,
    pub from: Date,
    pub to: Date,
}

/// [`DepositsBand`] for `now` and `monthly` over `months`, or `None` outside
/// 1 to 30 years (as [`band`]) or for no money at all.
///
/// Each stretch's end is worked out in one step from running sums of the
/// index's inverse, so every stretch costs the same however long it is; only
/// the three stretches reported are turned into a rate.
#[must_use]
pub fn deposits_band(now: Decimal, monthly: Decimal, months: u32) -> Option<DepositsBand> {
    if !(SHORTEST_YEARS * 12..=LONGEST_YEARS * 12).contains(&months) {
        return None;
    }
    let now = now.to_f64()?;
    let monthly = monthly.to_f64()?;
    let put_in = now + monthly * f64::from(months);
    if put_in <= 0.0 {
        return None;
    }
    let index = &sp500_months::INDEX;
    let n = months as usize;
    // inverse[j] is the sum of 1/index[i] for i < j.
    let mut inverse = Vec::with_capacity(index.len() + 1);
    inverse.push(0.0_f64);
    for value in index {
        inverse.push(inverse.last().copied().unwrap_or(0.0) + 1.0 / value);
    }
    // What the deposits end with, as a multiple of what was put in.
    let mut ends: Vec<f64> = (0..index.len() - n)
        .map(|start| {
            let end = index[start + n];
            let monthly_part = monthly * end * (inverse[start + n] - inverse[start]);
            (now * end / index[start] + monthly_part) / put_in
        })
        .collect();
    let lower = u32::try_from(ends.iter().filter(|&&end| end < 1.0).count()).ok()?;
    let windows = u32::try_from(ends.len()).ok()?;
    ends.sort_by(f64::total_cmp);
    let rate = |multiple: f64| flat_rate(now, monthly, months, multiple * put_in);
    Some(DepositsBand {
        months,
        worst: rate(*ends.first()?)?,
        median: rate(ends[ends.len() / 2])?,
        best: rate(*ends.last()?)?,
        lower,
        windows,
        from: sp500::FROM,
        to: sp500::TO,
    })
}

/// The flat yearly rate at which `now` and `monthly` (each put in at the
/// start of its month) end at `end` after `months`. Found by halving: the end
/// grows with the rate.
fn flat_rate(now: f64, monthly: f64, months: u32, end: f64) -> Option<Percent> {
    let ended = |yearly: f64| {
        let growth = (1.0 + yearly).powf(1.0 / 12.0);
        let mut total = now * growth.powf(f64::from(months));
        let mut factor = 1.0;
        for _ in 0..months {
            factor *= growth;
            total += monthly * factor;
        }
        total
    };
    let (mut low, mut high) = (-0.99_f64, 3.0_f64);
    for _ in 0..60 {
        let middle = f64::midpoint(low, high);
        if ended(middle) < end {
            low = middle;
        } else {
            high = middle;
        }
    }
    let percent = Decimal::try_from(f64::midpoint(low, high) * 100.0).ok()?;
    Some(Percent(percent.round_dp(2)))
}

fn share(row: &Row) -> Decimal {
    Decimal::from(row.lower) / Decimal::from(row.windows)
}

/// Where the history comes from, to link beside it.
#[must_use]
pub fn source() -> Page {
    Page {
        name: Text::new(
            "Robert Shiller's monthly S&P 500 data, Yale",
            "נתוני S&P 500 החודשיים של רוברט שילר, ייל",
        ),
        url: "http://www.econ.yale.edu/~shiller/data.htm".to_owned(),
    }
}
