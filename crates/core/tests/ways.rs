//! The ways to hold the S&P 500 side by side (`ways.rs`): which plans each
//! way is run on, that its best is the plan that leaves the most, and how
//! the ways are ordered. Numbers come from the core.

use std::cmp::Reverse;
use std::collections::HashSet;

use broker_fees::simulation::{Scenario, free_plan, simulate};
use broker_fees::ways::{Candidate, Way, WayOutcome, candidates, compare_ways};
use broker_fees::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// €1 = $1.25 = ₪4.625, so $1 = ₪3.70. Not real rates.
fn rates() -> ExchangeRates {
    ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
}

/// The app's defaults: ₪10,000, then ₪2,000 a month for 20 years at 10%.
/// The ways set the security, the exchange and the product.
fn defaults() -> Scenario {
    Scenario {
        security: Security::IndexFund,
        exchange: Exchange::Tlv,
        product: None,
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
        redeems_at_maturity: false,
    }
}

/// The ways for `scenario` on every listed plan.
fn ways(scenario: &Scenario) -> Vec<WayOutcome> {
    let brokers = listed();
    compare_ways(scenario, &candidates(&brokers), &rates())
}

fn way_of(ways: &[WayOutcome], product: Product) -> &WayOutcome {
    ways.iter().find(|way| way.way.product == product).unwrap()
}

fn may_cost_more() -> Caveat {
    Caveat::may_cost_more(
        Text::new("a price taken on the cheap side", "מחיר שהוערך בזול"),
        Text::new("may cost more", "עשוי לעלות יותר"),
    )
}

#[test]
fn every_way_is_a_product_held_where_it_can_be() {
    let all = Way::for_index();
    assert_eq!(all.len(), 7);
    assert_eq!(all[0].product, Product::IsraeliIndexFund, "the first way");
    for way in all {
        match way.exchange {
            // Bought through a broker's account, on an exchange that sells it.
            Some(exchange) => {
                assert_eq!(way.vehicle, Vehicle::Brokerage);
                let bought = Product::for_purchase(way.security(), exchange);
                assert!(bought.contains(&way.product), "{way:?}");
            }
            // A fund's own track.
            None => assert_eq!(Product::track_of(way.vehicle), Some(way.product)),
        }
        // The way's scenario is one the core accepts.
        way.scenario(&defaults()).check().unwrap();
        assert_ne!(way.name(Lang::En), way.name(Lang::He));
    }
}

/// The best plan of a way is the one that leaves the most after tax among
/// the plans the first deposit opens and the way can use: each plan is
/// simulated here on its own, not read from the comparison.
#[test]
fn a_ways_best_is_the_maximum_of_its_plans() {
    let brokers = listed();
    let all = candidates(&brokers);
    let scenario = defaults();
    for outcome in compare_ways(&scenario, &all, &rates()) {
        let name = outcome.way.name(Lang::En);
        let asked = outcome.way.scenario(&scenario);
        let of_the_vehicle: Vec<usize> = (0..all.len())
            .filter(|&at| all[at].plan.vehicle == outcome.way.vehicle)
            .collect();
        // Every plan of the vehicle is among the way's, once.
        let places: HashSet<usize> = outcome.plans.iter().map(|plan| plan.index).collect();
        assert_eq!(places, of_the_vehicle.iter().copied().collect(), "{name}");
        assert_eq!(places.len(), outcome.plans.len(), "{name}");

        let most = of_the_vehicle
            .iter()
            .filter(|&&at| all[at].plan.opens_with(scenario.first_deposit))
            .filter_map(|&at| simulate(all[at].plan, &asked, &rates()).ok())
            .map(|simulated| simulated.after_tax)
            .max()
            .expect(name);
        assert_eq!(outcome.after_tax(), Some(most), "{name}");
    }
}

#[test]
fn the_ways_are_listed_best_first_and_the_unusable_last() {
    let scenario = defaults();
    let ranked = ways(&scenario);
    assert_eq!(ranked.len(), Way::for_index().len());
    let left: Vec<Option<Decimal>> = ranked.iter().map(WayOutcome::after_tax).collect();
    for pair in left.windows(2) {
        match pair {
            [Some(a), Some(b)] => assert!(a.round_dp(0) >= b.round_dp(0), "{left:?}"),
            [None, Some(_)] => panic!("an unusable way before a usable one: {left:?}"),
            _ => {}
        }
    }
    // A study fund's money is locked for six years: with three, it can't be
    // used, whatever its fee, and comes last.
    let short = ways(&Scenario {
        years: 3,
        ..scenario
    });
    let study = short.last().unwrap();
    assert_eq!(study.way.vehicle, Vehicle::StudyFund);
    assert!(study.best().is_none());
    assert_eq!(study.cost_a_year, None);
    assert!(
        short[..short.len() - 1]
            .iter()
            .all(|way| way.best().is_some())
    );
}

/// What a way costs a year is what its best plan loses to the product and
/// to its own fees together: more than the product's cost alone, since no
/// plan is free.
#[test]
fn a_ways_cost_is_the_products_and_the_plans_together() {
    for way in ways(&defaults()) {
        let name = way.way.name(Lang::En);
        let cost = way.cost_a_year.expect(name).0;
        let product = way.way.product.costs().yearly().0;
        assert!(
            cost > product,
            "{name}: {cost} against the product's {product}"
        );
        let best = way.best().unwrap().outcome.as_ref().unwrap();
        assert_eq!(way.cost_a_year, Some(best.yearly_cost), "{name}");
    }
}

/// A fund kind's way holds its average-fee plan and no other: the cheapest,
/// the dearest and the most allowed are only how far one company's fee is
/// from the average. The fee is the regulator's (`real_tariffs.rs` has the
/// numbers); here, that it is the plan the way runs.
#[test]
fn a_fund_kinds_way_holds_its_average_fee_plan_only() {
    let brokers = listed();
    let all = candidates(&brokers);
    let ranked = compare_ways(&defaults(), &all, &rates());
    for product in [
        Product::InvestmentGemelTrack,
        Product::StudyFundTrack,
        Product::SavingsPolicyTrack,
    ] {
        let way = way_of(&ranked, product);
        assert_eq!(way.plans.len(), 1, "{product:?}");
        let plan = all[way.plans[0].index].plan;
        assert_eq!(plan.name.en, "Average fee");
        assert_eq!(plan.vehicle, way.way.vehicle);
    }
}

/// An Israeli ETF on the S&P 500 costs more a year than an Israeli index
/// fund on it, and leaves less, at the defaults: measured over the five
/// years to September 2026 (`policies/index-tracking.py`, 0.82% against
/// 0.26% before the plans' own fees). If a re-measure turns this around,
/// the data is wrong or the world changed: look before editing the test.
#[test]
fn an_etf_in_tel_aviv_is_dearer_than_an_index_fund() {
    let ranked = ways(&defaults());
    let fund = way_of(&ranked, Product::IsraeliIndexFund);
    let etf = way_of(&ranked, Product::IsraeliEtf);
    assert!(fund.cost_a_year < etf.cost_a_year);
    assert!(fund.after_tax() > etf.after_tax());
}

/// Two plans that leave the same to the shekel: the one whose price may be
/// too low comes second, whichever the list had first, and whether it says
/// so itself or its broker does.
#[test]
fn a_tie_goes_to_the_plan_that_cannot_cost_more() {
    let plain = free_plan();
    let flagged = Plan {
        caveats: vec![may_cost_more()],
        ..free_plan()
    };
    let order_of = |candidates: &[Candidate<'_>]| -> Vec<usize> {
        let ranked = compare_ways(&defaults(), candidates, &rates());
        let way = way_of(&ranked, Product::IsraeliIndexFund);
        way.plans.iter().map(|plan| plan.index).collect()
    };
    let candidate = |plan| Candidate {
        plan,
        broker_caveats: &[],
    };
    assert_eq!(order_of(&[candidate(&plain), candidate(&flagged)]), [0, 1]);
    assert_eq!(order_of(&[candidate(&flagged), candidate(&plain)]), [1, 0]);

    let brokers_doubt = [may_cost_more()];
    let doubted = Candidate {
        plan: &plain,
        broker_caveats: &brokers_doubt,
    };
    assert_eq!(order_of(&[doubted, candidate(&plain)]), [1, 0]);
}

/// A way with no plan of its vehicle among the candidates has no best and
/// no cost, and comes after the ones that do.
#[test]
fn a_way_without_plans_is_empty_and_last() {
    let plan = free_plan();
    let only_a_broker = [Candidate {
        plan: &plan,
        broker_caveats: &[],
    }];
    let ranked = compare_ways(&defaults(), &only_a_broker, &rates());
    let (usable, empty) = ranked.split_at(4);
    assert!(usable.iter().all(|way| way.best().is_some()));
    for way in empty {
        assert!(way.plans.is_empty());
        assert!(way.best().is_none());
        assert_eq!(way.cost_a_year, None);
    }
}

/// A US fund warns of the estate tax above $60,000: where the holdings pass
/// it, the way "may cost more", and loses a tie.
#[test]
fn the_us_estate_tax_marks_the_way_above_sixty_thousand_dollars() {
    let small = Scenario {
        first_deposit: dec!(1000),
        monthly_deposit: dec!(100),
        years: 5,
        ..defaults()
    };
    let flagged = |scenario: &Scenario| {
        let ranked = ways(scenario);
        way_of(&ranked, Product::UsFund).may_cost_more
    };
    assert!(!flagged(&small));
    assert!(flagged(&defaults()));
}

/// Sorting is by what's left: reversing the deposits' size reorders nothing
/// it shouldn't. The ways' ranking equals the ranking of their bests.
#[test]
fn the_ranking_is_the_rank_of_the_bests() {
    let ranked = ways(&defaults());
    let mut by_left: Vec<_> = ranked
        .iter()
        .map(|way| (way.after_tax().unwrap().round_dp(0), way.may_cost_more))
        .collect();
    let given = by_left.clone();
    by_left.sort_by_key(|&(left, doubtful)| (Reverse(left), doubtful));
    assert_eq!(given, by_left);
}
