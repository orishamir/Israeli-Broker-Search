//! The core, as the web app calls it: everything about the brokers, and a
//! comparison of plans for the user's inputs. Types cross to JavaScript
//! through serde, and `tsify` writes their TypeScript definitions, so the UI
//! gets typed results without the types being written twice.
//!
//! Amounts cross as plain numbers (ready for charts); the exact decimal
//! maths stays in the core.

#![allow(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen's exported functions must take their arguments by value"
)]

use broker_fees::describe::{
    self, About, CaveatGroup, Explained, FeeKind, FeesFor, PriceText, Priced,
};
use broker_fees::simulation::{self, Fees, InvalidScenario, Outcome, Scenario};
use broker_fees::yours::{
    Amount, ConversionFields, CustodyFields, HandlingFields, InvalidFee, MarkupFields, PriceKind,
    PriceList, SimpleFees, TradeFields,
};
use broker_fees::{
    Broker, Buying, Exchange, ExchangeRates, IntoEnumIterator, Money, Percent, Period, Plan,
    Security, Source, TradeFee, ils, tariffs,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

fn all_brokers() -> Vec<Broker> {
    tariffs::all()
}

// ─────────────────────────── Choices ───────────────────────────

/// Something the user can pick, with its name to show and what it means.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Choice<T> {
    pub value: T,
    pub name: String,
    pub explanation: String,
    /// What Israeli brokers call it: "קרן סל".
    pub hebrew_names: Vec<String>,
}

impl<T: Copy + Explained> From<T> for Choice<T> {
    fn from(value: T) -> Self {
        Choice {
            value,
            name: value.to_string(),
            explanation: value.explanation().to_owned(),
            hebrew_names: value
                .hebrew_names()
                .iter()
                .map(|&name| name.to_owned())
                .collect(),
        }
    }
}

/// The kinds of security, in the order to offer them.
#[wasm_bindgen]
pub fn securities() -> Result<Vec<Ts<Choice<Security>>>, JsError> {
    Security::iter()
        .map(|security| Ok(Choice::from(security).into_ts()?))
        .collect()
}

/// The exchanges, in the order to offer them.
#[wasm_bindgen]
pub fn exchanges() -> Result<Vec<Ts<Choice<Exchange>>>, JsError> {
    Exchange::iter()
        .map(|exchange| Ok(Choice::from(exchange).into_ts()?))
        .collect()
}

/// A kind of fee, like a [`Choice`], with its name under the fee it's part of.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FeeKindChoice {
    pub value: FeeKind,
    pub name: String,
    /// "by standing order", under "Buy or sell".
    pub label: String,
    pub explanation: String,
    pub hebrew_names: Vec<String>,
}

/// The kinds of fee, named and explained, in the order the editor shows them.
#[wasm_bindgen(js_name = feeKinds)]
pub fn fee_kinds() -> Result<Vec<Ts<FeeKindChoice>>, JsError> {
    FeeKind::iter()
        .map(|kind| {
            let Choice {
                value,
                name,
                explanation,
                hebrew_names,
            } = Choice::from(kind);
            let label = kind.label().to_owned();
            Ok(FeeKindChoice {
                value,
                name,
                label,
                explanation,
                hebrew_names,
            }
            .into_ts()?)
        })
        .collect()
}

// ─────────────────────────── Brokers ───────────────────────────

/// A broker, as the sidebar and its details show it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BrokerInfo {
    pub name: String,
    /// "Leumi", beside a plan's name where plan names repeat: "Leumi · Online".
    pub short_name: String,
    /// Which of `plans` a new customer usually gets, compared at first.
    pub new_customer_plan: usize,
    pub description: String,
    /// "Tariff of 29/06/2026"
    pub tariff_date: String,
    /// "Checked 28/09/2026": when the numbers were last checked against the
    /// broker's documents and site.
    pub checked: String,
    pub source_url: Option<String>,
    /// Its tariff document, then every page its and its plans' caveats rest
    /// on, each once.
    pub sources: Vec<Source>,
    pub plans: Vec<PlanInfo>,
}

impl From<&Broker> for BrokerInfo {
    fn from(broker: &Broker) -> Self {
        BrokerInfo {
            name: broker.name.clone(),
            short_name: broker.short_name.clone(),
            new_customer_plan: broker.new_customer_plan,
            description: broker.description.clone(),
            tariff_date: broker.tariff_date_text().to_string(),
            checked: Broker::checked_text(),
            source_url: broker.source_url.clone(),
            sources: broker.sources(),
            plans: broker
                .plans
                .iter()
                .map(|plan| PlanInfo {
                    sources: broker.sources_for(plan),
                    ..PlanInfo::from(plan)
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub name: String,
    pub description: String,
    /// Every row of the plan's tariff, in words.
    pub tariff: TariffInfo,
    /// The pages its numbers rest on: the tariff, then what the broker-wide
    /// caveats and its own rest on, each once.
    pub sources: Vec<Source>,
}

impl From<&Plan> for PlanInfo {
    fn from(plan: &Plan) -> Self {
        let trade_rows = |rows: &[TradeFee]| {
            rows.iter()
                .map(|row| TariffRow {
                    covers: row.coverage().to_string(),
                    price: row.price.price_text(),
                })
                .collect()
        };
        PlanInfo {
            name: plan.name.clone(),
            description: plan.description.clone(),
            // Filled in by the broker, which knows its tariff and its caveats.
            sources: vec![],
            tariff: TariffInfo {
                trading: trade_rows(&plan.trading),
                tracks: plan
                    .tracks
                    .iter()
                    .map(|track| TrackInfo {
                        name: track.name.clone(),
                        trading: trade_rows(&track.trading),
                    })
                    .collect(),
                standing_orders: trade_rows(&plan.standing_orders),
                custody: plan
                    .custody
                    .iter()
                    .map(|row| TariffRow {
                        covers: row.coverage().to_string(),
                        price: row.price_text(),
                    })
                    .collect(),
                handling: plan.handling.map(|fee| fee.price_text()),
                conversion: plan.conversion.price_text(),
                second_conversion: plan.conversion.or_if_less.map(|fee| fee.price_text()),
                standing_order_conversion: plan
                    .standing_order_conversion
                    .as_ref()
                    .map(Priced::price_text),
                markup: plan.conversion.markup.price_text(),
                fractions_on: plan.fractions_on.iter().map(ToString::to_string).collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TariffInfo {
    pub trading: Vec<TariffRow>,
    /// The tracks a customer chooses one of, each with its own trade prices
    /// taking over the plan's for what they cover.
    pub tracks: Vec<TrackInfo>,
    /// Empty unless the plan has cheaper prices for buying by standing order.
    pub standing_orders: Vec<TariffRow>,
    pub custody: Vec<TariffRow>,
    /// "₪15 a month, free for the first 2 years", if the plan has one.
    pub handling: Option<PriceText>,
    pub conversion: PriceText,
    /// The second conversion fee, if the plan has one: each conversion pays
    /// whichever is less.
    pub second_conversion: Option<PriceText>,
    /// What converting costs when buying by standing order, if different.
    pub standing_order_conversion: Option<PriceText>,
    pub markup: PriceText,
    /// The exchanges where fractions of a share are sold: "USA".
    pub fractions_on: Vec<String>,
}

/// "1¢ a share", and the trade prices it sets.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub name: String,
    pub trading: Vec<TariffRow>,
}

/// "ETF on Tel Aviv" → "0.15%, min ₪3.5".
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TariffRow {
    pub covers: String,
    pub price: PriceText,
}

/// Every broker the app knows. Other functions refer to them, and to their
/// plans, by position in this list.
#[wasm_bindgen]
pub fn brokers() -> Result<Vec<Ts<BrokerInfo>>, JsError> {
    all_brokers()
        .iter()
        .map(|broker| Ok(BrokerInfo::from(broker).into_ts()?))
        .collect()
}

/// What the user buys, for the fees and caveats that matter to them.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub security: Security,
    pub exchange: Exchange,
    /// The biggest single order in the exchange's currency, from the
    /// comparison ([`ComparisonData::largest_trade`]), if it ran: some
    /// caveats matter only above an amount.
    #[tsify(type = "number | null")]
    pub largest_trade: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_usd: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_eur: Option<f64>,
}

impl Purchase {
    /// What it describes, and the rates to compare amounts with. Without
    /// usable rates the biggest order counts as unknown, so a caveat about
    /// large orders is shown rather than hidden.
    fn buying(&self) -> (Buying, ExchangeRates) {
        let rates = self
            .ils_per_usd
            .zip(self.ils_per_eur)
            .and_then(|(usd, eur)| Decimal::from_f64_retain(usd).zip(Decimal::from_f64_retain(eur)))
            .and_then(|(usd, eur)| ExchangeRates::new(usd, eur).ok());
        let largest_trade = rates.as_ref().and_then(|_| {
            let amount = Decimal::from_f64_retain(self.largest_trade?)?;
            Some(Money::from_decimal(amount, self.exchange.currency()))
        });
        let buying = Buying {
            security: self.security,
            exchange: self.exchange,
            largest_trade,
        };
        let rates = rates.unwrap_or_else(|| {
            ExchangeRates::new(Decimal::ONE, Decimal::ONE).expect("1 is a valid rate")
        });
        (buying, rates)
    }
}

/// What plan `plan` of broker `broker` charges for `purchase`, in words, on
/// `track` if the comparison picked one of its tracks.
#[wasm_bindgen(js_name = feesFor)]
pub fn fees_for(
    broker: usize,
    plan: usize,
    purchase: Ts<Purchase>,
    track: Option<usize>,
) -> Result<Ts<FeesFor>, JsError> {
    let brokers = all_brokers();
    let broker = brokers
        .get(broker)
        .ok_or_else(|| JsError::new("no such broker"))?;
    let plan = broker
        .plans
        .get(plan)
        .ok_or_else(|| JsError::new("no such plan"))?;
    let (buying, rates) = purchase.to_rust()?.buying();
    Ok(broker
        .describe_fees_for(plan, buying, track, &rates)
        .into_ts()?)
}

/// The caveats about every plan of broker `broker` that matter to
/// `purchase`, grouped by kind.
#[wasm_bindgen(js_name = brokerCaveats)]
pub fn broker_caveats(
    broker: usize,
    purchase: Ts<Purchase>,
) -> Result<Vec<Ts<CaveatGroup>>, JsError> {
    let brokers = all_brokers();
    let broker = brokers
        .get(broker)
        .ok_or_else(|| JsError::new("no such broker"))?;
    let (buying, rates) = purchase.to_rust()?.buying();
    broker
        .caveats_for(buying, &rates)
        .into_iter()
        .map(|group| Ok(group.into_ts()?))
        .collect()
}

/// The page that explains the numbers: how they're made, what isn't
/// counted, and the sources.
#[wasm_bindgen]
pub fn about() -> Result<Ts<About>, JsError> {
    Ok(describe::about().into_ts()?)
}

/// "an ETF bought in the USA"
#[wasm_bindgen(js_name = purchasePhrase)]
pub fn purchase_phrase(security: Ts<Security>, exchange: Ts<Exchange>) -> Result<String, JsError> {
    Ok(describe::purchase(security.to_rust()?, exchange.to_rust()?).to_string())
}

// ─────────────────────────── Comparing plans ───────────────────────────

/// What the user invests in, how, and which plans to compare.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    pub security: Security,
    pub exchange: Exchange,
    // The numbers are null while their field is empty.
    /// ₪
    #[tsify(type = "number | null")]
    pub first_deposit: Option<f64>,
    /// ₪
    #[tsify(type = "number | null")]
    pub monthly_deposit: Option<f64>,
    /// 10 means 10% a year.
    #[tsify(type = "number | null")]
    pub yearly_return_percent: Option<f64>,
    pub years: u32,
    pub buy_every_months: u32,
    /// Today's price of one share, in the exchange's currency.
    #[tsify(type = "number | null")]
    pub share_price: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_usd: Option<f64>,
    #[tsify(type = "number | null")]
    pub ils_per_eur: Option<f64>,
    /// The plans to compare.
    pub plans: Vec<PlanKey>,
    /// The user's own plans that `plans` refers to.
    #[serde(default)]
    pub your_plans: Vec<YourPlanInput>,
}

/// One of the user's own plans, with the id the web app gave it.
#[derive(Debug, Deserialize, Tsify)]
pub struct YourPlanInput {
    pub id: String,
    pub plan: PlanData,
}

/// Inputs the simulation can't run with. Each holds what's wrong, in words.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum InvalidInputs {
    /// An empty field: "the one-time deposit".
    #[error("fill in {0}")]
    Missing(&'static str),
    /// A negative amount: "the one-time deposit".
    #[error("{0} can't be negative")]
    Negative(&'static str),
    /// Any other problem: "invest for at least a year".
    #[error("{0}")]
    Other(&'static str),
    /// Numbers the simulation can't handle: "the yearly return can't be below −100%".
    #[error(transparent)]
    Scenario(#[from] InvalidScenario),
}

/// The plans `inputs` compares, in its order.
fn chosen_plans<'a>(
    inputs: &'a Inputs,
    brokers: &'a [Broker],
) -> Result<Vec<&'a Plan>, InvalidInputs> {
    inputs
        .plans
        .iter()
        .map(|key| match key {
            PlanKey::Listed { broker, plan } => brokers
                .get(*broker)
                .and_then(|broker| broker.plans.get(*plan)),
            PlanKey::Yours { id } => inputs
                .your_plans
                .iter()
                .find(|yours| yours.id == *id)
                .map(|yours| &yours.plan.0),
        })
        .collect::<Option<Vec<&Plan>>>()
        .ok_or(InvalidInputs::Other("there's no such plan"))
}

/// What `inputs` describes, checked. The share price is needed only if it
/// matters on one of `plans`: otherwise its field is hidden.
fn scenario(inputs: &Inputs, plans: &[&Plan]) -> Result<Scenario, InvalidInputs> {
    if inputs.years == 0 {
        return Err(InvalidInputs::Other("invest for at least a year"));
    }
    if inputs.buy_every_months == 0 {
        return Err(InvalidInputs::Other("buy at least every 12 months"));
    }
    let amount = |value, what| {
        let amount = filled_in(value, what)?;
        if amount.is_sign_negative() {
            return Err(InvalidInputs::Negative(what));
        }
        Ok(amount)
    };
    let share_price = if simulation::share_price_matters(inputs.security, inputs.exchange, plans) {
        let share_price = amount(inputs.share_price, "the share price")?;
        if share_price.is_zero() {
            return Err(InvalidInputs::Other("the share price must be more than 0"));
        }
        share_price
    } else {
        Decimal::ZERO
    };
    let scenario = Scenario {
        security: inputs.security,
        exchange: inputs.exchange,
        first_deposit: amount(inputs.first_deposit, "the one-time deposit")?,
        monthly_deposit: amount(inputs.monthly_deposit, "the monthly deposit")?,
        yearly_return: Percent(filled_in(
            inputs.yearly_return_percent,
            "the yearly return",
        )?),
        years: inputs.years,
        buy_every_months: inputs.buy_every_months,
        share_price,
    };
    scenario.check()?;
    Ok(scenario)
}

/// A plan: a listed one, by the positions of its broker in [`brokers`] and
/// of it in the broker's plans, or one of the user's own, by its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlanKey {
    Listed { broker: usize, plan: usize },
    Yours { id: String },
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonData {
    /// Every shekel put in over the years.
    pub deposited: f64,
    /// The same investing with no fees at all, to measure fees against.
    pub no_fees: OutcomeData,
    /// The biggest single order, in the exchange's currency: a purchase, or
    /// selling everything at the end, with no fees. Some caveats matter
    /// only above an amount.
    pub largest_trade: f64,
    /// Best first (most left after selling); plans that don't offer the
    /// security last.
    pub plans: Vec<PlanOutcomeData>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanOutcomeData {
    pub key: PlanKey,
    /// Missing if the plan doesn't offer the security on that exchange.
    pub outcome: Option<OutcomeData>,
    /// The plan's track the comparison used, the cheapest, if its tracks
    /// price the security there.
    pub track: Option<usize>,
    /// Why the plan can't be used as the inputs are, though its numbers are
    /// still shown: "Needs a one-time deposit of at least ₪5,000".
    pub warning: Option<String>,
    /// Why its numbers may be too low, if a caveat says so: "May cost more:
    /// conversion markup not published". The plan stays ranked.
    pub may_cost_more: Option<String>,
    /// Something to know to make sense of its numbers: "US track: 1¢ a
    /// share, the cheapest for you", "A standing order buys every month, so
    /// it isn't used here".
    pub note: Option<String>,
    /// Why it has no numbers, if it doesn't offer the security there, in the
    /// most general terms that are true: "Nothing in Europe is offered".
    pub not_offered: Option<String>,
}

/// All amounts in ₪.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeData {
    /// At the start (index 0) and at the end of each month.
    pub value_by_month: Vec<f64>,
    /// Worth at the end, without selling.
    pub held: f64,
    /// Received by selling everything at the end, after fees.
    pub after_selling: f64,
    /// How much less is left after selling than with no fees at all: the
    /// fees paid plus the growth they'd have earned.
    pub lost_to_fees: f64,
    /// How much less the investment is worth than with no fees, at the start
    /// (index 0) and at the end of each month.
    pub lost_by_month: Vec<f64>,
    /// Every fee paid, including selling at the end.
    pub fees: FeeAmounts,
    /// The fees paid by the end of each year (index 0 is the first year); the
    /// last includes selling at the end.
    pub fees_up_to_year: Vec<FeeAmounts>,
}

impl OutcomeData {
    fn compared_to(outcome: &Outcome, no_fees: &Outcome) -> Self {
        OutcomeData {
            value_by_month: outcome.value_by_month.iter().copied().map(number).collect(),
            held: number(outcome.held),
            after_selling: number(outcome.after_selling),
            lost_to_fees: number(outcome.lost_to_fees(no_fees)),
            lost_by_month: outcome.lost_by_month(no_fees).map(number).collect(),
            fees: (&outcome.fees).into(),
            fees_up_to_year: outcome
                .fees_up_to_each_year()
                .iter()
                .map(FeeAmounts::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FeeAmounts {
    pub purchases: f64,
    pub conversions: f64,
    pub custody: f64,
    pub handling: f64,
    pub selling: f64,
    pub total: f64,
}

impl From<&Fees> for FeeAmounts {
    fn from(fees: &Fees) -> Self {
        FeeAmounts {
            purchases: number(fees.purchases),
            conversions: number(fees.conversions),
            custody: number(fees.custody),
            handling: number(fees.handling),
            selling: number(fees.selling),
            total: number(fees.total()),
        }
    }
}

/// Simulates the chosen plans for `inputs` and ranks them.
#[wasm_bindgen]
pub fn compare(inputs: Ts<Inputs>) -> Result<Ts<ComparisonData>, JsError> {
    Ok(compare_plans(&inputs.to_rust()?)?.into_ts()?)
}

pub fn compare_plans(inputs: &Inputs) -> Result<ComparisonData, InvalidInputs> {
    let brokers = all_brokers();
    let plans = chosen_plans(inputs, &brokers)?;
    let scenario = scenario(inputs, &plans)?;
    let rates = ExchangeRates::new(
        filled_in(inputs.ils_per_usd, "the dollar's rate")?,
        filled_in(inputs.ils_per_eur, "the euro's rate")?,
    )
    .map_err(|_| InvalidInputs::Other("exchange rates must be more than 0"))?;
    let comparison = simulation::compare(&plans, &scenario, &rates);
    let no_fees = &comparison.no_fees;
    let deposited = scenario.deposited();
    let (security, exchange) = (scenario.security, scenario.exchange);
    let buying = Buying {
        security,
        exchange,
        largest_trade: Some(Money::from_decimal(
            no_fees.largest_trade,
            exchange.currency(),
        )),
    };
    Ok(ComparisonData {
        deposited: number(deposited),
        no_fees: OutcomeData::compared_to(no_fees, no_fees),
        largest_trade: number(no_fees.largest_trade),
        plans: comparison
            .plans
            .iter()
            .map(|compared| {
                let plan = plans[compared.index];
                let key = &inputs.plans[compared.index];
                let outcome = compared.outcome.as_ref();
                let fees_warning = || outcome.and_then(|outcome| outcome.warning(deposited));
                let track = outcome.and_then(|outcome| outcome.track);
                // Only a listed plan has caveats; the user's own are their
                // own claim.
                let may_cost_more = match key {
                    PlanKey::Listed { broker, .. } => {
                        brokers[*broker].may_cost_more(plan, buying, &rates)
                    }
                    PlanKey::Yours { .. } => None,
                };
                PlanOutcomeData {
                    key: key.clone(),
                    outcome: outcome.map(|outcome| OutcomeData::compared_to(outcome, no_fees)),
                    track,
                    warning: plan
                        .first_deposit_warning(ils(scenario.first_deposit))
                        .or_else(|| fees_warning().map(str::to_owned)),
                    may_cost_more,
                    note: plan.track_note(security, exchange, track).or_else(|| {
                        plan.standing_order_note(security, exchange, scenario.buy_every_months)
                            .map(str::to_owned)
                    }),
                    not_offered: outcome
                        .is_none()
                        .then(|| plan.not_offered_reason(security, exchange)),
                }
            })
            .collect(),
    })
}

/// The share price's currency ("$"), if the share price matters to what
/// `inputs` compares; `null` if its field isn't needed.
#[wasm_bindgen(js_name = sharePriceSymbol)]
pub fn share_price_symbol(inputs: Ts<Inputs>) -> Result<Option<String>, JsError> {
    let inputs = inputs.to_rust()?;
    let brokers = all_brokers();
    let plans = chosen_plans(&inputs, &brokers)?;
    let matters = simulation::share_price_matters(inputs.security, inputs.exchange, &plans);
    Ok(matters.then(|| inputs.exchange.currency().symbol.to_owned()))
}

// ─────────────────────────── Your plans ───────────────────────────

/// One of the user's own plans. The web app keeps it but never looks inside:
/// only the functions here read and change it.
///
/// It's saved as JSON, so what serde writes as a map (a struct with a
/// `#[serde(flatten)]` field, like the conversion fee) must become a plain
/// object: JSON saves a JavaScript `Map` as `{}`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
#[tsify(hashmap_as_object)]
pub struct PlanData(#[tsify(type = "{ readonly __brand: 'PlanData' }")] Plan);

/// What a row of a price list covers. Empty lists mean all.
#[derive(Debug, Deserialize, Tsify)]
pub struct Coverage {
    pub securities: Vec<Security>,
    pub exchanges: Vec<Exchange>,
}

/// A choice in one of the editor's dropdowns: "per order".
#[derive(Debug, Serialize, Tsify)]
pub struct Named<T> {
    pub value: T,
    pub name: String,
}

/// A period, as the editor names it: "quarter" (as in "a quarter"), and
/// "quarterly" (as in "charged quarterly").
#[derive(Debug, Serialize, Tsify)]
pub struct PeriodName {
    pub value: Period,
    pub name: String,
    pub adverb: String,
}

fn listed(broker: usize, plan: usize) -> Result<Plan, JsError> {
    all_brokers()
        .into_iter()
        .nth(broker)
        .and_then(|broker| broker.plans.into_iter().nth(plan))
        .ok_or_else(|| JsError::new("no such plan"))
}

/// `plan` after `change`, or why it can't be changed so.
fn changed(
    plan: Ts<PlanData>,
    change: impl FnOnce(&mut Plan) -> Result<(), InvalidFee>,
) -> Result<Ts<PlanData>, JsError> {
    let PlanData(mut plan) = plan.to_rust()?;
    change(&mut plan)?;
    Ok(PlanData(plan).into_ts()?)
}

fn original(original: Option<Ts<PlanData>>) -> Result<Option<Plan>, JsError> {
    Ok(original
        .map(|plan| plan.to_rust())
        .transpose()?
        .map(|data| data.0))
}

/// A listed plan, as your plans are kept, to compare a copy with: on
/// `track`, as the copy was made.
#[wasm_bindgen(js_name = listedPlan)]
pub fn listed_plan(
    broker: usize,
    plan: usize,
    track: Option<usize>,
) -> Result<Ts<PlanData>, JsError> {
    Ok(PlanData(listed(broker, plan)?.on_track(track.unwrap_or(0))).into_ts()?)
}

/// A copy of a listed plan to change, "Pepper, your deal", on `track` (the
/// first if none).
#[wasm_bindgen(js_name = copyOf)]
pub fn copy_of(broker: usize, plan: usize, track: Option<usize>) -> Result<Ts<PlanData>, JsError> {
    Ok(PlanData(listed(broker, plan)?.copy_of(track)).into_ts()?)
}

/// A plan of the user's own, free until they fill it in.
#[wasm_bindgen(js_name = newPlan)]
pub fn new_plan(name: String) -> Result<Ts<PlanData>, JsError> {
    Ok(PlanData(Plan::new_own(&name)).into_ts()?)
}

/// A plan of the user's own, as the table and dialogs show plans.
#[wasm_bindgen(js_name = planInfo)]
pub fn plan_info(plan: Ts<PlanData>) -> Result<Ts<PlanInfo>, JsError> {
    Ok(PlanInfo::from(&plan.to_rust()?.0).into_ts()?)
}

/// Like [`fees_for`], for one of the user's own plans.
#[wasm_bindgen(js_name = feesForPlan)]
pub fn fees_for_plan(plan: Ts<PlanData>, purchase: Ts<Purchase>) -> Result<Ts<FeesFor>, JsError> {
    let (buying, rates) = purchase.to_rust()?.buying();
    Ok(plan
        .to_rust()?
        .0
        .describe_fees_for(buying, None, &[], &rates)
        .into_ts()?)
}

/// The editor's simple view: the fees for `security` on `exchange`, compared
/// to `original`'s.
#[wasm_bindgen(js_name = simpleFees)]
pub fn simple_fees(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
    original: Option<Ts<PlanData>>,
) -> Result<Ts<SimpleFees>, JsError> {
    let original = self::original(original)?;
    Ok(plan
        .to_rust()?
        .0
        .simple_fees(security.to_rust()?, exchange.to_rust()?, original.as_ref())
        .into_ts()?)
}

/// The editor's full view: every row, compared to `original`'s.
#[wasm_bindgen(js_name = priceList)]
pub fn price_list(
    plan: Ts<PlanData>,
    original: Option<Ts<PlanData>>,
) -> Result<Ts<PriceList>, JsError> {
    let original = self::original(original)?;
    Ok(plan.to_rust()?.0.price_list(original.as_ref()).into_ts()?)
}

#[wasm_bindgen(js_name = setTrade)]
pub fn set_trade(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
    fields: Ts<TradeFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (security, exchange, fields) =
        (security.to_rust()?, exchange.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| plan.set_trade(security, exchange, &fields))
}

/// Sets the price for buying `security` on `exchange` by standing order, in
/// the row it uses.
#[wasm_bindgen(js_name = setStandingOrder)]
pub fn set_standing_order(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
    fields: Ts<TradeFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (security, exchange, fields) =
        (security.to_rust()?, exchange.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| {
        plan.set_standing_order(security, exchange, &fields)
    })
}

/// Sets the custody for holding `security` on `exchange`, in the row it uses.
#[wasm_bindgen(js_name = setCustody)]
pub fn set_custody(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
    fields: Ts<CustodyFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (security, exchange, fields) =
        (security.to_rust()?, exchange.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| plan.set_custody(security, exchange, &fields))
}

#[wasm_bindgen(js_name = setHandling)]
pub fn set_handling(
    plan: Ts<PlanData>,
    fields: Ts<HandlingFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_handling(&fields))
}

#[wasm_bindgen(js_name = setConversion)]
pub fn set_conversion(
    plan: Ts<PlanData>,
    fields: Ts<ConversionFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_conversion(&fields))
}

/// Sets the second conversion fee: each conversion pays whichever is less.
#[wasm_bindgen(js_name = setSecondConversion)]
pub fn set_second_conversion(
    plan: Ts<PlanData>,
    fields: Ts<ConversionFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_second_conversion(&fields))
}

/// Sets what converting costs when buying by standing order.
#[wasm_bindgen(js_name = setStandingOrderConversion)]
pub fn set_standing_order_conversion(
    plan: Ts<PlanData>,
    fields: Ts<ConversionFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_standing_order_conversion(&fields))
}

/// Whether fractions of a share are sold on `exchange`.
#[wasm_bindgen(js_name = setSellsFractions)]
pub fn set_sells_fractions(
    plan: Ts<PlanData>,
    exchange: Ts<Exchange>,
    sells: bool,
) -> Result<Ts<PlanData>, JsError> {
    let exchange = exchange.to_rust()?;
    changed(plan, |plan| {
        plan.set_sells_fractions(exchange, sells);
        Ok(())
    })
}

#[wasm_bindgen(js_name = setMarkup)]
pub fn set_markup(plan: Ts<PlanData>, fields: Ts<MarkupFields>) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_markup(&fields))
}

#[wasm_bindgen(js_name = addTradeRow)]
pub fn add_trade_row(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
) -> Result<Ts<PlanData>, JsError> {
    let (security, exchange) = (security.to_rust()?, exchange.to_rust()?);
    changed(plan, |plan| {
        plan.add_trade_row(security, exchange);
        Ok(())
    })
}

#[wasm_bindgen(js_name = setTradeRow)]
pub fn set_trade_row(
    plan: Ts<PlanData>,
    index: usize,
    coverage: Ts<Coverage>,
    fields: Ts<TradeFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (coverage, fields) = (coverage.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| {
        plan.set_trade_row(index, &coverage.securities, &coverage.exchanges, &fields)
    })
}

#[wasm_bindgen(js_name = removeTradeRow)]
pub fn remove_trade_row(plan: Ts<PlanData>, index: usize) -> Result<Ts<PlanData>, JsError> {
    changed(plan, |plan| plan.remove_trade_row(index))
}

#[wasm_bindgen(js_name = setStandingOrderRow)]
pub fn set_standing_order_row(
    plan: Ts<PlanData>,
    index: usize,
    coverage: Ts<Coverage>,
    fields: Ts<TradeFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (coverage, fields) = (coverage.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| {
        plan.set_standing_order_row(index, &coverage.securities, &coverage.exchanges, &fields)
    })
}

#[wasm_bindgen(js_name = addCustodyRow)]
pub fn add_custody_row(
    plan: Ts<PlanData>,
    exchange: Ts<Exchange>,
) -> Result<Ts<PlanData>, JsError> {
    let exchange = exchange.to_rust()?;
    changed(plan, |plan| {
        plan.add_custody_row(exchange);
        Ok(())
    })
}

#[wasm_bindgen(js_name = setCustodyRow)]
pub fn set_custody_row(
    plan: Ts<PlanData>,
    index: usize,
    coverage: Ts<Coverage>,
    fields: Ts<CustodyFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (coverage, fields) = (coverage.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| {
        plan.set_custody_row(index, &coverage.securities, &coverage.exchanges, &fields)
    })
}

#[wasm_bindgen(js_name = removeCustodyRow)]
pub fn remove_custody_row(plan: Ts<PlanData>, index: usize) -> Result<Ts<PlanData>, JsError> {
    changed(plan, |plan| plan.remove_custody_row(index))
}

/// In ₪; `null` for none.
#[wasm_bindgen(js_name = setMinFirstDeposit)]
pub fn set_min_first_deposit(
    plan: Ts<PlanData>,
    amount: Option<f64>,
) -> Result<Ts<PlanData>, JsError> {
    let amount = amount.and_then(decimal).map(Amount);
    changed(plan, |plan| plan.set_min_first_deposit(amount))
}

#[wasm_bindgen]
pub fn rename(plan: Ts<PlanData>, name: String) -> Result<Ts<PlanData>, JsError> {
    changed(plan, |plan| plan.rename(&name))
}

/// The ways a trade's price can be stated, for the editor's dropdown.
#[wasm_bindgen(js_name = priceKinds)]
pub fn price_kinds() -> Result<Vec<Ts<Named<PriceKind>>>, JsError> {
    PriceKind::iter()
        .map(|value| {
            let name = value.to_string();
            Ok(Named { value, name }.into_ts()?)
        })
        .collect()
}

/// The periods custody is quoted and charged per, for the editor's dropdowns.
#[wasm_bindgen]
pub fn periods() -> Result<Vec<Ts<PeriodName>>, JsError> {
    Period::iter()
        .map(|value| {
            let (name, adverb) = (value.to_string(), value.adverb().to_owned());
            Ok(PeriodName {
                value,
                name,
                adverb,
            }
            .into_ts()?)
        })
        .collect()
}

fn number(value: Decimal) -> f64 {
    value.to_f64().unwrap_or_default()
}

/// Inputs come as numbers; four decimal places are plenty for money and rates.
/// `None` for NaN and infinities.
/// A field's number, or which field to fill in.
fn filled_in(value: Option<f64>, what: &'static str) -> Result<Decimal, InvalidInputs> {
    value.and_then(decimal).ok_or(InvalidInputs::Missing(what))
}

fn decimal(value: f64) -> Option<Decimal> {
    Decimal::try_from(value).ok().map(|value| value.round_dp(4))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> Inputs {
        Inputs {
            security: Security::Etf,
            exchange: Exchange::Usa,
            first_deposit: Some(10_000.0),
            monthly_deposit: Some(2_000.0),
            yearly_return_percent: Some(10.0),
            years: 20,
            buy_every_months: 1,
            share_price: Some(500.0),
            ils_per_usd: Some(3.7),
            ils_per_eur: Some(4.3),
            plans: vec![
                PlanKey::Listed { broker: 0, plan: 0 },
                PlanKey::Listed { broker: 1, plan: 3 },
            ],
            your_plans: vec![],
        }
    }

    #[test]
    fn a_plan_that_offers_nothing_there_says_why() {
        let mut inputs = inputs();
        inputs.exchange = Exchange::Europe;
        let comparison = compare_plans(&inputs).unwrap();
        let altshuler = comparison
            .plans
            .iter()
            .find(|plan| matches!(plan.key, PlanKey::Listed { broker: 0, .. }))
            .unwrap();
        assert!(altshuler.outcome.is_none());
        assert_eq!(
            altshuler.not_offered.as_deref(),
            Some("Nothing in Europe is offered")
        );
        let leumi = comparison
            .plans
            .iter()
            .find(|plan| matches!(plan.key, PlanKey::Listed { broker: 1, .. }))
            .unwrap();
        assert!(leumi.outcome.is_some());
        assert_eq!(leumi.not_offered, None);
    }

    #[test]
    fn compare_returns_every_plan_with_its_years() {
        let comparison = compare_plans(&inputs()).unwrap();
        assert_eq!(comparison.plans.len(), 2);
        assert_eq!(comparison.no_fees.value_by_month.len(), 20 * 12 + 1); // start + 20 years
        assert_eq!(comparison.no_fees.lost_to_fees, 0.0);
        assert_eq!(comparison.deposited, 10_000.0 + 2_000.0 * 12.0 * 20.0);
        for plan in &comparison.plans {
            let outcome = plan.outcome.as_ref().expect("both plans offer US ETFs");
            assert_eq!(outcome.fees_up_to_year.len(), 20);
            assert_eq!(outcome.lost_by_month.len(), 20 * 12 + 1);
            assert_eq!(outcome.lost_by_month.last(), Some(&outcome.lost_to_fees));
            assert!(outcome.lost_to_fees > outcome.fees.total);
            let last = outcome.fees_up_to_year.last().unwrap();
            assert!((last.total - outcome.fees.total).abs() < 0.01);
        }
    }

    #[test]
    fn bad_inputs_are_errors_not_panics() {
        use InvalidInputs::*;
        let with = |change: fn(&mut Inputs)| {
            let mut inputs = inputs();
            change(&mut inputs);
            compare_plans(&inputs).err()
        };
        assert!(matches!(with(|i| i.years = 0), Some(Other(_))));
        assert!(matches!(with(|i| i.buy_every_months = 0), Some(Other(_))));
        assert_eq!(
            with(|i| i.monthly_deposit = Some(-5.0)),
            Some(Negative("the monthly deposit"))
        );
        assert!(matches!(
            with(|i| i.ils_per_usd = Some(0.0)),
            Some(Other(_))
        ));
        assert_eq!(
            with(|i| i.share_price = Some(f64::NAN)),
            Some(Missing("the share price"))
        );
        assert_eq!(
            with(|i| i.yearly_return_percent = None).map(|error| error.to_string()),
            Some("fill in the yearly return".into())
        );
        assert_eq!(
            with(|i| i.yearly_return_percent = Some(-150.0)).map(|error| error.to_string()),
            Some("the yearly return can't be below −100%".into())
        );
        assert_eq!(
            with(|i| i.yearly_return_percent = Some(500.0)).map(|error| error.to_string()),
            Some("the deposits would grow too large to calculate".into())
        );
        assert!(matches!(
            with(|i| i.share_price = Some(0.0)),
            Some(Other(_))
        ));
        assert!(with(|_| {}).is_none());
    }

    #[test]
    fn the_share_price_is_needed_only_where_it_matters() {
        let mut inputs = inputs();
        assert_eq!(
            share_price_matters(&inputs).as_deref(),
            Some("$") // US ETFs are bought in whole shares
        );
        inputs.exchange = Exchange::Tlv;
        assert_eq!(share_price_matters(&inputs), None);
        // Its field is hidden then, and may be empty.
        inputs.share_price = None;
        assert!(compare_plans(&inputs).is_ok());
    }

    fn share_price_matters(inputs: &Inputs) -> Option<String> {
        let brokers = all_brokers();
        let plans = chosen_plans(inputs, &brokers).unwrap();
        simulation::share_price_matters(inputs.security, inputs.exchange, &plans)
            .then(|| inputs.exchange.currency().symbol.to_owned())
    }

    #[test]
    fn plans_say_what_their_numbers_need_to_make_sense() {
        let plan = |comparison: &ComparisonData, key: &PlanKey| {
            let plan = comparison.plans.iter().find(|p| p.key == *key).unwrap();
            (plan.warning.clone(), plan.note.clone())
        };
        let mut inputs = inputs();
        inputs.security = Security::IndexFund;
        inputs.exchange = Exchange::Tlv;
        inputs.buy_every_months = 3;
        let standing_order = PlanKey::Listed { broker: 1, plan: 2 };
        inputs.plans.push(standing_order.clone());
        let (warning, note) = plan(&compare_plans(&inputs).unwrap(), &standing_order);
        assert_eq!(warning, None);
        assert!(note.unwrap().contains("standing order"));

        // Altshuler's custody is at least ₪75 a month.
        inputs.first_deposit = Some(0.0);
        inputs.monthly_deposit = Some(50.0);
        let altshuler = PlanKey::Listed { broker: 0, plan: 0 };
        let (warning, _) = plan(&compare_plans(&inputs).unwrap(), &altshuler);
        assert_eq!(
            warning.as_deref(),
            Some("Its fees are more than you deposit")
        );
    }

    #[test]
    fn the_cheapest_track_is_named() {
        let comparison = compare_plans(&inputs()).unwrap();
        let full_tariff = PlanKey::Listed { broker: 0, plan: 0 };
        let altshuler = comparison
            .plans
            .iter()
            .find(|p| p.key == full_tariff)
            .unwrap();
        let track = altshuler.track.expect("its tracks price US ETFs");
        let name = &listed(0, 0).unwrap().tracks[track].name;
        assert_eq!(
            altshuler.note,
            Some(format!("US track: {name}, the cheapest for you"))
        );
        // Its fees count the handling fee.
        let fees = &altshuler.outcome.as_ref().unwrap().fees;
        assert!(fees.handling > 0.0);
        let parts = fees.purchases + fees.conversions + fees.custody + fees.handling + fees.selling;
        assert!((parts - fees.total).abs() < 0.01);
        // A copy made from the comparison is on that track, and so is the
        // listed plan it's compared with.
        let copy = listed(0, 0).unwrap().copy_of(Some(track));
        let original = listed(0, 0).unwrap().on_track(track);
        let etf = |plan: &Plan| {
            plan.trade_row(Security::Etf, Exchange::Usa)
                .map(|row| row.price.clone())
        };
        assert!(etf(&copy).is_some());
        assert_eq!(etf(&copy), etf(&original));
    }

    #[test]
    fn your_plans_are_compared_by_id() {
        let mut inputs = inputs();
        inputs.your_plans = vec![YourPlanInput {
            id: "abc".into(),
            plan: PlanData(listed(1, 3).unwrap().copy_of(None)),
        }];
        inputs.plans.push(PlanKey::Yours { id: "abc".into() });
        let comparison = compare_plans(&inputs).unwrap();
        let outcome = |key: &PlanKey| {
            let plan = comparison.plans.iter().find(|p| p.key == *key).unwrap();
            plan.outcome.as_ref().unwrap().after_selling
        };
        // An unchanged copy of Pepper ends with what Pepper does.
        let pepper = PlanKey::Listed { broker: 1, plan: 3 };
        assert_eq!(
            outcome(&PlanKey::Yours { id: "abc".into() }),
            outcome(&pepper)
        );

        inputs.plans.push(PlanKey::Yours { id: "gone".into() });
        assert!(matches!(
            compare_plans(&inputs),
            Err(InvalidInputs::Other(_))
        ));
    }

    #[test]
    fn broker_info_describes_every_plan() {
        let leumi = BrokerInfo::from(&tariffs::leumi());
        assert_eq!(leumi.tariff_date, "Tariff of 29/06/2026");
        assert_eq!(leumi.plans.len(), 4);
        assert_eq!(leumi.plans[0].tariff.trading.len(), 2);
        assert_ne!(leumi.plans[3].description, "");
        let standing_order = &leumi.plans[2].tariff;
        assert_eq!(
            standing_order.standing_orders[0].covers,
            "Index fund on Tel Aviv"
        );
        assert_eq!(
            leumi.plans[1]
                .tariff
                .second_conversion
                .as_ref()
                .map(|price| price.text.as_str()),
            Some("0.16%, min $5.76, max $2,400")
        );
    }

    #[test]
    fn choices_have_display_names() {
        let choice = Choice::from(Security::IndexFund);
        assert_eq!(choice.name, "Index fund");
    }
}
