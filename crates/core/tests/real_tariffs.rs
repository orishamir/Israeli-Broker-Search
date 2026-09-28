use broker_fees::simulation::{Scenario, simulate};
use broker_fees::tariffs::{altshuler, excellence, ibi, interactive, leumi, meitav};
use broker_fees::*;

use Exchange::{Europe, Tlv, Usa};
use Security::{Bond, Etf, MutualFund, Stock};
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

/// Plan `name` of `b`, for a customer on its track called `track`.
fn on_track(b: Broker, name: &str, track: &str) -> Plan {
    let p = plan(b, name);
    let index = p.tracks.iter().position(|t| t.name == track).unwrap();
    p.on_track(index)
}

fn trade(security: Security, exchange: Exchange, shares: Decimal, value: Money) -> Trade {
    Trade {
        security,
        exchange,
        shares,
        value,
    }
}

/// What `p` charges to trade `shares` of `security` on `exchange`, worth
/// `value`; `None` if it lists no price.
fn trade_fee(
    p: &Plan,
    security: Security,
    exchange: Exchange,
    shares: Decimal,
    value: Money,
) -> Option<Money> {
    p.trade_fee(&trade(security, exchange, shares, value), &rates())
}

/// A year of custody on `value` of one security.
fn custody_year(p: &Plan, security: Security, exchange: Exchange, value: Money) -> Money {
    let holding = Holding {
        security,
        exchange,
        value,
    };
    p.custody_per_year(&[holding], &rates())
}

/// The handling fee in ₪ for `month` (0 is the first), with no trade fees
/// to take off.
fn handling(p: &Plan, month: u32) -> Decimal {
    p.handling
        .map_or(Decimal::ZERO, |fee| fee.for_month(month, Decimal::ZERO))
}

fn eur(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::EUR)
}

/// Rounded to the cent. A minimum in shekels, converted to dollars, has
/// digits far past it: 1/3.7 has no exact decimal.
fn cents(money: Money) -> Money {
    Money::from_decimal(money.amount().round_dp(2), money.currency())
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
        let p = on_track(altshuler(), "Full tariff", track);
        trade_fee(&p, Etf, Usa, shares, usd(value)).unwrap()
    };
    // 100 shares × $50
    assert_eq!(fee("1¢ a share", dec!(100), dec!(5000)), usd(dec!(9))); // $1 → minimum
    assert_eq!(fee("$11 per order", dec!(100), dec!(5000)), usd(dec!(11)));
    assert_eq!(fee("0.15%", dec!(100), dec!(5000)), usd(dec!(9))); // $7.50 → minimum
    // 2,000 shares × $5
    assert_eq!(fee("1¢ a share", dec!(2000), dec!(10000)), usd(dec!(20)));
    assert_eq!(fee("$11 per order", dec!(2000), dec!(10000)), usd(dec!(11)));
    assert_eq!(fee("0.15%", dec!(2000), dec!(10000)), usd(dec!(15)));
}

#[test]
fn the_cheapest_track_is_picked() {
    let p = plan(altshuler(), "Full tariff");
    let picked = |monthly_deposit, share_price| {
        let scenario = Scenario {
            security: Etf,
            exchange: Usa,
            first_deposit: dec!(0),
            monthly_deposit,
            yearly_return: Percent(dec!(0)),
            years: 1,
            buy_every_months: 1,
            share_price,
        };
        let outcome = simulate(&p, &scenario, &rates()).unwrap();
        p.tracks[outcome.track.unwrap()].name.clone()
    };
    // ₪3,700 a month is $1,000, less the 0.7% markup: about $990 to buy
    // with. In $100 shares, 1¢ a share and 0.15% both pay their $9 minimum
    // on each purchase, but selling the year's ~118 shares (~$11,800) costs
    // $9 on the first and ~$17.70 on the second.
    assert_eq!(picked(dec!(3700), dec!(100)), "1¢ a share");
    // In 10¢ shares, 1¢ a share costs ~$90 a purchase (~9,000 shares); 0.15%
    // pays its $9 minimum, below the $11 flat fee.
    assert_eq!(picked(dec!(3700), dec!(0.1)), "0.15%");
    // ₪74,000 a month (~$19,860 to buy with) in $1 shares: 1¢ a share costs
    // ~$197, 0.15% ~$30, and the flat fee $11.
    assert_eq!(picked(dec!(74000), dec!(1)), "$11 per order");
    // Tel Aviv isn't priced by the tracks.
    let tel_aviv = Scenario {
        security: Etf,
        exchange: Tlv,
        first_deposit: dec!(0),
        monthly_deposit: dec!(1000),
        yearly_return: Percent(dec!(0)),
        years: 1,
        buy_every_months: 1,
        share_price: dec!(100),
    };
    assert_eq!(simulate(&p, &tel_aviv, &rates()).unwrap().track, None);
}

#[test]
fn altshuler_tase_etf_vs_mutual_fund() {
    let p = plan(altshuler(), "Full tariff");
    let fee = |s| trade_fee(&p, s, Tlv, dec!(1), ils(dec!(1000))).unwrap();
    assert_eq!(fee(Etf), ils(dec!(3.5))); // 0.15% = ₪1.50 → minimum
    assert_eq!(fee(MutualFund), ils(dec!(16)));
}

#[test]
fn altshuler_handling_fee() {
    // ₪80 a month from the first month; the new-customer offer waives it.
    assert_eq!(handling(&plan(altshuler(), "Full tariff"), 0), dec!(80));
    assert_eq!(handling(&plan(altshuler(), "New customers"), 0), dec!(0));
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
        security: Security::Etf,
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
    let p = on_track(altshuler(), "Full tariff", "$11 per order");
    assert_eq!(trade_fee(&p, Etf, Europe, dec!(10), eur(dec!(1000))), None);
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
    let p = plan(altshuler(), "Full tariff");
    let year = |v| custody_year(&p, Etf, Tlv, ils(v));
    assert_eq!(year(dec!(100000)), ils(dec!(900))); // ₪12.50 a month → ₪75 minimum
    assert_eq!(year(dec!(1000000)), ils(dec!(1500))); // ₪125 a month
}

#[test]
fn leumi_custody_by_exchange() {
    let p = plan(leumi(), "Online");
    let holdings = [
        Holding {
            security: Security::Etf,
            exchange: Exchange::Tlv,
            value: ils(dec!(200000)),
        }, // 0.6% → ₪1,200
        Holding {
            security: Security::Etf,
            exchange: Exchange::Usa,
            value: usd(dec!(10000)),
        }, // 0.8% of ₪37,000 → ₪296
    ];
    assert_eq!(p.custody_per_year(&holdings, &rates()), ils(dec!(1496)));
}

#[test]
fn conversion_fee_vs_spread() {
    let fee = |b, name| plan(b, name).conversion_fee(usd(dec!(1000)), &rates());
    assert_eq!(fee(altshuler(), "Full tariff"), usd(dec!(7))); // no fee, 0.7% spread
    assert_eq!(fee(leumi(), "Online"), usd(dec!(5.76))); // 0.16% = $1.60 → minimum
}

#[test]
fn every_broker_names_its_new_customer_plan() {
    for b in tariffs::all() {
        let plan = &b.plans[b.new_customer_plan];
        let expected = match b.short_name.as_str() {
            "Altshuler" => "New customers",
            "Leumi" => "Online",
            "Interactive" => "Standard",
            _ => "Typical offer",
        };
        assert_eq!(plan.name, expected, "{}", b.name);
    }
}

#[test]
fn no_caveat_is_shown_twice() {
    for b in tariffs::all() {
        for p in &b.plans {
            for exchange in Exchange::iter() {
                for security in Security::iter() {
                    let buying = Buying::any_amount(security, exchange);
                    let fees = b.describe_fees_for(p, buying, None, &rates());
                    let caveats: Vec<&str> = fees
                        .caveats
                        .iter()
                        .flat_map(|group| group.caveats.iter().map(|c| c.text.as_str()))
                        .collect();
                    let mut unique = caveats.clone();
                    unique.sort_unstable();
                    unique.dedup();
                    assert_eq!(
                        unique.len(),
                        caveats.len(),
                        "{} {}: {caveats:?}",
                        b.name,
                        p.name
                    );
                }
            }
        }
    }
}

/// A markup that isn't published understates the cost, and one stated as a
/// maximum overstates it: each plan says which beside its markup.
#[test]
fn every_markup_says_how_sure_it_is() {
    for b in tariffs::all() {
        for p in &b.plans {
            let about_markup: Vec<&Caveat> = b
                .caveats
                .iter()
                .chain(&p.caveats)
                .filter(|c| c.fee == Some(FeeKind::Markup))
                .collect();
            let says =
                |basis: &dyn Fn(&Basis) -> bool| about_markup.iter().any(|c| basis(&c.basis));
            match p.conversion.markup {
                Markup::NotPublished => assert!(
                    says(&|basis| matches!(
                        basis,
                        Basis::Assumed {
                            errs: Errs::MayCostMore { .. }
                        }
                    )),
                    "{} {}: unpublished markup without a \"may cost more\" caveat",
                    b.name,
                    p.name
                ),
                Markup::UpTo(percent) if !percent.is_zero() => assert!(
                    says(&|basis| *basis == Basis::Assumed { errs: Errs::AtMost }),
                    "{} {}: a maximum markup without an \"at most\" caveat",
                    b.name,
                    p.name
                ),
                Markup::MarketRate => assert!(
                    says(&|basis| *basis == Basis::Published),
                    "{} {}: a market-rate markup without the caveat saying who publishes it",
                    b.name,
                    p.name
                ),
                Markup::UpTo(_) | Markup::PerDollar(_) => {}
            }
        }
    }
}

/// Where an offer is silent, the full tariff's price is used, and an "at
/// most" caveat says so for every security it happens to: never cheaper
/// than can be shown, and never without saying.
#[test]
fn offers_say_where_the_full_tariff_is_used() {
    let offers = [
        (altshuler(), "New customers"),
        (excellence(), "Typical offer"),
        (ibi(), "Typical offer"),
        (meitav(), "Typical offer"),
    ];
    for (b, offer) in offers {
        let full = b.plans.iter().find(|p| p.name == "Full tariff").unwrap();
        let offer = b.plans.iter().find(|p| p.name == offer).unwrap();
        for exchange in Exchange::iter() {
            for security in Security::iter() {
                let (Some(full_row), Some(offer_row)) = (
                    full.trade_row(security, exchange),
                    offer.trade_row(security, exchange),
                ) else {
                    continue;
                };
                if full_row.price != offer_row.price {
                    continue;
                }
                let says_so = b.caveats.iter().chain(&offer.caveats).any(|c| {
                    c.applies_to(security, exchange)
                        && c.fee == Some(FeeKind::Trade)
                        && c.basis == Basis::Assumed { errs: Errs::AtMost }
                });
                assert!(
                    says_so,
                    "{} {}: {security} on {exchange} costs the full tariff without a caveat",
                    b.name, offer.name
                );
            }
        }
    }
}

#[test]
fn brokers_round_trip_through_json() {
    for b in tariffs::all() {
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
    let on_tel_aviv = |s, v| trade(s, Exchange::Tlv, dec!(1), ils(v));
    let by_standing_order = |s, v| {
        p.standing_order_row(s, Exchange::Tlv)
            .map(|row| row.price.apply(&on_tel_aviv(s, v), &rates()))
    };
    // Buying by standing order: 0.225%, at least ₪5.
    let fund = Security::MutualFund;
    assert_eq!(by_standing_order(fund, dec!(2000)), Some(ils(dec!(5)))); // 0.225% = ₪4.50 → minimum
    assert_eq!(by_standing_order(fund, dec!(10000)), Some(ils(dec!(22.5))));
    assert_eq!(by_standing_order(Security::Etf, dec!(2000)), None); // only index funds
    // Every other order, selling included, pays Online's 0.4%, at least ₪26.
    let fee = |v| p.trade_fee(&on_tel_aviv(fund, v), &rates()).unwrap();
    assert_eq!(fee(dec!(2000)), ils(dec!(26))); // 0.4% = ₪8 → minimum
    assert_eq!(fee(dec!(10000)), ils(dec!(40)));
}

#[test]
fn leumi_18_plus_conversion_is_the_better_of_two() {
    let fee = |v| plan(leumi(), "Online, 'Leumi 18+'").conversion_fee(usd(v), &rates());
    // $1,000: 18+ is 0.1% = $1 → its $7.20 minimum; online, 0.16% = $1.60 →
    // $5.76 minimum. The lower is online's.
    assert_eq!(fee(dec!(1000)), usd(dec!(5.76)));
    // $4,000: 18+ is still $7.20; online, 0.16% = $6.40.
    assert_eq!(fee(dec!(4000)), usd(dec!(6.4)));
    // $10,000: 18+ is 0.1% = $10; online, 0.16% = $16.
    assert_eq!(fee(dec!(10000)), usd(dec!(10)));
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
        security: Security::Etf,
        exchange: Exchange::Usa,
        value: usd(dec!(10000)),
    }];
    assert_eq!(p.custody_per_year(&holdings, &rates()), ils(dec!(296)));
}

// ─────────────────────────── Excellence Trade ───────────────────────────

#[test]
fn excellence_full_tariff() {
    let p = plan(excellence(), "Full tariff");
    // Every Tel Aviv security, index funds included: 0.4%, at least ₪10.
    for security in Security::iter() {
        let fee = |v| trade_fee(&p, security, Tlv, dec!(1), ils(v)).unwrap();
        assert_eq!(fee(dec!(1000)), ils(dec!(10))); // ₪4 → minimum
        assert_eq!(fee(dec!(10000)), ils(dec!(40)));
    }
    // Europe: 0.4%, at least €35.
    let europe = |s, v| trade_fee(&p, s, Europe, dec!(10), eur(v)).unwrap();
    assert_eq!(europe(Etf, dec!(5000)), eur(dec!(35))); // €20 → minimum
    assert_eq!(europe(Bond, dec!(10000)), eur(dec!(40)));
    // Custody: 0.6% a quarter is 0.2% a month, at least ₪40 a month.
    let year = |v| custody_year(&p, Etf, Tlv, ils(v));
    assert_eq!(year(dec!(10000)), ils(dec!(480))); // ₪20 → ₪40, × 12
    assert_eq!(year(dec!(50000)), ils(dec!(1200))); // ₪100 × 12
    // Converting: 0.1%; the markup isn't published, so it's 0.
    assert_eq!(p.conversion_fee(usd(dec!(1000)), &rates()), usd(dec!(1)));
    assert_eq!(handling(&p, 0), dec!(99));
    assert!(p.sells_fractions_on(Usa));
    assert!(!p.sells_fractions_on(Europe));
}

#[test]
fn excellence_us_tracks_price_every_security() {
    let fee = |track, security, shares, value| {
        let p = on_track(excellence(), "Full tariff", track);
        trade_fee(&p, security, Usa, shares, usd(value)).unwrap()
    };
    // 100 shares × $50
    assert_eq!(fee("3¢ a share", Etf, dec!(100), dec!(5000)), usd(dec!(8))); // $3 → minimum
    assert_eq!(
        fee("$11 per order", Etf, dec!(100), dec!(5000)),
        usd(dec!(11))
    );
    let percent = "0.3% plus the broker's fee";
    assert_eq!(fee(percent, Etf, dec!(100), dec!(5000)), usd(dec!(15)));
    // 1,000 shares × $5
    assert_eq!(
        fee("3¢ a share", Stock, dec!(1000), dec!(5000)),
        usd(dec!(30))
    );
    // Bonds and funds are on the same tracks.
    assert_eq!(fee(percent, Bond, dec!(10), dec!(10000)), usd(dec!(30)));
    assert_eq!(
        fee("$11 per order", MutualFund, dec!(10), dec!(10000)),
        usd(dec!(11))
    );
}

#[test]
fn excellence_typical_offer() {
    let p = plan(excellence(), "Typical offer");
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value).unwrap();
    // Tel Aviv stocks and ETFs: 0.07%, at least ₪3.
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(10000))), ils(dec!(7)));
    assert_eq!(fee(Stock, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(3))); // ₪0.70 → minimum
    // Tel Aviv bonds and funds aren't in the offer: the full tariff's 0.4%,
    // at least ₪10.
    assert_eq!(fee(Bond, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(10)));
    assert_eq!(
        fee(MutualFund, Tlv, dec!(1), ils(dec!(10000))),
        ils(dec!(40))
    );
    // US stocks and ETFs: 1¢ a share, at least $6.
    assert_eq!(fee(Etf, Usa, dec!(100), usd(dec!(5000))), usd(dec!(6))); // $1 → minimum
    assert_eq!(fee(Etf, Usa, dec!(1000), usd(dec!(5000))), usd(dec!(10)));
    // US bonds and funds: the full tariff's tracks, which leave the offer's
    // stocks and ETFs alone.
    let flat = on_track(excellence(), "Typical offer", "$11 per order");
    let on_flat = |s| trade_fee(&flat, s, Usa, dec!(100), usd(dec!(5000))).unwrap();
    assert_eq!(on_flat(Bond), usd(dec!(11)));
    assert_eq!(on_flat(Etf), usd(dec!(6)));
    // Europe: the full tariff's 0.4%, at least €35.
    assert_eq!(fee(Etf, Europe, dec!(10), eur(dec!(5000))), eur(dec!(35)));
    // Converting: no fee, 2 agorot a dollar. $1,850 → ₪37 = $10; €1,850 =
    // $2,312.50 → ₪46.25 = €10.
    let conversion = |amount| cents(p.conversion_fee(amount, &rates()));
    assert_eq!(conversion(usd(dec!(1850))), usd(dec!(10)));
    assert_eq!(conversion(eur(dec!(1850))), eur(dec!(10)));
    // No custody; ₪15 a month after two free years.
    assert_eq!(custody_year(&p, Etf, Tlv, ils(dec!(100000))), ils(dec!(0)));
    assert_eq!(handling(&p, 23), dec!(0));
    assert_eq!(handling(&p, 24), dec!(15));
    assert_eq!(
        p.first_deposit_warning(ils(dec!(9999))).as_deref(),
        Some("Needs a one-time deposit of at least ₪10,000")
    );
}

// ─────────────────────────── IBI ───────────────────────────

#[test]
fn ibi_full_tariff() {
    let p = plan(ibi(), "Full tariff");
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value).unwrap();
    // Tel Aviv stocks, ETFs and bonds: 0.15%, at least ₪3.50.
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(3.5))); // ₪1.50 → minimum
    assert_eq!(fee(Bond, Tlv, dec!(1), ils(dec!(10000))), ils(dec!(15)));
    // Tel Aviv index funds: 0.08%, at least ₪5.
    assert_eq!(fee(MutualFund, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(5))); // ₪0.80 → minimum
    assert_eq!(
        fee(MutualFund, Tlv, dec!(1), ils(dec!(10000))),
        ils(dec!(8))
    );
    // Foreign funds: 0.275%, no minimum.
    assert_eq!(
        fee(MutualFund, Usa, dec!(10), usd(dec!(10000))),
        usd(dec!(27.5))
    );
    assert_eq!(
        fee(MutualFund, Europe, dec!(10), eur(dec!(1000))),
        eur(dec!(2.75))
    );
    // Foreign bonds: 0.225%, at least 20 in the trade's currency.
    assert_eq!(fee(Bond, Usa, dec!(5), usd(dec!(5000))), usd(dec!(20))); // $11.25 → minimum
    assert_eq!(fee(Bond, Usa, dec!(20), usd(dec!(20000))), usd(dec!(45)));
    assert_eq!(fee(Bond, Europe, dec!(5), eur(dec!(5000))), eur(dec!(20))); // €11.25 → minimum
    // European stocks and ETFs: 0.3%, at least €30.
    assert_eq!(fee(Etf, Europe, dec!(50), eur(dec!(5000))), eur(dec!(30))); // €15 → minimum
    assert_eq!(
        fee(Stock, Europe, dec!(50), eur(dec!(20000))),
        eur(dec!(60))
    );
    // Custody: 0.1% a quarter, no minimum. $10,000 = ₪37,000 → ₪37 a
    // quarter. None on Tel Aviv index funds, as its site promises.
    assert_eq!(custody_year(&p, Etf, Usa, usd(dec!(10000))), ils(dec!(148)));
    assert_eq!(
        custody_year(&p, Bond, Tlv, ils(dec!(100000))),
        ils(dec!(400))
    );
    assert_eq!(
        custody_year(&p, MutualFund, Tlv, ils(dec!(100000))),
        ils(dec!(0))
    );
    assert_eq!(
        custody_year(&p, MutualFund, Usa, usd(dec!(10000))),
        ils(dec!(148))
    );
    // Converting: no fee, and the markup of up to 0.7%.
    assert_eq!(p.conversion_fee(usd(dec!(1000)), &rates()), usd(dec!(7)));
    assert_eq!(handling(&p, 0), dec!(50));
    assert_eq!(
        p.first_deposit_warning(ils(dec!(14999))).as_deref(),
        Some("Needs a one-time deposit of at least ₪15,000")
    );
}

#[test]
fn ibi_us_tracks() {
    let fee = |track, shares, value| {
        let p = on_track(ibi(), "Full tariff", track);
        trade_fee(&p, Etf, Usa, shares, usd(value)).unwrap()
    };
    let both = "0.15% + 1¢ a share";
    // 100 shares × $50
    assert_eq!(fee("1¢ a share", dec!(100), dec!(5000)), usd(dec!(10))); // $1 → minimum
    assert_eq!(fee("$14 per order", dec!(100), dec!(5000)), usd(dec!(14)));
    assert_eq!(fee("0.15%", dec!(100), dec!(5000)), usd(dec!(10))); // $7.50 → minimum
    assert_eq!(fee(both, dec!(100), dec!(5000)), usd(dec!(8.5))); // $7.50 + $1
    // 10 shares × $300: $4.50 + 10¢ → minimum
    assert_eq!(fee(both, dec!(10), dec!(3000)), usd(dec!(6)));
    // 2,000 shares × $5
    assert_eq!(fee("1¢ a share", dec!(2000), dec!(10000)), usd(dec!(20)));
    assert_eq!(fee("0.15%", dec!(2000), dec!(10000)), usd(dec!(15)));
    assert_eq!(fee(both, dec!(2000), dec!(10000)), usd(dec!(35))); // $15 + $20
}

#[test]
fn ibi_typical_offer() {
    let p = plan(ibi(), "Typical offer");
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value).unwrap();
    // Tel Aviv stocks and ETFs: 0.08%, at least ₪2.35.
    assert_eq!(fee(Stock, Tlv, dec!(1), ils(dec!(10000))), ils(dec!(8)));
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(2.35))); // ₪0.80 → minimum
    // Tel Aviv bonds and funds: the full tariff's.
    assert_eq!(fee(Bond, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(3.5)));
    assert_eq!(fee(MutualFund, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(5)));
    // US stocks and ETFs: 1¢ a share, at least $7.50.
    assert_eq!(fee(Etf, Usa, dec!(100), usd(dec!(5000))), usd(dec!(7.5))); // $1 → minimum
    assert_eq!(fee(Etf, Usa, dec!(1000), usd(dec!(20000))), usd(dec!(10)));
    // US bonds: the full tariff's 0.225%.
    assert_eq!(fee(Bond, Usa, dec!(20), usd(dec!(20000))), usd(dec!(45)));
    // No custody; ₪15 a month from the start.
    assert_eq!(custody_year(&p, Etf, Usa, usd(dec!(10000))), ils(dec!(0)));
    assert_eq!(handling(&p, 0), dec!(15));
}

// ─────────────────────────── Interactive Israel ───────────────────────────

#[test]
fn interactive_prices() {
    let p = plan(interactive(), "Standard");
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value);
    // US stocks and ETFs: 1¢ a share, at least $2.50.
    assert_eq!(
        fee(Etf, Usa, dec!(10), usd(dec!(5000))),
        Some(usd(dec!(2.5)))
    ); // 10¢ → minimum
    assert_eq!(
        fee(Stock, Usa, dec!(1000), usd(dec!(5000))),
        Some(usd(dec!(10)))
    );
    // European stocks and ETFs: 0.15%, at least €2.50.
    assert_eq!(
        fee(Etf, Europe, dec!(10), eur(dec!(1000))),
        Some(eur(dec!(2.5)))
    ); // €1.50 → minimum
    assert_eq!(
        fee(Etf, Europe, dec!(100), eur(dec!(10000))),
        Some(eur(dec!(15)))
    );
    // Bonds: 0.2%, at least 10 in the trade's currency.
    assert_eq!(
        fee(Bond, Usa, dec!(1), usd(dec!(1000))),
        Some(usd(dec!(10)))
    ); // $2 → minimum
    assert_eq!(
        fee(Bond, Europe, dec!(10), eur(dec!(10000))),
        Some(eur(dec!(20)))
    );
    // Not offered: mutual funds, and Tel Aviv.
    assert_eq!(fee(MutualFund, Usa, dec!(10), usd(dec!(1000))), None);
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(1000))), None);
    // No custody, handling fee or minimum deposit.
    assert_eq!(custody_year(&p, Etf, Usa, usd(dec!(10000))), ils(dec!(0)));
    assert_eq!(handling(&p, 0), dec!(0));
    assert_eq!(p.first_deposit_warning(ils(dec!(1))), None);
    // Fractions of US shares.
    assert!(p.sells_fractions_on(Usa));
    assert!(!p.sells_fractions_on(Europe));
}

#[test]
fn interactive_conversion() {
    let p = plan(interactive(), "Standard");
    // 0.002%, at least ₪10. On $1,000 that's 2¢, so the ₪10 minimum: $2.70.
    let fee = |amount| cents(p.conversion_fee(amount, &rates()));
    assert_eq!(fee(usd(dec!(1000))), usd(dec!(2.7)));
    // Above ₪500,000 the percentage counts: 0.002% of $200,000 is $4.
    assert_eq!(fee(usd(dec!(200000))), usd(dec!(4)));
    // Its automatic plan buys at the usual price, and converts for free.
    let by_standing_order = p.standing_order_row(Etf, Usa).unwrap();
    assert_eq!(
        by_standing_order
            .price
            .apply(&trade(Etf, Usa, dec!(10), usd(dec!(1000))), &rates()),
        usd(dec!(2.5))
    );
    let conversion = p.standing_order_conversion.as_ref().unwrap();
    assert_eq!(conversion.cost(usd(dec!(1000)), &rates()), usd(dec!(0)));
}

// ─────────────────────────── Meitav Trade ───────────────────────────

#[test]
fn meitav_full_tariff() {
    let p = plan(meitav(), "Full tariff");
    // Every Tel Aviv security: 0.3%, at least ₪10.
    for security in Security::iter() {
        let fee = |v| trade_fee(&p, security, Tlv, dec!(1), ils(v)).unwrap();
        assert_eq!(fee(dec!(1000)), ils(dec!(10))); // ₪3 → minimum
        assert_eq!(fee(dec!(10000)), ils(dec!(30)));
    }
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value).unwrap();
    // Foreign bonds: 0.3%, at least $25.
    assert_eq!(fee(Bond, Usa, dec!(5), usd(dec!(5000))), usd(dec!(25))); // $15 → minimum
    assert_eq!(fee(Bond, Europe, dec!(5), eur(dec!(5000))), eur(dec!(20))); // €15; $25 = €20
    // Foreign funds: 0.2%, at least $20.
    assert_eq!(
        fee(MutualFund, Usa, dec!(10), usd(dec!(20000))),
        usd(dec!(40))
    );
    assert_eq!(
        fee(MutualFund, Europe, dec!(10), eur(dec!(5000))),
        eur(dec!(16))
    ); // €10; $20 = €16
    // European stocks and ETFs: 0.25%, at least €25.
    assert_eq!(fee(Etf, Europe, dec!(50), eur(dec!(5000))), eur(dec!(25))); // €12.50 → minimum
    assert_eq!(
        fee(Stock, Europe, dec!(50), eur(dec!(20000))),
        eur(dec!(50))
    );
    // Custody: 0.15% a quarter, at least ₪270 a quarter.
    let year = |v| custody_year(&p, Etf, Tlv, ils(v));
    assert_eq!(year(dec!(100000)), ils(dec!(1080))); // ₪150 → ₪270, × 4
    assert_eq!(year(dec!(400000)), ils(dec!(2400))); // ₪600 × 4
    // Converting: 0.13%, plus the markup of up to 0.7%.
    assert_eq!(p.conversion_fee(usd(dec!(1000)), &rates()), usd(dec!(8.3)));
    assert_eq!(handling(&p, 0), dec!(90));
    assert!(!p.sells_fractions_on(Usa));
}

#[test]
fn meitav_us_tracks() {
    let fee = |track, shares, value| {
        let p = on_track(meitav(), "Full tariff", track);
        trade_fee(&p, Stock, Usa, shares, usd(value)).unwrap()
    };
    // 100 shares × $50
    assert_eq!(fee("1¢ a share", dec!(100), dec!(5000)), usd(dec!(12))); // $1 → minimum
    assert_eq!(fee("0.7%", dec!(100), dec!(5000)), usd(dec!(35)));
    // 2,000 shares × $5
    assert_eq!(fee("1¢ a share", dec!(2000), dec!(10000)), usd(dec!(20)));
    // 10 shares × $100
    assert_eq!(fee("0.7%", dec!(10), dec!(1000)), usd(dec!(12))); // $7 → minimum
}

#[test]
fn meitav_typical_offer() {
    let p = plan(meitav(), "Typical offer");
    let fee = |s, e, shares, value| trade_fee(&p, s, e, shares, value).unwrap();
    // Tel Aviv: ETFs 0.07%, stocks 0.08%, both at least ₪4.65.
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(10000))), ils(dec!(7)));
    assert_eq!(fee(Stock, Tlv, dec!(1), ils(dec!(10000))), ils(dec!(8)));
    assert_eq!(fee(Etf, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(4.65))); // ₪0.70 → minimum
    // Tel Aviv bonds: the full tariff's 0.3%, at least ₪10.
    assert_eq!(fee(Bond, Tlv, dec!(1), ils(dec!(1000))), ils(dec!(10)));
    // US stocks and ETFs: 1¢ a share, at least $5.
    assert_eq!(fee(Etf, Usa, dec!(100), usd(dec!(5000))), usd(dec!(5))); // $1 → minimum
    assert_eq!(fee(Etf, Usa, dec!(1000), usd(dec!(5000))), usd(dec!(10)));
    // Converting: no fee, but the markup of up to 0.7%.
    assert_eq!(p.conversion_fee(usd(dec!(1000)), &rates()), usd(dec!(7)));
    // No custody; ₪15 a month after two free years.
    assert_eq!(custody_year(&p, Etf, Tlv, ils(dec!(100000))), ils(dec!(0)));
    assert_eq!(handling(&p, 23), dec!(0));
    assert_eq!(handling(&p, 24), dec!(15));
}
