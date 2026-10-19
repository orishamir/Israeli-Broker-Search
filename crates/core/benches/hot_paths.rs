//! How long the core's work takes, for the calls the web app makes on every
//! keystroke: the comparison of every listed plan, one plan's description,
//! and the editor's views. Run with `cargo bench -p broker-fees`; divan
//! prints the timings, so a change that slows the app shows up as a number.

use broker_fees::guide;
use broker_fees::simulation::{Scenario, compare, simulate};
use broker_fees::ways::{candidates, compare_ways};
use broker_fees::{
    Buying, Exchange, ExchangeRates, Lang, Percent, Plan, Product, Security, Withdrawal, listed,
    tariffs,
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
        product: Product::for_purchase(security, exchange).first().copied(),
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

/// Every way to hold the S&P 500, each on all the plans that can hold it:
/// the guide's and the workbench's table, on every input change.
#[divan::bench]
fn compare_every_way(bencher: divan::Bencher) {
    let brokers = listed();
    let candidates = candidates(&brokers);
    let scenario = defaults(Security::IndexFund, Exchange::Tlv);
    let rates = rates();
    bencher.bench_local(|| compare_ways(&scenario, &candidates, &rates));
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

// ─────────────────────────── The guide ───────────────────────────

fn guide_answers(
    now: rust_decimal::Decimal,
    monthly: rust_decimal::Decimal,
    horizon: guide::Horizon,
) -> guide::Answers {
    guide::Answers {
        situation: None,
        now,
        monthly,
        horizon,
        flexibility: Some(guide::Flexibility::Fixed),
        cushion: Some(guide::Cushion::HasNot {
            monthly_expenses: Some(dec!(17000)),
        }),
        needs_any_day: false,
        bank: None,
    }
}

fn guide_bench(bencher: divan::Bencher, answers: &guide::Answers) {
    let data = guide::Data::listed();
    let settings = guide::Settings::new(rates());
    bencher.bench_local(|| guide::recommend(answers, &settings, &data).unwrap());
}

/// The tariffs and rates the guide works on, built once by the page.
#[divan::bench]
fn guide_data(bencher: divan::Bencher) {
    bencher.bench_local(guide::Data::listed);
}

/// A fixed date in five years: the safe kinds, a bond among them, and one
/// stock fund for the alternatives.
#[divan::bench]
fn recommend_safe(bencher: divan::Bencher) {
    guide_bench(
        bencher,
        &guide_answers(dec!(50000), dec!(0), guide::Horizon::Months(60)),
    );
}

/// Twenty years of deposits: every way on the new customers' plans, the safe
/// kinds beside them.
#[divan::bench]
fn recommend_index(bencher: divan::Bencher) {
    guide_bench(
        bencher,
        &guide_answers(dec!(10000), dec!(2000), guide::Horizon::Months(240)),
    );
}

/// Thirty years with no first deposit: plans a first deposit would open are
/// tried too (the unlock line).
#[divan::bench]
fn recommend_index_with_unlock(bencher: divan::Bencher) {
    guide_bench(
        bencher,
        &guide_answers(dec!(0), dec!(500), guide::Horizon::Months(360)),
    );
}

/// The cushion first, then the index kinds that can be opened any time.
#[divan::bench]
fn recommend_cushion(bencher: divan::Bencher) {
    guide_bench(
        bencher,
        &guide_answers(dec!(0), dec!(3000), guide::Horizon::Unknown),
    );
}

/// The ladder's five rungs (six for a sum alone), three of them with
/// every way: the Web Worker's work.
#[divan::bench(args = [0, 3000])]
fn ladder_rungs(bencher: divan::Bencher, monthly: u32) {
    let data = guide::Data::listed();
    let settings = guide::Settings::new(rates());
    let answers = guide_answers(
        dec!(15000),
        rust_decimal::Decimal::from(monthly),
        guide::Horizon::Unknown,
    );
    bencher.bench_local(|| guide::ladder(&answers, &settings, &data).unwrap());
}
