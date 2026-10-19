//! How sure the answer is (REDESIGN.md §2.3): the comparison run again under
//! plausible versions of what the site can't know (the Bank of Israel's
//! average rate, inflation, the index's return, a provident fund's fee),
//! and the noise margins that say when two options are a tie.
//!
//! The versions are the owner's (2026-10-19): the rate ±0.5 point around the
//! one in use, inflation at 1% and 3%, the index at 5% and 10% a year and at
//! the historical worst and median for the saver's own deposits, and the
//! funds at their average fee and at the cheapest company's.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::Percent;
use crate::history::DepositsBand;

use super::Settings;

/// Between kinds (a fund against a bond, an index against a safe kind): a
/// difference below the larger of this...
pub const KIND_MARGIN_AT_LEAST: Decimal = dec!(1000);
/// ... and this share of the deposits is noise.
pub const KIND_MARGIN_SHARE: Decimal = dec!(0.01);
/// Between providers of the same product: a difference below the larger of
/// this...
pub const PROVIDER_MARGIN_AT_LEAST: Decimal = dec!(200);
/// ... and this share of the deposits is a tie.
pub const PROVIDER_MARGIN_SHARE: Decimal = dec!(0.0025);
/// How far the Bank of Israel's average rate is moved each way.
pub const RATE_STEP: Decimal = dec!(0.5);
/// The inflations tried besides the one in use.
pub const INFLATIONS: [Decimal; 2] = [dec!(1), dec!(3)];
/// The index's yearly returns tried besides the one in use.
pub const INDEX_RETURNS: [Decimal; 2] = [dec!(5), dec!(10)];

/// The noise margin between kinds, for deposits of `deposited` (today's ₪).
#[must_use]
pub fn kind_margin(deposited: Decimal) -> Decimal {
    KIND_MARGIN_AT_LEAST.max(deposited * KIND_MARGIN_SHARE)
}

/// The noise margin between providers of one product.
#[must_use]
pub fn provider_margin(deposited: Decimal) -> Decimal {
    PROVIDER_MARGIN_AT_LEAST.max(deposited * PROVIDER_MARGIN_SHARE)
}

/// Which fee a provident fund, a study fund or a savings policy charges in
/// a version: the companies' average, or the cheapest company's (what a
/// saver who negotiates can get).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FundFee {
    Average,
    Negotiated,
}

/// What a version changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Varies {
    /// The settings as they are.
    Nothing,
    Rate,
    Inflation,
    IndexReturn,
    /// The index at the worst or the median the history gives these deposits.
    History,
    FundFee,
}

/// One version of the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Version {
    pub varies: Varies,
    pub rate: Percent,
    pub inflation: Percent,
    pub index_return: Percent,
    pub fund_fee: FundFee,
}

/// The versions to run: the settings first, then each change alone.
pub(super) fn versions(settings: &Settings, history: Option<&DepositsBand>) -> Vec<Version> {
    let base = Version {
        varies: Varies::Nothing,
        rate: settings.rate,
        inflation: settings.inflation,
        index_return: settings.index_return,
        fund_fee: FundFee::Average,
    };
    let mut all = vec![base];
    for rate in [settings.rate.0 - RATE_STEP, settings.rate.0 + RATE_STEP] {
        all.push(Version {
            varies: Varies::Rate,
            rate: Percent(rate.max(Decimal::ZERO)),
            ..base
        });
    }
    for inflation in INFLATIONS {
        if inflation != settings.inflation.0 {
            all.push(Version {
                varies: Varies::Inflation,
                inflation: Percent(inflation),
                ..base
            });
        }
    }
    for index_return in INDEX_RETURNS {
        if index_return != settings.index_return.0 {
            all.push(Version {
                varies: Varies::IndexReturn,
                index_return: Percent(index_return),
                ..base
            });
        }
    }
    if let Some(history) = history {
        for index_return in [history.worst, history.median] {
            all.push(Version {
                varies: Varies::History,
                index_return,
                ..base
            });
        }
    }
    all.push(Version {
        varies: Varies::FundFee,
        fund_fee: FundFee::Negotiated,
        ..base
    });
    all
}

/// How one option stands against another across the versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Standing {
    /// Ahead by more than the margin in every version.
    Clear,
    /// Within the margin as things are.
    Tie,
    /// Ahead as things are, but within the margin or behind in some version.
    Flips,
    /// Behind as things are.
    Behind,
}

/// `pairs` are (this option, the other) in each version, the first being
/// the settings as they are.
pub(super) fn standing(pairs: &[(Decimal, Decimal)], margin: Decimal) -> Standing {
    let Some(&(first, other)) = pairs.first() else {
        return Standing::Tie;
    };
    if (first - other).abs() <= margin {
        Standing::Tie
    } else if first < other {
        Standing::Behind
    } else if pairs.iter().all(|&(this, that)| this - that > margin) {
        Standing::Clear
    } else {
        Standing::Flips
    }
}

/// Whether the order of the safe options changes within the versions: it
/// does across them (`Flips`), or another option's figure lies within what
/// the money market fund leaves at the rates tried (`fund`, low and high),
/// so the rate at which they trade places lies within those rates. A tie as
/// things are can flip too: being within the margin says nothing about which
/// side of the switch point the rate is.
pub(super) fn order_flips(
    standing: Standing,
    fund: Option<(Decimal, Decimal)>,
    others: &[Decimal],
) -> bool {
    standing == Standing::Flips
        || fund
            .is_some_and(|(low, high)| others.iter().any(|other| low <= *other && *other <= high))
}

/// The Bank of Israel's average rate at which `ahead(rate)` (one option less
/// the other) changes sign, between 0% and 10%, to a tenth of a point: the
/// switch point said under a decision. `None` when it doesn't change sign
/// there.
pub(super) fn switch_rate(ahead: impl Fn(Percent) -> Option<Decimal>) -> Option<Percent> {
    let (mut low, mut high) = (Decimal::ZERO, dec!(10));
    let at_low = ahead(Percent(low))?;
    let at_high = ahead(Percent(high))?;
    if at_low.is_sign_negative() == at_high.is_sign_negative() {
        return None;
    }
    // Twelve halvings take ten points to a fortieth of one.
    for _ in 0..12 {
        let middle = (low + high) / Decimal::TWO;
        let at_middle = ahead(Percent(middle))?;
        if at_middle.is_sign_negative() == at_low.is_sign_negative() {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some(Percent(((low + high) / Decimal::TWO).round_dp(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_margins_take_the_larger_part() {
        assert_eq!(kind_margin(dec!(10_000)), dec!(1000));
        assert_eq!(kind_margin(dec!(200_000)), dec!(2000));
        assert_eq!(provider_margin(dec!(10_000)), dec!(200));
        assert_eq!(provider_margin(dec!(200_000)), dec!(500));
    }

    #[test]
    fn a_standing_needs_every_version() {
        let margin = dec!(10);
        assert_eq!(standing(&[(dec!(100), dec!(50))], margin), Standing::Clear);
        assert_eq!(standing(&[(dec!(100), dec!(95))], margin), Standing::Tie);
        assert_eq!(standing(&[(dec!(50), dec!(100))], margin), Standing::Behind);
        assert_eq!(
            standing(&[(dec!(100), dec!(50)), (dec!(100), dec!(120))], margin),
            Standing::Flips
        );
        assert_eq!(
            standing(&[(dec!(100), dec!(50)), (dec!(100), dec!(95))], margin),
            Standing::Flips
        );
    }

    #[test]
    fn a_switch_point_within_the_rates_tried_flips_the_order() {
        // The fund leaves 100 to 120 at the rates tried.
        let fund = Some((dec!(100), dec!(120)));
        // The apartment's: a tie as things are, the bond inside the bar.
        assert!(order_flips(Standing::Tie, fund, &[dec!(110)]));
        assert!(order_flips(Standing::Clear, fund, &[dec!(130), dec!(100)]));
        assert!(order_flips(Standing::Tie, fund, &[dec!(120)]));
        assert!(!order_flips(Standing::Tie, fund, &[dec!(121), dec!(99)]));
        assert!(!order_flips(Standing::Tie, None, &[dec!(110)]));
        assert!(order_flips(Standing::Flips, None, &[]));
    }

    #[test]
    fn the_switch_point_is_where_the_sign_changes() {
        let at = switch_rate(|rate| Some(rate.0 - dec!(3.6))).unwrap();
        assert_eq!(at, Percent(dec!(3.6)));
        assert_eq!(switch_rate(|_| Some(dec!(1))), None);
    }
}
