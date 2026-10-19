//! The monthly pension's range, against the payers' published tables.

use broker_fees::annuity::{self, OLDEST, YOUNGEST};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// ₪500,000 at 60 in 2030, so the birth year is 1970: a column of both
/// pension funds' tables, read as printed.
/// - Lowest coefficient: Clal Pensia Mashlima, a man of 60 born in 1970,
///   213.96 (Meitav prints 214.91 for him; women's are higher).
/// - Highest: Harel's woman of 60, 232.77 in 2024, growing 0.163% a year:
///   232.77 × (1 + 0.00163 × 6) = 232.77 + 2.2764906 = 235.0464906
///   (Harel's man: 226.70 × (1 + 0.00133 × 6) = 228.509066).
/// - So 500,000 / 235.0464906 ≈ ₪2,127.24 to 500,000 / 213.96 ≈ ₪2,336.89.
#[test]
fn a_sum_at_60_in_2030_by_hand() {
    let estimate = annuity::monthly(dec!(500_000), 60, 2030).unwrap();
    assert_eq!(estimate.high, dec!(500_000) / dec!(213.96));
    assert_eq!(estimate.low, dec!(500_000) / dec!(235.0464906));
    assert_eq!(estimate.high.round_dp(2), dec!(2336.89));
    assert_eq!(estimate.low.round_dp(2), dec!(2127.24));
}

/// Between two columns the coefficient runs straight: a man of 67 born in
/// 1992 (starting 2059) is two fifths of the way from Clal's 1990 column
/// (194.99) to its 1995 one (195.66): 194.99 + 0.67 × 2 / 5 = 195.258. That
/// is the lowest coefficient there (Meitav: 195.47 + 0.66 × 0.4 = 195.734).
#[test]
fn between_columns_the_coefficient_runs_straight() {
    let estimate = annuity::monthly(dec!(1_000_000), 67, 2059).unwrap();
    assert_eq!(estimate.high, dec!(1_000_000) / dec!(195.258));
}

#[test]
fn outside_the_tables_there_is_no_estimate() {
    assert!(annuity::monthly(dec!(100_000), YOUNGEST - 1, 2040).is_none());
    assert!(annuity::monthly(dec!(100_000), OLDEST + 1, 2040).is_none());
}

fn every_case() -> impl Iterator<Item = (u32, i32)> {
    (YOUNGEST..=OLDEST).flat_map(|age| (2024..=2080).map(move |year| (age, year)))
}

#[test]
fn the_low_end_is_never_above_the_high_end() {
    for (age, year) in every_case() {
        let estimate = annuity::monthly(dec!(300_000), age, year).unwrap();
        assert!(estimate.low > Decimal::ZERO, "{age} in {year}");
        assert!(estimate.low <= estimate.high, "{age} in {year}");
    }
}

/// Starting later in life pays more a month, at either end of the range.
#[test]
fn an_older_start_never_pays_less() {
    for (age, year) in every_case().filter(|(age, _)| *age < OLDEST) {
        let now = annuity::monthly(dec!(300_000), age, year).unwrap();
        let later = annuity::monthly(dec!(300_000), age + 1, year).unwrap();
        assert!(later.low >= now.low, "{age} in {year}");
        assert!(later.high >= now.high, "{age} in {year}");
    }
}

/// The later-born are expected to live longer, so the same age a year later
/// never pays more.
#[test]
fn a_later_year_never_pays_more() {
    for (age, year) in every_case().filter(|(_, year)| *year < 2080) {
        let now = annuity::monthly(dec!(300_000), age, year).unwrap();
        let next = annuity::monthly(dec!(300_000), age, year + 1).unwrap();
        assert!(next.low <= now.low, "{age} in {year}");
        assert!(next.high <= now.high, "{age} in {year}");
    }
}

#[test]
fn the_pension_is_in_proportion_to_the_sum() {
    let one = annuity::monthly(dec!(100_000), 64, 2045).unwrap();
    let two = annuity::monthly(dec!(200_000), 64, 2045).unwrap();
    assert_eq!(two.low, one.low * dec!(2));
    assert_eq!(two.high, one.high * dec!(2));
}

/// The words name the age and every payer whose table answered, and every
/// table has its page.
#[test]
fn the_assumption_names_the_age_and_the_payers() {
    let at_67 = annuity::monthly(dec!(100_000), 67, 2050).unwrap();
    for name in ["כלל פנסיה משלימה", "מיטב פנסיה כללית", "הראל", "67", "240"]
    {
        assert!(
            at_67.assumes.he.contains(name),
            "{name}: {}",
            at_67.assumes.he
        );
    }
    let at_68 = annuity::monthly(dec!(100_000), 68, 2050).unwrap();
    assert!(!at_68.assumes.he.contains("מיטב"), "{}", at_68.assumes.he);
    assert!(at_68.assumes.he.contains("87"), "{}", at_68.assumes.he);
    assert_eq!(at_67.sources.len(), 3);
    assert_eq!(at_68.sources.len(), 2);
    assert!(
        at_67
            .sources
            .iter()
            .all(|page| page.url.starts_with("https://"))
    );
}
