//! The user's own plans: a copy of a listed plan with the fees they think
//! they can get by asking, or a plan of a broker the app doesn't list.
//!
//! The app edits them in two views, both built here:
//! - simple: the fees for one security on one exchange ([`Plan::simple_fees`]);
//! - full: every row of the price list ([`Plan::price_list`]).
//!
//! Where two rows cover the same trade, the more specific one counts: the one
//! covering fewer securities and exchanges. Your plans keep their rows sorted
//! that way ([`Plan::sort_rows`]), so the first row that covers a trade, which
//! is what [`Plan::trade_row`] uses, is also the most specific. Listed tariffs
//! don't all follow this (an offer can be laid over a regular price list and
//! win by being on top), so a copy is rewritten once to keep its prices
//! ([`Plan::most_specific_first`]).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::describe;
use crate::{
    ConversionFee, Currency, CustodyFee, Exchange, Markup, Money, Percent, Period, Plan, Price,
    Security, TradeFee, empty_or_contains, iso,
};

// ─────────────────────────── Rows, by what they cover ───────────────────────────

/// A row of a price list, seen as what it covers: trade rows cover pairs of
/// security and exchange, custody rows cover exchanges.
trait Row: Clone {
    type Covered: Copy + PartialEq;

    /// Everything the row covers, one by one.
    fn covered(&self) -> Vec<Self::Covered>;

    /// Copies of the row that together cover exactly `covered`, or none if
    /// it's empty.
    fn covering(&self, covered: &[Self::Covered]) -> Vec<Self>;
}

impl Row for TradeFee {
    type Covered = (Security, Exchange);

    fn covered(&self) -> Vec<(Security, Exchange)> {
        Exchange::iter()
            .filter(|&exchange| empty_or_contains(&self.exchanges, exchange))
            .flat_map(|exchange| {
                Security::iter()
                    .filter(|&security| empty_or_contains(&self.securities, security))
                    .map(move |security| (security, exchange))
            })
            .collect()
    }

    fn covering(&self, covered: &[(Security, Exchange)]) -> Vec<Self> {
        // A row covers every security it lists on every exchange it lists, so
        // exchanges with the same securities share a row.
        let mut rows: Vec<TradeFee> = vec![];
        for exchange in Exchange::iter() {
            let securities: Vec<Security> = Security::iter()
                .filter(|&security| covered.contains(&(security, exchange)))
                .collect();
            if securities.is_empty() {
                continue;
            }
            match rows.iter_mut().find(|row| row.securities == securities) {
                Some(row) => row.exchanges.push(exchange),
                None => rows.push(TradeFee {
                    securities,
                    exchanges: vec![exchange],
                    price: self.price.clone(),
                }),
            }
        }
        for row in &mut rows {
            row.securities = all_as_empty(&row.securities);
            row.exchanges = all_as_empty(&row.exchanges);
        }
        rows
    }
}

impl Row for CustodyFee {
    type Covered = Exchange;

    fn covered(&self) -> Vec<Exchange> {
        Exchange::iter()
            .filter(|&exchange| empty_or_contains(&self.exchanges, exchange))
            .collect()
    }

    fn covering(&self, covered: &[Exchange]) -> Vec<Self> {
        if covered.is_empty() {
            return vec![];
        }
        vec![CustodyFee {
            exchanges: all_as_empty(covered),
            ..self.clone()
        }]
    }
}

/// `items` in their usual order, or empty if it has all of them, as tariffs
/// write "any security".
fn all_as_empty<T: IntoEnumIterator + PartialEq + Copy>(items: &[T]) -> Vec<T> {
    let sorted: Vec<T> = T::iter().filter(|item| items.contains(item)).collect();
    if sorted.len() == T::iter().count() {
        vec![]
    } else {
        sorted
    }
}

/// Rows rewritten so that the most specific row covering a trade gives the
/// same price as the first one did.
///
/// A row that loses a trade to an earlier, less specific row (an offer laid
/// over a regular price list) keeps only the trades it actually priced; a row
/// that only loses to more specific rows stays whole, since those still win
/// once the rows are sorted.
fn most_specific_first<R: Row>(rows: &[R]) -> Vec<R> {
    let winner = |covered: R::Covered| {
        rows.iter()
            .position(|row| row.covered().contains(&covered))
            .expect("some row covers it")
    };
    let mut rewritten = vec![];
    for (index, row) in rows.iter().enumerate() {
        let covered = row.covered();
        let loses_to_a_broader_row = covered.iter().any(|&each| {
            let winner = winner(each);
            winner != index && rows[winner].covered().len() >= covered.len()
        });
        if loses_to_a_broader_row {
            let won: Vec<_> = covered
                .into_iter()
                .filter(|&each| winner(each) == index)
                .collect();
            rewritten.extend(row.covering(&won));
        } else {
            rewritten.push(row.clone());
        }
    }
    sort_most_specific_first(&mut rewritten);
    rewritten
}

/// Fewest covered first. The sort is stable, so equally specific rows keep
/// their order.
fn sort_most_specific_first<R: Row>(rows: &mut [R]) {
    rows.sort_by_key(|row| row.covered().len());
}

/// The earlier rows that take some of what row `index` covers, and whether
/// they take all of it.
fn taken_by<R: Row>(rows: &[R], index: usize) -> (Vec<usize>, bool) {
    let mut by = vec![];
    let mut all = true;
    for covered in rows[index].covered() {
        match rows.iter().position(|row| row.covered().contains(&covered)) {
            Some(winner) if winner != index => {
                if !by.contains(&winner) {
                    by.push(winner);
                }
            }
            _ => all = false,
        }
    }
    (by, all)
}

// ─────────────────────────── The editor's fields ───────────────────────────

/// A number typed into one of the editor's fields: a percentage or an amount
/// of money. JavaScript gets it as a plain number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct Amount(
    #[serde(with = "rust_decimal::serde::float")]
    #[cfg_attr(feature = "ts", tsify(type = "number"))]
    pub Decimal,
);

/// How a trade's price is stated, as the editor offers it. Its `Display` is
/// its name: "of the trade", after the field's "%".
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter, strum::Display,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum PriceKind {
    #[strum(to_string = "of the trade")]
    Percent,
    #[strum(to_string = "per share")]
    PerShare,
    #[strum(to_string = "per order")]
    PerOrder,
}

/// A trade fee as the editor's fields. Empty fields are `None`: an empty
/// amount is free, an empty bound is no bound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct TradeFields {
    pub kind: PriceKind,
    /// The percentage, or the amount per share or per order.
    pub amount: Option<Amount>,
    /// Not for [`PriceKind::PerOrder`], which ignores them.
    pub min: Option<Amount>,
    pub max: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct CustodyFields {
    pub percent: Option<Amount>,
    /// The period `percent` is quoted per.
    pub per: Period,
    /// How often it's charged; `min` is per charge.
    pub billed: Period,
    pub min: Option<Amount>,
}

/// The conversion fee, without the markup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct ConversionFields {
    pub percent: Option<Amount>,
    pub min: Option<Amount>,
    pub max: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct MarkupFields {
    /// Empty means the broker doesn't publish it.
    pub percent: Option<Amount>,
}

/// One fee in the editor: its fields, the same in words, and the original
/// plan's if it's different.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Fee<F> {
    pub fields: F,
    /// The symbol amounts are in: "$".
    pub currency: String,
    /// "$4 per order"
    pub text: String,
    pub was: Option<Was<F>>,
}

/// What the original plan charges, to show and to go back to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Was<F> {
    pub fields: F,
    pub text: String,
}

impl<F> Fee<F> {
    fn new(fields: F, currency: &Currency, text: String) -> Self {
        Fee {
            fields,
            currency: currency.symbol.to_owned(),
            text,
            was: None,
        }
    }

    /// Adds what the original plan charges, if it's different.
    fn compared_to(self, original: Option<Fee<F>>) -> Self {
        let was = original
            .filter(|original| original.text != self.text)
            .map(|original| Was {
                fields: original.fields,
                text: original.text,
            });
        Fee { was, ..self }
    }
}

/// The fees for one security on one exchange: the editor's simple view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct SimpleFees {
    /// `None` if the plan doesn't offer it.
    pub trade: Option<Fee<TradeFields>>,
    pub custody: Fee<CustodyFields>,
    /// Only abroad: nothing is converted on Tel Aviv.
    pub conversion: Option<Fee<ConversionFields>>,
    pub markup: Option<Fee<MarkupFields>>,
}

/// Every row of a plan's price list: the editor's full view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PriceList {
    pub trading: Vec<PriceRow<TradeFields>>,
    pub custody: Vec<PriceRow<CustodyFields>>,
    pub conversion: Fee<ConversionFields>,
    pub markup: Fee<MarkupFields>,
    /// In ₪.
    pub min_first_deposit: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PriceRow<F> {
    /// Empty means all. Always empty for custody, which covers exchanges.
    pub securities: Vec<Security>,
    /// Empty means all.
    pub exchanges: Vec<Exchange>,
    /// "ETF on Tel Aviv"
    pub covers: String,
    /// "except Mutual fund on Tel Aviv (its own row above)"
    pub except: Option<String>,
    /// More specific rows take everything it covers.
    pub never_used: bool,
    pub fee: Fee<F>,
}

/// Why fields can't be saved, in words to show under them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidFee {
    #[error("Fees can't be negative")]
    Negative,
    #[error("The minimum is more than the maximum")]
    MinAboveMax,
    #[error("Give the plan a name")]
    NoName,
    #[error("There's no such row")]
    NoSuchRow,
}

// ─────────────────────────── Fields ↔ prices ───────────────────────────

/// A typed number, or 0 if the field is empty.
fn amount_or_zero(value: Option<Amount>) -> Result<Decimal, InvalidFee> {
    let amount = value.map_or(Decimal::ZERO, |Amount(amount)| amount);
    if amount.is_sign_negative() && !amount.is_zero() {
        return Err(InvalidFee::Negative);
    }
    Ok(amount)
}

/// A minimum and maximum in `currency`, checked.
fn bounds(
    min: Option<Amount>,
    max: Option<Amount>,
    currency: &'static Currency,
) -> Result<(Option<Money>, Option<Money>), InvalidFee> {
    let money = |value: Option<Amount>| -> Result<Option<Money>, InvalidFee> {
        value
            .map(|value| Ok(Money::from_decimal(amount_or_zero(Some(value))?, currency)))
            .transpose()
    };
    let (min, max) = (money(min)?, money(max)?);
    if let (Some(min), Some(max)) = (min, max)
        && min.amount() > max.amount()
    {
        return Err(InvalidFee::MinAboveMax);
    }
    Ok((min, max))
}

fn field(money: Option<Money>) -> Option<Amount> {
    money.map(|money| Amount(*money.amount()))
}

/// The currency tariffs state a row's amounts in: shekels on Tel Aviv, dollars
/// abroad (Israeli brokers quote European trades in dollars too).
fn usual_currency(exchanges: &[Exchange]) -> &'static Currency {
    if exchanges.is_empty() || exchanges.contains(&Exchange::Tlv) {
        iso::ILS
    } else {
        iso::USD
    }
}

impl Price {
    fn fields(&self) -> TradeFields {
        match self {
            Price::Percent { percent, min, max } => TradeFields {
                kind: PriceKind::Percent,
                amount: Some(Amount(percent.0)),
                min: field(*min),
                max: field(*max),
            },
            Price::PerShare {
                per_share,
                min,
                max,
            } => TradeFields {
                kind: PriceKind::PerShare,
                amount: field(Some(*per_share)),
                min: field(*min),
                max: field(*max),
            },
            Price::Flat(amount) => TradeFields {
                kind: PriceKind::PerOrder,
                amount: field(Some(*amount)),
                min: None,
                max: None,
            },
        }
    }

    /// The currency of its first amount, if it has any.
    fn currency(&self) -> Option<&'static Currency> {
        match self {
            Price::Percent { min, max, .. } => min.or(*max).map(|money| money.currency()),
            Price::PerShare { per_share, .. } => Some(per_share.currency()),
            Price::Flat(amount) => Some(amount.currency()),
        }
    }

    fn from_fields(fields: &TradeFields, currency: &'static Currency) -> Result<Self, InvalidFee> {
        let amount = amount_or_zero(fields.amount)?;
        let (min, max) = bounds(fields.min, fields.max, currency)?;
        Ok(match fields.kind {
            PriceKind::Percent => Price::Percent {
                percent: Percent(amount),
                min,
                max,
            },
            PriceKind::PerShare => Price::PerShare {
                per_share: Money::from_decimal(amount, currency),
                min,
                max,
            },
            PriceKind::PerOrder => Price::Flat(Money::from_decimal(amount, currency)),
        })
    }
}

impl TradeFee {
    fn currency(&self) -> &'static Currency {
        self.price
            .currency()
            .unwrap_or_else(|| usual_currency(&self.exchanges))
    }

    fn fee(&self) -> Fee<TradeFields> {
        Fee::new(self.price.fields(), self.currency(), self.price.to_string())
    }
}

impl CustodyFee {
    /// What a plan without a custody row charges: nothing.
    fn none(exchanges: Vec<Exchange>) -> Self {
        CustodyFee {
            exchanges,
            percent: Percent(Decimal::ZERO),
            per: Period::Year,
            billed: Period::Quarter,
            min: None,
        }
    }

    fn currency(&self) -> &'static Currency {
        self.min.map_or(iso::ILS, |min| min.currency())
    }

    fn fee(&self) -> Fee<CustodyFields> {
        let fields = CustodyFields {
            percent: Some(Amount(self.percent.0)),
            per: self.per,
            billed: self.billed,
            min: field(self.min),
        };
        Fee::new(fields, self.currency(), self.to_string())
    }

    fn set(&mut self, fields: &CustodyFields) -> Result<(), InvalidFee> {
        let (min, _) = bounds(fields.min, None, self.currency())?;
        self.percent = Percent(amount_or_zero(fields.percent)?);
        self.per = fields.per;
        self.billed = fields.billed;
        self.min = min;
        Ok(())
    }
}

impl ConversionFee {
    fn currency(&self) -> &'static Currency {
        self.min
            .or(self.max)
            .map_or(iso::USD, |money| money.currency())
    }

    fn fee(&self) -> Fee<ConversionFields> {
        let fields = ConversionFields {
            percent: Some(Amount(self.percent.0)),
            min: field(self.min),
            max: field(self.max),
        };
        Fee::new(fields, self.currency(), self.to_string())
    }

    fn markup_fee(&self) -> Fee<MarkupFields> {
        let percent = match self.markup {
            Markup::UpTo(percent) => Some(Amount(percent.0)),
            Markup::NotPublished => None,
        };
        Fee::new(
            MarkupFields { percent },
            self.currency(),
            self.markup.to_string(),
        )
    }
}

// ─────────────────────────── Your plans ───────────────────────────

impl Plan {
    /// A copy to change, named "Pepper, your deal". It charges exactly what
    /// the original does, with its rows most specific first. The caveats stay
    /// with the original: the user's fees are their own claim.
    #[must_use]
    pub fn copy_of(&self) -> Plan {
        Plan {
            name: format!("{}, your deal", self.name),
            description: String::new(),
            caveats: vec![],
            ..self.most_specific_first()
        }
    }

    /// A plan of the user's own, charging nothing until they fill it in.
    #[must_use]
    pub fn new_own(name: &str) -> Plan {
        // Two trade rows, so a fee set while buying in Tel Aviv doesn't
        // change the fee abroad.
        let free = |exchanges| TradeFee {
            securities: vec![],
            exchanges,
            price: Price::Percent {
                percent: Percent(Decimal::ZERO),
                min: None,
                max: None,
            },
        };
        Plan {
            name: name.to_owned(),
            description: String::new(),
            trading: vec![
                free(vec![Exchange::Tlv]),
                free(vec![Exchange::Usa, Exchange::Europe]),
            ],
            custody: vec![CustodyFee::none(vec![])],
            conversion: ConversionFee::default(),
            min_first_deposit: None,
            caveats: vec![],
        }
    }

    /// The same plan, its rows rewritten so that the most specific row
    /// covering a trade gives the price the first one did (see
    /// [`most_specific_first`]).
    #[must_use]
    pub fn most_specific_first(&self) -> Plan {
        Plan {
            trading: most_specific_first(&self.trading),
            custody: most_specific_first(&self.custody),
            ..self.clone()
        }
    }

    /// Sorts the rows most specific first, after they change.
    pub fn sort_rows(&mut self) {
        sort_most_specific_first(&mut self.trading);
        sort_most_specific_first(&mut self.custody);
    }

    pub fn rename(&mut self, name: &str) -> Result<(), InvalidFee> {
        if name.trim().is_empty() {
            return Err(InvalidFee::NoName);
        }
        name.clone_into(&mut self.name);
        Ok(())
    }

    /// The fees for `security` on `exchange`, compared to `original`'s.
    #[must_use]
    pub fn simple_fees(
        &self,
        security: Security,
        exchange: Exchange,
        original: Option<&Plan>,
    ) -> SimpleFees {
        let trade = |plan: &Plan| plan.trade_row(security, exchange).map(TradeFee::fee);
        let custody = |plan: &Plan| {
            plan.custody_row(exchange).map_or_else(
                || Fee {
                    text: "none".into(),
                    ..CustodyFee::none(vec![exchange]).fee()
                },
                CustodyFee::fee,
            )
        };
        let abroad = exchange != Exchange::Tlv;
        SimpleFees {
            trade: trade(self).map(|fee| fee.compared_to(original.and_then(trade))),
            custody: custody(self).compared_to(original.map(custody)),
            conversion: abroad.then(|| {
                self.conversion
                    .fee()
                    .compared_to(original.map(|plan| plan.conversion.fee()))
            }),
            markup: abroad.then(|| {
                self.conversion
                    .markup_fee()
                    .compared_to(original.map(|plan| plan.conversion.markup_fee()))
            }),
        }
    }

    /// Sets the trade fee for `security` on `exchange`: the row it uses, or a
    /// new row for exactly it if there's none (a copy of a plan that doesn't
    /// offer it).
    pub fn set_trade(
        &mut self,
        security: Security,
        exchange: Exchange,
        fields: &TradeFields,
    ) -> Result<(), InvalidFee> {
        let index = self
            .trading
            .iter()
            .position(|row| row.applies_to(security, exchange));
        if let Some(index) = index {
            let row = &mut self.trading[index];
            row.price = Price::from_fields(fields, row.currency())?;
        } else {
            let exchanges = vec![exchange];
            self.trading.push(TradeFee {
                price: Price::from_fields(fields, usual_currency(&exchanges))?,
                securities: vec![security],
                exchanges,
            });
            self.sort_rows();
        }
        Ok(())
    }

    /// Sets custody on `exchange`: the row it uses, or a new one for it.
    pub fn set_custody(
        &mut self,
        exchange: Exchange,
        fields: &CustodyFields,
    ) -> Result<(), InvalidFee> {
        let index = self
            .custody
            .iter()
            .position(|row| empty_or_contains(&row.exchanges, exchange));
        if let Some(index) = index {
            return self.custody[index].set(fields);
        }
        let mut row = CustodyFee::none(vec![exchange]);
        row.set(fields)?;
        self.custody.push(row);
        self.sort_rows();
        Ok(())
    }

    pub fn set_conversion(&mut self, fields: &ConversionFields) -> Result<(), InvalidFee> {
        let (min, max) = bounds(fields.min, fields.max, self.conversion.currency())?;
        self.conversion = ConversionFee {
            percent: Percent(amount_or_zero(fields.percent)?),
            min,
            max,
            ..self.conversion
        };
        Ok(())
    }

    pub fn set_markup(&mut self, fields: &MarkupFields) -> Result<(), InvalidFee> {
        self.conversion.markup = match fields.percent {
            Some(percent) => Markup::UpTo(Percent(amount_or_zero(Some(percent))?)),
            None => Markup::NotPublished,
        };
        Ok(())
    }

    /// Every row, and what the most specific rule does to each, compared to
    /// `original`'s row with the same coverage.
    #[must_use]
    pub fn price_list(&self, original: Option<&Plan>) -> PriceList {
        let original = original.map(Plan::most_specific_first);
        let trading = self
            .trading
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let (by, never_used) = taken_by(&self.trading, index);
                let was = original.as_ref().and_then(|original| {
                    original
                        .trading
                        .iter()
                        .find(|other| other.covered() == row.covered())
                        .map(TradeFee::fee)
                });
                PriceRow {
                    securities: row.securities.clone(),
                    exchanges: row.exchanges.clone(),
                    covers: row.coverage().to_string(),
                    except: describe::except(
                        &by.iter()
                            .map(|&other| self.trading[other].coverage().to_string())
                            .collect::<Vec<_>>(),
                    ),
                    never_used,
                    fee: row.fee().compared_to(was),
                }
            })
            .collect();
        let custody = self
            .custody
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let (by, never_used) = taken_by(&self.custody, index);
                let was = original.as_ref().and_then(|original| {
                    original
                        .custody
                        .iter()
                        .find(|other| other.covered() == row.covered())
                        .map(CustodyFee::fee)
                });
                PriceRow {
                    securities: vec![],
                    exchanges: row.exchanges.clone(),
                    covers: row.coverage().to_string(),
                    except: describe::except(
                        &by.iter()
                            .map(|&other| self.custody[other].coverage().to_string())
                            .collect::<Vec<_>>(),
                    ),
                    never_used,
                    fee: row.fee().compared_to(was),
                }
            })
            .collect();
        PriceList {
            trading,
            custody,
            conversion: self
                .conversion
                .fee()
                .compared_to(original.as_ref().map(|plan| plan.conversion.fee())),
            markup: self
                .conversion
                .markup_fee()
                .compared_to(original.as_ref().map(|plan| plan.conversion.markup_fee())),
            min_first_deposit: field(self.min_first_deposit),
        }
    }

    /// Adds a free trade row for exactly `security` on `exchange`.
    pub fn add_trade_row(&mut self, security: Security, exchange: Exchange) {
        self.trading.push(TradeFee {
            securities: vec![security],
            exchanges: vec![exchange],
            price: Price::Percent {
                percent: Percent(Decimal::ZERO),
                min: None,
                max: None,
            },
        });
        self.sort_rows();
    }

    /// Changes what trade row `index` covers and charges. Empty lists mean
    /// all.
    pub fn set_trade_row(
        &mut self,
        index: usize,
        securities: &[Security],
        exchanges: &[Exchange],
        fields: &TradeFields,
    ) -> Result<(), InvalidFee> {
        let row = self.trading.get(index).ok_or(InvalidFee::NoSuchRow)?;
        let exchanges = all_as_empty(exchanges);
        // A row that moves between Tel Aviv and abroad without amounts of its
        // own takes the currency of where it is now.
        let currency = row
            .price
            .currency()
            .unwrap_or_else(|| usual_currency(&exchanges));
        self.trading[index] = TradeFee {
            securities: all_as_empty(securities),
            exchanges,
            price: Price::from_fields(fields, currency)?,
        };
        self.sort_rows();
        Ok(())
    }

    pub fn remove_trade_row(&mut self, index: usize) -> Result<(), InvalidFee> {
        if index >= self.trading.len() {
            return Err(InvalidFee::NoSuchRow);
        }
        self.trading.remove(index);
        Ok(())
    }

    /// Adds a free custody row for exactly `exchange`.
    pub fn add_custody_row(&mut self, exchange: Exchange) {
        self.custody.push(CustodyFee::none(vec![exchange]));
        self.sort_rows();
    }

    pub fn set_custody_row(
        &mut self,
        index: usize,
        exchanges: &[Exchange],
        fields: &CustodyFields,
    ) -> Result<(), InvalidFee> {
        let row = self.custody.get_mut(index).ok_or(InvalidFee::NoSuchRow)?;
        row.set(fields)?;
        row.exchanges = all_as_empty(exchanges);
        self.sort_rows();
        Ok(())
    }

    pub fn remove_custody_row(&mut self, index: usize) -> Result<(), InvalidFee> {
        if index >= self.custody.len() {
            return Err(InvalidFee::NoSuchRow);
        }
        self.custody.remove(index);
        Ok(())
    }

    /// In ₪; empty for none.
    pub fn set_min_first_deposit(&mut self, amount: Option<Amount>) -> Result<(), InvalidFee> {
        self.min_first_deposit = bounds(amount, None, iso::ILS)?.0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::tariffs::{altshuler, leumi};
    use crate::{Broker, IntoEnumIterator};

    fn listed() -> Vec<Plan> {
        [altshuler(), leumi()]
            .into_iter()
            .flat_map(|broker: Broker| broker.plans)
            .collect()
    }

    fn named(name: &str) -> Plan {
        listed().into_iter().find(|plan| plan.name == name).unwrap()
    }

    /// What `plan` charges for each security on each exchange: the trade
    /// price and custody row it uses.
    fn prices(plan: &Plan) -> Vec<(Option<Price>, Option<CustodyFee>)> {
        Exchange::iter()
            .flat_map(|exchange| {
                Security::iter().map(move |security| {
                    let custody = plan.custody_row(exchange).map(|row| CustodyFee {
                        exchanges: vec![],
                        ..row.clone()
                    });
                    let price = plan.trade_row(security, exchange).map(|r| r.price.clone());
                    (price, custody)
                })
            })
            .collect()
    }

    #[test]
    fn rewritten_plans_charge_the_same() {
        for plan in listed() {
            let rewritten = plan.most_specific_first();
            assert_eq!(prices(&rewritten), prices(&plan), "{}", plan.name);
            let sizes: Vec<usize> = rewritten
                .trading
                .iter()
                .map(|r| r.covered().len())
                .collect();
            assert!(sizes.is_sorted(), "{}: {sizes:?}", plan.name);
        }
    }

    #[test]
    fn only_new_customers_is_rewritten() {
        for plan in listed() {
            if plan.name != "New customers" {
                assert_eq!(plan.most_specific_first(), plan, "{}", plan.name);
            }
        }
        // Its offer row is laid over the regular list: the regular ETF and
        // mutual fund rows on Tel Aviv never applied, and its stocks were
        // always the offer's.
        let rewritten = named("New customers").most_specific_first();
        let covers: Vec<String> = rewritten
            .trading
            .iter()
            .map(|row| row.coverage().to_string())
            .collect();
        assert_eq!(
            covers,
            [
                "Bond on Tel Aviv",
                "Stock, ETF on USA",
                "Stock, ETF, Mutual fund on Tel Aviv",
                "Bond, Mutual fund on USA, Europe",
            ]
        );
    }

    #[test]
    fn a_row_left_uneven_is_split() {
        // {Stock, Bond} on {Tel Aviv, USA} losing stocks on Tel Aviv to an
        // earlier row that's as broad leaves bonds on Tel Aviv and both on
        // the USA: two rows.
        use Exchange::*;
        use Security::*;
        let row = |securities: Vec<Security>, exchanges: Vec<Exchange>| TradeFee {
            securities,
            exchanges,
            price: Price::Flat(crate::ils(dec!(1))),
        };
        let rows = [
            row(vec![Stock, Etf], vec![Tlv, Europe]),
            row(vec![Stock, Bond], vec![Tlv, Usa]),
        ];
        let rewritten = most_specific_first(&rows);
        let covers: Vec<String> = rewritten.iter().map(|r| r.coverage().to_string()).collect();
        assert_eq!(
            covers,
            [
                "Bond on Tel Aviv",
                "Bond, Stock on USA",
                "Stock, ETF on Tel Aviv, Europe"
            ]
        );
    }

    #[test]
    fn a_copy_charges_what_the_original_does_without_its_caveats() {
        let pepper = named("Pepper");
        let copy = pepper.copy_of();
        assert_eq!(copy.name, "Pepper, your deal");
        assert_eq!(copy.caveats, []);
        assert_eq!(prices(&copy), prices(&pepper));
        let simple = copy.simple_fees(Security::Etf, Exchange::Usa, Some(&pepper));
        assert_eq!(simple.trade.unwrap().text, "$4 per order");
        assert!(simple.custody.was.is_none());
    }

    #[test]
    fn simple_fees_round_trip_and_show_the_original() {
        let pepper = named("Pepper");
        let mut copy = pepper.copy_of();
        let fields = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.1))),
            min: Some(Amount(dec!(2))),
            max: None,
        };
        copy.set_trade(Security::Etf, Exchange::Usa, &fields)
            .unwrap();
        let trade = copy
            .simple_fees(Security::Etf, Exchange::Usa, Some(&pepper))
            .trade
            .unwrap();
        assert_eq!(trade.fields, fields);
        assert_eq!(trade.text, "0.1%, min $2");
        assert_eq!(trade.was.unwrap().text, "$4 per order");

        let custody = CustodyFields {
            percent: Some(Amount(dec!(0.1))),
            per: Period::Quarter,
            billed: Period::Month,
            min: Some(Amount(dec!(20))),
        };
        copy.set_custody(Exchange::Usa, &custody).unwrap();
        let simple = copy.simple_fees(Security::Etf, Exchange::Usa, None);
        assert_eq!(simple.custody.fields, custody);
        assert_eq!(simple.custody.currency, "₪");

        copy.set_markup(&MarkupFields { percent: None }).unwrap();
        assert_eq!(copy.conversion.markup, Markup::NotPublished);
    }

    #[test]
    fn a_shared_row_changes_everywhere_it_covers() {
        // Leumi Online prices the USA and Europe in one row.
        let mut copy = named("Online").copy_of();
        let fields = TradeFields {
            kind: PriceKind::PerOrder,
            amount: Some(Amount(dec!(5))),
            min: None,
            max: None,
        };
        copy.set_trade(Security::Etf, Exchange::Usa, &fields)
            .unwrap();
        let europe = copy.trade_row(Security::Bond, Exchange::Europe).unwrap();
        assert_eq!(europe.price.to_string(), "$5 per order");
    }

    #[test]
    fn adding_a_fee_where_none_is_offered() {
        // Altshuler has no row for ETFs in Europe.
        let mut copy = named("US 1¢/share").copy_of();
        let simple = copy.simple_fees(Security::Etf, Exchange::Europe, None);
        assert!(simple.trade.is_none());
        let fields = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.2))),
            min: None,
            max: None,
        };
        copy.set_trade(Security::Etf, Exchange::Europe, &fields)
            .unwrap();
        let row = copy.trade_row(Security::Etf, Exchange::Europe).unwrap();
        assert_eq!(row.coverage().to_string(), "ETF on Europe");
    }

    #[test]
    fn the_more_specific_row_wins_and_the_other_says_so() {
        let mut plan = Plan::new_own("Mine");
        plan.add_trade_row(Security::Etf, Exchange::Usa);
        let fields = TradeFields {
            kind: PriceKind::PerOrder,
            amount: Some(Amount(dec!(1))),
            min: None,
            max: None,
        };
        plan.set_trade_row(0, &[Security::Etf], &[Exchange::Usa], &fields)
            .unwrap();
        assert_eq!(
            plan.trade_row(Security::Etf, Exchange::Usa)
                .unwrap()
                .price
                .to_string(),
            "$1 per order"
        );
        let list = plan.price_list(None);
        let abroad = &list.trading[2];
        assert_eq!(abroad.covers, "Anything on USA, Europe");
        assert_eq!(
            abroad.except.as_deref(),
            Some("except ETF on USA (its own row above)")
        );
        assert!(!abroad.never_used);

        // Covering everything abroad makes it a row nothing else can reach.
        plan.set_trade_row(0, &[], &[Exchange::Usa, Exchange::Europe], &fields)
            .unwrap();
        assert!(plan.price_list(None).trading[2].never_used);
    }

    #[test]
    fn bad_fields_are_errors() {
        let mut plan = Plan::new_own("Mine");
        let with = |min, max| TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.1))),
            min: Some(Amount(min)),
            max: Some(Amount(max)),
        };
        assert_eq!(
            plan.set_trade(Security::Etf, Exchange::Tlv, &with(dec!(-1), dec!(5))),
            Err(InvalidFee::Negative)
        );
        assert_eq!(
            plan.set_trade(Security::Etf, Exchange::Tlv, &with(dec!(9), dec!(5))),
            Err(InvalidFee::MinAboveMax)
        );
        assert_eq!(plan.rename("  "), Err(InvalidFee::NoName));
        assert_eq!(plan.remove_trade_row(9), Err(InvalidFee::NoSuchRow));
    }

    #[test]
    fn a_new_plan_is_free_until_filled_in() {
        let plan = Plan::new_own("Your plan");
        for (price, custody) in prices(&plan) {
            assert_eq!(price.unwrap().to_string(), "0%");
            assert!(custody.unwrap().percent.is_zero());
        }
        let simple = plan.simple_fees(Security::Etf, Exchange::Usa, None);
        assert_eq!(simple.trade.unwrap().currency, "$");
        assert_eq!(simple.markup.unwrap().text, "none");
    }

    #[test]
    fn plans_survive_json() {
        let plan = named("Online").copy_of();
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(serde_json::from_str::<Plan>(&json).unwrap(), plan);
        let fields = serde_json::to_string(&MarkupFields {
            percent: Some(Amount(dec!(0.7))),
        })
        .unwrap();
        assert_eq!(fields, r#"{"percent":0.7}"#);
    }
}
