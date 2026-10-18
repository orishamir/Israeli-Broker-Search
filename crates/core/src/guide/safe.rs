//! The safe kinds, priced through [`crate::short_term`]: a money market fund,
//! a fixed-rate deposit, a government bond held to maturity. The guide asks
//! each for its best place and states the outcome in today's money.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::short_term::government_bonds::Linkage;
use crate::short_term::{self, Pays, Place};
use crate::{Percent, Text};

use super::{Amount, Choice, Data, Kind, Ranked};

/// How much prices have risen after `months` at `inflation` a year: what
/// money at the end is divided by to read as today's. The same figure
/// `simulation::Outcome::in_todays_money` divides by (its monthly factor,
/// applied once for each month).
pub(super) fn price_level(inflation: Percent, months: u32) -> Decimal {
    let yearly: f64 = (Decimal::ONE + inflation.of(Decimal::ONE))
        .to_f64()
        .unwrap_or(1.0);
    let monthly = Decimal::try_from(yearly.powf(1.0 / 12.0)).unwrap_or(Decimal::ONE);
    let level = (0..months).fold(Decimal::ONE, |level, _| level * monthly);
    // A rate of -99% makes it vanish; nothing can be divided by that.
    if level.is_zero() { Decimal::ONE } else { level }
}

/// `nominal`, at the end of the horizon, as today's money.
pub(super) fn amount(nominal: Decimal, level: Decimal) -> Amount {
    Amount {
        today: nominal / level,
        nominal,
    }
}

/// What the deposits come to put end to end in today's shekels: each
/// deposit divided by the prices of its month ([`crate::simulation::Scenario::deposited_in_todays_money`]
/// for a whole number of years).
pub(super) fn deposited_in_todays_money(
    first: Decimal,
    monthly: Decimal,
    months: u32,
    inflation: Percent,
) -> Decimal {
    let level_after_a_month = price_level(inflation, 1);
    let mut prices = Decimal::ONE;
    let mut total = first;
    for _ in 0..months {
        total += monthly / prices;
        prices *= level_after_a_month;
    }
    total
}

/// The safe kind a place is.
pub(super) fn kind_of(pays: Pays) -> Kind {
    match pays {
        Pays::Fixed(_) => Kind::FixedDeposit,
        Pays::TheRateLess(_) => Kind::MoneyMarketFund,
        Pays::GovernmentBond(linkage) => Kind::GovernmentBond {
            linked: linkage == Linkage::CpiLinked,
        },
    }
}

/// A safe kind, at its best place, for the money.
pub(super) struct Priced {
    pub kind: Kind,
    pub choice: Choice,
    pub left: Amount,
    pub outcome: short_term::Outcome,
    /// Every place of the kind that can hold the money, best first.
    pub places: Vec<Ranked>,
    pub may_cost_more: bool,
}

fn may_cost_more(place: &Place) -> bool {
    place
        .caveats
        .iter()
        .any(|caveat| caveat.may_cost_more_summary().is_some())
}

/// The best place of `kind` for `scenario`, or `None` when no place of it
/// can hold the money (a deposit takes one sum, up to five years; a bond
/// from one year to fifteen).
pub(super) fn price(
    kind: Kind,
    data: &Data,
    scenario: &short_term::Scenario,
    level: Decimal,
) -> Option<Priced> {
    let places: Vec<&Place> = data
        .safe
        .iter()
        .flat_map(|kind| &kind.places)
        .filter(|place| kind_of(place.pays) == kind)
        .collect();
    if places.is_empty() {
        return None;
    }
    let comparison = short_term::compare(&places, scenario);
    let mut places_ranked: Vec<Ranked> = comparison
        .places
        .iter()
        .filter_map(|place| {
            let outcome = place.outcome.as_ref().ok()?;
            let place = places[place.index];
            Some(Ranked {
                kind,
                name: place.name.clone(),
                left: amount(outcome.after_tax, level),
                cost_a_year: Some(outcome.yearly_cost),
                may_cost_more: may_cost_more(place),
                fits: true,
                yours: false,
            })
        })
        .collect();
    // A bond is bought through a broker, so the places that matter are the
    // brokers' accounts, not the two bonds.
    if let Kind::GovernmentBond { linked } = kind {
        let linkage = if linked {
            Linkage::CpiLinked
        } else {
            Linkage::Unlinked
        };
        if let Ok(brokers) = short_term::government_bonds::broker_ranking(linkage, scenario) {
            places_ranked = brokers
                .into_iter()
                .map(|(via, after_tax)| Ranked {
                    kind,
                    name: joined(&via.broker, &via.plan),
                    left: amount(after_tax, level),
                    cost_a_year: None,
                    may_cost_more: false,
                    fits: true,
                    yours: false,
                })
                .collect();
        }
    }
    let best = comparison
        .places
        .iter()
        .find(|place| place.outcome.is_ok())?;
    let outcome = best.outcome.clone().ok()?;
    let place = places[best.index];
    let choice = match &outcome.via {
        Some(via) => Choice {
            provider: via.broker.clone(),
            plan: via.plan.clone(),
        },
        None => Choice {
            provider: place.name.clone(),
            plan: kind.text(),
        },
    };
    Some(Priced {
        kind,
        choice,
        left: amount(outcome.after_tax, level),
        may_cost_more: may_cost_more(place),
        outcome,
        places: places_ranked,
    })
}

/// The name in both languages of a plan inside a provider: "Leumi · Online".
pub(super) fn joined(provider: &Text, plan: &Text) -> Text {
    Text::owned(
        format!("{} · {}", provider.en, plan.en),
        format!("{}\u{a0}· {}", provider.he, plan.he),
    )
}
