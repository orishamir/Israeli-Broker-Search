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

use broker_fees::describe::{self, Explained, FeeKind, FeesFor};
use broker_fees::simulation::{self, Fees, Outcome, Scenario};
use broker_fees::yours::{
    Amount, ConversionFields, CustodyFields, InvalidFee, MarkupFields, PriceKind, PriceList,
    SimpleFees, TradeFields,
};
use broker_fees::{
    Broker, Caveat, Exchange, ExchangeRates, IntoEnumIterator, Percent, Period, Plan, Security,
    ils, tariffs,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

fn all_brokers() -> Vec<Broker> {
    vec![tariffs::altshuler(), tariffs::leumi()]
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

/// The kinds of fee, named and explained, in the order the editor shows them.
#[wasm_bindgen(js_name = feeKinds)]
pub fn fee_kinds() -> Result<Vec<Ts<Choice<FeeKind>>>, JsError> {
    FeeKind::iter()
        .map(|kind| Ok(Choice::from(kind).into_ts()?))
        .collect()
}

// ─────────────────────────── Brokers ───────────────────────────

/// A broker, as the sidebar and its details show it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BrokerInfo {
    pub name: String,
    pub description: String,
    /// "Tariff of 29/06/2026"
    pub tariff_date: String,
    pub source_url: Option<String>,
    /// Caveats that apply to every plan, for every security and exchange.
    pub caveats: Vec<CaveatInfo>,
    pub plans: Vec<PlanInfo>,
}

impl From<&Broker> for BrokerInfo {
    fn from(broker: &Broker) -> Self {
        BrokerInfo {
            name: broker.name.clone(),
            description: broker.description.clone(),
            tariff_date: broker.tariff_date_text().to_string(),
            source_url: broker.source_url.clone(),
            caveats: broker.caveats.iter().map(CaveatInfo::from).collect(),
            plans: broker.plans.iter().map(PlanInfo::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub name: String,
    pub description: String,
    /// For every security and exchange.
    pub caveats: Vec<CaveatInfo>,
    /// Every row of the plan's tariff, in words.
    pub tariff: TariffInfo,
}

impl From<&Plan> for PlanInfo {
    fn from(plan: &Plan) -> Self {
        PlanInfo {
            name: plan.name.clone(),
            description: plan.description.clone(),
            caveats: plan.caveats.iter().map(CaveatInfo::from).collect(),
            tariff: TariffInfo {
                trading: plan
                    .trading
                    .iter()
                    .map(|row| TariffRow {
                        covers: row.coverage().to_string(),
                        price: row.price.to_string(),
                    })
                    .collect(),
                custody: plan
                    .custody
                    .iter()
                    .map(|row| TariffRow {
                        covers: row.coverage().to_string(),
                        price: row.to_string(),
                    })
                    .collect(),
                conversion: plan.conversion.to_string(),
                markup: plan.conversion.markup.to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TariffInfo {
    pub trading: Vec<TariffRow>,
    pub custody: Vec<TariffRow>,
    pub conversion: String,
    pub markup: String,
}

/// A caveat, and what it's about: "Mutual fund on Tel Aviv".
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct CaveatInfo {
    pub text: String,
    pub covers: String,
}

impl From<&Caveat> for CaveatInfo {
    fn from(caveat: &Caveat) -> Self {
        CaveatInfo {
            text: caveat.text.clone(),
            covers: caveat.coverage().to_string(),
        }
    }
}

/// "ETF on Tel Aviv" → "0.15%, min ₪3.5".
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TariffRow {
    pub covers: String,
    pub price: String,
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

/// What plan `plan` of broker `broker` charges for `security` on `exchange`,
/// in words.
#[wasm_bindgen(js_name = feesFor)]
pub fn fees_for(
    broker: usize,
    plan: usize,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
) -> Result<Ts<FeesFor>, JsError> {
    let brokers = all_brokers();
    let broker = brokers
        .get(broker)
        .ok_or_else(|| JsError::new("no such broker"))?;
    let plan = broker
        .plans
        .get(plan)
        .ok_or_else(|| JsError::new("no such plan"))?;
    Ok(broker
        .describe_fees_for(plan, security.to_rust()?, exchange.to_rust()?)
        .into_ts()?)
}

/// The caveats about every plan of broker `broker` that matter to someone
/// buying `security` on `exchange`.
#[wasm_bindgen(js_name = brokerCaveats)]
pub fn broker_caveats(
    broker: usize,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
) -> Result<Vec<String>, JsError> {
    let brokers = all_brokers();
    let broker = brokers
        .get(broker)
        .ok_or_else(|| JsError::new("no such broker"))?;
    Ok(broker.caveats_for(security.to_rust()?, exchange.to_rust()?))
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
}

impl TryFrom<&Inputs> for Scenario {
    type Error = InvalidInputs;

    fn try_from(inputs: &Inputs) -> Result<Self, Self::Error> {
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
        Ok(Scenario {
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
            share_price: amount(inputs.share_price, "the share price")?,
        })
    }
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
    /// Why the plan can't be used as the inputs are, though its numbers are
    /// still shown: "Needs a one-time deposit of at least ₪5,000".
    pub warning: Option<String>,
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
    pub selling: f64,
    pub total: f64,
}

impl From<&Fees> for FeeAmounts {
    fn from(fees: &Fees) -> Self {
        FeeAmounts {
            purchases: number(fees.purchases),
            conversions: number(fees.conversions),
            custody: number(fees.custody),
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
    let scenario = Scenario::try_from(inputs)?;
    let rates = ExchangeRates::new(
        filled_in(inputs.ils_per_usd, "the dollar's rate")?,
        filled_in(inputs.ils_per_eur, "the euro's rate")?,
    )
    .map_err(|_| InvalidInputs::Other("exchange rates must be more than 0"))?;
    let brokers = all_brokers();
    let plans = inputs
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
        .ok_or(InvalidInputs::Other("there's no such plan"))?;
    let comparison = simulation::compare(&plans, &scenario, &rates);
    let no_fees = &comparison.no_fees;
    Ok(ComparisonData {
        deposited: number(scenario.deposited()),
        no_fees: OutcomeData::compared_to(no_fees, no_fees),
        plans: comparison
            .plans
            .iter()
            .map(|compared| PlanOutcomeData {
                key: inputs.plans[compared.index].clone(),
                outcome: compared
                    .outcome
                    .as_ref()
                    .map(|outcome| OutcomeData::compared_to(outcome, no_fees)),
                warning: plans[compared.index].first_deposit_warning(ils(scenario.first_deposit)),
            })
            .collect(),
    })
}

// ─────────────────────────── Your plans ───────────────────────────

/// One of the user's own plans. The web app keeps it but never looks inside:
/// only the functions here read and change it.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
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

/// A listed plan, as your plans are kept: to compare them with.
#[wasm_bindgen(js_name = listedPlan)]
pub fn listed_plan(broker: usize, plan: usize) -> Result<Ts<PlanData>, JsError> {
    Ok(PlanData(listed(broker, plan)?).into_ts()?)
}

/// A copy of a listed plan to change: "Pepper, your deal".
#[wasm_bindgen(js_name = copyOf)]
pub fn copy_of(broker: usize, plan: usize) -> Result<Ts<PlanData>, JsError> {
    Ok(PlanData(listed(broker, plan)?.copy_of()).into_ts()?)
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
pub fn fees_for_plan(
    plan: Ts<PlanData>,
    security: Ts<Security>,
    exchange: Ts<Exchange>,
) -> Result<Ts<FeesFor>, JsError> {
    Ok(plan
        .to_rust()?
        .0
        .describe_fees_for(security.to_rust()?, exchange.to_rust()?)
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

#[wasm_bindgen(js_name = setCustody)]
pub fn set_custody(
    plan: Ts<PlanData>,
    exchange: Ts<Exchange>,
    fields: Ts<CustodyFields>,
) -> Result<Ts<PlanData>, JsError> {
    let (exchange, fields) = (exchange.to_rust()?, fields.to_rust()?);
    changed(plan, |plan| plan.set_custody(exchange, &fields))
}

#[wasm_bindgen(js_name = setConversion)]
pub fn set_conversion(
    plan: Ts<PlanData>,
    fields: Ts<ConversionFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_conversion(&fields))
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
        plan.set_custody_row(index, &coverage.exchanges, &fields)
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
        assert!(with(|_| {}).is_none());
    }

    #[test]
    fn your_plans_are_compared_by_id() {
        let mut inputs = inputs();
        inputs.your_plans = vec![YourPlanInput {
            id: "abc".into(),
            plan: PlanData(listed(1, 3).unwrap().copy_of()),
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
    }

    #[test]
    fn choices_have_display_names() {
        let choice = Choice::from(Security::MutualFund);
        assert_eq!(choice.name, "Mutual fund");
    }
}
