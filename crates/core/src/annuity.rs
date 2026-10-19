//! A monthly pension (קצבה) from a provident fund for investment. The fund
//! pays none itself: from 60 the saver moves the money to a pension fund or
//! an insurer, which divides it by its conversion coefficient (מקדם המרה)
//! for the saver's sex, age and birth year. Nobody can say today which
//! payer the saver will choose, nor whether they are a man or a woman (the
//! site doesn't ask: it changes the pension by a few percent, not the
//! advice), so the estimate is a range: the sum over the highest coefficient
//! any of them publishes, up to the sum over the lowest.
//!
//! The coefficients are the payers' own published tables
//! (`policies/sources.md`, "The monthly pension"), for a pension with 240
//! guaranteed months and none for a spouse: the standard choice the tables
//! all print. They are worked out on a real return (Clal's: "תשואה ריאלית
//! אפקטיבית", 4% gross), so a sum in today's money gives a pension in
//! today's money.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{Page, Percent, Text};

/// The youngest age the pension can start at: the fund's pension is tax-free
/// only from 60 (`vehicles.rs`), and the money can't move to a payer before.
pub const YOUNGEST: u32 = 60;

/// The oldest age the tables below answer for. Past 67 a pension fund's
/// guarantee may not run past 87, so from 68 its tables guarantee less than
/// 240 months (see [`CLAL_MEN`]).
pub const OLDEST: u32 = 70;

/// What a sum would pay each month, from the least any payer would pay to
/// the most.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub low: Decimal,
    pub high: Decimal,
    /// Whose coefficients, for what age, with what guarantee: "לפי מקדמי
    /// ההמרה שמפרסמות …, לגבר או לאישה שמתחילים לקבל קצבה בגיל 67, עם 240
    /// חודשים מובטחים, וללא קצבה לבן או לבת זוג."
    pub assumes: Text,
    /// The tables the range comes from: only those that print the age.
    pub sources: Vec<Page>,
}

/// The monthly pension `sum` would pay from `age` (60 to 70) if it starts in
/// the year `starts_in`, or `None` for an age the tables don't cover. The
/// year matters because each table expects the later-born to live longer:
/// the pension funds print a column per birth year, the insurer raises its
/// coefficients by a share each year.
#[must_use]
pub fn monthly(sum: Decimal, age: u32, starts_in: i32) -> Option<Estimate> {
    if !(YOUNGEST..=OLDEST).contains(&age) {
        return None;
    }
    let born = starts_in - i32::try_from(age).ok()?;
    let mut all_sources = sources().into_iter();
    let mut payers = Vec::new();
    let mut used = Vec::new();
    let mut coefficients = Vec::new();
    // `sources()` lists Clal's, Meitav's and Harel's tables in this order.
    for fund in pension_funds() {
        let source = all_sources.next()?;
        let (Some(man), Some(woman)) = (
            by_birth_year(fund.men, age, born),
            by_birth_year(fund.women, age, born),
        ) else {
            continue;
        };
        coefficients.extend([man, woman]);
        payers.push(fund.name);
        used.push(source);
    }
    let source = all_sources.next()?;
    if let Some(row) = HAREL.iter().find(|row| row.age == age) {
        coefficients.extend([row.man.in_year(starts_in), row.woman.in_year(starts_in)]);
        payers.push(Text::new("Harel", "הראל"));
        used.push(source);
    }
    let highest = coefficients.iter().copied().max()?;
    let lowest = coefficients.iter().copied().min()?;
    Some(Estimate {
        low: sum / highest,
        high: sum / lowest,
        assumes: assumes(&payers, age),
        sources: used,
    })
}

/// "According to the conversion coefficients published by A, B and C, for
/// age 67, …".
fn assumes(payers: &[Text], age: u32) -> Text {
    let list = |lang: fn(&Text) -> &str, and: &str, comma: &str| match payers {
        [] => String::new(),
        [one] => lang(one).to_owned(),
        [init @ .., last] => {
            let init: Vec<&str> = init.iter().map(lang).collect();
            format!("{}{and}{}", init.join(comma), lang(last))
        }
    };
    let en = list(|t| &t.en, " and ", ", ");
    let he = list(|t| &t.he, " ו", ", ");
    let guarantee = if age > LONGEST_FULL_GUARANTEE {
        (
            "with payments guaranteed until 87 at the pension fund and for 240 months at the insurer",
            "עם תשלומים מובטחים עד גיל\u{a0}87 בקרן הפנסיה ו\u{2011}240 חודשים מובטחים בחברת הביטוח",
        )
    } else {
        ("with 240 guaranteed months", "עם 240\u{a0}חודשים מובטחים")
    };
    Text::owned(
        format!(
            "By the conversion coefficients {en} publish, for a man or a woman starting at {age}, {}, and nothing for a spouse.",
            guarantee.0
        ),
        format!(
            "לפי מקדמי ההמרה שמפרסמות {he}, לגבר או לאישה שמתחילים לקבל קצבה בגיל\u{a0}{age}, {}, וללא קצבה לבן או לבת זוג.",
            guarantee.1
        ),
    )
}

/// The oldest age at which a pension fund's table still guarantees 240
/// months: its guarantee may not run past 87.
const LONGEST_FULL_GUARANTEE: u32 = 67;

/// The tables, in the order `sources.md` lists them.
#[must_use]
pub fn sources() -> Vec<Page> {
    let page = |en: &'static str, he: &'static str, url: &str| Page {
        name: Text::new(en, he),
        url: url.to_owned(),
    };
    vec![
        page(
            "Clal Pensia Mashlima's regulations, April 2024 edition: appendix 7-1 (240 months, no survivors) and 7-6 (from 68)",
            "תקנון כלל פנסיה משלימה, מהדורת אפריל 2024: נספח ז׳\u{2011}1 (240 חודשים, ללא שאירים) ונספח ז׳\u{2011}6 (מגיל\u{a0}68)",
            "https://www.clalbit.co.il/media/29581/%D7%AA%D7%A7%D7%A0%D7%95%D7%9F-%D7%9B%D7%9C%D7%9C-%D7%A4%D7%A0%D7%A1%D7%99%D7%94-%D7%9E%D7%A9%D7%9C%D7%99%D7%9E%D7%94-%D7%90%D7%A4%D7%A8%D7%99%D7%9C-2024-101432.pdf",
        ),
        page(
            "Meitav Pensia Klalit's regulations, June 2025 edition: appendix 12-9 (unmarried, 20 years guaranteed)",
            "תקנון מיטב פנסיה כללית, מהדורת 06/2025: נספח יב׳\u{2011}9 (לא נשוי, הבטחה ל\u{2011}20 שנים)",
            "https://www.prisha.co.il/UserFiles/File/pdf/takanon/meitav2.pdf",
        ),
        page(
            "Harel's conversion coefficients for its More policy, 2024, not guaranteed: appendix A (240 payments guaranteed)",
            "מקדמי ההמרה לקצבה של הראל בפוליסת More, 2024, לא מובטחים: נספח א׳ (240 תשלומים מובטחים)",
            "https://www.harel-group.co.il/long-term-savings/independent-and-employees/policies/DocLib/%D7%94%D7%A8%D7%90%D7%9C%20More%20%D7%9C%D7%A9%D7%9B%D7%99%D7%A8%D7%99%D7%9D/%D7%9E%D7%A7%D7%93%D7%9E%D7%99%20%D7%94%D7%9E%D7%A8%D7%94%20%D7%9C%D7%A7%D7%A6%D7%91%D7%94%20%D7%9E%D7%95%D7%A8%202024.pdf",
        ),
    ]
}

/// A pension fund's coefficients as its regulations print them: a row per
/// age, a column per birth year (every fifth), the columns ascending.
type ByBirthYear = &'static [(u32, &'static [(i32, Decimal)])];

struct PensionFund {
    name: Text,
    men: ByBirthYear,
    women: ByBirthYear,
}

/// The coefficient for `age` and a birth year, straight between the two
/// columns around it. A birth year past the table's columns takes the
/// nearest one: the columns move by about half a percent in five years, so
/// the nearest is close, and inventing a trend beyond them would be a guess.
/// `None` where the fund prints no row for the age.
fn by_birth_year(table: ByBirthYear, age: u32, born: i32) -> Option<Decimal> {
    let (_, columns) = table.iter().find(|(row, _)| *row == age)?;
    let (first, last) = (columns.first()?, columns.last()?);
    if born <= first.0 {
        return Some(first.1);
    }
    if born >= last.0 {
        return Some(last.1);
    }
    columns.windows(2).find_map(|pair| {
        let [(from, a), (to, b)] = pair else {
            return None;
        };
        (*from..=*to)
            .contains(&born)
            .then(|| *a + (*b - *a) * Decimal::from(born - from) / Decimal::from(to - from))
    })
}

fn pension_funds() -> [PensionFund; 2] {
    [
        PensionFund {
            name: Text::new("Clal Pensia Mashlima", "כלל פנסיה משלימה"),
            men: CLAL_MEN,
            women: CLAL_WOMEN,
        },
        PensionFund {
            name: Text::new("Meitav Pensia Klalit", "מיטב פנסיה כללית"),
            men: MEITAV_MEN,
            women: MEITAV_WOMEN,
        },
    ]
}

/// An insurer's coefficient for the year its table is dated, and the share
/// it grows by for each year after: in the year Y it is
/// `coefficient × (1 + growth × (Y − 2024))`.
#[derive(Clone, Copy)]
struct Grows {
    coefficient: Decimal,
    growth: Percent,
}

impl Grows {
    /// The table's own year. Its formula is given only from it on, so an
    /// earlier year takes the table as printed.
    const BASE_YEAR: i32 = 2024;

    fn in_year(self, year: i32) -> Decimal {
        let years = Decimal::from((year - Self::BASE_YEAR).max(0));
        self.coefficient + self.growth.of(self.coefficient) * years
    }
}

struct InsurerRow {
    age: u32,
    man: Grows,
    woman: Grows,
}

const fn harel(
    age: u32,
    man: Decimal,
    man_growth: Decimal,
    woman: Decimal,
    woman_growth: Decimal,
) -> InsurerRow {
    InsurerRow {
        age,
        man: Grows {
            coefficient: man,
            growth: Percent(man_growth),
        },
        woman: Grows {
            coefficient: woman,
            growth: Percent(woman_growth),
        },
    }
}

/// Harel's More policy, 2024, not guaranteed, for those who joined from
/// 1 January 2025 before turning 60 (the table's own heading): 240 payments
/// guaranteed at every age. Its growth column is in percent a year: 0.149%
/// at 67 for a man.
const HAREL: &[InsurerRow] = &[
    harel(60, dec!(226.70), dec!(0.133), dec!(232.77), dec!(0.163)),
    harel(61, dec!(223.16), dec!(0.136), dec!(229.17), dec!(0.169)),
    harel(62, dec!(219.59), dec!(0.140), dec!(225.52), dec!(0.174)),
    harel(63, dec!(215.99), dec!(0.142), dec!(221.84), dec!(0.180)),
    harel(64, dec!(212.38), dec!(0.145), dec!(218.15), dec!(0.184)),
    harel(65, dec!(208.80), dec!(0.147), dec!(214.47), dec!(0.189)),
    harel(66, dec!(205.26), dec!(0.148), dec!(210.82), dec!(0.193)),
    harel(67, dec!(201.81), dec!(0.149), dec!(207.22), dec!(0.195)),
    harel(68, dec!(198.47), dec!(0.148), dec!(203.70), dec!(0.197)),
    harel(69, dec!(195.29), dec!(0.146), dec!(200.29), dec!(0.197)),
    harel(70, dec!(192.29), dec!(0.143), dec!(197.02), dec!(0.196)),
];

/// Clal Pensia Mashlima (the supplementary fund, which takes money from a
/// provident fund for investment for a pension: its regulations, "קופת גמל
/// להשקעה"). To 67, appendix 7-1's "240 months, no survivors"; from 68,
/// appendix 7-6's "no spouse, guaranteed until 87", since 240 months from 68
/// would pass 87. Birth years the fund leaves blank (0.00: already past
/// that age when the table was written) are left out.
const CLAL_MEN: ByBirthYear = &[
    (
        60,
        &[
            (1965, dec!(212.85)),
            (1970, dec!(213.96)),
            (1975, dec!(215.00)),
            (1980, dec!(215.99)),
            (1985, dec!(216.95)),
            (1990, dec!(217.87)),
            (1995, dec!(218.76)),
            (2000, dec!(219.61)),
            (2005, dec!(220.43)),
        ],
    ),
    (
        61,
        &[
            (1965, dec!(209.68)),
            (1970, dec!(210.76)),
            (1975, dec!(211.77)),
            (1980, dec!(212.74)),
            (1985, dec!(213.68)),
            (1990, dec!(214.58)),
            (1995, dec!(215.45)),
            (2000, dec!(216.29)),
            (2005, dec!(217.10)),
        ],
    ),
    (
        62,
        &[
            (1960, dec!(205.36)),
            (1965, dec!(206.50)),
            (1970, dec!(207.54)),
            (1975, dec!(208.53)),
            (1980, dec!(209.47)),
            (1985, dec!(210.38)),
            (1990, dec!(211.27)),
            (1995, dec!(212.12)),
            (2000, dec!(212.94)),
            (2005, dec!(213.73)),
        ],
    ),
    (
        63,
        &[
            (1960, dec!(202.26)),
            (1965, dec!(203.34)),
            (1970, dec!(204.34)),
            (1975, dec!(205.29)),
            (1980, dec!(206.20)),
            (1985, dec!(207.09)),
            (1990, dec!(207.94)),
            (1995, dec!(208.77)),
            (2000, dec!(209.57)),
            (2005, dec!(210.34)),
        ],
    ),
    (
        64,
        &[
            (1960, dec!(199.20)),
            (1965, dec!(200.22)),
            (1970, dec!(201.18)),
            (1975, dec!(202.08)),
            (1980, dec!(202.96)),
            (1985, dec!(203.81)),
            (1990, dec!(204.63)),
            (1995, dec!(205.43)),
            (2000, dec!(206.19)),
            (2005, dec!(206.94)),
        ],
    ),
    (
        65,
        &[
            (1960, dec!(196.21)),
            (1965, dec!(197.17)),
            (1970, dec!(198.07)),
            (1975, dec!(198.92)),
            (1980, dec!(199.76)),
            (1985, dec!(200.57)),
            (1990, dec!(201.35)),
            (1995, dec!(202.11)),
            (2000, dec!(202.84)),
            (2005, dec!(203.55)),
        ],
    ),
    (
        66,
        &[
            (1960, dec!(193.30)),
            (1965, dec!(194.19)),
            (1970, dec!(195.03)),
            (1975, dec!(195.84)),
            (1980, dec!(196.62)),
            (1985, dec!(197.39)),
            (1990, dec!(198.13)),
            (1995, dec!(198.84)),
            (2000, dec!(199.54)),
            (2005, dec!(200.21)),
        ],
    ),
    (
        67,
        &[
            (1960, dec!(190.51)),
            (1965, dec!(191.33)),
            (1970, dec!(192.11)),
            (1975, dec!(192.86)),
            (1980, dec!(193.58)),
            (1985, dec!(194.30)),
            (1990, dec!(194.99)),
            (1995, dec!(195.66)),
            (2000, dec!(196.31)),
            (2005, dec!(196.93)),
        ],
    ),
    (
        68,
        &[
            (1960, dec!(185.47)),
            (1965, dec!(186.32)),
            (1970, dec!(187.11)),
            (1975, dec!(187.89)),
            (1980, dec!(188.64)),
            (1985, dec!(189.37)),
            (1990, dec!(190.09)),
            (1995, dec!(190.78)),
            (2000, dec!(191.45)),
            (2005, dec!(192.09)),
        ],
    ),
    (
        69,
        &[
            (1960, dec!(180.27)),
            (1965, dec!(181.14)),
            (1970, dec!(181.96)),
            (1975, dec!(182.75)),
            (1980, dec!(183.53)),
            (1985, dec!(184.29)),
            (1990, dec!(185.02)),
            (1995, dec!(185.73)),
            (2000, dec!(186.42)),
            (2005, dec!(187.09)),
        ],
    ),
    (
        70,
        &[
            (1960, dec!(174.89)),
            (1965, dec!(175.78)),
            (1970, dec!(176.63)),
            (1975, dec!(177.45)),
            (1980, dec!(178.25)),
            (1985, dec!(179.03)),
            (1990, dec!(179.79)),
            (1995, dec!(180.52)),
            (2000, dec!(181.23)),
            (2005, dec!(181.92)),
        ],
    ),
];

const CLAL_WOMEN: ByBirthYear = &[
    (
        60,
        &[
            (1965, dec!(222.35)),
            (1970, dec!(223.42)),
            (1975, dec!(224.42)),
            (1980, dec!(225.36)),
            (1985, dec!(226.25)),
            (1990, dec!(227.10)),
            (1995, dec!(227.90)),
            (2000, dec!(228.66)),
            (2005, dec!(229.38)),
        ],
    ),
    (
        61,
        &[
            (1965, dec!(219.07)),
            (1970, dec!(220.13)),
            (1975, dec!(221.12)),
            (1980, dec!(222.05)),
            (1985, dec!(222.94)),
            (1990, dec!(223.78)),
            (1995, dec!(224.58)),
            (2000, dec!(225.34)),
            (2005, dec!(226.05)),
        ],
    ),
    (
        62,
        &[
            (1960, dec!(214.64)),
            (1965, dec!(215.76)),
            (1970, dec!(216.80)),
            (1975, dec!(217.78)),
            (1980, dec!(218.70)),
            (1985, dec!(219.57)),
            (1990, dec!(220.41)),
            (1995, dec!(221.19)),
            (2000, dec!(221.94)),
            (2005, dec!(222.65)),
        ],
    ),
    (
        63,
        &[
            (1960, dec!(211.33)),
            (1965, dec!(212.43)),
            (1970, dec!(213.44)),
            (1975, dec!(214.40)),
            (1980, dec!(215.30)),
            (1985, dec!(216.16)),
            (1990, dec!(216.98)),
            (1995, dec!(217.75)),
            (2000, dec!(218.49)),
            (2005, dec!(219.18)),
        ],
    ),
    (
        64,
        &[
            (1960, dec!(208.02)),
            (1965, dec!(209.08)),
            (1970, dec!(210.07)),
            (1975, dec!(210.99)),
            (1980, dec!(211.87)),
            (1985, dec!(212.71)),
            (1990, dec!(213.51)),
            (1995, dec!(214.27)),
            (2000, dec!(214.99)),
            (2005, dec!(215.66)),
        ],
    ),
    (
        65,
        &[
            (1960, dec!(204.72)),
            (1965, dec!(205.74)),
            (1970, dec!(206.69)),
            (1975, dec!(207.58)),
            (1980, dec!(208.43)),
            (1985, dec!(209.25)),
            (1990, dec!(210.02)),
            (1995, dec!(210.75)),
            (2000, dec!(211.45)),
            (2005, dec!(212.11)),
        ],
    ),
    (
        66,
        &[
            (1960, dec!(201.46)),
            (1965, dec!(202.42)),
            (1970, dec!(203.33)),
            (1975, dec!(204.18)),
            (1980, dec!(205.00)),
            (1985, dec!(205.78)),
            (1990, dec!(206.53)),
            (1995, dec!(207.23)),
            (2000, dec!(207.90)),
            (2005, dec!(208.54)),
        ],
    ),
    (
        67,
        &[
            (1960, dec!(198.24)),
            (1965, dec!(199.15)),
            (1970, dec!(200.01)),
            (1975, dec!(200.82)),
            (1980, dec!(201.60)),
            (1985, dec!(202.35)),
            (1990, dec!(203.05)),
            (1995, dec!(203.73)),
            (2000, dec!(204.37)),
            (2005, dec!(204.97)),
        ],
    ),
    (
        68,
        &[
            (1960, dec!(193.46)),
            (1965, dec!(194.39)),
            (1970, dec!(195.28)),
            (1975, dec!(196.12)),
            (1980, dec!(196.92)),
            (1985, dec!(197.69)),
            (1990, dec!(198.42)),
            (1995, dec!(199.11)),
            (2000, dec!(199.77)),
            (2005, dec!(200.39)),
        ],
    ),
    (
        69,
        &[
            (1960, dec!(188.51)),
            (1965, dec!(189.48)),
            (1970, dec!(190.38)),
            (1975, dec!(191.25)),
            (1980, dec!(192.08)),
            (1985, dec!(192.87)),
            (1990, dec!(193.62)),
            (1995, dec!(194.33)),
            (2000, dec!(195.01)),
            (2005, dec!(195.66)),
        ],
    ),
    (
        70,
        &[
            (1960, dec!(183.40)),
            (1965, dec!(184.39)),
            (1970, dec!(185.33)),
            (1975, dec!(186.22)),
            (1980, dec!(187.07)),
            (1985, dec!(187.89)),
            (1990, dec!(188.66)),
            (1995, dec!(189.40)),
            (2000, dec!(190.09)),
            (2005, dec!(190.76)),
        ],
    ),
];

/// Meitav Pensia Klalit (the general fund), appendix 12-9: unmarried, 20
/// years guaranteed, with a 0.3% management fee assumed. It prints ages 60 to
/// 67 only ("full tables are at the fund's offices"), so from 68 the range
/// is Clal's and Harel's.
const MEITAV_MEN: ByBirthYear = &[
    (
        60,
        &[
            (1965, dec!(213.83)),
            (1970, dec!(214.91)),
            (1975, dec!(215.93)),
            (1980, dec!(216.89)),
            (1985, dec!(217.81)),
            (1990, dec!(218.70)),
            (1995, dec!(219.56)),
            (2000, dec!(220.39)),
            (2005, dec!(221.18)),
            (2010, dec!(221.94)),
        ],
    ),
    (
        61,
        &[
            (1965, dec!(210.63)),
            (1970, dec!(211.68)),
            (1975, dec!(212.67)),
            (1980, dec!(213.61)),
            (1985, dec!(214.52)),
            (1990, dec!(215.40)),
            (1995, dec!(216.24)),
            (2000, dec!(217.06)),
            (2005, dec!(217.84)),
            (2010, dec!(218.58)),
        ],
    ),
    (
        62,
        &[
            (1965, dec!(207.41)),
            (1970, dec!(208.43)),
            (1975, dec!(209.39)),
            (1980, dec!(210.31)),
            (1985, dec!(211.20)),
            (1990, dec!(212.06)),
            (1995, dec!(212.88)),
            (2000, dec!(213.68)),
            (2005, dec!(214.44)),
            (2010, dec!(215.17)),
        ],
    ),
    (
        63,
        &[
            (1965, dec!(204.21)),
            (1970, dec!(205.19)),
            (1975, dec!(206.12)),
            (1980, dec!(207.00)),
            (1985, dec!(207.86)),
            (1990, dec!(208.70)),
            (1995, dec!(209.50)),
            (2000, dec!(210.27)),
            (2005, dec!(211.02)),
            (2010, dec!(211.73)),
        ],
    ),
    (
        64,
        &[
            (1965, dec!(201.03)),
            (1970, dec!(201.96)),
            (1975, dec!(202.85)),
            (1980, dec!(203.71)),
            (1985, dec!(204.53)),
            (1990, dec!(205.33)),
            (1995, dec!(206.11)),
            (2000, dec!(206.85)),
            (2005, dec!(207.57)),
            (2010, dec!(208.26)),
        ],
    ),
    (
        65,
        &[
            (1960, dec!(196.95)),
            (1965, dec!(197.90)),
            (1970, dec!(198.78)),
            (1975, dec!(199.62)),
            (1980, dec!(200.44)),
            (1985, dec!(201.23)),
            (1990, dec!(201.99)),
            (1995, dec!(202.73)),
            (2000, dec!(203.45)),
            (2005, dec!(204.14)),
            (2010, dec!(204.79)),
        ],
    ),
    (
        66,
        &[
            (1960, dec!(193.95)),
            (1965, dec!(194.84)),
            (1970, dec!(195.67)),
            (1975, dec!(196.46)),
            (1980, dec!(197.23)),
            (1985, dec!(197.97)),
            (1990, dec!(198.70)),
            (1995, dec!(199.40)),
            (2000, dec!(200.07)),
            (2005, dec!(200.73)),
            (2010, dec!(201.35)),
        ],
    ),
    (
        67,
        &[
            (1960, dec!(191.07)),
            (1965, dec!(191.88)),
            (1970, dec!(192.65)),
            (1975, dec!(193.38)),
            (1980, dec!(194.10)),
            (1985, dec!(194.80)),
            (1990, dec!(195.47)),
            (1995, dec!(196.13)),
            (2000, dec!(196.76)),
            (2005, dec!(197.38)),
            (2010, dec!(197.96)),
        ],
    ),
];

const MEITAV_WOMEN: ByBirthYear = &[
    (
        60,
        &[
            (1965, dec!(222.47)),
            (1970, dec!(223.52)),
            (1975, dec!(224.49)),
            (1980, dec!(225.40)),
            (1985, dec!(226.27)),
            (1990, dec!(227.09)),
            (1995, dec!(227.87)),
            (2000, dec!(228.61)),
            (2005, dec!(229.30)),
            (2010, dec!(229.95)),
        ],
    ),
    (
        61,
        &[
            (1965, dec!(219.15)),
            (1970, dec!(220.19)),
            (1975, dec!(221.15)),
            (1980, dec!(222.06)),
            (1985, dec!(222.92)),
            (1990, dec!(223.74)),
            (1995, dec!(224.51)),
            (2000, dec!(225.25)),
            (2005, dec!(225.94)),
            (2010, dec!(226.59)),
        ],
    ),
    (
        62,
        &[
            (1965, dec!(215.80)),
            (1970, dec!(216.81)),
            (1975, dec!(217.76)),
            (1980, dec!(218.66)),
            (1985, dec!(219.51)),
            (1990, dec!(220.32)),
            (1995, dec!(221.09)),
            (2000, dec!(221.81)),
            (2005, dec!(222.50)),
            (2010, dec!(223.14)),
        ],
    ),
    (
        63,
        &[
            (1965, dec!(212.41)),
            (1970, dec!(213.40)),
            (1975, dec!(214.33)),
            (1980, dec!(215.21)),
            (1985, dec!(216.05)),
            (1990, dec!(216.85)),
            (1995, dec!(217.60)),
            (2000, dec!(218.32)),
            (2005, dec!(219.00)),
            (2010, dec!(219.63)),
        ],
    ),
    (
        64,
        &[
            (1965, dec!(209.00)),
            (1970, dec!(209.97)),
            (1975, dec!(210.88)),
            (1980, dec!(211.74)),
            (1985, dec!(212.56)),
            (1990, dec!(213.34)),
            (1995, dec!(214.08)),
            (2000, dec!(214.78)),
            (2005, dec!(215.44)),
            (2010, dec!(216.06)),
        ],
    ),
    (
        65,
        &[
            (1960, dec!(204.61)),
            (1965, dec!(205.61)),
            (1970, dec!(206.54)),
            (1975, dec!(207.41)),
            (1980, dec!(208.25)),
            (1985, dec!(209.04)),
            (1990, dec!(209.80)),
            (1995, dec!(210.52)),
            (2000, dec!(211.20)),
            (2005, dec!(211.84)),
            (2010, dec!(212.45)),
        ],
    ),
    (
        66,
        &[
            (1960, dec!(201.28)),
            (1965, dec!(202.23)),
            (1970, dec!(203.12)),
            (1975, dec!(203.96)),
            (1980, dec!(204.76)),
            (1985, dec!(205.53)),
            (1990, dec!(206.26)),
            (1995, dec!(206.95)),
            (2000, dec!(207.60)),
            (2005, dec!(208.23)),
            (2010, dec!(208.81)),
        ],
    ),
    (
        67,
        &[
            (1960, dec!(198.01)),
            (1965, dec!(198.91)),
            (1970, dec!(199.75)),
            (1975, dec!(200.55)),
            (1980, dec!(201.31)),
            (1985, dec!(202.04)),
            (1990, dec!(202.74)),
            (1995, dec!(203.40)),
            (2000, dec!(204.02)),
            (2005, dec!(204.62)),
            (2010, dec!(205.17)),
        ],
    ),
];
