//! The tariffs in plain words, for showing to users: "0.3%, min $24, max
//! $6,750", "0.15% a quarter (0.6% a year)". Kept in the core so every UI
//! says the same thing.
//!
//! A type's own text is its [`Display`] (derived with strum for the simple
//! enums, where they're declared); other ways to show it are methods
//! returning `impl Display`, built with [`fmt::from_fn`].

use std::fmt::{self, Display, Formatter};

use rusty_money::{Formatter as MoneyFormatter, Params};
use serde::Serialize;
use time::macros::format_description;

use crate::{
    Broker, Caveat, ConversionFee, CustodyFee, Exchange, Markup, Money, Period, Plan, Price,
    Security, TradeFee,
};

/// A choice explained for someone who doesn't know the term, with the Hebrew
/// names that Israeli brokers and sites use for it.
pub trait Explained: Display {
    fn explanation(&self) -> &'static str;
    fn hebrew_names(&self) -> &'static [&'static str];
}

impl Explained for Security {
    fn explanation(&self) -> &'static str {
        match self {
            Security::Etf => {
                "A fund that holds many securities, such as the 500 companies of the S&P 500, \
                 and trades on the exchange all day like a share."
            }
            Security::MutualFund => {
                "A fund bought from and sold to its manager, at one price a day set after the \
                 exchange closes. Index-tracking ones (קרן\u{a0}מחקה) are mutual funds too."
            }
            Security::Bond => {
                "A loan to a government or a company, which pays interest and is traded on the \
                 exchange."
            }
            Security::Stock => "A share of a single company.",
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        match self {
            Security::Etf => &["קרן סל", "קרן סל מחקה מדד", "קרן סל במסלול רציף"],
            Security::MutualFund => &["קרן נאמנות"],
            // Both spellings of the full name are common.
            Security::Bond => &["אג\"ח", "איגרת חוב", "אגרת חוב"],
            Security::Stock => &["מניה"],
        }
    }
}

impl Explained for Exchange {
    fn explanation(&self) -> &'static str {
        match self {
            Exchange::Tlv => {
                "The Tel Aviv Stock Exchange. Prices are in shekels, so nothing is converted."
            }
            Exchange::Usa => {
                "NYSE or Nasdaq. Prices are in dollars, so your shekels are converted, which \
                 some brokers charge for."
            }
            Exchange::Europe => {
                "A European exchange, such as Xetra or Euronext. Prices are in euros, so your \
                 shekels are converted, which some brokers charge for."
            }
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        match self {
            Exchange::Tlv => &["הבורסה לניירות ערך בתל אביב"],
            Exchange::Usa | Exchange::Europe => &[],
        }
    }
}

/// "month", as in "0.15% a month".
impl Display for Period {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Period::Month => "month",
            Period::Quarter => "quarter",
            Period::Year => "year",
        })
    }
}

impl Period {
    /// "monthly", as in "charged monthly".
    #[must_use]
    pub fn adverb(self) -> &'static str {
        match self {
            Period::Month => "monthly",
            Period::Quarter => "quarterly",
            Period::Year => "yearly",
        }
    }
}

/// "an ETF bought in the USA", "a mutual fund bought in Tel Aviv".
#[must_use]
pub fn purchase(security: Security, exchange: Exchange) -> impl Display {
    fmt::from_fn(move |f| {
        let security = match security {
            Security::Etf => "an ETF",
            Security::MutualFund => "a mutual fund",
            Security::Bond => "a bond",
            Security::Stock => "a stock",
        };
        match exchange {
            Exchange::Usa => write!(f, "{security} bought in the USA"),
            other => write!(f, "{security} bought in {other}"),
        }
    })
}

/// The kinds of fee a plan charges, to name and explain them. Its `Display`
/// is its name: "Buy or sell", "Conversion markup".
/// In the order the editor shows them: `FeeKind::iter()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, strum::Display, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum FeeKind {
    #[strum(to_string = "Buy or sell")]
    Trade,
    Custody,
    Conversion,
    #[strum(to_string = "Conversion markup")]
    Markup,
}

impl Explained for FeeKind {
    fn explanation(&self) -> &'static str {
        match self {
            FeeKind::Trade => {
                "Charged on every purchase and sale, including selling everything at the end."
            }
            FeeKind::Custody => {
                "Charged for holding your securities, as a share of what they're worth."
            }
            FeeKind::Conversion => {
                "Charged for converting your shekels to the security's currency, and back \
                 when you sell."
            }
            FeeKind::Markup => {
                "The Currency Conversion Markup: the broker converts at a rate worse than \
                 the market's by this much. It isn't listed as a fee, but it costs the same."
            }
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        match self {
            FeeKind::Trade => &["עמלת קנייה/מכירה"],
            FeeKind::Custody => &["דמי משמרת"],
            FeeKind::Conversion => &["עמלת המרת מט\"ח"],
            FeeKind::Markup => &["מרווח המרה"],
        }
    }
}

/// One fee, named and explained, and what a plan charges for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeeLine {
    pub name: String,
    pub explanation: String,
    pub hebrew_names: Vec<String>,
    /// "0.15%, min ₪3.5", or why there's no price: "not offered", "none".
    pub price: String,
    /// True when there's no price.
    pub missing: bool,
}

impl FeeLine {
    fn new(kind: FeeKind, price: Option<String>, when_missing: &str) -> Self {
        FeeLine {
            name: kind.to_string(),
            explanation: kind.explanation().to_owned(),
            hebrew_names: kind
                .hebrew_names()
                .iter()
                .map(|&name| name.to_owned())
                .collect(),
            missing: price.is_none(),
            price: price.unwrap_or_else(|| when_missing.to_owned()),
        }
    }
}

/// What a plan charges for one security on one exchange, in words, and the
/// caveats that apply to it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeesFor {
    /// Conversion fees only abroad: nothing is converted on Tel Aviv.
    pub fees: Vec<FeeLine>,
    /// The conversion markup, shown as part of the conversion fee: it's the
    /// other half of what converting costs. Only abroad, like conversion.
    pub markup: Option<FeeLine>,
    pub caveats: Vec<String>,
}

impl Plan {
    /// The plan's own caveats; the broker's are added by
    /// [`Broker::describe_fees_for`].
    pub fn describe_fees_for(&self, security: Security, exchange: Exchange) -> FeesFor {
        let mut fees = vec![
            FeeLine::new(
                FeeKind::Trade,
                self.trade_row(security, exchange)
                    .map(|row| row.price.to_string()),
                "not offered",
            ),
            FeeLine::new(
                FeeKind::Custody,
                self.custody_row(exchange).map(ToString::to_string),
                "none",
            ),
        ];
        let abroad = exchange != Exchange::Tlv;
        if abroad {
            fees.push(FeeLine::new(
                FeeKind::Conversion,
                Some(self.conversion.to_string()),
                "",
            ));
        }
        FeesFor {
            fees,
            markup: abroad.then(|| {
                FeeLine::new(
                    FeeKind::Markup,
                    Some(self.conversion.markup.to_string()),
                    "",
                )
            }),
            caveats: caveats_for(&self.caveats, security, exchange),
        }
    }

    /// "Needs a one-time deposit of at least ₪5,000", if `first_deposit` is less
    /// than the plan's minimum.
    #[must_use]
    pub fn first_deposit_warning(&self, first_deposit: Money) -> Option<String> {
        self.min_first_deposit
            .filter(|min| first_deposit.amount() < min.amount())
            .map(|min| format!("Needs a one-time deposit of at least {}", format_money(min)))
    }
}

impl Broker {
    /// "Tariff of 29/06/2026", or that the tariff isn't dated.
    #[must_use]
    pub fn tariff_date_text(&self) -> impl Display + '_ {
        fmt::from_fn(|f| match self.tariff_date {
            Some(date) => {
                let date = date
                    .format(format_description!("[day]/[month]/[year]"))
                    .map_err(|_| fmt::Error)?;
                write!(f, "Tariff of {date}")
            }
            None => f.write_str("Tariff date not stated"),
        })
    }
}

impl TradeFee {
    /// What the row covers: "ETF on Tel Aviv", "Anything on USA, Europe".
    #[must_use]
    pub fn coverage(&self) -> impl Display + '_ {
        fmt::from_fn(|f| {
            write!(
                f,
                "{} on {}",
                list_or(&self.securities, "Anything"),
                list_or(&self.exchanges, "any exchange"),
            )
        })
    }
}

/// "0.3%, min $24, max $6,750", "$0.01 per share, min $9", "$4 per order".
impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Price::Percent { percent, min, max } => {
                write!(f, "{percent}")?;
                write_bounds(f, *min, *max)
            }
            Price::PerShare {
                per_share,
                min,
                max,
            } => {
                write!(f, "{} per share", format_money(*per_share))?;
                write_bounds(f, *min, *max)
            }
            Price::Flat(amount) => write!(f, "{} per order", format_money(*amount)),
        }
    }
}

impl CustodyFee {
    /// What the row covers: "Tel Aviv", "Any exchange".
    #[must_use]
    pub fn coverage(&self) -> impl Display + '_ {
        fmt::from_fn(|f| f.write_str(&list_or(&self.exchanges, "Any exchange")))
    }
}

/// "0.15% a quarter (0.6% a year)", "0.15% a year, charged monthly, min ₪75 a month".
impl Display for CustodyFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} a {}", self.percent, self.per)?;
        if self.per != Period::Year {
            write!(f, " ({} a year)", self.percent_per_year())?;
        }
        if self.billed != self.per {
            write!(f, ", charged {}", self.billed.adverb())?;
        }
        if let Some(min) = self.min {
            write!(f, ", min {} a {}", format_money(min), self.billed)?;
        }
        Ok(())
    }
}

/// The fee alone, without the markup: "0.16%, min $5.76, max $2,400", "none".
impl Display for ConversionFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.percent.is_zero() {
            f.write_str("none")?;
        } else {
            write!(f, "{}", self.percent)?;
            write_bounds(f, self.min, self.max)?;
        }
        Ok(())
    }
}

/// "up to 0.7%", "none", "not published"
impl Display for Markup {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Markup::UpTo(percent) if percent.is_zero() => f.write_str("none"),
            Markup::UpTo(percent) => write!(f, "up to {percent}"),
            Markup::NotPublished => f.write_str("not published"),
        }
    }
}

/// An amount as a tariff writes it: "$6,750", "$5.76", "₪3.5".
///
/// A function rather than a `Display` impl: `Money` and `Display` are both
/// defined in other crates, and Rust's orphan rule only allows implementing a
/// trait for a type if one of them is ours.
#[must_use]
pub fn format_money(money: Money) -> String {
    // Normalized, so whole amounts have no ".00".
    let money = Money::from_decimal(money.amount().normalize(), money.currency());
    let params = Params {
        symbol: Some(money.currency().symbol),
        ..Params::default()
    };
    MoneyFormatter::money(&money, params)
}

fn write_bounds(f: &mut Formatter<'_>, min: Option<Money>, max: Option<Money>) -> fmt::Result {
    if let Some(min) = min {
        write!(f, ", min {}", format_money(min))?;
    }
    if let Some(max) = max {
        write!(f, ", max {}", format_money(max))?;
    }
    Ok(())
}

/// "except Mutual fund on Tel Aviv (its own row above)", for a row of your
/// plan that more specific rows take part of; `covers` is what each of them
/// covers.
#[must_use]
pub fn except(covers: &[String]) -> Option<String> {
    let (last, rest) = covers.split_last()?;
    if rest.is_empty() {
        return Some(format!("except {last} (its own row above)"));
    }
    Some(format!(
        "except {} and {last} (their own rows above)",
        rest.join(", ")
    ))
}

impl Broker {
    /// What plan `plan` charges for `security` on `exchange`, with the
    /// broker's caveats first.
    #[must_use]
    pub fn describe_fees_for(
        &self,
        plan: &Plan,
        security: Security,
        exchange: Exchange,
    ) -> FeesFor {
        let mut fees = plan.describe_fees_for(security, exchange);
        let mut caveats = self.caveats_for(security, exchange);
        caveats.append(&mut fees.caveats);
        FeesFor { caveats, ..fees }
    }

    /// The broker-wide caveats that matter to someone buying `security` on
    /// `exchange`.
    #[must_use]
    pub fn caveats_for(&self, security: Security, exchange: Exchange) -> Vec<String> {
        caveats_for(&self.caveats, security, exchange)
    }
}

fn caveats_for(caveats: &[Caveat], security: Security, exchange: Exchange) -> Vec<String> {
    caveats
        .iter()
        .filter(|caveat| caveat.applies_to(security, exchange))
        .map(|caveat| caveat.text.clone())
        .collect()
}

impl Caveat {
    /// "Mutual fund on Tel Aviv", "Everything"
    #[must_use]
    pub fn coverage(&self) -> impl Display + '_ {
        fmt::from_fn(|f| {
            if self.securities.is_empty() && self.exchanges.is_empty() {
                f.write_str("Everything")
            } else {
                write!(
                    f,
                    "{} on {}",
                    list_or(&self.securities, "Anything"),
                    list_or(&self.exchanges, "any exchange"),
                )
            }
        })
    }
}

/// "ETF, Stock", or `any` when the list is empty (a tariff row that doesn't
/// limit it).
fn list_or<T: Display>(items: &[T], any: &str) -> String {
    if items.is_empty() {
        any.to_owned()
    } else {
        items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ils, iso, tariffs, usd};
    use rust_decimal_macros::dec;

    #[test]
    fn amounts_as_tariffs_write_them() {
        assert_eq!(format_money(usd(dec!(6750.00))), "$6,750");
        assert_eq!(format_money(usd(dec!(5.76))), "$5.76");
        assert_eq!(format_money(ils(dec!(3.5))), "₪3.5");
        assert_eq!(
            format_money(Money::from_decimal(dec!(2.5), iso::EUR)),
            "€2.5"
        );
    }

    #[test]
    fn leumi_online_for_a_us_etf() {
        let leumi = tariffs::leumi();
        let fees = leumi.describe_fees_for(&leumi.plans[0], Security::Etf, Exchange::Usa);
        let lines: Vec<_> = fees
            .fees
            .iter()
            .map(|fee| (fee.name.as_str(), fee.price.as_str()))
            .collect();
        assert_eq!(
            lines,
            [
                ("Buy or sell", "0.3%, min $24, max $6,750"),
                ("Custody", "0.2% a quarter (0.8% a year)"),
                ("Conversion", "0.16%, min $5.76, max $2,400"),
            ]
        );
        assert_eq!(fees.markup.unwrap().price, "not published");
        assert_eq!(fees.fees[1].hebrew_names, ["דמי משמרת"]);
    }

    #[test]
    fn caveats_follow_what_is_bought() {
        let leumi = tariffs::leumi();
        let pepper = leumi
            .plans
            .iter()
            .find(|plan| plan.name == "Pepper")
            .unwrap();
        let caveats =
            |security, exchange| leumi.describe_fees_for(pepper, security, exchange).caveats;

        let abroad = caveats(Security::Etf, Exchange::Usa);
        assert!(abroad.iter().any(|caveat| caveat.contains("markup")));
        assert!(abroad.iter().any(|caveat| caveat.starts_with("$4")));
        assert!(!abroad.iter().any(|caveat| caveat.starts_with("₪4")));

        let tel_aviv_etf = caveats(Security::Etf, Exchange::Tlv);
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.contains("onversion"))
        );
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.contains("index fund"))
        );

        let tel_aviv_fund = caveats(Security::MutualFund, Exchange::Tlv);
        assert!(
            tel_aviv_fund
                .iter()
                .any(|caveat| caveat.contains("index fund"))
        );
    }

    #[test]
    fn altshuler_custody_and_markup() {
        let plan = &tariffs::altshuler().plans[0];
        // Nothing is converted on Tel Aviv, so no conversion lines.
        let fees = plan.describe_fees_for(Security::Etf, Exchange::Tlv).fees;
        assert_eq!(fees.len(), 2);
        assert_eq!(
            fees[1].price,
            "0.15% a year, charged monthly, min ₪75 a month"
        );
        assert_eq!(plan.conversion.to_string(), "none");
        assert_eq!(plan.conversion.markup.to_string(), "up to 0.7%");
    }

    #[test]
    fn not_offered_coverage_and_dates() {
        let plan = &tariffs::altshuler().plans[0];
        let trade = &plan.describe_fees_for(Security::Etf, Exchange::Europe).fees[0];
        assert!(trade.missing);
        assert_eq!(trade.price, "not offered");
        assert_eq!(plan.trading[0].coverage().to_string(), "ETF on Tel Aviv");
        assert_eq!(
            purchase(Security::Etf, Exchange::Usa).to_string(),
            "an ETF bought in the USA"
        );
        assert_eq!(
            purchase(Security::MutualFund, Exchange::Tlv).to_string(),
            "a mutual fund bought in Tel Aviv"
        );
        assert_eq!(
            tariffs::leumi().tariff_date_text().to_string(),
            "Tariff of 29/06/2026"
        );
        assert_eq!(
            tariffs::altshuler().tariff_date_text().to_string(),
            "Tariff date not stated"
        );
    }
}
