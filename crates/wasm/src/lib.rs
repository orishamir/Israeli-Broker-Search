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

use broker_fees::describe::{self, Explained, FeesFor};
use broker_fees::simulation::{self, Fees, Outcome, Scenario};
use broker_fees::{
    Broker, Caveat, Exchange, ExchangeRates, IntoEnumIterator, Percent, Plan, Security, ils,
    tariffs,
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
}

/// Inputs the simulation can't run with. Each holds what's wrong, in words.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum InvalidInputs {
    /// An empty field: "the first deposit".
    #[error("fill in {0}")]
    Missing(&'static str),
    /// A negative amount: "the first deposit".
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
            first_deposit: amount(inputs.first_deposit, "the first deposit")?,
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

/// A plan, by the positions of its broker in [`brokers`] and of it in the
/// broker's plans.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
pub struct PlanKey {
    pub broker: usize,
    pub plan: usize,
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
    /// still shown: "Needs a first deposit of at least ₪5,000".
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
    let plans: Vec<(usize, usize)> = inputs
        .plans
        .iter()
        .map(|key| (key.broker, key.plan))
        .collect();
    let brokers = all_brokers();
    let comparison = simulation::compare(&brokers, &plans, &scenario, &rates);
    let no_fees = &comparison.no_fees;
    Ok(ComparisonData {
        deposited: number(scenario.deposited()),
        no_fees: OutcomeData::compared_to(no_fees, no_fees),
        plans: comparison
            .plans
            .iter()
            .map(|plan| PlanOutcomeData {
                key: PlanKey {
                    broker: plan.broker,
                    plan: plan.plan,
                },
                outcome: plan
                    .outcome
                    .as_ref()
                    .map(|outcome| OutcomeData::compared_to(outcome, no_fees)),
                warning: brokers[plan.broker].plans[plan.plan]
                    .first_deposit_warning(ils(scenario.first_deposit)),
            })
            .collect(),
    })
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
                PlanKey { broker: 0, plan: 0 },
                PlanKey { broker: 1, plan: 3 },
            ],
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
