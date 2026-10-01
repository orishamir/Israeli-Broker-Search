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

use std::cell::Cell;

use broker_fees::describe::{
    self, About, CaveatGroup, Explained, FeeKind, FeesFor, PriceText, Priced, Source,
};
use broker_fees::examples;
use broker_fees::short_term::{self, Liquidity, Pays, Place, Term};
use broker_fees::simulation::{self, Fees, InvalidScenario, NotOffered, Outcome, Scenario, Swept};
use broker_fees::yours::{
    Amount, ConversionFields, CustodyFields, HandlingFields, InvalidFee, ManagementFields,
    MarkupFields, PriceKind, PriceList, SimpleFees, TradeFields,
};
use broker_fees::{
    Broker, BrokerKind, Buying, Exchange, ExchangeRates, IntoEnumIterator, Lang, Money, Named,
    Percent, Period, Plan, Security, TradeFee, Withdrawal, ils,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// The brokers, then the kinds of fund.
fn all_brokers() -> Vec<Broker> {
    broker_fees::listed()
}

// ─────────────────────────── Language ───────────────────────────

thread_local! {
    /// The language every binding answers in. WebAssembly runs on one
    /// thread, so a thread-local is the whole picture.
    static LANG: Cell<Lang> = const { Cell::new(Lang::En) };
}

/// Sets the language the app is shown in. Everything the bindings return
/// after this is in it.
#[wasm_bindgen(js_name = setLang)]
pub fn set_lang(lang: Ts<Lang>) -> Result<(), JsError> {
    let lang = lang.to_rust()?;
    LANG.with(|current| current.set(lang));
    Ok(())
}

fn lang() -> Lang {
    LANG.with(Cell::get)
}

/// An error for the app to show, in the app's language.
fn shown(text: &str) -> JsError {
    JsError::new(text)
}

/// Inputs the simulation can't run with, as the error the app shows.
fn invalid(error: InvalidInputs) -> JsError {
    shown(&error.text(lang()))
}

// ─────────────────────────── Choices ───────────────────────────

/// Something the user can pick, with its name to show and what it means.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Choice<T> {
    pub value: T,
    pub name: String,
    /// Its English name, shown beside the Hebrew one in Hebrew; missing
    /// where it wouldn't help (an exchange).
    pub english_name: Option<String>,
    pub explanation: String,
    /// What Israeli brokers call it: "קרן סל".
    pub hebrew_names: Vec<String>,
}

impl<T: Explained> Choice<T> {
    fn new(value: T, lang: Lang) -> Self {
        Choice {
            value,
            name: value.name(lang).to_owned(),
            english_name: value.english_name().map(str::to_owned),
            explanation: value.explanation(lang).to_owned(),
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
        .map(|security| Ok(Choice::new(security, lang()).into_ts()?))
        .collect()
}

/// The exchanges, in the order to offer them.
#[wasm_bindgen]
pub fn exchanges() -> Result<Vec<Ts<Choice<Exchange>>>, JsError> {
    Exchange::iter()
        .map(|exchange| Ok(Choice::new(exchange, lang()).into_ts()?))
        .collect()
}

/// A kind of fee, like a [`Choice`], with its name under the fee it's part of.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FeeKindChoice {
    pub value: FeeKind,
    pub name: String,
    pub english_name: Option<String>,
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
                english_name,
                explanation,
                hebrew_names,
            } = Choice::new(kind, lang());
            let label = kind.label(lang()).to_owned();
            Ok(FeeKindChoice {
                value,
                name,
                english_name,
                label,
                explanation,
                hebrew_names,
            }
            .into_ts()?)
        })
        .collect()
}

// ─────────────────────────── Brokers ───────────────────────────

/// A broker or a kind of fund, as the sidebar and its details show it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BrokerInfo {
    /// A bank, an investment house, or a kind of fund: the sidebar lists
    /// the funds apart.
    pub kind: BrokerKind,
    pub name: String,
    /// The English name, which saved copies of its plans name it by.
    pub english_name: String,
    /// "Leumi", beside a plan's name where plan names repeat: "Leumi · Online".
    pub short_name: String,
    /// The English short name, for links, which name plans the same in
    /// every language.
    pub english_short_name: String,
    /// Which of `plans` a new customer usually gets.
    pub new_customer_plan: usize,
    /// Whether that plan is ticked when the app opens.
    pub compared_at_first: bool,
    /// A fund's tax rule in a line, shown under its name: "No tax on gains
    /// after 6 years, on up to ₪20,566 deposited a year". Missing for a
    /// broker.
    pub tax_rule: Option<String>,
    pub description: String,
    /// What "usual" means beside the plan a new customer gets: a bank's
    /// online prices, an investment house's joining offer.
    pub usual_plan: String,
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

impl BrokerInfo {
    fn new(broker: &Broker, lang: Lang) -> Self {
        BrokerInfo {
            kind: broker.kind,
            name: broker.name[lang].to_owned(),
            english_name: broker.name.en.to_string(),
            short_name: broker.short_name[lang].to_owned(),
            english_short_name: broker.short_name.en.to_string(),
            new_customer_plan: broker.new_customer_plan,
            compared_at_first: broker.compared_at_first,
            // Every plan of a broker or a fund is in the same vehicle.
            tax_rule: broker
                .plans
                .first()
                .and_then(|plan| plan.vehicle.tax_rule(lang)),
            description: broker.description[lang].to_owned(),
            usual_plan: broker.usual_plan_text(lang),
            tariff_date: broker.tariff_date_text(lang),
            checked: broker.checked_on_text(lang),
            source_url: broker.source_url.clone(),
            sources: broker.sources(lang),
            plans: broker
                .plans
                .iter()
                .map(|plan| PlanInfo {
                    sources: broker.sources_for(plan, lang),
                    ..PlanInfo::new(plan, lang)
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub name: String,
    /// The English name, for links, which name plans the same in every
    /// language.
    pub english_name: String,
    pub description: String,
    /// The age from which it can be taken as a monthly pension, taxed
    /// differently: 60 for a provident fund for investment. Missing where
    /// there's no such way out.
    pub pension_from_age: Option<u32>,
    /// Every row of the plan's tariff, in words.
    pub tariff: TariffInfo,
    /// The pages its numbers rest on: the tariff, then what the broker-wide
    /// caveats and its own rest on, each once.
    pub sources: Vec<Source>,
}

impl PlanInfo {
    fn new(plan: &Plan, lang: Lang) -> Self {
        let trade_rows = |rows: &[TradeFee]| {
            rows.iter()
                .map(|row| TariffRow {
                    covers: row.coverage(lang),
                    price: row.price.price_text(lang),
                })
                .collect()
        };
        PlanInfo {
            name: plan.name[lang].to_owned(),
            english_name: plan.name.en.to_string(),
            description: plan.description[lang].to_owned(),
            pension_from_age: plan.vehicle.rules().pension.map(|pension| pension.from_age),
            // Filled in by the broker, which knows its tariff and its caveats.
            sources: vec![],
            tariff: TariffInfo {
                management: plan
                    .management
                    .filter(|_| plan.vehicle.invests_for_you())
                    .map(|fee| ManagementInfo {
                        of_balance: fee.balance_price_text(lang),
                        of_deposits: fee.deposit_price_text(lang),
                    }),
                trading: trade_rows(&plan.trading),
                tracks: plan
                    .tracks
                    .iter()
                    .map(|track| TrackInfo {
                        name: track.name[lang].to_owned(),
                        english_name: track.name.en.to_string(),
                        trading: trade_rows(&track.trading),
                    })
                    .collect(),
                standing_orders: trade_rows(&plan.standing_orders),
                custody: plan
                    .custody
                    .iter()
                    .map(|row| TariffRow {
                        covers: row.coverage(lang),
                        price: row.price_text(lang),
                    })
                    .collect(),
                handling: plan.handling.map(|fee| fee.price_text(lang)),
                conversion: plan.conversion.price_text(lang),
                second_conversion: plan.conversion.or_if_less.map(|fee| fee.price_text(lang)),
                standing_order_conversion: plan
                    .standing_order_conversion
                    .as_ref()
                    .map(|fee| fee.price_text(lang)),
                markup: plan.conversion.markup.price_text(lang),
                fractions_on: plan
                    .fractions_on
                    .iter()
                    .map(|exchange| exchange.name(lang).to_owned())
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TariffInfo {
    /// A fund's or a policy's fee: all it charges, so the rest isn't shown.
    pub management: Option<ManagementInfo>,
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

/// "0.62% of the balance a year" and "none": a manager's two fees.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ManagementInfo {
    pub of_balance: PriceText,
    pub of_deposits: PriceText,
}

/// "1¢ a share", and the trade prices it sets.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub name: String,
    /// The English name, which a copy made on the track is saved with.
    pub english_name: String,
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
        .map(|broker| Ok(BrokerInfo::new(broker, lang()).into_ts()?))
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
        .describe_fees_for(plan, buying, track, &rates, lang())
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
        .caveats_for(buying, &rates, lang())
        .into_iter()
        .map(|group| Ok(group.into_ts()?))
        .collect()
}

/// The page that explains the numbers: how they're made, what isn't
/// counted, and the sources.
#[wasm_bindgen]
pub fn about() -> Result<Ts<About>, JsError> {
    Ok(describe::about(lang()).into_ts()?)
}

/// "an ETF bought in the USA"
#[wasm_bindgen(js_name = purchasePhrase)]
pub fn purchase_phrase(security: Ts<Security>, exchange: Ts<Exchange>) -> Result<String, JsError> {
    Ok(describe::purchase(
        security.to_rust()?,
        exchange.to_rust()?,
        lang(),
    ))
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
    /// How much more is deposited each year than the year before: 3 means 3%.
    #[tsify(type = "number | null")]
    pub deposit_growth_percent: Option<f64>,
    /// How much prices rise a year: 2 means 2%. The tax is on the gain
    /// beyond it.
    #[tsify(type = "number | null")]
    pub inflation_percent: Option<f64>,
    /// Whether every amount is shown in today's shekels, with the rise in
    /// prices taken off.
    pub in_todays_money: bool,
    /// Whether the money is taken as a monthly pension where a plan pays
    /// one, rather than all at once.
    pub as_pension: bool,
    /// The saver's age today. Asked for only with `as_pension`: a pension
    /// opens from an age.
    #[tsify(type = "number | null")]
    pub age: Option<f64>,
    /// Whether everything is sold at the end, or kept.
    pub sell_at_end: bool,
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

/// A field of the inputs, to name in an error: "the one-time deposit".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    FirstDeposit,
    MonthlyDeposit,
    DepositGrowth,
    YearlyReturn,
    SharePrice,
    Inflation,
    Age,
    UsdRate,
    EurRate,
    Rate,
    YourRate,
}

impl Named for Field {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Field::FirstDeposit => lang.pick("the one-time deposit", "ההפקדה החד\u{2011}פעמית"),
            Field::MonthlyDeposit => lang.pick("the monthly deposit", "ההפקדה החודשית"),
            Field::DepositGrowth => {
                lang.pick("the deposits' yearly growth", "הגידול השנתי של ההפקדות")
            }
            Field::YearlyReturn => lang.pick("the yearly return", "התשואה השנתית"),
            Field::SharePrice => lang.pick("the share price", "מחיר המניה"),
            Field::Inflation => lang.pick("the inflation", "האינפלציה"),
            Field::Age => lang.pick("your age", "הגיל שלכם"),
            Field::UsdRate => lang.pick("the dollar's rate", "שער הדולר"),
            Field::EurRate => lang.pick("the euro's rate", "שער האירו"),
            Field::Rate => lang.pick("the Bank of Israel's rate", "ריבית בנק ישראל"),
            Field::YourRate => lang.pick("your deposit's rate", "הריבית של הפיקדון שלכם"),
        }
    }
}

/// A problem with the inputs that isn't about one field's number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    NoSuchPlan,
    NoYears,
    NeverBuys,
    SharePriceZero,
    InflationTooLow,
    RatesNotPositive,
}

impl Named for Problem {
    fn name(self, lang: Lang) -> &'static str {
        match self {
            Problem::NoSuchPlan => lang.pick("there's no such plan", "אין מסלול כזה"),
            Problem::NoYears => lang.pick("invest for at least a year", "השקיעו לפחות שנה אחת"),
            Problem::NeverBuys => {
                lang.pick("buy at least every 12 months", "קנו לפחות פעם ב-12 חודשים")
            }
            Problem::SharePriceZero => lang.pick(
                "the share price must be more than 0",
                "מחיר המניה חייב להיות גדול מ-0",
            ),
            Problem::InflationTooLow => lang.pick(
                "inflation can't be −100% or below",
                "האינפלציה לא יכולה להיות −100% או פחות",
            ),
            Problem::RatesNotPositive => lang.pick(
                "exchange rates must be more than 0",
                "שערי החליפין חייבים להיות גדולים מ-0",
            ),
        }
    }
}

/// Inputs the simulation can't run with. Each holds what's wrong; its
/// `Display` says so in English, and [`InvalidInputs::text`] in the app's
/// language.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum InvalidInputs {
    /// An empty field: "the one-time deposit".
    #[error("fill in {}", .0.name(Lang::En))]
    Missing(Field),
    /// A negative amount: "the one-time deposit".
    #[error("{} can't be negative", .0.name(Lang::En))]
    Negative(Field),
    /// Any other problem: "invest for at least a year".
    #[error("{}", .0.name(Lang::En))]
    Other(Problem),
    /// Numbers the simulation can't handle: "the yearly return can't be below −100%".
    #[error(transparent)]
    Scenario(#[from] InvalidScenario),
    /// The same for the short term: "the money can be kept for 1 to 60 months".
    #[error(transparent)]
    ShortTerm(#[from] short_term::InvalidScenario),
}

impl InvalidInputs {
    /// What's wrong, in `lang`.
    fn text(&self, lang: Lang) -> String {
        match self {
            InvalidInputs::Missing(field) => {
                let field = field.name(lang);
                match lang {
                    Lang::En => format!("fill in {field}"),
                    Lang::He => format!("מלאו את {field}"),
                }
            }
            InvalidInputs::Negative(field) => {
                let field = field.name(lang);
                match lang {
                    Lang::En => format!("{field} can't be negative"),
                    Lang::He => format!("אי אפשר להזין ערך שלילי ב{field}"),
                }
            }
            InvalidInputs::Other(problem) => problem.name(lang).to_owned(),
            InvalidInputs::Scenario(error) => error.text(lang).to_owned(),
            InvalidInputs::ShortTerm(error) => error.text(lang).to_owned(),
        }
    }
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
        .ok_or(InvalidInputs::Other(Problem::NoSuchPlan))
}

/// What `inputs` describes, checked. The share price is needed only if it
/// matters on one of `plans`: otherwise its field is hidden.
fn scenario(inputs: &Inputs, plans: &[&Plan]) -> Result<Scenario, InvalidInputs> {
    if inputs.years == 0 {
        return Err(InvalidInputs::Other(Problem::NoYears));
    }
    if inputs.buy_every_months == 0 {
        return Err(InvalidInputs::Other(Problem::NeverBuys));
    }
    let amount = |value, what| {
        let amount = filled_in(value, what)?;
        if amount.is_sign_negative() {
            return Err(InvalidInputs::Negative(what));
        }
        Ok(amount)
    };
    let share_price = if simulation::share_price_matters(inputs.security, inputs.exchange, plans) {
        let share_price = amount(inputs.share_price, Field::SharePrice)?;
        if share_price.is_zero() {
            return Err(InvalidInputs::Other(Problem::SharePriceZero));
        }
        share_price
    } else {
        Decimal::ZERO
    };
    let scenario = Scenario {
        security: inputs.security,
        exchange: inputs.exchange,
        first_deposit: amount(inputs.first_deposit, Field::FirstDeposit)?,
        monthly_deposit: amount(inputs.monthly_deposit, Field::MonthlyDeposit)?,
        deposit_growth: Percent(filled_in(
            inputs.deposit_growth_percent,
            Field::DepositGrowth,
        )?),
        yearly_return: Percent(filled_in(
            inputs.yearly_return_percent,
            Field::YearlyReturn,
        )?),
        years: inputs.years,
        buy_every_months: inputs.buy_every_months,
        share_price,
        sell_at_end: inputs.sell_at_end,
        inflation: inflation(inputs)?,
        age: age(inputs)?,
        withdrawal: if inputs.as_pension {
            Withdrawal::Pension
        } else {
            Withdrawal::LumpSum
        },
    };
    scenario.check()?;
    Ok(scenario)
}

/// The inflation to take off, checked: prices can't fall to nothing.
fn inflation(inputs: &Inputs) -> Result<Percent, InvalidInputs> {
    let inflation = filled_in(inputs.inflation_percent, Field::Inflation)?;
    if inflation <= -Decimal::ONE_HUNDRED {
        return Err(InvalidInputs::Other(Problem::InflationTooLow));
    }
    Ok(Percent(inflation))
}

/// The saver's age today, in whole years. It only matters for a pension,
/// so taking the money at once doesn't ask for it.
fn age(inputs: &Inputs) -> Result<u32, InvalidInputs> {
    if !inputs.as_pension {
        return Ok(0);
    }
    let age = filled_in(inputs.age, Field::Age)?;
    if age.is_sign_negative() {
        return Err(InvalidInputs::Negative(Field::Age));
    }
    // Past any age, a saver is simply old enough.
    Ok(age.floor().to_u32().unwrap_or(u32::MAX))
}

fn exchange_rates(inputs: &Inputs) -> Result<ExchangeRates, InvalidInputs> {
    ExchangeRates::new(
        filled_in(inputs.ils_per_usd, Field::UsdRate)?,
        filled_in(inputs.ils_per_eur, Field::EurRate)?,
    )
    .map_err(|_| InvalidInputs::Other(Problem::RatesNotPositive))
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
    /// Best first (most left after selling and paying the tax); plans that
    /// can't be used last.
    pub plans: Vec<PlanOutcomeData>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanOutcomeData {
    pub key: PlanKey,
    /// Missing if the plan can't be used for the inputs.
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
    /// What became of a fund's tax, under its name: "No tax: your deposits
    /// are within ₪20,566 a year". Missing for a broker's plan, when
    /// nothing is sold, and with the outcome.
    pub tax_note: Option<String>,
    /// Why it has no numbers, in a sentence: it doesn't offer the security
    /// there, in the most general terms that are true ("Nothing in Europe
    /// is offered"), the deposits are over its yearly ceiling, and by how
    /// much, or its money is still locked at the end.
    pub not_offered: Option<String>,
    /// Which of those it is, for the few words the table itself says.
    pub why_not: Option<WhyNot>,
}

/// Why a plan can't be used for the inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Tsify)]
pub enum WhyNot {
    /// It doesn't sell the security on that exchange.
    NotSold,
    /// A year's deposits are more than it takes.
    OverTheCeiling,
    /// Its money can't be taken out yet when the years are up.
    Locked,
}

impl From<NotOffered> for WhyNot {
    fn from(reason: NotOffered) -> Self {
        match reason {
            NotOffered::NoPrice => WhyNot::NotSold,
            NotOffered::OverTheCeiling { .. } => WhyNot::OverTheCeiling,
            NotOffered::Locked { .. } => WhyNot::Locked,
        }
    }
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
    /// The tax on the gain, paid on selling: nothing while the holdings
    /// are kept.
    pub tax: f64,
    /// What's left of `after_selling` after the tax: what the plans are
    /// ranked by.
    pub after_tax: f64,
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
    /// What the fees amount to as a yearly charge on the holdings, like a
    /// fund's management fee: 0.42 means 0.42% a year. 100 when nothing is
    /// left.
    pub yearly_cost_percent: f64,
}

impl OutcomeData {
    fn compared_to(outcome: &Outcome, no_fees: &Outcome) -> Self {
        OutcomeData {
            value_by_month: outcome.value_by_month.iter().copied().map(number).collect(),
            held: number(outcome.held),
            after_selling: number(outcome.after_selling),
            tax: number(outcome.tax),
            after_tax: number(outcome.after_tax),
            lost_to_fees: number(outcome.lost_to_fees(no_fees)),
            lost_by_month: outcome.lost_by_month(no_fees).map(number).collect(),
            fees: (&outcome.fees).into(),
            fees_up_to_year: outcome
                .fees_up_to_each_year()
                .iter()
                .map(FeeAmounts::from)
                .collect(),
            yearly_cost_percent: number(outcome.yearly_cost.0),
        }
    }
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct FeeAmounts {
    pub purchases: f64,
    pub conversions: f64,
    /// What's paid just for holding the money: a broker's custody and
    /// monthly handling fee, or a fund's or a policy's management fee. One
    /// amount, since no plan charges both, so ranking by it puts brokers and
    /// funds side by side.
    pub holding: f64,
    pub selling: f64,
    pub total: f64,
}

impl From<&Fees> for FeeAmounts {
    fn from(fees: &Fees) -> Self {
        FeeAmounts {
            purchases: number(fees.purchases),
            conversions: number(fees.conversions),
            holding: number(fees.custody + fees.handling + fees.management),
            selling: number(fees.selling),
            total: number(fees.total()),
        }
    }
}

/// Simulates the chosen plans for `inputs` and ranks them.
#[wasm_bindgen]
pub fn compare(inputs: Ts<Inputs>) -> Result<Ts<ComparisonData>, JsError> {
    Ok(compare_plans(&inputs.to_rust()?)
        .map_err(invalid)?
        .into_ts()?)
}

pub fn compare_plans(inputs: &Inputs) -> Result<ComparisonData, InvalidInputs> {
    let brokers = all_brokers();
    let plans = chosen_plans(inputs, &brokers)?;
    let scenario = scenario(inputs, &plans)?;
    let rates = exchange_rates(inputs)?;
    let inflation = inflation(inputs)?;
    let mut comparison = simulation::compare(&plans, &scenario, &rates);
    let mut deposited = scenario.deposited();
    if inputs.in_todays_money && !inflation.is_zero() {
        comparison = comparison.in_todays_money(inflation);
        deposited = scenario.deposited_in_todays_money(inflation);
    }
    let no_fees = &comparison.no_fees;
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
                let outcome = compared.outcome.as_ref().ok();
                let fees_warning =
                    || outcome.and_then(|outcome| outcome.warning(deposited, lang()));
                let track = outcome.and_then(|outcome| outcome.track);
                // Only a listed plan has caveats; the user's own are their
                // own claim.
                let may_cost_more = match key {
                    PlanKey::Listed { broker, .. } => {
                        brokers[*broker].may_cost_more(plan, buying, &rates, lang())
                    }
                    PlanKey::Yours { .. } => None,
                };
                PlanOutcomeData {
                    key: key.clone(),
                    outcome: outcome.map(|outcome| OutcomeData::compared_to(outcome, no_fees)),
                    track,
                    warning: plan
                        .first_deposit_warning(ils(scenario.first_deposit), lang())
                        .or_else(|| fees_warning().map(str::to_owned)),
                    may_cost_more,
                    note: plan
                        .track_note(security, exchange, track, lang())
                        .or_else(|| {
                            plan.standing_order_note(
                                security,
                                exchange,
                                scenario.buy_every_months,
                                lang(),
                            )
                            .map(str::to_owned)
                        }),
                    tax_note: outcome.and_then(|_| plan.tax_note(&scenario, lang())),
                    not_offered: compared
                        .outcome
                        .as_ref()
                        .err()
                        .map(|&reason| plan.why_not_offered(reason, &scenario, lang())),
                    why_not: compared.outcome.as_ref().err().copied().map(WhyNot::from),
                }
            })
            .collect(),
    })
}

/// Each plan's yearly cost at each of a range of deposits: the lines of the
/// chart by deposit, where the ranking flips at the crossings. It comes
/// back for [`around`].
#[derive(Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SweepData {
    /// Which deposit varies.
    pub swept: Swept,
    /// ₪, smallest first.
    pub amounts: Vec<f64>,
    /// In the order of the inputs' plans.
    pub plans: Vec<PlanSweepData>,
}

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanSweepData {
    pub key: PlanKey,
    /// The yearly cost at each amount, in percent; missing where the plan
    /// can't be used: it doesn't offer the security on that exchange, at
    /// any amount, or the deposits are over its ceiling.
    #[tsify(type = "(number | null)[]")]
    pub costs: Vec<Option<f64>>,
}

/// Runs the chosen plans over a range of the `swept` deposit, with the other
/// inputs as they are. The swept deposit's own value is ignored, so it can
/// be left at zero.
#[wasm_bindgen]
pub fn sweep(inputs: Ts<Inputs>, swept: Ts<Swept>) -> Result<Ts<SweepData>, JsError> {
    Ok(sweep_plans(&inputs.to_rust()?, swept.to_rust()?)
        .map_err(invalid)?
        .into_ts()?)
}

pub fn sweep_plans(inputs: &Inputs, swept: Swept) -> Result<SweepData, InvalidInputs> {
    let brokers = all_brokers();
    let plans = chosen_plans(inputs, &brokers)?;
    let scenario = scenario(inputs, &plans)?;
    let rates = exchange_rates(inputs)?;
    let sweep = simulation::sweep(&plans, &scenario, &rates, swept);
    Ok(SweepData {
        swept: sweep.swept,
        amounts: sweep.amounts.into_iter().map(number).collect(),
        plans: inputs
            .plans
            .iter()
            .zip(sweep.costs)
            .map(|(key, costs)| PlanSweepData {
                key: key.clone(),
                costs: costs
                    .into_iter()
                    .map(|cost| cost.map(|cost| number(cost.0)))
                    .collect(),
            })
            .collect(),
    })
}

/// What [`around`] works from: the sweep, and the comparison at the user's
/// own value of the deposit the sweep varies.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AroundInputs {
    pub sweep: SweepData,
    /// ₪: the user's own value of the swept deposit.
    pub deposit: f64,
    /// Each plan's yearly cost there, best first, as the comparison ranks
    /// them.
    pub costs: Vec<PlanCost>,
}

#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlanCost {
    pub key: PlanKey,
    /// In percent; missing if the plan doesn't offer the security there.
    #[tsify(type = "number | null")]
    pub cost: Option<f64>,
}

/// Where the cheapest plan at the user's deposit stops being the cheapest,
/// as the swept deposit moves away from it. All amounts in ₪.
#[derive(Debug, PartialEq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AroundData {
    /// The least deposit looked at, the sweep's or the user's own.
    pub from: f64,
    /// The most.
    pub to: f64,
    /// The nearest crossing below the user's deposit, past which another
    /// plan is cheaper; none if the plan stays the cheapest down to `from`.
    pub below: Option<CrossingData>,
    /// The nearest above it, up to `to`.
    pub above: Option<CrossingData>,
}

#[derive(Debug, PartialEq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct CrossingData {
    /// To two significant digits.
    pub amount: f64,
    /// The plan that's cheaper past it.
    pub key: PlanKey,
}

/// Where the cheapest plan at the user's own deposit stops being the
/// cheapest ([`simulation::Sweep::around`]). Nothing if the sweep and the
/// comparison disagree about which plan that is, as a sweep for other
/// inputs might.
#[wasm_bindgen]
pub fn around(inputs: Ts<AroundInputs>) -> Result<Option<Ts<AroundData>>, JsError> {
    Ok(around_of(&inputs.to_rust()?)
        .as_ref()
        .map(AroundData::into_ts)
        .transpose()?)
}

#[must_use]
pub fn around_of(inputs: &AroundInputs) -> Option<AroundData> {
    let AroundInputs {
        sweep,
        deposit,
        costs,
    } = inputs;
    // The comparison's best is the first plan it has a cost for.
    let best_key = &costs.iter().find(|plan| plan.cost.is_some())?.key;
    let best = sweep.plans.iter().position(|plan| &plan.key == best_key)?;
    let cost_at_deposit = |key: &PlanKey| costs.iter().find(|plan| &plan.key == key)?.cost;
    let swept = simulation::Sweep {
        swept: sweep.swept,
        amounts: sweep
            .amounts
            .iter()
            .map(|&amount| decimal(amount))
            .collect::<Option<_>>()?,
        costs: sweep
            .plans
            .iter()
            .map(|plan| {
                plan.costs
                    .iter()
                    .map(|&cost| cost.and_then(decimal).map(Percent))
                    .collect()
            })
            .collect(),
    };
    let at_deposit: Vec<Option<Percent>> = sweep
        .plans
        .iter()
        .map(|plan| cost_at_deposit(&plan.key).and_then(decimal).map(Percent))
        .collect();
    let around = swept.around(best, decimal(*deposit)?, &at_deposit)?;
    let crossing = |crossing: simulation::Crossing| CrossingData {
        amount: number(crossing.amount),
        key: sweep.plans[crossing.plan].key.clone(),
    };
    Some(AroundData {
        from: number(around.from),
        to: number(around.to),
        below: around.below.map(crossing),
        above: around.above.map(crossing),
    })
}

/// A ready-made investing pattern, and the inputs it sets.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ExampleData {
    /// "Saving from your salary"
    pub name: String,
    /// The pattern in words, for its tip.
    pub explanation: String,
    pub security: Security,
    pub exchange: Exchange,
    pub first_deposit: f64,
    pub monthly_deposit: f64,
    pub yearly_return_percent: f64,
    pub years: u32,
    /// Whether the money is taken as a monthly pension where a plan pays
    /// one, rather than all at once.
    pub as_pension: bool,
    /// The saver's age today: a pension opens from an age.
    pub age: u32,
}

/// The examples, in the order to offer them.
#[wasm_bindgen]
pub fn examples() -> Result<Vec<Ts<ExampleData>>, JsError> {
    examples::all()
        .into_iter()
        .map(|example| {
            let scenario = example.scenario;
            Ok(ExampleData {
                name: example.name[lang()].to_owned(),
                explanation: example.explanation[lang()].to_owned(),
                security: scenario.security,
                exchange: scenario.exchange,
                first_deposit: number(scenario.first_deposit),
                monthly_deposit: number(scenario.monthly_deposit),
                yearly_return_percent: number(scenario.yearly_return.0),
                years: scenario.years,
                as_pension: scenario.withdrawal == Withdrawal::Pension,
                age: scenario.age,
            }
            .into_ts()?)
        })
        .collect()
}

/// How much prices are taken to rise a year unless the user says otherwise,
/// in percent: what the inflation field starts with.
#[wasm_bindgen(js_name = usualInflationPercent)]
#[must_use]
pub fn usual_inflation_percent() -> f64 {
    number(simulation::USUAL_INFLATION.0)
}

/// The share price's currency ("$"), if the share price matters to what
/// `inputs` compares; `null` if its field isn't needed.
#[wasm_bindgen(js_name = sharePriceSymbol)]
pub fn share_price_symbol(inputs: Ts<Inputs>) -> Result<Option<String>, JsError> {
    let inputs = inputs.to_rust()?;
    let brokers = all_brokers();
    let plans = chosen_plans(&inputs, &brokers).map_err(invalid)?;
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
#[serde(rename = "Named")]
pub struct NamedChoice<T> {
    pub value: T,
    pub name: String,
}

/// A period, as the editor names it: "quarter", "a quarter" (after a
/// minimum) and "quarterly" (as in "charged quarterly").
#[derive(Debug, Serialize, Tsify)]
pub struct PeriodName {
    pub value: Period,
    pub name: String,
    pub each: String,
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
    change(&mut plan).map_err(|error| shown(error.text(lang())))?;
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
    Ok(PlanInfo::new(&plan.to_rust()?.0, lang()).into_ts()?)
}

/// Like [`fees_for`], for one of the user's own plans.
#[wasm_bindgen(js_name = feesForPlan)]
pub fn fees_for_plan(plan: Ts<PlanData>, purchase: Ts<Purchase>) -> Result<Ts<FeesFor>, JsError> {
    let (buying, rates) = purchase.to_rust()?.buying();
    Ok(plan
        .to_rust()?
        .0
        .describe_fees_for(buying, None, &[], &rates, lang())
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
        .simple_fees(
            security.to_rust()?,
            exchange.to_rust()?,
            original.as_ref(),
            lang(),
        )
        .into_ts()?)
}

/// The editor's full view: every row, compared to `original`'s.
#[wasm_bindgen(js_name = priceList)]
pub fn price_list(
    plan: Ts<PlanData>,
    original: Option<Ts<PlanData>>,
) -> Result<Ts<PriceList>, JsError> {
    let original = self::original(original)?;
    Ok(plan
        .to_rust()?
        .0
        .price_list(original.as_ref(), lang())
        .into_ts()?)
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

/// Sets a fund's or a policy's fee.
#[wasm_bindgen(js_name = setManagement)]
pub fn set_management(
    plan: Ts<PlanData>,
    fields: Ts<ManagementFields>,
) -> Result<Ts<PlanData>, JsError> {
    let fields = fields.to_rust()?;
    changed(plan, |plan| plan.set_management(&fields))
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
pub fn price_kinds() -> Result<Vec<Ts<NamedChoice<PriceKind>>>, JsError> {
    PriceKind::iter()
        .map(|value| {
            let name = value.name(lang()).to_owned();
            Ok(NamedChoice { value, name }.into_ts()?)
        })
        .collect()
}

/// The periods custody is quoted and charged per, for the editor's dropdowns.
#[wasm_bindgen]
pub fn periods() -> Result<Vec<Ts<PeriodName>>, JsError> {
    Period::iter()
        .map(|value| {
            let (name, each, adverb) = (
                value.name(lang()).to_owned(),
                value.each(lang()).to_owned(),
                value.adverb(lang()).to_owned(),
            );
            Ok(PeriodName {
                value,
                name,
                each,
                adverb,
            }
            .into_ts()?)
        })
        .collect()
}

// ─────────────────────────── The short term ───────────────────────────

/// A kind of place to keep money for the short term, and each one of it.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KindInfo {
    pub name: String,
    pub english_name: String,
    pub description: String,
    /// "Rates given, by the data of 08/2026"
    pub data_of: String,
    /// "Checked 01/10/2026"
    pub checked: String,
    pub source: Source,
    /// The flag its caveats raise for every place of the kind, when they say
    /// the numbers may be too good: "August's rates, before a cut". Said
    /// once, not under each place.
    pub may_cost_more: Option<String>,
    pub places: Vec<PlaceInfo>,
}

/// One place to keep the money.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInfo {
    pub name: String,
    /// The English name, which links name listed places by.
    pub english_name: String,
    pub short_name: String,
    pub description: String,
    /// A fund's: "The Bank of Israel's rate, less 0.169% a year".
    pub pays: Option<String>,
    /// A fund's fee, in percent a year.
    pub fee_percent: Option<f64>,
    /// A deposit's rate for each term, the shortest first; empty for a fund.
    pub rates: Vec<TermRate>,
    /// "15% of all the interest"
    pub tax: String,
    pub liquidity: Liquidity,
    /// "Any day"
    pub liquidity_name: String,
    /// Its caveats and its kind's, grouped by how sure.
    pub caveats: Vec<CaveatGroup>,
    /// The flag under its row when one of its own caveats says the numbers
    /// may be too good: "A new fund: its fee may rise". Its kind's flag is
    /// the kind's.
    pub may_cost_more: Option<String>,
    /// Every page its numbers rest on: its kind's source, then its caveats'.
    pub sources: Vec<Source>,
    /// Ticked when the calculator opens.
    pub compared_at_first: bool,
}

/// A deposit's rate for one term.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TermRate {
    /// "6 months to a year"
    pub term: String,
    /// The longest deposit the term takes, in months.
    pub longest_months: u32,
    /// Percent a year; none where the Bank of Israel published none.
    pub rate: Option<f64>,
}

impl PlaceInfo {
    fn new(place: &Place, kind: &short_term::Kind, lang: Lang) -> Self {
        let rates = match place.pays {
            Pays::Fixed(rates) => Term::iter()
                .map(|term| TermRate {
                    term: term.name(lang).to_owned(),
                    longest_months: term.longest(),
                    rate: rates.of(term).map(|rate| number(rate.0)),
                })
                .collect(),
            Pays::TheRateLess(_) => vec![],
        };
        let caveats: Vec<&broker_fees::Caveat> =
            place.caveats.iter().chain(&kind.caveats).collect();
        PlaceInfo {
            name: place.name[lang].to_owned(),
            english_name: place.name.en.to_string(),
            short_name: place.short_name[lang].to_owned(),
            description: place.description[lang].to_owned(),
            pays: place.pays_text(lang),
            fee_percent: match place.pays {
                Pays::TheRateLess(fee) => Some(number(fee.0)),
                Pays::Fixed(_) => None,
            },
            rates,
            tax: place.pays.tax().name(lang).to_owned(),
            liquidity: place.pays.liquidity(),
            liquidity_name: place.pays.liquidity().name(lang).to_owned(),
            caveats: describe::caveat_groups(&caveats, lang),
            may_cost_more: may_cost_more(&place.caveats, lang),
            sources: place.sources(kind, lang),
            compared_at_first: place.compared_at_first,
        }
    }
}

/// The first flag among `caveats` saying the numbers may be too good.
fn may_cost_more(caveats: &[broker_fees::Caveat], lang: Lang) -> Option<String> {
    caveats
        .iter()
        .find_map(broker_fees::Caveat::may_cost_more_summary)
        .map(|summary| summary[lang].to_owned())
}

/// The kinds of place, the funds first, each with every one of it.
#[wasm_bindgen(js_name = shortTermKinds)]
pub fn short_term_kinds() -> Result<Vec<Ts<KindInfo>>, JsError> {
    let lang = lang();
    short_term::kinds()
        .iter()
        .map(|kind| {
            Ok(KindInfo {
                name: kind.name[lang].to_owned(),
                english_name: kind.name.en.to_string(),
                description: kind.description[lang].to_owned(),
                data_of: kind.data_of_text(lang),
                checked: kind.checked_on_text(lang),
                source: Source::new(&kind.source, lang),
                may_cost_more: may_cost_more(&kind.caveats, lang),
                places: kind
                    .places
                    .iter()
                    .map(|place| PlaceInfo::new(place, kind, lang))
                    .collect(),
            }
            .into_ts()?)
        })
        .collect()
}

/// Your own deposit, before a rate is typed: its name, description, tax
/// and lock. The deposits' caveats and sources are about the banks' rates,
/// not yours.
#[wasm_bindgen(js_name = yourDepositInfo)]
pub fn your_deposit_info() -> Result<Ts<PlaceInfo>, JsError> {
    let place = short_term::your_deposit(Percent(Decimal::ZERO));
    let kind = short_term::Kind {
        places: vec![],
        caveats: vec![],
        ..short_term::deposits::kind()
    };
    Ok(PlaceInfo {
        rates: vec![],
        sources: vec![],
        ..PlaceInfo::new(&place, &kind, lang())
    }
    .into_ts()?)
}

/// A place to compare: a listed one, by its kind and its position there,
/// or one of the saver's own deposits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlaceKey {
    Listed { group: usize, place: usize },
    Yours { id: String },
}

/// One of the saver's own deposits, with the id the web app gave it.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct YourDepositInput {
    pub id: String,
    /// 3.9 means 3.9% a year.
    #[tsify(type = "number | null")]
    pub rate_percent: Option<f64>,
}

/// What the saver keeps, for how long, what they expect, and where to
/// compare.
#[derive(Debug, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ShortTermInputs {
    // The numbers are null while their field is empty.
    /// ₪, at the start.
    #[tsify(type = "number | null")]
    pub first_deposit: Option<f64>,
    /// ₪, every month.
    #[tsify(type = "number | null")]
    pub monthly_deposit: Option<f64>,
    pub months: u32,
    /// The Bank of Israel's rate on average over the months: 3.25 means 3.25%.
    #[tsify(type = "number | null")]
    pub rate_percent: Option<f64>,
    /// How much prices rise a year: 2 means 2%.
    #[tsify(type = "number | null")]
    pub inflation_percent: Option<f64>,
    pub places: Vec<PlaceKey>,
    /// The saver's own deposits that `places` refers to.
    #[serde(default)]
    pub your_deposits: Vec<YourDepositInput>,
}

/// The places compared, best first, and what the money would come to at the
/// Bank of Israel's rate.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ShortTermComparisonData {
    /// Every shekel put in.
    pub deposited: f64,
    /// What the money would be worth after each month at the Bank of
    /// Israel's rate, with nothing kept and no tax.
    pub at_the_rate: Vec<f64>,
    /// The deposits' term for these months: "6 months to a year".
    pub term: String,
    /// Each compared place, most left after tax first, the ones not offered
    /// last.
    pub places: Vec<ShortTermRow>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ShortTermRow {
    pub key: PlaceKey,
    /// Missing when the money can't be kept there.
    pub outcome: Option<ShortTermOutcomeData>,
    /// Why not, in a few words: "Takes one sum".
    pub not_offered: Option<String>,
    /// And in full, for its "?".
    pub not_offered_reason: Option<String>,
}

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ShortTermOutcomeData {
    /// At the start and after each month, before tax.
    pub value_by_month: Vec<f64>,
    pub earned: f64,
    pub tax: f64,
    pub after_tax: f64,
    /// What the place kept of what the rate would make; negative where a
    /// bank pays more.
    pub kept: f64,
    pub yearly_cost_percent: f64,
    pub yearly_after_tax_percent: f64,
}

/// Compares the places `inputs` names for its money.
#[wasm_bindgen(js_name = compareShortTerm)]
pub fn compare_short_term(
    inputs: Ts<ShortTermInputs>,
) -> Result<Ts<ShortTermComparisonData>, JsError> {
    let inputs = inputs.to_rust()?;
    Ok(compare_places(&inputs).map_err(invalid)?.into_ts()?)
}

/// What `compare_short_term` does, without the JavaScript around it.
///
/// # Errors
///
/// An empty or wrong field, or a place that isn't there.
pub fn compare_places(inputs: &ShortTermInputs) -> Result<ShortTermComparisonData, InvalidInputs> {
    let lang = lang();
    let amount = |value: Option<f64>, field| {
        let amount = value
            .map_or(Some(Decimal::ZERO), decimal)
            .ok_or(InvalidInputs::Missing(field))?;
        if amount < Decimal::ZERO {
            return Err(InvalidInputs::Negative(field));
        }
        Ok(amount)
    };
    let scenario = short_term::Scenario {
        first_deposit: amount(inputs.first_deposit, Field::FirstDeposit)?,
        monthly_deposit: amount(inputs.monthly_deposit, Field::MonthlyDeposit)?,
        months: inputs.months,
        rate: Percent(filled_in(inputs.rate_percent, Field::Rate)?),
        inflation: Percent(filled_in(inputs.inflation_percent, Field::Inflation)?),
    };
    scenario.check()?;
    let kinds = short_term::kinds();
    let places = inputs
        .places
        .iter()
        .map(|key| match key {
            PlaceKey::Listed { group, place } => kinds
                .get(*group)
                .and_then(|kind| kind.places.get(*place))
                .cloned()
                .ok_or(InvalidInputs::Other(Problem::NoSuchPlan)),
            PlaceKey::Yours { id } => {
                let yours = inputs
                    .your_deposits
                    .iter()
                    .find(|yours| &yours.id == id)
                    .ok_or(InvalidInputs::Other(Problem::NoSuchPlan))?;
                let rate = filled_in(yours.rate_percent, Field::YourRate)?;
                if rate < Decimal::ZERO {
                    return Err(InvalidInputs::Negative(Field::YourRate));
                }
                Ok(short_term::your_deposit(Percent(rate)))
            }
        })
        .collect::<Result<Vec<Place>, InvalidInputs>>()?;
    let refs: Vec<&Place> = places.iter().collect();
    let comparison = short_term::compare(&refs, &scenario);
    Ok(ShortTermComparisonData {
        deposited: number(scenario.deposited()),
        at_the_rate: comparison.at_the_rate.iter().copied().map(number).collect(),
        term: Term::for_months(scenario.months)
            .expect("a checked scenario")
            .name(lang)
            .to_owned(),
        places: comparison
            .places
            .into_iter()
            .map(|compared| {
                let key = inputs.places[compared.index].clone();
                match compared.outcome {
                    Ok(outcome) => ShortTermRow {
                        key,
                        outcome: Some(ShortTermOutcomeData {
                            value_by_month: outcome
                                .value_by_month
                                .iter()
                                .copied()
                                .map(number)
                                .collect(),
                            earned: number(outcome.earned),
                            tax: number(outcome.tax),
                            after_tax: number(outcome.after_tax),
                            kept: number(outcome.kept),
                            yearly_cost_percent: number(outcome.yearly_cost.0),
                            yearly_after_tax_percent: number(outcome.yearly_after_tax.0),
                        }),
                        not_offered: None,
                        not_offered_reason: None,
                    },
                    Err(why) => ShortTermRow {
                        key,
                        outcome: None,
                        not_offered: Some(why.short(lang).to_owned()),
                        not_offered_reason: Some(why.reason(lang)),
                    },
                }
            })
            .collect(),
    })
}

/// A short-term pattern for its button.
#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ShortTermExampleData {
    pub name: String,
    pub explanation: String,
    pub first_deposit: f64,
    pub monthly_deposit: f64,
    pub months: u32,
}

/// The short-term examples, in the order to offer them.
#[wasm_bindgen(js_name = shortTermExamples)]
pub fn short_term_examples() -> Result<Vec<Ts<ShortTermExampleData>>, JsError> {
    examples::short_term()
        .into_iter()
        .map(|example| {
            Ok(ShortTermExampleData {
                name: example.name[lang()].to_owned(),
                explanation: example.explanation[lang()].to_owned(),
                first_deposit: number(example.first_deposit),
                monthly_deposit: number(example.monthly_deposit),
                months: example.months,
            }
            .into_ts()?)
        })
        .collect()
}

/// The short-term calculator's page on its numbers.
#[wasm_bindgen(js_name = aboutShortTerm)]
pub fn about_short_term() -> Result<Ts<About>, JsError> {
    Ok(describe::about_short_term(lang()).into_ts()?)
}

/// The Bank of Israel's rate when the numbers were checked, in percent: what
/// the expected rate starts at.
#[wasm_bindgen(js_name = todaysRatePercent)]
#[must_use]
pub fn todays_rate_percent() -> f64 {
    number(short_term::TODAYS_RATE.0)
}

/// Which of a deposit's rates `months` falls in: its position in
/// `PlaceInfo::rates`. None past the longest term.
#[wasm_bindgen(js_name = termFor)]
#[must_use]
pub fn term_for(months: u32) -> Option<usize> {
    Term::for_months(months).map(|term| term as usize)
}

/// The longest the money can be kept, in months.
#[wasm_bindgen(js_name = shortTermLongest)]
#[must_use]
pub fn short_term_longest() -> u32 {
    short_term::LONGEST
}

fn number(value: Decimal) -> f64 {
    value.to_f64().unwrap_or_default()
}

/// Inputs come as numbers; four decimal places are plenty for money and rates.
/// `None` for NaN and infinities.
/// A field's number, or which field to fill in.
fn filled_in(value: Option<f64>, what: Field) -> Result<Decimal, InvalidInputs> {
    value.and_then(decimal).ok_or(InvalidInputs::Missing(what))
}

fn decimal(value: f64) -> Option<Decimal> {
    Decimal::try_from(value).ok().map(|value| value.round_dp(4))
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "the numbers compared are made the same way, or are zero"
)]
mod tests {
    use super::*;
    use broker_fees::{funds, tariffs};

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
            deposit_growth_percent: Some(0.0),
            inflation_percent: Some(0.0),
            in_todays_money: false,
            as_pension: false,
            age: None,
            sell_at_end: true,
            ils_per_usd: Some(3.7),
            ils_per_eur: Some(4.3),
            plans: vec![
                PlanKey::Listed { broker: 0, plan: 0 },
                PlanKey::Listed { broker: 1, plan: 3 },
            ],
            your_plans: vec![],
        }
    }

    /// The sweep and the comparison are matched by plan, whatever their
    /// order: the comparison's first plan with a cost is the best, and a
    /// best that isn't the cheapest in the sweep has nothing to say.
    #[test]
    fn around_matches_the_sweep_and_the_comparison_by_plan() {
        let comparison = compare_plans(&inputs()).unwrap();
        let sweep = || {
            let inputs = Inputs {
                monthly_deposit: Some(0.0),
                ..inputs()
            };
            sweep_plans(&inputs, Swept::Monthly).unwrap()
        };
        let costs = |plans: &mut dyn Iterator<Item = &PlanOutcomeData>| {
            plans
                .map(|plan| PlanCost {
                    key: plan.key.clone(),
                    cost: plan
                        .outcome
                        .as_ref()
                        .map(|outcome| outcome.yearly_cost_percent),
                })
                .collect()
        };
        let around = around_of(&AroundInputs {
            sweep: sweep(),
            deposit: 2_000.0,
            costs: costs(&mut comparison.plans.iter()),
        })
        .unwrap();
        assert_eq!((around.from, around.to), (100.0, 32_000.0));
        let keys = crossing_keys(&around);
        assert!(
            keys.iter().all(|key| inputs().plans.contains(key)),
            "{around:?}"
        );

        let upside_down = around_of(&AroundInputs {
            sweep: sweep(),
            deposit: 2_000.0,
            costs: costs(&mut comparison.plans.iter().rev()),
        });
        assert_eq!(upside_down, None);
    }

    /// The plans named past the crossings.
    fn crossing_keys(around: &AroundData) -> Vec<&PlanKey> {
        [&around.below, &around.above]
            .into_iter()
            .flatten()
            .map(|crossing| &crossing.key)
            .collect()
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
            Some(Negative(Field::MonthlyDeposit))
        );
        assert!(matches!(
            with(|i| i.ils_per_usd = Some(0.0)),
            Some(Other(_))
        ));
        assert_eq!(
            with(|i| i.share_price = Some(f64::NAN)),
            Some(Missing(Field::SharePrice))
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
        let name = &listed(0, 0).unwrap().tracks[track].name.en;
        assert_eq!(
            altshuler.note,
            Some(format!("US track: {name}, the cheapest for you"))
        );
        // Its fees count keeping the account: custody and the handling fee.
        let fees = &altshuler.outcome.as_ref().unwrap().fees;
        assert!(fees.holding > 0.0);
        let parts = fees.purchases + fees.conversions + fees.holding + fees.selling;
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
        let leumi = BrokerInfo::new(&tariffs::leumi(), Lang::En);
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

    /// The provident fund for investment's average plan: the first of the
    /// funds, which come after the brokers.
    fn the_fund() -> PlanKey {
        PlanKey::Listed {
            broker: tariffs::all().len(),
            plan: 0,
        }
    }

    fn compared<'a>(comparison: &'a ComparisonData, key: &PlanKey) -> &'a PlanOutcomeData {
        comparison
            .plans
            .iter()
            .find(|plan| &plan.key == key)
            .unwrap()
    }

    /// A fund's management fee is counted where a broker's custody is, so
    /// comparing by what holding costs puts the two side by side, rather than
    /// the fund first at nothing.
    #[test]
    fn a_funds_fee_is_what_holding_costs() {
        let mut inputs = inputs();
        inputs.plans.push(the_fund());
        let comparison = compare_plans(&inputs).unwrap();
        let fees = &compared(&comparison, &the_fund())
            .outcome
            .as_ref()
            .unwrap()
            .fees;
        assert!(fees.holding > 0.0);
        assert!((fees.holding - fees.total).abs() < 0.01);
    }

    /// Taken at once, a fund pays the tax a broker does and ranks by its
    /// fee; as a pension at 60 or more it pays none, and ranks first. The
    /// age is asked for only with the pension.
    #[test]
    fn a_pension_from_60_is_not_taxed() {
        let mut inputs = inputs();
        inputs.inflation_percent = Some(2.0);
        let (fund, interactive) = (the_fund(), PlanKey::Listed { broker: 4, plan: 0 });
        inputs.plans = vec![fund.clone(), interactive.clone()];

        let at_once = compare_plans(&inputs).unwrap();
        assert_eq!(at_once.plans[0].key, interactive);
        for plan in &at_once.plans {
            let outcome = plan.outcome.as_ref().unwrap();
            assert!(outcome.tax > 0.0);
            // Each is rounded from a decimal on its own.
            let left = outcome.after_selling - outcome.tax;
            assert!((outcome.after_tax - left).abs() < 0.01);
        }

        inputs.as_pension = true;
        assert_eq!(
            compare_plans(&inputs).unwrap_err().text(Lang::En),
            "fill in your age"
        );
        inputs.age = Some(-1.0);
        assert_eq!(
            compare_plans(&inputs).unwrap_err().text(Lang::He),
            "אי אפשר להזין ערך שלילי בהגיל שלכם"
        );
        inputs.age = Some(40.0);
        let as_pension = compare_plans(&inputs).unwrap();
        assert_eq!(as_pension.plans[0].key, fund);
        let outcome = |comparison: &ComparisonData, key| {
            let plan = compared(comparison, key);
            let outcome = plan.outcome.as_ref().unwrap();
            (outcome.tax, outcome.after_tax, outcome.after_selling)
        };
        let (tax, after_tax, after_selling) = outcome(&as_pension, &fund);
        assert_eq!(tax, 0.0);
        assert_eq!(after_tax, after_selling);
        // Each row says what became of its tax; a broker's has nothing to say.
        assert_eq!(
            compared(&as_pension, &fund).tax_note.as_deref(),
            Some("No tax: taken as a pension from 60")
        );
        assert_eq!(
            compared(&at_once, &fund).tax_note.as_deref(),
            Some("Taxed like a broker; no tax as a pension from 60")
        );
        assert_eq!(compared(&as_pension, &interactive).tax_note, None);
        // A broker pays no pension: nothing changes for it.
        assert_eq!(
            outcome(&as_pension, &interactive),
            outcome(&at_once, &interactive)
        );
        // At 39 today, 59 at the end: too young.
        inputs.age = Some(39.9);
        let too_young = compare_plans(&inputs).unwrap();
        assert_eq!(outcome(&too_young, &fund), outcome(&at_once, &fund));
    }

    /// The inflation is always used for the tax; the amounts are in today's
    /// money only when asked.
    #[test]
    fn inflation_lowers_the_tax_and_todays_money_is_a_choice() {
        let mut inputs = inputs();
        let after_tax = |inputs: &Inputs| {
            let comparison = compare_plans(inputs).unwrap();
            let best = comparison.plans[0].outcome.as_ref().unwrap();
            (best.tax, best.after_selling, comparison.deposited)
        };
        let (tax_without, sold_without, deposited) = after_tax(&inputs);
        inputs.inflation_percent = Some(2.0);
        let (tax, sold, deposited_with) = after_tax(&inputs);
        assert!(tax < tax_without);
        assert_eq!(sold, sold_without);
        assert_eq!(deposited_with, deposited);
        inputs.in_todays_money = true;
        let (tax_today, sold_today, deposited_today) = after_tax(&inputs);
        assert!(sold_today < sold && tax_today < tax && deposited_today < deposited);
        assert_eq!(usual_inflation_percent(), 2.0);
    }

    /// Deposits over a fund's ceiling leave it without numbers, saying by
    /// how much; its line in the sweep ends where they pass it.
    #[test]
    fn a_fund_over_its_ceiling_says_so() {
        let mut inputs = inputs();
        inputs.plans.push(the_fund());
        let within = compare_plans(&inputs).unwrap();
        let fund = compared(&within, &the_fund());
        assert!(fund.outcome.is_some());
        assert_eq!((fund.not_offered.as_ref(), fund.why_not), (None, None));
        let sweep = sweep_plans(&inputs, Swept::Monthly).unwrap();
        let costs = |key: &PlanKey| {
            let plan = sweep.plans.iter().find(|plan| &plan.key == key).unwrap();
            plan.costs.clone()
        };
        let fund_costs = costs(&the_fund());
        assert!(fund_costs.first().unwrap().is_some());
        assert!(fund_costs.last().unwrap().is_none());
        assert!(costs(&inputs.plans[0]).iter().all(Option::is_some));

        inputs.first_deposit = Some(100_000.0);
        let over = compare_plans(&inputs).unwrap();
        let fund = over.plans.last().unwrap();
        assert_eq!(fund.key, the_fund());
        assert!(fund.outcome.is_none());
        assert_eq!(fund.why_not, Some(WhyNot::OverTheCeiling));
        assert_eq!(
            fund.not_offered.as_deref(),
            Some(
                "No more than ₪83,641 can be deposited in a year, and your first year's \
                 deposits come to ₪124,000"
            )
        );
        // Not selling the security isn't about a ceiling.
        inputs.exchange = Exchange::Europe;
        let europe = compare_plans(&inputs).unwrap();
        let altshuler = compared(&europe, &inputs.plans[0]);
        assert!(altshuler.not_offered.is_some());
        assert_eq!(altshuler.why_not, Some(WhyNot::NotSold));
        // A study fund is locked for six years.
        inputs.years = 5;
        inputs.plans = vec![PlanKey::Listed {
            broker: tariffs::all().len() + 1,
            plan: 0,
        }];
        let locked = &compare_plans(&inputs).unwrap().plans[0];
        assert_eq!(locked.why_not, Some(WhyNot::Locked));
        assert_eq!(
            locked.not_offered.as_deref(),
            Some(
                "Its money can be taken out on these terms only 6 years after the first \
                 deposit. Sooner, it's taxed as income"
            )
        );
    }

    /// A kind of fund is listed after the brokers, with its Hebrew name,
    /// its manager's fee in place of a price list, and the age its pension
    /// opens at.
    #[test]
    fn funds_are_listed_after_the_brokers() {
        let listed = all_brokers();
        let infos: Vec<BrokerInfo> = listed
            .iter()
            .map(|broker| BrokerInfo::new(broker, Lang::En))
            .collect();
        let brokers = tariffs::all().len();
        assert_eq!(infos.len(), brokers + funds::all().len());
        for broker in &infos[..brokers] {
            assert_ne!(broker.kind, BrokerKind::Funds);
            assert!(broker.compared_at_first);
            for plan in &broker.plans {
                assert!(plan.tariff.management.is_none() && plan.pension_from_age.is_none());
            }
        }
        let gemel = &infos[brokers];
        assert_eq!(gemel.kind, BrokerKind::Funds);
        assert_eq!(gemel.name, "Provident fund for investment");
        assert_eq!(gemel.tariff_date, "Fees paid, by the data of 08/2026");
        assert_eq!(gemel.checked, "Checked 30/09/2026");
        assert!(gemel.compared_at_first);
        assert_eq!(
            gemel.tax_rule.as_deref(),
            Some("No tax as a pension from 60; otherwise taxed like a broker")
        );
        assert!(
            infos[..brokers]
                .iter()
                .all(|broker| broker.tax_rule.is_none())
        );
        let most = gemel.plans[3].tariff.management.as_ref().unwrap();
        assert_eq!(most.of_balance.text, "1.05% of the balance a year");
        assert_eq!(most.of_deposits.text, "4% of each deposit");
        assert!(
            gemel
                .plans
                .iter()
                .all(|plan| plan.pension_from_age == Some(60))
        );
        let study = &infos[brokers + 1];
        assert_eq!(study.name, "Study fund");
        assert!(!study.compared_at_first);
        assert!(
            study
                .plans
                .iter()
                .all(|plan| plan.pension_from_age.is_none())
        );
        let policy = &infos[brokers + 2];
        assert_eq!(policy.name, "Savings policy");
        assert!(!policy.compared_at_first);
        assert!(
            policy
                .plans
                .iter()
                .all(|plan| plan.pension_from_age.is_none())
        );
        let average = policy.plans[0].tariff.management.as_ref().unwrap();
        assert!(average.of_deposits.nothing);
    }

    #[test]
    fn choices_have_display_names() {
        let choice = Choice::new(Security::IndexFund, Lang::En);
        assert_eq!(choice.name, "Index fund");
        let hebrew = Choice::new(Security::IndexFund, Lang::He);
        assert_eq!(hebrew.name, "קרן מחקה");
        assert_eq!(hebrew.english_name.as_deref(), Some("Index fund"));
        // An exchange is a place, known by its Hebrew name alone.
        assert_eq!(Choice::new(Exchange::Usa, Lang::He).english_name, None);
        assert_eq!(
            BrokerInfo::new(&tariffs::leumi(), Lang::He).tariff_date,
            "תעריפון מ-29/06/2026"
        );
    }

    fn short_term_inputs(places: Vec<PlaceKey>) -> ShortTermInputs {
        ShortTermInputs {
            first_deposit: Some(100_000.0),
            monthly_deposit: None,
            months: 12,
            rate_percent: Some(3.25),
            inflation_percent: Some(2.0),
            places,
            your_deposits: vec![],
        }
    }

    /// ₪100,000 for a year at the Bank of Jerusalem's 3.87% and in the
    /// average fund, as the core works them out: the deposit first, its
    /// term named; with money every month the deposit says why it's out.
    #[test]
    fn the_short_term_comparison_is_the_cores() {
        let deposits = short_term::kinds()[1]
            .places
            .iter()
            .position(|place| place.name.en == "Bank of Jerusalem")
            .unwrap();
        let jerusalem = PlaceKey::Listed {
            group: 1,
            place: deposits,
        };
        let average = PlaceKey::Listed { group: 0, place: 0 };
        let data =
            compare_places(&short_term_inputs(vec![average.clone(), jerusalem.clone()])).unwrap();
        assert_eq!(data.deposited, 100_000.0);
        assert_eq!(data.term, "6 months to a year");
        assert_eq!(data.places[0].key, jerusalem);
        assert_eq!(
            data.places[0].outcome.as_ref().unwrap().after_tax,
            103_289.5
        );
        assert_eq!(data.places[1].key, average);
        assert_eq!(data.at_the_rate.len(), 13);

        let monthly = ShortTermInputs {
            monthly_deposit: Some(1000.0),
            ..short_term_inputs(vec![jerusalem])
        };
        let data = compare_places(&monthly).unwrap();
        assert!(data.places[0].outcome.is_none());
        assert_eq!(data.places[0].not_offered.as_deref(), Some("Takes one sum"));
    }

    /// Your own deposit is compared at the rate you typed; an empty or
    /// negative rate names the field.
    #[test]
    fn your_deposit_is_compared_at_its_rate() {
        let yours = PlaceKey::Yours { id: "a".into() };
        let with_rate = |rate| ShortTermInputs {
            your_deposits: vec![YourDepositInput {
                id: "a".into(),
                rate_percent: rate,
            }],
            ..short_term_inputs(vec![yours.clone()])
        };
        let data = compare_places(&with_rate(Some(4.0))).unwrap();
        // 4% of ₪100,000, less 15%: ₪3,400.
        assert_eq!(
            data.places[0].outcome.as_ref().unwrap().after_tax,
            103_400.0
        );
        assert_eq!(
            compare_places(&with_rate(None)).unwrap_err(),
            InvalidInputs::Missing(Field::YourRate)
        );
        assert_eq!(
            compare_places(&with_rate(Some(-1.0))).unwrap_err(),
            InvalidInputs::Negative(Field::YourRate)
        );
    }

    #[test]
    fn short_term_inputs_say_whats_wrong() {
        let wrong = |change: fn(&mut ShortTermInputs)| {
            let mut inputs = short_term_inputs(vec![PlaceKey::Listed { group: 0, place: 0 }]);
            change(&mut inputs);
            compare_places(&inputs).unwrap_err()
        };
        assert_eq!(
            wrong(|inputs| inputs.rate_percent = None),
            InvalidInputs::Missing(Field::Rate)
        );
        assert_eq!(
            wrong(|inputs| inputs.first_deposit = Some(-5.0)),
            InvalidInputs::Negative(Field::FirstDeposit)
        );
        assert_eq!(
            wrong(|inputs| inputs.months = 0),
            InvalidInputs::ShortTerm(short_term::InvalidScenario::Months)
        );
        assert_eq!(
            wrong(|inputs| inputs.places = vec![PlaceKey::Listed { group: 5, place: 0 }]),
            InvalidInputs::Other(Problem::NoSuchPlan)
        );
        assert_eq!(
            wrong(|inputs| inputs.months = 0).text(Lang::He),
            "אפשר לחסוך לתקופה של חודש עד 60 חודשים"
        );
    }

    /// A place as the page lists it: a deposit's rates by term, its tax and
    /// lock, and the flag its kind's caveat raises; a fund's line.
    #[test]
    fn places_are_described_for_the_page() {
        let kinds = short_term::kinds();
        let deposits = &kinds[1];
        let leumi = deposits
            .places
            .iter()
            .find(|place| place.name.en == "Bank Leumi")
            .unwrap();
        let info = PlaceInfo::new(leumi, deposits, Lang::He);
        assert_eq!(info.rates.len(), 7);
        assert_eq!(info.rates[3].term, "6 חודשים עד שנה");
        assert_eq!(info.rates[3].rate, Some(3.75));
        assert_eq!(info.tax, "15% מכל הריבית");
        assert_eq!(info.liquidity, Liquidity::AtTheEnd);
        // The deposits' flag is said once, by their kind; One Zero's own
        // flag is its own.
        assert_eq!(info.may_cost_more, None);
        assert_eq!(
            may_cost_more(&deposits.caveats, Lang::He).as_deref(),
            Some("ריביות אוגוסט, לפני הורדת ריבית")
        );
        let one_zero = deposits
            .places
            .iter()
            .find(|place| place.name.en == "One Zero")
            .unwrap();
        assert!(
            PlaceInfo::new(one_zero, deposits, Lang::He)
                .may_cost_more
                .is_some()
        );
        assert!(info.compared_at_first);
        let funds = &kinds[0];
        let average = PlaceInfo::new(&funds.places[0], funds, Lang::He);
        assert_eq!(
            average.pays.as_deref(),
            Some("ריבית בנק ישראל, פחות 0.169% בשנה")
        );
        assert_eq!(average.fee_percent, Some(0.169));
        assert_eq!(info.fee_percent, None);
        // A year falls in "6 months to a year", the fourth term.
        assert_eq!(term_for(12), Some(3));
        assert_eq!(info.rates[term_for(12).unwrap()].longest_months, 12);
        assert_eq!(term_for(61), None);
        assert!(average.rates.is_empty());
        assert_eq!(average.may_cost_more, None);
    }
}
