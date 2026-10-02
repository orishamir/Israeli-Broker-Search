//! How long the core's work takes, for the calls the web app makes on every
//! keystroke: the comparison of every listed plan, one plan's description,
//! and the editor's views. Run with `cargo bench -p broker-fees`; divan
//! prints the timings, so a change that slows the app shows up as a number.

use broker_fees::simulation::{Scenario, compare, simulate};
use broker_fees::{
    Buying, Exchange, ExchangeRates, Lang, Percent, Plan, Security, Withdrawal, tariffs,
};
use rust_decimal_macros::dec;

fn main() {
    divan::main();
}

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

fn every_plan() -> Vec<Plan> {
    tariffs::all()
        .into_iter()
        .flat_map(|broker| broker.plans)
        .collect()
}

/// Every listed plan on the defaults: what one input change costs.
#[divan::bench(args = [Exchange::Tlv, Exchange::Usa])]
fn compare_every_plan(bencher: divan::Bencher, exchange: Exchange) {
    let plans = every_plan();
    let refs: Vec<&Plan> = plans.iter().collect();
    let scenario = defaults(Security::Etf, exchange);
    let rates = rates();
    bencher.bench_local(|| compare(&refs, &scenario, &rates));
}

/// One plan, on its cheapest track, over 20 years.
#[divan::bench]
fn simulate_one_plan(bencher: divan::Bencher) {
    let altshuler = tariffs::altshuler();
    let scenario = defaults(Security::Etf, Exchange::Usa);
    let rates = rates();
    bencher.bench_local(|| simulate(&altshuler.plans[0], &scenario, &rates));
}

/// A plan's fees in words, with its caveats sorted: the details dialog.
#[divan::bench]
fn describe_fees(bencher: divan::Bencher) {
    let leumi = tariffs::leumi();
    let buying = Buying::any_amount(Security::Etf, Exchange::Usa);
    let rates = rates();
    bencher
        .bench_local(|| leumi.describe_fees_for(&leumi.plans[3], buying, None, &rates, Lang::En));
}

/// The editor's full view of a copy, compared with its original.
#[divan::bench]
fn price_list(bencher: divan::Bencher) {
    let leumi = tariffs::leumi();
    let copy = leumi.plans[3].copy_of(None);
    bencher.bench_local(|| copy.price_list(Some(&leumi.plans[3]), Lang::En));
}

/// A plan through JSON and back: what crossing to JavaScript costs.
#[divan::bench]
fn plan_through_json(bencher: divan::Bencher) {
    let plan = tariffs::ibi().plans.remove(0);
    bencher.bench_local(|| {
        let json = serde_json::to_string(&plan).unwrap();
        serde_json::from_str::<Plan>(&json).unwrap()
    });
}
