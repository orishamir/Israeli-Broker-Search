//! The ways to hold the S&P 500, side by side. A way is what the saver
//! actually chooses between: an index fund through a bank, an ETF in Tel
//! Aviv, a US ETF, a provident fund's track. Each is held through a product
//! that keeps part of the index back ([`Product`]) inside an account that
//! charges its own fees and taxes the gain its own way ([`Vehicle`]), so
//! the plans of a way are the accounts that can hold it, and the way is
//! worth what its best plan leaves.
//!
//! [`compare_ways`] runs every plan of every way on the same deposits and
//! returns the ways, best first. Within a way, [`simulation::compare`]'s rules
//! decide the plans' order (a plan the deposit doesn't open comes after the
//! ones it does, a tie in the shekel goes to the plan without a "may cost
//! more" caveat); between ways the same tie rule applies to each way's best.

use std::cmp::Reverse;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::describe::may_cost_more_text;
use crate::simulation::{PlanOutcome, Scenario, compare_with_broker_caveats};
use crate::{
    Broker, BrokerKind, Buying, Caveat, Exchange, ExchangeRates, Lang, Money, Percent, Plan,
    Product, Security, Text, Vehicle,
};

/// How the saver holds the index: the fund that holds it, the kind of
/// account it sits in, and where it's bought.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Way {
    pub product: Product,
    pub vehicle: Vehicle,
    /// Where a broker's account buys it; `None` for a fund's track, which
    /// the saver doesn't buy on any exchange.
    pub exchange: Option<Exchange>,
}

impl Way {
    /// The ways to hold the S&P 500, the one most Israelis start with
    /// first: an index fund in Tel Aviv through a bank or an investment
    /// house, an ETF in Tel Aviv, in the US, in Europe, then a provident
    /// fund for investment's S&P 500 track, a study fund's and a savings
    /// policy's.
    ///
    /// An ETF in Tel Aviv is the Israeli kind, the one most held there; the
    /// foreign ones listed there ([`Product::ForeignEtfInTelAviv`]) are
    /// cheaper and can be compared by a way of their own.
    #[must_use]
    pub fn for_index() -> Vec<Way> {
        let bought = |product, exchange| Way {
            product,
            vehicle: Vehicle::Brokerage,
            exchange: Some(exchange),
        };
        let track = |vehicle| Way {
            product: Product::track_of(vehicle).expect("a fund holds a track"),
            vehicle,
            exchange: None,
        };
        vec![
            bought(Product::IsraeliIndexFund, Exchange::Tlv),
            bought(Product::IsraeliEtf, Exchange::Tlv),
            bought(Product::UsFund, Exchange::Usa),
            bought(Product::IrishFund, Exchange::Europe),
            track(Vehicle::InvestmentGemel),
            track(Vehicle::StudyFund),
            track(Vehicle::SavingsPolicy),
        ]
    }

    /// What a broker's plans are asked to buy for this way: an index fund,
    /// or an ETF. A fund's plans hold their own track whatever is asked.
    #[must_use]
    pub fn security(self) -> Security {
        match self.product {
            Product::IsraeliIndexFund => Security::IndexFund,
            _ => Security::Etf,
        }
    }

    /// `base` bought this way: the same deposits and years, the way's
    /// security, exchange and product. A fund isn't bought on an exchange
    /// and converts nothing, so it's asked for in shekels, with the product
    /// a Tel Aviv index fund would be (the scenario's product has to be one
    /// that's bought where it's bought; the fund's plans hold their own
    /// track instead).
    #[must_use]
    pub fn scenario(self, base: &Scenario) -> Scenario {
        let (exchange, product) = match self.exchange {
            Some(exchange) => (exchange, self.product),
            None => (Exchange::Tlv, Product::IsraeliIndexFund),
        };
        let security = if self.exchange.is_some() {
            self.security()
        } else {
            Security::IndexFund
        };
        Scenario {
            security,
            exchange,
            product: Some(product),
            ..base.clone()
        }
    }

    /// Its name as the guide says it: what the product is, then the index it
    /// follows, the S&P 500 (REDESIGN.md §2.8). A fund's ticker is never part
    /// of the name: it's an example to search for, said in "how to do it".
    #[must_use]
    pub fn kind_name(self, lang: Lang) -> &'static str {
        match (self.product, self.vehicle) {
            (Product::IsraeliIndexFund, _) => lang.pick(
                "Index fund (קרן מחקה) on the S&P 500",
                "קרן מחקה על מדד S&P\u{a0}500",
            ),
            (Product::IsraeliEtf, _) => lang.pick(
                "ETF (קרן סל) on the S&P 500, in Tel Aviv",
                "קרן סל על מדד S&P\u{a0}500, בתל אביב",
            ),
            (Product::ForeignEtfInTelAviv, _) => lang.pick(
                "Irish ETF (קרן סל) on the S&P 500, bought in Tel Aviv",
                "קרן סל אירית על מדד S&P\u{a0}500, בתל אביב",
            ),
            (Product::UsFund, _) => lang.pick(
                "ETF (קרן סל) on the S&P 500, in the US",
                "קרן סל על מדד S&P\u{a0}500, בארה״ב",
            ),
            (Product::IrishFund, _) => lang.pick(
                "Irish ETF (קרן סל) on the S&P 500, bought in Europe",
                "קרן סל אירית על מדד S&P\u{a0}500, באירופה",
            ),
            (_, Vehicle::InvestmentGemel) => lang.pick(
                "Provident fund for investment (קופת גמל להשקעה), S&P 500 track",
                "קופת גמל להשקעה, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::StudyFund) => lang.pick(
                "Study fund (קרן השתלמות), S&P 500 track",
                "קרן השתלמות, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::SavingsPolicy | Vehicle::Brokerage) => lang.pick(
                "Savings policy (פוליסת חיסכון), S&P 500 track",
                "פוליסת חיסכון, מסלול מדד S&P\u{a0}500",
            ),
        }
    }

    /// Its name in the workbench, where the ways are compared side by side:
    /// where it's bought, and for a fund abroad an example ticker, marked as
    /// an example (principle 17).
    #[must_use]
    pub fn name(self, lang: Lang) -> &'static str {
        match (self.product, self.vehicle) {
            (Product::IsraeliIndexFund, _) => lang.pick(
                "Index fund (קרן מחקה) in Tel Aviv",
                "קרן מחקה בתל אביב, דרך בנק או בית השקעות",
            ),
            (Product::IsraeliEtf, _) => {
                lang.pick("ETF (קרן סל) in Tel Aviv", "קרן סל ישראלית בבורסה בתל אביב")
            }
            (Product::UsFund, _) => lang.pick(
                "ETF (קרן סל) in the US, for example VOO",
                "קרן סל אמריקאית בבורסה בארה״ב, למשל VOO",
            ),
            (Product::IrishFund, _) => lang.pick(
                "ETF (קרן סל) in Europe, for example CSPX",
                "קרן סל אירית בבורסה באירופה, למשל CSPX",
            ),
            (_, Vehicle::InvestmentGemel) => lang.pick(
                "Provident fund for investment (קופת גמל להשקעה), S&P 500 track",
                "קופת גמל להשקעה, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::StudyFund) => lang.pick(
                "Study fund (קרן השתלמות), S&P 500 track",
                "קרן השתלמות, מסלול מדד S&P\u{a0}500",
            ),
            (_, Vehicle::SavingsPolicy | Vehicle::Brokerage) => lang.pick(
                "Savings policy (פוליסת חיסכון), S&P 500 track",
                "פוליסת חיסכון, מסלול מדד S&P\u{a0}500",
            ),
        }
    }
}

/// A plan that can hold some way, with the caveats of the broker it comes
/// from (which decide a tie, see [`Plan::has_may_cost_more`]).
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    pub plan: &'a Plan,
    pub broker_caveats: &'a [Caveat],
}

/// Every plan worth comparing, in the order of `brokers`: all of a bank's or
/// an investment house's, and of a kind of fund its average-fee plan only,
/// since the cheapest, the dearest and the most allowed are how far one
/// company's fee can be from the average and not plans to pick between
/// ([`crate::Spread`]). A saver's own plans join by being added to the list
/// with no caveats.
#[must_use]
pub fn candidates(brokers: &[Broker]) -> Vec<Candidate<'_>> {
    brokers
        .iter()
        .flat_map(|broker| {
            let plans = match broker.kind {
                BrokerKind::Funds => {
                    &broker.plans[broker.new_customer_plan..=broker.new_customer_plan]
                }
                BrokerKind::Bank | BrokerKind::InvestmentHouse => &broker.plans[..],
            };
            plans.iter().map(|plan| Candidate {
                plan,
                broker_caveats: &broker.caveats,
            })
        })
        .collect()
}

/// One way, with every plan that could hold it.
#[derive(Debug, Clone)]
pub struct WayOutcome {
    pub way: Way,
    /// Every candidate of the way's vehicle, best first: [`PlanOutcome::index`]
    /// is the plan's place in the list given to [`compare_ways`]. The ones
    /// that can't be used (a security not sold, a locked fund) come last,
    /// with why.
    pub plans: Vec<PlanOutcome>,
    /// What the best plan loses a year to the product and its own fees
    /// together, stated as a fund's fee is ([`Outcome::yearly_cost`](crate::simulation::Outcome::yearly_cost)).
    /// `None` when no plan can be used.
    pub cost_a_year: Option<Percent>,
    /// Whether the best plan's price, or the product's, may be too low: a
    /// tie against another way goes to the one without.
    pub may_cost_more: bool,
    /// Why, in words: the plan's reason, or else the product's.
    pub warning: Option<Text>,
    /// Set by [`WayOutcome::put_unflagged_first`]: the plan (its place in the
    /// candidates) that left more but may cost more, and now stands behind
    /// the best.
    pub flagged_ahead: Option<usize>,
}

impl WayOutcome {
    /// The plan that leaves the most after tax among the ones the deposits
    /// open and the way can use, if there is one.
    #[must_use]
    pub fn best(&self) -> Option<&PlanOutcome> {
        self.plans
            .first()
            .filter(|plan| plan.fits && plan.outcome.is_ok())
    }

    /// For the guide, which names one plan: if the best plan may cost more
    /// (a caveat that its price is too low), the best plan without such a
    /// caveat takes its place at the front and the flagged one stays right
    /// behind it, noted in `flagged_ahead`. The table (`compare_ways`) keeps
    /// its order; only a tie in the shekel goes to the unflagged plan there.
    /// Nothing changes when the best plan has no caveat or every plan has one.
    pub fn put_unflagged_first(
        &mut self,
        base: &Scenario,
        candidates: &[Candidate<'_>],
        rates: &ExchangeRates,
    ) {
        let scenario = self.way.scenario(base);
        let flagged = |plan: &PlanOutcome| plan_may_cost_more(plan, candidates, &scenario, rates);
        let Some(best) = self.best() else { return };
        if !flagged(best) {
            return;
        }
        let Some(at) = self
            .plans
            .iter()
            .position(|plan| plan.fits && plan.outcome.is_ok() && !flagged(plan))
        else {
            return;
        };
        self.flagged_ahead = Some(self.plans[0].index);
        let plan = self.plans.remove(at);
        self.plans.insert(0, plan);
        describe_best(self, candidates, &scenario, rates);
    }

    /// What the best plan leaves after selling and the tax.
    #[must_use]
    pub fn after_tax(&self) -> Option<Decimal> {
        self.best()
            .and_then(|best| best.outcome.as_ref().ok())
            .map(|outcome| outcome.after_tax)
    }
}

/// Runs `base`, once for each of [`Way::for_index`], on the candidates that
/// can hold it, and returns the ways best first: by what the best plan
/// leaves after tax, a tie in the shekel going to the way whose price can't
/// be too low, and the ways with no plan that can be used last.
///
/// `base` says how much is deposited and for how long; its security,
/// exchange and product are the ways'.
#[must_use]
pub fn compare_ways(
    base: &Scenario,
    candidates: &[Candidate<'_>],
    rates: &ExchangeRates,
) -> Vec<WayOutcome> {
    compare_chosen_ways(&Way::for_index(), base, candidates, rates)
}

/// [`compare_ways`] for some of the ways only: the guide asks about the ones
/// that can be opened any time, or about a single one, and a way costs a
/// run of every plan that can hold it.
#[must_use]
pub fn compare_chosen_ways(
    chosen: &[Way],
    base: &Scenario,
    candidates: &[Candidate<'_>],
    rates: &ExchangeRates,
) -> Vec<WayOutcome> {
    let mut ways: Vec<WayOutcome> = chosen
        .iter()
        .map(|&way| compare_way(way, base, candidates, rates))
        .collect();
    sort_ways(&mut ways);
    ways
}

/// Best first: what the best plan leaves after tax, a tie in the shekel
/// going to the way whose price can't be too low.
pub fn sort_ways(ways: &mut [WayOutcome]) {
    ways.sort_by_key(|way| {
        let after_tax = way.after_tax();
        (
            after_tax.is_none(),
            Reverse(after_tax.map(|left| left.round_dp(0))),
            way.may_cost_more,
            Reverse(after_tax),
        )
    });
}

fn compare_way(
    way: Way,
    base: &Scenario,
    candidates: &[Candidate<'_>],
    rates: &ExchangeRates,
) -> WayOutcome {
    let scenario = way.scenario(base);
    let chosen: Vec<usize> = (0..candidates.len())
        .filter(|&at| candidates[at].plan.vehicle == way.vehicle)
        .collect();
    let plans: Vec<&Plan> = chosen.iter().map(|&at| candidates[at].plan).collect();
    let broker_caveats: Vec<&[Caveat]> = chosen
        .iter()
        .map(|&at| candidates[at].broker_caveats)
        .collect();
    let mut plans = compare_with_broker_caveats(&plans, &broker_caveats, &scenario, rates).plans;
    // Back to the places in the list the caller gave.
    for plan in &mut plans {
        plan.index = chosen[plan.index];
    }

    let mut outcome = WayOutcome {
        way,
        plans,
        cost_a_year: None,
        may_cost_more: false,
        warning: None,
        flagged_ahead: None,
    };
    describe_best(&mut outcome, candidates, &scenario, rates);
    outcome
}

/// What buying looks like for one plan's outcome: its largest trade decides
/// which caveats matter.
fn buying_of(scenario: &Scenario, outcome: Option<&crate::simulation::Outcome>) -> Buying {
    Buying {
        security: scenario.security,
        exchange: scenario.exchange,
        product: scenario.product,
        largest_trade: outcome
            .map(|used| Money::from_decimal(used.largest_trade, scenario.exchange.currency())),
    }
}

/// Whether the plan's own price, or its broker's, may be too low.
fn plan_may_cost_more(
    plan: &PlanOutcome,
    candidates: &[Candidate<'_>],
    scenario: &Scenario,
    rates: &ExchangeRates,
) -> bool {
    let candidate = candidates[plan.index];
    let buying = buying_of(scenario, plan.outcome.as_ref().ok());
    candidate
        .plan
        .has_may_cost_more(buying, candidate.broker_caveats, rates)
}

/// Fills the fields that describe the best plan: its cost a year and
/// whether, and why, its price may be too low.
fn describe_best(
    outcome: &mut WayOutcome,
    candidates: &[Candidate<'_>],
    scenario: &Scenario,
    rates: &ExchangeRates,
) {
    let way = outcome.way;
    outcome.cost_a_year = None;
    outcome.may_cost_more = false;
    outcome.warning = None;
    if let Some(best) = outcome.best() {
        let used = best.outcome.as_ref().ok();
        let candidate = candidates[best.index];
        let buying = buying_of(scenario, used);
        let plan_may = candidate
            .plan
            .has_may_cost_more(buying, candidate.broker_caveats, rates);
        let product_may = way.product.caveats().iter().any(|caveat| {
            caveat.may_cost_more_summary().is_some() && caveat.matters_for(buying, rates)
        });
        outcome.cost_a_year = used.map(|used| used.yearly_cost);
        outcome.may_cost_more = plan_may || product_may;
        outcome.warning = may_cost_more_text(
            candidate
                .broker_caveats
                .iter()
                .chain(&candidate.plan.caveats),
            buying,
            rates,
        )
        .or_else(|| may_cost_more_text(way.product.caveats().iter(), buying, rates));
    }
}
