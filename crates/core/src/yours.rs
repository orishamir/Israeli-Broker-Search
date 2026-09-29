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

use crate::describe::{self, PriceText, Priced};
use crate::simulation::whole_shares;
use crate::{
    ConversionFee, Currency, CustodyFee, Exchange, HandlingFee, Markup, Money, Percent, PercentFee,
    Period, Plan, Price, Security, TradeFee, empty_or_contains, iso,
};

// ─────────────────────────── Rows, by what they cover ───────────────────────────

/// A row of a price list, seen as what it covers: pairs of security and
/// exchange.
trait Row: Clone {
    /// Everything the row covers, one by one.
    fn covered(&self) -> Vec<(Security, Exchange)>;

    /// A copy of the row covering `securities` on `exchanges` instead.
    fn covering_only(&self, securities: Vec<Security>, exchanges: Vec<Exchange>) -> Self;

    /// Copies of the row that together cover exactly `covered`, or none if
    /// it's empty.
    fn covering(&self, covered: &[(Security, Exchange)]) -> Vec<Self> {
        // A row covers every security it lists on every exchange it lists, so
        // exchanges with the same securities share a row.
        let mut groups: Vec<(Vec<Security>, Vec<Exchange>)> = vec![];
        for exchange in Exchange::iter() {
            let securities: Vec<Security> = Security::iter()
                .filter(|&security| covered.contains(&(security, exchange)))
                .collect();
            if securities.is_empty() {
                continue;
            }
            match groups.iter_mut().find(|(each, _)| *each == securities) {
                Some((_, exchanges)) => exchanges.push(exchange),
                None => groups.push((securities, vec![exchange])),
            }
        }
        groups
            .into_iter()
            .map(|(securities, exchanges)| {
                self.covering_only(all_as_empty(&securities), all_as_empty(&exchanges))
            })
            .collect()
    }
}

/// Every pair of `securities` and `exchanges`, empty meaning all.
fn pairs(securities: &[Security], exchanges: &[Exchange]) -> Vec<(Security, Exchange)> {
    Exchange::iter()
        .filter(|&exchange| empty_or_contains(exchanges, exchange))
        .flat_map(|exchange| {
            Security::iter()
                .filter(|&security| empty_or_contains(securities, security))
                .map(move |security| (security, exchange))
        })
        .collect()
}

impl Row for TradeFee {
    fn covered(&self) -> Vec<(Security, Exchange)> {
        pairs(&self.securities, &self.exchanges)
    }

    fn covering_only(&self, securities: Vec<Security>, exchanges: Vec<Exchange>) -> Self {
        TradeFee {
            securities,
            exchanges,
            price: self.price.clone(),
        }
    }
}

impl Row for CustodyFee {
    fn covered(&self) -> Vec<(Security, Exchange)> {
        pairs(&self.securities, &self.exchanges)
    }

    fn covering_only(&self, securities: Vec<Security>, exchanges: Vec<Exchange>) -> Self {
        CustodyFee {
            securities,
            exchanges,
            ..self.clone()
        }
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
    let winner = |covered: (Security, Exchange)| {
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
    /// The percentage in `amount`, plus `per_share`. Short enough for a
    /// phone: after the field's "%", it reads "0.15 % + per share".
    #[strum(to_string = "+ per share")]
    PercentPlusPerShare,
}

/// A trade fee as the editor's fields. Empty fields are `None`: an empty
/// amount is free, an empty bound is no bound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TradeFields {
    pub kind: PriceKind,
    /// The percentage, or the amount per share or per order.
    pub amount: Option<Amount>,
    /// Only for [`PriceKind::PercentPlusPerShare`]: the amount per share.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_share: Option<Amount>,
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
#[serde(rename_all = "camelCase")]
pub struct MarkupFields {
    /// Empty, with no `per_dollar`, means the broker doesn't publish it.
    pub percent: Option<Amount>,
    /// Shekels for every dollar converted, instead of a percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_dollar: Option<Amount>,
}

/// A monthly handling fee as the editor's fields. An empty amount means none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct HandlingFields {
    /// In ₪.
    pub per_month: Option<Amount>,
    pub free_months: u32,
    pub less_trade_fees: bool,
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
    pub price: PriceText,
    pub was: Option<Was<F>>,
}

/// What the original plan charges, to show and to go back to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Was<F> {
    pub fields: F,
    pub price: PriceText,
}

impl<F> Fee<F> {
    fn new(fields: F, currency: &Currency, price: PriceText) -> Self {
        Fee {
            fields,
            currency: currency.symbol.to_owned(),
            price,
            was: None,
        }
    }

    /// Adds what the original plan charges, if it's different.
    fn compared_to(self, original: Option<Fee<F>>) -> Self {
        let was = original
            .filter(|original| original.price != self.price)
            .map(|original| Was {
                fields: original.fields,
                price: original.price,
            });
        Fee { was, ..self }
    }
}

/// The fees for one security on one exchange: the editor's simple view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SimpleFees {
    /// `None` if the plan doesn't offer it.
    pub trade: Option<Fee<TradeFields>>,
    /// Only if the plan has a cheaper price for buying it by standing order.
    pub standing_order: Option<Fee<TradeFields>>,
    pub custody: Fee<CustodyFields>,
    pub handling: Fee<HandlingFields>,
    /// Only abroad: nothing is converted on Tel Aviv.
    pub conversion: Option<Fee<ConversionFields>>,
    /// Only abroad, and only if the plan has a second conversion fee.
    pub second_conversion: Option<Fee<ConversionFields>>,
    /// Only abroad, and only if purchases by standing order convert
    /// differently.
    pub standing_order_conversion: Option<Fee<ConversionFields>>,
    pub markup: Option<Fee<MarkupFields>>,
    /// Whether fractions of a share are sold, for a security that's
    /// otherwise bought in whole shares there; `None` for others.
    pub sells_fractions: Option<bool>,
}

/// Every row of a plan's price list: the editor's full view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PriceList {
    pub trading: Vec<PriceRow<TradeFields>>,
    /// Empty unless the plan has cheaper prices for buying by standing order.
    pub standing_orders: Vec<PriceRow<TradeFields>>,
    pub custody: Vec<PriceRow<CustodyFields>>,
    pub handling: Fee<HandlingFields>,
    pub conversion: Fee<ConversionFields>,
    pub second_conversion: Option<Fee<ConversionFields>>,
    pub standing_order_conversion: Option<Fee<ConversionFields>>,
    pub markup: Fee<MarkupFields>,
    /// The exchanges where shares are otherwise bought whole, and whether
    /// fractions of a share are sold there.
    pub fractions: Vec<Fractions>,
    /// In ₪.
    pub min_first_deposit: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Fractions {
    pub exchange: Exchange,
    pub sold: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PriceRow<F> {
    /// Empty means all.
    pub securities: Vec<Security>,
    /// Empty means all.
    pub exchanges: Vec<Exchange>,
    /// "ETF on Tel Aviv"
    pub covers: String,
    /// "except Index fund on Tel Aviv (its own row above)"
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
        let (kind, amount, per_share, min, max) = match *self {
            Price::Percent { percent, min, max } => {
                (PriceKind::Percent, Some(Amount(percent.0)), None, min, max)
            }
            Price::PerShare {
                per_share,
                min,
                max,
            } => (PriceKind::PerShare, field(Some(per_share)), None, min, max),
            Price::Flat(amount) => (PriceKind::PerOrder, field(Some(amount)), None, None, None),
            Price::PercentPlusPerShare {
                percent,
                per_share,
                min,
                max,
            } => (
                PriceKind::PercentPlusPerShare,
                Some(Amount(percent.0)),
                field(Some(per_share)),
                min,
                max,
            ),
        };
        TradeFields {
            kind,
            amount,
            per_share,
            min: field(min),
            max: field(max),
        }
    }

    /// The currency of its first amount, if it has any.
    fn currency(&self) -> Option<&'static Currency> {
        match self {
            Price::Percent { min, max, .. } => min.or(*max).map(|money| money.currency()),
            Price::PerShare { per_share, .. } | Price::PercentPlusPerShare { per_share, .. } => {
                Some(per_share.currency())
            }
            Price::Flat(amount) => Some(amount.currency()),
        }
    }

    fn from_fields(fields: &TradeFields, currency: &'static Currency) -> Result<Self, InvalidFee> {
        let amount = amount_or_zero(fields.amount)?;
        let money = |amount| Money::from_decimal(amount, currency);
        let (min, max) = bounds(fields.min, fields.max, currency)?;
        Ok(match fields.kind {
            PriceKind::Percent => Price::Percent {
                percent: Percent(amount),
                min,
                max,
            },
            PriceKind::PerShare => Price::PerShare {
                per_share: money(amount),
                min,
                max,
            },
            PriceKind::PerOrder => Price::Flat(money(amount)),
            PriceKind::PercentPlusPerShare => Price::PercentPlusPerShare {
                percent: Percent(amount),
                per_share: money(amount_or_zero(fields.per_share)?),
                min,
                max,
            },
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
        Fee::new(
            self.price.fields(),
            self.currency(),
            self.price.price_text(),
        )
    }
}

impl CustodyFee {
    /// What a plan without a custody row charges: nothing.
    fn none(exchanges: Vec<Exchange>) -> Self {
        CustodyFee {
            securities: vec![],
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
        Fee::new(fields, self.currency(), self.price_text())
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

impl PercentFee {
    fn fields(&self) -> ConversionFields {
        ConversionFields {
            percent: Some(Amount(self.percent.0)),
            min: field(self.min),
            max: field(self.max),
        }
    }

    fn from_fields(
        fields: &ConversionFields,
        currency: &'static Currency,
    ) -> Result<Self, InvalidFee> {
        let (min, max) = bounds(fields.min, fields.max, currency)?;
        Ok(PercentFee {
            percent: Percent(amount_or_zero(fields.percent)?),
            min,
            max,
        })
    }
}

impl ConversionFee {
    /// The currency of its amounts: dollars, unless its bounds say otherwise.
    fn currency(&self) -> &'static Currency {
        let second = self.or_if_less.unwrap_or_default();
        [self.fee.min, self.fee.max, second.min, second.max]
            .into_iter()
            .flatten()
            .next()
            .map_or(iso::USD, |money| money.currency())
    }

    fn fee(&self) -> Fee<ConversionFields> {
        Fee::new(self.fee.fields(), self.currency(), self.price_text())
    }

    fn second_fee(&self) -> Option<Fee<ConversionFields>> {
        self.or_if_less
            .map(|second| Fee::new(second.fields(), self.currency(), second.price_text()))
    }

    fn markup_fee(&self) -> Fee<MarkupFields> {
        let (percent, per_dollar) = match self.markup {
            Markup::UpTo(percent) => (Some(Amount(percent.0)), None),
            Markup::PerDollar(amount) => (None, field(Some(amount))),
            // The market rate is a markup of 0%.
            Markup::MarketRate => (Some(Amount(Decimal::ZERO)), None),
            Markup::NotPublished => (None, None),
        };
        Fee::new(
            MarkupFields {
                percent,
                per_dollar,
            },
            self.currency(),
            self.markup.price_text(),
        )
    }
}

impl HandlingFee {
    fn fee(handling: Option<&HandlingFee>) -> Fee<HandlingFields> {
        let fields = HandlingFields {
            per_month: handling.and_then(|fee| field(Some(fee.per_month))),
            free_months: handling.map_or(0, |fee| fee.free_months),
            less_trade_fees: handling.is_some_and(|fee| fee.less_trade_fees),
        };
        let text = handling.map_or_else(|| PriceText::nothing("none"), Priced::price_text);
        Fee::new(fields, iso::ILS, text)
    }
}

// ─────────────────────────── Your plans ───────────────────────────

impl Plan {
    /// A copy to change, named "Pepper, your deal". It charges exactly what
    /// the original does on `track` (the first if none is given), with its
    /// rows most specific first: a customer is on one track, so a copy has
    /// no choice left. The caveats stay with the original: the user's fees
    /// are their own claim.
    #[must_use]
    pub fn copy_of(&self, track: Option<usize>) -> Plan {
        let index = track.unwrap_or(0);
        let description = self.tracks.get(index).map_or_else(String::new, |track| {
            format!("On the \u{201c}{}\u{201d} track.", track.name)
        });
        Plan {
            name: format!("{}, your deal", self.name),
            description,
            caveats: vec![],
            ..self.on_track(index).most_specific_first()
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
            tracks: vec![],
            standing_orders: vec![],
            standing_order_conversion: None,
            custody: vec![CustodyFee::none(vec![])],
            conversion: ConversionFee::FREE,
            handling: None,
            fractions_on: vec![],
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
            standing_orders: most_specific_first(&self.standing_orders),
            custody: most_specific_first(&self.custody),
            ..self.clone()
        }
    }

    /// Sorts the rows most specific first, after they change.
    pub fn sort_rows(&mut self) {
        sort_most_specific_first(&mut self.trading);
        sort_most_specific_first(&mut self.standing_orders);
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
        let standing_order = |plan: &Plan| {
            plan.standing_order_row(security, exchange)
                .map(TradeFee::fee)
        };
        let custody = |plan: &Plan| {
            plan.custody_row(security, exchange)
                .map_or_else(|| CustodyFee::none(vec![exchange]).fee(), CustodyFee::fee)
        };
        let handling = |plan: &Plan| HandlingFee::fee(plan.handling.as_ref());
        let second_conversion = |plan: &Plan| plan.conversion.second_fee();
        let standing_order_conversion = |plan: &Plan| {
            plan.standing_order_conversion
                .as_ref()
                .map(ConversionFee::fee)
        };
        let abroad = exchange != Exchange::Tlv;
        SimpleFees {
            trade: trade(self).map(|fee| fee.compared_to(original.and_then(trade))),
            standing_order: standing_order(self)
                .map(|fee| fee.compared_to(original.and_then(standing_order))),
            custody: custody(self).compared_to(original.map(custody)),
            handling: handling(self).compared_to(original.map(handling)),
            conversion: abroad.then(|| {
                self.conversion
                    .fee()
                    .compared_to(original.map(|plan| plan.conversion.fee()))
            }),
            second_conversion: second_conversion(self)
                .filter(|_| abroad)
                .map(|fee| fee.compared_to(original.and_then(second_conversion))),
            standing_order_conversion: standing_order_conversion(self)
                .filter(|_| abroad)
                .map(|fee| fee.compared_to(original.and_then(standing_order_conversion))),
            markup: abroad.then(|| {
                self.conversion
                    .markup_fee()
                    .compared_to(original.map(|plan| plan.conversion.markup_fee()))
            }),
            sells_fractions: whole_shares(security, exchange)
                .then(|| self.sells_fractions_on(exchange)),
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

    /// Sets the standing order price for `security` on `exchange`, in the
    /// row it uses. There's none to set if the plan has no such price.
    pub fn set_standing_order(
        &mut self,
        security: Security,
        exchange: Exchange,
        fields: &TradeFields,
    ) -> Result<(), InvalidFee> {
        let row = self
            .standing_orders
            .iter_mut()
            .find(|row| row.applies_to(security, exchange))
            .ok_or(InvalidFee::NoSuchRow)?;
        row.price = Price::from_fields(fields, row.currency())?;
        Ok(())
    }

    /// Sets custody on `security` on `exchange`: the row it uses, or a new
    /// one for the exchange.
    pub fn set_custody(
        &mut self,
        security: Security,
        exchange: Exchange,
        fields: &CustodyFields,
    ) -> Result<(), InvalidFee> {
        let index = self
            .custody
            .iter()
            .position(|row| row.applies_to(security, exchange));
        if let Some(index) = index {
            return self.custody[index].set(fields);
        }
        let mut row = CustodyFee::none(vec![exchange]);
        row.set(fields)?;
        self.custody.push(row);
        self.sort_rows();
        Ok(())
    }

    /// Sets the monthly handling fee; an empty amount removes it.
    pub fn set_handling(&mut self, fields: &HandlingFields) -> Result<(), InvalidFee> {
        self.handling = match fields.per_month {
            Some(amount) => Some(HandlingFee {
                per_month: Money::from_decimal(amount_or_zero(Some(amount))?, iso::ILS),
                free_months: fields.free_months,
                less_trade_fees: fields.less_trade_fees,
            }),
            None => None,
        };
        Ok(())
    }

    pub fn set_conversion(&mut self, fields: &ConversionFields) -> Result<(), InvalidFee> {
        self.conversion.fee = PercentFee::from_fields(fields, self.conversion.currency())?;
        Ok(())
    }

    /// Sets the second conversion fee, of which each conversion pays the
    /// lower.
    pub fn set_second_conversion(&mut self, fields: &ConversionFields) -> Result<(), InvalidFee> {
        let second = PercentFee::from_fields(fields, self.conversion.currency())?;
        self.conversion.or_if_less = Some(second);
        Ok(())
    }

    /// Sets what converting costs for purchases by standing order.
    pub fn set_standing_order_conversion(
        &mut self,
        fields: &ConversionFields,
    ) -> Result<(), InvalidFee> {
        let conversion = self
            .standing_order_conversion
            .clone()
            .unwrap_or(ConversionFee::FREE);
        let fee = PercentFee::from_fields(fields, conversion.currency())?;
        self.standing_order_conversion = Some(ConversionFee { fee, ..conversion });
        Ok(())
    }

    pub fn set_markup(&mut self, fields: &MarkupFields) -> Result<(), InvalidFee> {
        self.conversion.markup = match (fields.per_dollar, fields.percent) {
            (Some(per_dollar), _) => Markup::PerDollar(Money::from_decimal(
                amount_or_zero(Some(per_dollar))?,
                iso::ILS,
            )),
            (None, Some(percent)) => Markup::UpTo(Percent(amount_or_zero(Some(percent))?)),
            (None, None) => Markup::NotPublished,
        };
        Ok(())
    }

    /// Sets whether fractions of a share are sold on `exchange`.
    pub fn set_sells_fractions(&mut self, exchange: Exchange, sells: bool) {
        // Rebuilt in the usual order. Empty means none here, not all.
        self.fractions_on = Exchange::iter()
            .filter(|&each| {
                if each == exchange {
                    sells
                } else {
                    self.fractions_on.contains(&each)
                }
            })
            .collect();
    }

    /// Every row, and what the most specific rule does to each, compared to
    /// `original`'s row with the same coverage.
    #[must_use]
    pub fn price_list(&self, original: Option<&Plan>) -> PriceList {
        let original = original.map(Plan::most_specific_first);
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
                    securities: row.securities.clone(),
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
            trading: trade_price_rows(
                &self.trading,
                original.as_ref().map(|plan| plan.trading.as_slice()),
            ),
            standing_orders: trade_price_rows(
                &self.standing_orders,
                original
                    .as_ref()
                    .map(|plan| plan.standing_orders.as_slice()),
            ),
            custody,
            handling: HandlingFee::fee(self.handling.as_ref()).compared_to(
                original
                    .as_ref()
                    .map(|plan| HandlingFee::fee(plan.handling.as_ref())),
            ),
            conversion: self
                .conversion
                .fee()
                .compared_to(original.as_ref().map(|plan| plan.conversion.fee())),
            second_conversion: self.conversion.second_fee().map(|fee| {
                fee.compared_to(
                    original
                        .as_ref()
                        .and_then(|plan| plan.conversion.second_fee()),
                )
            }),
            standing_order_conversion: self.standing_order_conversion.as_ref().map(|fee| {
                fee.fee().compared_to(
                    original
                        .as_ref()
                        .and_then(|plan| plan.standing_order_conversion.as_ref())
                        .map(ConversionFee::fee),
                )
            }),
            markup: self
                .conversion
                .markup_fee()
                .compared_to(original.as_ref().map(|plan| plan.conversion.markup_fee())),
            fractions: Exchange::iter()
                .filter(|&exchange| {
                    Security::iter().any(|security| whole_shares(security, exchange))
                })
                .map(|exchange| Fractions {
                    exchange,
                    sold: self.sells_fractions_on(exchange),
                })
                .collect(),
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
        set_row(&mut self.trading, index, securities, exchanges, fields)
    }

    pub fn remove_trade_row(&mut self, index: usize) -> Result<(), InvalidFee> {
        remove_row(&mut self.trading, index)
    }

    /// Changes what standing order row `index` covers and charges.
    pub fn set_standing_order_row(
        &mut self,
        index: usize,
        securities: &[Security],
        exchanges: &[Exchange],
        fields: &TradeFields,
    ) -> Result<(), InvalidFee> {
        set_row(
            &mut self.standing_orders,
            index,
            securities,
            exchanges,
            fields,
        )
    }

    /// Adds a free custody row for exactly `exchange`.
    pub fn add_custody_row(&mut self, exchange: Exchange) {
        self.custody.push(CustodyFee::none(vec![exchange]));
        self.sort_rows();
    }

    /// Changes what custody row `index` covers and charges. Empty lists mean
    /// all.
    pub fn set_custody_row(
        &mut self,
        index: usize,
        securities: &[Security],
        exchanges: &[Exchange],
        fields: &CustodyFields,
    ) -> Result<(), InvalidFee> {
        let row = self.custody.get_mut(index).ok_or(InvalidFee::NoSuchRow)?;
        row.set(fields)?;
        row.securities = all_as_empty(securities);
        row.exchanges = all_as_empty(exchanges);
        self.sort_rows();
        Ok(())
    }

    pub fn remove_custody_row(&mut self, index: usize) -> Result<(), InvalidFee> {
        remove_row(&mut self.custody, index)
    }

    /// In ₪; empty for none.
    pub fn set_min_first_deposit(&mut self, amount: Option<Amount>) -> Result<(), InvalidFee> {
        self.min_first_deposit = bounds(amount, None, iso::ILS)?.0;
        Ok(())
    }
}

// A plan has two tables of trade rows, the usual one and standing orders',
// which the editor shows and changes the same way.

/// Every row of `rows`, and what the most specific rule does to each,
/// compared to `original`'s row with the same coverage.
fn trade_price_rows(
    rows: &[TradeFee],
    original: Option<&[TradeFee]>,
) -> Vec<PriceRow<TradeFields>> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let (by, never_used) = taken_by(rows, index);
            let was = original.and_then(|original| {
                original
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
                        .map(|&other| rows[other].coverage().to_string())
                        .collect::<Vec<_>>(),
                ),
                never_used,
                fee: row.fee().compared_to(was),
            }
        })
        .collect()
}

/// Changes what row `index` covers and charges. Empty lists mean all.
fn set_row(
    rows: &mut [TradeFee],
    index: usize,
    securities: &[Security],
    exchanges: &[Exchange],
    fields: &TradeFields,
) -> Result<(), InvalidFee> {
    let row = rows.get(index).ok_or(InvalidFee::NoSuchRow)?;
    let exchanges = all_as_empty(exchanges);
    // A row that moves between Tel Aviv and abroad without amounts of its
    // own takes the currency of where it is now.
    let currency = row
        .price
        .currency()
        .unwrap_or_else(|| usual_currency(&exchanges));
    rows[index] = TradeFee {
        securities: all_as_empty(securities),
        exchanges,
        price: Price::from_fields(fields, currency)?,
    };
    sort_most_specific_first(rows);
    Ok(())
}

fn remove_row<R>(rows: &mut Vec<R>, index: usize) -> Result<(), InvalidFee> {
    if index >= rows.len() {
        return Err(InvalidFee::NoSuchRow);
    }
    rows.remove(index);
    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::tariffs::{self, altshuler};
    use crate::{Broker, IntoEnumIterator};

    fn listed() -> Vec<Plan> {
        tariffs::all()
            .into_iter()
            .flat_map(|broker: Broker| broker.plans)
            .collect()
    }

    /// Every listed plan, and each on each of its tracks.
    fn listed_on_tracks() -> Vec<Plan> {
        listed()
            .into_iter()
            .flat_map(|plan| {
                let on_tracks: Vec<Plan> = (0..plan.tracks.len())
                    .map(|index| plan.on_track(index))
                    .collect();
                std::iter::once(plan).chain(on_tracks)
            })
            .collect()
    }

    fn named(name: &str) -> Plan {
        listed().into_iter().find(|plan| plan.name == name).unwrap()
    }

    /// What `plan` charges for each security on each exchange: to buy or
    /// sell, to buy by standing order, and to hold it.
    fn prices(plan: &Plan) -> Vec<(Option<Price>, Option<Price>, Option<CustodyFee>)> {
        Exchange::iter()
            .flat_map(|exchange| {
                Security::iter().map(move |security| {
                    let custody = plan.custody_row(security, exchange).map(|row| CustodyFee {
                        exchanges: vec![],
                        ..row.clone()
                    });
                    let price = plan.trade_row(security, exchange).map(|r| r.price.clone());
                    let standing_order = plan
                        .standing_order_row(security, exchange)
                        .map(|r| r.price.clone());
                    (price, standing_order, custody)
                })
            })
            .collect()
    }

    #[test]
    fn rewritten_plans_charge_the_same() {
        for plan in listed_on_tracks() {
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
                // Only sorted: the listed order is the document's, but no
                // row loses anything.
                let mut sorted = plan.clone();
                sorted.sort_rows();
                assert_eq!(plan.most_specific_first(), sorted, "{}", plan.name);
            }
        }
        // Its offer row is laid over the regular list: the regular ETF and
        // index fund rows on Tel Aviv never applied, and its stocks were
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
                "Bond, Index fund on USA",
                "Stock, ETF, Index fund on Tel Aviv",
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
        let copy = pepper.copy_of(None);
        assert_eq!(copy.name, "Pepper, your deal");
        assert_eq!(copy.caveats, []);
        assert_eq!(prices(&copy), prices(&pepper));
        let simple = copy.simple_fees(Security::Etf, Exchange::Usa, Some(&pepper));
        assert_eq!(simple.trade.unwrap().price.text, "$4 per order");
        assert!(simple.custody.was.is_none());
    }

    #[test]
    fn simple_fees_round_trip_and_show_the_original() {
        let pepper = named("Pepper");
        let mut copy = pepper.copy_of(None);
        let fields = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.1))),
            per_share: None,
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
        assert_eq!(trade.price.text, "0.1%, min $2");
        assert_eq!(trade.was.unwrap().price.text, "$4 per order");

        let custody = CustodyFields {
            percent: Some(Amount(dec!(0.1))),
            per: Period::Quarter,
            billed: Period::Month,
            min: Some(Amount(dec!(20))),
        };
        copy.set_custody(Security::Etf, Exchange::Usa, &custody)
            .unwrap();
        let simple = copy.simple_fees(Security::Etf, Exchange::Usa, None);
        assert_eq!(simple.custody.fields, custody);
        assert_eq!(simple.custody.currency, "₪");

        copy.set_markup(&MarkupFields {
            percent: None,
            per_dollar: None,
        })
        .unwrap();
        assert_eq!(copy.conversion.markup, Markup::NotPublished);
    }

    #[test]
    fn a_shared_row_changes_everywhere_it_covers() {
        // Leumi Online prices the USA and Europe in one row.
        let mut copy = named("Online").copy_of(None);
        let fields = TradeFields {
            kind: PriceKind::PerOrder,
            amount: Some(Amount(dec!(5))),
            per_share: None,
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
        let mut copy = altshuler().plans[0].copy_of(None);
        let simple = copy.simple_fees(Security::Etf, Exchange::Europe, None);
        assert!(simple.trade.is_none());
        let fields = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.2))),
            per_share: None,
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
            per_share: None,
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
            per_share: None,
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
        for (price, standing_order, custody) in prices(&plan) {
            assert_eq!(price.unwrap().to_string(), "0%");
            assert_eq!(standing_order, None);
            assert!(custody.unwrap().percent.is_zero());
        }
        let simple = plan.simple_fees(Security::Etf, Exchange::Usa, None);
        assert_eq!(simple.trade.unwrap().currency, "$");
        assert_eq!(simple.markup.unwrap().price.text, "none");
    }

    #[test]
    fn plans_survive_json() {
        let plan = named("Online").copy_of(None);
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(serde_json::from_str::<Plan>(&json).unwrap(), plan);
        for name in ["Online, monthly standing order", "Online, 'Leumi 18+'"] {
            let plan = named(name).copy_of(None);
            let json = serde_json::to_string(&plan).unwrap();
            assert_eq!(serde_json::from_str::<Plan>(&json).unwrap(), plan);
        }

        // Saved before plans had standing orders: they have none.
        let mut saved = serde_json::to_value(&plan).unwrap();
        saved.as_object_mut().unwrap().remove("standing_orders");
        assert_eq!(serde_json::from_value::<Plan>(saved).unwrap(), plan);
        let fields = serde_json::to_string(&MarkupFields {
            percent: Some(Amount(dec!(0.7))),
            per_dollar: None,
        })
        .unwrap();
        assert_eq!(fields, r#"{"percent":0.7}"#);
    }

    #[test]
    fn a_standing_order_price_is_edited() {
        let listed = named("Online, monthly standing order");
        let mut copy = listed.copy_of(None);
        let (fund, tlv) = (Security::IndexFund, Exchange::Tlv);
        let cheaper = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.1))),
            per_share: None,
            min: Some(Amount(dec!(3))),
            max: None,
        };
        copy.set_standing_order(fund, tlv, &cheaper).unwrap();
        let simple = copy.simple_fees(fund, tlv, Some(&listed));
        let standing_order = simple.standing_order.unwrap();
        assert_eq!(standing_order.price.text, "0.1%, min ₪3");
        assert_eq!(
            standing_order.was.unwrap().price.text,
            "0.225%, min ₪5, max ₪6,300"
        );
        // Buying or selling otherwise is still Online's.
        assert_eq!(
            simple.trade.unwrap().price.text,
            "0.4%, min ₪26, max ₪6,300"
        );

        // ETFs have no standing order price to set.
        let etf = copy.simple_fees(Security::Etf, tlv, None);
        assert_eq!(etf.standing_order, None);
        assert_eq!(
            copy.set_standing_order(Security::Etf, tlv, &cheaper),
            Err(InvalidFee::NoSuchRow)
        );

        let rows = copy.price_list(Some(&listed)).standing_orders;
        assert_eq!(rows[0].covers, "Index fund on Tel Aviv");
        let wider = copy.set_standing_order_row(0, &[], &[tlv], &cheaper);
        assert_eq!(wider, Ok(()));
        assert!(
            copy.simple_fees(Security::Etf, tlv, None)
                .standing_order
                .is_some()
        );
    }

    #[test]
    fn a_second_conversion_fee_is_edited() {
        let listed = named("Online, 'Leumi 18+'");
        let mut copy = listed.copy_of(None);
        let (etf, usa) = (Security::Etf, Exchange::Usa);
        let second = |plan: &Plan| plan.simple_fees(etf, usa, Some(&listed)).second_conversion;
        assert_eq!(
            second(&copy).unwrap().price.text,
            "0.16%, min $5.76, max $2,400"
        );
        // Not shown on Tel Aviv, where nothing is converted.
        assert_eq!(
            copy.simple_fees(etf, Exchange::Tlv, None).second_conversion,
            None
        );

        let fields = ConversionFields {
            percent: Some(Amount(dec!(0.12))),
            min: Some(Amount(dec!(4))),
            max: None,
        };
        copy.set_second_conversion(&fields).unwrap();
        let edited = second(&copy).unwrap();
        assert_eq!(
            (edited.price.text.as_str(), edited.currency.as_str()),
            ("0.12%, min $4", "$")
        );
        assert_eq!(
            edited.was.unwrap().price.text,
            "0.16%, min $5.76, max $2,400"
        );
        let listed_online = named("Online");
        assert_eq!(listed_online.price_list(None).second_conversion, None);
    }

    #[test]
    fn custody_rows_are_rewritten_like_trade_rows() {
        use Exchange::*;
        use Security::*;
        let row = |securities, exchanges| CustodyFee {
            securities,
            exchanges,
            percent: Percent(dec!(0.1)),
            per: Period::Quarter,
            billed: Period::Quarter,
            min: None,
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
        assert!(rewritten.iter().all(|r| r.percent == Percent(dec!(0.1))));
    }

    #[test]
    fn the_rewrite_sorts_standing_orders_too() {
        let row = |securities, exchanges| TradeFee {
            securities,
            exchanges,
            price: Price::Flat(crate::ils(dec!(1))),
        };
        let plan = Plan {
            standing_orders: vec![
                row(vec![Security::Etf], vec![Exchange::Tlv, Exchange::Usa]),
                row(vec![Security::IndexFund], vec![Exchange::Tlv]),
            ],
            ..named("Online, monthly standing order")
        };
        let rewritten = plan.most_specific_first();
        assert_eq!(rewritten.standing_orders[0].covered().len(), 1);
        assert_eq!(rewritten.standing_orders.len(), 2);
    }

    #[test]
    fn a_minimum_may_equal_the_maximum() {
        let mut plan = Plan::new_own("Mine");
        let fields = TradeFields {
            kind: PriceKind::Percent,
            amount: Some(Amount(dec!(0.1))),
            per_share: None,
            min: Some(Amount(dec!(5))),
            max: Some(Amount(dec!(5))),
        };
        assert_eq!(
            plan.set_trade(Security::Etf, Exchange::Tlv, &fields),
            Ok(())
        );
    }

    #[test]
    fn amounts_are_in_shekels_on_tel_aviv_and_dollars_abroad_unless_the_row_says() {
        let per_order = |amount| TradeFields {
            kind: PriceKind::PerOrder,
            amount: Some(Amount(amount)),
            per_share: None,
            min: None,
            max: None,
        };
        let price = |plan: &Plan, exchange| {
            plan.trade_row(Security::Etf, exchange)
                .unwrap()
                .price
                .to_string()
        };
        let mut plan = Plan::new_own("Mine");
        plan.set_trade(Security::Etf, Exchange::Usa, &per_order(dec!(4)))
            .unwrap();
        plan.set_trade(Security::Etf, Exchange::Tlv, &per_order(dec!(3)))
            .unwrap();
        assert_eq!(price(&plan, Exchange::Usa), "$4 per order");
        assert_eq!(price(&plan, Exchange::Tlv), "₪3 per order");
        // A row with amounts of its own keeps their currency when it moves.
        let tel_aviv = plan
            .trading
            .iter()
            .position(|row| row.exchanges == [Exchange::Tlv])
            .unwrap();
        plan.set_trade_row(tel_aviv, &[], &[Exchange::Europe], &per_order(dec!(3)))
            .unwrap();
        assert_eq!(price(&plan, Exchange::Europe), "₪3 per order");
        // One without takes the currency of where it is: shekels for a row
        // covering every exchange, as tariffs write them.
        let mut fresh = Plan::new_own("Mine");
        fresh
            .set_trade_row(1, &[], &[], &per_order(dec!(2)))
            .unwrap();
        assert_eq!(price(&fresh, Exchange::Usa), "₪2 per order");
        let mut fresh = Plan::new_own("Mine");
        fresh
            .set_trade_row(1, &[], &[Exchange::Usa], &per_order(dec!(2)))
            .unwrap();
        assert_eq!(price(&fresh, Exchange::Usa), "$2 per order");
    }

    #[test]
    fn a_copy_says_which_track_it_is_on() {
        let full = altshuler().plans.remove(0);
        assert_eq!(
            full.copy_of(Some(1)).description,
            "On the \u{201c}$11 per order\u{201d} track."
        );
        assert_eq!(named("Pepper").copy_of(None).description, "");
    }

    #[test]
    fn a_standing_order_conversion_takes_its_fields() {
        let fields = ConversionFields {
            percent: Some(Amount(dec!(0.5))),
            min: Some(Amount(dec!(2))),
            max: None,
        };
        // Interactive's automatic plan converts for free; the fee is set on it.
        let mut standard = named("Standard");
        standard.set_standing_order_conversion(&fields).unwrap();
        let fee = standard.standing_order_conversion.as_ref().unwrap().fee;
        assert_eq!(fee.percent, Percent(dec!(0.5)));
        assert_eq!(*fee.min.unwrap().amount(), dec!(2));
        // A plan without one gets one, converting at the market rate otherwise.
        let mut own = Plan::new_own("Mine");
        own.set_standing_order_conversion(&fields).unwrap();
        let conversion = own.standing_order_conversion.unwrap();
        assert_eq!(conversion.fee.percent, Percent(dec!(0.5)));
        assert_eq!(conversion.markup, Markup::NONE);
    }

    #[test]
    fn fractions_are_set_per_exchange() {
        let mut plan = Plan::new_own("Mine");
        plan.set_sells_fractions(Exchange::Usa, true);
        assert_eq!(plan.fractions_on, [Exchange::Usa]);
        plan.set_sells_fractions(Exchange::Usa, true);
        assert_eq!(plan.fractions_on, [Exchange::Usa]);
        plan.set_sells_fractions(Exchange::Europe, true);
        plan.set_sells_fractions(Exchange::Usa, false);
        assert_eq!(plan.fractions_on, [Exchange::Europe]);
    }

    #[test]
    fn custody_rows_are_added_changed_and_removed() {
        let mut plan = Plan::new_own("Mine");
        plan.add_custody_row(Exchange::Usa);
        assert_eq!(plan.custody.len(), 2);
        // The new row, for the USA only, sorts before the one for everything.
        assert_eq!(plan.custody[0].exchanges, [Exchange::Usa]);
        assert!(plan.custody[0].is_free());
        let fields = CustodyFields {
            percent: Some(Amount(dec!(0.2))),
            per: Period::Year,
            billed: Period::Month,
            min: Some(Amount(dec!(5))),
        };
        plan.set_custody_row(0, &[Security::Etf], &[Exchange::Usa], &fields)
            .unwrap();
        assert_eq!(plan.custody[0].securities, [Security::Etf]);
        assert_eq!(plan.custody[0].percent, Percent(dec!(0.2)));
        assert_eq!(plan.custody[0].billed, Period::Month);
        assert_eq!(plan.custody[0].min, Some(crate::ils(dec!(5))));
        assert_eq!(
            plan.set_custody_row(5, &[], &[], &fields),
            Err(InvalidFee::NoSuchRow)
        );
        plan.remove_custody_row(0).unwrap();
        assert_eq!(plan.custody.len(), 1);
        assert_eq!(plan.remove_custody_row(1), Err(InvalidFee::NoSuchRow));
    }

    #[test]
    fn the_minimum_first_deposit_is_set_in_shekels_or_cleared() {
        let mut plan = Plan::new_own("Mine");
        plan.set_min_first_deposit(Some(Amount(dec!(5000))))
            .unwrap();
        assert_eq!(plan.min_first_deposit, Some(crate::ils(dec!(5000))));
        assert_eq!(
            plan.price_list(None).min_first_deposit,
            Some(Amount(dec!(5000)))
        );
        assert_eq!(
            plan.set_min_first_deposit(Some(Amount(dec!(-1)))),
            Err(InvalidFee::Negative)
        );
        plan.set_min_first_deposit(None).unwrap();
        assert_eq!(plan.min_first_deposit, None);
    }

    #[test]
    fn the_price_list_shows_what_the_original_charged_where_it_differs() {
        fn was<F>(rows: &[PriceRow<F>]) -> Vec<Option<String>> {
            rows.iter()
                .map(|row| row.fee.was.as_ref().map(|was| was.price.text.clone()))
                .collect()
        }
        let online = named("Online");
        let mut copy = online.copy_of(None);
        // Custody on Tel Aviv doubled, and a flat price there; abroad unchanged.
        copy.set_custody(
            Security::Etf,
            Exchange::Tlv,
            &CustodyFields {
                percent: Some(Amount(dec!(0.3))),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        )
        .unwrap();
        copy.set_trade(
            Security::Etf,
            Exchange::Tlv,
            &TradeFields {
                kind: PriceKind::PerOrder,
                amount: Some(Amount(dec!(9))),
                per_share: None,
                min: None,
                max: None,
            },
        )
        .unwrap();
        let list = copy.price_list(Some(&online));
        assert_eq!(
            was(&list.custody),
            [Some("0.15% a quarter (0.6% a year)".into()), None]
        );
        assert_eq!(
            was(&list.trading),
            [Some("0.4%, min ₪26, max ₪6,300".into()), None]
        );
    }

    #[test]
    fn the_handling_fee_and_the_conversion_fee_are_set_from_their_fields() {
        let mut plan = Plan::new_own("Mine");
        plan.set_handling(&HandlingFields {
            per_month: Some(Amount(dec!(20))),
            free_months: 6,
            less_trade_fees: true,
        })
        .unwrap();
        assert_eq!(
            plan.handling,
            Some(HandlingFee {
                per_month: crate::ils(dec!(20)),
                free_months: 6,
                less_trade_fees: true,
            })
        );
        plan.set_handling(&HandlingFields {
            per_month: None,
            free_months: 0,
            less_trade_fees: false,
        })
        .unwrap();
        assert_eq!(plan.handling, None);

        plan.set_conversion(&ConversionFields {
            percent: Some(Amount(dec!(0.3))),
            min: Some(Amount(dec!(5))),
            max: None,
        })
        .unwrap();
        assert_eq!(plan.conversion.fee.percent, Percent(dec!(0.3)));
        assert_eq!(*plan.conversion.fee.min.unwrap().amount(), dec!(5));
        assert_eq!(
            plan.set_conversion(&ConversionFields {
                percent: Some(Amount(dec!(-1))),
                min: None,
                max: None,
            }),
            Err(InvalidFee::Negative)
        );
    }
}
