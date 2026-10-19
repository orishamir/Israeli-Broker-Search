//! The rules of REDESIGN.md §3, one [`Route`] for each row of its table, and
//! the answer built from the route's ranking: the reasons (each naming the
//! circumstance it comes from), the alternatives drawn beside it, the
//! unlock line, the steps.
//!
//! A route first becomes a [`Decision`] (the kind chosen, the arena ranked,
//! the figures), which is all the ladder needs; `recommend` goes on to the
//! parts that cost more: the alternatives, the unlock line and the words.

use std::cmp::Reverse;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::describe::reasons;
use crate::history;
use crate::short_term;
use crate::simulation;
use crate::ways::{Way, WayOutcome};
use crate::{Lang, Percent, Text, Vehicle};

use super::index::{self, Run};
use super::safe::{self, Priced, amount, price_level};
use super::series::{self, Alternatives, BandSeries, KindSeries};
use super::{
    Alternative, Amount, Answers, Arena, Breakdown, CUSHION_MONTHS, Choice, Circumstance, Context,
    Cushion, CushionPlan, Data, FLEXIBILITY_ASKED_UP_TO_MONTHS, Flexibility, Horizon,
    INDEX_IF_FLEXIBLE_FROM_MONTHS, InvalidAnswers, Kind, LADDER_YEARS, REFERENCE_YEARS, Ranked,
    Reason, Recommendation, Rung, SAFE_ONLY_UP_TO_MONTHS, SHORT_GOAL_UP_TO_MONTHS, Settings,
    Situation, StepText, check, follow_ups,
};

/// Which row of the table the answers fall in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    /// Needs any day, or a year or less.
    Short,
    /// 13 to 36 months, a known date.
    KnownDate,
    /// 37 to 84 months, the date fixed.
    FixedDate,
    /// 37 to 59 months, the date can move.
    FlexibleShort,
    /// 60 to 84 months, the date can move.
    IndexFlexible,
    /// 85 months and more.
    IndexLong,
    Retirement,
    /// No date, money every month, no cushion: the cushion first.
    CushionFirst,
    /// No date, money every month: a place that can be opened any time.
    OpenAnyTime,
    /// No date, one sum.
    UnknownSum,
    /// The saver chose the arena (or the kind's) over what the rules say.
    Forced(Arena),
}

impl Route {
    fn arena(self) -> Arena {
        match self {
            Route::Short
            | Route::KnownDate
            | Route::FixedDate
            | Route::FlexibleShort
            | Route::CushionFirst
            | Route::UnknownSum => Arena::Safe,
            Route::IndexFlexible | Route::IndexLong | Route::Retirement | Route::OpenAnyTime => {
                Arena::Index
            }
            Route::Forced(arena) => arena,
        }
    }
}

/// The target of the cushion and the months of saving that fill it, when a
/// cushion is still to be filled.
fn cushion_to_fill(answers: &Answers) -> Option<(Decimal, u32)> {
    let Some(Cushion::HasNot { monthly_expenses }) = answers.cushion else {
        return None;
    };
    if answers.monthly <= Decimal::ZERO {
        return None;
    }
    let target = monthly_expenses * Decimal::from(CUSHION_MONTHS);
    let missing = (target - answers.now).max(Decimal::ZERO);
    let months = (missing / answers.monthly).ceil().to_u32()?;
    (months > 0).then_some((target, months))
}

fn natural_route(answers: &Answers) -> Route {
    if answers.needs_any_day {
        return Route::Short;
    }
    match answers.horizon {
        Horizon::Retirement { .. } => Route::Retirement,
        Horizon::Unknown => {
            if answers.monthly <= Decimal::ZERO {
                Route::UnknownSum
            } else if cushion_to_fill(answers).is_some() {
                Route::CushionFirst
            } else {
                Route::OpenAnyTime
            }
        }
        Horizon::Months(months) if months <= SAFE_ONLY_UP_TO_MONTHS => Route::Short,
        Horizon::Months(months) if months <= SHORT_GOAL_UP_TO_MONTHS => Route::KnownDate,
        Horizon::Months(months) if months <= FLEXIBILITY_ASKED_UP_TO_MONTHS => {
            match answers.flexibility {
                Some(Flexibility::CanWait) if months >= INDEX_IF_FLEXIBLE_FROM_MONTHS => {
                    Route::IndexFlexible
                }
                Some(Flexibility::CanWait) => Route::FlexibleShort,
                Some(Flexibility::Fixed) | None => Route::FixedDate,
            }
        }
        Horizon::Months(_) => Route::IndexLong,
    }
}

/// The arena the expert asked for, or the situation implies.
fn forced_arena(answers: &Answers, settings: &Settings) -> Option<Arena> {
    settings
        .overrides
        .kind
        .and_then(Kind::arena)
        .or(settings.overrides.arena)
        .or_else(|| {
            matches!(answers.situation, Some(Situation::KnowsWhatToBuy)).then_some(Arena::Index)
        })
}

fn route_for(answers: &Answers, settings: &Settings) -> Route {
    let natural = natural_route(answers);
    match forced_arena(answers, settings) {
        Some(arena) if arena != natural.arena() => Route::Forced(arena),
        _ => natural,
    }
}

/// How long the money is kept in a safe kind, in months.
fn safe_months(answers: &Answers, route: Route) -> u32 {
    match (route, answers.horizon) {
        (Route::CushionFirst, _) => cushion_to_fill(answers).map_or(1, |(_, months)| months),
        (_, Horizon::Months(months)) => months,
        (_, Horizon::Retirement { age, retire_at }) => retire_at.saturating_sub(age).max(1) * 12,
        (_, Horizon::Unknown) => 12,
    }
}

/// How long the money is kept in an index kind, in whole years.
fn index_years(answers: &Answers) -> u32 {
    match answers.horizon {
        Horizon::Months(months) => index::years_for(months),
        Horizon::Retirement { age, retire_at } => retire_at.saturating_sub(age).max(1),
        Horizon::Unknown => REFERENCE_YEARS,
    }
}

/// Everything the ranking decides, before the parts that cost more.
struct Decision<'a> {
    route: Route,
    arena: Arena,
    kind: Kind,
    choice: Choice,
    ranked: Vec<Ranked>,
    plans: Vec<Ranked>,
    left: Amount,
    put_in: Amount,
    current_account: Amount,
    breakdown: Breakdown,
    months: u32,
    level: Decimal,
    cost_a_year: Option<Percent>,
    /// The index ways that were run, and which of them was chosen.
    run: Option<(Run<'a>, usize)>,
}

/// The deposits, in today's shekels and as they are, and the same left in a
/// current account.
fn yardsticks(
    answers: &Answers,
    settings: &Settings,
    months: u32,
    level: Decimal,
) -> (Amount, Amount) {
    let deposited = answers.now + answers.monthly * Decimal::from(months);
    let put_in = Amount {
        today: safe::deposited_in_todays_money(
            answers.now,
            answers.monthly,
            months,
            settings.inflation,
        ),
        nominal: deposited,
    };
    (put_in, amount(deposited, level))
}

/// The tie rule of the arenas: equal to the shekel is a tie, and a tie goes
/// to the one without a price that may be too low.
fn sort_priced(priced: &mut [Priced]) {
    priced.sort_by_key(|p| {
        (
            Reverse(p.left.nominal.round_dp(0)),
            p.may_cost_more,
            Reverse(p.left.nominal),
        )
    });
}

fn safe_scenario(answers: &Answers, settings: &Settings, months: u32) -> short_term::Scenario {
    short_term::Scenario {
        first_deposit: answers.now,
        monthly_deposit: answers.monthly,
        months,
        rate: settings.rate,
        inflation: settings.inflation,
    }
}

/// The safe kinds that fit the route, in the order a tie is settled in.
fn safe_pool(route: Route, answers: &Answers) -> Vec<Kind> {
    let bonds = [
        Kind::GovernmentBond { linked: false },
        Kind::GovernmentBond { linked: true },
    ];
    match route {
        // The money may be needed any day: only what pays out the next day.
        Route::Short if answers.needs_any_day => vec![Kind::MoneyMarketFund],
        Route::Short => vec![Kind::MoneyMarketFund, Kind::FixedDeposit],
        Route::KnownDate => {
            let mut pool = vec![Kind::FixedDeposit, Kind::MoneyMarketFund];
            pool.extend(bonds);
            pool
        }
        // Only what has a sum known in advance; the money market fund is an
        // alternative ("the rate changes").
        // (A linked bond's sum in shekels is not known: it follows the index.)
        Route::FixedDate | Route::FlexibleShort => {
            vec![Kind::FixedDeposit, Kind::GovernmentBond { linked: false }]
        }
        Route::CushionFirst | Route::UnknownSum => vec![Kind::MoneyMarketFund],
        Route::Forced(_)
        | Route::IndexFlexible
        | Route::IndexLong
        | Route::Retirement
        | Route::OpenAnyTime => {
            let mut pool = vec![Kind::MoneyMarketFund, Kind::FixedDeposit];
            pool.extend(bonds);
            pool
        }
    }
}

fn decide_safe<'a>(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
    route: Route,
) -> Decision<'a> {
    let months = safe_months(answers, route);
    let level = price_level(settings.inflation, months);
    let scenario = safe_scenario(answers, settings, months);
    let price_pool = |pool: Vec<Kind>| -> Vec<Priced> {
        let mut priced: Vec<Priced> = pool
            .into_iter()
            .filter_map(|kind| safe::price(kind, data, &scenario, level))
            .collect();
        sort_priced(&mut priced);
        priced
    };
    let mut pool = safe_pool(route, answers);
    // The kind the expert chose is ranked even where the rules would leave it out.
    if let Some(kind) = settings.overrides.kind
        && kind.arena() == Some(Arena::Safe)
        && !pool.contains(&kind)
    {
        pool.push(kind);
    }
    let mut priced = price_pool(pool);
    if priced.is_empty() {
        // Nothing with a known sum fits (a bond has no place for this wait):
        // the fund is always there.
        priced = price_pool(vec![Kind::MoneyMarketFund]);
    }
    let at = settings
        .overrides
        .kind
        .and_then(|kind| priced.iter().position(|p| p.kind == kind))
        .unwrap_or(0);
    let chosen = priced.remove(at);
    let (put_in, current_account) = yardsticks(answers, settings, months, level);
    let mut ranked: Vec<Ranked> = Vec::with_capacity(priced.len() + 1);
    ranked.push(kind_line(&chosen));
    ranked.extend(priced.iter().map(kind_line));
    ranked.sort_by_key(|line| {
        (
            Reverse(line.left.nominal.round_dp(0)),
            line.may_cost_more,
            Reverse(line.left.nominal),
        )
    });
    let outcome = &chosen.outcome;
    Decision {
        route,
        arena: Arena::Safe,
        kind: chosen.kind,
        choice: chosen.choice.clone(),
        ranked,
        plans: chosen.places.clone(),
        left: chosen.left,
        put_in,
        current_account,
        breakdown: Breakdown {
            put_in: put_in.nominal / level,
            fees: Decimal::ZERO,
            product: Decimal::ZERO,
            kept_by_the_place: outcome.kept / level,
            tax: outcome.tax / level,
            left: chosen.left.today,
        },
        months,
        level,
        cost_a_year: Some(outcome.yearly_cost),
        run: None,
    }
}

/// Where an index way's money went: the product's own cost apart from the
/// broker's and the manager's.
fn index_breakdown(today: &simulation::Outcome, put_in: Decimal) -> Breakdown {
    Breakdown {
        put_in,
        fees: today.fees.total() - today.fees.product,
        product: today.fees.product,
        kept_by_the_place: Decimal::ZERO,
        tax: today.tax,
        left: today.after_tax,
    }
}

/// The same for a safe kind, from what the short term's engine says.
fn safe_breakdown(priced: &Priced, put_in: Decimal, level: Decimal) -> Breakdown {
    Breakdown {
        put_in,
        fees: Decimal::ZERO,
        product: Decimal::ZERO,
        kept_by_the_place: priced.outcome.kept / level,
        tax: priced.outcome.tax / level,
        left: priced.left.today,
    }
}

/// An index way's breakdown in today's shekels, when it has a plan.
fn way_breakdown(outcome: &WayOutcome, inflation: Percent, put_in: Decimal) -> Option<Breakdown> {
    let today = outcome
        .best()?
        .outcome
        .as_ref()
        .ok()?
        .in_todays_money(inflation);
    Some(index_breakdown(&today, put_in))
}

fn kind_line(priced: &Priced) -> Ranked {
    Ranked {
        kind: priced.kind,
        name: priced.kind.text(),
        left: priced.left,
        cost_a_year: Some(priced.outcome.yearly_cost),
        may_cost_more: priced.may_cost_more,
        warning: priced.warning.clone(),
        fees: None,
        tax: Some(priced.tax),
        fits: true,
        yours: false,
        own_plan: None,
    }
}

/// `None` when no way has a plan the deposits open.
fn decide_index<'a>(
    answers: &Answers,
    settings: &Settings,
    data: &'a Data,
    route: Route,
) -> Option<Decision<'a>> {
    let years = index_years(answers);
    let months = years * 12;
    let level = price_level(settings.inflation, months);
    let base = index::base_scenario(answers, settings, years);
    let cands = index::candidates(answers, settings, data, &base);
    let ways = if route == Route::OpenAnyTime {
        index::open_any_time()
    } else {
        Way::for_index()
    };
    let run = index::run(&ways, base, cands, &settings.rates);
    let ranked = run.ranked(level);
    let at = settings
        .overrides
        .kind
        .and_then(|kind| match kind {
            Kind::Way(way) => ranked.iter().position(|line| line.kind == Kind::Way(way)),
            _ => None,
        })
        // The first that anyone can open; a study fund that leaves more is
        // drawn beside the answer, with who can open one.
        .or_else(|| {
            ranked
                .iter()
                .position(|line| matches!(line.kind, Kind::Way(way) if index::open_to_anyone(way)))
        })
        .unwrap_or(0);
    let chosen = ranked.get(at)?.clone();
    // The ranking skips ways nothing opens, and so does `ranked`'s order
    // match `run.ways`' for the ones it keeps.
    let Kind::Way(way) = chosen.kind else {
        return None;
    };
    let way_at = run.ways.iter().position(|outcome| outcome.way == way)?;
    let way_outcome = &run.ways[way_at];
    let choice = run.choice(way_outcome)?;
    let plans = run.plans_ranked(way_outcome, level, &settings.rates);
    let best = way_outcome.best()?;
    let today = best
        .outcome
        .as_ref()
        .ok()?
        .in_todays_money(settings.inflation);
    let (put_in, current_account) = yardsticks(answers, settings, months, level);
    let breakdown = index_breakdown(&today, run.base.deposited() / level);
    Some(Decision {
        route,
        arena: Arena::Index,
        kind: chosen.kind,
        choice,
        ranked,
        plans,
        left: chosen.left,
        put_in,
        current_account,
        breakdown,
        months,
        level,
        cost_a_year: chosen.cost_a_year,
        run: Some((run, way_at)),
    })
}

fn decide<'a>(answers: &Answers, settings: &Settings, data: &'a Data) -> Decision<'a> {
    let route = route_for(answers, settings);
    if route.arena() == Arena::Index
        && let Some(decision) = decide_index(answers, settings, data, route)
    {
        return decision;
    }
    // A safe route; or an index one that no plan can open, which falls back to
    // the money that can be reached any day.
    let route = if route.arena() == Arena::Safe {
        route
    } else {
        Route::Short
    };
    decide_safe(answers, settings, data, route)
}

// ─────────────────────────── The words ───────────────────────────

fn context(answers: &Answers, settings: &Settings, d: &Decision<'_>) -> Context {
    Context {
        answers: answers.clone(),
        months: d.months,
        arena: d.arena,
        kind: d.kind,
        left: d.left,
        put_in: d.put_in,
        current_account: d.current_account,
        band: band_for(d.months),
        rate: settings.rate,
        index_return: settings.index_return,
        inflation: settings.inflation,
    }
}

fn band_for(months: u32) -> Option<history::Band> {
    history::band(Decimal::from(months) / Decimal::from(12))
}

fn reason(circumstance: Circumstance, kind: Kind, ctx: &Context) -> Reason {
    Reason {
        text: Text::owned(
            reasons::reason(&circumstance, kind, ctx, Lang::En),
            reasons::reason(&circumstance, kind, ctx, Lang::He),
        ),
        from: circumstance,
    }
}

fn why_not(kind: Kind, circumstance: Circumstance, left: Option<Amount>, ctx: &Context) -> Reason {
    Reason {
        text: Text::owned(
            reasons::why_not(kind, &circumstance, left, ctx, Lang::En),
            reasons::why_not(kind, &circumstance, left, ctx, Lang::He),
        ),
        from: circumstance,
    }
}

/// How the saver's answer to "how many shekels, how many months" reads as a
/// circumstance: one sum, or money every month.
fn money_pattern(answers: &Answers) -> Circumstance {
    if answers.monthly > Decimal::ZERO {
        Circumstance::MonthlyDeposits
    } else {
        Circumstance::OneSum
    }
}

/// What about the saver's horizon or date decides the route.
fn horizon_circumstance(route: Route, answers: &Answers, d: &Decision<'_>) -> Circumstance {
    match route {
        Route::Short if answers.needs_any_day => Circumstance::NeedsAnyDay,
        Route::Short => Circumstance::ShortHorizon { months: d.months },
        Route::KnownDate => Circumstance::KnownDate { months: d.months },
        Route::FixedDate => Circumstance::FixedDate { months: d.months },
        Route::FlexibleShort | Route::IndexFlexible => {
            Circumstance::FlexibleDate { months: d.months }
        }
        Route::IndexLong | Route::Forced(_) => Circumstance::LongHorizon { months: d.months },
        Route::Retirement => match answers.horizon {
            Horizon::Retirement { age, retire_at } => Circumstance::Retirement { age, retire_at },
            Horizon::Months(_) | Horizon::Unknown => Circumstance::LongHorizon { months: d.months },
        },
        Route::CushionFirst | Route::OpenAnyTime | Route::UnknownSum => Circumstance::NoDate,
    }
}

/// What the chosen safe kind is, as a reason.
fn safe_kind_circumstances(d: &Decision<'_>, ctx: &Context) -> Vec<Circumstance> {
    match d.kind {
        Kind::MoneyMarketFund => vec![Circumstance::EarnsBankRate { rate: ctx.rate }],
        Kind::FixedDeposit => vec![
            Circumstance::DepositLocked { months: d.months },
            Circumstance::SumKnownInAdvance,
        ],
        Kind::GovernmentBond { linked: true } => vec![
            Circumstance::RealValueKnownInAdvance,
            Circumstance::BondCanBeSold,
        ],
        _ => vec![Circumstance::SumKnownInAdvance, Circumstance::BondCanBeSold],
    }
}

fn reasons_for(
    answers: &Answers,
    settings: &Settings,
    d: &Decision<'_>,
    ctx: &Context,
) -> Vec<Reason> {
    let mut circumstances: Vec<Circumstance> = Vec::new();
    if settings.overrides.kind.is_some() || settings.overrides.arena.is_some() {
        circumstances.push(Circumstance::ChosenByYou);
    }
    match answers.situation {
        Some(Situation::KnowsWhatToBuy) => circumstances.push(Circumstance::KnowsWhatToBuy),
        Some(Situation::ExistingPortfolio { .. }) => {
            circumstances.push(Circumstance::ExistingPortfolio);
        }
        None => {}
    }
    let years = Decimal::from(d.months) / Decimal::from(12);
    let history = |circumstances: &mut Vec<Circumstance>| {
        if ctx.band.is_some() {
            circumstances.push(Circumstance::HistoryOfTheHorizon { years });
        }
    };
    let cheapest = Circumstance::CheapestAfterTax {
        cost_a_year: d.cost_a_year,
    };
    circumstances.push(horizon_circumstance(d.route, answers, d));
    match d.route {
        Route::Short | Route::Forced(Arena::Safe) => {
            circumstances.extend(safe_kind_circumstances(d, ctx));
        }
        Route::KnownDate | Route::FixedDate | Route::FlexibleShort => {
            circumstances.push(money_pattern(answers));
            circumstances.extend(safe_kind_circumstances(d, ctx));
        }
        Route::IndexFlexible => {
            circumstances.push(Circumstance::IfItsDownYouWait { months: d.months });
            history(&mut circumstances);
            circumstances.push(cheapest);
            circumstances.push(Circumstance::WhatSafetyCosts);
        }
        Route::IndexLong => {
            history(&mut circumstances);
            circumstances.push(cheapest);
            circumstances.push(Circumstance::WhatSafetyCosts);
        }
        Route::Retirement => {
            let in_the_fund =
                matches!(d.kind, Kind::Way(way) if way.vehicle == Vehicle::InvestmentGemel);
            if let Horizon::Retirement { retire_at, .. } = answers.horizon {
                if retire_at >= 60 && in_the_fund {
                    circumstances.push(Circumstance::TaxFreePension {
                        age_at_the_end: retire_at,
                    });
                } else if retire_at < 60 {
                    circumstances.push(Circumstance::PensionNeedsSixty {
                        age_at_the_end: retire_at,
                    });
                }
            }
            if in_the_fund && let Some(ceiling) = Vehicle::InvestmentGemel.rules().deposit_ceiling {
                circumstances.push(Circumstance::DepositCeiling { ceiling });
            }
            history(&mut circumstances);
            circumstances.push(cheapest);
        }
        Route::CushionFirst => {
            if let Some(Cushion::HasNot { monthly_expenses }) = answers.cushion {
                circumstances.push(Circumstance::NoCushion {
                    monthly_expenses,
                    target: monthly_expenses * Decimal::from(CUSHION_MONTHS),
                });
            }
            circumstances.push(Circumstance::MonthlyDeposits);
            circumstances.push(Circumstance::EarnsBankRate { rate: ctx.rate });
        }
        Route::OpenAnyTime => {
            circumstances.push(Circumstance::MonthlyDeposits);
            if answers.cushion == Some(Cushion::Has) {
                circumstances.push(Circumstance::HasCushion);
            }
            circumstances.push(cheapest);
        }
        Route::UnknownSum => {
            circumstances.push(Circumstance::OneSum);
            circumstances.push(Circumstance::EarnsBankRate { rate: ctx.rate });
        }
        Route::Forced(Arena::Index) => {
            history(&mut circumstances);
            circumstances.push(cheapest);
        }
    }
    circumstances
        .into_iter()
        .map(|circumstance| reason(circumstance, d.kind, ctx))
        .collect()
}

// ─────────────────────────── The alternatives ───────────────────────────

fn alternative(
    kind: Kind,
    circumstance: Circumstance,
    left: Option<Amount>,
    band: Option<history::Band>,
    breakdown: Option<Breakdown>,
    ctx: &Context,
) -> Alternative {
    Alternative {
        kind,
        name: kind.text(),
        arena: kind.arena(),
        left,
        safety: kind.safety(),
        why_not: why_not(kind, circumstance, left, ctx),
        band,
        breakdown,
    }
}

// One match over the two arenas, each drawing its own kinds: splitting it
// would only scatter the shared bookkeeping.
#[allow(clippy::too_many_lines)]
fn alternatives_for(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
    d: &Decision<'_>,
    ctx: &Context,
) -> Vec<Alternative> {
    let mut alternatives = Vec::new();
    let level = d.level;
    let months = d.months;
    match d.arena {
        Arena::Safe => {
            let scenario = safe_scenario(answers, settings, months);
            let main = horizon_circumstance(d.route, answers, d);
            let kinds = [
                Kind::MoneyMarketFund,
                Kind::FixedDeposit,
                Kind::GovernmentBond { linked: false },
                Kind::GovernmentBond { linked: true },
            ];
            for kind in kinds.into_iter().filter(|&kind| kind != d.kind) {
                let because = match kind {
                    Kind::MoneyMarketFund => Circumstance::RateNotPromised {
                        rate: settings.rate,
                    },
                    Kind::FixedDeposit if answers.monthly > Decimal::ZERO => {
                        Circumstance::MonthlyDeposits
                    }
                    Kind::FixedDeposit if answers.needs_any_day => {
                        Circumstance::DepositLocked { months }
                    }
                    _ => main.clone(),
                };
                match safe::price(kind, data, &scenario, level) {
                    Some(priced) => {
                        let breakdown =
                            safe_breakdown(&priced, d.put_in.nominal / d.level, d.level);
                        alternatives.push(alternative(
                            kind,
                            because,
                            Some(priced.left),
                            None,
                            Some(breakdown),
                            ctx,
                        ));
                    }
                    // A deposit that money coming every month can't use is
                    // still worth saying why not.
                    None if kind == Kind::FixedDeposit
                        && answers.monthly > Decimal::ZERO
                        && months <= short_term::DEPOSITS_LONGEST =>
                    {
                        alternatives.push(alternative(kind, because, None, None, None, ctx));
                    }
                    None => {}
                }
            }
            if let Some(stock) = stock_fund(answers, settings, data, months) {
                alternatives.push(alternative(
                    Kind::Way(stock.0),
                    main,
                    Some(stock.1),
                    ctx.band,
                    stock.2,
                    ctx,
                ));
            }
        }
        Arena::Index => {
            let band = ctx.band;
            let scenario = safe_scenario(answers, settings, months);
            let kinds = [
                Kind::MoneyMarketFund,
                Kind::FixedDeposit,
                Kind::GovernmentBond { linked: false },
                Kind::GovernmentBond { linked: true },
            ];
            for kind in kinds {
                if let Some(priced) = safe::price(kind, data, &scenario, level) {
                    let breakdown = safe_breakdown(&priced, d.put_in.nominal / level, level);
                    alternatives.push(alternative(
                        kind,
                        Circumstance::WhatSafetyCosts,
                        Some(priced.left),
                        band,
                        Some(breakdown),
                        ctx,
                    ));
                }
            }
            alternatives.extend(other_ways(d, answers, settings.inflation, band, ctx));
        }
    }
    alternatives.push(alternative(
        Kind::CurrentAccount,
        Circumstance::PricesRise {
            inflation: settings.inflation,
        },
        Some(d.current_account),
        None,
        None,
        ctx,
    ));
    alternatives
}

/// The index fund in Tel Aviv, the stock fund the safe arena's answer is
/// drawn against, over the same years.
fn stock_fund(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
    months: u32,
) -> Option<(Way, Amount, Option<Breakdown>)> {
    let years = index::years_for(months);
    let level = price_level(settings.inflation, years * 12);
    let base = index::base_scenario(answers, settings, years);
    let cands = index::candidates(answers, settings, data, &base);
    let way = Way::for_index()[0];
    let run = index::run(&[way], base, cands, &settings.rates);
    let left = run.ranked(level).into_iter().next()?.left;
    let breakdown = run.ways.first().and_then(|outcome| {
        way_breakdown(outcome, settings.inflation, run.base.deposited() / level)
    });
    Some((way, left, breakdown))
}

/// What is shown beside the fund of an index answer: the study fund, with
/// who can open one, and, for a retirement saver (`all`), the brokerage
/// account after tax and the policy too.
fn other_ways(
    d: &Decision<'_>,
    answers: &Answers,
    inflation: Percent,
    band: Option<history::Band>,
    ctx: &Context,
) -> Vec<Alternative> {
    let Some((run, chosen)) = &d.run else {
        return Vec::new();
    };
    let (chosen, level) = (*chosen, d.level);
    let all = d.route == Route::Retirement;
    let age_at_the_end = match answers.horizon {
        Horizon::Retirement { retire_at, .. } => retire_at,
        Horizon::Months(_) | Horizon::Unknown => 0,
    };
    let mut shown: Vec<Alternative> = Vec::new();
    let mut brokerage_seen = false;
    for (at, outcome) in run.ways.iter().enumerate() {
        if at == chosen || (!all && outcome.way.vehicle != Vehicle::StudyFund) {
            continue;
        }
        let Some(left) = outcome.after_tax().map(|nominal| amount(nominal, level)) else {
            continue;
        };
        let because = match outcome.way.vehicle {
            Vehicle::Brokerage if brokerage_seen => continue,
            Vehicle::Brokerage => {
                brokerage_seen = true;
                Circumstance::TaxFreePension { age_at_the_end }
            }
            Vehicle::StudyFund => Circumstance::StudyFundNeedsEligibility,
            Vehicle::InvestmentGemel | Vehicle::SavingsPolicy => {
                Circumstance::TaxFreePension { age_at_the_end }
            }
        };
        shown.push(alternative(
            Kind::Way(outcome.way),
            because,
            Some(left),
            band,
            way_breakdown(outcome, inflation, run.base.deposited() / level),
            ctx,
        ));
    }
    shown
}

// ─────────────────────────── The public calls ───────────────────────────

/// What fits money like this, and why.
///
/// Runs the engines for the arena the rules choose (one `compare_ways` for
/// an index answer) and, for the alternatives, the safe kinds and one more
/// way: far less than the ladder's several rungs, and what it costs is in
/// `benches/hot_paths.rs`.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn recommend(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
) -> Result<Recommendation, InvalidAnswers> {
    check(answers)?;
    let decision = decide(answers, settings, data);
    let ctx = context(answers, settings, &decision);
    let reasons = reasons_for(answers, settings, &decision, &ctx);
    let alternatives = alternatives_for(answers, settings, data, &decision, &ctx);
    let unlock = decision
        .run
        .as_ref()
        .and_then(|(run, at)| run.unlock(&run.ways[*at], decision.level, &settings.rates));
    let two_parts = cushion_plan(answers, settings, data, &decision);
    let mut recommendation = Recommendation {
        answers: answers.clone(),
        arena: decision.arena,
        kind: decision.kind,
        choice: decision.choice,
        ranked: decision.ranked,
        plans: decision.plans,
        left: decision.left,
        put_in: decision.put_in,
        current_account: decision.current_account,
        breakdown: decision.breakdown,
        months: decision.months,
        reasons,
        alternatives,
        band: ctx.band,
        unlock,
        steps: Vec::new(),
        follow_ups: follow_ups(answers),
        two_parts,
    };
    recommendation.steps = {
        let english = reasons::steps_for(&recommendation, Lang::En);
        let hebrew = reasons::steps_for(&recommendation, Lang::He);
        english
            .into_iter()
            .zip(hebrew)
            .map(|(en, he)| StepText {
                title: Text::owned(en.title, he.title),
                text: Text::owned(en.text, he.text),
            })
            .collect()
    };
    Ok(recommendation)
}

/// The second part of a cushion-first answer: where the monthly money goes
/// once the cushion is full.
fn cushion_plan(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
    d: &Decision<'_>,
) -> Option<CushionPlan> {
    if d.route != Route::CushionFirst {
        return None;
    }
    let (target, months_to_fill) = cushion_to_fill(answers)?;
    let years = REFERENCE_YEARS;
    let level = price_level(settings.inflation, years * 12);
    // What is put aside goes to the cushion; after it, only the monthly
    // money is left to invest.
    let mut then_answers = answers.clone();
    then_answers.now = Decimal::ZERO;
    then_answers.horizon = Horizon::Unknown;
    let base = index::base_scenario(&then_answers, settings, years);
    let cands = index::candidates(&then_answers, settings, data, &base);
    let run = index::run(&index::open_any_time(), base, cands, &settings.rates);
    let then_ranked = run.ranked(level);
    let then = then_ranked.first()?.kind;
    Some(CushionPlan {
        target,
        months_to_fill,
        then,
        then_ranked,
        then_years: years,
    })
}

/// The same money at the ladder's horizons: 1, 3, 5, 10 and 20 years (each
/// worked out as money without a date, whose date can move), and, for a sum
/// with nothing coming in every month, the sum kept for any day.
///
/// Each rung is a [`Decision`] without the alternatives, the unlock line and
/// the words: three of the five need `compare_ways`, so the page asks for the
/// ladder from its Web Worker.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn ladder(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
) -> Result<Vec<Rung>, InvalidAnswers> {
    check(answers)?;
    let rung_answers = |months: u32, any_day: bool| Answers {
        situation: None,
        horizon: Horizon::Months(months),
        flexibility: Some(Flexibility::CanWait),
        needs_any_day: any_day,
        ..answers.clone()
    };
    let mut rungs = Vec::new();
    if answers.monthly <= Decimal::ZERO {
        rungs.push(rung(
            &rung_answers(SAFE_ONLY_UP_TO_MONTHS, true),
            settings,
            data,
            true,
        ));
    }
    for years in LADDER_YEARS {
        rungs.push(rung(
            &rung_answers(years * 12, false),
            settings,
            data,
            false,
        ));
    }
    Ok(rungs)
}

fn rung(answers: &Answers, settings: &Settings, data: &Data, any_day: bool) -> Rung {
    let decision = decide(answers, settings, data);
    Rung {
        months: decision.months,
        any_day,
        arena: decision.arena,
        kind: decision.kind,
        choice: decision.choice,
        left: decision.left,
        safety: decision.kind.safety(),
    }
}

/// The lines of the alternatives chart: the answer's kind and each
/// alternative's, month by month in today's shekels, and the band of what the
/// index did over the horizon. It runs the kinds again for their whole
/// lines, which `recommend` keeps only the end of, so the page asks for it
/// from its Web Worker.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn alternatives(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
) -> Result<Alternatives, InvalidAnswers> {
    check(answers)?;
    let decision = decide(answers, settings, data);
    let ctx = context(answers, settings, &decision);
    let months = decision.months;
    let mut kinds = vec![(decision.kind, true)];
    kinds.extend(
        alternatives_for(answers, settings, data, &decision, &ctx)
            .into_iter()
            .map(|alternative| (alternative.kind, false)),
    );
    let scenario = safe_scenario(answers, settings, months);
    let level = decision.level;
    // The index kinds are worked out for whole years: the answer's own when
    // it is an index one, rounded otherwise.
    let years = if decision.arena == Arena::Index {
        months / 12
    } else {
        index::years_for(months)
    };
    let series = kinds
        .into_iter()
        .filter_map(|(kind, recommended)| {
            let (value_by_month, left) = match kind {
                Kind::CurrentAccount => (
                    series::deposits_by_month(
                        answers.now,
                        answers.monthly,
                        months,
                        settings.inflation,
                    ),
                    decision.current_account,
                ),
                Kind::Way(way) => way_line(way, answers, settings, data, years)?,
                _ => {
                    let priced = safe::price(kind, data, &scenario, level)?;
                    // Only the money market fund's engine counts the next
                    // deposit a month early.
                    let waiting = if kind == Kind::MoneyMarketFund {
                        answers.monthly
                    } else {
                        Decimal::ZERO
                    };
                    let line = series::deflated(
                        &priced.outcome.value_by_month,
                        settings.inflation,
                        priced.left.today,
                        waiting,
                    );
                    (line, priced.left)
                }
            };
            Some(KindSeries {
                kind,
                name: kind.text(),
                safety: kind.safety(),
                recommended,
                value_by_month,
                left,
            })
        })
        .collect();
    let band = ctx.band.map(|band| BandSeries {
        band,
        worst: series::grown_at(
            answers.now,
            answers.monthly,
            months,
            settings.inflation,
            band.worst,
        ),
        best: series::grown_at(
            answers.now,
            answers.monthly,
            months,
            settings.inflation,
            band.best,
        ),
    });
    Ok(Alternatives {
        months,
        series,
        band,
    })
}

/// A way's best plan's money month by month in today's shekels, and what it
/// leaves.
fn way_line(
    way: Way,
    answers: &Answers,
    settings: &Settings,
    data: &Data,
    years: u32,
) -> Option<(Vec<Decimal>, Amount)> {
    let base = index::base_scenario(answers, settings, years);
    let cands = index::candidates(answers, settings, data, &base);
    let run = index::run(&[way], base, cands, &settings.rates);
    let outcome = run.ways.first()?.best()?.outcome.as_ref().ok()?;
    let left = amount(
        outcome.after_tax,
        price_level(settings.inflation, years * 12),
    );
    Some((
        series::deflated(
            &outcome.value_by_month,
            settings.inflation,
            left.today,
            Decimal::ZERO,
        ),
        left,
    ))
}
