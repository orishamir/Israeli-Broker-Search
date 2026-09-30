//! Where the money is kept, in the law's eyes: a brokerage account, a
//! provident fund for investment (Kupat Gemel Le'hashkaa), a study fund
//! (Keren Hishtalmut), a savings policy (Polisat Hisachon). The vehicle
//! decides what no price list does: the tax on the gain at the end, how
//! much may be deposited in a year, and when the money can come out. Fees
//! belong to the [`Plan`](crate::Plan); these rules are the same whoever
//! runs the money.
//!
//! The rules are the law's for the 2026 tax year; `policies/sources.md` says
//! where each comes from, and `policies/not-modeled.md` what's left out.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use time::Date;
use time::macros::date;

use crate::{ManagementFee, Percent};

/// When the rules were last checked against the law and the ceilings
/// published for the year.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 09 - 30)
}

/// The most a provident fund for investment may charge, by regulation: 4% of
/// each deposit and 1.05% of the balance a year. What savers pay is agreed
/// with the fund, and is far less.
pub const LARGEST_GEMEL_FEE: ManagementFee = ManagementFee {
    of_deposits: Percent(dec!(4)),
    of_balance: Percent(dec!(1.05)),
};

/// The most a study fund may charge, by the same regulations: 2% of the
/// balance a year, and nothing from deposits.
pub const LARGEST_STUDY_FUND_FEE: ManagementFee = ManagementFee {
    of_deposits: Percent(dec!(0)),
    of_balance: Percent(dec!(2)),
};

/// What kind of account a plan is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
pub enum Vehicle {
    /// An account at a bank or an investment house: the saver buys and
    /// sells securities themselves.
    Brokerage,
    /// A provident fund for investment (Kupat Gemel Le'hashkaa): open to
    /// anyone, withdrawn at any time, up to a ceiling of deposits a year.
    InvestmentGemel,
    /// A study fund (Keren Hishtalmut): the gains on deposits up to a yearly
    /// amount aren't taxed, once six years have passed. Counted as a
    /// self-employed saver's, who can open one and deposit as they like; an
    /// employee's depends on the employer.
    StudyFund,
    /// An insurer's savings policy (Polisat Hisachon): like the fund, with
    /// no ceiling and no pension at the end.
    SavingsPolicy,
}

impl Vehicle {
    /// What plans saved before there were vehicles are: every one was a
    /// broker's. Only for reading those saves; a plan in code names its
    /// vehicle.
    #[must_use]
    pub fn of_older_saves() -> Self {
        Vehicle::Brokerage
    }

    /// The law's rules for this vehicle.
    #[must_use]
    pub fn rules(self) -> Rules {
        let a_quarter_of_the_real_gain = GainsTax::OfRealGain(Percent(dec!(25)));
        match self {
            Vehicle::Brokerage | Vehicle::SavingsPolicy => Rules {
                deposit_ceiling: None,
                tax_free_deposits: None,
                open_after_years: None,
                lump_sum: a_quarter_of_the_real_gain,
                pension: None,
            },
            Vehicle::StudyFund => Rules {
                deposit_ceiling: None,
                tax_free_deposits: Some(dec!(20566)),
                open_after_years: Some(6),
                lump_sum: a_quarter_of_the_real_gain,
                pension: None,
            },
            Vehicle::InvestmentGemel => Rules {
                deposit_ceiling: Some(dec!(83641)),
                tax_free_deposits: None,
                open_after_years: None,
                lump_sum: a_quarter_of_the_real_gain,
                pension: Some(Pension {
                    from_age: 60,
                    tax: GainsTax::Exempt,
                }),
            },
        }
    }

    /// Whether a manager invests each deposit as it arrives. The saver
    /// places no orders, so how often they'd buy at a broker doesn't apply.
    #[must_use]
    pub fn invests_for_you(self) -> bool {
        match self {
            Vehicle::Brokerage => false,
            Vehicle::InvestmentGemel | Vehicle::StudyFund | Vehicle::SavingsPolicy => true,
        }
    }
}

/// What the law says about a [`Vehicle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    /// The most that may be deposited in a calendar year, in ₪ of the year
    /// checked, if there's a limit. The law raises it with prices every
    /// January.
    pub deposit_ceiling: Option<Decimal>,
    /// The yearly deposits whose gains aren't taxed at all, in ₪ of the
    /// year checked and raised with prices like the ceiling, where the
    /// vehicle has such a part. More may be deposited: the gains on the
    /// rest are taxed by `lump_sum`.
    pub tax_free_deposits: Option<Decimal>,
    /// How many years from the first deposit until the money can be taken
    /// out on these terms. Sooner, it's taxed as income, which the app
    /// doesn't work out: the plan isn't offered.
    pub open_after_years: Option<u32>,
    /// The tax on taking everything out at once.
    pub lump_sum: GainsTax,
    /// Taking it as a monthly pension instead, where the vehicle has that
    /// way out.
    pub pension: Option<Pension>,
}

impl Rules {
    /// The tax for a saver who is `age` at the end and takes the money out
    /// by `withdrawal`. Someone who wanted a pension but is too young for
    /// one, or whose vehicle pays none, takes a lump sum: the money is
    /// counted at the end of the period, not left to wait.
    #[must_use]
    pub fn tax_at(&self, withdrawal: Withdrawal, age: u32) -> GainsTax {
        match (withdrawal, self.pension) {
            (Withdrawal::Pension, Some(pension)) if age >= pension.from_age => pension.tax,
            _ => self.lump_sum,
        }
    }

    /// The deposit ceiling in the year that starts `years` from now: today's,
    /// raised with prices at `inflation` a year.
    #[must_use]
    pub fn deposit_ceiling_in(&self, years: u32, inflation: Percent) -> Option<Decimal> {
        self.deposit_ceiling
            .map(|ceiling| raised_with_prices(ceiling, years, inflation))
    }

    /// The deposits whose gains aren't taxed, in the year that starts
    /// `years` from now.
    #[must_use]
    pub fn tax_free_deposits_in(&self, years: u32, inflation: Percent) -> Option<Decimal> {
        self.tax_free_deposits
            .map(|amount| raised_with_prices(amount, years, inflation))
    }
}

/// Today's `amount` after `years` of `inflation`: the law's ceilings follow
/// the price index.
fn raised_with_prices(amount: Decimal, years: u32, inflation: Percent) -> Decimal {
    let yearly = Decimal::ONE + inflation.of(Decimal::ONE);
    (0..years).fold(amount, |amount, _| amount * yearly)
}

/// A monthly pension as the way out of a vehicle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pension {
    /// The youngest the saver can be to take it.
    pub from_age: u32,
    pub tax: GainsTax,
}

/// The tax on what an investment gained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainsTax {
    Exempt,
    /// This share of the real gain: what came out, less what went in raised
    /// by how much prices rose since. Inflation alone isn't taxed.
    OfRealGain(Percent),
}

impl GainsTax {
    /// The tax, in ₪, on `real_gain`. A loss pays nothing.
    #[must_use]
    pub fn on(self, real_gain: Decimal) -> Decimal {
        match self {
            GainsTax::Exempt => Decimal::ZERO,
            GainsTax::OfRealGain(rate) => rate.of(real_gain.max(Decimal::ZERO)),
        }
    }
}

/// How the saver takes the money out at the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Withdrawal {
    /// All at once.
    LumpSum,
    /// As a monthly pension (Kitzba), where the vehicle pays one and the
    /// saver is old enough.
    Pension,
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn a_quarter_of_the_gain_and_nothing_on_a_loss() {
        let tax = GainsTax::OfRealGain(Percent(dec!(25)));
        assert_eq!(tax.on(dec!(200000)), dec!(50000));
        assert_eq!(tax.on(dec!(0)), dec!(0));
        assert_eq!(tax.on(dec!(-1000)), dec!(0));
        assert_eq!(GainsTax::Exempt.on(dec!(200000)), dec!(0));
    }

    /// The fund's pension is tax-free from 60, and only as a pension: a lump
    /// sum at 60 is taxed, and so is someone of 59 who wanted the pension.
    #[test]
    fn the_funds_pension_is_exempt_from_60() {
        let rules = Vehicle::InvestmentGemel.rules();
        assert_eq!(rules.tax_at(Withdrawal::Pension, 60), GainsTax::Exempt);
        assert_eq!(rules.tax_at(Withdrawal::Pension, 59), rules.lump_sum);
        assert_eq!(rules.tax_at(Withdrawal::LumpSum, 60), rules.lump_sum);
        assert_ne!(rules.lump_sum, GainsTax::Exempt);
    }

    /// A broker, a study fund and an insurer pay no pension: asking for one
    /// changes nothing, at any age.
    #[test]
    fn only_the_fund_has_a_pension() {
        for vehicle in [
            Vehicle::Brokerage,
            Vehicle::StudyFund,
            Vehicle::SavingsPolicy,
        ] {
            let rules = vehicle.rules();
            assert_eq!(rules.pension, None);
            assert_eq!(rules.tax_at(Withdrawal::Pension, 70), rules.lump_sum);
        }
    }

    /// ₪83,641 this year; at 2% inflation ₪85,313.82 next year and
    /// 83,641 × 1.02² = ₪87,020.0964 the year after.
    #[test]
    fn the_ceiling_rises_with_prices() {
        let rules = Vehicle::InvestmentGemel.rules();
        let in_year = |years| rules.deposit_ceiling_in(years, Percent(dec!(2)));
        assert_eq!(in_year(0), Some(dec!(83641)));
        assert_eq!(in_year(1), Some(dec!(85313.82)));
        assert_eq!(in_year(2), Some(dec!(87020.0964)));
        assert_eq!(
            Vehicle::Brokerage
                .rules()
                .deposit_ceiling_in(5, Percent(dec!(2))),
            None
        );
    }

    /// A study fund takes any deposit; the gains on ₪20,566 a year aren't
    /// taxed, an amount that rises with prices: 20,566 × 1.02 = ₪20,977.32
    /// the year after. Only it has such a part, and only it is locked.
    #[test]
    fn the_study_funds_tax_free_part_rises_with_prices() {
        let rules = Vehicle::StudyFund.rules();
        assert_eq!(rules.deposit_ceiling, None);
        assert_eq!(rules.open_after_years, Some(6));
        let in_year = |years| rules.tax_free_deposits_in(years, Percent(dec!(2)));
        assert_eq!(in_year(0), Some(dec!(20566)));
        assert_eq!(in_year(1), Some(dec!(20977.32)));
        for vehicle in Vehicle::iter().filter(|&vehicle| vehicle != Vehicle::StudyFund) {
            let rules = vehicle.rules();
            assert_eq!(rules.tax_free_deposits, None, "{vehicle:?}");
            assert_eq!(rules.open_after_years, None, "{vehicle:?}");
        }
    }
}
