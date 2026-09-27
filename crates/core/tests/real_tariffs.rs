use broker_fees::tariffs::{altshuler, leumi};
use broker_fees::*;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Fixed rates so the expected numbers are exact: €1 = $1.25 = ₪4.625, so
/// $1 = ₪3.70. Not real rates.
fn rates() -> ExchangeRates {
    ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
}

fn plan(b: Broker, name: &str) -> Plan {
    b.plans.into_iter().find(|p| p.name == name).unwrap()
}

fn trade(security: Security, exchange: Exchange, shares: Decimal, value: Money) -> Trade {
    Trade {
        security,
        exchange,
        shares,
        value,
    }
}

#[test]
fn leumi_tase_minimum_and_maximum() {
    let p = plan(leumi(), "Online");
    let fee = |v| {
        p.trade_fee(
            &trade(Security::Stock, Exchange::Tlv, dec!(1), ils(v)),
            &rates(),
        )
        .unwrap()
    };
    assert_eq!(fee(dec!(1000)), ils(dec!(26))); // 0.4% = ₪4 → minimum
    assert_eq!(fee(dec!(100000)), ils(dec!(400)));
    assert_eq!(fee(dec!(2000000)), ils(dec!(6300))); // 0.4% = ₪8,000 → maximum
}

#[test]
fn altshuler_best_us_track_depends_on_share_price() {
    let fee = |track, shares, value| {
        plan(altshuler(), track)
            .trade_fee(
                &trade(Security::Etf, Exchange::Usa, shares, usd(value)),
                &rates(),
            )
            .unwrap()
    };
    // 100 shares × $50
    assert_eq!(fee("US 1¢/share", dec!(100), dec!(5000)), usd(dec!(9)));
    assert_eq!(fee("US $11 flat", dec!(100), dec!(5000)), usd(dec!(11)));
    assert_eq!(fee("US 0.15%", dec!(100), dec!(5000)), usd(dec!(9)));
    // 2,000 shares × $5
    assert_eq!(fee("US 1¢/share", dec!(2000), dec!(10000)), usd(dec!(20)));
    assert_eq!(fee("US $11 flat", dec!(2000), dec!(10000)), usd(dec!(11)));
    assert_eq!(fee("US 0.15%", dec!(2000), dec!(10000)), usd(dec!(15)));
}

#[test]
fn altshuler_tase_etf_vs_mutual_fund() {
    let p = plan(altshuler(), "US $11 flat");
    let fee = |s| {
        p.trade_fee(&trade(s, Exchange::Tlv, dec!(1), ils(dec!(1000))), &rates())
            .unwrap()
    };
    assert_eq!(fee(Security::Etf), ils(dec!(3.5)));
    assert_eq!(fee(Security::MutualFund), ils(dec!(16)));
}

#[test]
fn altshuler_new_customers() {
    let p = plan(altshuler(), "New customers");
    let fee = |security, exchange, shares, value| {
        p.trade_fee(&trade(security, exchange, shares, value), &rates())
            .unwrap()
    };
    // Tel Aviv: 0.07%, at least ₪2.90, index funds included.
    assert_eq!(
        fee(Security::Etf, Exchange::Tlv, dec!(1), ils(dec!(10000))),
        ils(dec!(7))
    );
    assert_eq!(
        fee(
            Security::MutualFund,
            Exchange::Tlv,
            dec!(1),
            ils(dec!(1000))
        ),
        ils(dec!(2.9))
    );
    // Bonds aren't in the offer: the regular 0.15%, at least ₪3.5.
    assert_eq!(
        fee(Security::Bond, Exchange::Tlv, dec!(1), ils(dec!(1000))),
        ils(dec!(3.5))
    );
    // US: 1¢ a share, at least $6.
    assert_eq!(
        fee(Security::Etf, Exchange::Usa, dec!(10), usd(dec!(5000))),
        usd(dec!(6))
    );
    assert_eq!(
        fee(Security::Etf, Exchange::Usa, dec!(1000), usd(dec!(5000))),
        usd(dec!(10))
    );

    let holdings = [Holding {
        exchange: Exchange::Tlv,
        value: ils(dec!(100000)),
    }];
    assert_eq!(p.custody_per_year(&holdings, &rates()), ils(dec!(0)));

    assert_eq!(
        p.first_deposit_warning(ils(dec!(4999))).as_deref(),
        Some("Needs a one-time deposit of at least ₪5,000")
    );
    assert_eq!(p.first_deposit_warning(ils(dec!(5000))), None);
}

#[test]
fn no_row_means_none_not_free() {
    let p = plan(altshuler(), "US $11 flat");
    let eur = Money::from_decimal(dec!(1000), iso::EUR);
    assert_eq!(
        p.trade_fee(
            &trade(Security::Etf, Exchange::Europe, dec!(10), eur),
            &rates()
        ),
        None
    );
}

#[test]
fn dollar_minimum_on_a_euro_trade() {
    // Leumi's foreign row covers Europe too, with its minimum in dollars.
    let p = plan(leumi(), "Online");
    let eur = |amount| Money::from_decimal(amount, iso::EUR);
    let fee = p.trade_fee(
        &trade(Security::Etf, Exchange::Europe, dec!(10), eur(dec!(1000))),
        &rates(),
    );
    assert_eq!(fee, Some(eur(dec!(19.20)))); // 0.3% = €3; $24 minimum = €19.20
}

#[test]
fn altshuler_monthly_minimum_dominates_small_portfolios() {
    let p = plan(altshuler(), "US $11 flat");
    let year = |v| {
        p.custody_per_year(
            &[Holding {
                exchange: Exchange::Tlv,
                value: ils(v),
            }],
            &rates(),
        )
    };
    assert_eq!(year(dec!(100000)), ils(dec!(900))); // ₪12.50 a month → ₪75 minimum
    assert_eq!(year(dec!(1000000)), ils(dec!(1500))); // ₪125 a month
}

#[test]
fn leumi_custody_by_exchange() {
    let p = plan(leumi(), "Online");
    let holdings = [
        Holding {
            exchange: Exchange::Tlv,
            value: ils(dec!(200000)),
        }, // 0.6% → ₪1,200
        Holding {
            exchange: Exchange::Usa,
            value: usd(dec!(10000)),
        }, // 0.8% of ₪37,000 → ₪296
    ];
    assert_eq!(p.custody_per_year(&holdings, &rates()), ils(dec!(1496)));
}

#[test]
fn conversion_fee_vs_spread() {
    let fee = |b, name| plan(b, name).conversion_fee(usd(dec!(1000)), &rates());
    assert_eq!(fee(altshuler(), "US $11 flat"), usd(dec!(7))); // no fee, 0.7% spread
    assert_eq!(fee(leumi(), "Online"), usd(dec!(5.76))); // 0.16% = $1.60 → minimum
}

#[test]
fn brokers_round_trip_through_json() {
    for b in [altshuler(), leumi()] {
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(serde_json::from_str::<Broker>(&json).unwrap(), b);
    }
}

#[test]
fn leumi_pepper_tase_flat_except_index_funds() {
    let p = plan(leumi(), "Pepper");
    let fee = |s| {
        p.trade_fee(&trade(s, Exchange::Tlv, dec!(1), ils(dec!(2000))), &rates())
            .unwrap()
    };
    assert_eq!(fee(Security::Etf), ils(dec!(4)));
    assert_eq!(fee(Security::MutualFund), ils(dec!(26))); // Online: 0.4% = ₪8 → minimum
}

#[test]
fn leumi_pepper_conversion() {
    let fee = |v| plan(leumi(), "Pepper").conversion_fee(usd(v), &rates());
    assert_eq!(fee(dec!(1000)), usd(dec!(3))); // 0.1% = $1 → minimum
    assert_eq!(fee(dec!(10000)), usd(dec!(10)));
}

#[test]
fn leumi_standing_order_index_fund() {
    let p = plan(leumi(), "Online, monthly standing order");
    let fee = |s, v| {
        p.trade_fee(&trade(s, Exchange::Tlv, dec!(1), ils(v)), &rates())
            .unwrap()
    };
    assert_eq!(fee(Security::MutualFund, dec!(2000)), ils(dec!(5))); // 0.225% = ₪4.50 → minimum
    assert_eq!(fee(Security::MutualFund, dec!(10000)), ils(dec!(22.5)));
    assert_eq!(fee(Security::Etf, dec!(2000)), ils(dec!(26))); // only index funds; ETFs pay Online
}

#[test]
fn leumi_online_us_etf() {
    let p = plan(leumi(), "Online");
    // Buying 10 shares of a US ETF for $5,000: 0.3% is $15, below the $24 minimum.
    assert_eq!(
        p.trade_fee(
            &trade(Security::Etf, Exchange::Usa, dec!(10), usd(dec!(5000))),
            &rates()
        ),
        Some(usd(dec!(24)))
    );
    // Holding $10,000 abroad for a year: 0.2% a quarter is 0.8% of ₪37,000.
    let holdings = [Holding {
        exchange: Exchange::Usa,
        value: usd(dec!(10000)),
    }];
    assert_eq!(p.custody_per_year(&holdings, &rates()), ils(dec!(296)));
}
