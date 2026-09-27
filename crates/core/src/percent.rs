use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A percentage, written the way tariffs write it: `Percent(dec!(0.15))` is
/// 0.15%, not 15%.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Percent(pub Decimal);

impl Percent {
    /// This percentage of `amount`: 0.15% of 1,000 is 1.5.
    #[must_use]
    pub fn of(self, amount: Decimal) -> Decimal {
        amount * self.0 / Decimal::ONE_HUNDRED
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }
}

/// "0.15%"
impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0.normalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn of_and_display() {
        let percent = Percent(dec!(0.150));
        assert_eq!(percent.of(dec!(1000)), dec!(1.5));
        assert_eq!(percent.to_string(), "0.15%");
    }
}
