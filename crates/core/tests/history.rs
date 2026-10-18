//! The S&P 500's history (`policies/sp500-history.py`): the bands must be
//! ordered and the share that ended lower must fall as the years grow.

use broker_fees::history::{Band, LONGEST_YEARS, SHORTEST_YEARS, band};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn at(years: u32) -> Band {
    band(Decimal::from(years)).expect("a listed horizon")
}

#[test]
fn every_band_is_ordered_and_its_share_is_a_share() {
    for years in SHORTEST_YEARS..=LONGEST_YEARS {
        let b = at(years);
        assert!(b.worst <= b.p10 && b.p10 <= b.median, "{years}: {b:?}");
        assert!(b.median <= b.p90 && b.p90 <= b.best, "{years}: {b:?}");
        assert!(
            (Decimal::ZERO..=Decimal::ONE).contains(&b.share_lower),
            "{years}"
        );
        assert!(b.lower <= b.windows, "{years}");
        // The data's months, less the stretch itself.
        assert_eq!(b.windows, 1170 - years * 12, "{years}");
    }
}

#[test]
fn waiting_longer_does_not_make_ending_lower_likelier() {
    assert!(at(20).share_lower <= at(5).share_lower);
    assert!(at(10).share_lower <= at(1).share_lower);
}

/// The data says 20 years never ended lower, nominal: the worst 20-year
/// stretch still gained about 2% a year. Five years did lose.
#[test]
fn twenty_years_never_lost_in_dollars_but_five_did() {
    let b = at(20);
    assert!(b.worst.0 > Decimal::ZERO, "{b:?}");
    assert_eq!(b.lower, 0);
    assert!(at(5).lower > 0);
    assert!(at(5).worst.0 < Decimal::ZERO);
}

#[test]
fn outside_one_to_thirty_years_it_says_nothing() {
    assert!(band(dec!(0.9)).is_none());
    assert!(band(dec!(30.5)).is_none());
    assert!(band(dec!(1)).is_some() && band(dec!(30)).is_some());
}

#[test]
fn between_years_it_lies_between_them() {
    let (a, mid, c) = (at(5), band(dec!(5.5)).expect("inside"), at(6));
    let (lo, hi) = (a.worst.min(c.worst), a.worst.max(c.worst));
    assert!(lo <= mid.worst && mid.worst <= hi);
    assert_eq!(mid.windows, 1170 - 66);
    // A whole number gives the listed row.
    assert_eq!(band(dec!(5.0)).map(|b| b.lower), Some(a.lower));
}

#[test]
fn it_says_which_months_it_is_cut_from() {
    let b = at(10);
    assert!(b.from < b.to);
    assert_eq!(b.from.year(), 1926);
}
