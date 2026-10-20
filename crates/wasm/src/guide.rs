//! The guide's bindings: the answer, the recommendation the workbench
//! ranks from, the ways compared side by side, the situations, and the learn
//! layer's cards. They are thin: every figure and every sentence is the
//! core's, in the language `setLang` chose. This side turns the page's
//! fields (numbers that are `null` while a field is empty) into the core's
//! types, and the core's answers into plain data.

use broker_fees::describe::reasons::Span;
use broker_fees::describe::{self, About, LearnCard, Term};
use broker_fees::guide::{
    self, Amount, Answers, Arena, Bank, Breakdown, Choice as GuideChoice, Circumstance, ColumnId,
    Cushion, Data, Horizon, InvalidAnswers, Kind, Overrides, Ranked, Reason, Safety, Settings,
    Situation,
};
use broker_fees::history;
use broker_fees::simulation::{self, Outcome};
use broker_fees::ways::{self, Candidate, Way};
use broker_fees::{
    Broker, BrokerKind, Caveat, ExchangeRates, IntoEnumIterator, Named, Percent, Product, Text,
    situations,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use super::{
    Field, Inputs, InvalidInputs, OutcomeData, PlanKey, PlanOutcomeData, Shared, YourPlanInput,
    all_brokers, chosen_plans, decimal, filled_in, lang, number, plan_data, scenario_with, shown,
};

// ─────────────────────────── What the page sends ───────────────────────────

/// The guide's questions as the page holds them: a number is `null` while its
/// field is empty, and counts as nothing.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AnswersInput {
    pub situation: Option<Situation>,
    /// ₪ put in at the start.
    #[tsify(type = "number | null")]
    pub now: Option<f64>,
    /// ₪ put in every month.
    #[tsify(type = "number | null")]
    pub monthly: Option<f64>,
    pub horizon: Horizon,
    pub flexibility: Option<guide::Flexibility>,
    pub cushion: Option<CushionInput>,
    pub needs_any_day: bool,
    /// The visitor's bank, when they named it.
    #[serde(default)]
    pub bank: Option<Bank>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CushionInput {
    Has,
    #[serde(rename_all = "camelCase")]
    HasNot {
        /// ₪ a month.
        #[tsify(type = "number | null")]
        monthly_expenses: Option<f64>,
    },
}

/// The workbench's settings the guide takes: null is the app's usual.
#[derive(Debug, Clone, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GuideSettings {
    /// 10 means 10% a year.
    #[tsify(type = "number | null")]
    pub index_return_percent: Option<f64>,
    #[tsify(type = "number | null")]
    pub inflation_percent: Option<f64>,
    /// The Bank of Israel's rate the saver expects, 4.5 means 4.5%.
    #[tsify(type = "number | null")]
    pub boi_rate_percent: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_usd: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_eur: Option<f64>,
    /// The plans ticked in the workbench; `null` is what a new customer
    /// usually gets at each broker.
    pub plans: Option<Vec<PlanKey>>,
    #[serde(default)]
    pub your_plans: Vec<YourPlanInput>,
    /// What the expert chose over the guide.
    #[serde(default)]
    pub overrides: Overrides,
    /// The calendar year now, for "in 2041's shekels": the core keeps no
    /// clock, so the page says. `null` leaves that note out.
    #[serde(default)]
    #[tsify(type = "number | null")]
    pub this_year: Option<i32>,
    /// The month now, 1 to 12, with `this_year`: for "in October 2029".
    #[serde(default)]
    #[tsify(type = "number | null")]
    pub this_month: Option<u8>,
}

/// Why the guide can't answer: its words are in the app's language.
#[derive(Debug)]
pub enum Problem {
    Answers(InvalidAnswers),
    Inputs(InvalidInputs),
}

impl Problem {
    fn text(&self) -> String {
        match self {
            Problem::Answers(error) => error.text(lang()).to_owned(),
            Problem::Inputs(error) => error.text(lang()),
        }
    }
}

impl From<InvalidAnswers> for Problem {
    fn from(error: InvalidAnswers) -> Self {
        Problem::Answers(error)
    }
}

impl From<InvalidInputs> for Problem {
    fn from(error: InvalidInputs) -> Self {
        Problem::Inputs(error)
    }
}

fn amount_of(value: Option<f64>) -> Decimal {
    value.and_then(decimal).unwrap_or(Decimal::ZERO)
}

impl AnswersInput {
    fn to_core(&self) -> Answers {
        Answers {
            situation: self.situation,
            now: amount_of(self.now),
            monthly: amount_of(self.monthly),
            horizon: self.horizon,
            flexibility: self.flexibility,
            cushion: self.cushion.as_ref().map(|cushion| match cushion {
                CushionInput::Has => Cushion::Has,
                // Until the visitor types theirs, the core sizes the cushion
                // by its stand-in, and the answer says it is one.
                CushionInput::HasNot { monthly_expenses } => Cushion::HasNot {
                    monthly_expenses: monthly_expenses.and_then(decimal),
                },
            }),
            needs_any_day: self.needs_any_day,
            bank: self.bank,
        }
    }

    fn from_core(answers: &Answers) -> Self {
        AnswersInput {
            situation: answers.situation,
            now: Some(number(answers.now)),
            monthly: Some(number(answers.monthly)),
            horizon: answers.horizon,
            flexibility: answers.flexibility,
            cushion: answers.cushion.map(|cushion| match cushion {
                Cushion::Has => CushionInput::Has,
                Cushion::HasNot { monthly_expenses } => CushionInput::HasNot {
                    monthly_expenses: monthly_expenses.map(number),
                },
            }),
            needs_any_day: answers.needs_any_day,
            bank: answers.bank,
        }
    }
}

/// The user's own plans, as a "broker" at the end of the listed ones, so the
/// guide's plan references reach them.
fn yours(plans: &[YourPlanInput]) -> Broker {
    let name = Text::new("Your plans", "המסלולים שלכם");
    Broker {
        name: name.clone(),
        short_name: name,
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 0,
        compared_at_first: false,
        description: Text::new("Plans you made.", "מסלולים שיצרתם."),
        tariff_date: None,
        latest_appendix: None,
        source_url: None,
        caveats: Vec::new(),
        plans: plans.iter().map(|yours| yours.plan.0.clone()).collect(),
        spread: None,
        exchange_notes: Vec::new(),
    }
}

/// What the guide works on, with the saver's own plans after the listed ones.
fn data_for(settings: &GuideSettings) -> Data {
    let mut data = Data::listed();
    if !settings.your_plans.is_empty() {
        data.brokers.push(yours(&settings.your_plans));
    }
    data
}

fn settings_for(settings: &GuideSettings, data: &Data) -> Result<Settings, InvalidInputs> {
    let rates = ExchangeRates::new(
        filled_in(settings.ils_per_usd, Field::UsdRate)?,
        filled_in(settings.ils_per_eur, Field::EurRate)?,
    )
    .map_err(|_| InvalidInputs::Other(super::Problem::RatesNotPositive))?;
    let mut core = Settings::new(rates);
    let percent =
        |value: Option<f64>, usual: Percent| value.and_then(decimal).map_or(usual, Percent);
    core.index_return = percent(settings.index_return_percent, core.index_return);
    core.inflation = percent(settings.inflation_percent, core.inflation);
    core.rate = percent(settings.boi_rate_percent, core.rate);
    core.overrides = settings.overrides;
    core.today = settings
        .this_year
        .zip(settings.this_month)
        .and_then(|(year, month)| {
            broker_fees::time::Date::from_calendar_date(
                year,
                broker_fees::time::Month::try_from(month).ok()?,
                1,
            )
            .ok()
        });
    // The plans ticked: a listed plan is where the data has it, your own
    // come after the listed brokers.
    let listed = data.brokers.len() - usize::from(!settings.your_plans.is_empty());
    core.ticked = settings.plans.as_ref().map(|keys| {
        keys.iter()
            .filter_map(|key| match key {
                PlanKey::Listed { broker, plan } => Some(guide::PlanRef {
                    broker: *broker,
                    plan: *plan,
                }),
                PlanKey::Yours { id } => settings
                    .your_plans
                    .iter()
                    .position(|yours| yours.id == *id)
                    .map(|plan| guide::PlanRef {
                        broker: listed,
                        plan,
                    }),
            })
            .collect()
    });
    Ok(core)
}

// ─────────────────────────── What the page gets ───────────────────────────

/// An amount in today's shekels, with the nominal figure beside it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AmountData {
    pub today: f64,
    pub nominal: f64,
}

impl From<Amount> for AmountData {
    fn from(amount: Amount) -> Self {
        AmountData {
            today: number(amount.today),
            nominal: number(amount.nominal),
        }
    }
}

/// A piece of a sentence, and whether the page should emphasise it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SpanData {
    pub text: String,
    pub bold: bool,
}

impl From<&Span> for SpanData {
    fn from(span: &Span) -> Self {
        SpanData {
            text: span.text.clone(),
            bold: span.bold,
        }
    }
}

fn spans(parts: &[Span]) -> Vec<SpanData> {
    parts.iter().map(SpanData::from).collect()
}

/// The note under the answer's amount.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LeftNoteData {
    pub today: String,
    /// The same sum in the money of the year it arrives, when that differs.
    pub nominal: Option<String>,
}

/// One of the two arenas, named for the "change" menu.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ArenaChoice {
    pub arena: Arena,
    pub name: String,
}

/// The place or plan chosen, by name.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceData {
    pub provider: String,
    pub plan: String,
}

impl From<&GuideChoice> for ChoiceData {
    fn from(choice: &GuideChoice) -> Self {
        ChoiceData {
            provider: choice.provider[lang()].to_owned(),
            plan: choice.plan[lang()].to_owned(),
        }
    }
}

/// A line of the answer: why, with the circumstance it comes from.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ReasonData {
    pub text: String,
    /// The name of the circumstance of the saver's that produced it
    /// (`NeedsAnyDay`, `ShortHorizon`...): the page can mark the answer that
    /// the reason comes from.
    pub from: String,
}

impl From<&Reason> for ReasonData {
    fn from(reason: &Reason) -> Self {
        let circumstance: &Circumstance = &reason.from;
        ReasonData {
            text: reason.text[lang()].to_owned(),
            from: <&'static str>::from(circumstance).to_owned(),
        }
    }
}

/// The learn card a kind has, if any.
fn card_of(kind: Kind) -> Option<Term> {
    use describe::Explained;
    match kind {
        Kind::MoneyMarketFund => Some(Term::MoneyMarketFund),
        Kind::FixedDeposit => Some(Term::FixedDeposit),
        Kind::LinkedDeposit => Some(Term::CpiLinkedDeposit),
        Kind::GovernmentBond { .. } => Some(Term::GovernmentBond),
        Kind::Way(way) => way.product.card(),
        Kind::CurrentAccount => None,
    }
}

/// A kind named in the terms people meet in their bank's app.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KindData {
    pub kind: Kind,
    pub name: String,
    pub card: Option<Term>,
    pub safety: Safety,
    /// The tag beside the kind ("sum known in advance"), and its sentence for
    /// the tip.
    pub safety_tag: String,
    pub safety_note: String,
    pub arena: Option<Arena>,
}

impl From<Kind> for KindData {
    fn from(kind: Kind) -> Self {
        KindData::tagged(kind, kind.safety())
    }
}

impl KindData {
    /// A kind with the safety the core worked out for its stretch of time.
    fn tagged(kind: Kind, safety: Safety) -> Self {
        KindData {
            kind,
            name: kind.name(lang()).to_owned(),
            card: card_of(kind),
            safety,
            safety_tag: broker_fees::describe::reasons::safety_tag(safety, lang()).to_owned(),
            safety_note: broker_fees::describe::reasons::safety_note(safety, lang()).to_owned(),
            arena: kind.arena(),
        }
    }
}

/// One line of a ranking: a kind, a place or a plan.
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is a fact the page shows on its own"
)]
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RankedData {
    pub kind: Kind,
    pub name: String,
    pub left: AmountData,
    /// What it loses a year, as a fund's fee is stated: 0.4 means 0.4%.
    pub cost_a_year: Option<f64>,
    pub may_cost_more: bool,
    /// Why, in words, where `may_cost_more` is.
    pub may_cost_more_text: Option<String>,
    pub fits: bool,
    /// Its place, 1 for the best; lines equal to the shekel share one.
    /// Missing for a line the first deposit can't open while another can.
    pub rank: Option<u32>,
    /// Within 1% of the best (or, for the best, of the second): not clearly
    /// apart.
    pub near_tie: bool,
    /// The saver's own bank's branch plan.
    pub yours: bool,
    /// For `yours`, the plan as the tariffs name it, which `name` isn't when
    /// "another bank" was chosen.
    pub own_plan: Option<String>,
    /// Fees paid over the years, where the kind has them.
    pub fees: Option<AmountData>,
    /// Tax paid on the gain, where the engine states it.
    pub tax: Option<AmountData>,
}

/// A ranking's lines, each with its place.
fn ranked_data(lines: &[Ranked]) -> Vec<RankedData> {
    lines
        .iter()
        .zip(guide::ranked_standings(lines))
        .map(|(line, standing)| RankedData::of(line, standing))
        .collect()
}

impl RankedData {
    fn of(ranked: &Ranked, standing: Option<simulation::Standing>) -> Self {
        RankedData {
            rank: standing.and_then(|standing| u32::try_from(standing.rank).ok()),
            near_tie: standing.is_some_and(|standing| standing.near_tie),
            kind: ranked.kind,
            name: ranked.name[lang()].to_owned(),
            left: ranked.left.into(),
            cost_a_year: ranked.cost_a_year.map(|cost| number(cost.0)),
            may_cost_more: ranked.may_cost_more,
            may_cost_more_text: ranked.warning.as_ref().map(|text| text[lang()].to_owned()),
            fits: ranked.fits,
            yours: ranked.yours,
            own_plan: ranked.own_plan.as_ref().map(|plan| plan[lang()].to_owned()),
            fees: ranked.fees.map(AmountData::from),
            tax: ranked.tax.map(AmountData::from),
        }
    }
}

/// What the index did over stretches as long as the horizon, yearly rates in
/// percent (before any fund's fee and tax).
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BandData {
    pub years: f64,
    pub worst: f64,
    pub p10: f64,
    pub median: f64,
    pub p90: f64,
    pub best: f64,
    /// 0 to 1.
    pub share_lower: f64,
    pub lower: u32,
    pub windows: u32,
    pub from_year: i32,
    pub from_month: u8,
    pub to_year: i32,
    pub to_month: u8,
}

impl From<&history::Band> for BandData {
    fn from(band: &history::Band) -> Self {
        BandData {
            years: number(band.years),
            worst: number(band.worst.0),
            p10: number(band.p10.0),
            median: number(band.median.0),
            p90: number(band.p90.0),
            best: number(band.best.0),
            share_lower: number(band.share_lower),
            lower: band.lower,
            windows: band.windows,
            from_year: band.from.year(),
            from_month: u8::from(band.from.month()),
            to_year: band.to.year(),
            to_month: u8::from(band.to.month()),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct UnlockData {
    /// ₪, the first deposit that opens the plan.
    pub first_deposit: f64,
    pub plan: ChoiceData,
    pub kind: Kind,
    pub after_tax: AmountData,
    pub instead_of: AmountData,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct StepData {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct CushionPlanData {
    pub target: f64,
    pub months_to_fill: u32,
    pub then: KindData,
    pub then_ranked: Vec<RankedData>,
    pub then_years: u32,
}

/// Where every shekel went, in today's money.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownData {
    pub put_in: f64,
    /// Fees to the broker and the manager.
    pub fees: f64,
    /// What the product holding the index took on its own.
    pub product: f64,
    pub kept_by_the_place: f64,
    pub tax: f64,
    pub left: f64,
}

impl From<&Breakdown> for BreakdownData {
    fn from(split: &Breakdown) -> Self {
        BreakdownData {
            put_in: number(split.put_in),
            fees: number(split.fees),
            product: number(split.product),
            kept_by_the_place: number(split.kept_by_the_place),
            tax: number(split.tax),
            left: number(split.left),
        }
    }
}

/// The answer, every text in the app's language.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationData {
    pub arena: Arena,
    pub kind: KindData,
    pub choice: ChoiceData,
    /// The arena's kinds, best first.
    pub ranked: Vec<RankedData>,
    /// The places or plans of the kind, best first.
    pub plans: Vec<RankedData>,
    pub left: AmountData,
    pub put_in: AmountData,
    /// The same deposits left in a current account.
    pub current_account: AmountData,
    pub breakdown: BreakdownData,
    pub months: u32,
    pub reasons: Vec<ReasonData>,
    pub band: Option<BandData>,
    pub unlock: Option<UnlockData>,
    pub steps: Vec<StepData>,
    pub two_parts: Option<CushionPlanData>,
    /// What was chosen, in the terms of the bank's app (or, for money with no
    /// cushion, the plan in two parts), with the thing named in bold.
    pub what: Vec<SpanData>,
    /// The second part of the cushion plan's sentence.
    pub what_then: Vec<SpanData>,
    pub left_note: LeftNoteData,
    pub current_account_line: String,
    pub unlock_text: Option<String>,
    /// A plan that leaves more but may cost more, with its caveat in words.
    pub also_possible_text: Option<String>,
    /// What moving the portfolio from the bank saves, with the pages it rests on.
    pub move_portfolio_text: Option<String>,
    pub move_portfolio_sources: Vec<describe::Source>,
    pub arena_choices: Vec<ArenaChoice>,
    /// The columns of the list of `plans`.
    pub columns: Vec<ColumnData>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ColumnData {
    pub id: ColumnId,
    pub name: String,
}

fn columns(arena: Arena) -> Vec<ColumnData> {
    guide::columns_for(arena)
        .into_iter()
        .map(|column| ColumnData {
            id: column.id,
            name: column.name[lang()].to_owned(),
        })
        .collect()
}

pub fn recommend_for(
    answers: &AnswersInput,
    settings: &GuideSettings,
) -> Result<RecommendationData, Problem> {
    let data = data_for(settings);
    let core = settings_for(settings, &data)?;
    let r = guide::recommend(&answers.to_core(), &core, &data)?;
    let (what, what_then) = match &r.two_parts {
        Some(plan) => {
            let (first, then) = describe::reasons::cushion_text(plan, r.answers.monthly, lang());
            let mut first = spans(&first);
            first.push(SpanData {
                text: " ".to_owned(),
                bold: false,
            });
            (first, spans(&then))
        }
        None => (
            spans(&describe::answer::what(
                r.kind,
                &r.choice,
                r.months,
                &r.answers.horizon,
                lang(),
            )),
            Vec::new(),
        ),
    };
    let note = describe::answer::left_note(r.left, r.months, settings.this_year, lang());
    Ok(RecommendationData {
        arena: r.arena,
        kind: KindData::tagged(r.kind, r.kind.safety_over(r.band)),
        choice: (&r.choice).into(),
        ranked: ranked_data(&r.ranked),
        plans: ranked_data(&r.plans),
        left: r.left.into(),
        put_in: r.put_in.into(),
        current_account: r.current_account.into(),
        breakdown: (&r.breakdown).into(),
        months: r.months,
        reasons: r.reasons.iter().map(ReasonData::from).collect(),
        band: r.band.as_ref().map(BandData::from),
        unlock: r.unlock.as_ref().map(|unlock| UnlockData {
            first_deposit: number(unlock.first_deposit),
            plan: (&unlock.plan).into(),
            kind: unlock.kind,
            after_tax: unlock.after_tax.into(),
            instead_of: unlock.instead_of.into(),
        }),
        steps: r
            .steps
            .iter()
            .map(|step| StepData {
                title: step.title[lang()].to_owned(),
                text: step.text[lang()].to_owned(),
            })
            .collect(),
        two_parts: r.two_parts.as_ref().map(|plan| CushionPlanData {
            target: number(plan.target),
            months_to_fill: plan.months_to_fill,
            then: plan.then.into(),
            then_ranked: ranked_data(&plan.then_ranked),
            then_years: plan.then_years,
        }),
        what,
        what_then,
        left_note: LeftNoteData {
            today: note.today,
            nominal: note.nominal,
        },
        current_account_line: describe::answer::current_account_line(r.current_account, lang()),
        unlock_text: r
            .unlock
            .as_ref()
            .map(|unlock| describe::reasons::unlock_text(unlock, lang())),
        also_possible_text: r
            .also_possible
            .as_ref()
            .map(|also| describe::answer::also_possible_line(also, lang())),
        move_portfolio_text: r
            .move_portfolio
            .as_ref()
            .map(|m| describe::answer::move_portfolio_line(m, r.months, lang())),
        move_portfolio_sources: if r.move_portfolio.is_some() {
            describe::answer::move_portfolio_sources()
                .iter()
                .map(|page| describe::Source::new(page, lang()))
                .collect()
        } else {
            Vec::new()
        },
        arena_choices: [Arena::Safe, Arena::Index]
            .into_iter()
            .map(|arena| ArenaChoice {
                arena,
                name: describe::answer::arena_name(arena, lang()).to_owned(),
            })
            .collect(),
        columns: columns(r.arena),
    })
}

/// What fits money like this, and why.
#[wasm_bindgen]
pub fn recommend(
    answers: Ts<AnswersInput>,
    settings: Ts<GuideSettings>,
) -> Result<Ts<RecommendationData>, JsError> {
    Ok(recommend_for(&answers.to_rust()?, &settings.to_rust()?)
        .map_err(|problem| shown(&problem.text()))?
        .into_ts()?)
}

// ─────────────────────────── The ways, side by side ───────────────────────────

/// The workbench's inputs and which ways to compare. `inputs.security`,
/// `exchange` and `product` are replaced by each way's own.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WaysInputs {
    pub inputs: Inputs,
    /// The ways to run; empty is all of them.
    pub ways: Vec<Way>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WayData {
    pub way: Way,
    pub name: String,
    pub card: Option<Term>,
    /// ₪ left after tax by the best plan the deposits open.
    pub after_tax: Option<f64>,
    /// What the best plan loses a year to the product and its own fees:
    /// 0.4 means 0.4%.
    pub cost_a_year: Option<f64>,
    pub may_cost_more: bool,
    /// Its place among the ways, 1 for the best; ways equal to the shekel
    /// share one. Missing for a way no plan the deposits open can hold.
    pub rank: Option<u32>,
    /// Within 1% of the best (or, for the best, of the second).
    pub near_tie: bool,
    /// The fund's own cost over the years, ₪: shown as "הקרן עצמה".
    pub product_cost: Option<f64>,
    /// Every plan that could hold the way, best first (the page shows the
    /// top ones and expands to all).
    pub plans: Vec<PlanOutcomeData>,
    /// The way's no-fees line, to measure fees against.
    pub no_fees: OutcomeData,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WaysComparisonData {
    pub deposited: f64,
    /// Best first.
    pub ways: Vec<WayData>,
}

pub fn compare_ways_for(request: &WaysInputs) -> Result<WaysComparisonData, InvalidInputs> {
    let inputs = &request.inputs;
    let brokers = all_brokers();
    let plans = chosen_plans(inputs, &brokers)?;
    let chosen: Vec<Way> = if request.ways.is_empty() {
        Way::for_index()
    } else {
        request.ways.clone()
    };
    // Each way buys on its own exchange: the share price is asked for if it
    // matters on any of them.
    let probe = scenario_with(inputs, false)?;
    let matters = chosen.iter().any(|way| {
        let scenario = way.scenario(&probe);
        simulation::share_price_matters(scenario.security, scenario.exchange, &plans)
    });
    let base = scenario_with(inputs, matters)?;
    let rates = super::exchange_rates(inputs)?;
    let inflation = super::inflation(inputs)?;
    let todays = inputs.in_todays_money && !inflation.is_zero();
    let caveats: Vec<&[Caveat]> = super::chosen_broker_caveats(inputs, &brokers);
    let candidates: Vec<Candidate<'_>> = plans
        .iter()
        .zip(&caveats)
        .map(|(&plan, &broker_caveats)| Candidate {
            plan,
            broker_caveats,
        })
        .collect();
    let deposited = if todays {
        base.deposited_in_todays_money(inflation)
    } else {
        base.deposited()
    };
    let compared = ways::compare_chosen_ways(&chosen, &base, &candidates, &rates);
    let lefts: Vec<Decimal> = compared
        .iter()
        .filter_map(|way| Some(way.best()?.outcome.as_ref().ok()?.after_tax))
        .collect();
    let mut standings = simulation::standings(&lefts).into_iter();
    let ways = compared
        .into_iter()
        .map(|compared| {
            let standing = compared.best().and_then(|_| standings.next());
            let scenario = compared.way.scenario(&base);
            let mut no_fees = simulation::compare(&[], &scenario, &rates).no_fees;
            if todays {
                no_fees = no_fees.in_todays_money(inflation);
            }
            let shared = Shared {
                inputs,
                brokers: &brokers,
                plans: &plans,
                rates: &rates,
                inflation,
                deposited,
            };
            let rows: Vec<PlanOutcomeData> = compared
                .plans
                .iter()
                .zip(simulation::plan_standings(&compared.plans))
                .map(|(row, row_standing)| {
                    let mut row = row.clone();
                    if todays {
                        row.outcome = row
                            .outcome
                            .map(|outcome| outcome.in_todays_money(inflation));
                    }
                    plan_data(&shared, &scenario, &no_fees, &row, row_standing)
                })
                .collect();
            let best: Option<&Outcome> =
                compared.best().and_then(|best| best.outcome.as_ref().ok());
            let (after_tax, product_cost) = best.map_or((None, None), |outcome| {
                let outcome = if todays {
                    outcome.in_todays_money(inflation)
                } else {
                    outcome.clone()
                };
                (
                    Some(number(outcome.after_tax)),
                    Some(number(outcome.fees.product)),
                )
            });
            WayData {
                way: compared.way,
                name: compared.way.name(lang()).to_owned(),
                card: describe::Explained::card(compared.way.product),
                after_tax,
                cost_a_year: compared.cost_a_year.map(|cost| number(cost.0)),
                may_cost_more: compared.may_cost_more,
                rank: standing.and_then(|standing| u32::try_from(standing.rank).ok()),
                near_tie: standing.is_some_and(|standing| standing.near_tie),
                product_cost,
                plans: rows,
                no_fees: OutcomeData::compared_to(&no_fees, &no_fees),
            }
        })
        .collect();
    Ok(WaysComparisonData {
        deposited: number(deposited),
        ways,
    })
}

/// The ways to hold the index, each with every plan that could hold it, best
/// first: the workbench's "which way" table. Each way's plans expand to the
/// full list.
#[wasm_bindgen(js_name = compareWays)]
pub fn compare_ways(request: Ts<WaysInputs>) -> Result<Ts<WaysComparisonData>, JsError> {
    Ok(compare_ways_for(&request.to_rust()?)
        .map_err(super::invalid)?
        .into_ts()?)
}

/// The ways the workbench offers, in order, named.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WayChoice {
    pub way: Way,
    pub name: String,
    pub card: Option<Term>,
    pub product: Product,
}

#[wasm_bindgen]
pub fn ways() -> Result<Vec<Ts<WayChoice>>, JsError> {
    Way::for_index()
        .into_iter()
        .map(|way| {
            Ok(WayChoice {
                way,
                name: way.name(lang()).to_owned(),
                card: describe::Explained::card(way.product),
                product: way.product,
            }
            .into_ts()?)
        })
        .collect()
}

// ─────────────────────────── The questions, the situations ───────────────────────────

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SituationData {
    /// Stable, for links.
    pub id: String,
    pub name: String,
    /// The situation in words, for the chip's tip.
    pub explanation: String,
    /// What it teaches, in a sentence.
    pub lesson: String,
    pub answers: AnswersInput,
    /// Opens the dialog about where to find a portfolio's fees.
    pub opens_dialog: bool,
    /// Shows the workbench with every way.
    pub workbench: bool,
}

/// The chips, in order.
#[wasm_bindgen]
pub fn situations() -> Result<Vec<Ts<SituationData>>, JsError> {
    situations::situations()
        .into_iter()
        .map(|preset| {
            Ok(SituationData {
                id: preset.id.to_owned(),
                name: preset.name[lang()].to_owned(),
                explanation: preset.explanation[lang()].to_owned(),
                lesson: preset.lesson[lang()].to_owned(),
                answers: AnswersInput::from_core(&preset.answers),
                opens_dialog: preset.opens_dialog,
                workbench: preset.workbench,
            }
            .into_ts()?)
        })
        .collect()
}

/// The banks whose branch price list is modelled, for the "I already have a
/// portfolio at a bank" chip.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BankChoice {
    pub bank: Bank,
    pub name: String,
}

#[wasm_bindgen]
pub fn banks() -> Result<Vec<Ts<BankChoice>>, JsError> {
    let listed = all_brokers();
    Bank::iter()
        .map(|bank| {
            let short = bank.short_name();
            let name = listed
                .iter()
                .find(|broker| broker.short_name.en == short)
                .map_or_else(|| short.to_owned(), |broker| broker.name[lang()].to_owned());
            Ok(BankChoice { bank, name }.into_ts()?)
        })
        .collect()
}

/// The columns of the list within an arena.
#[wasm_bindgen(js_name = columnsFor)]
pub fn columns_for(arena: Ts<Arena>) -> Result<Vec<Ts<ColumnData>>, JsError> {
    columns(arena.to_rust()?)
        .into_iter()
        .map(|column| Ok(column.into_ts()?))
        .collect()
}

// ─────────────────────────── The learn layer ───────────────────────────

/// One term's card.
#[wasm_bindgen(js_name = learnCard)]
pub fn learn_card(term: Ts<Term>) -> Result<Ts<LearnCard>, JsError> {
    Ok(describe::learn_card(term.to_rust()?, lang()).into_ts()?)
}

/// Every card, in the glossary's order.
#[wasm_bindgen]
pub fn glossary() -> Result<Vec<Ts<LearnCard>>, JsError> {
    describe::glossary(lang())
        .into_iter()
        .map(|card| Ok(card.into_ts()?))
        .collect()
}

/// The one page on the numbers for the whole site.
#[wasm_bindgen(js_name = aboutAll)]
pub fn about_all() -> Result<Ts<About>, JsError> {
    Ok(describe::about_all(lang()).into_ts()?)
}

/// The lines at the foot of every page.
#[wasm_bindgen]
pub fn footer() -> Vec<String> {
    describe::footer(lang())
}

// ─────────────────────────── The answer ───────────────────────────

/// What the page asks the answer for: the facts, the visitor's choices on
/// the answer, the settings, and whether to work out the horizon section
/// (most of a short answer's time: the page can show the rest first).
#[derive(Debug, Clone, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AnswerRequest {
    pub answers: AnswersInput,
    #[serde(default)]
    pub choices: guide::Choices,
    pub settings: GuideSettings,
    /// `null` or true works the horizons out too.
    #[serde(default)]
    pub horizons: Option<bool>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceData {
    pub state: guide::ConfidenceState,
    pub note: String,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OptData {
    pub pick: guide::Pick,
    pub title: String,
    pub amount: String,
    pub value: Option<AmountData>,
    pub facts: Vec<String>,
    pub fits_if: Option<String>,
    pub button: String,
    pub rows: Vec<Kind>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DecisionData {
    pub id: guide::DecisionId,
    pub question: String,
    pub why: String,
    pub options: Vec<OptData>,
    pub difference: Option<String>,
    pub chosen: Option<guide::Pick>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LineStateData {
    Stated {
        text: String,
        pill: guide::Pill,
    },
    Decision {
        decision: DecisionData,
    },
    /// "יתברר אחרי ההחלטה למעלה".
    Waiting,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LineData {
    pub kind: guide::LineKind,
    pub label: String,
    pub state: LineStateData,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MarkData {
    Known,
    RateRange {
        low: AmountData,
        high: AmountData,
    },
    Market {
        worst: AmountData,
        median: AmountData,
        assumed: AmountData,
        best: AmountData,
        losses: String,
    },
    Expected,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RowData {
    pub id: Kind,
    pub name: String,
    pub tags: Vec<String>,
    pub mark: MarkData,
    pub value: AmountData,
    /// `value` in words, rounded to how sure it is: what the row shows.
    pub amount: String,
    /// Its figure lies beyond the axis: drawn as an arrow at the edge.
    pub tail: bool,
    pub active: bool,
    pub excluded: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AnswerOutcomeData {
    pub date: String,
    pub title: String,
    pub deposited: AmountData,
    pub current_account: AmountData,
    pub axis: (f64, f64),
    pub rows: Vec<RowData>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SearchData {
    pub text: String,
    /// A placeholder the data can't fill (the text says why): shown as such.
    pub placeholder: bool,
    /// The pages its names and numbers come from.
    pub sources: Vec<describe::Source>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ProviderData {
    pub name: String,
    pub left: AmountData,
    pub versus_yours: Option<AmountData>,
    pub yours: bool,
    pub tie: u8,
    /// "May cost more: …", marked ⚠ on the row.
    pub warning: Option<String>,
    /// `left` in words, rounded as the chart's rows are.
    pub amount: String,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WhereToBuyData {
    pub title: String,
    pub rows: Vec<ProviderData>,
    pub bank_range: Option<(AmountData, AmountData)>,
    pub note: Option<String>,
    pub asks_bank: bool,
    pub unlock: Option<String>,
    pub may_cost_more: Option<String>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct HowData {
    pub row: Kind,
    pub name: String,
    pub identity: Vec<String>,
    pub search: Option<SearchData>,
    pub steps: Vec<String>,
    pub not_offered: Option<String>,
    pub where_to_buy: Option<WhereToBuyData>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GainRowData {
    pub name: String,
    pub kept: AmountData,
    pub tax: AmountData,
    pub fees: AmountData,
    pub product: AmountData,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GainData {
    pub lead: String,
    pub rows: Vec<GainRowData>,
    pub takeaway: String,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct HorizonRowData {
    pub years: u32,
    pub flexibility: Option<guide::Flexibility>,
    pub condition: Option<String>,
    pub current: bool,
    pub deposited: AmountData,
    pub deposits: String,
    pub kind: Kind,
    pub name: String,
    pub low: AmountData,
    pub high: AmountData,
    pub range: String,
    pub range_note: String,
    pub per_shekel: (f64, f64),
    pub per_shekel_text: String,
}

/// The answer, in the page's language.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AnswerData {
    pub confidence: ConfidenceData,
    pub title: String,
    pub body: String,
    pub lines: Vec<LineData>,
    pub plan: Option<String>,
    pub open_decisions: u32,
    pub outcome: AnswerOutcomeData,
    pub focus: Option<Kind>,
    pub how: Vec<HowData>,
    pub gain: Option<GainData>,
    pub horizons: Vec<HorizonRowData>,
    pub annuity: Option<String>,
}

fn said(text: &Text) -> String {
    text[lang()].to_owned()
}

fn said_all(texts: &[Text]) -> Vec<String> {
    texts.iter().map(said).collect()
}

fn decision_data(decision: &guide::Decision) -> DecisionData {
    DecisionData {
        id: decision.id,
        question: said(&decision.question),
        why: said(&decision.why),
        options: decision
            .options
            .iter()
            .map(|option| OptData {
                pick: option.pick,
                title: said(&option.title),
                amount: said(&option.amount),
                value: option.value.map(AmountData::from),
                facts: said_all(&option.facts),
                fits_if: option.fits_if.as_ref().map(said),
                button: said(&option.button),
                rows: option.rows.clone(),
            })
            .collect(),
        difference: decision.difference.as_ref().map(said),
        chosen: decision.chosen,
    }
}

fn mark_data(mark: &guide::Mark) -> MarkData {
    match mark {
        guide::Mark::Known => MarkData::Known,
        guide::Mark::Expected => MarkData::Expected,
        guide::Mark::RateRange { low, high } => MarkData::RateRange {
            low: (*low).into(),
            high: (*high).into(),
        },
        guide::Mark::Market {
            worst,
            median,
            assumed,
            best,
            losses,
        } => MarkData::Market {
            worst: (*worst).into(),
            median: (*median).into(),
            assumed: (*assumed).into(),
            best: (*best).into(),
            losses: said(losses),
        },
    }
}

fn where_data(where_to_buy: &guide::WhereToBuy) -> WhereToBuyData {
    WhereToBuyData {
        title: said(&where_to_buy.title),
        rows: where_to_buy
            .rows
            .iter()
            .map(|row| ProviderData {
                name: said(&row.name),
                left: row.left.into(),
                versus_yours: row.versus_yours.map(AmountData::from),
                yours: row.yours,
                tie: row.tie,
                warning: row.warning.as_ref().map(said),
                amount: said(&row.amount),
            })
            .collect(),
        bank_range: where_to_buy
            .bank_range
            .map(|(low, high)| (low.into(), high.into())),
        note: where_to_buy.note.as_ref().map(said),
        asks_bank: where_to_buy.asks_bank,
        unlock: where_to_buy.unlock.as_ref().map(said),
        may_cost_more: where_to_buy.may_cost_more.as_ref().map(said),
    }
}

fn answer_data(answer: &guide::Answer) -> AnswerData {
    AnswerData {
        confidence: ConfidenceData {
            state: answer.confidence.state,
            note: said(&answer.confidence.note),
        },
        title: said(&answer.title),
        body: said(&answer.body),
        lines: answer
            .lines
            .iter()
            .map(|line| LineData {
                kind: line.kind,
                label: said(&line.label),
                state: match &line.state {
                    guide::LineState::Stated { text, pill } => LineStateData::Stated {
                        text: said(text),
                        pill: *pill,
                    },
                    guide::LineState::Decision(decision) => LineStateData::Decision {
                        decision: decision_data(decision),
                    },
                    guide::LineState::Waiting => LineStateData::Waiting,
                },
            })
            .collect(),
        plan: answer.plan.as_ref().map(said),
        open_decisions: answer.open_decisions,
        outcome: AnswerOutcomeData {
            date: said(&answer.outcome.date),
            title: said(&answer.outcome.title),
            deposited: answer.outcome.deposited.into(),
            current_account: answer.outcome.current_account.into(),
            axis: (number(answer.outcome.axis.0), number(answer.outcome.axis.1)),
            rows: answer
                .outcome
                .rows
                .iter()
                .map(|row| RowData {
                    id: row.id,
                    name: said(&row.name),
                    tags: said_all(&row.tags),
                    mark: mark_data(&row.mark),
                    value: row.value.into(),
                    amount: said(&row.amount),
                    tail: row.tail,
                    active: row.active,
                    excluded: row.excluded.as_ref().map(said),
                    note: row.note.as_ref().map(said),
                })
                .collect(),
        },
        focus: answer.focus,
        how: answer
            .how
            .iter()
            .map(|how| HowData {
                row: how.row,
                name: said(&how.name),
                identity: said_all(&how.identity),
                search: how.search.as_ref().map(|search| SearchData {
                    text: said(&search.text),
                    placeholder: search.placeholder,
                    sources: search
                        .sources
                        .iter()
                        .map(|page| describe::Source::new(page, lang()))
                        .collect(),
                }),
                steps: said_all(&how.steps),
                not_offered: how.not_offered.as_ref().map(said),
                where_to_buy: how.where_to_buy.as_ref().map(where_data),
            })
            .collect(),
        gain: answer.gain.as_ref().map(|gain| GainData {
            lead: said(&gain.lead),
            rows: gain
                .rows
                .iter()
                .map(|row| GainRowData {
                    name: said(&row.name),
                    kept: row.kept.into(),
                    tax: row.tax.into(),
                    fees: row.fees.into(),
                    product: row.product.into(),
                })
                .collect(),
            takeaway: said(&gain.takeaway),
        }),
        horizons: answer.horizons.iter().map(horizon_data).collect(),
        annuity: answer.annuity.as_ref().map(said),
    }
}

fn horizon_data(row: &guide::HorizonRow) -> HorizonRowData {
    HorizonRowData {
        years: row.years,
        flexibility: row.flexibility,
        condition: row.condition.as_ref().map(said),
        current: row.current,
        deposited: row.deposited.into(),
        deposits: said(&row.deposits),
        kind: row.kind,
        name: said(&row.name),
        low: row.low.into(),
        high: row.high.into(),
        range: said(&row.range),
        range_note: said(&row.range_note),
        per_shekel: (number(row.per_shekel.0), number(row.per_shekel.1)),
        per_shekel_text: said(&row.per_shekel_text),
    }
}

pub fn answer_for(request: &AnswerRequest) -> Result<AnswerData, Problem> {
    let data = data_for(&request.settings);
    let core = settings_for(&request.settings, &data)?;
    let answer = guide::answer_with(
        &request.answers.to_core(),
        &request.choices,
        &core,
        &data,
        request.horizons.unwrap_or(true),
    )?;
    Ok(answer_data(&answer))
}

/// The redesigned answer (REDESIGN.md Part 2): the chip, the lines and their
/// decisions, the outcome chart, how to do it, the gain, the horizons.
#[wasm_bindgen]
pub fn answer(request: Ts<AnswerRequest>) -> Result<Ts<AnswerData>, JsError> {
    Ok(answer_for(&request.to_rust()?)
        .map_err(|problem| shown(&problem.text()))?
        .into_ts()?)
}

/// The outcome chart's fixed words (`describe::outcome::words`).
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeWordsData {
    pub subtitle: String,
    pub deposited: String,
    pub current_account: String,
    pub known: String,
    pub market: String,
    pub worst: String,
    pub median: String,
    pub best: String,
    pub per_year: String,
    pub if_it_rises: String,
}

#[wasm_bindgen(js_name = outcomeWords)]
pub fn outcome_words() -> Result<Ts<OutcomeWordsData>, JsError> {
    let words = broker_fees::describe::outcome::words();
    Ok(OutcomeWordsData {
        subtitle: said(&words.subtitle),
        deposited: said(&words.deposited),
        current_account: said(&words.current_account),
        known: said(&words.known),
        market: said(&words.market),
        worst: said(&words.worst),
        median: said(&words.median),
        best: said(&words.best),
        per_year: said(&words.per_year),
        if_it_rises: said(&words.if_it_rises),
    }
    .into_ts()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use broker_fees::{Lang, guide};

    fn settings() -> GuideSettings {
        GuideSettings {
            index_return_percent: None,
            inflation_percent: None,
            boi_rate_percent: None,
            ils_per_usd: Some(3.7),
            ils_per_eur: Some(4.3),
            plans: None,
            your_plans: vec![],
            overrides: Overrides::default(),
            this_year: Some(2026),
            this_month: Some(10),
        }
    }

    fn core_settings() -> Settings {
        Settings::new(ExchangeRates::new(Decimal::new(37, 1), Decimal::new(43, 1)).unwrap())
    }

    /// Every situation the page offers is answered with the arena and kind
    /// the core gives.
    #[test]
    fn recommend_is_the_cores_answer() {
        let data = Data::listed();
        for preset in situations::situations() {
            let answers = AnswersInput::from_core(&preset.answers);
            let bound = recommend_for(&answers, &settings()).unwrap();
            let core = guide::recommend(&preset.answers, &core_settings(), &data).unwrap();
            assert_eq!(bound.arena, core.arena, "{}", preset.id);
            assert_eq!(bound.kind.kind, core.kind, "{}", preset.id);
            assert_eq!(bound.reasons.len(), core.reasons.len());
            assert_eq!(bound.steps.len(), core.steps.len());
            assert!((number(core.left.today) - bound.left.today).abs() < 0.01);
            assert!(!bound.columns.is_empty());
        }
    }

    #[test]
    fn empty_money_is_the_guides_error() {
        let mut answers = AnswersInput::from_core(&situations::situations()[0].answers);
        answers.now = None;
        answers.monthly = None;
        let error = recommend_for(&answers, &settings()).unwrap_err();
        assert!(matches!(error, Problem::Answers(InvalidAnswers::NoMoney)));
    }

    #[test]
    fn the_ticked_plans_are_the_cores_references() {
        let data = data_for(&settings());
        assert_eq!(data.brokers.len(), all_brokers().len());
        let ticked = settings_for(
            &GuideSettings {
                plans: Some(vec![PlanKey::Listed { broker: 0, plan: 0 }]),
                ..settings()
            },
            &data,
        )
        .unwrap()
        .ticked;
        assert_eq!(ticked, Some(vec![guide::PlanRef { broker: 0, plan: 0 }]));
    }

    fn ways_inputs() -> Inputs {
        Inputs {
            security: broker_fees::Security::Etf,
            product: None,
            exchange: broker_fees::Exchange::Usa,
            first_deposit: Some(100_000.0),
            monthly_deposit: Some(0.0),
            yearly_return_percent: Some(10.0),
            years: 15,
            buy_every_months: 1,
            share_price: Some(500.0),
            deposit_growth_percent: Some(0.0),
            inflation_percent: Some(2.0),
            in_todays_money: true,
            as_pension: false,
            age: None,
            sell_at_end: true,
            ils_per_usd: Some(3.7),
            ils_per_eur: Some(4.3),
            plans: vec![
                PlanKey::Listed { broker: 0, plan: 0 },
                PlanKey::Listed { broker: 1, plan: 0 },
            ],
            your_plans: vec![],
        }
    }

    #[test]
    fn the_ways_are_compared_with_the_funds_own_cost() {
        let comparison = compare_ways_for(&WaysInputs {
            inputs: ways_inputs(),
            ways: vec![],
        })
        .unwrap();
        assert_eq!(comparison.ways.len(), Way::for_index().len());
        let best = comparison
            .ways
            .iter()
            .find_map(|way| way.after_tax)
            .unwrap();
        // Best first.
        assert_eq!(comparison.ways[0].after_tax, Some(best));
        let fund = comparison
            .ways
            .iter()
            .find(|way| way.way == Way::for_index()[0])
            .unwrap();
        assert!(fund.product_cost.unwrap() > 0.0);
        assert!(!fund.plans.is_empty());
        assert!(fund.card.is_some());
    }

    #[test]
    fn the_learn_layer_is_complete() {
        use broker_fees::IntoEnumIterator;
        assert_eq!(describe::glossary(Lang::He).len(), Term::iter().count());
        assert_eq!(describe::footer(Lang::He).len(), 2);
    }

    #[test]
    fn the_answer_is_the_cores_in_the_pages_language() {
        let request = AnswerRequest {
            answers: AnswersInput {
                situation: None,
                now: Some(100_000.0),
                monthly: Some(2000.0),
                horizon: Horizon::Months(36),
                flexibility: None,
                cushion: None,
                needs_any_day: false,
                bank: Some(Bank::Hapoalim),
            },
            choices: guide::Choices::default(),
            settings: settings(),
            horizons: Some(false),
        };
        let answer = answer_for(&request).unwrap();
        assert_eq!(answer.lines.len(), 3);
        assert!(answer.horizons.is_empty());
        assert!(!answer.outcome.rows.is_empty());
        assert!(
            answer.outcome.title.ends_with("2029"),
            "{}",
            answer.outcome.title
        );
    }

    #[test]
    fn the_types_the_page_reads_exist() {
        for declaration in [
            RecommendationData::DECL,
            AnswersInput::DECL,
            GuideSettings::DECL,
            WaysComparisonData::DECL,
            SituationData::DECL,
            AnswerRequest::DECL,
            AnswerData::DECL,
        ] {
            assert!(declaration.contains("export"), "{declaration}");
        }
    }
}
