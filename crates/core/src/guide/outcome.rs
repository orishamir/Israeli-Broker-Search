//! The figures behind the answer's parts (REDESIGN.md §2.6, §2.7, §2.9): each
//! option's value in a version of the world, the market's range at the
//! saver's date, the places that sell a product (from the saver's bank),
//! the gain split, and the rows of "what if the date moves".
//!
//! Every figure is asked of the engines that already exist (the short term's
//! for the safe kinds, the simulation for a plan), after tax and in today's
//! money; nothing here invents one.

use rust_decimal::Decimal;

use crate::short_term;
use crate::simulation::{self, Scenario};
use crate::ways::{Way, WayOutcome};
use crate::{Broker, BrokerKind, ExchangeRates, Percent, Text};

use super::index::{Cand, Run};
use super::robust::{FundFee, Version};
use super::safe::{self, Priced, price_level};
use super::{Amount, Answers, Bank, Data, Kind};

// ─────────────────────────── The safe kinds ───────────────────────────

/// The saver's deposits over `months` at a version's rate and inflation.
pub(super) fn safe_scenario(
    answers: &Answers,
    months: u32,
    version: &Version,
) -> short_term::Scenario {
    short_term::Scenario {
        first_deposit: answers.now,
        monthly_deposit: answers.monthly,
        months,
        rate: version.rate,
        inflation: version.inflation,
    }
}

/// A safe kind at its best place in a version, or `None` where it can't
/// hold the money.
pub(super) fn safe_priced(
    kind: Kind,
    answers: &Answers,
    months: u32,
    version: &Version,
    data: &Data,
) -> Option<Priced> {
    let scenario = safe_scenario(answers, months, version);
    safe::price(
        kind,
        data,
        &scenario,
        price_level(version.inflation, months),
    )
}

/// What a safe kind leaves in a version, in today's money.
pub(super) fn safe_value(
    kind: Kind,
    answers: &Answers,
    months: u32,
    version: &Version,
    data: &Data,
) -> Option<Decimal> {
    safe_priced(kind, answers, months, version, data).map(|priced| priced.left.today)
}

// ─────────────────────────── A plan of a way ───────────────────────────

/// The plan a candidate stands for in a version: a fund's cheapest company
/// when the version is about the fee a saver can negotiate.
fn plan_in<'a>(cand: &Cand<'a>, version: &Version) -> &'a crate::Plan {
    let broker: &'a Broker = cand.broker;
    match (version.fund_fee, broker.kind) {
        (FundFee::Negotiated, BrokerKind::Funds) => broker
            .spread
            .and_then(|spread| spread.cheapest)
            .and_then(|at| broker.plans.get(at))
            .unwrap_or(cand.candidate.plan),
        _ => cand.candidate.plan,
    }
}

/// The scenario of a way in a version: the index's return and inflation
/// changed, the deposits and years the same.
fn way_scenario(way: Way, base: &Scenario, version: &Version) -> Scenario {
    Scenario {
        yearly_return: version.index_return,
        inflation: version.inflation,
        ..way.scenario(base)
    }
}

/// One plan of a way, run in a version: the engine's outcome in today's
/// money.
pub(super) fn plan_outcome(
    cand: &Cand<'_>,
    way: Way,
    base: &Scenario,
    version: &Version,
    rates: &ExchangeRates,
) -> Option<simulation::Outcome> {
    let scenario = way_scenario(way, base, version);
    simulation::simulate(plan_in(cand, version), &scenario, rates)
        .ok()
        .map(|outcome| outcome.in_todays_money(version.inflation))
}

/// What the best plan of a way leaves in a version, in today's money.
pub(super) fn way_value(
    run: &Run<'_>,
    way: &WayOutcome,
    version: &Version,
    rates: &ExchangeRates,
) -> Option<Decimal> {
    let best = way.best()?;
    plan_outcome(&run.cands[best.index], way.way, &run.base, version, rates)
        .map(|outcome| outcome.after_tax)
}

// ─────────────────────────── Where to buy ───────────────────────────

/// A place that sells the product, with what it leaves.
#[derive(Debug, Clone)]
pub(super) struct Place {
    pub name: Text,
    /// Who it is, without the plan: what a sentence names.
    pub provider: Text,
    pub left: Amount,
    /// A bank (whose app the saver may already have).
    pub bank: Option<Bank>,
    /// The saver's own bank's plan: the one compared from.
    pub yours: bool,
    /// What a year costs, where the engine says.
    pub cost_a_year: Option<Percent>,
    /// Where it is in the run's candidates, for an index way.
    pub cand: Option<usize>,
}

/// Which modelled bank a broker is, if it's one.
pub(super) fn bank_of(broker: &Broker) -> Option<Bank> {
    use strum::IntoEnumIterator;
    if broker.kind != BrokerKind::Bank {
        return None;
    }
    Bank::iter().find(|bank| broker.short_name.en == bank.short_name())
}

/// The plans of an index way that the deposits open, best first, each with
/// the bank it is and whether it's the saver's own: their bank's online plan
/// (what a new customer gets), or for a portfolio already at a bank, its
/// branch plan.
pub(super) fn way_places(
    run: &Run<'_>,
    way: &WayOutcome,
    yours: Option<Bank>,
    existing: bool,
    level: Decimal,
) -> Vec<Place> {
    way.plans
        .iter()
        .filter(|plan| plan.fits)
        .filter_map(|plan| {
            let outcome = plan.outcome.as_ref().ok()?;
            let cand = &run.cands[plan.index];
            let broker = cand.broker;
            let bank = bank_of(broker);
            let online = broker
                .plans
                .get(broker.new_customer_plan)
                .is_some_and(|usual| std::ptr::eq(usual, cand.candidate.plan));
            // A branch plan is compared only as a portfolio's own.
            if bank.is_some() && !online && !cand.yours {
                return None;
            }
            let is_yours = if existing {
                cand.yours
            } else {
                online && bank.is_some() && bank == yours
            };
            Some(Place {
                name: super::safe::joined(&broker.short_name, &cand.candidate.plan.name),
                provider: broker.short_name.clone(),
                left: super::safe::amount(outcome.after_tax, level),
                bank: if cand.yours && !existing { None } else { bank },
                yours: is_yours,
                cost_a_year: Some(outcome.yearly_cost),
                cand: Some(plan.index),
            })
        })
        .collect()
}

/// The deposits' bank whose short name the tariffs give a bank.
fn deposit_bank_name(bank: Bank) -> &'static str {
    match bank {
        Bank::Beinleumi => "First International",
        other => other.short_name(),
    }
}

/// The places of a safe kind, best first: a bond's are the accounts that buy
/// it, a deposit's are the banks.
pub(super) fn safe_places(priced: &Priced, data: &Data, yours: Option<Bank>) -> Vec<Place> {
    use strum::IntoEnumIterator;
    let banks_by_name = |name: &Text| -> Option<Bank> {
        match priced.kind {
            Kind::GovernmentBond { .. } => data
                .brokers
                .iter()
                .filter(|broker| broker.kind == BrokerKind::Bank)
                .find(|broker| {
                    name.en.starts_with(&*broker.short_name.en)
                        || name.en.starts_with(&*broker.name.en)
                })
                .and_then(bank_of),
            _ => Bank::iter().find(|bank| name.en.contains(deposit_bank_name(*bank))),
        }
    };
    priced
        .places
        .iter()
        .filter(|place| place.fits)
        .map(|place| {
            let bank = banks_by_name(&place.name);
            Place {
                provider: provider_of(&place.name),
                name: place.name.clone(),
                left: place.left,
                bank,
                yours: bank.is_some() && bank == yours,
                cost_a_year: place.cost_a_year,
                cand: None,
            }
        })
        .collect()
}

/// The provider's part of a "provider · plan" name.
fn provider_of(name: &Text) -> Text {
    let before = |text: &str, mark: &str| text.split(mark).next().unwrap_or(text).trim().to_owned();
    Text::owned(before(&name.en, " · "), before(&name.he, "\u{a0}· "))
}

/// The lowest and highest the banks' places leave, if any bank sells it.
pub(super) fn bank_range(places: &[Place]) -> Option<(Amount, Amount)> {
    let banks: Vec<Amount> = places
        .iter()
        .filter(|place| place.bank.is_some())
        .map(|place| place.left)
        .collect();
    let low = banks.iter().copied().min_by(|a, b| a.today.cmp(&b.today))?;
    let high = banks.iter().copied().max_by(|a, b| a.today.cmp(&b.today))?;
    Some((low, high))
}

/// Ties among places: each takes the group of the first place within
/// `margin` above it (best first), so the page shows ties as ties.
pub(super) fn tie_groups(places: &[Place], margin: Decimal) -> Vec<u8> {
    let mut groups = Vec::with_capacity(places.len());
    let mut head: Option<Decimal> = None;
    let mut group: u8 = 0;
    for place in places {
        match head {
            Some(top) if top - place.left.today <= margin => {}
            Some(_) => {
                group = group.saturating_add(1);
                head = Some(place.left.today);
            }
            None => head = Some(place.left.today),
        }
        groups.push(group);
    }
    groups
}

// ─────────────────────────── The gain ───────────────────────────

/// Where the gain of one plan went, in today's money: what the saver keeps
/// above the deposits, the tax, the broker's and the manager's fees, and the
/// product's own cost.
#[derive(Debug, Clone, Copy)]
pub(super) struct Split {
    pub kept: Decimal,
    pub tax: Decimal,
    pub fees: Decimal,
    pub product: Decimal,
}

pub(super) fn split(outcome: &simulation::Outcome, deposited_today: Decimal) -> Split {
    Split {
        kept: outcome.after_tax - deposited_today,
        tax: outcome.tax,
        fees: outcome.fees.total() - outcome.fees.product,
        product: outcome.fees.product,
    }
}

// ─────────────────────────── Months and money ───────────────────────────

/// The deposits put end to end in today's money, and the same left in a
/// current account at the end.
pub(super) fn yardsticks(answers: &Answers, months: u32, inflation: Percent) -> (Amount, Amount) {
    let deposited = answers.now + answers.monthly * Decimal::from(months);
    let level = price_level(inflation, months);
    let put_in = Amount {
        today: safe::deposited_in_todays_money(answers.now, answers.monthly, months, inflation),
        nominal: deposited,
    };
    (put_in, safe::amount(deposited, level))
}

/// The same `today` figure with its nominal one at the end of `months`.
pub(super) fn at_end(today: Decimal, months: u32, inflation: Percent) -> Amount {
    Amount {
        today,
        nominal: today * price_level(inflation, months),
    }
}
