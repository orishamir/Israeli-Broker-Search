//! What holds the index for the saver, and what that costs on its own. A
//! broker's plans hold a fund bought on the exchange; a fund's plans hold its
//! own index track. Either keeps back part of the index's return before the
//! saver sees it: its own fees, the part of the index's dividends that tax
//! takes and nobody gives back, and whatever else made it trail. The return
//! the user expects is the index's, dividends included, and every plan is
//! charged its product's yearly cost besides its own fees.
//!
//! A product's yearly cost is the larger of what it publishes, with the
//! dividend tax it loses, and how far it trailed the index over the five
//! years to [`MEASURED_TO`], so that none is shown cheaper than it has been.
//! A kind of product is its funds' average, each counted by the money in it.
//! `policies/index-tracking.py` measures the gaps and lists the fees;
//! `policies/sources.md` says how they were read.
//!
//! Only the S&P 500 is measured: it's what the examples buy and what most
//! savers hold. A share or a bond is held directly, with nothing between.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use time::Date;
use time::macros::date;

use crate::{Caveat, Exchange, FeeKind, Page, Percent, Security, Text, Vehicle, funds, usd};

/// The S&P 500's dividends a year over the five years measured: how much
/// the index with its dividends gained beyond the index of prices alone.
pub const DIVIDENDS: Percent = Percent(dec!(1.44));

/// The last day of the five years the products were measured over. The
/// funds' tracks are measured to the month before, the regulator's last.
pub const MEASURED_TO: Date = date!(2026 - 09 - 30);

/// What holds the index for the saver: a kind of fund bought at a broker,
/// or a kind of fund's index track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "ts", derive(tsify::Tsify))]
pub enum Product {
    /// An Israeli index fund (קרן מחקה) on the S&P 500, bought from its
    /// manager once a day.
    IsraeliIndexFund,
    /// An Israeli ETF (קרן סל) on the S&P 500, traded on the exchange.
    IsraeliEtf,
    /// An Irish ETF on the S&P 500 that's also listed in Tel Aviv, and
    /// bought there in shekels: iShares' or Invesco's.
    ForeignEtfInTelAviv,
    /// A US fund on the S&P 500, such as VOO: it pays its dividends out,
    /// and the US keeps a quarter of them.
    UsFund,
    /// An Irish fund on the S&P 500, such as CSPX, bought in Europe: it
    /// keeps its dividends, after the US has kept 15% of them.
    IrishFund,
    /// The S&P 500 track of a study fund.
    StudyFundTrack,
    /// The S&P 500 track of a provident fund for investment.
    InvestmentGemelTrack,
    /// The S&P 500 track of a savings policy.
    SavingsPolicyTrack,
}

/// What a product costs a year, by where it comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Costs {
    /// What it publishes, a year: its own fee, the most its variable fee can
    /// be and its trustee's, and the share of the index's dividends that
    /// tax takes from it by how it holds the index. None for a fund's
    /// track: the manager's fee is its plans' own.
    pub published: Percent,
    /// How far it trailed the index with all its dividends, a year, over the
    /// five years to [`MEASURED_TO`], if it was measured.
    pub trailed: Option<Percent>,
    /// Where it pays the dividends out: the share of them tax keeps then.
    pub dividends_paid_out_taxed: Option<Percent>,
}

impl Costs {
    /// What a plan holding it is charged a year: the larger of what it
    /// publishes and how far it trailed.
    #[must_use]
    pub fn yearly(&self) -> Percent {
        self.trailed
            .map_or(self.published, |trailed| self.published.max(trailed))
    }

    /// What reaches the saver of the dividends a year, where they're paid
    /// out, after their tax. Bought anew, it's part of what the holdings
    /// cost, and lowers the tax at the end.
    #[must_use]
    pub fn paid_out(&self) -> Option<Percent> {
        self.dividends_paid_out_taxed
            .map(|tax| Percent(DIVIDENDS.0 - tax.of(DIVIDENDS.0)))
    }
}

impl Product {
    /// What a broker's plans can hold `security` through on `exchange`, the
    /// one most held first. None for a share or a bond, held directly.
    #[must_use]
    pub fn for_purchase(security: Security, exchange: Exchange) -> &'static [Product] {
        use Product::{ForeignEtfInTelAviv, IrishFund, IsraeliEtf, IsraeliIndexFund, UsFund};
        match (security, exchange) {
            (Security::Etf, Exchange::Tlv) => &[IsraeliEtf, ForeignEtfInTelAviv],
            (Security::IndexFund, Exchange::Tlv) => &[IsraeliIndexFund],
            (Security::Etf | Security::IndexFund, Exchange::Usa) => &[UsFund],
            (Security::Etf | Security::IndexFund, Exchange::Europe) => &[IrishFund],
            (Security::Bond | Security::Stock, _) => &[],
        }
    }

    /// The index track a fund's plans hold, by the kind of fund. None for a
    /// broker's account, which holds what the saver buys.
    #[must_use]
    pub fn track_of(vehicle: Vehicle) -> Option<Product> {
        match vehicle {
            Vehicle::Brokerage => None,
            Vehicle::InvestmentGemel => Some(Product::InvestmentGemelTrack),
            Vehicle::StudyFund => Some(Product::StudyFundTrack),
            Vehicle::SavingsPolicy => Some(Product::SavingsPolicyTrack),
        }
    }

    /// Its costs, as `policies/index-tracking.py` found them in October
    /// 2026, rounded to the hundredth of a percent.
    #[must_use]
    pub fn costs(self) -> Costs {
        let measured = |published, trailed| Costs {
            published: Percent(published),
            trailed: Some(Percent(trailed)),
            dividends_paid_out_taxed: None,
        };
        match self {
            // Eight funds, 0.02%–0.42% a year with the trustee's, and up to
            // 0.3% more when they beat the index after its dividends' tax.
            Product::IsraeliIndexFund => measured(dec!(0.26), dec!(0.15)),
            // Six funds, 0.11%–0.81% with the trustee's, and up to 0.3% more.
            Product::IsraeliEtf => measured(dec!(0.82), dec!(0.75)),
            // iShares 0.07% and 15% of the dividends, 0.29%; Invesco, by
            // swaps, 0.05%.
            Product::ForeignEtfInTelAviv => measured(dec!(0.20), dec!(0.19)),
            // VOO's 0.03%, and a quarter of the dividends.
            Product::UsFund => Costs {
                published: Percent(dec!(0.03) + QUARTER.of(DIVIDENDS.0)),
                trailed: None,
                dividends_paid_out_taxed: Some(QUARTER),
            },
            // CSPX's 0.07%, and 15% of the dividends: 0.2860%.
            Product::IrishFund => {
                measured(dec!(0.07) + Percent(dec!(15)).of(DIVIDENDS.0), dec!(0.28))
            }
            // Before the manager's fee, which the fund's plans charge.
            Product::StudyFundTrack => measured(Decimal::ZERO, dec!(0.42)),
            Product::InvestmentGemelTrack => measured(Decimal::ZERO, dec!(0.54)),
            Product::SavingsPolicyTrack => measured(Decimal::ZERO, dec!(0.33)),
        }
    }

    /// Where each part of its cost comes from, and what else it means for
    /// the saver: each about the product's own line in a plan's fees.
    #[must_use]
    pub fn caveats(self) -> Vec<Caveat> {
        self.what_backs_it()
            .into_iter()
            .map(|caveat| caveat.about_fee(FeeKind::Product))
            .collect()
    }

    #[allow(clippy::too_many_lines, reason = "the products' data")]
    fn what_backs_it(self) -> Vec<Caveat> {
        match self {
            Product::IsraeliIndexFund => vec![
                Caveat::published(t(
                    "The eight Israeli index funds on the S&P 500 charge 0.02%–0.42% a year, \
                     the trustee's share included, and most may take up to 0.1%–0.3% more \
                     when they beat the index after the US tax on its dividends: 0.26% a \
                     year at most, by the money in each (Maya, September 2026).",
                    "שמונה הקרנות המחקות הישראליות על S&P\u{a0}500 גובות 0.02%–0.42% בשנה, כולל דמי הנאמן, ורובן רשאיות לגבות עוד 0.1%–0.3% כשהן מכות את המדד אחרי המס האמריקאי על הדיבידנדים: לכל היותר 0.26% בשנה, בממוצע לפי הכסף שבכל קרן (מאי״ה, ספטמבר 2026).",
                ))
                .source(&maya()),
                measured_caveat(
                    t(
                        "Over the five years to September 2026 they trailed the S&P 500 with \
                         all its dividends, in shekels, by 0.15% a year on average: they follow \
                         it by futures and swaps rather than holding the shares, so no tax is \
                         taken from its dividends. The larger figure, 0.26%, is counted.",
                        "בחמש השנים עד ספטמבר 2026 הן פיגרו אחרי מדד S&P\u{a0}500 כולל כל הדיבידנדים, בשקלים, ב-0.15% בשנה בממוצע: הן עוקבות אחריו בחוזים ולא מחזיקות את המניות, ולכן לא נלקח מס מהדיבידנדים. המחשבון סופר את הנתון הגבוה מבין השניים, 0.26%.",
                    ),
                    t(
                        "each fund's daily price on Maya, against the index with its \
                         dividends at the Bank of Israel's rate",
                        "המחיר היומי של כל קרן במאי״ה, מול המדד עם הדיבידנדים בשער בנק ישראל",
                    ),
                    &[&maya(), &the_marker_on_tracking()],
                ),
            ],
            Product::IsraeliEtf => vec![
                Caveat::published(t(
                    "The six Israeli ETFs on the S&P 500 charge 0.1%–0.8% a year, and up to \
                     0.3% more when they beat the index after the US tax on its dividends; \
                     their trustees take up to 0.03%: 0.82% a year at most, by the money in \
                     each (Funder, October 2026).",
                    "שש קרנות הסל הישראליות על S&P\u{a0}500 גובות 0.1%–0.8% בשנה, ועוד עד 0.3% כשהן מכות את המדד אחרי המס האמריקאי על הדיבידנדים, והנאמן לוקח עד 0.03%: לכל היותר 0.82% בשנה, בממוצע לפי הכסף שבכל קרן (פאנדר, אוקטובר 2026).",
                ))
                .sources(&[&funder(), &calcalist_on_etf_fees()]),
                measured_caveat(
                    t(
                        "Over the five years to September 2026 they trailed the S&P 500 with \
                         all its dividends, in shekels, by 0.38%–1.01% a year, 0.75% on \
                         average: about what they charge. The larger figure, 0.82%, is \
                         counted.",
                        "בחמש השנים עד ספטמבר 2026 הן פיגרו אחרי מדד S&P\u{a0}500 כולל כל הדיבידנדים, בשקלים, ב-0.38%–1.01% בשנה, 0.75% בממוצע: בערך מה שהן גובות. המחשבון סופר את הנתון הגבוה מבין השניים, 0.82%.",
                    ),
                    t(
                        "each fund's closing prices on the exchange, averaged over the first \
                         and last month, against the index with its dividends at the Bank of \
                         Israel's rate",
                        "מחירי הנעילה של כל קרן בבורסה, בממוצע על החודש הראשון והאחרון, מול המדד עם הדיבידנדים בשער בנק ישראל",
                    ),
                    &[&the_exchange()],
                ),
            ],
            Product::ForeignEtfInTelAviv => vec![
                Caveat::published(t(
                    "iShares Core S&P 500 charges 0.07% a year, and holds the shares, so the US \
                     keeps 15% of their dividends: 0.29% in all. Invesco S&P 500 charges 0.05% \
                     and follows the index by swaps, so nothing is taken from its dividends: \
                     0.20% a year, by the money each holds in Tel Aviv.",
                    "קרן הסל iShares Core S&P\u{a0}500 גובה 0.07% בשנה ומחזיקה את המניות עצמן, ולכן ארה״ב שומרת לעצמה 15% מהדיבידנדים שלהן: 0.29% בסך הכול. Invesco S&P\u{a0}500 גובה 0.05% ועוקבת אחרי המדד בחוזי החלף (סוואפ), ולכן לא נלקח דבר מהדיבידנדים: 0.20% בשנה, בממוצע לפי הכסף שכל אחת מחזיקה בתל אביב.",
                ))
                .sources(&[&ishares(), &invesco(), &treaty_rates()]),
                measured_caveat(
                    t(
                        "Over the five years to September 2026, iShares trailed the S&P 500 \
                         with all its dividends, in shekels, by 0.33% a year; Invesco, listed \
                         since 2022, by none. The larger figure, 0.20%, is counted.",
                        "בחמש השנים עד ספטמבר 2026 פיגרה iShares אחרי מדד S&P\u{a0}500 כולל כל הדיבידנדים, בשקלים, ב-0.33% בשנה, ו-Invesco, שנסחרת כאן מאז 2022, לא פיגרה בכלל. המחשבון סופר את הנתון הגבוה מבין השניים, 0.20%.",
                    ),
                    t(
                        "each fund's closing prices on the exchange, against the index with \
                         its dividends at the Bank of Israel's rate",
                        "מחירי הנעילה של כל קרן בבורסה, מול המדד עם הדיבידנדים בשער בנק ישראל",
                    ),
                    &[&the_exchange()],
                ),
            ],
            Product::UsFund => vec![
                Caveat::published(t(
                    "VOO and IVV charge 0.03% a year; SPY, 0.0945%.",
                    "VOO ו-IVV גובות 0.03% בשנה, ו-SPY גובה 0.0945%.",
                ))
                .source(&vanguard()),
                Caveat::published(t(
                    "A US fund pays its dividends out, and the US keeps a quarter of them, by \
                     its tax treaty with Israel; Israel's own tax on them is set against it. \
                     At the S&P 500's 1.44% a year, that's 0.36%. The rest is reinvested and \
                     counts as bought then, so the tax at the end is on less.",
                    "קרן אמריקאית מחלקת את הדיבידנדים, וארה״ב מנכה מהם רבע במקור, לפי אמנת המס עם ישראל. המס הישראלי על הדיבידנד מתקזז מול הניכוי הזה. לפי תשואת דיבידנד של 1.44% בשנה ב-S&P\u{a0}500, זה 0.36% בשנה. השאר מושקע מחדש ונחשב כקנייה באותו מועד, ולכן מס רווחי ההון בסוף מחושב על רווח קטן יותר.",
                ))
                .sources(&[&treaty_rates(), &bizportal_on_dividends()]),
                Caveat::may_cost_more(
                    t(
                        "Someone who dies holding more than $60,000 in US securities leaves \
                         their heirs a US estate tax of up to 40% on the rest: Israel has no \
                         treaty against it. Irish and Israeli funds aren't US securities.",
                        "מי שנפטר כשהוא מחזיק יותר מ-$60,000 בניירות ערך אמריקאיים משאיר ליורשיו מס עיזבון אמריקאי של עד 40% על היתרה: לישראל אין אמנה שפוטרת ממנו. קרנות איריות וישראליות אינן ניירות ערך אמריקאיים.",
                    ),
                    t("US estate tax above $60,000", "מס עיזבון אמריקאי מעל $60,000"),
                )
                .when_above(usd(dec!(60000)))
                .source(&irs_estate_tax()),
            ],
            Product::IrishFund => vec![
                Caveat::published(t(
                    "iShares Core S&P 500 (CSPX) charges 0.07% a year, and holds the shares, \
                     so the US keeps 15% of their dividends, by its tax treaty with Ireland: \
                     0.29% a year in all. It keeps the rest, so nothing is taxed before the \
                     sale.",
                    "קרן הסל iShares Core S&P\u{a0}500 (CSPX) גובה 0.07% בשנה ומחזיקה את המניות עצמן, ולכן ארה״ב מנכה 15% מהדיבידנדים שלהן, לפי אמנת המס עם אירלנד: 0.29% בשנה בסך הכול. הקרן צוברת את השאר, ולכן אין מס עד המכירה.",
                ))
                .sources(&[&ishares(), &treaty_rates()]),
                measured_caveat(
                    t(
                        "Over the five years to September 2026, in London, it trailed the S&P \
                         500 with all its dividends by 0.28% a year: what it publishes.",
                        "בחמש השנים עד ספטמבר 2026, בבורסת לונדון, היא פיגרה אחרי מדד S&P\u{a0}500 כולל כל הדיבידנדים ב-0.28% בשנה: מה שהיא מפרסמת.",
                    ),
                    t(
                        "its closing prices in London, in dollars, against the index with its \
                         dividends",
                        "מחירי הנעילה שלה בלונדון, בדולרים, מול המדד עם הדיבידנדים",
                    ),
                    &[&ishares()],
                ),
            ],
            Product::StudyFundTrack => vec![track_caveat(
                t(
                    "The study funds' S&P 500 tracks trailed the index with all its \
                     dividends, in shekels, by 0.42% a year before their management fee, over \
                     the five years to August 2026, by the money in each: what they pay to \
                     trade, the tax they lose on dividends, and their direct expenses. It's \
                     counted besides the fee.",
                    "מסלולי S&P\u{a0}500 של קרנות ההשתלמות פיגרו אחרי המדד כולל כל הדיבידנדים, בשקלים, ב-0.42% בשנה לפני דמי הניהול, בחמש השנים עד אוגוסט 2026, בממוצע לפי הכסף שבכל מסלול: עמלות המסחר שלהם, המס שהם מפסידים על דיבידנדים וההוצאות הישירות. המחשבון סופר את זה בנוסף לדמי הניהול.",
                ),
                &funds::gemel_net(),
            )],
            Product::InvestmentGemelTrack => vec![track_caveat(
                t(
                    "The provident funds' S&P 500 tracks trailed the index with all its \
                     dividends, in shekels, by 0.54% a year before their management fee, over \
                     the five years to August 2026, by the money in each: what they pay to \
                     trade, the tax they lose on dividends, and their direct expenses. It's \
                     counted besides the fee.",
                    "מסלולי S&P\u{a0}500 של קופות הגמל להשקעה פיגרו אחרי המדד כולל כל הדיבידנדים, בשקלים, ב-0.54% בשנה לפני דמי הניהול, בחמש השנים עד אוגוסט 2026, בממוצע לפי הכסף שבכל מסלול: עמלות המסחר שלהם, המס שהם מפסידים על דיבידנדים וההוצאות הישירות. המחשבון סופר את זה בנוסף לדמי הניהול.",
                ),
                &funds::gemel_net(),
            )],
            Product::SavingsPolicyTrack => vec![track_caveat(
                t(
                    "The insurers' S&P 500 tracks trailed the index with all its dividends, \
                     in shekels, by 0.33% a year before their management fee, over the five \
                     years to August 2026, by the money in each: what they pay to trade, the \
                     tax they lose on dividends, and their direct expenses. It's counted \
                     besides the fee.",
                    "מסלולי S&P\u{a0}500 של חברות הביטוח פיגרו אחרי המדד כולל כל הדיבידנדים, בשקלים, ב-0.33% בשנה לפני דמי הניהול, בחמש השנים עד אוגוסט 2026, בממוצע לפי הכסף שבכל מסלול: עמלות המסחר שלהם, המס שהם מפסידים על דיבידנדים וההוצאות הישירות. המחשבון סופר את זה בנוסף לדמי הניהול.",
                ),
                &funds::bituach_net(),
            )],
        }
    }
}

/// A quarter: the US tax on an Israeli's dividends, by the treaty.
const QUARTER: Percent = Percent(dec!(25));

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

/// How far a kind of product trailed the index, measured from `what`.
fn measured_caveat(text: Text, what: Text, sources: &[&Page]) -> Caveat {
    let Text { en, he } = what;
    Caveat::reading(
        text,
        Text::owned(
            format!("measured from {en} (policies/index-tracking.py)"),
            format!("נמדד לפי {he} (policies/index-tracking.py)"),
        ),
    )
    .sources(sources)
    .source(&the_script())
}

/// How far a kind of fund's track trailed, from the regulator's reports.
fn track_caveat(text: Text, reports: &Page) -> Caveat {
    measured_caveat(
        text,
        t(
            "each track's monthly returns, which are before the management fee, against the \
             index with its dividends at the Bank of Israel's rate",
            "התשואות החודשיות של כל מסלול, שמדווחות לפני דמי הניהול, מול המדד עם הדיבידנדים בשער בנק ישראל",
        ),
        &[
            reports,
            &returns_before_fees(),
            &funds::direct_expenses_regulations(),
        ],
    )
}

// ─────────────────────────── Sources ───────────────────────────

fn page(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

fn the_script() -> Page {
    page(
        t(
            "The script that measures the products",
            "הסקריפט שמודד את הקרנות",
        ),
        "https://github.com/orishamir/Israeli-Broker-Search/blob/main/policies/index-tracking.py",
    )
}

fn maya() -> Page {
    page(
        t(
            "Maya, the Tel Aviv Stock Exchange's list of mutual funds",
            "מאי״ה, רשימת קרנות הנאמנות של הבורסה לניירות ערך",
        ),
        "https://maya.tase.co.il/he/funds/mutual-funds",
    )
}

fn funder() -> Page {
    page(
        t(
            "Funder's page of an Israeli S&P 500 ETF (Kesem)",
            "הדף של קרן סל ישראלית על S&P\u{a0}500 (קסם) בפאנדר",
        ),
        "https://www.funder.co.il/etf/1146471",
    )
}

fn the_exchange() -> Page {
    page(
        t(
            "The Tel Aviv Stock Exchange's page of an S&P 500 ETF (Kesem)",
            "הדף של קרן סל על S&P\u{a0}500 (קסם) באתר הבורסה",
        ),
        "https://market.tase.co.il/he/market_data/etf/1146471/major_data",
    )
}

fn calcalist_on_etf_fees() -> Page {
    page(
        t(
            "Calcalist: Israeli S&P 500 ETFs charge ten times the foreign ones (January 2025)",
            "כלכליסט: קרנות הסל הישראליות על S&P\u{a0}500 גובות פי 10 מהזרות (ינואר 2025)",
        ),
        "https://www.calcalist.co.il/market/article/r140625p1g",
    )
}

fn the_marker_on_tracking() -> Page {
    page(
        t(
            "TheMarker's guide to ETFs and index funds",
            "המדריך של TheMarker לקרנות סל ולקרנות מחקות",
        ),
        "https://www.supermarker.themarker.com/Investments/EtfAndIndexTrackingFunds.aspx",
    )
}

fn ishares() -> Page {
    page(
        t(
            "iShares Core S&P 500 UCITS ETF (CSPX)",
            "iShares Core S&P\u{a0}500 UCITS ETF (CSPX)",
        ),
        "https://www.ishares.com/uk/individual/en/products/253743/ishares-sp-500-b-ucits-etf-acc-fund",
    )
}

fn invesco() -> Page {
    page(
        t(
            "Invesco S&P 500 UCITS ETF",
            "Invesco S&P\u{a0}500 UCITS ETF",
        ),
        "https://etf.invesco.com/gb/private/en/product/invesco-sp-500-ucits-etf-acc/",
    )
}

fn vanguard() -> Page {
    page(
        t(
            "Vanguard S&P 500 ETF (VOO)",
            "Vanguard S&P\u{a0}500 ETF (VOO)",
        ),
        "https://investor.vanguard.com/investment-products/etfs/profile/voo",
    )
}

fn treaty_rates() -> Page {
    page(
        t(
            "The IRS's tables of tax treaty rates on dividends",
            "טבלאות שיעורי המס על דיבידנדים לפי אמנות המס, באתר רשות המסים האמריקאית",
        ),
        "https://www.irs.gov/individuals/international-taxpayers/tax-treaty-tables",
    )
}

fn bizportal_on_dividends() -> Page {
    page(
        t(
            "Bizportal: what reaches Israel of a $1,000 dividend from Wall Street",
            "ביזפורטל: דיבידנד של 1,000 דולר מוול סטריט, כמה מגיע בפועל לחשבון בישראל",
        ),
        "https://www.bizportal.co.il/guides/news/article/20038373",
    )
}

fn irs_estate_tax() -> Page {
    page(
        t(
            "The IRS: some nonresidents with US assets must file estate tax returns",
            "רשות המסים האמריקאית: תושבי חוץ עם נכסים בארה״ב חייבים בדוח מס עיזבון",
        ),
        "https://www.irs.gov/individuals/international-taxpayers/some-nonresidents-with-us-assets-must-file-estate-tax-returns",
    )
}

fn returns_before_fees() -> Page {
    page(
        t(
            "Analyst: Gemel Net's returns are before the management fee",
            "אנליסט: התשואות בגמל נט הן לפני דמי ניהול",
        ),
        "https://www.analyst.co.il/yield-comparison-calculator/",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntoEnumIterator;

    #[test]
    fn the_larger_of_what_is_published_and_what_was_measured_is_counted() {
        let costs = |published, trailed: Option<Decimal>| Costs {
            published: Percent(published),
            trailed: trailed.map(Percent),
            dividends_paid_out_taxed: None,
        };
        assert_eq!(costs(dec!(0.29), None).yearly(), Percent(dec!(0.29)));
        assert_eq!(
            costs(dec!(0.29), Some(dec!(0.2))).yearly(),
            Percent(dec!(0.29))
        );
        assert_eq!(
            costs(dec!(0.29), Some(dec!(0.4))).yearly(),
            Percent(dec!(0.4))
        );
    }

    #[test]
    fn a_us_fund_loses_a_quarter_of_its_dividends_and_reinvests_the_rest() {
        let us = Product::UsFund.costs();
        // 0.03% + 25% of 1.44%
        assert_eq!(us.yearly(), Percent(dec!(0.39)));
        // 1.44% less a quarter of it.
        assert_eq!(us.paid_out(), Some(Percent(dec!(1.08))));
        assert_eq!(Product::IrishFund.costs().paid_out(), None);
    }

    #[test]
    fn every_fund_bought_at_a_broker_holds_a_product_and_a_share_or_bond_none() {
        for security in Security::iter() {
            for exchange in Exchange::iter() {
                let products = Product::for_purchase(security, exchange);
                let a_fund = matches!(security, Security::Etf | Security::IndexFund);
                assert_eq!(!products.is_empty(), a_fund, "{security:?} on {exchange:?}");
            }
        }
    }

    #[test]
    fn every_product_is_either_bought_or_a_funds_track() {
        for product in Product::iter() {
            let bought = Security::iter().any(|security| {
                Exchange::iter()
                    .any(|exchange| Product::for_purchase(security, exchange).contains(&product))
            });
            let a_track =
                Vehicle::iter().any(|vehicle| Product::track_of(vehicle) == Some(product));
            assert!(bought != a_track, "{product:?}");
        }
    }

    #[test]
    fn every_products_cost_says_where_it_comes_from() {
        for product in Product::iter() {
            let caveats = product.caveats();
            assert!(!caveats.is_empty(), "{product:?}");
            assert!(
                caveats.iter().all(|caveat| !caveat.sources.is_empty()),
                "{product:?}"
            );
            let costs = product.costs();
            assert!(costs.yearly().0 > Decimal::ZERO, "{product:?}");
        }
    }
}
