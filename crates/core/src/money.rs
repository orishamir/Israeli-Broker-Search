//! Money and exchange rates, from `rusty-money`. The tariffs only use
//! shekels, dollars and euros.

use rust_decimal::Decimal;
pub use rusty_money::iso::{self, Currency};

pub type Money = rusty_money::Money<'static, Currency>;

/// Rates between shekels, dollars and euros.
///
/// The European Central Bank's daily reference rates, which the app shows by
/// default, differ from the Bank of Israel's representative rate (Sha'ar
/// Yatzig) the tariffs refer to by a fraction of a percent: a few agorot on
/// these fees.
pub struct ExchangeRates(rusty_money::Exchange<'static, Currency>);

impl ExchangeRates {
    /// From what a dollar and a euro cost in shekels. Both must be positive.
    pub fn new(ils_per_usd: Decimal, ils_per_eur: Decimal) -> Result<Self, InvalidRate> {
        if ils_per_usd <= Decimal::ZERO || ils_per_eur <= Decimal::ZERO {
            return Err(InvalidRate);
        }
        let usd_per_eur = ils_per_eur / ils_per_usd;
        // rusty-money needs every pair's rate, in both directions.
        let rates = [
            (iso::USD, iso::ILS, ils_per_usd),
            (iso::EUR, iso::ILS, ils_per_eur),
            (iso::EUR, iso::USD, usd_per_eur),
        ];
        let mut exchange = rusty_money::Exchange::new();
        for (from, to, rate) in rates {
            for (from, to, rate) in [(from, to, rate), (to, from, Decimal::ONE / rate)] {
                let rate = rusty_money::ExchangeRate::new(from, to, rate)
                    .expect("the currencies of a pair differ");
                exchange.set_rate(&rate);
            }
        }
        Ok(Self(exchange))
    }

    /// `money` in `currency`.
    ///
    /// # Panics
    ///
    /// Never for shekels, dollars and euros, the only currencies the tariffs
    /// use: rates between all of them are always set.
    #[must_use]
    pub fn convert(&self, money: Money, currency: &'static Currency) -> Money {
        // rusty-money has no rate from a currency to itself.
        if money.currency() == currency {
            return money;
        }
        money
            .exchange_to(currency, &self.0)
            .expect("rates are set between shekels, dollars and euros")
    }
}

/// An exchange rate that isn't a positive number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("exchange rates must be positive")]
pub struct InvalidRate;

/// An amount in shekels: `ils(dec!(3.5))` is ₪3.50.
#[must_use]
pub fn ils(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::ILS)
}

/// An amount in US dollars: `usd(dec!(24))` is $24.
#[must_use]
pub fn usd(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::USD)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn converts_between_any_two_currencies() {
        // €1 = $1.25 = ₪4.625
        let rates = ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap();
        assert_eq!(rates.convert(usd(dec!(10)), iso::ILS), ils(dec!(37)));
        assert_eq!(
            rates.convert(ils(dec!(37)), iso::USD).amount().round_dp(10),
            dec!(10)
        );
        let euros = rates.convert(usd(dec!(25)), iso::EUR);
        assert_eq!(euros, Money::from_decimal(dec!(20), iso::EUR));
        assert_eq!(rates.convert(ils(dec!(5)), iso::ILS), ils(dec!(5)));
    }

    #[test]
    fn rates_must_be_positive() {
        assert_eq!(
            ExchangeRates::new(dec!(0), dec!(4)).err(),
            Some(InvalidRate)
        );
    }
}
