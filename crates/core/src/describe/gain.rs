//! "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7) and the words of "ואם התאריך
//! יזוז?" (§2.9): the sentence with the deposits, the takeaway, the
//! horizon rows' condition, range and its meaning.
//!
//! Every figure is rounded to how sure it is (principle 13): the gain is
//! worked out at the expected return, and the rows' ranges from the past.

use rust_decimal::Decimal;

use super::card::{about, both, joined_to};
use super::reasons::shekels;
use crate::guide::{Amount, Flexibility};
use crate::{Lang, Percent, Text};

/// A figure rounded as [`about`] rounds it, for the bars' labels: to three
/// significant figures (principle 13).
#[must_use]
pub fn rounded(amount: Decimal) -> Decimal {
    super::card::three_figures(amount)
}

/// The sentence that gives the deposits, which the bars don't draw: the
/// sum the visitor will put in, and what it is in today's money.
#[must_use]
pub fn lead(deposited: Amount) -> Text {
    // Within ₪10 the two are one figure: no inflation, or no time.
    let same = (deposited.nominal - deposited.today).abs() < Decimal::TEN;
    both(|lang| match (lang, same) {
        (Lang::En, true) => format!(
            "You'll put in {}. This is the gain above it, and where it goes in each:",
            shekels(deposited.nominal)
        ),
        (Lang::He, true) => format!(
            "תפקידו {}. זה הרווח שמעליהם, ולאן הוא הולך בכל אחת מהאפשרויות:",
            shekels(deposited.nominal)
        ),
        (Lang::En, false) => format!(
            "You'll put in {} (about {} in today's money). This is the gain above it, and where it goes in each:",
            shekels(deposited.nominal),
            about(deposited.today)
        ),
        (Lang::He, false) => format!(
            "תפקידו {} (בערך {} בכסף של היום). זה הרווח שמעליהם, ולאן הוא הולך בכל אחת מהאפשרויות:",
            shekels(deposited.nominal),
            about(deposited.today)
        ),
    })
}

/// The visitor's bank's row: "הבנק שלכם: הפועלים" (principle 15).
#[must_use]
pub fn your_bank(provider: &Text) -> Text {
    both(|lang| match lang {
        Lang::En => format!("Your bank: {}", &provider[lang]),
        Lang::He => format!("הבנק שלכם: {}", &provider[lang]),
    })
}

/// Until the visitor names their bank, the bank row is the bank that leaves
/// most, as every other comparison on the page measures from it (the
/// review's N4), and says so: "הבנק הזול מבין הבנקים: לאומי".
#[must_use]
pub fn cheapest_bank(provider: &Text) -> Text {
    both(|lang| match lang {
        Lang::En => format!("The cheapest bank: {}", &provider[lang]),
        Lang::He => format!("הבנק הזול מבין הבנקים: {}", &provider[lang]),
    })
}

/// For a portfolio at a bank not named, the bank row is the cheapest
/// branch price list, and says so.
#[must_use]
pub fn cheapest_branch(provider: &Text) -> Text {
    both(|lang| match lang {
        Lang::En => format!("The cheapest branch price list: {}", &provider[lang]),
        Lang::He => format!("תעריף הסניף הזול מבין הבנקים: {}", &provider[lang]),
    })
}

/// A place's row beside another way's: the way first, "קרן סל · אינטראקטיב".
#[must_use]
pub fn way_at(way: crate::ways::Way, place: &Text) -> Text {
    both(|lang| {
        let short = super::card::way_short(way, lang);
        let mut chars = short.chars();
        let short = match (lang, chars.next()) {
            (Lang::En, Some(first)) => first.to_uppercase().collect::<String>() + chars.as_str(),
            _ => short.to_owned(),
        };
        format!("{short}{}{}", lang.pick(" · ", "\u{a0}· "), &place[lang])
    })
}

/// One option's gain, for the takeaway.
#[derive(Debug, Clone, Copy)]
pub struct Part {
    pub kept: Decimal,
    pub tax: Decimal,
    /// The broker's and the manager's fees, apart from the fund's own cost
    /// (§4.7: the bars draw it as a part of its own).
    pub fees: Decimal,
    pub product: Decimal,
}

impl Part {
    fn total(self) -> Decimal {
        self.kept + self.tax + self.fees + self.product
    }
}

/// A gain part's name on the bar (§4.7).
#[must_use]
pub fn part_label(which: crate::guide::GainPartKind) -> Text {
    use crate::guide::GainPartKind;
    match which {
        GainPartKind::Kept => Text::new("Yours", "נשאר לכם"),
        GainPartKind::Tax => Text::new("Tax", "מס"),
        GainPartKind::Fees => Text::new("Fees", "עמלות"),
        GainPartKind::Fund => Text::new("The fund itself", "הקרן עצמה"),
    }
}

/// A gain part's amount, already rounded: "₪292,000".
#[must_use]
pub fn part_amount(today: Decimal) -> Text {
    both(|_| shekels(today))
}

/// Beside a row whose gain pays no tax.
#[must_use]
pub fn untaxed() -> Text {
    Text::new("tax-free", "פטורה ממס")
}

/// "2.7 times", or nothing where the ratio is too near 1 to say.
fn times(more: Decimal, less: Decimal) -> Option<Decimal> {
    if less <= Decimal::ZERO {
        return None;
    }
    let ratio = more / less;
    (ratio >= Decimal::new(15, 1)).then(|| {
        if ratio >= Decimal::TEN {
            ratio.round()
        } else {
            ratio.round_dp(1)
        }
    })
}

/// What the bars come to, in one sentence: where the difference between
/// them comes from. It names no amount: the bars carry them, and the
/// decision above carries the difference (each number once, principle 13).
#[must_use]
pub fn takeaway(first_name: &Text, first: Part, second_name: &Text, second: Part) -> Text {
    let ((better_name, better), (worse_name, worse)) = if first.kept >= second.kept {
        ((first_name, first), (second_name, second))
    } else {
        ((second_name, second), (first_name, first))
    };
    // What each part gives the better one: fees it doesn't pay, tax it doesn't.
    let by_fees = worse.fees - better.fees >= worse.tax - better.tax;
    let smaller_whole = worse.total() < better.total();
    let ratio = times(worse.fees, better.fees);
    let untaxed = better.tax.is_zero();
    let dearer = better.fees > worse.fees;
    both(|lang| {
        let (better_name, worse_name) = (&better_name[lang], &worse_name[lang]);
        match (lang, by_fees) {
            (Lang::En, true) => {
                let how_much = match ratio {
                    Some(ratio) => format!("{ratio} times as much as"),
                    None => "more than".to_owned(),
                };
                let mut text = format!(
                    "The difference comes mostly from fees: at {worse_name} they take {how_much} at {better_name}, and what they take no longer grows."
                );
                if smaller_whole {
                    text.push_str(" So the whole gain is smaller there too.");
                }
                text
            }
            (Lang::He, true) => {
                let how_much = match ratio {
                    Some(ratio) => format!("פי {ratio} מאשר"),
                    None => "יותר מאשר".to_owned(),
                };
                let mut text = format!(
                    "ההבדל בא בעיקר מהעמלות: {} הן לוקחות {how_much} {}, ומה שהן לוקחות כבר לא צומח.",
                    joined_to('ב', worse_name),
                    joined_to('ב', better_name)
                );
                if smaller_whole {
                    text.push_str(" לכן גם הרווח כולו קטן שם יותר.");
                }
                text
            }
            (Lang::En, false) => {
                let tax = if untaxed {
                    format!("at {better_name} the gain pays no tax")
                } else {
                    format!("at {better_name} the tax takes less of the gain")
                };
                let fees = if dearer {
                    ", and that is worth more than its higher fees"
                } else {
                    ""
                };
                format!("The difference comes mostly from tax: {tax}{fees}.")
            }
            (Lang::He, false) => {
                let tax = if untaxed {
                    format!("{} אין מס על הרווח", joined_to('ב', better_name))
                } else {
                    format!("{} המס לוקח פחות מהרווח", joined_to('ב', better_name))
                };
                let fees = if dearer {
                    ", וזה שווה יותר מהעמלות הגבוהות יותר שם"
                } else {
                    ""
                };
                format!("ההבדל בא בעיקר מהמס: {tax}{fees}.")
            }
        }
    })
}

// ─────────────────────────── The horizons ───────────────────────────

/// The answer about the date a fit is for, while the visitor hasn't given
/// it (§4.10), before what fits: "אם תצטרכו בדיוק אז:".
#[must_use]
pub fn condition(flexibility: Flexibility) -> Text {
    match flexibility {
        Flexibility::Fixed => Text::new("If you'll need it exactly then:", "אם תצטרכו בדיוק אז:"),
        Flexibility::CanWait => Text::new("If you can wait:", "אם תוכלו לחכות:"),
    }
}

/// A row's date: "בעוד 5 שנים".
#[must_use]
pub fn when(months: u32) -> Text {
    both(|lang| format!("{} {}", lang.pick("In", "בעוד"), super::reasons::span(months, lang)))
}

/// On the visitor's own row.
#[must_use]
pub fn your_date() -> Text {
    Text::new("Your date", "התאריך שלכם")
}

/// A row's deposits by then, in today's money, under "הפקדתם": "₪28,600".
#[must_use]
pub fn deposits(deposited: Amount) -> Text {
    both(|_| about(deposited.today))
}

/// Several ways the visitor chooses between: "קרן כספית, פיקדון בריבית
/// קבועה או אג״ח צמודה של מדינת ישראל".
#[must_use]
pub fn either(names: &[Text]) -> Text {
    both(|lang| {
        let names: Vec<&str> = names.iter().map(|name| &name[lang]).collect();
        match names.as_slice() {
            [] => String::new(),
            [one] => (*one).to_owned(),
            [rest @ .., last] => {
                let or = lang.pick(" or ", " או ");
                format!("{}{or}{last}", rest.join(", "))
            }
        }
    })
}

/// What a row's range is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOf {
    /// Every sum is known today.
    Known,
    /// One way, its sum moving with the rate.
    Rate,
    /// Several safe ways, the visitor's to choose.
    Ways { rate: bool },
    /// The index, through the past's worst and middle stretches.
    Past,
    /// The index where the past doesn't cover the years: the expected return.
    Expected(Percent),
}

/// The outcome as words: "₪15,960", or "₪9,100 עד ₪33,400" (a hyphen
/// between two amounts is reordered by the bidirectional algorithm).
#[must_use]
pub fn range(low: Amount, high: Amount) -> Text {
    let (low, high) = (low.today, high.today);
    both(|lang| {
        if high - low < Decimal::TEN {
            about(low)
        } else {
            match lang {
                Lang::En => format!("{} to {}", about(low), about(high)),
                Lang::He => format!("{} עד {}", about(low), about(high)),
            }
        }
    })
}

/// What a row's range means: how sure it is, said where it is read.
#[must_use]
pub fn range_note(of: RangeOf) -> Text {
    match of {
        RangeOf::Known => Text::new("Known today", "ידוע כבר היום"),
        RangeOf::Rate => Text::new("Depends on the rate", "לפי הריבית"),
        RangeOf::Ways { rate: false } => Text::new("By the way you choose", "לפי הדרך שתבחרו"),
        RangeOf::Ways { rate: true } => Text::new(
            "By the way you choose, and the rate",
            "לפי הדרך שתבחרו והריבית",
        ),
        RangeOf::Past => Text::new(
            "From the worst stretch since 1926 to the middle one, after fees and tax, by the S&P 500 index as an example",
            "מהתקופה הגרועה ביותר מאז 1926 ועד האמצעית, אחרי עמלות ומס, לפי מדד S&P\u{a0}500 כדוגמה",
        ),
        RangeOf::Expected(rate) => both(|lang| match lang {
            Lang::En => format!("If the broad stock index rises {rate} a year"),
            Lang::He => format!("אם מדד המניות הרחב יעלה {rate} בשנה"),
        }),
    }
}

