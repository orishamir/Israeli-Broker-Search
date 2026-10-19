//! The index kinds, priced through [`crate::ways`]: the plans of the saver's
//! brokers and funds, run on the deposits for each way to hold the index.

use rust_decimal::Decimal;

use crate::describe::may_cost_more_text;
use crate::simulation::{self, PlanOutcome, Scenario};
use crate::ways::{Candidate, Way, WayOutcome, compare_chosen_ways};
use crate::{Broker, Buying, Exchange, ExchangeRates, Money, Security, Text, Vehicle, Withdrawal};

use super::safe::{amount, joined};
use super::{
    Answers, BRANCH_PLAN, Choice, Data, Horizon, Kind, PlanRef, Ranked, Settings, Situation,
    USUAL_AGE, USUAL_SHARE_PRICE, UnlockLine,
};

/// The years an index kind is worked out for, from a number of months: the
/// simulation counts whole years, so 70 months is 6 years.
pub(super) fn years_for(months: u32) -> u32 {
    (months.saturating_add(6) / 12).max(1)
}

/// The deposits for `years`, bought through each way in turn (a way sets the
/// security, exchange and product).
pub(super) fn base_scenario(answers: &Answers, settings: &Settings, years: u32) -> Scenario {
    // A pension from the provident fund opens at 60: whoever retires at 60
    // or later takes the money that way, which is the tax-free one.
    let (age, withdrawal) = match answers.horizon {
        Horizon::Retirement { age, retire_at } if retire_at >= 60 => (age, Withdrawal::Pension),
        Horizon::Retirement { age, .. } => (age, Withdrawal::LumpSum),
        Horizon::Months(_) | Horizon::Unknown => (USUAL_AGE, Withdrawal::LumpSum),
    };
    Scenario {
        security: Security::IndexFund,
        exchange: Exchange::Tlv,
        first_deposit: answers.now,
        monthly_deposit: answers.monthly,
        deposit_growth: crate::Percent(Decimal::ZERO),
        yearly_return: settings.index_return,
        product: None,
        years,
        buy_every_months: 1,
        share_price: USUAL_SHARE_PRICE,
        sell_at_end: true,
        inflation: settings.inflation,
        age,
        withdrawal,
        redeems_at_maturity: false,
    }
}

/// A plan that can hold some way, where it is in the data.
pub(super) struct Cand<'a> {
    pub broker: &'a Broker,
    pub candidate: Candidate<'a>,
    /// The saver's own bank's branch plan.
    pub yours: bool,
}

/// Every branch plan of a bank (the plan for an order given to a banker or
/// at a branch).
fn branch_plans(data: &Data) -> Vec<PlanRef> {
    data.brokers
        .iter()
        .enumerate()
        .flat_map(|(broker, listed)| {
            listed
                .plans
                .iter()
                .enumerate()
                .filter(|(_, plan)| {
                    plan.name.en == BRANCH_PLAN && plan.vehicle == Vehicle::Brokerage
                })
                .map(move |(plan, _)| PlanRef { broker, plan })
        })
        .collect()
}

/// The branch plan of the saver's bank. For "another bank" the one that
/// leaves the least after tax on these deposits, held in an index fund in
/// Tel Aviv: not cheaper than any bank's could be shown to be, and said so.
fn branch_plan(
    bank: Option<super::Bank>,
    data: &Data,
    base: &Scenario,
    rates: &ExchangeRates,
) -> Option<PlanRef> {
    let branches = branch_plans(data);
    if let Some(bank) = bank {
        branches
            .into_iter()
            .find(|at| data.brokers[at.broker].short_name.en == bank.short_name())
    } else {
        let scenario = Way::for_index()[0].scenario(base);
        branches.into_iter().min_by_key(|at| {
            let plan = &data.brokers[at.broker].plans[at.plan];
            simulation::simulate(plan, &scenario, rates)
                .map_or(Decimal::MAX, |outcome| outcome.after_tax)
        })
    }
}

/// The plans compared: the ones ticked (or, by default, what a new customer
/// usually gets at each broker, and a fund's average), and the branch plan
/// when the saver already has a portfolio at a bank.
pub(super) fn candidates<'a>(
    answers: &Answers,
    settings: &Settings,
    data: &'a Data,
    base: &Scenario,
) -> Vec<Cand<'a>> {
    let mut chosen: Vec<PlanRef> = settings.ticked.clone().unwrap_or_else(|| {
        data.brokers
            .iter()
            .enumerate()
            .map(|(broker, listed)| PlanRef {
                broker,
                plan: listed.new_customer_plan,
            })
            .collect()
    });
    let mut yours = None;
    if let Some(Situation::ExistingPortfolio { bank }) = answers.situation {
        yours = branch_plan(bank, data, base, &settings.rates);
        if let Some(at) = yours
            && !chosen.contains(&at)
        {
            chosen.push(at);
        }
    }
    chosen
        .into_iter()
        .filter_map(|at| {
            let broker = data.brokers.get(at.broker)?;
            let plan = broker.plans.get(at.plan)?;
            Some(Cand {
                broker,
                candidate: Candidate {
                    plan,
                    broker_caveats: &broker.caveats,
                },
                yours: Some(at) == yours,
            })
        })
        .collect()
}

/// Some ways run on the deposits.
pub(super) struct Run<'a> {
    pub base: Scenario,
    pub cands: Vec<Cand<'a>>,
    /// Best first.
    pub ways: Vec<WayOutcome>,
}

pub(super) fn run<'a>(
    ways: &[Way],
    base: Scenario,
    cands: Vec<Cand<'a>>,
    rates: &ExchangeRates,
) -> Run<'a> {
    let candidates: Vec<Candidate<'_>> = cands.iter().map(|cand| cand.candidate).collect();
    let ways = compare_chosen_ways(ways, &base, &candidates, rates);
    Run { base, cands, ways }
}

/// The ways that can be opened at any time, with no waiting and no
/// ceiling that turns a large sum away: a broker's account and a provident
/// fund for investment (a study fund is locked six years, and a savings
/// policy is not what someone with no date is offered).
pub(super) fn open_any_time() -> Vec<Way> {
    Way::for_index()
        .into_iter()
        .filter(|way| matches!(way.vehicle, Vehicle::Brokerage | Vehicle::InvestmentGemel))
        .collect()
}

/// Whether anyone can open the account the way holds the index in. A study
/// fund opens for the self-employed, or through an employer: the guide
/// shows it with who can open one rather than naming it to someone who
/// may not be able to.
pub(super) fn open_to_anyone(way: Way) -> bool {
    way.vehicle != Vehicle::StudyFund
}

pub(super) fn way_text(way: Way) -> Text {
    Text::new(way.name(crate::Lang::En), way.name(crate::Lang::He))
}

impl Run<'_> {
    /// The ways that have a plan the deposits open, best first, as a ranking.
    pub fn ranked(&self, level: Decimal) -> Vec<Ranked> {
        self.ways
            .iter()
            .filter_map(|way| {
                let best = way.best()?;
                let outcome = best.outcome.as_ref().ok()?;
                Some(Ranked {
                    kind: Kind::Way(way.way),
                    name: way_text(way.way),
                    left: amount(outcome.after_tax, level),
                    cost_a_year: way.cost_a_year,
                    may_cost_more: way.may_cost_more,
                    warning: way.warning.clone(),
                    fees: Some(amount(outcome.fees.total(), level)),
                    tax: Some(amount(outcome.tax, level)),
                    fits: true,
                    yours: self.cands[best.index].yours,
                })
            })
            .collect()
    }

    /// The plans of one way, best first.
    pub fn plans_ranked(
        &self,
        way: &WayOutcome,
        level: Decimal,
        rates: &ExchangeRates,
    ) -> Vec<Ranked> {
        way.plans
            .iter()
            .filter_map(|plan| {
                let outcome = plan.outcome.as_ref().ok()?;
                let cand = &self.cands[plan.index];
                let scenario = way.way.scenario(&self.base);
                let buying = Buying {
                    security: scenario.security,
                    exchange: scenario.exchange,
                    product: scenario.product,
                    largest_trade: Some(Money::from_decimal(
                        outcome.largest_trade,
                        scenario.exchange.currency(),
                    )),
                };
                let may_cost_more = cand.candidate.plan.has_may_cost_more(
                    buying,
                    cand.candidate.broker_caveats,
                    rates,
                );
                let warning = may_cost_more_text(
                    cand.candidate
                        .broker_caveats
                        .iter()
                        .chain(&cand.candidate.plan.caveats),
                    buying,
                    rates,
                );
                Some(Ranked {
                    kind: Kind::Way(way.way),
                    name: joined(&cand.broker.short_name, &cand.candidate.plan.name),
                    left: amount(outcome.after_tax, level),
                    cost_a_year: Some(outcome.yearly_cost),
                    may_cost_more,
                    warning,
                    fees: Some(amount(outcome.fees.total(), level)),
                    tax: Some(amount(outcome.tax, level)),
                    fits: plan.fits,
                    yours: cand.yours,
                })
            })
            .collect()
    }

    /// Who runs the best plan of a way, and on what plan (and track).
    pub fn choice(&self, way: &WayOutcome) -> Option<Choice> {
        let best = way.best()?;
        Some(self.choice_of(best))
    }

    fn choice_of(&self, plan: &PlanOutcome) -> Choice {
        let cand = &self.cands[plan.index];
        let name = &cand.candidate.plan.name;
        let track = plan
            .outcome
            .as_ref()
            .ok()
            .and_then(|outcome| outcome.track)
            .and_then(|track| cand.candidate.plan.tracks.get(track));
        Choice {
            provider: cand.broker.short_name.clone(),
            plan: match track {
                Some(track) => joined(name, &track.name),
                None => name.clone(),
            },
        }
    }

    /// What a bigger first deposit would open that leaves more than `chosen`
    /// does at that same deposit, if anything: the smallest such deposit.
    /// Each way's plans are tried with [`simulation::unlock`] (which keeps
    /// the deposit the same for every plan), and the plan it finds is held
    /// against the way that was recommended, so a bigger sum alone never
    /// counts as a plan's doing.
    pub fn unlock(
        &self,
        chosen: &WayOutcome,
        level: Decimal,
        rates: &ExchangeRates,
    ) -> Option<UnlockLine> {
        let first = self.base.first_deposit;
        if self
            .cands
            .iter()
            .all(|cand| cand.candidate.plan.opens_with(first))
        {
            return None;
        }
        let best = chosen.best()?;
        let mut found: Vec<(simulation::Unlock, Way, usize)> = Vec::new();
        for way in &self.ways {
            let at: Vec<usize> = (0..self.cands.len())
                .filter(|&at| self.cands[at].candidate.plan.vehicle == way.way.vehicle)
                .collect();
            let plans: Vec<&crate::Plan> =
                at.iter().map(|&at| self.cands[at].candidate.plan).collect();
            let caveats: Vec<&[crate::Caveat]> = at
                .iter()
                .map(|&at| self.cands[at].candidate.broker_caveats)
                .collect();
            if let Some(unlock) =
                simulation::unlock(&plans, &caveats, &way.way.scenario(&self.base), rates)
            {
                let plan = at[unlock.plan];
                found.push((unlock, way.way, plan));
            }
        }
        found.sort_by_key(|(unlock, _, _)| {
            (unlock.first_deposit, std::cmp::Reverse(unlock.after_tax))
        });
        found.into_iter().find_map(|(unlock, way, plan)| {
            let bigger = Scenario {
                first_deposit: unlock.first_deposit,
                ..chosen.way.scenario(&self.base)
            };
            let instead =
                simulation::simulate(self.cands[best.index].candidate.plan, &bigger, rates)
                    .ok()?
                    .after_tax;
            (unlock.after_tax > instead).then(|| {
                let opened = PlanOutcome {
                    index: plan,
                    outcome: simulation::simulate(
                        self.cands[plan].candidate.plan,
                        &Scenario {
                            first_deposit: unlock.first_deposit,
                            ..way.scenario(&self.base)
                        },
                        rates,
                    ),
                    fits: true,
                };
                UnlockLine {
                    first_deposit: unlock.first_deposit,
                    plan: self.choice_of(&opened),
                    kind: Kind::Way(way),
                    after_tax: amount(unlock.after_tax, level),
                    instead_of: amount(instead, level),
                }
            })
        })
    }
}
