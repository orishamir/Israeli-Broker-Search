//! The guide's bindings: the recommendation, the ladder, the alternatives'
//! lines, the ways compared side by side, the situations, and the learn
//! layer's cards. They are thin: every figure and every sentence is the
//! core's, in the language `setLang` chose. This side turns the page's
//! fields (numbers that are `null` while a field is empty) into the core's
//! types, and the core's answers into plain data.

use broker_fees::describe::reasons::Span;
use broker_fees::describe::{self, About, LearnCard, Term};
use broker_fees::guide::{
    self, Alternative, Amount, Answers, Arena, Bank, Breakdown, Choice as GuideChoice,
    Circumstance, ColumnId, Cushion, Data, FollowUp, Horizon, InvalidAnswers, Kind, Overrides,
    Ranked, Reason, Safety, Settings, Situation,
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
                CushionInput::HasNot { monthly_expenses } => Cushion::HasNot {
                    monthly_expenses: amount_of(*monthly_expenses),
                },
            }),
            needs_any_day: self.needs_any_day,
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
                    monthly_expenses: Some(number(monthly_expenses)),
                },
            }),
            needs_any_day: answers.needs_any_day,
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

/// A follow-up question as a card.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FollowCardData {
    pub follow_up: FollowUp,
    pub title: String,
    pub text: String,
    /// The buttons, in the order of the answers they stand for.
    pub answers: Vec<String>,
    /// The label of the number field the card holds.
    pub field: Option<String>,
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

/// A kind the saver could have chosen and wasn't offered.
// `hashmap_as_object`: a flattened struct is a map to serde, and without this
// the page would get a JavaScript `Map`, whose fields `.kind` and `.name` read
// as nothing.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(hashmap_as_object)]
pub struct AlternativeData {
    #[serde(flatten)]
    pub kind: KindData,
    pub left: Option<AmountData>,
    pub why_not: ReasonData,
    pub band: Option<BandData>,
    /// Where its money would go, for the donut's other tab.
    pub breakdown: Option<BreakdownData>,
}

impl From<&Alternative> for AlternativeData {
    fn from(alternative: &Alternative) -> Self {
        AlternativeData {
            kind: KindData::tagged(alternative.kind, alternative.safety),
            left: alternative.left.map(AmountData::from),
            why_not: (&alternative.why_not).into(),
            band: alternative.band.as_ref().map(BandData::from),
            breakdown: alternative.breakdown.as_ref().map(BreakdownData::from),
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
// The outermost type decides how maps are written: see `AlternativeData`.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(hashmap_as_object)]
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
    pub alternatives: Vec<AlternativeData>,
    pub band: Option<BandData>,
    pub unlock: Option<UnlockData>,
    pub steps: Vec<StepData>,
    pub follow_ups: Vec<FollowUp>,
    pub two_parts: Option<CushionPlanData>,
    /// What was chosen, in the terms of the bank's app (or, for money with no
    /// cushion, the plan in two parts), with the thing named in bold.
    pub what: Vec<SpanData>,
    /// The second part of the cushion plan's sentence.
    pub what_then: Vec<SpanData>,
    pub left_note: LeftNoteData,
    pub current_account_line: String,
    pub unlock_text: Option<String>,
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
        alternatives: r.alternatives.iter().map(AlternativeData::from).collect(),
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
        follow_ups: r.follow_ups.clone(),
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

// ─────────────────────────── The ladder ───────────────────────────

/// What the same money does if it's needed after this long.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RungData {
    pub months: u32,
    /// The rung for money that can be needed on any day.
    pub any_day: bool,
    pub arena: Arena,
    pub kind: KindData,
    pub choice: ChoiceData,
    pub left: AmountData,
}

pub fn ladder_for(
    answers: &AnswersInput,
    settings: &GuideSettings,
) -> Result<Vec<RungData>, Problem> {
    let data = data_for(settings);
    let core = settings_for(settings, &data)?;
    Ok(guide::ladder(&answers.to_core(), &core, &data)?
        .into_iter()
        .map(|rung| RungData {
            months: rung.months,
            any_day: rung.any_day,
            arena: rung.arena,
            kind: KindData::tagged(rung.kind, rung.safety),
            choice: (&rung.choice).into(),
            left: rung.left.into(),
        })
        .collect())
}

/// The same money at 1, 3, 5, 10 and 20 years (and "any day" for a sum with
/// nothing coming in every month). Several comparisons' work: ask from the
/// worker.
#[wasm_bindgen]
pub fn ladder(
    answers: Ts<AnswersInput>,
    settings: Ts<GuideSettings>,
) -> Result<Vec<Ts<RungData>>, JsError> {
    ladder_for(&answers.to_rust()?, &settings.to_rust()?)
        .map_err(|problem| shown(&problem.text()))?
        .into_iter()
        .map(|rung| Ok(rung.into_ts()?))
        .collect()
}

// ─────────────────────────── The alternatives' lines ───────────────────────────

/// One line of the alternatives chart.
// Flattened too: see `AlternativeData`.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(hashmap_as_object)]
pub struct KindSeriesData {
    #[serde(flatten)]
    pub kind: KindData,
    /// The kind the answer recommends.
    pub recommended: bool,
    /// Today's shekels at the start (index 0) and after each month; the last
    /// point is what is left after tax.
    pub value_by_month: Vec<f64>,
    pub left: AmountData,
}

/// What the stretches of the history did to the same deposits, in today's
/// shekels, before fees and tax.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BandSeriesData {
    pub band: BandData,
    pub worst: Vec<f64>,
    pub best: Vec<f64>,
    /// "In 52 of 1,086 stretches of 5 years since 1926 it ended lower", in
    /// dollars: the band's caption's first half.
    pub sentence: String,
    /// What the sentence leaves out (the stretches overlap, the data's end,
    /// a shekel fund also follows the exchange rate): its second half.
    pub caveat: String,
}

// The outermost type decides how maps are written: see `AlternativeData`.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(hashmap_as_object)]
pub struct AlternativesData {
    pub months: u32,
    pub series: Vec<KindSeriesData>,
    pub band: Option<BandSeriesData>,
}

fn numbers(values: &[Decimal]) -> Vec<f64> {
    values.iter().copied().map(number).collect()
}

pub fn alternatives_for(
    answers: &AnswersInput,
    settings: &GuideSettings,
) -> Result<AlternativesData, Problem> {
    let data = data_for(settings);
    let core = settings_for(settings, &data)?;
    let found = guide::alternatives(&answers.to_core(), &core, &data)?;
    Ok(AlternativesData {
        months: found.months,
        series: found
            .series
            .iter()
            .map(|line| KindSeriesData {
                kind: KindData::tagged(line.kind, line.safety),
                recommended: line.recommended,
                value_by_month: numbers(&line.value_by_month),
                left: line.left.into(),
            })
            .collect(),
        band: found.band.as_ref().map(|band| BandSeriesData {
            band: (&band.band).into(),
            worst: numbers(&band.worst),
            best: numbers(&band.best),
            sentence: broker_fees::describe::reasons::history_sentence(&band.band, lang()),
            caveat: broker_fees::describe::reasons::history_caveat(&band.band, lang()),
        }),
    })
}

/// The chart beside the answer: each kind's money month by month. Several
/// comparisons' work: ask from the worker.
#[wasm_bindgen]
pub fn alternatives(
    answers: Ts<AnswersInput>,
    settings: Ts<GuideSettings>,
) -> Result<Ts<AlternativesData>, JsError> {
    Ok(alternatives_for(&answers.to_rust()?, &settings.to_rust()?)
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

/// The questions to show now, given what is answered so far.
#[wasm_bindgen(js_name = followUps)]
pub fn follow_ups(answers: Ts<AnswersInput>) -> Result<Vec<Ts<FollowUp>>, JsError> {
    guide::follow_ups(&answers.to_rust()?.to_core())
        .into_iter()
        .map(|follow_up| Ok(follow_up.into_ts()?))
        .collect()
}

/// One follow-up question, as a card in the app's language.
#[wasm_bindgen(js_name = followCard)]
pub fn follow_card(follow_up: Ts<FollowUp>) -> Result<Ts<FollowCardData>, JsError> {
    let follow_up = follow_up.to_rust()?;
    let card = describe::answer::follow_card(follow_up, lang());
    Ok(FollowCardData {
        follow_up,
        title: card.title,
        text: card.text,
        answers: card.answers,
        field: card.field,
    }
    .into_ts()?)
}

/// What "another bank" is priced as.
#[wasm_bindgen(js_name = bankStandInNote)]
pub fn bank_stand_in_note() -> String {
    describe::answer::bank_stand_in_note(lang()).to_owned()
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

// ─────────────────────────── History ───────────────────────────

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct HistoryData {
    pub band: BandData,
    pub source_name: String,
    pub source_url: String,
}

/// What the S&P 500 did over every stretch of `years` years, 1 to 30;
/// missing outside them.
#[wasm_bindgen(js_name = historyBand)]
pub fn history_band(years: f64) -> Result<Option<Ts<HistoryData>>, JsError> {
    let Some(years) = decimal(years) else {
        return Ok(None);
    };
    let source = history::source();
    history::band(years)
        .map(|band| {
            Ok(HistoryData {
                band: (&band).into(),
                source_name: source.name[lang()].to_owned(),
                source_url: source.url.clone(),
            }
            .into_ts()?)
        })
        .transpose()
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

    #[test]
    fn the_alternatives_lines_end_where_the_answer_says() {
        let answers = AnswersInput::from_core(&situations::situations()[0].answers);
        let lines = alternatives_for(&answers, &settings()).unwrap();
        let recommendation = recommend_for(&answers, &settings()).unwrap();
        assert!(lines.series[0].recommended);
        assert_eq!(lines.series[0].kind.kind, recommendation.kind.kind);
        for line in &lines.series {
            let last = *line.value_by_month.last().unwrap();
            assert!(
                (last - line.left.today).abs() < 0.01,
                "{:?}",
                line.kind.kind
            );
        }
        assert!(lines.band.is_some());
    }

    #[test]
    fn the_ladder_has_a_rung_for_each_horizon() {
        let answers = AnswersInput::from_core(&situations::situations()[4].answers);
        let rungs = ladder_for(&answers, &settings()).unwrap();
        let core = guide::ladder(&answers.to_core(), &core_settings(), &Data::listed()).unwrap();
        assert_eq!(rungs.len(), core.len());
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
    fn the_types_the_page_reads_exist() {
        for declaration in [
            RecommendationData::DECL,
            AnswersInput::DECL,
            GuideSettings::DECL,
            AlternativesData::DECL,
            WaysComparisonData::DECL,
            RungData::DECL,
            SituationData::DECL,
        ] {
            assert!(declaration.contains("export"), "{declaration}");
        }
    }
}
