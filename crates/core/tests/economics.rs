//! What must hold for any plan and any realistic investing pattern, whatever
//! the tariffs say: the accounting identities, orderings and closed forms
//! that make the comparison mean what it claims. `real_tariffs.rs` checks
//! each tariff's numbers; this checks the simulation as a piece of economics,
//! on the listed plans and on plans made up to isolate one rule.
//!
//! The property tests (`proptest!`) run each rule on dozens of random
//! scenarios rather than a hand-picked few: `proptest` draws the inputs from
//! the ranges given, and if a case fails it shrinks the inputs to the
//! smallest failing ones and prints them.

use broker_fees::simulation::{
    Around, Comparison, Crossing, Fees, InvalidScenario, NotOffered, Outcome, Scenario, Sweep,
    Swept, compare, free_plan, simulate, sweep,
};
use broker_fees::*;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

/// €1 = $1.25 = ₪4.625, so $1 = ₪3.70. Not real rates.
fn rates() -> ExchangeRates {
    ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
}

/// The app's defaults: ₪10,000, then ₪2,000 a month for 20 years at 10%.
fn defaults(security: Security, exchange: Exchange) -> Scenario {
    Scenario {
        security,
        exchange,
        first_deposit: dec!(10000),
        monthly_deposit: dec!(2000),
        deposit_growth: Percent(dec!(0)),
        yearly_return: Percent(dec!(10)),
        years: 20,
        buy_every_months: 1,
        share_price: dec!(500),
        sell_at_end: true,
        inflation: Percent(dec!(0)),
        age: 30,
        withdrawal: Withdrawal::LumpSum,
    }
}

/// Every listed plan, the brokers' and the funds', each named with its
/// broker or its kind of fund.
fn listed_plans() -> Vec<(String, Plan)> {
    named_plans(listed())
}

/// The funds' and policies' listed plans: a manager's fee and nothing else.
fn fund_plans() -> Vec<(String, Plan)> {
    named_plans(funds::all())
}

/// Whether part of what's deposited in `plan` is tax-free, as in a study
/// fund: then its tax isn't on the whole gain.
fn partly_tax_free(plan: &Plan) -> bool {
    plan.vehicle.rules().tax_free_deposits.is_some()
}

fn named_plans(brokers: Vec<Broker>) -> Vec<(String, Plan)> {
    brokers
        .into_iter()
        .flat_map(|broker| {
            broker
                .plans
                .into_iter()
                .map(move |plan| (format!("{} · {}", broker.short_name.en, plan.name.en), plan))
        })
        .collect()
}

/// The plan each broker's new customers get, and each kind of fund's
/// average.
fn usual_plans() -> Vec<(String, Plan)> {
    listed()
        .into_iter()
        .map(|mut broker| {
            let plan = broker.plans.swap_remove(broker.new_customer_plan);
            (format!("{} · {}", broker.short_name.en, plan.name.en), plan)
        })
        .collect()
}

fn every_purchase() -> impl Iterator<Item = (Security, Exchange)> {
    Security::iter().flat_map(|security| Exchange::iter().map(move |exchange| (security, exchange)))
}

/// Equal to within a millionth of a shekel: converting through a rate and
/// back leaves digits far past the agora.
fn close(a: Decimal, b: Decimal) -> bool {
    (a - b).abs() < dec!(0.000001)
}

/// A plan with one price for every trade and nothing else to pay, buying
/// whole shares like most brokers.
fn priced(price: Price) -> Plan {
    Plan {
        trading: vec![TradeFee {
            securities: vec![],
            exchanges: vec![],
            price,
        }],
        fractions_on: vec![],
        ..free_plan()
    }
}

/// A fund or a policy: a manager's share of the deposits and of the balance
/// (percentages), and nothing to pay for trades, custody or conversion.
fn managed(vehicle: Vehicle, of_deposits: Decimal, of_balance: Decimal) -> Plan {
    Plan {
        vehicle,
        management: Some(ManagementFee {
            of_deposits: Percent(of_deposits),
            of_balance: Percent(of_balance),
        }),
        ..free_plan()
    }
}

fn percent(percent: Decimal) -> Price {
    Price::Percent {
        percent: Percent(percent),
        min: None,
        max: None,
    }
}

// ─────────────────────────── Where the money goes ───────────────────────────

/// With no growth, every shekel put in is either handed back at the end or
/// paid in fees: nothing is created or lost in the bookkeeping. Amounts are
/// realistic (a first deposit of ₪1,000 or more), since a holding worth less
/// than its own sell fee is kept unsold and counts as nothing.
#[test]
fn without_growth_deposits_end_as_value_or_fees() {
    for (name, plan) in listed_plans() {
        for (security, exchange) in every_purchase() {
            for (buy_every_months, share_price) in [(1, dec!(500)), (3, dec!(5)), (6, dec!(4000))] {
                let scenario = Scenario {
                    yearly_return: Percent(Decimal::ZERO),
                    years: 5,
                    buy_every_months,
                    share_price,
                    ..defaults(security, exchange)
                };
                let Ok(outcome) = simulate(&plan, &scenario, &rates()) else {
                    continue;
                };
                let gap = scenario.deposited() - outcome.after_selling - outcome.fees.total();
                assert!(
                    close(gap, Decimal::ZERO),
                    "{name}, {security:?} on {exchange:?}, every {buy_every_months} months: \
                     ₪{gap} unaccounted for"
                );
            }
        }
    }
}

/// Selling everything costs exactly the difference between what's held and
/// what selling brings: the sell fee and converting back, nothing else.
#[test]
fn selling_costs_the_gap_between_held_and_sold() {
    for (name, plan) in listed_plans() {
        for (security, exchange) in every_purchase() {
            let scenario = defaults(security, exchange);
            let Ok(outcome) = simulate(&plan, &scenario, &rates()) else {
                continue;
            };
            assert!(
                close(outcome.held - outcome.after_selling, outcome.fees.selling),
                "{name}, {security:?} on {exchange:?}: held {}, sold {}, selling fees {}",
                outcome.held,
                outcome.after_selling,
                outcome.fees.selling
            );
        }
    }
}

/// The fees of the years add up to every fee but selling, and the running
/// totals end at the whole.
#[test]
fn the_years_add_up_to_the_whole() {
    for (name, plan) in listed_plans() {
        let scenario = defaults(Security::Etf, Exchange::Usa);
        let Ok(outcome) = simulate(&plan, &scenario, &rates()) else {
            continue;
        };
        assert_eq!(outcome.fees_by_year.len(), 20, "{name}");
        let by_year: Fees = outcome.fees_by_year.iter().copied().sum();
        assert_eq!(by_year.selling, Decimal::ZERO, "{name}");
        assert_eq!(
            by_year.total() + outcome.fees.selling,
            outcome.fees.total(),
            "{name}"
        );
        assert_eq!(
            outcome.fees_up_to_each_year().last(),
            Some(&outcome.fees),
            "{name}"
        );
    }
}

/// While the security doesn't fall, a plan is never worth more than the same
/// deposits with no fees at all, in any month: fees only take, and money
/// waiting for a purchase only misses growth.
#[test]
fn never_worth_more_than_with_no_fees_in_any_month() {
    for (name, plan) in listed_plans() {
        for (security, exchange) in every_purchase() {
            for yearly_return in [dec!(0), dec!(4), dec!(10)] {
                let scenario = Scenario {
                    yearly_return: Percent(yearly_return),
                    ..defaults(security, exchange)
                };
                let comparison = compare(&[&plan], &scenario, &rates());
                let Ok(outcome) = &comparison.plans[0].outcome else {
                    continue;
                };
                let worst = outcome.lost_by_month(&comparison.no_fees).min().unwrap();
                assert!(
                    close(worst.min(Decimal::ZERO), Decimal::ZERO),
                    "{name}, {security:?} on {exchange:?} at {yearly_return}%: \
                     worth ₪{} more than with no fees in some month",
                    -worst
                );
            }
        }
    }
}

// ─────────────────────────── The arithmetic ───────────────────────────

/// With no fees and every shekel invested at once, the simulation must agree
/// with the textbook formula for regular deposits compounding: each month's
/// deposit grows for the months left, so the total is a geometric series.
#[test]
fn compounding_matches_the_closed_form() {
    let scenario = Scenario {
        security: Security::IndexFund,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    let outcome = simulate(&free_plan(), &scenario, &rates()).unwrap();
    let months = i32::try_from(scenario.years * 12).unwrap();
    let monthly = 1.1_f64.powf(1.0 / 12.0);
    let first = 10_000.0 * monthly.powi(months);
    let regular = 2_000.0 * monthly * (monthly.powi(months) - 1.0) / (monthly - 1.0);
    let expected = first + regular;
    let held = outcome.held.to_f64().unwrap();
    assert!(
        ((held - expected) / expected).abs() < 1e-9,
        "held ₪{held}, the formula gives ₪{expected}"
    );
    // Nothing else to pay: selling brings the same.
    assert_eq!(outcome.after_selling, outcome.held);
    assert_eq!(outcome.fees, Fees::default());
}

/// A plan priced only in percentages charges the same share of any amount:
/// tripling the deposits triples what's left and what's paid. A flat fee or
/// a minimum would break this, so it catches one slipping in where the
/// tariff has none.
#[test]
fn percentage_fees_scale_with_the_money() {
    let plan = Plan {
        custody: vec![CustodyFee {
            securities: vec![],
            exchanges: vec![],
            percent: Percent(dec!(0.1)),
            per: Period::Year,
            billed: Period::Month,
            min: None,
        }],
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.2)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup: Markup::UpTo(Percent(dec!(0.5))),
        },
        fractions_on: Exchange::iter().collect(),
        ..priced(percent(dec!(0.1)))
    };
    // Index funds are bought by amount, so no share is left unbought.
    let small = defaults(Security::IndexFund, Exchange::Usa);
    let large = Scenario {
        first_deposit: small.first_deposit * dec!(3),
        monthly_deposit: small.monthly_deposit * dec!(3),
        ..small.clone()
    };
    let small = simulate(&plan, &small, &rates()).unwrap();
    let large = simulate(&plan, &large, &rates()).unwrap();
    let ratio = |a: Decimal, b: Decimal| (a / b).round_dp(9);
    assert_eq!(ratio(large.after_selling, small.after_selling), dec!(3));
    assert_eq!(ratio(large.fees.total(), small.fees.total()), dec!(3));
    assert_eq!(ratio(large.fees.custody, small.fees.custody), dec!(3));
    assert_eq!(
        ratio(large.fees.conversions, small.fees.conversions),
        dec!(3)
    );
}

/// Nothing is converted on Tel Aviv, so the exchange rates can't change
/// what a Tel Aviv investment comes to, on any plan.
#[test]
fn exchange_rates_dont_touch_tel_aviv() {
    let other_rates = ExchangeRates::new(dec!(5), dec!(6)).unwrap();
    for (name, plan) in listed_plans() {
        for security in Security::iter() {
            let scenario = defaults(security, Exchange::Tlv);
            let Ok(usual) = simulate(&plan, &scenario, &rates()) else {
                continue;
            };
            let other = simulate(&plan, &scenario, &other_rates).unwrap();
            assert_eq!(
                usual.after_selling, other.after_selling,
                "{name}, {security:?}"
            );
            assert_eq!(usual.fees, other.fees, "{name}, {security:?}");
        }
    }
}

// ─────────────────────────── Orderings ───────────────────────────

/// A minimum per order beats a percentage on small orders and loses on large
/// ones: the ranking must flip where the fees cross. 0.4% of a trade
/// against a flat ₪26 cross at ₪6,500 a trade.
#[test]
fn the_ranking_flips_where_the_fees_cross() {
    let percentage = Plan {
        name: Text::same("0.4%"),
        ..priced(percent(dec!(0.4)))
    };
    let flat = Plan {
        name: Text::same("₪26 an order"),
        ..priced(Price::Flat(ils(dec!(26))))
    };
    let ranking = |monthly_deposit| {
        let scenario = Scenario {
            first_deposit: Decimal::ZERO,
            monthly_deposit,
            yearly_return: Percent(Decimal::ZERO),
            years: 1,
            ..defaults(Security::Etf, Exchange::Tlv)
        };
        let comparison = compare(&[&percentage, &flat], &scenario, &rates());
        comparison
            .plans
            .iter()
            .map(|plan| plan.index)
            .collect::<Vec<_>>()
    };
    assert_eq!(ranking(dec!(2000)), [0, 1], "₪8 an order beats ₪26");
    assert_eq!(ranking(dec!(20000)), [1, 0], "₪26 beats ₪80 an order");
}

/// `base`, with each kind of fee raised on its own.
fn each_fee_raised(base: &Plan) -> Vec<(&'static str, Plan)> {
    let trading = |price| priced(price).trading;
    vec![
        (
            "trade percentage",
            Plan {
                trading: trading(Price::Percent {
                    percent: Percent(dec!(0.5)),
                    min: Some(usd(dec!(5))),
                    max: None,
                }),
                ..base.clone()
            },
        ),
        (
            "trade minimum",
            Plan {
                trading: trading(Price::Percent {
                    percent: Percent(dec!(0.2)),
                    min: Some(usd(dec!(30))),
                    max: None,
                }),
                ..base.clone()
            },
        ),
        (
            "custody",
            Plan {
                custody: vec![CustodyFee {
                    percent: Percent(dec!(0.8)),
                    min: Some(ils(dec!(75))),
                    ..base.custody[0].clone()
                }],
                ..base.clone()
            },
        ),
        (
            "conversion fee",
            Plan {
                conversion: ConversionFee {
                    fee: PercentFee {
                        percent: Percent(dec!(0.5)),
                        min: Some(usd(dec!(10))),
                        max: None,
                    },
                    ..base.conversion.clone()
                },
                ..base.clone()
            },
        ),
        (
            "markup",
            Plan {
                conversion: ConversionFee {
                    markup: Markup::UpTo(Percent(dec!(1))),
                    ..base.conversion.clone()
                },
                ..base.clone()
            },
        ),
        (
            "handling",
            Plan {
                handling: Some(HandlingFee {
                    per_month: ils(dec!(40)),
                    free_months: 0,
                    less_trade_fees: false,
                }),
                ..base.clone()
            },
        ),
        (
            "manager's share of deposits",
            Plan {
                management: Some(ManagementFee {
                    of_deposits: Percent(dec!(2)),
                    of_balance: Percent(dec!(0.3)),
                }),
                ..base.clone()
            },
        ),
        (
            "manager's share of the balance",
            Plan {
                management: Some(ManagementFee {
                    of_deposits: Percent(dec!(1)),
                    of_balance: Percent(dec!(0.9)),
                }),
                ..base.clone()
            },
        ),
    ]
}

/// Raising any one fee never leaves more at the end, while the security
/// doesn't fall. Each kind of fee is raised on its own, on a plan that has
/// them all, buying abroad so that conversion counts too.
#[test]
fn raising_any_fee_never_helps() {
    let base = Plan {
        custody: vec![CustodyFee {
            securities: vec![],
            exchanges: vec![],
            percent: Percent(dec!(0.2)),
            per: Period::Year,
            billed: Period::Quarter,
            min: Some(ils(dec!(10))),
        }],
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(5))),
                max: None,
            },
            or_if_less: None,
            markup: Markup::UpTo(Percent(dec!(0.3))),
        },
        handling: Some(HandlingFee {
            per_month: ils(dec!(10)),
            free_months: 12,
            less_trade_fees: true,
        }),
        management: Some(ManagementFee {
            of_deposits: Percent(dec!(1)),
            of_balance: Percent(dec!(0.3)),
        }),
        ..priced(Price::Percent {
            percent: Percent(dec!(0.2)),
            min: Some(usd(dec!(5))),
            max: None,
        })
    };
    for (security, exchange) in [
        (Security::Etf, Exchange::Usa),
        (Security::Bond, Exchange::Europe),
    ] {
        for share_price in [dec!(50), dec!(1500)] {
            let scenario = Scenario {
                share_price,
                ..defaults(security, exchange)
            };
            let before = simulate(&base, &scenario, &rates()).unwrap();
            for (fee, plan) in each_fee_raised(&base) {
                let after = simulate(&plan, &scenario, &rates()).unwrap();
                assert!(
                    after.after_selling <= before.after_selling,
                    "a higher {fee} left more: ₪{} rather than ₪{} ({security:?} on {exchange:?}, \
                     shares at {share_price})",
                    after.after_selling,
                    before.after_selling
                );
                assert!(
                    after.fees.total() >= before.fees.total(),
                    "a higher {fee} cost less"
                );
                assert!(
                    after.after_tax <= before.after_tax,
                    "a higher {fee} left more after tax"
                );
            }
        }
    }
}

/// A plan with tracks costs what its cheapest track does: no track, taken on
/// its own, ends with more.
#[test]
fn the_track_used_is_the_cheapest() {
    for (name, plan) in listed_plans() {
        if plan.tracks.is_empty() {
            continue;
        }
        for (security, exchange) in every_purchase() {
            for share_price in [dec!(5), dec!(500)] {
                let scenario = Scenario {
                    share_price,
                    ..defaults(security, exchange)
                };
                let Ok(chosen) = simulate(&plan, &scenario, &rates()) else {
                    continue;
                };
                for (index, track) in plan.tracks.iter().enumerate() {
                    let alone = simulate(&plan.on_track(index), &scenario, &rates()).unwrap();
                    assert!(
                        alone.after_selling <= chosen.after_selling,
                        "{name}, {security:?} on {exchange:?} at {share_price}: \
                         track {} ends with ₪{}, the one used with ₪{}",
                        track.name.en,
                        alone.after_selling,
                        chosen.after_selling
                    );
                }
            }
        }
    }
}

/// Waiting for a whole share costs only growth: with none, a broker selling
/// fractions ends with exactly what one selling whole shares does, and with
/// growth it ends with more.
#[test]
fn fractions_only_matter_through_growth() {
    let whole = priced(Price::Flat(usd(Decimal::ZERO)));
    let fractions = Plan {
        fractions_on: vec![Exchange::Usa],
        ..whole.clone()
    };
    let at = |yearly_return| Scenario {
        yearly_return: Percent(yearly_return),
        ..defaults(Security::Etf, Exchange::Usa)
    };
    let ends_with = |plan: &Plan, scenario: &Scenario| {
        simulate(plan, scenario, &rates()).unwrap().after_selling
    };
    assert!(close(
        ends_with(&whole, &at(dec!(0))),
        ends_with(&fractions, &at(dec!(0)))
    ));
    assert!(ends_with(&whole, &at(dec!(10))) < ends_with(&fractions, &at(dec!(10))));
}

// ─────────────────────────── The listed plans ───────────────────────────

/// On the app's defaults, every usual plan loses a plausible share to fees
/// over 20 years: between a tenth of a percent and a third of the no-fee
/// value. A tariff or model change that leaves this band is worth a look.
#[test]
fn usual_plans_lose_a_plausible_share() {
    for (name, plan) in usual_plans() {
        for (security, exchange) in every_purchase() {
            let scenario = defaults(security, exchange);
            let comparison = compare(&[&plan], &scenario, &rates());
            let Ok(outcome) = &comparison.plans[0].outcome else {
                continue;
            };
            let share =
                outcome.lost_to_fees(&comparison.no_fees) / comparison.no_fees.after_selling;
            assert!(
                (dec!(0.001)..dec!(0.34)).contains(&share),
                "{name}, {security:?} on {exchange:?}: {:.1}% lost to fees",
                share * Decimal::ONE_HUNDRED
            );
            assert!(
                outcome.fees.total() < scenario.deposited(),
                "{name}, {security:?} on {exchange:?}: fees of ₪{} on ₪{} deposited",
                outcome.fees.total(),
                scenario.deposited()
            );
        }
    }
}

/// Every plan compared gets a rank, best first by what's left after selling
/// and paying the tax, with the ones that can't be used last. That isn't
/// always the order of what's left before tax: what a broker takes for
/// keeping the account is taxed as gain, so of two plans that leave about
/// the same, the one charging on trades pays less tax.
#[test]
fn the_comparison_ranks_by_what_is_left() {
    let plans = listed_plans();
    let plans: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
    for (security, exchange) in every_purchase() {
        let comparison = compare(&plans, &defaults(security, exchange), &rates());
        assert_eq!(comparison.plans.len(), plans.len());
        let left: Vec<Option<Decimal>> = comparison
            .plans
            .iter()
            .map(|plan| {
                let outcome = plan.outcome.as_ref().ok();
                outcome.map(|outcome| outcome.after_tax)
            })
            .collect();
        let offered: Vec<Decimal> = left.iter().flatten().copied().collect();
        assert!(
            offered.windows(2).all(|pair| pair[0] >= pair[1]),
            "{security:?} on {exchange:?}: {left:?}"
        );
        let first_unoffered = left.iter().position(Option::is_none).unwrap_or(left.len());
        assert!(
            left[first_unoffered..].iter().all(Option::is_none),
            "{security:?} on {exchange:?}: {left:?}"
        );
    }
}

// ─────────────────────────── Random scenarios ───────────────────────────

/// A realistic scenario: amounts a person might type. The first deposit is
/// at least ₪1,000, so no holding is worth less than its own sell fee.
fn scenarios() -> impl Strategy<Value = Scenario> {
    let securities = prop::sample::select(Security::iter().collect::<Vec<_>>());
    let exchanges = prop::sample::select(Exchange::iter().collect::<Vec<_>>());
    (
        securities,
        exchanges,
        1..=200u32,
        0..=100u32,
        0..=15u32,
        1..=25u32,
        prop::sample::select(vec![1u32, 2, 3, 6, 12]),
        prop::sample::select(vec![dec!(5), dec!(50), dec!(500), dec!(4000)]),
        0..=6u32,
        18..=70u32,
        prop::sample::select(vec![Withdrawal::LumpSum, Withdrawal::Pension]),
    )
        .prop_map(
            |(
                security,
                exchange,
                first,
                monthly,
                yearly_return,
                years,
                buy_every_months,
                share_price,
                inflation,
                age,
                withdrawal,
            )| {
                Scenario {
                    security,
                    exchange,
                    first_deposit: Decimal::from(first) * dec!(1000),
                    monthly_deposit: Decimal::from(monthly) * dec!(100),
                    deposit_growth: Percent(Decimal::ZERO),
                    yearly_return: Percent(Decimal::from(yearly_return)),
                    years,
                    buy_every_months,
                    share_price,
                    sell_at_end: true,
                    inflation: Percent(Decimal::from(inflation)),
                    age,
                    withdrawal,
                }
            },
        )
}

/// Every listed plan that can be used for the scenario (a broker's that
/// offers its security there, a fund that takes its deposits), with its
/// outcome and the no-fee outcome to compare it with.
fn outcomes(scenario: &Scenario) -> (Outcome, Vec<(String, Outcome)>) {
    let plans = listed_plans();
    let refs: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
    let comparison = compare(&refs, scenario, &rates());
    let outcomes = comparison
        .plans
        .into_iter()
        .filter_map(|compared| Some((plans[compared.index].0.clone(), compared.outcome.ok()?)))
        .collect();
    (comparison.no_fees, outcomes)
}

proptest! {
    // No file of past failures: an integration test has no `lib.rs` to keep
    // it beside, and the failing inputs are printed anyway.
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn the_no_fee_baseline_is_free(scenario in scenarios()) {
        scenario.check().unwrap();
        let (no_fees, _) = outcomes(&scenario);
        prop_assert_eq!(no_fees.fees, Fees::default());
        prop_assert_eq!(no_fees.after_selling, no_fees.held);
        prop_assert_eq!(no_fees.value_by_month.len(), scenario.years as usize * 12 + 1);
    }

    #[test]
    fn selling_always_costs_the_gap_between_held_and_sold(scenario in scenarios()) {
        for (name, outcome) in outcomes(&scenario).1 {
            prop_assert!(
                close(outcome.held - outcome.after_selling, outcome.fees.selling),
                "{name}: held {}, sold {}, selling fees {}",
                outcome.held, outcome.after_selling, outcome.fees.selling
            );
        }
    }

    #[test]
    fn without_growth_nothing_is_unaccounted_for(scenario in scenarios()) {
        let scenario = Scenario { yearly_return: Percent(Decimal::ZERO), ..scenario };
        for (name, outcome) in outcomes(&scenario).1 {
            let gap = scenario.deposited() - outcome.after_selling - outcome.fees.total();
            prop_assert!(close(gap, Decimal::ZERO), "{name}: ₪{gap} unaccounted for");
        }
    }

    #[test]
    fn no_plan_is_ever_worth_more_than_no_fees(scenario in scenarios()) {
        let (no_fees, outcomes) = outcomes(&scenario);
        for (name, outcome) in outcomes {
            let worst = outcome.lost_by_month(&no_fees).min().unwrap();
            prop_assert!(close(worst.min(Decimal::ZERO), Decimal::ZERO), "{name}: ₪{} ahead of no fees in some month", -worst);
            // Never less than the fees themselves: growth is only ever missed.
            let lost = outcome.lost_to_fees(&no_fees);
            prop_assert!(lost + dec!(0.000001) >= outcome.fees.total(), "{name}: lost ₪{lost}, paid ₪{}", outcome.fees.total());
        }
    }
}

// ─────────────────────────── A manager's fees ───────────────────────────

/// A fee on the balance alone is the yearly cost itself, which is defined
/// as the fund fee with the same effect. Taken a twelfth every month, 1% a
/// year compounds to 1 − (1 − 0.01 / 12)¹² = 0.9954%.
#[test]
fn a_fee_on_the_balance_is_the_yearly_cost() {
    let plan = managed(Vehicle::SavingsPolicy, dec!(0), dec!(1));
    for (security, exchange) in every_purchase() {
        let outcome = simulate(&plan, &defaults(security, exchange), &rates()).unwrap();
        assert_eq!(
            outcome.yearly_cost.0.round_dp(3),
            dec!(0.995),
            "{security:?} on {exchange:?}"
        );
    }
}

/// A fee on deposits takes its share of every deposit, and so of everything
/// the deposits grow to: 4% leaves 96% of what no fees would, and pays 4% of
/// what's deposited.
#[test]
fn a_fee_on_deposits_takes_its_share_of_the_end() {
    let plan = managed(Vehicle::SavingsPolicy, dec!(4), dec!(0));
    let scenario = defaults(Security::Etf, Exchange::Usa);
    let comparison = compare(&[&plan], &scenario, &rates());
    let outcome = comparison.plans[0].outcome.as_ref().unwrap();
    assert!(about(
        outcome.after_selling,
        comparison.no_fees.after_selling * dec!(0.96)
    ));
    assert!(close(
        outcome.fees.management,
        scenario.deposited() * dec!(0.04)
    ));
    assert_eq!(outcome.fees.total(), outcome.fees.management);
}

/// A fund invests each deposit as it arrives: how often the saver would buy
/// at a broker changes nothing. The same fees in a brokerage account do
/// wait, and miss the growth.
#[test]
fn a_fund_invests_every_deposit_at_once() {
    let fund = managed(Vehicle::InvestmentGemel, dec!(1), dec!(0.6));
    let monthly = defaults(Security::Etf, Exchange::Usa);
    let twice_a_year = Scenario {
        buy_every_months: 6,
        ..monthly.clone()
    };
    let left = |plan: &Plan, scenario: &Scenario| {
        simulate(plan, scenario, &rates()).unwrap().after_selling
    };
    assert_eq!(left(&fund, &twice_a_year), left(&fund, &monthly));
    let broker = Plan {
        vehicle: Vehicle::Brokerage,
        ..fund.clone()
    };
    assert!(left(&broker, &twice_a_year) < left(&broker, &monthly));
}

/// A provident fund takes ₪83,641 a year and not a shekel more; a broker and
/// a savings policy take anything.
#[test]
fn deposits_over_the_ceiling_arent_taken() {
    let fund = managed(Vehicle::InvestmentGemel, dec!(0), dec!(0.6));
    let at_the_ceiling = Scenario {
        first_deposit: dec!(83641),
        monthly_deposit: dec!(0),
        years: 3,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    assert!(simulate(&fund, &at_the_ceiling, &rates()).is_ok());
    let over = Scenario {
        first_deposit: dec!(83642),
        ..at_the_ceiling
    };
    assert_eq!(
        simulate(&fund, &over, &rates()).err(),
        Some(NotOffered::OverTheCeiling { year: 0 })
    );
    for vehicle in [Vehicle::Brokerage, Vehicle::SavingsPolicy] {
        let plan = managed(vehicle, dec!(0), dec!(0.6));
        assert!(simulate(&plan, &over, &rates()).is_ok(), "{vehicle:?}");
    }
    // A plan the deposits are too much for goes last, like one that doesn't
    // sell the security.
    let policy = managed(Vehicle::SavingsPolicy, dec!(0), dec!(2));
    let comparison = compare(&[&fund, &policy], &over, &rates());
    let order: Vec<usize> = comparison.plans.iter().map(|plan| plan.index).collect();
    assert_eq!(order, [1, 0]);
}

/// The ceiling rises with prices. ₪6,900 a month is ₪82,800 a year; growing
/// 3% a year it's ₪85,284 in the second year, over the ceiling if prices
/// stand still, and under it for good if they rise 3% a year too.
#[test]
fn the_ceiling_rises_with_prices() {
    let fund = managed(Vehicle::InvestmentGemel, dec!(0), dec!(0.6));
    let growing = Scenario {
        first_deposit: dec!(0),
        monthly_deposit: dec!(6900),
        deposit_growth: Percent(dec!(3)),
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    assert_eq!(
        simulate(&fund, &growing, &rates()).err(),
        Some(NotOffered::OverTheCeiling { year: 1 })
    );
    let with_prices = Scenario {
        inflation: Percent(dec!(3)),
        ..growing
    };
    assert!(simulate(&fund, &with_prices, &rates()).is_ok());
}

// ─────────────────────────── Tax on the gain ───────────────────────────

/// The provident fund's pension is the only way out that isn't taxed: from
/// 60, as a pension, everything is kept. A year younger, or as a lump sum,
/// the gain is taxed. A broker and a policy tax every way out alike.
#[test]
fn only_the_funds_pension_is_exempt() {
    // 20 years: 40 today is 60 at the end.
    let saver = |age, withdrawal| Scenario {
        age,
        withdrawal,
        ..defaults(Security::Etf, Exchange::Usa)
    };
    let fund = managed(Vehicle::InvestmentGemel, dec!(0), dec!(0.6));
    let run = |plan: &Plan, scenario: &Scenario| simulate(plan, scenario, &rates()).unwrap();

    let lump_sum = run(&fund, &saver(40, Withdrawal::LumpSum));
    assert!(lump_sum.tax > dec!(0));
    let pension = run(&fund, &saver(40, Withdrawal::Pension));
    assert_eq!(pension.tax, dec!(0));
    assert_eq!(pension.after_tax, pension.after_selling);
    assert_eq!(pension.after_selling, lump_sum.after_selling);
    let too_young = run(&fund, &saver(39, Withdrawal::Pension));
    assert_eq!(too_young.tax, lump_sum.tax);

    for vehicle in [Vehicle::Brokerage, Vehicle::SavingsPolicy] {
        let plan = managed(vehicle, dec!(0), dec!(0.6));
        let as_pension = run(&plan, &saver(40, Withdrawal::Pension));
        assert_eq!(as_pension.tax, lump_sum.tax, "{vehicle:?}");
    }
}

/// A study fund's gains aren't taxed on what's deposited within its yearly
/// amount, and are taxed like any gain on the rest: the same money in a
/// savings policy at the same fee ends the same before tax, and pays tax on
/// all of its gain.
#[test]
fn a_study_fund_taxes_only_the_gains_on_what_is_over_its_yearly_amount() {
    let fund = managed(Vehicle::StudyFund, dec!(0), dec!(0.6));
    let policy = managed(Vehicle::SavingsPolicy, dec!(0), dec!(0.6));
    let run = |plan: &Plan, scenario: &Scenario| simulate(plan, scenario, &rates()).unwrap();
    let once = |first_deposit| Scenario {
        first_deposit,
        monthly_deposit: dec!(0),
        years: 10,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    // ₪20,566 in a year, the whole amount: no tax.
    let within = run(&fund, &once(dec!(20566)));
    assert_eq!(within.tax, dec!(0));
    assert_eq!(within.after_tax, within.after_selling);
    assert!(run(&policy, &once(dec!(20566))).tax > dec!(0));
    // Twice the amount: half the deposit's gain is taxed.
    let twice = once(dec!(41132));
    let (in_fund, in_policy) = (run(&fund, &twice), run(&policy, &twice));
    assert_eq!(in_fund.after_selling, in_policy.after_selling);
    assert!(about(in_fund.tax * dec!(2), in_policy.tax));

    // Each year's deposits count together. ₪1,700 a month is ₪20,400 a
    // year: within. ₪2,000 a month is ₪24,000, of which 3,434 is over:
    // that share of the gain is taxed, every year alike.
    let monthly = |monthly_deposit| Scenario {
        first_deposit: dec!(0),
        monthly_deposit,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    assert_eq!(run(&fund, &monthly(dec!(1700))).tax, dec!(0));
    let over = monthly(dec!(2000));
    let share = dec!(3434) / dec!(24000);
    assert!(about(
        run(&fund, &over).tax,
        run(&policy, &over).tax * share
    ));

    // The amount follows prices. Deposits growing 3% a year from ₪20,400
    // pass it in the second year (₪21,012) if prices stand still, and never
    // if they rise 3% a year too.
    let growing = Scenario {
        deposit_growth: Percent(dec!(3)),
        ..monthly(dec!(1700))
    };
    assert!(run(&fund, &growing).tax > dec!(0));
    let with_prices = Scenario {
        inflation: Percent(dec!(3)),
        ..growing
    };
    assert_eq!(run(&fund, &with_prices).tax, dec!(0));
}

/// A study fund's money comes out on its terms only after six years: a
/// shorter period isn't offered, and goes last in a comparison.
#[test]
fn a_study_fund_opens_after_six_years() {
    let fund = managed(Vehicle::StudyFund, dec!(0), dec!(0.6));
    let years = |years| Scenario {
        years,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    assert_eq!(
        simulate(&fund, &years(5), &rates()).err(),
        Some(NotOffered::Locked { years: 6 })
    );
    assert!(simulate(&fund, &years(6), &rates()).is_ok());
    let policy = managed(Vehicle::SavingsPolicy, dec!(0), dec!(2));
    let comparison = compare(&[&fund, &policy], &years(5), &rates());
    let order: Vec<usize> = comparison.plans.iter().map(|plan| plan.index).collect();
    assert_eq!(order, [1, 0]);
    // The other vehicles are open from the first day.
    for vehicle in [
        Vehicle::Brokerage,
        Vehicle::InvestmentGemel,
        Vehicle::SavingsPolicy,
    ] {
        let plan = managed(vehicle, dec!(0), dec!(0.6));
        assert!(simulate(&plan, &years(1), &rates()).is_ok(), "{vehicle:?}");
    }
}

/// Only the gain beyond the rise in prices is taxed, which is the gain in
/// today's money: restated in it, the tax is a quarter of what came out
/// beyond what went in. Checked where every fee is a cost of the holdings:
/// with no fees, and in a fund.
#[test]
fn the_tax_is_a_quarter_of_the_gain_in_todays_money() {
    let mut plans = fund_plans();
    plans.retain(|(_, plan)| !partly_tax_free(plan));
    plans.push(("no fees".to_owned(), free_plan()));
    for inflation in [dec!(0), dec!(2), dec!(5)] {
        let inflation = Percent(inflation);
        let scenario = Scenario {
            inflation,
            ..defaults(Security::IndexFund, Exchange::Tlv)
        };
        for (name, plan) in &plans {
            let outcome = simulate(plan, &scenario, &rates())
                .unwrap()
                .in_todays_money(inflation);
            let gain = outcome.after_selling - scenario.deposited_in_todays_money(inflation);
            assert!(gain > dec!(0));
            assert!(
                about(outcome.tax, gain / dec!(4)),
                "{name} at {inflation}: tax ₪{}, gain ₪{gain}",
                outcome.tax
            );
        }
    }
}

/// The more prices rise, the less of the gain is taxed; and falling prices
/// count as standing still, since the law never lowers what something cost.
/// A study fund's tax-free amount does follow prices down, so there falling
/// prices leave more to tax.
#[test]
fn inflation_only_ever_lowers_the_tax() {
    for (name, plan) in &listed_plans() {
        let scenario = defaults(Security::Etf, Exchange::Usa);
        if simulate(plan, &scenario, &rates()).is_err() {
            continue;
        }
        let tax = |inflation| {
            let scenario = Scenario {
                inflation: Percent(inflation),
                ..scenario.clone()
            };
            simulate(plan, &scenario, &rates()).unwrap().tax
        };
        if partly_tax_free(plan) {
            assert!(tax(dec!(-3)) > tax(dec!(0)), "{name}");
        } else {
            assert_eq!(tax(dec!(-3)), tax(dec!(0)), "{name}");
        }
        assert!(tax(dec!(2)) < tax(dec!(0)), "{name}");
        assert!(tax(dec!(5)) < tax(dec!(2)), "{name}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]

    /// With prices standing still, what's taxed is what came out beyond
    /// what went in. What was paid to keep the account (custody, the
    /// monthly fee) isn't part of what the holdings cost, and comes off the
    /// proceeds only in the year of the sale: the earlier years' is taxed
    /// with the gain. A manager's fees and the fees on each order are cost.
    /// A study fund, where part of the gain is tax-free, never pays more.
    #[test]
    fn the_tax_is_a_quarter_of_what_came_out_beyond_what_went_in(scenario in scenarios()) {
        let scenario = Scenario {
            inflation: Percent(Decimal::ZERO),
            withdrawal: Withdrawal::LumpSum,
            ..scenario
        };
        let partly_free: Vec<String> = listed_plans()
            .into_iter()
            .filter(|(_, plan)| partly_tax_free(plan))
            .map(|(name, _)| name)
            .collect();
        for (name, outcome) in outcomes(&scenario).1 {
            let last_year = outcome.fees_by_year.last().copied().unwrap_or_default();
            let gain = outcome.after_selling - scenario.deposited()
                + outcome.fees.custody
                + outcome.fees.handling
                - last_year.custody
                - last_year.handling;
            let on_it_all = gain.max(Decimal::ZERO) / dec!(4);
            if partly_free.contains(&name) {
                prop_assert!(outcome.tax <= on_it_all + dec!(0.000001), "{name}: tax ₪{}", outcome.tax);
            } else {
                prop_assert!(close(outcome.tax, on_it_all), "{name}: tax ₪{}, gain ₪{gain}", outcome.tax);
            }
        }
    }

    /// Tax only takes, and what's left after it is what selling brought
    /// less the tax.
    #[test]
    fn tax_comes_off_what_is_left(scenario in scenarios()) {
        for (name, outcome) in outcomes(&scenario).1 {
            prop_assert!(outcome.tax >= Decimal::ZERO, "{name}");
            prop_assert_eq!(outcome.after_tax, outcome.after_selling - outcome.tax, "{}", name);
        }
    }

    /// A gain that only keeps up with prices isn't taxed: when the return
    /// is no more than inflation, no plan pays tax.
    #[test]
    fn keeping_up_with_prices_isnt_taxed(scenario in scenarios()) {
        let scenario = Scenario { inflation: scenario.yearly_return, ..scenario };
        for (name, outcome) in outcomes(&scenario).1 {
            prop_assert!(close(outcome.tax, Decimal::ZERO), "{name}: tax ₪{}", outcome.tax);
        }
    }
}

// ─────────────────────────── The yearly cost ───────────────────────────

/// A plan's yearly cost is what it claims to be: investing the same deposits
/// with no fees, buying every month, at a return lowered by that charge,
/// ends with just what the plan leaves.
#[test]
fn the_yearly_cost_is_a_fund_fee_with_the_same_effect() {
    for (security, exchange) in every_purchase() {
        let scenario = defaults(security, exchange);
        let (no_fees, outcomes) = outcomes(&scenario);
        for (name, outcome) in outcomes {
            assert_yearly_cost_holds(&name, &scenario, &outcome, &no_fees).unwrap();
        }
    }
}

/// A plan that charges nothing costs nothing a year; one that leaves nothing
/// costs everything.
#[test]
fn the_yearly_cost_runs_from_nothing_to_everything() {
    let scenario = defaults(Security::IndexFund, Exchange::Tlv);
    let free = simulate(&free_plan(), &scenario, &rates()).unwrap();
    assert_eq!(free.yearly_cost, Percent(Decimal::ZERO));
    // Handling fee of ₪1,000 a month on ₪100 a month: nothing is left.
    let ruinous = Plan {
        handling: Some(HandlingFee {
            per_month: ils(dec!(1000)),
            free_months: 0,
            less_trade_fees: false,
        }),
        ..free_plan()
    };
    let small = Scenario {
        first_deposit: Decimal::ZERO,
        monthly_deposit: dec!(100),
        ..scenario
    };
    let ruined = simulate(&ruinous, &small, &rates()).unwrap();
    assert!(ruined.after_selling <= Decimal::ZERO);
    assert_eq!(ruined.yearly_cost, Percent(Decimal::ONE_HUNDRED));
}

/// Asserts, for one plan's outcome, that the no-fee investing at the return
/// lowered by the yearly cost ends where the plan does, to within a ten
/// thousandth of the no-fee value (the cost is rounded to four decimals).
fn assert_yearly_cost_holds(
    name: &str,
    scenario: &Scenario,
    outcome: &Outcome,
    no_fees: &Outcome,
) -> Result<(), TestCaseError> {
    let cost = outcome.yearly_cost;
    prop_assert!(
        cost.0 >= Decimal::ZERO && cost.0 <= Decimal::ONE_HUNDRED,
        "{name}: yearly cost {cost}"
    );
    if cost.0 == Decimal::ONE_HUNDRED {
        prop_assert!(
            outcome.after_selling <= Decimal::ZERO,
            "{name}: costs everything but leaves ₪{}",
            outcome.after_selling
        );
        return Ok(());
    }
    let growth = Decimal::ONE + scenario.yearly_return.of(Decimal::ONE);
    let lowered = growth * (Decimal::ONE - cost.of(Decimal::ONE)) - Decimal::ONE;
    let as_a_fund_fee = Scenario {
        yearly_return: Percent(lowered * Decimal::ONE_HUNDRED),
        buy_every_months: 1,
        ..scenario.clone()
    };
    let same = simulate(&free_plan(), &as_a_fund_fee, &rates()).unwrap();
    let tolerance = no_fees.after_selling * dec!(0.0001);
    prop_assert!(
        (same.after_selling - outcome.after_selling).abs() <= tolerance,
        "{name}: a fund fee of {cost} leaves ₪{}, the plan ₪{}",
        same.after_selling,
        outcome.after_selling
    );
    Ok(())
}

// ─────────────────────────── More options ───────────────────────────

/// Deposits that grow 10% a year: the second year's are 10% larger, and so
/// on, and they add up to what's deposited.
#[test]
fn growing_deposits_add_up() {
    let scenario = Scenario {
        first_deposit: dec!(5000),
        monthly_deposit: dec!(1000),
        deposit_growth: Percent(dec!(10)),
        yearly_return: Percent(Decimal::ZERO),
        years: 3,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    let deposits: Vec<Decimal> = scenario.monthly_deposits().collect();
    assert_eq!(deposits.len(), 36);
    assert_eq!(deposits[0], dec!(1000));
    assert_eq!(deposits[11], dec!(1000));
    assert_eq!(deposits[12], dec!(1100));
    assert_eq!(deposits[35], dec!(1210));
    // 5,000 + 12 × (1,000 + 1,100 + 1,210)
    assert_eq!(scenario.deposited(), dec!(44720));
    // With no fees and no growth, every deposit is still there at the end.
    let outcome = simulate(&free_plan(), &scenario, &rates()).unwrap();
    assert!(close(outcome.after_selling, dec!(44720)));
}

/// Deposits may shrink by up to all of themselves a year, not more, and the
/// check on what the arithmetic can hold counts their growth: ₪1,000 a
/// month growing 200% a year for 50 years is beyond it, 10% a year is not.
#[test]
fn the_check_counts_the_growth_of_deposits() {
    let growing = |percent, years| Scenario {
        first_deposit: Decimal::ZERO,
        monthly_deposit: dec!(1000),
        deposit_growth: Percent(percent),
        yearly_return: Percent(Decimal::ZERO),
        years,
        ..defaults(Security::IndexFund, Exchange::Tlv)
    };
    assert_eq!(growing(dec!(-100), 5).check(), Ok(()));
    assert_eq!(
        growing(dec!(-100.5), 5).check(),
        Err(InvalidScenario::DepositGrowthBelowMinus100)
    );
    // Deposits that stop after the first year.
    assert_eq!(growing(dec!(-100), 5).deposited(), dec!(12000));
    assert_eq!(
        growing(dec!(200), 50).check(),
        Err(InvalidScenario::TooLarge)
    );
    assert_eq!(growing(dec!(10), 50).check(), Ok(()));
    assert_eq!(growing(Decimal::ZERO, 50).check(), Ok(()));
    // Without growth, the same deposits over 50 years are ₪600,000, well within it.
    let outcome = simulate(&free_plan(), &growing(dec!(10), 50), &rates()).unwrap();
    assert!(close(
        outcome.after_selling,
        growing(dec!(10), 50).deposited()
    ));
}

/// In today's money at 0% inflation, nothing changes; at 2%, every amount at
/// the end is divided by 1.02 to the power of the years, and the amounts in
/// between by the months' share of that.
#[test]
fn todays_money_divides_by_the_rise_in_prices() {
    let scenario = defaults(Security::Etf, Exchange::Usa);
    let (name, plan) = usual_plans().remove(0);
    let nominal = simulate(&plan, &scenario, &rates()).unwrap();
    let same = nominal.in_todays_money(Percent(Decimal::ZERO));
    assert_eq!(same.after_selling, nominal.after_selling, "{name}");
    assert_eq!(same.value_by_month, nominal.value_by_month);
    assert_eq!(same.fees, nominal.fees);

    let real = nominal.in_todays_money(Percent(dec!(2)));
    let at_end = compounded(dec!(1.02), 20);
    assert!(about(real.after_selling * at_end, nominal.after_selling));
    assert!(about(real.held * at_end, nominal.held));
    assert!(about(real.fees.selling * at_end, nominal.fees.selling));
    assert!(about(real.tax * at_end, nominal.tax));
    assert!(about(real.after_tax * at_end, nominal.after_tax));
    assert_eq!(real.value_by_month[0], nominal.value_by_month[0]);
    // The end of the first year: one year's rise.
    assert!(about(
        real.value_by_month[12] * dec!(1.02),
        nominal.value_by_month[12]
    ));
    assert!(about(
        real.fees_by_year[0].purchases * dec!(1.02),
        nominal.fees_by_year[0].purchases
    ));
    // The fees add up as before, each year in its own money.
    let yearly: Fees = real.fees_by_year.iter().copied().sum();
    assert!(about(yearly.total() + real.fees.selling, real.fees.total()));
    assert!(real.fees.total() < nominal.fees.total());
    assert_eq!(real.yearly_cost, nominal.yearly_cost);
    assert_eq!(
        scenario.deposited_in_todays_money(Percent(Decimal::ZERO)),
        scenario.deposited()
    );
    assert!(scenario.deposited_in_todays_money(Percent(dec!(2))) < scenario.deposited());
}

/// The deposit sweep tries a range of deposits, evenly spaced on a log
/// scale, and at each its numbers are what the table would show for it.
#[test]
fn the_sweep_matches_the_table_at_each_deposit() {
    let scenario = defaults(Security::Etf, Exchange::Usa);
    assert_eq!(Swept::for_scenario(&scenario), Swept::Monthly);
    let lump_sum = Scenario {
        monthly_deposit: Decimal::ZERO,
        ..scenario.clone()
    };
    assert_eq!(Swept::for_scenario(&lump_sum), Swept::OneTime);

    let monthly = Swept::Monthly.amounts();
    assert_eq!(monthly.first(), Some(&dec!(100)));
    assert_eq!(monthly.last(), Some(&dec!(32000)));
    assert_eq!(
        &monthly[..7],
        [
            dec!(100),
            dec!(150),
            dec!(220),
            dec!(320),
            dec!(460),
            dec!(680),
            dec!(1000)
        ]
    );
    let once = Swept::OneTime.amounts();
    assert_eq!(once.first(), Some(&dec!(1000)));
    assert_eq!(once.last(), Some(&dec!(4600000)));
    for amounts in [&monthly, &once] {
        assert!(
            amounts.windows(2).all(|pair| pair[0] < pair[1]),
            "{amounts:?}"
        );
    }

    let plans = usual_plans();
    let refs: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
    let swept = sweep(&refs, &scenario, &rates(), Swept::Monthly);
    assert_eq!(swept.amounts, monthly);
    assert_eq!(swept.costs.len(), plans.len());
    for ((name, plan), costs) in plans.iter().zip(&swept.costs) {
        assert_eq!(costs.len(), swept.amounts.len());
        for (&amount, &cost) in swept.amounts.iter().zip(costs) {
            let at_amount = Scenario {
                monthly_deposit: amount,
                ..scenario.clone()
            };
            let outcome = simulate(plan, &at_amount, &rates()).ok();
            assert_eq!(
                outcome.map(|outcome| outcome.yearly_cost),
                cost,
                "{name} at ₪{amount} a month"
            );
        }
        // A broker takes any deposit. The provident fund's line ends at its
        // ceiling: with the ₪10,000 of the first year's one-time deposit,
        // (83,641 − 10,000) / 12 = ₪6,136.75 a month is the most that fits.
        let takes_every_amount = costs.iter().all(Option::is_some);
        let has_ceiling = plan.vehicle.rules().deposit_ceiling.is_some();
        assert_eq!(takes_every_amount, !has_ceiling, "{name}");
        if has_ceiling {
            let taken = costs.iter().filter(|cost| cost.is_some()).count();
            assert!(swept.amounts[taken - 1] <= dec!(6136.75), "{name}");
            assert!(swept.amounts[taken] > dec!(6136.75), "{name}");
            assert!(costs[taken..].iter().all(Option::is_none), "{name}");
        }
    }
    // Sweeping the one-time deposit varies that one: with nothing monthly,
    // the cost falls as the lump sum grows, and each point is the table's.
    let lump_sum = Scenario {
        monthly_deposit: Decimal::ZERO,
        ..scenario.clone()
    };
    let once = sweep(&refs, &lump_sum, &rates(), Swept::OneTime);
    assert_eq!(once.amounts, Swept::OneTime.amounts());
    for ((name, plan), costs) in plans.iter().zip(&once.costs) {
        for (&amount, &cost) in once.amounts.iter().zip(costs) {
            let at_amount = Scenario {
                first_deposit: amount,
                ..lump_sum.clone()
            };
            let outcome = simulate(plan, &at_amount, &rates()).ok();
            assert_eq!(
                outcome.map(|outcome| outcome.yearly_cost),
                cost,
                "{name} at ₪{amount} once"
            );
        }
        // A broker's fixed fees weigh less on a larger sum; a manager's
        // share is the same at any size.
        if plan.management.is_none() {
            assert!(costs.first() > costs.last(), "{name}: {costs:?}");
        }
    }

    // A plan that doesn't offer the security has no point at any amount.
    let altshuler = plan_of(&plans, "Altshuler");
    let europe = Scenario {
        exchange: Exchange::Europe,
        ..scenario
    };
    let none = sweep(&[altshuler], &europe, &rates(), Swept::Monthly);
    assert_eq!(none.costs, [vec![None; none.amounts.len()]]);
}

/// Around the user's deposit, the cheapest plan's lead ends where another
/// plan's cost meets its own, the costs taken to change steadily between
/// the amounts tried, on a logarithmic scale of amounts. Worked out:
/// - below ₪2,000, the first plan is no dearer at ₪1,000 (1% against 2%)
///   but dearer at ₪100 (3% against 1%): the gaps are −1 and +2, so they
///   meet a third of the way down, at 1,000 × (100 / 1,000)^⅓ = ₪464, which
///   is ₪460 to two digits;
/// - above, the third plan: its gaps are 0.9 − 1.2 = −0.3 at ₪2,000 and
///   0.5 − 0.3 = +0.2 at ₪10,000, so they meet 0.6 of the way up, at
///   2,000 × 5^0.6 = ₪5,253, which is ₪5,300.
#[test]
fn around_the_deposit_the_nearest_crossings_are_found() {
    let costs = |costs: &[Decimal]| -> Vec<Option<Percent>> {
        costs.iter().copied().map(Percent).map(Some).collect()
    };
    let sweep = Sweep {
        swept: Swept::Monthly,
        amounts: vec![dec!(100), dec!(1000), dec!(10000)],
        costs: vec![
            costs(&[dec!(3), dec!(1), dec!(0.5)]),
            costs(&[dec!(1), dec!(2), dec!(2)]),
            costs(&[dec!(5), dec!(1.5), dec!(0.3)]),
            // Not offered: never cheaper.
            vec![None; 3],
        ],
    };
    let at_deposit = [
        Some(Percent(dec!(0.9))),
        Some(Percent(dec!(2))),
        Some(Percent(dec!(1.2))),
        None,
    ];
    assert_eq!(
        sweep.around(0, dec!(2000), &at_deposit),
        Some(Around {
            from: dec!(100),
            to: dec!(10000),
            below: Some(Crossing {
                amount: dec!(460),
                plan: 1
            }),
            above: Some(Crossing {
                amount: dec!(5300),
                plan: 2
            }),
        })
    );
    // Only the cheapest plan at the user's deposit has a lead to lose.
    assert_eq!(sweep.around(1, dec!(2000), &at_deposit), None);
    // A plan as cheap isn't cheaper: a tie at the user's deposit keeps the
    // lead, and one at an amount tried isn't a crossing.
    let tied = [
        Some(Percent(dec!(0.9))),
        Some(Percent(dec!(0.9))),
        Some(Percent(dec!(1.2))),
        None,
    ];
    assert!(sweep.around(0, dec!(2000), &tied).is_some());
    let level = Sweep {
        costs: vec![
            costs(&[dec!(1), dec!(1), dec!(1)]),
            costs(&[dec!(1), dec!(1), dec!(1)]),
        ],
        ..sweep.clone()
    };
    let level_at = [Some(Percent(dec!(1))), Some(Percent(dec!(1)))];
    assert_eq!(
        level
            .around(0, dec!(2000), &level_at)
            .map(|around| (around.below, around.above)),
        Some((None, None))
    );
    // Nothing to say without a deposit, or with costs for other plans.
    assert_eq!(sweep.around(0, Decimal::ZERO, &at_deposit), None);
    assert_eq!(sweep.around(0, dec!(2000), &at_deposit[..3]), None);
    // At an amount tried, the user's own costs are the ones used: with the
    // second plan at 0.5% there, it's the cheapest, and stays so below.
    let at_1000 = [
        Some(Percent(dec!(1))),
        Some(Percent(dec!(0.5))),
        Some(Percent(dec!(1.5))),
        None,
    ];
    let around = sweep.around(1, dec!(1000), &at_1000).unwrap();
    assert_eq!(around.below, None);
    assert_eq!(around.above.map(|crossing| crossing.plan), Some(0));
    // Past the amounts tried, the range reaches the user's deposit. The
    // third plan is dearer there (0.45% against 0.4%) and cheaper at
    // ₪10,000: the gaps −0.05 and +0.2 meet a fifth of the way down, at
    // 50,000 × (10,000 / 50,000)^0.2 = ₪36,239, which is ₪36,000.
    let at_50000 = [
        Some(Percent(dec!(0.4))),
        Some(Percent(dec!(2))),
        Some(Percent(dec!(0.45))),
        None,
    ];
    assert_eq!(
        sweep.around(0, dec!(50000), &at_50000),
        Some(Around {
            from: dec!(100),
            to: dec!(50000),
            below: Some(Crossing {
                amount: dec!(36000),
                plan: 2
            }),
            above: None,
        })
    );
}

/// A fund's line ends at its ceiling: the range ends at the last amount it
/// takes, and what another plan costs past it is no crossing.
#[test]
fn the_cheapest_range_ends_where_the_plan_cant_be_used() {
    let some = |costs: &[Decimal]| -> Vec<Option<Percent>> {
        costs.iter().copied().map(Percent).map(Some).collect()
    };
    let with_a_ceiling = Sweep {
        swept: Swept::Monthly,
        amounts: vec![dec!(100), dec!(1000), dec!(10000)],
        costs: vec![
            vec![Some(Percent(dec!(0.6))), Some(Percent(dec!(0.6))), None],
            some(&[dec!(3), dec!(1), dec!(0.2)]),
        ],
    };
    let at_500 = [Some(Percent(dec!(0.6))), Some(Percent(dec!(1.5)))];
    assert_eq!(
        with_a_ceiling.around(0, dec!(500), &at_500),
        Some(Around {
            from: dec!(100),
            to: dec!(1000),
            below: None,
            above: None,
        })
    );
    // And from the first amount it takes, when smaller ones can't use it.
    let from_1000 = Sweep {
        costs: vec![
            vec![None, Some(Percent(dec!(0.6))), Some(Percent(dec!(0.6)))],
            some(&[dec!(0.1), dec!(1), dec!(2)]),
        ],
        ..with_a_ceiling.clone()
    };
    let at_5000 = [Some(Percent(dec!(0.6))), Some(Percent(dec!(1.5)))];
    assert_eq!(
        from_1000.around(0, dec!(5000), &at_5000),
        Some(Around {
            from: dec!(1000),
            to: dec!(10000),
            below: None,
            above: None,
        })
    );
    // The other way: the second plan is the cheapest at ₪20,000, where the
    // fund takes nothing. The fund is cheaper at ₪1,000 and can't be used
    // at ₪10,000, so ₪1,000 is where it's known to be cheaper from.
    let at_20000 = [None, Some(Percent(dec!(0.1)))];
    assert_eq!(
        with_a_ceiling.around(1, dec!(20000), &at_20000),
        Some(Around {
            from: dec!(100),
            to: dec!(20000),
            below: Some(Crossing {
                amount: dec!(1000),
                plan: 0
            }),
            above: None,
        })
    );
}

proptest! {
    // A sweep is a comparison's work at each amount tried, so fewer cases.
    #![proptest_config(ProptestConfig { cases: 16, failure_persistence: None, ..ProptestConfig::default() })]

    /// With the real plans: the cheapest plan at the user's deposit is the
    /// cheapest at every amount tried between it and the nearest crossing
    /// each way, and the plan named past a crossing is cheaper at an amount
    /// tried beyond it. Two significant digits move a crossing by 5% at most,
    /// hence the margin.
    #[test]
    fn the_cheapest_plan_stays_cheapest_up_to_its_crossings(scenario in scenarios()) {
        let plans = usual_plans();
        let refs: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
        let swept = Swept::for_scenario(&scenario);
        let deposit = match swept {
            Swept::Monthly => scenario.monthly_deposit,
            Swept::OneTime => scenario.first_deposit,
        };
        let comparison = compare(&refs, &scenario, &rates());
        let mut at_deposit = vec![None; refs.len()];
        for compared in &comparison.plans {
            at_deposit[compared.index] = compared.outcome.as_ref().ok().map(|outcome| outcome.yearly_cost);
        }
        // The cheapest in fees, which the tax can keep from being the
        // comparison's best.
        let Some(best) = (0..refs.len()).filter(|&plan| at_deposit[plan].is_some()).min_by_key(|&plan| at_deposit[plan]) else {
            return Ok(());
        };
        let sweep = sweep(&refs, &scenario, &rates(), swept);
        let around = sweep.around(best, deposit, &at_deposit);
        prop_assert!(around.is_some(), "the cheapest plan isn't the cheapest at ₪{}", deposit);
        let around = around.unwrap();

        // Only the amounts the cheapest plan can be used at count.
        let points: Vec<(Decimal, Vec<Option<Percent>>)> = sweep
            .amounts
            .iter()
            .enumerate()
            .map(|(index, &amount)| (amount, sweep.costs.iter().map(|costs| costs[index]).collect()))
            .filter(|(amount, _)| (around.from..=around.to).contains(amount))
            .collect();
        let cheapest = |costs: &[Option<Percent>]| costs.iter().flatten().all(|&cost| costs[best].unwrap() <= cost);
        let cheaper = |costs: &[Option<Percent>], plan: usize| costs[plan].is_some_and(|cost| cost < costs[best].unwrap());
        let margin = dec!(1.05);

        let lowest = around.below.map_or(Decimal::ZERO, |crossing| crossing.amount * margin);
        for (amount, costs) in points.iter().filter(|(amount, _)| *amount < deposit && *amount >= lowest) {
            prop_assert!(cheapest(costs), "not the cheapest at ₪{} a {:?} deposit of ₪{}: {:?}", amount, swept, deposit, around);
        }
        if let Some(crossing) = around.below {
            prop_assert!(crossing.amount <= deposit, "{:?}", around);
            prop_assert!(
                points.iter().any(|(amount, costs)| *amount <= crossing.amount * margin && cheaper(costs, crossing.plan)),
                "plan {} isn't cheaper below the crossing: {:?}", crossing.plan, around
            );
        }

        let highest = around.above.map(|crossing| crossing.amount / margin);
        for (amount, costs) in points.iter().filter(|(amount, _)| *amount > deposit && highest.is_none_or(|highest| *amount <= highest)) {
            prop_assert!(cheapest(costs), "not the cheapest at ₪{} a {:?} deposit of ₪{}: {:?}", amount, swept, deposit, around);
        }
        if let Some(crossing) = around.above {
            prop_assert!(crossing.amount >= deposit, "{:?}", around);
            prop_assert!(
                points.iter().any(|(amount, costs)| *amount >= crossing.amount / margin && cheaper(costs, crossing.plan)),
                "plan {} isn't cheaper above the crossing: {:?}", crossing.plan, around
            );
        }

        prop_assert!(around.from <= deposit && deposit <= around.to, "{:?}", around);
        if sweep.costs[best].iter().all(Option::is_some) {
            prop_assert_eq!(around.from, sweep.amounts[0].min(deposit));
            prop_assert_eq!(around.to, sweep.amounts[sweep.amounts.len() - 1].max(deposit));
        }
    }
}

/// `rate` compounded over `years`: 1.02 for 20 years. Decimal has no power
/// without a feature.
fn compounded(rate: Decimal, years: u32) -> Decimal {
    (0..years).fold(Decimal::ONE, |total, _| total * rate)
}

/// Equal to within a billionth of the amount, and a millionth of a shekel:
/// the rise in prices is compounded from a monthly factor that came
/// through floating point, so on millions of shekels the last digits drift.
fn about(a: Decimal, b: Decimal) -> bool {
    (a - b).abs() <= a.abs().max(b.abs()) * dec!(0.000000001) + dec!(0.000001)
}

/// The listed plan of the broker whose short name starts `name`.
fn plan_of<'a>(plans: &'a [(String, Plan)], name: &str) -> &'a Plan {
    &plans
        .iter()
        .find(|(label, _)| label.starts_with(name))
        .unwrap_or_else(|| panic!("no plan of {name}"))
        .1
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn the_yearly_cost_holds_for_any_pattern(scenario in scenarios()) {
        let (no_fees, outcomes) = outcomes(&scenario);
        for (name, outcome) in outcomes {
            assert_yearly_cost_holds(&name, &scenario, &outcome, &no_fees)?;
        }
    }

    /// Ranking by yearly cost is ranking by what's left, the other way up.
    #[test]
    fn the_yearly_cost_ranks_like_what_is_left(scenario in scenarios()) {
        let (_, mut outcomes) = outcomes(&scenario);
        outcomes.sort_by_key(|(_, outcome)| std::cmp::Reverse(outcome.after_selling));
        let costs: Vec<Percent> = outcomes.iter().map(|(_, outcome)| outcome.yearly_cost).collect();
        prop_assert!(costs.windows(2).all(|pair| pair[0] <= pair[1]), "{costs:?}");
    }

    /// Keeping the holdings at the end: nothing is paid for selling, what's
    /// held is what's left, and it's never less than selling would leave.
    #[test]
    fn keeping_the_holdings_pays_nothing_at_the_end(scenario in scenarios()) {
        let kept = Scenario { sell_at_end: false, ..scenario.clone() };
        let (no_fees, sold) = outcomes(&scenario);
        let (no_fees_kept, kept_outcomes) = outcomes(&kept);
        prop_assert_eq!(no_fees_kept.after_selling, no_fees.held);
        for (name, kept) in &kept_outcomes {
            // Ranked differently, perhaps: paired by name.
            let (_, sold) = sold.iter().find(|(sold, _)| sold == name).unwrap();
            prop_assert_eq!(kept.fees.selling, Decimal::ZERO, "{}", name);
            prop_assert_eq!(kept.after_selling, kept.held, "{}", name);
            // Nothing is sold, so no gain is taxed yet.
            prop_assert_eq!(kept.tax, Decimal::ZERO, "{}", name);
            prop_assert_eq!(kept.after_tax, kept.held, "{}", name);
            // The cheapest track for keeping may not be the one for selling.
            if kept.track == sold.track {
                prop_assert_eq!(kept.held, sold.held, "{}", name);
            }
            prop_assert!(kept.after_selling >= sold.after_selling, "{}: kept ₪{}, sold ₪{}", name, kept.after_selling, sold.after_selling);
            // Only purchases count as orders now.
            prop_assert!(kept.largest_trade <= sold.largest_trade, "{}", name);
        }
    }

    /// Restating everything in today's money changes no ranking, and what's
    /// lost to fees shrinks by the same factor as everything else.
    #[test]
    fn todays_money_keeps_the_ranking(scenario in scenarios()) {
        let plans = listed_plans();
        let refs: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
        let nominal = compare(&refs, &scenario, &rates());
        let real = nominal.clone().in_todays_money(Percent(dec!(3)));
        let order = |comparison: &Comparison| comparison.plans.iter().map(|plan| plan.index).collect::<Vec<_>>();
        prop_assert_eq!(order(&nominal), order(&real));
        let at_end = compounded(dec!(1.03), scenario.years);
        for (before, after) in nominal.plans.iter().zip(&real.plans) {
            let (Ok(before), Ok(after)) = (&before.outcome, &after.outcome) else { continue };
            let lost_before = before.lost_to_fees(&nominal.no_fees);
            let lost_after = after.lost_to_fees(&real.no_fees);
            prop_assert!(about(lost_after * at_end, lost_before), "lost ₪{} nominal, ₪{} real", lost_before, lost_after);
        }
    }
}

// ─────────────────────────── Money for the short term ───────────────────────────

mod short_term_economics {
    use super::close;
    use broker_fees::short_term::{
        self, NotOffered, Pays, Place, Rates, Scenario, Term, compare, simulate,
    };
    use broker_fees::{Percent, Text};
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use strum::EnumCount;

    /// What someone might keep: up to ₪5 million at the start and up to
    /// ₪10,000 a month (at least ₪1,000 in all), for 1 to 60 months, at a
    /// rate of −1% to 8% in quarters, with prices rising 0% to 6%.
    fn sums() -> impl Strategy<Value = Scenario> {
        (0..=5000u32, 0..=20u32, 1..=60u32, -4..=32i32, 0..=6u32).prop_map(
            |(thousands, five_hundreds, months, quarters, inflation)| Scenario {
                first_deposit: Decimal::from(thousands.max(u32::from(five_hundreds == 0)))
                    * dec!(1000),
                monthly_deposit: Decimal::from(five_hundreds) * dec!(500),
                months,
                rate: Percent(Decimal::from(quarters) * dec!(0.25)),
                inflation: Percent(Decimal::from(inflation)),
            },
        )
    }

    /// The same money as one sum at the start, which a deposit can take.
    fn one_sum(scenario: &Scenario) -> Scenario {
        Scenario {
            first_deposit: scenario.deposited(),
            monthly_deposit: Decimal::ZERO,
            ..scenario.clone()
        }
    }

    fn place(pays: Pays) -> Place {
        Place {
            name: Text::same("Made up"),
            short_name: Text::same("Made up"),
            description: Text::same(""),
            pays,
            caveats: vec![],
            compared_at_first: true,
        }
    }

    /// A fund taking `hundredths` of a percent a year.
    fn fund(hundredths: u32) -> Place {
        place(Pays::TheRateLess(Percent(
            Decimal::from(hundredths) / dec!(100),
        )))
    }

    /// A deposit paying `rate` for every term.
    fn deposit(rate: Percent) -> Place {
        place(Pays::Fixed(Rates::new([Some(rate); Term::COUNT])))
    }

    fn every_listed_place() -> Vec<Place> {
        short_term::kinds()
            .into_iter()
            .flat_map(|kind| kind.places)
            .collect()
    }

    /// Every rate the Bank of Israel published is between 0% and 10% a
    /// year, every bank has one for some term, and every fund's fee is under
    /// 1%: a typo in the data would show here.
    #[test]
    fn every_listed_figure_is_plausible() {
        for place in every_listed_place() {
            match place.pays {
                Pays::Fixed(rates) => {
                    let published: Vec<Percent> =
                        Term::iter().filter_map(|term| rates.of(term)).collect();
                    assert!(!published.is_empty(), "{}", place.name.en);
                    for rate in published {
                        assert!(
                            rate.0 > dec!(0) && rate.0 < dec!(10),
                            "{}: {rate}",
                            place.name.en
                        );
                    }
                }
                Pays::TheRateLess(fee) => assert!(
                    fee.0 >= dec!(0) && fee.0 < dec!(1),
                    "{}: {fee}",
                    place.name.en
                ),
            }
        }
    }

    use strum::IntoEnumIterator;

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, failure_persistence: None, ..ProptestConfig::default() })]

        /// A fund takes its fee from what the rate makes, so in no month is
        /// it worth more than the money at the rate.
        #[test]
        fn no_fund_beats_the_rate(scenario in sums(), fee in 0..=100u32) {
            let outcome = simulate(&fund(fee), &scenario).unwrap();
            for (month, (value, at_the_rate)) in outcome.value_by_month.iter().zip(scenario.at_the_rate()).enumerate() {
                prop_assert!(*value <= at_the_rate + dec!(0.01), "month {month}: ₪{value} against ₪{at_the_rate}");
            }
            prop_assert!(outcome.kept >= dec!(-0.01));
        }

        /// A dearer fund never leaves more, and a deposit paying more never
        /// leaves less.
        #[test]
        fn a_dearer_fund_or_a_poorer_deposit_never_helps(scenario in sums(), fee in 0..=100u32, rate in 0..=32u32) {
            let left = |place: &Place, scenario: &Scenario| simulate(place, scenario).unwrap().after_tax;
            prop_assert!(left(&fund(fee), &scenario) >= left(&fund(fee + 5), &scenario) - dec!(0.01));
            let rate = Percent(Decimal::from(rate) * dec!(0.25));
            let more = Percent(rate.0 + dec!(0.25));
            let one_sum = one_sum(&scenario);
            prop_assert!(left(&deposit(more), &one_sum) >= left(&deposit(rate), &one_sum));
        }

        /// A deposit's tax is 15% of its interest; a fund's is at most 25% of
        /// its gain; neither is ever negative; and what's left is what it's
        /// worth less the tax.
        #[test]
        fn the_tax_is_the_laws(scenario in sums(), fee in 0..=100u32, rate in 0..=32u32) {
            let one_sum = one_sum(&scenario);
            let deposit = simulate(&deposit(Percent(Decimal::from(rate) * dec!(0.25))), &one_sum).unwrap();
            prop_assert_eq!(deposit.tax, deposit.earned * dec!(0.15));
            let fund = simulate(&fund(fee), &scenario).unwrap();
            prop_assert!(fund.tax >= Decimal::ZERO);
            prop_assert!(fund.tax <= (fund.earned * dec!(0.25)).max(Decimal::ZERO));
            for (outcome, scenario) in [(deposit, one_sum), (fund, scenario)] {
                let end = *outcome.value_by_month.last().unwrap();
                prop_assert_eq!(outcome.after_tax, end - outcome.tax);
                prop_assert_eq!(outcome.earned, end - scenario.deposited());
            }
        }

        /// Growing at the rate less its fee, a fund's yearly cost is its fee,
        /// however many months it's kept.
        #[test]
        fn a_funds_yearly_cost_is_its_fee(scenario in sums(), fee in 0..=100u32) {
            let outcome = simulate(&fund(fee), &scenario).unwrap();
            let fee = Decimal::from(fee) / dec!(100);
            prop_assert!((outcome.yearly_cost.0 - fee).abs() <= dec!(0.0002), "cost {}, fee {fee}", outcome.yearly_cost);
        }

        /// A deposit paying the rate, kept for whole years, ends where the
        /// money at the rate does: its interest compounds once a year, and a
        /// year at the rate is 1 + the rate.
        #[test]
        fn a_deposit_at_the_rate_keeps_nothing_over_whole_years(scenario in sums(), years in 1..=5u32) {
            let scenario = Scenario { months: years * 12, ..one_sum(&scenario) };
            let outcome = simulate(&deposit(scenario.rate), &scenario).unwrap();
            prop_assert!(close(outcome.kept, Decimal::ZERO), "kept ₪{}", outcome.kept);
        }

        /// A fixed-rate deposit is one sum: with money put in every month,
        /// no bank's is offered, while every fund is.
        #[test]
        fn a_deposit_takes_one_sum(scenario in sums()) {
            prop_assume!(!scenario.monthly_deposit.is_zero());
            for place in every_listed_place() {
                let outcome = simulate(&place, &scenario);
                match place.pays {
                    Pays::Fixed(_) => prop_assert_eq!(outcome, Err(NotOffered::TakesOneSum), "{}", place.name.en),
                    Pays::TheRateLess(_) => prop_assert!(outcome.is_ok(), "{}", place.name.en),
                }
            }
        }

        /// The comparison ranks every listed place by what's left after tax,
        /// most first, and puts the ones that can't be used last.
        #[test]
        fn the_comparison_ranks_by_what_is_left(scenario in sums()) {
            let places = every_listed_place();
            let refs: Vec<&Place> = places.iter().collect();
            let comparison = compare(&refs, &scenario);
            prop_assert_eq!(comparison.places.len(), places.len());
            let left: Vec<Option<Decimal>> = comparison.places.iter().map(|place| place.outcome.as_ref().ok().map(|o| o.after_tax)).collect();
            prop_assert!(left.windows(2).all(|pair| pair[0] >= pair[1]), "{left:?}");
            prop_assert_eq!(comparison.at_the_rate, scenario.at_the_rate());
        }
    }
}
