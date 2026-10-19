//! "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7) and the horizon section's
//! words. First drafts.

use rust_decimal::Decimal;

use super::card::{about, both};
use super::reasons::shekels;
use crate::{Lang, Text};

/// The sentence that gives the deposits, which the bars don't draw.
#[must_use]
pub fn lead(deposited: Decimal) -> Text {
    both(|lang| match lang {
        Lang::En => format!(
            "You'll put in {}, in today's money. This is the gain above it, before tax and fees, and where it goes:",
            shekels(deposited)
        ),
        Lang::He => format!(
            "תפקידו {}, בכסף של היום. זה הרווח שמעליהם, לפני מס ועמלות, ולאן הוא הולך:",
            shekels(deposited)
        ),
    })
}

/// One part of a gain, for the takeaway.
#[derive(Debug, Clone, Copy)]
pub struct Part {
    pub kept: Decimal,
    pub tax: Decimal,
    /// Fees and the product's own cost.
    pub fees: Decimal,
}

/// What the bars come to, in one sentence: where the difference goes.
#[must_use]
pub fn takeaway(better_name: &Text, better: Part, worse_name: &Text, worse: Part) -> Text {
    let gap = better.kept - worse.kept;
    let by_fees = worse.fees - better.fees >= better.tax - worse.tax;
    both(|lang| {
        let (better_name, worse_name) = (&better_name[lang], &worse_name[lang]);
        match (lang, by_fees) {
            (Lang::En, true) => format!(
                "At {worse_name} the fees take {} instead of {}, so about {} less is left.",
                shekels(worse.fees),
                shekels(better.fees),
                about(gap)
            ),
            (Lang::He, true) => format!(
                "ב{worse_name} העמלות לוקחות {} במקום {}, ולכן נשאר בערך {} פחות.",
                shekels(worse.fees),
                shekels(better.fees),
                about(gap)
            ),
            (Lang::En, false) => format!(
                "{better_name} pays {} in tax instead of {}, so about {} more is left.",
                shekels(better.tax),
                shekels(worse.tax),
                about(gap)
            ),
            (Lang::He, false) => format!(
                "ב{better_name} המס הוא {} במקום {}, ולכן נשאר בערך {} יותר.",
                shekels(better.tax),
                shekels(worse.tax),
                about(gap)
            ),
        }
    })
}
