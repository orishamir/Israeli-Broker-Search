//! The tariffs in plain words, for showing to users: "0.3%, min $24, max
//! $6,750", "0.15% a quarter (0.6% a year)". Kept in the core so every UI
//! says the same thing.
//!
//! A type's own text is its [`Display`] (derived with strum for the simple
//! enums, where they're declared); other ways to show it are methods
//! returning `impl Display`, built with [`fmt::from_fn`].

use std::fmt::{self, Display, Formatter};

use rust_decimal::Decimal;
use rusty_money::{Formatter as MoneyFormatter, Params};
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use crate::simulation::Outcome;
use crate::{
    Basis, Broker, Buying, Caveat, ConversionFee, CustodyFee, Errs, Exchange, ExchangeRates,
    HandlingFee, IntoEnumIterator, Markup, Money, PercentFee, Period, Plan, Price, Security,
    Source, TariffDate, TradeFee, tariffs,
};

/// A price in words, and whether it's nothing, so that every view dims the
/// same ones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct PriceText {
    /// "0.15%, min ₪3.5", "none", "not offered"
    pub text: String,
    /// True when nothing is charged, or no price is given.
    pub nothing: bool,
    /// For "not offered", the most general mismatch that's true, for its
    /// "?": "Nothing in Europe is offered", "No index funds are offered
    /// anywhere" (see [`Plan::not_offered_reason`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl PriceText {
    /// Why there's no price: "not offered".
    #[must_use]
    pub fn nothing(why: &str) -> Self {
        PriceText {
            text: why.to_owned(),
            nothing: true,
            reason: None,
        }
    }

    /// No price is given, and `reason` says how general the gap is.
    #[must_use]
    pub fn not_offered(reason: String) -> Self {
        PriceText {
            reason: Some(reason),
            ..PriceText::nothing("not offered")
        }
    }
}

/// A fee shown as a price: its [`Display`], and whether that says it's
/// nothing ("none").
pub trait Priced: Display {
    fn is_nothing(&self) -> bool {
        false
    }

    fn price_text(&self) -> PriceText {
        PriceText {
            text: self.to_string(),
            nothing: self.is_nothing(),
            reason: None,
        }
    }
}

impl Priced for Price {}

impl Priced for HandlingFee {}

impl Priced for PercentFee {
    fn is_nothing(&self) -> bool {
        self.is_free()
    }
}

impl Priced for ConversionFee {
    fn is_nothing(&self) -> bool {
        self.fee.is_free()
    }
}

impl Priced for CustodyFee {
    fn is_nothing(&self) -> bool {
        self.is_free()
    }
}

impl Priced for Markup {
    /// No markup, or none that's published.
    fn is_nothing(&self) -> bool {
        match self {
            Markup::UpTo(percent) => percent.is_zero(),
            Markup::PerDollar(_) => false,
            Markup::MarketRate | Markup::NotPublished => true,
        }
    }
}

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
                "A basket of many companies in one security, such as the 500 biggest in the \
                 US. You buy and sell it on the stock exchange like a share: at any moment of \
                 the trading day, at whatever price it's going for right then. Brokers charge \
                 it as they charge a share. Called תעודת\u{a0}סל until 2018."
            }
            Security::IndexFund => {
                "The same kind of basket, but not traded on the exchange. You buy it from the \
                 fund company through your broker, and sell it back the same way, at one price \
                 a day, set after the exchange closes; any amount will do, even ₪100. Brokers \
                 charge it as a fund, which at most of them is a different price from a \
                 share: on Altshuler's full tariff an ETF costs at least ₪3.5 a trade and an \
                 index fund at least ₪16, for the same S&P 500. Managed (active) and \
                 money-market funds aren't compared here."
            }
            Security::Bond => {
                "A loan you give to a government or a company: it pays you interest, say 4% a \
                 year, and you can sell it on the exchange before it's repaid. Short-term \
                 government bills (מק״מ) are priced separately and aren't compared."
            }
            Security::Stock => "A share in one company, such as Teva or Apple.",
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        match self {
            Security::Etf => &[
                "קרן סל",
                "קרן סל מחקה מדד",
                "קרן סל במסלול רציף",
                "תעודת סל",
            ],
            Security::IndexFund => &["קרן מחקה", "קרן נאמנות מחקה"],
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
                "The Tel Aviv Stock Exchange. Prices are in shekels, so nothing is converted. \
                 An S&P 500 ETF is sold here too, as an Israeli קרן\u{a0}סל."
            }
            Exchange::Usa => {
                "NYSE or Nasdaq. Prices are in dollars, so your shekels are converted, which \
                 some brokers charge for. The same S&P 500 ETF bought here, such as VOO, needs \
                 shekels turned into dollars, and back when you sell."
            }
            Exchange::Europe => {
                "A European exchange, such as Xetra or Euronext. Prices are in euros, so your \
                 shekels are converted, which some brokers charge for. An Irish-based S&P 500 \
                 ETF listed in Amsterdam, for example."
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

/// "ETFs", "index funds": what isn't offered.
fn plural(security: Security) -> &'static str {
    match security {
        Security::Etf => "ETFs",
        Security::IndexFund => "index funds",
        Security::Bond => "bonds",
        Security::Stock => "stocks",
    }
}

/// "in the USA", "in Tel Aviv": where something is bought.
fn place(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Tlv => "in Tel Aviv",
        Exchange::Usa => "in the USA",
        Exchange::Europe => "in Europe",
    }
}

/// "an ETF bought in the USA", "an index fund bought in Tel Aviv".
#[must_use]
pub fn purchase(security: Security, exchange: Exchange) -> impl Display {
    fmt::from_fn(move |f| {
        let security = match security {
            Security::Etf => "an ETF",
            Security::IndexFund => "an index fund",
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
/// In the order the editor shows them: `FeeKind::iter()`. A standing order,
/// the second conversion fee and the markup are shown under the fee they're
/// part of ([`FeeKind::label`]).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumIter,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum FeeKind {
    #[strum(to_string = "Buy or sell")]
    Trade,
    Track,
    #[strum(to_string = "Standing order")]
    StandingOrder,
    Custody,
    #[strum(to_string = "Handling fee")]
    Handling,
    Conversion,
    #[strum(to_string = "Second conversion fee")]
    SecondConversion,
    #[strum(to_string = "Conversion markup")]
    Markup,
}

impl FeeKind {
    /// Its name under the fee it's part of: "markup", "by standing order".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            FeeKind::Trade => "buy or sell",
            FeeKind::Track => "track picked for you",
            FeeKind::StandingOrder => "by standing order",
            FeeKind::Custody => "custody",
            FeeKind::Handling => "handling fee",
            FeeKind::Conversion => "conversion",
            FeeKind::SecondConversion => "or, if less",
            FeeKind::Markup => "markup",
        }
    }
}

impl Explained for FeeKind {
    fn explanation(&self) -> &'static str {
        match self {
            FeeKind::Trade => {
                "Charged on every purchase and sale, including selling everything at the end. \
                 Usually a share of the trade with a minimum: at 0.07% with a ₪3 minimum, a \
                 ₪2,000 purchase is charged the ₪3, since 0.07% of it is only ₪1.40."
            }
            FeeKind::Track => {
                "Some brokers let you choose, when you open the account, how trades abroad \
                 are priced: per share, per order or as a share of the trade. Altshuler, for \
                 example: 1¢ a share, $11 an order, or 0.15% of the trade. The app picks the \
                 cheapest for your inputs; ask for it when you open the account."
            }
            FeeKind::StandingOrder => {
                "An instruction to buy the same amount every month, automatically. Some \
                 brokers charge less for these purchases: Leumi charges 0.225% instead of \
                 0.4% for an index fund bought this way. A one-time deposit and selling cost \
                 the usual fee."
            }
            FeeKind::Custody => {
                "Charged for holding your securities, as a share of what they're worth, \
                 whether or not you trade: 0.6% a year on ₪100,000 is ₪600, every year."
            }
            FeeKind::Handling => {
                "A fee for keeping the account, charged every month whether or not you trade: \
                 ₪15 a month is ₪180 a year. New customers often get it free for a while, and \
                 some brokers take that month's trade fees off it, so a month with ₪15 of \
                 purchases pays nothing more."
            }
            FeeKind::Conversion => {
                "Charged for converting your shekels to the security's currency, and back \
                 when you sell."
            }
            FeeKind::SecondConversion => {
                "A second price for converting that you also get, such as a customer \
                 group's next to the online one. Discounts don't add up, so each conversion \
                 costs whichever is less."
            }
            FeeKind::Markup => {
                "The Currency Conversion Markup: the broker converts at a rate worse than \
                 the market's by this much. It isn't listed as a fee, but it costs the same: \
                 at a market rate of ₪3.50 a dollar, a 0.7% markup means paying ₪3.52, which \
                 is ₪14 on ₪2,000."
            }
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        match self {
            FeeKind::Trade => &["עמלת קנייה/מכירה"],
            FeeKind::Track => &["מסלול עמלות"],
            FeeKind::StandingOrder => &["הוראת קבע"],
            FeeKind::Custody => &["דמי משמרת"],
            // As the tariffs name it: IBI and Meitav, Altshuler.
            FeeKind::Handling => &["דמי טיפול", "דמי שימוש", "דמי ניהול תקופתיים"],
            FeeKind::Conversion => &["עמלת המרת מט\"ח"],
            FeeKind::SecondConversion => &[],
            FeeKind::Markup => &["מרווח המרה"],
        }
    }
}

/// A caveat's kind, as a plan's details label it: its [`Basis`] as the user
/// sees it, most serious first. Its `Display` is the label: "Our reading".
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, strum::Display, strum::EnumIter,
)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum CaveatKind {
    #[strum(to_string = "Assumed, may cost more")]
    MayCostMore,
    #[strum(to_string = "Assumed, at most")]
    AtMost,
    #[strum(to_string = "Our reading")]
    Reading,
    #[strum(to_string = "Not counted")]
    NotCounted,
    #[strum(to_string = "As published")]
    Published,
}

impl From<&Basis> for CaveatKind {
    fn from(basis: &Basis) -> Self {
        match basis {
            Basis::Published => CaveatKind::Published,
            Basis::Reading { .. } => CaveatKind::Reading,
            Basis::Assumed { errs: Errs::AtMost } => CaveatKind::AtMost,
            Basis::Assumed {
                errs: Errs::MayCostMore { .. },
            } => CaveatKind::MayCostMore,
            Basis::NotCounted => CaveatKind::NotCounted,
        }
    }
}

impl Explained for CaveatKind {
    fn explanation(&self) -> &'static str {
        match self {
            CaveatKind::MayCostMore => {
                "Nothing is published to go on, so a stand-in was used that errs on the \
                 cheap side: the plan may cost more than shown. A bank that doesn't publish \
                 its conversion markup, for example, is counted as converting at the market \
                 rate. If you know the real number, change these fees with ✎."
            }
            CaveatKind::AtMost => {
                "Nothing is published to go on, so the tariff's full price or maximum is \
                 used: the plan can only be cheaper than shown. An offer that doesn't \
                 mention bonds, for example, gets the full tariff's bond price. Ask the \
                 broker what it really charges."
            }
            CaveatKind::Reading => {
                "The tariff is unclear or silent here. This is how it was read, and what \
                 supports the reading: a custody rate with no period stated was read as \
                 yearly, for example, because the exchange's data on what customers pay \
                 says so."
            }
            CaveatKind::NotCounted => {
                "A real cost the app leaves out, and why: too small or too rare to change a \
                 long-term comparison, or a one-off, such as a ₪200 gift for opening the \
                 account."
            }
            CaveatKind::Published => {
                "What the tariff or the broker's site says. Nothing is uncertain; it's worth \
                 knowing before you choose."
            }
        }
    }

    fn hebrew_names(&self) -> &'static [&'static str] {
        &[]
    }
}

/// A caveat in words, for a plan's details.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct CaveatText {
    pub text: String,
    pub kind: CaveatKind,
    /// The kind's label, for one shown outside its group: "Our reading".
    pub label: String,
    /// For a reading, what supports it.
    pub support: Option<String>,
    /// The pages it rests on, to link beside it.
    pub sources: Vec<Source>,
    /// What it's about, for one shown among caveats about other choices:
    /// "Index fund on Tel Aviv", "Orders above $8,000".
    pub covers: String,
}

impl From<&Caveat> for CaveatText {
    fn from(caveat: &Caveat) -> Self {
        let kind = CaveatKind::from(&caveat.basis);
        CaveatText {
            text: caveat.text.clone(),
            kind,
            label: kind.to_string(),
            support: match &caveat.basis {
                Basis::Reading { support } => Some(support.clone()),
                _ => None,
            },
            sources: caveat.sources.clone(),
            covers: caveat.coverage().to_string(),
        }
    }
}

/// The caveats of one kind, under its label.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct CaveatGroup {
    pub kind: CaveatKind,
    /// "Assumed, may cost more"
    pub label: String,
    /// What the label means, for its "?".
    pub explanation: String,
    pub caveats: Vec<CaveatText>,
}

/// `caveats` sorted for a plan's details: the ones that matter to `buying`,
/// grouped by kind, most serious first, and the rest, each saying what it's
/// about.
fn sort_caveats(
    caveats: &[&Caveat],
    buying: Buying,
    rates: &ExchangeRates,
) -> (Vec<CaveatGroup>, Vec<CaveatText>) {
    let (matter, others): (Vec<&Caveat>, Vec<&Caveat>) = caveats
        .iter()
        .partition(|caveat| caveat.matters_for(buying, rates));
    let groups = CaveatKind::iter()
        .filter_map(|kind| {
            let texts: Vec<CaveatText> = matter
                .iter()
                .filter(|caveat| CaveatKind::from(&caveat.basis) == kind)
                .map(|&caveat| CaveatText::from(caveat))
                .collect();
            (!texts.is_empty()).then(|| CaveatGroup {
                kind,
                label: kind.to_string(),
                explanation: kind.explanation().to_owned(),
                caveats: texts,
            })
        })
        .collect();
    let others = others.into_iter().map(CaveatText::from).collect();
    (groups, others)
}

/// One fee, named and explained, and what a plan charges for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeeLine {
    pub kind: FeeKind,
    pub name: String,
    /// Its name when it's shown under another fee: "markup".
    pub label: String,
    pub explanation: String,
    pub hebrew_names: Vec<String>,
    /// "0.15%, min ₪3.5", or that it's nothing: "not offered", "none".
    pub price: PriceText,
    /// Shown under it, as part of it: a standing order's price under buying,
    /// the second fee and the markup under conversion.
    pub parts: Vec<FeeLine>,
    /// Beside the price, if a caveat is about this fee.
    pub mark: Option<Mark>,
}

/// A mark beside a fee's price: how sure the number is, in a word or two,
/// and the caveats that say why.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Mark {
    /// The most serious kind among the caveats.
    pub kind: CaveatKind,
    /// "may cost more"
    pub text: String,
    pub caveats: Vec<CaveatText>,
}

impl CaveatKind {
    /// The kind beside a price, in a word or two: "may cost more".
    #[must_use]
    pub fn mark_text(self) -> &'static str {
        match self {
            CaveatKind::MayCostMore => "may cost more",
            CaveatKind::AtMost => "at most",
            CaveatKind::Reading => "our reading",
            CaveatKind::NotCounted => "not counted",
            CaveatKind::Published => "as published",
        }
    }
}

impl FeeLine {
    fn new(kind: FeeKind, price: PriceText) -> Self {
        FeeLine {
            kind,
            name: kind.to_string(),
            label: kind.label().to_owned(),
            explanation: kind.explanation().to_owned(),
            hebrew_names: kind
                .hebrew_names()
                .iter()
                .map(|&name| name.to_owned())
                .collect(),
            price,
            parts: vec![],
            mark: None,
        }
    }

    /// Puts each of `caveats` beside the fee it's about, in this line or
    /// its parts, marking the line with the most serious kind among them.
    fn attach(&mut self, caveats: &[&Caveat]) {
        let about_this: Vec<CaveatText> = caveats
            .iter()
            .filter(|caveat| caveat.fee == Some(self.kind))
            .map(|&caveat| CaveatText::from(caveat))
            .collect();
        self.mark = about_this
            .iter()
            .map(|caveat| caveat.kind)
            .min()
            .map(|kind| Mark {
                kind,
                text: kind.mark_text().to_owned(),
                caveats: about_this,
            });
        for part in &mut self.parts {
            part.attach(caveats);
        }
    }

    /// A price that's never nothing: a track's name, a standing order's.
    fn part(kind: FeeKind, price: String) -> Self {
        FeeLine::new(
            kind,
            PriceText {
                text: price,
                nothing: false,
                reason: None,
            },
        )
    }
}

/// What a plan charges for one security on one exchange, in words, and the
/// caveats that apply to it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct FeesFor {
    /// Conversion only abroad: nothing is converted on Tel Aviv. Its parts
    /// include the markup, the other half of what converting costs.
    pub fees: Vec<FeeLine>,
    /// The caveats that matter to the purchase, grouped by kind, most
    /// serious first.
    pub caveats: Vec<CaveatGroup>,
    /// The rest: about other securities, exchanges or amounts, each saying
    /// which.
    pub others: Vec<CaveatText>,
}

impl Plan {
    /// What the plan charges for `buying`, on `track` if the comparison
    /// picked one of its tracks, with `broker_caveats` (the ones about every
    /// plan of the broker) before the plan's own.
    /// Every page its caveats rest on, each once, in the order first named.
    #[must_use]
    pub fn sources(&self) -> Vec<Source> {
        each_once(self.caveats.iter().flat_map(|caveat| &caveat.sources))
    }

    /// Why `security` on `exchange` has no price, in the most general terms
    /// that are true: "Nothing in Europe is offered" when nothing there is,
    /// "No index funds are offered anywhere" when the security isn't, and
    /// otherwise "No index funds are offered in Europe".
    #[must_use]
    pub fn not_offered_reason(&self, security: Security, exchange: Exchange) -> String {
        let rows = || {
            self.trading
                .iter()
                .chain(self.tracks.iter().flat_map(|track| &track.trading))
        };
        let anything_there =
            rows().any(|row| Security::iter().any(|each| row.applies_to(each, exchange)));
        let anywhere =
            rows().any(|row| Exchange::iter().any(|each| row.applies_to(security, each)));
        let (securities, place) = (plural(security), place(exchange));
        match (anything_there, anywhere) {
            (false, _) => format!("Nothing {place} is offered"),
            (true, false) => format!("No {securities} are offered anywhere"),
            (true, true) => format!("No {securities} are offered {place}"),
        }
    }

    pub fn describe_fees_for(
        &self,
        buying: Buying,
        track: Option<usize>,
        broker_caveats: &[Caveat],
        rates: &ExchangeRates,
    ) -> FeesFor {
        let Buying {
            security, exchange, ..
        } = buying;
        let row_for = |rows: &[TradeFee]| {
            rows.iter()
                .find(|row| row.applies_to(security, exchange))
                .map(|row| row.price.to_string())
        };
        // The tracks that price this trade, with their prices.
        let tracks: Vec<(usize, String)> = self
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(index, each)| Some((index, row_for(&each.trading)?)))
            .collect();
        let picked = tracks.iter().find(|(index, _)| Some(*index) == track);
        let price = match (picked, tracks.is_empty()) {
            (Some((_, price)), _) => Some(price.clone()),
            (None, true) => row_for(&self.trading),
            (None, false) => Some("one of the tracks below".to_owned()),
        };
        let mut trade = FeeLine::new(
            FeeKind::Trade,
            price.clone().map_or_else(
                || PriceText::not_offered(self.not_offered_reason(security, exchange)),
                |text| PriceText {
                    text,
                    nothing: false,
                    reason: None,
                },
            ),
        );
        if !tracks.is_empty() {
            let others: Vec<&str> = tracks
                .iter()
                .filter(|(index, _)| Some(*index) != track)
                .map(|(_, price)| price.as_str())
                .collect();
            let mut part = FeeLine::part(FeeKind::Track, others.join("; "));
            match picked {
                Some(&(index, _)) if others.is_empty() => {
                    self.tracks[index].name.clone_into(&mut part.price.text);
                }
                Some(&(index, _)) => {
                    let name = &self.tracks[index].name;
                    part.price.text = format!("{name} (others: {})", part.price.text);
                }
                None => "tracks".clone_into(&mut part.label),
            }
            trade.parts.push(part);
        }
        // A standing order at the usual price isn't worth a line.
        if let Some(by_standing_order) = row_for(&self.standing_orders)
            && Some(&by_standing_order) != price.as_ref()
        {
            trade
                .parts
                .push(FeeLine::part(FeeKind::StandingOrder, by_standing_order));
        }
        let custody = FeeLine::new(
            FeeKind::Custody,
            self.custody_row(security, exchange)
                .map_or_else(|| PriceText::nothing("none"), Priced::price_text),
        );
        let mut fees = vec![trade, custody];
        if let Some(handling) = self.handling {
            fees.push(FeeLine::new(FeeKind::Handling, handling.price_text()));
        }
        if exchange != Exchange::Tlv {
            let conversion = &self.conversion;
            let mut line = FeeLine::new(FeeKind::Conversion, conversion.price_text());
            if let Some(second) = conversion.or_if_less {
                line.parts
                    .push(FeeLine::new(FeeKind::SecondConversion, second.price_text()));
            }
            if let Some(by_standing_order) = &self.standing_order_conversion {
                line.parts.push(FeeLine::new(
                    FeeKind::StandingOrder,
                    by_standing_order.price_text(),
                ));
            }
            let markup = conversion.markup.price_text();
            line.parts.push(FeeLine::new(FeeKind::Markup, markup));
            fees.push(line);
        }
        let caveats: Vec<&Caveat> = broker_caveats.iter().chain(&self.caveats).collect();
        let (groups, others) = sort_caveats(&caveats, buying, rates);
        let mattering: Vec<&Caveat> = caveats
            .iter()
            .copied()
            .filter(|caveat| caveat.matters_for(buying, rates))
            .collect();
        for fee in &mut fees {
            fee.attach(&mattering);
        }
        FeesFor {
            fees,
            caveats: groups,
            others,
        }
    }

    /// Why the plan may cost more than shown for `buying`, if a caveat says
    /// so: "May cost more: conversion markup not published". Flagged in the
    /// comparison, since such a plan may rank better than it should.
    #[must_use]
    pub fn may_cost_more(
        &self,
        buying: Buying,
        broker_caveats: &[Caveat],
        rates: &ExchangeRates,
    ) -> Option<String> {
        let mut summaries: Vec<&str> = broker_caveats
            .iter()
            .chain(&self.caveats)
            .filter(|caveat| caveat.matters_for(buying, rates))
            .filter_map(Caveat::may_cost_more_summary)
            .collect();
        summaries.dedup();
        (!summaries.is_empty()).then(|| format!("May cost more: {}", summaries.join("; ")))
    }

    /// "US track: 1¢ a share, the cheapest for you", if `track` is one of the
    /// plan's tracks and prices `security` on `exchange`.
    #[must_use]
    pub fn track_note(
        &self,
        security: Security,
        exchange: Exchange,
        track: Option<usize>,
    ) -> Option<String> {
        let track = self.track_for(track?, security, exchange)?;
        let place = match exchange {
            Exchange::Tlv => "Tel Aviv",
            Exchange::Usa => "US",
            Exchange::Europe => "European",
        };
        Some(format!(
            "{place} track: {}, the cheapest for you",
            track.name
        ))
    }

    /// "Needs a one-time deposit of at least ₪5,000", if `first_deposit` is less
    /// than the plan's minimum.
    #[must_use]
    pub fn first_deposit_warning(&self, first_deposit: Money) -> Option<String> {
        self.min_first_deposit
            .filter(|min| first_deposit.amount() < min.amount())
            .map(|min| format!("Needs a one-time deposit of at least {}", format_money(min)))
    }

    /// Why the plan's standing order price isn't used, if it has one for
    /// `security` on `exchange` and purchases are less often than monthly.
    #[must_use]
    pub fn standing_order_note(
        &self,
        security: Security,
        exchange: Exchange,
        buy_every_months: u32,
    ) -> Option<&'static str> {
        (buy_every_months != 1 && self.standing_order_row(security, exchange).is_some())
            .then_some("A standing order buys every month, so it isn't used here")
    }
}

impl Outcome {
    /// "Its fees are more than you deposit", when they are: then its numbers
    /// go below zero, as the fees become a debt.
    #[must_use]
    pub fn warning(&self, deposited: Decimal) -> Option<&'static str> {
        (self.fees.total() > deposited).then_some("Its fees are more than you deposit")
    }
}

impl Broker {
    /// "Tariff of 29/06/2026", "Tariff of 01/2025", or that the tariff isn't
    /// dated.
    #[must_use]
    pub fn tariff_date_text(&self) -> impl Display + '_ {
        fmt::from_fn(|f| {
            let date = match self.tariff_date {
                Some(TariffDate::Day(date)) => {
                    date.format(format_description!("[day]/[month]/[year]"))
                }
                Some(TariffDate::Month(date)) => date.format(format_description!("[month]/[year]")),
                None => return f.write_str("Tariff date not stated"),
            };
            write!(f, "Tariff of {}", date.map_err(|_| fmt::Error)?)
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
            Price::PercentPlusPerShare {
                percent,
                per_share,
                min,
                max,
            } => {
                write!(f, "{percent} + {} per share", format_money(*per_share))?;
                write_bounds(f, *min, *max)
            }
        }
    }
}

impl CustodyFee {
    /// What the row covers: "Tel Aviv", "Any exchange", "Index fund on Tel
    /// Aviv".
    #[must_use]
    pub fn coverage(&self) -> impl Display + '_ {
        fmt::from_fn(|f| {
            if self.securities.is_empty() {
                f.write_str(&list_or(&self.exchanges, "Any exchange"))
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

/// "₪15 a month, free for the first 2 years, less that month's trade fees".
impl Display for HandlingFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} a month", format_money(self.per_month))?;
        match self.free_months {
            0 => {}
            12 => f.write_str(", free for the first year")?,
            months if months % 12 == 0 => write!(f, ", free for the first {} years", months / 12)?,
            months => write!(f, ", free for the first {months} months")?,
        }
        if self.less_trade_fees {
            f.write_str(", less that month's trade fees")?;
        }
        Ok(())
    }
}

/// "0.15% a quarter (0.6% a year)", "0.15% a year, charged monthly, min ₪75 a
/// month", or "none" if it never charges anything.
impl Display for CustodyFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_free() {
            return f.write_str("none");
        }
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

/// "0.16%, min $5.76, max $2,400", or "none" if it never charges anything.
impl Display for PercentFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_free() {
            return f.write_str("none");
        }
        write!(f, "{}", self.percent)?;
        write_bounds(f, self.min, self.max)
    }
}

/// The listed fee alone, without the second fee or the markup.
impl Display for ConversionFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fee.fmt(f)
    }
}

/// "up to 0.7%", "none", "not published"
impl Display for Markup {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Markup::UpTo(percent) if percent.is_zero() => f.write_str("none"),
            Markup::UpTo(percent) => write!(f, "up to {percent}"),
            Markup::PerDollar(amount) => write!(f, "{} per dollar", format_money(*amount)),
            Markup::MarketRate => f.write_str("at the market rate"),
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

/// "except Index fund on Tel Aviv (its own row above)", for a row of your
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
    /// What plan `plan` charges for `buying`, with the broker's caveats
    /// first.
    #[must_use]
    pub fn describe_fees_for(
        &self,
        plan: &Plan,
        buying: Buying,
        track: Option<usize>,
        rates: &ExchangeRates,
    ) -> FeesFor {
        plan.describe_fees_for(buying, track, &self.caveats, rates)
    }

    /// Its tariff document, then every page its caveats and its plans' rest
    /// on, each once.
    #[must_use]
    pub fn sources(&self) -> Vec<Source> {
        self.sources_of(self.plans.iter().flat_map(|plan| &plan.caveats))
    }

    /// The pages behind `plan`'s numbers: the tariff document, then what the
    /// broker-wide caveats and the plan's own rest on, each once.
    #[must_use]
    pub fn sources_for(&self, plan: &Plan) -> Vec<Source> {
        self.sources_of(plan.caveats.iter())
    }

    fn sources_of<'a>(&'a self, caveats: impl Iterator<Item = &'a Caveat>) -> Vec<Source> {
        let tariff = self.source_url.iter().map(|url| Source {
            name: "Tariff (PDF)".to_owned(),
            url: url.clone(),
        });
        let mut sources = tariff.collect::<Vec<Source>>();
        let caveats = self.caveats.iter().chain(caveats);
        sources.extend(caveats.flat_map(|caveat| caveat.sources.iter().cloned()));
        each_once(sources.iter())
    }

    /// The broker-wide caveats that matter to `buying`, grouped by kind.
    #[must_use]
    pub fn caveats_for(&self, buying: Buying, rates: &ExchangeRates) -> Vec<CaveatGroup> {
        let caveats: Vec<&Caveat> = self.caveats.iter().collect();
        sort_caveats(&caveats, buying, rates).0
    }

    /// Why `plan` may cost more than shown for `buying`, if a caveat of the
    /// broker's or the plan's says so (see [`Plan::may_cost_more`]).
    #[must_use]
    pub fn may_cost_more(
        &self,
        plan: &Plan,
        buying: Buying,
        rates: &ExchangeRates,
    ) -> Option<String> {
        plan.may_cost_more(buying, &self.caveats, rates)
    }

    /// "Checked 29/09/2026": when the numbers were last checked against the
    /// brokers' documents and sites.
    #[must_use]
    pub fn checked_text() -> String {
        let date = tariffs::checked()
            .format(format_description!("[day]/[month]/[year]"))
            .expect("a fixed format");
        format!("Checked {date}")
    }
}

impl Caveat {
    /// "Index fund on Tel Aviv", "Everything", "Stock, ETF on USA, orders
    /// above $8,000"
    #[must_use]
    pub fn coverage(&self) -> impl Display + '_ {
        fmt::from_fn(|f| {
            match (
                self.securities.is_empty() && self.exchanges.is_empty(),
                self.above,
            ) {
                (true, None) => f.write_str("Everything")?,
                (true, Some(above)) => write!(f, "Orders above {}", format_money(above))?,
                (false, above) => {
                    write!(
                        f,
                        "{} on {}",
                        list_or(&self.securities, "Anything"),
                        list_or(&self.exchanges, "any exchange"),
                    )?;
                    if let Some(above) = above {
                        write!(f, ", orders above {}", format_money(above))?;
                    }
                }
            }
            Ok(())
        })
    }
}

// ─────────────────────────── About the numbers ───────────────────────────

/// How the numbers are made, what isn't counted and where the numbers come
/// from, for the page that explains the app. Plain text, section by section.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct About {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Section {
    pub title: String,
    pub paragraphs: Vec<String>,
    /// A list after the paragraphs, if the section has one.
    pub items: Vec<Item>,
    /// Pages to link, in groups, if the section is the sources.
    pub sources: Vec<SourceGroup>,
}

/// The pages behind one broker's numbers, or one kind of source: "Bank
/// Leumi · Tariff of 29/06/2026".
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct SourceGroup {
    pub title: String,
    pub sources: Vec<Source>,
}

/// `sources`, each URL once, in the order first seen.
fn each_once<'a>(sources: impl Iterator<Item = &'a Source>) -> Vec<Source> {
    let mut seen: Vec<Source> = vec![];
    for source in sources {
        if !seen.iter().any(|known| known.url == source.url) {
            seen.push(source.clone());
        }
    }
    seen
}

/// One item of a section's list, linked if it has somewhere to go.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub struct Item {
    pub text: String,
    pub url: Option<String>,
}

fn item(text: &str) -> Item {
    Item {
        text: text.to_owned(),
        url: None,
    }
}

/// The page that explains the numbers: what the app does with your inputs,
/// how sure it is of each price, what it leaves out and why, and its sources.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the page's text")]
pub fn about() -> About {
    let method = Section {
        title: "How the numbers are made".to_owned(),
        paragraphs: vec![
            "Each plan is run month by month on your deposits. Money arrives at the start \
             of the month and waits as shekels until the next purchase, which converts it \
             (abroad) and buys with it, whole shares only where the broker sells no \
             fractions. Custody and the handling fee are paid every month out of the \
             shekels. At the end everything is sold and converted back, and that's the \
             value the table ranks by."
                .to_owned(),
            "The return is the security's own, in its own currency; today's exchange rates \
             stay as they are, and money waiting for a purchase earns nothing. A plan with \
             several price tracks costs what its cheapest does for your inputs, and the \
             track is named."
                .to_owned(),
            "Banks publish what they charge. Investment houses publish only a full tariff, \
             the most they may charge, and offer new customers far less by phone. Their \
             \u{201c}Typical offer\u{201d} plan is what comparison sites list for joining, \
             and wherever the offer is silent the full tariff's price is used: a plan is \
             never shown cheaper than its documents allow."
                .to_owned(),
            "Every plan's details say how sure each number is. As published: the tariff or \
             the broker's site says so. Our reading: the tariff is unclear or silent, and \
             this is how it was read, with what supports it. Assumed: a stand-in, either at \
             most (the full price, so the plan can only be cheaper) or may cost more (the \
             cheap side, which the comparison flags). Not counted: a real cost left out, and \
             why."
                .to_owned(),
        ],
        items: vec![],
        sources: vec![],
    };
    let left_out = Section {
        title: "What isn't counted".to_owned(),
        paragraphs: vec![
            "Costs every broker has that the app leaves out, and why. Each plan's own gaps \
             are in its details, under \u{201c}Not counted\u{201d}."
                .to_owned(),
        ],
        sources: vec![],
        items: vec![
            item("Taxes: they don't depend on the broker."),
            item(
                "Dividends, and the fees some brokers take on them (Altshuler and Meitav \
                 0.3% of the payment): the return is taken as total return, dividends \
                 reinvested, and 0.3% of a dividend is a few thousandths of a percent a \
                 year.",
            ),
            item(
                "Third-party fees on US trades (SEC, FINRA, exchange fees): fractions of a \
                 cent a share, passed on by every broker alike.",
            ),
            item("Real-time quotes and advanced trading systems: optional extras."),
            item(
                "Joining gifts and refunds (₪100–₪300, or a refund of early commissions): \
                 one-off, and named in each plan's details.",
            ),
            item(
                "Interest on credit or on idle cash: the app keeps no debt, and money \
                 waiting for a purchase earns nothing.",
            ),
            item(
                "Moving securities to another broker (₪20–₪35 a security, $10–$40 abroad) \
                 and cancelled orders: once, if ever.",
            ),
        ],
    };
    let mut groups: Vec<SourceGroup> = tariffs::all()
        .iter()
        .map(|broker| SourceGroup {
            title: format!("{} · {}", broker.name, broker.tariff_date_text()),
            sources: broker.sources(),
        })
        .collect();
    groups.push(SourceGroup {
        title: "Comparison sites, for the typical offers and the investment houses' \
                conversion markups (September 2026)"
            .to_owned(),
        sources: vec![
            tariffs::gemeltop_comparison(),
            tariffs::tradingil_comparison(),
            tariffs::tradingil_conversions(),
            tariffs::broker_co_il(),
        ],
    });
    groups.push(SourceGroup {
        title: "The Tel Aviv Stock Exchange: its members' tariffs and actual average fees \
                (June 2026), which confirm the offers and settled unclear rows"
            .to_owned(),
        sources: vec![tariffs::exchange_calculator()],
    });
    let sources = Section {
        title: "Sources".to_owned(),
        paragraphs: vec![format!(
            "{} against each broker's tariff document and site. Tariffs change several \
             times a year: each plan's details link to its document, and each caveat to \
             the page it rests on.",
            Broker::checked_text()
        )],
        items: vec![],
        sources: groups,
    };
    About {
        sections: vec![method, left_out, sources],
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

    fn rates() -> ExchangeRates {
        ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
    }

    fn buying(security: Security, exchange: Exchange) -> Buying {
        Buying::any_amount(security, exchange)
    }

    fn pepper() -> (Broker, Plan) {
        let leumi = tariffs::leumi();
        let pepper = leumi
            .plans
            .iter()
            .find(|plan| plan.name == "Pepper")
            .unwrap()
            .clone();
        (leumi, pepper)
    }

    #[test]
    fn leumi_online_for_a_us_etf() {
        let leumi = tariffs::leumi();
        let fees = leumi.describe_fees_for(
            &leumi.plans[0],
            buying(Security::Etf, Exchange::Usa),
            None,
            &rates(),
        );
        let lines: Vec<_> = fees
            .fees
            .iter()
            .map(|fee| (fee.name.as_str(), fee.price.text.as_str()))
            .collect();
        assert_eq!(
            lines,
            [
                ("Buy or sell", "0.3%, min $24, max $6,750"),
                ("Custody", "0.2% a quarter (0.8% a year)"),
                ("Conversion", "0.16%, min $5.76, max $2,400"),
            ]
        );
        let parts: Vec<_> = fees.fees[2]
            .parts
            .iter()
            .map(|part| (part.label.as_str(), part.price.text.as_str()))
            .collect();
        assert_eq!(parts, [("markup", "up to 0.9%")]);
        assert_eq!(fees.fees[1].hebrew_names, ["דמי משמרת"]);

        // The markup, measured from the bank's published rates, is marked
        // beside its price as a reading, and its caveat heads the list, under
        // the label that says what it means.
        let mark = fees.fees[2].parts[0].mark.as_ref().unwrap();
        assert_eq!(mark.kind, CaveatKind::Reading);
        assert_eq!(mark.text, "our reading");
        assert_eq!(mark.caveats.len(), 1);
        assert!(mark.caveats[0].support.is_some());
        assert_eq!(fees.fees[0].mark, None);
        assert_eq!(fees.caveats[0].kind, CaveatKind::Reading);
        assert_eq!(fees.caveats[0].label, "Our reading");
        assert!(fees.caveats[0].explanation.contains("unclear or silent"));
        assert_eq!(fees.caveats[0].caveats[0].text, mark.caveats[0].text);
    }

    #[test]
    fn second_prices_are_parts_of_their_fee() {
        let leumi = tariffs::leumi();
        let plan = |name| leumi.plans.iter().find(|plan| plan.name == name).unwrap();
        let parts = |fee: &FeeLine| -> Vec<(String, String)> {
            let each = |part: &FeeLine| (part.label.clone(), part.price.text.clone());
            fee.parts.iter().map(each).collect()
        };
        let describe = |plan: &Plan, security, exchange| {
            plan.describe_fees_for(buying(security, exchange), None, &[], &rates())
        };

        let standing_order = plan("Online, monthly standing order");
        let fund = describe(standing_order, Security::IndexFund, Exchange::Tlv);
        assert_eq!(fund.fees[0].price.text, "0.4%, min ₪26, max ₪6,300");
        assert_eq!(
            parts(&fund.fees[0]),
            [(
                "by standing order".into(),
                "0.225%, min ₪5, max ₪6,300".into()
            )]
        );
        let etf = describe(standing_order, Security::Etf, Exchange::Tlv);
        assert_eq!(etf.fees[0].parts, []);

        let plus18 = describe(plan("Online, 'Leumi 18+'"), Security::Etf, Exchange::Usa);
        assert_eq!(plus18.fees[2].price.text, "0.1%, min $7.2, max $3,000");
        assert_eq!(
            parts(&plus18.fees[2]),
            [
                ("or, if less".into(), "0.16%, min $5.76, max $2,400".into()),
                ("markup".into(), "up to 0.9%".into()),
            ]
        );
    }

    #[test]
    fn a_standing_order_needs_monthly_purchases() {
        let leumi = tariffs::leumi();
        let plan = &leumi.plans[2];
        let note = |security, every| plan.standing_order_note(security, Exchange::Tlv, every);
        assert_eq!(note(Security::IndexFund, 1), None);
        assert!(note(Security::IndexFund, 3).is_some());
        assert_eq!(note(Security::Etf, 3), None); // it has no standing order for ETFs
    }

    #[test]
    fn caveats_follow_what_is_bought_and_how_much() {
        let (leumi, pepper) = pepper();
        let caveats = |security, exchange, largest_trade| -> Vec<String> {
            let buying = Buying {
                security,
                exchange,
                largest_trade,
            };
            leumi
                .describe_fees_for(&pepper, buying, None, &rates())
                .caveats
                .iter()
                .flat_map(|group| group.caveats.iter().map(|caveat| caveat.text.clone()))
                .collect()
        };

        // With the biggest order unknown, a caveat about large orders shows.
        let abroad = caveats(Security::Etf, Exchange::Usa, None);
        assert!(abroad.iter().any(|caveat| caveat.contains("markup")));
        assert!(abroad.iter().any(|caveat| caveat.starts_with("$4")));
        assert!(!abroad.iter().any(|caveat| caveat.starts_with("₪4")));
        // $4 is stated up to $8,000: €7,000 is more than that, $7,000 isn't.
        let small = caveats(Security::Etf, Exchange::Usa, Some(usd(dec!(7000))));
        assert!(!small.iter().any(|caveat| caveat.starts_with("$4")));
        let euros = Money::from_decimal(dec!(7000), iso::EUR);
        let big = caveats(Security::Etf, Exchange::Europe, Some(euros));
        assert!(big.iter().any(|caveat| caveat.starts_with("$4")));

        let tel_aviv_etf = caveats(Security::Etf, Exchange::Tlv, None);
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.contains("onversion"))
        );
        assert!(
            !tel_aviv_etf
                .iter()
                .any(|caveat| caveat.to_lowercase().contains("index fund"))
        );

        let tel_aviv_fund = caveats(Security::IndexFund, Exchange::Tlv, None);
        assert!(
            tel_aviv_fund
                .iter()
                .any(|caveat| caveat.to_lowercase().contains("index fund"))
        );
    }

    #[test]
    fn the_rest_say_what_they_are_about() {
        let (leumi, pepper) = pepper();
        let buying = Buying {
            security: Security::Etf,
            exchange: Exchange::Usa,
            largest_trade: Some(usd(dec!(1000))),
        };
        let fees = leumi.describe_fees_for(&pepper, buying, None, &rates());
        let covers: Vec<&str> = fees
            .others
            .iter()
            .map(|caveat| caveat.covers.as_str())
            .collect();
        assert!(covers.contains(&"Anything on Tel Aviv, orders above ₪30,000"));
        assert!(covers.contains(&"Anything on USA, Europe, orders above $8,000"));
        assert!(covers.contains(&"Index fund on Tel Aviv"));
        // What matters: the markup's reading, then Pepper's package and its
        // custody note.
        let kinds: Vec<CaveatKind> = fees.caveats.iter().map(|group| group.kind).collect();
        assert_eq!(
            kinds,
            [
                CaveatKind::Reading,
                CaveatKind::NotCounted,
                CaveatKind::Published
            ]
        );
        assert_eq!(
            fees.caveats[0].caveats.len(),
            1,
            "only the markup is a reading: {:?}",
            fees.caveats
        );
    }

    #[test]
    fn what_may_cost_more_is_flagged() {
        let (leumi, pepper) = pepper();
        let flag = |exchange, largest_trade| {
            let buying = Buying {
                security: Security::Etf,
                exchange,
                largest_trade,
            };
            leumi.may_cost_more(&pepper, buying, &rates())
        };
        // The markup is a reading of Leumi's published rates, not a stand-in.
        assert_eq!(flag(Exchange::Usa, Some(usd(dec!(7000)))), None);
        assert_eq!(
            flag(Exchange::Usa, Some(usd(dec!(9000)))).as_deref(),
            Some("May cost more: $4 is stated only for orders up to $8,000")
        );
        assert_eq!(flag(Exchange::Tlv, Some(ils(dec!(1000)))), None);

        // A markup stated as a maximum can only overstate: nothing to flag.
        let altshuler = tariffs::altshuler();
        let offer = &altshuler.plans[1];
        let buying = buying(Security::Etf, Exchange::Usa);
        assert_eq!(altshuler.may_cost_more(offer, buying, &rates()), None);
    }

    #[test]
    fn interactive_converts_at_the_market_rate() {
        let interactive = tariffs::interactive();
        let plan = &interactive.plans[0];
        let buying = buying(Security::Etf, Exchange::Usa);
        let fees = interactive.describe_fees_for(plan, buying, None, &rates());
        // Under conversion, after the standing order's part.
        let markup = fees.fees[2]
            .parts
            .iter()
            .find(|part| part.kind == FeeKind::Markup)
            .unwrap();
        assert_eq!(markup.price, PriceText::nothing("at the market rate"));
        assert_eq!(markup.mark.as_ref().unwrap().kind, CaveatKind::Published);
        assert_eq!(interactive.may_cost_more(plan, buying, &rates()), None);
        let kinds: Vec<CaveatKind> = fees.caveats.iter().map(|group| group.kind).collect();
        assert_eq!(
            kinds,
            [
                CaveatKind::Reading,
                CaveatKind::NotCounted,
                CaveatKind::Published
            ]
        );
    }

    #[test]
    fn altshuler_custody_and_markup() {
        let plan = &tariffs::altshuler().plans[0];
        // Nothing is converted on Tel Aviv, so no conversion lines.
        let fees = plan
            .describe_fees_for(buying(Security::Etf, Exchange::Tlv), None, &[], &rates())
            .fees;
        assert_eq!(fees.len(), 3);
        assert_eq!(
            fees[1].price.text,
            "0.15% a year, charged monthly, min ₪75 a month"
        );
        assert_eq!(fees[2].price.text, "₪80 a month");
        assert_eq!(plan.conversion.to_string(), "none");
        assert_eq!(plan.conversion.markup.to_string(), "up to 0.7%");
        // The custody period was read, with support; the handling fee is a
        // maximum.
        let custody = fees[1].mark.as_ref().unwrap();
        assert_eq!(custody.kind, CaveatKind::Reading);
        let support = custody.caveats[0].support.as_deref().unwrap();
        assert!(support.contains("June 2026"));
        assert_eq!(fees[2].mark.as_ref().unwrap().kind, CaveatKind::AtMost);
    }

    #[test]
    fn prices_of_nothing_say_so() {
        let plan = &tariffs::altshuler().plans[0];
        let fees = plan
            .describe_fees_for(buying(Security::Etf, Exchange::Usa), None, &[], &rates())
            .fees;
        let conversion = fees.last().unwrap();
        assert_eq!(
            conversion.price,
            PriceText::nothing("none"),
            "no conversion fee, like no custody"
        );
        assert!(
            !conversion.parts[0].price.nothing,
            "the markup is up to 0.7%"
        );

        // IBI's index funds on Tel Aviv have a custody row of 0%.
        let free = &tariffs::ibi().plans[0].custody[0];
        assert_eq!(free.price_text(), PriceText::nothing("none"));
    }

    #[test]
    fn not_offered_says_the_most_general_mismatch() {
        let reason = |plan: &Plan, security, exchange| plan.not_offered_reason(security, exchange);
        let interactive = tariffs::interactive();
        let standard = &interactive.plans[0];
        assert_eq!(
            reason(standard, Security::Bond, Exchange::Tlv),
            "Nothing in Tel Aviv is offered"
        );
        assert_eq!(
            reason(standard, Security::IndexFund, Exchange::Usa),
            "No index funds are offered anywhere"
        );
        // Index funds in the USA only: Europe is the gap.
        let mut with_us_funds = standard.clone();
        with_us_funds.trading.push(TradeFee {
            securities: vec![Security::IndexFund],
            exchanges: vec![Exchange::Usa],
            price: Price::Flat(usd(dec!(1))),
        });
        assert_eq!(
            reason(&with_us_funds, Security::IndexFund, Exchange::Europe),
            "No index funds are offered in Europe"
        );
    }

    #[test]
    fn not_offered_coverage_and_dates() {
        let plan = &tariffs::altshuler().plans[0];
        let trade = &plan
            .describe_fees_for(buying(Security::Etf, Exchange::Europe), None, &[], &rates())
            .fees[0];
        assert!(trade.price.nothing);
        assert_eq!(trade.price.text, "not offered");
        assert_eq!(
            trade.price.reason.as_deref(),
            Some("Nothing in Europe is offered")
        );
        assert_eq!(plan.trading[0].coverage().to_string(), "ETF on Tel Aviv");
        assert_eq!(
            purchase(Security::Etf, Exchange::Usa).to_string(),
            "an ETF bought in the USA"
        );
        assert_eq!(
            purchase(Security::IndexFund, Exchange::Tlv).to_string(),
            "an index fund bought in Tel Aviv"
        );
        assert_eq!(
            tariffs::leumi().tariff_date_text().to_string(),
            "Tariff of 29/06/2026"
        );
        assert_eq!(
            tariffs::altshuler().tariff_date_text().to_string(),
            "Tariff date not stated"
        );
        assert_eq!(Broker::checked_text(), "Checked 29/09/2026");
    }

    #[test]
    fn the_about_page_has_its_three_sections() {
        let about = about();
        let titles: Vec<&str> = about
            .sections
            .iter()
            .map(|section| section.title.as_str())
            .collect();
        assert_eq!(
            titles,
            ["How the numbers are made", "What isn't counted", "Sources"]
        );
        let sources = &about.sections[2];
        assert!(sources.paragraphs[0].starts_with("Checked 29/09/2026"));
        // A group per broker, its tariff first, then the comparison sites and
        // the exchange.
        assert_eq!(sources.sources.len(), tariffs::all().len() + 2);
        for (group, broker) in sources.sources.iter().zip(tariffs::all()) {
            assert!(group.title.starts_with(&broker.name), "{}", group.title);
            assert_eq!(group.sources[0].name, "Tariff (PDF)");
            assert!(group.sources.len() > 1, "{}: only the tariff", broker.name);
        }
        assert_eq!(sources.items, Vec::new());
    }
}
