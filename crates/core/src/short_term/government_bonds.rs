//! Shekel government bonds held to maturity (אג"ח ממשלתי): lending to the
//! state, which repays the sum at the end with a yield known on the day the
//! bond is bought. Unlike a bank deposit, it is sold on the Tel Aviv Stock
//! Exchange through an account at a bank or an investment house, so the
//! account's fees count; unlike a money market fund, its yield is fixed for
//! the term. `policies/yield-curve.py` prints the yields from the Bank of
//! Israel's zero-coupon curve; `policies/sources.md` says how they're used.

use std::sync::OnceLock;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use super::{Kind, NotOffered, Pays, Place, Scenario, Via};
use crate::money::ExchangeRates;
use crate::simulation::{self, USUAL_INFLATION};
use crate::vehicles::Withdrawal;
use crate::{Caveat, Exchange, Page, Percent, Plan, Security, TariffDate, Text, Vehicle, funds};

/// When the yields were last taken from the Bank of Israel's figures.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 10 - 19)
}

/// Whether redeeming a bond at maturity costs no trade fee. The brokers'
/// price lists name redemption beside buying and selling in the row for
/// trades ("קניה, מכירה ופדיון"), most of them without saying a redemption
/// is free (Otsar Hahayal's club says so, which suggests the others don't).
/// The plan of record (REDESIGN.md 4.2) takes it as free, so the result is
/// on the cheap side: the kind says so in a caveat that `may_cost_more`
/// flags. Flip it to charge a redemption like a sale.
const REDEMPTION_IS_FREE: bool = true;

/// The longest the curve goes, in years: past it no yield is published.
const LONGEST_YEARS: u32 = 15;

/// The shortest, in years: the curve starts at one year. A shorter wait is
/// makam (Bank of Israel bills), which isn't modelled.
const SHORTEST_YEARS: u32 = 1;

// The yields are rewritten by `policies/yield-curve.py --write`.
// BEGIN yield-curve.py
const MONTH: (i32, u8) = (2026, 9);
/// (years to maturity, % a year), unlinked.
const NOMINAL: [(u32, Decimal); 8] = [
    (1, dec!(3.19)),
    (2, dec!(3.45)),
    (3, dec!(3.60)),
    (4, dec!(3.66)),
    (5, dec!(3.72)),
    (7, dec!(3.85)),
    (10, dec!(4.06)),
    (15, dec!(4.29)),
];
/// (years to maturity, % a year), before the index.
const REAL: [(u32, Decimal); 8] = [
    (1, dec!(1.59)),
    (2, dec!(1.59)),
    (3, dec!(1.65)),
    (4, dec!(1.71)),
    (5, dec!(1.78)),
    (7, dec!(1.91)),
    (10, dec!(2.10)),
    (15, dec!(2.26)),
];
// END yield-curve.py

/// What the bond's repayment is tied to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    /// A shekel bond (שקלי): a nominal yield, whatever prices do. Its whole
    /// gain is taxed at 15%.
    Unlinked,
    /// A bond linked to the consumer price index (צמוד מדד): a real yield on
    /// top of the index. Only the gain beyond prices is taxed, at 25%.
    CpiLinked,
}

/// Why the curve gives no yield for a wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutsideTheCurve {
    TooShort,
    TooLong,
}

/// The month the yields are averages of.
fn data_of() -> TariffDate {
    TariffDate::Month(
        Date::from_calendar_date(
            MONTH.0,
            time::Month::try_from(MONTH.1).expect("a month of the year"),
            1,
        )
        .expect("the first of a month"),
    )
}

/// The yearly yield of a bond held `years` years (a fraction, as the wait
/// is), by straight lines between the published terms.
///
/// # Errors
///
/// A wait before the first term or after the last.
pub fn yield_for_years(linkage: Linkage, years: Decimal) -> Result<Percent, OutsideTheCurve> {
    let curve = match linkage {
        Linkage::Unlinked => &NOMINAL,
        Linkage::CpiLinked => &REAL,
    };
    if years < Decimal::from(SHORTEST_YEARS) {
        return Err(OutsideTheCurve::TooShort);
    }
    let pairs = curve.windows(2).map(|pair| (pair[0], pair[1]));
    for ((from, low), (to, high)) in pairs {
        if years <= Decimal::from(to) {
            let share = (years - Decimal::from(from)) / Decimal::from(to - from);
            return Ok(Percent(low + (high - low) * share));
        }
    }
    Err(OutsideTheCurve::TooLong)
}

/// The yield for a wait of `months`, for [`yield_for_years`].
///
/// # Errors
///
/// A wait outside the curve.
pub fn yield_for_months(linkage: Linkage, months: u32) -> Result<Percent, OutsideTheCurve> {
    yield_for_years(linkage, Decimal::from(months) / Decimal::from(12))
}

/// The most months a bond can be held: [`LONGEST_YEARS`].
pub const LONGEST_MONTHS: u32 = LONGEST_YEARS * 12;

/// The fewest months: a year.
pub const SHORTEST_MONTHS: u32 = SHORTEST_YEARS * 12;

// ─────────────────────────── The money ───────────────────────────

/// What the account that buys the bonds is, and what the money is worth
/// month by month.
pub(super) struct Run {
    pub value_by_month: Vec<Decimal>,
    pub via: Via,
}

/// The plan a new customer usually gets at each bank and investment house
/// (`Broker::new_customer_plan`), with its broker's name. Not the other
/// plans: some are for the young, for soldiers or for a club's members, and
/// the cheapest of them would be named for savers who can't have it. Built
/// once.
fn plans() -> &'static [(Text, Plan)] {
    static PLANS: OnceLock<Vec<(Text, Plan)>> = OnceLock::new();
    PLANS.get_or_init(|| {
        crate::tariffs::all()
            .into_iter()
            .filter_map(|broker| {
                let plan = broker.plans.get(broker.new_customer_plan)?.clone();
                (plan.vehicle == Vehicle::Brokerage).then_some((broker.name, plan))
            })
            .collect()
    })
}

/// Every broker's plan on the saver's deposits held to the end, as the
/// long-term simulation compares them (see [`run`] for what each deposit
/// earns).
///
/// Each deposit earns the yield for the wait that's left when it's made
/// (never less than a year's), the curve being taken to stay as it is. The
/// brokers' fees are worked out by the long-term simulation on one yearly
/// return, so that return is the one that ends with the same money before
/// fees; the simulation's own flat-return growth then carries the fees.
fn compared(linkage: Linkage, scenario: &Scenario) -> Result<simulation::Comparison, NotOffered> {
    let months = scenario.months;
    let curve = |remaining: u32| {
        let remaining = remaining.max(SHORTEST_MONTHS);
        yield_for_months(linkage, remaining).map(|real_or_nominal| {
            let yearly = Decimal::ONE + real_or_nominal.of(Decimal::ONE);
            match linkage {
                Linkage::Unlinked => yearly,
                Linkage::CpiLinked => yearly * (Decimal::ONE + scenario.inflation.of(Decimal::ONE)),
            }
        })
    };
    // The curve is asked about the whole wait first, so a wait outside it is
    // refused whatever the deposits' own waits.
    match yield_for_months(linkage, months) {
        Err(OutsideTheCurve::TooShort) => return Err(NotOffered::BondTooShort),
        Err(OutsideTheCurve::TooLong) => return Err(NotOffered::BondTooLong),
        Ok(_) => {}
    }
    let deposits: Vec<(u32, Decimal, Decimal)> = (0..months)
        .map(|month| {
            let amount = scenario.monthly_deposit
                + if month == 0 {
                    scenario.first_deposit
                } else {
                    Decimal::ZERO
                };
            let yearly = curve(months - month).expect("inside the curve");
            (months - month, amount, yearly)
        })
        .collect();
    let yearly_return = if scenario.monthly_deposit.is_zero() {
        // One sum: its own yield, exactly.
        deposits[0].2 - Decimal::ONE
    } else {
        equivalent_return(&deposits)
    };

    let sim_scenario = simulation::Scenario {
        security: Security::Bond,
        exchange: Exchange::Tlv,
        first_deposit: scenario.first_deposit,
        monthly_deposit: scenario.monthly_deposit,
        deposit_growth: Percent(Decimal::ZERO),
        yearly_return: Percent(yearly_return * Decimal::ONE_HUNDRED),
        years: months.div_ceil(12),
        buy_every_months: 1,
        // Shekel units cost little next to a deposit: a bond is bought by
        // amount (`simulation::whole_shares`), so the price changes nothing.
        share_price: dec!(100),
        sell_at_end: true,
        inflation: USUAL_INFLATION,
        age: 40,
        withdrawal: Withdrawal::LumpSum,
        redeems_at_maturity: REDEMPTION_IS_FREE,
        product: None,
    };
    // The prices are in shekels, so the rates of exchange are never used.
    let rates = ExchangeRates::new(dec!(3.7), dec!(4.625)).expect("positive rates");
    let plans = plans();
    let refs: Vec<&Plan> = plans.iter().map(|(_, plan)| plan).collect();
    let comparison = simulation::compare(&refs, &sim_scenario, &rates);
    Ok(comparison)
}

/// Every broker's plan that can hold the bonds for `scenario` and what it leaves
/// after the long-term simulation's tax and fees, best first: the ranking the
/// cheapest plan comes from.
///
/// # Errors
///
/// A wait outside the curve, or no plan that opens with the first deposit.
pub fn broker_ranking(
    linkage: Linkage,
    scenario: &Scenario,
) -> Result<Vec<(Via, Decimal)>, NotOffered> {
    let comparison = compared(linkage, scenario)?;
    let plans = plans();
    Ok(comparison
        .plans
        .iter()
        .filter(|plan| plan.fits)
        .filter_map(|plan| {
            let outcome = plan.outcome.as_ref().ok()?;
            let (broker, plan) = &plans[plan.index];
            Some((
                Via {
                    broker: broker.clone(),
                    plan: plan.name.clone(),
                },
                outcome.after_tax,
            ))
        })
        .collect())
}

/// What the saver's deposits come to held to the end, month by month, in
/// the cheapest account for bonds.
///
/// Each deposit earns the yield for the wait that's left when it's made
/// (never less than a year's), the curve being taken to stay as it is.
pub(super) fn run(linkage: Linkage, scenario: &Scenario) -> Result<Run, NotOffered> {
    let months = scenario.months;
    let comparison = compared(linkage, scenario)?;
    let plans = plans();
    let best = comparison
        .plans
        .iter()
        .find(|plan| plan.fits && plan.outcome.is_ok())
        .ok_or(NotOffered::NoBrokerPlan)?;
    let outcome = best.outcome.as_ref().expect("checked above");
    let (broker, plan) = &plans[best.index];

    // The simulation counts whole years: the money is read at the month the
    // wait ends. At whole years that is what the redemption pays out.
    let mut value_by_month: Vec<Decimal> = outcome
        .value_by_month
        .iter()
        .take(months as usize + 1)
        .copied()
        .collect();
    if months.is_multiple_of(12) {
        *value_by_month
            .last_mut()
            .expect("the start is always there") = outcome.after_selling;
    }
    Ok(Run {
        value_by_month,
        via: Via {
            broker: broker.clone(),
            plan: plan.name.clone(),
        },
    })
}

/// The yearly return (a fraction) at which the deposits, each growing to the
/// end, end with what they do at their own yields. `deposits` are
/// (months to the end, amount, yearly factor). Found by halving, in
/// floating point like the simulation's own growth.
fn equivalent_return(deposits: &[(u32, Decimal, Decimal)]) -> Decimal {
    let number = |value: Decimal| value.to_f64().unwrap_or(0.0);
    let grown = |factor: &dyn Fn(&(u32, Decimal, Decimal)) -> f64| {
        deposits
            .iter()
            .map(|deposit| number(deposit.1) * factor(deposit).powf(f64::from(deposit.0) / 12.0))
            .sum::<f64>()
    };
    let target = grown(&|deposit| number(deposit.2));
    let (mut low, mut high) = (0.0_f64, 3.0_f64);
    for _ in 0..64 {
        let middle = f64::midpoint(low, high);
        if grown(&|_| middle) < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    Decimal::try_from(f64::midpoint(low, high) - 1.0).unwrap_or(Decimal::ZERO)
}

// ─────────────────────────── The kind ───────────────────────────

fn t(en: &'static str, he: &'static str) -> Text {
    Text::new(en, he)
}

fn source(name: Text, url: &str) -> Page {
    Page {
        name,
        url: url.to_owned(),
    }
}

/// The Bank of Israel's zero-coupon yields, the dataflow the figures are
/// read from.
#[must_use]
pub fn yield_curve() -> Page {
    source(
        t(
            "The Bank of Israel's zero-coupon yield curve (data, CSV)",
            "עקום התשואות אפס־קופון של בנק ישראל (נתונים, CSV)",
        ),
        "https://edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/BOI.STATISTICS/ZCM/1.0?format=csv&lastNObservations=1",
    )
}

fn themarker_unlinked() -> Page {
    source(
        t(
            "A shekel government bond on TheMarker (ממשלתי שקלי 0347)",
            "אג״ח ממשלתי שקלי בדה־מרקר (ממשלתי שקלי 0347)",
        ),
        "https://finance.themarker.com/bond/1140193",
    )
}

fn themarker_linked() -> Page {
    source(
        t(
            "A CPI-linked government bond on TheMarker (ממשלתי צמוד 0536)",
            "אג״ח ממשלתי צמוד מדד בדה־מרקר (ממשלתי צמוד 0536)",
        ),
        "https://finance.themarker.com/bond/1097708",
    )
}

fn tax_on_interest() -> Page {
    source(
        t(
            "Bizportal on the tax on interest and discount",
            "ביזפורטל על שיעור המס על הכנסה מריבית ומדמי ניכיון",
        ),
        "https://www.bizportal.co.il/general/news/article/104576",
    )
}

fn excellence_tariff() -> Page {
    source(
        t("Excellence's tariff (PDF)", "התעריפון של אקסלנס (PDF)"),
        "https://www.xnes.co.il/media/nwogkoos/taarifon.pdf",
    )
}

fn leumi_tariff() -> Page {
    source(
        t("Leumi's tariff (PDF)", "התעריפון של לאומי (PDF)"),
        "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf",
    )
}

fn place(linkage: Linkage, caveats: Vec<Caveat>) -> Place {
    let (name, short_name, description) = match linkage {
        Linkage::Unlinked => (
            t("Shekel government bond", "אג״ח ממשלתי שקלי"),
            t("Shekel bond", "שקלי"),
            t(
                "A bond of the State of Israel in shekels, not linked to anything (as the \
                 banks' apps name it: ממשלתי\u{a0}שקלי, then the month and year it is repaid). \
                 The state repays the sum on a set day and pays a yield fixed when you buy. \
                 Bought through a bank or investment house on the Tel Aviv Stock Exchange.",
                "אג״ח של מדינת ישראל בשקלים, לא צמוד לשום דבר (כך הוא נקרא באפליקציות של הבנקים: ממשלתי שקלי, ואחריו חודש ושנת הפירעון). המדינה מחזירה את הסכום ביום קבוע ומשלמת תשואה שנקבעת ביום הקנייה. קונים אותו דרך בנק או בית השקעות בבורסה בתל אביב.",
            ),
        ),
        Linkage::CpiLinked => (
            t("CPI-linked government bond", "אג״ח ממשלתי צמוד מדד"),
            t("CPI-linked bond", "צמוד מדד"),
            t(
                "A bond of the State of Israel whose sum rises with the consumer price index \
                 (as the banks' apps name it: ממשלתי\u{a0}צמוד, then the month and year it \
                 is repaid), and which pays a real yield on top, fixed when you buy. It keeps \
                 its buying power, whatever prices do. Bought through a bank or investment \
                 house on the Tel Aviv Stock Exchange.",
                "אג״ח של מדינת ישראל שהסכום בו עולה עם מדד המחירים לצרכן, ומעליו תשואה ריאלית שנקבעת ביום הקנייה (כך הוא נקרא באפליקציות של הבנקים: ממשלתי צמוד, ואחריו חודש ושנת הפירעון). הכסף שומר על כוח הקנייה שלו, מה שלא יקרה למחירים. קונים אותו דרך בנק או בית השקעות בבורסה בתל אביב.",
            ),
        ),
    };
    Place {
        name,
        short_name,
        description,
        pays: Pays::GovernmentBond(linkage),
        caveats,
        compared_at_first: false,
    }
}

/// The two government bonds, with the caveats of the kind.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the kind's data and caveats")]
pub fn kind() -> Kind {
    let curve = yield_curve();
    let unlinked = place(
        Linkage::Unlinked,
        vec![
            Caveat::published(t(
                "Taxed at 15% of the whole gain, inflation or not: a bond not linked to the \
                 index is taxed like interest (the Income Tax Ordinance, section 91(ב)(3)).",
                "מס של 15% על כל הרווח, גם על החלק שרק שמר על ערך הכסף: אג״ח שלא צמוד למדד המחירים לצרכן ממוסה כמו ריבית (פקודת מס הכנסה, סעיף 91(ב)(3)).",
            ))
            .sources(&[&funds::income_tax_ordinance(), &tax_on_interest()]),
            Caveat::reading(
                t(
                    "The name in a bank's app is the kind, then the month and year of \
                     repayment: ממשלתי\u{a0}שקלי\u{a0}0347 is repaid in March 2047. The \
                     model takes a bond repaid when the wait ends, which the stock \
                     exchange's list has to have.",
                    "השם באפליקציה של הבנק הוא הסוג, ואחריו חודש ושנת הפירעון: ממשלתי\u{a0}שקלי\u{a0}0347 נפרע במרץ 2047. ההנחה היא שיש אג״ח שנפרע בדיוק כשהתקופה נגמרת, ובמציאות צריך לבדוק ברשימה של הבורסה.",
                ),
                t(
                    "the bond pages of TheMarker, which show the names and repayment dates",
                    "עמודי האג״ח של דה־מרקר, שמראים את השמות ואת תאריכי הפירעון",
                ),
            )
            .source(&themarker_unlinked()),
        ],
    );
    let linked = place(
        Linkage::CpiLinked,
        vec![
            Caveat::published(t(
                "Taxed at 25% of the real gain: the part of the gain that only follows the \
                 index isn't taxed (the Income Tax Ordinance, section 91(ב)(1)). The model \
                 takes prices to rise as much as you expect, 2% a year unless you say \
                 otherwise.",
                "מס של 25% על הרווח הריאלי: החלק שרק עוקב אחרי מדד המחירים לצרכן לא ממוסה (פקודת מס הכנסה, סעיף 91(ב)(1)). ההנחה היא שהמחירים עולים כמו שציפיתם, 2% בשנה אלא אם בחרתם אחרת.",
            ))
            .sources(&[&funds::income_tax_ordinance(), &tax_on_interest()]),
            Caveat::reading(
                t(
                    "The index is taken to rise as much as the saver expects, every year. \
                     The real yield is the Bank of Israel's, before the index; the nominal \
                     one is that and the index together. The bank's app names such a bond \
                     ממשלתי\u{a0}צמוד\u{a0}0536 (repaid in May 2036).",
                    "המדד נלקח כעולה כמו שהחוסך מצפה, בכל שנה. התשואה הריאלית היא של בנק ישראל, לפני המדד, והנומינלית היא היא והמדד יחד. באפליקציה של הבנק אג״ח כזה נקרא ממשלתי צמוד 0536 (נפרע במאי 2036).",
                ),
                t(
                    "the bond pages of TheMarker, which show the names and repayment dates",
                    "עמודי האג״ח של דה־מרקר, שמראים את השמות ואת תאריכי הפירעון",
                ),
            )
            .source(&themarker_linked()),
        ],
    );
    Kind {
        name: t("Government bond", "אג״ח ממשלתי"),
        description: t(
            "A loan to the State of Israel that you hold until it is repaid, on a day you \
             choose by choosing the bond. The state pays a yield fixed when you buy, so you \
             know in advance what you will have at the end, and the risk is the lowest there \
             is. It suits money you will need on a known day, a few years from now. The \
             downsides: sold earlier, the price may be lower than you paid, and buying it \
             takes an account with a bank or an investment house, whose fees are counted here.",
            "הלוואה למדינת ישראל שמחזיקים עד שהיא נפרעת, ביום שבוחרים כשבוחרים את האג״ח. המדינה משלמת תשואה שנקבעת ביום הקנייה, ולכן אפשר לדעת מראש כמה יהיה בסוף, והסיכון הוא הנמוך ביותר שיש. מתאים לכסף שיידרש ביום ידוע, בעוד כמה שנים. החסרונות: אם מוכרים מוקדם, המחיר יכול להיות נמוך ממה ששילמתם, וצריך חשבון בבנק או בבית השקעות כדי לקנות אותו, ודמי החשבון נספרים כאן.",
        ),
        data_of: data_of(),
        checked: checked(),
        source: curve.clone(),
        caveats: vec![
            Caveat::published(t(
                "The yields are the Bank of Israel's zero-coupon yields, the monthly \
                 average of September 2026, for waits of 1 to 15 years; between the \
                 published terms they are read on a straight line. A bond bought on \
                 another day earns the yield of that day.",
                "התשואות הן תשואות אפס־קופון של בנק ישראל, הממוצע החודשי של ספטמבר 2026, לתקופות של שנה עד 15 שנים; בין התקופות שפורסמו הן נקראות על קו ישר. אג״ח שנקנה ביום אחר מרוויח את התשואה של אותו יום.",
            ))
            .source(&curve),
            Caveat::not_counted(t(
                "A bond pays interest along the way (a coupon); the model has it reinvested \
                 at the same yield. In reality the coupons are reinvested at the rates of \
                 the day, which may be lower or higher.",
                "אג״ח משלם ריבית לאורך הדרך (קופון). במודל הריבית הזו מושקעת מחדש באותה תשואה, ובמציאות היא מושקעת בריביות של אותו יום, שיכולות להיות נמוכות או גבוהות יותר.",
            )),
            Caveat::not_counted(t(
                "Until it is repaid the bond can be sold any day at the day's price, which \
                 moves with interest rates: higher rates lower it. The sum is known only \
                 if you hold it to the end. The model does not show that price risk.",
                "אפשר למכור את האג״ח בכל יום במחיר של אותו יום, והמחיר זז עם הריבית: ריבית גבוהה יותר מורידה אותו. הסכום ידוע רק אם מחזיקים עד הסוף. המודל לא מראה את סיכון המחיר הזה.",
            )),
            Caveat::reading(
                t(
                    "The account is the cheapest of the plans a new customer gets at the banks \
                     and investment houses, for bonds on Tel Aviv, by their price lists, and the plan is named in \
                     the result. Each monthly deposit buys a bond repaid at the end, at \
                     the yield for the wait that is left (a year's yield when less than a \
                     year is left).",
                    "החשבון הוא המסלול הזול ביותר של הבנקים ובתי ההשקעות לאג״ח בתל אביב, לפי התעריפונים שלהם, והמסלול נקרא בתוצאה. כל הפקדה חודשית קונה אג״ח שנפרע בסוף, בתשואה של התקופה שנשארה (תשואה של שנה כשנשארה פחות משנה).",
                ),
                t(
                    "the price lists of the banks and investment houses",
                    "התעריפונים של הבנקים ובתי ההשקעות",
                ),
            )
            .source(&excellence_tariff()),
            Caveat::may_cost_more(
                t(
                    "No fee is counted when the state repays the bond. The price lists name \
                     redemption (פדיון) in the row for buying and selling, and say nowhere \
                     that it is free, apart from one club's: a broker may charge it like a \
                     sale.",
                    "לא נספרת עמלה כשהמדינה פורעת את האג״ח. בתעריפונים הפדיון מופיע באותה שורה כמו קנייה ומכירה, ובשום מקום לא כתוב שהוא בחינם, חוץ מבמועדון אחד: ייתכן שברוקר יגבה עליו כמו על מכירה.",
                ),
                t(
                    "Redemption may cost a trade fee",
                    "ייתכן שהפדיון יעלה עמלת מסחר",
                ),
            )
            .sources(&[&excellence_tariff(), &leumi_tariff()]),
        ],
        places: vec![unlinked, linked],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_published_term_is_read_as_published() {
        assert_eq!(
            yield_for_years(Linkage::Unlinked, dec!(5)),
            Ok(Percent(dec!(3.72)))
        );
        assert_eq!(
            yield_for_years(Linkage::CpiLinked, dec!(15)),
            Ok(Percent(dec!(2.26)))
        );
    }

    #[test]
    fn between_terms_is_a_straight_line() {
        // Halfway from 5 years (3.72) to 7 (3.85) is 6 years: 3.785.
        assert_eq!(
            yield_for_years(Linkage::Unlinked, dec!(6)),
            Ok(Percent(dec!(3.785)))
        );
    }

    #[test]
    fn outside_the_curve_there_is_no_yield() {
        assert_eq!(
            yield_for_months(Linkage::Unlinked, 11),
            Err(OutsideTheCurve::TooShort)
        );
        assert_eq!(
            yield_for_months(Linkage::Unlinked, 16 * 12),
            Err(OutsideTheCurve::TooLong)
        );
    }
}
