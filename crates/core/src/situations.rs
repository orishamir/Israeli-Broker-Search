//! The situations the guide opens with, each a chip: a reason Israelis save,
//! with the answers it sets and a different lesson. Click one and the
//! guide's answer shows what the page teaches by that example. Each lesson
//! is named in the explanation, and no two are the same.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::Text;
use crate::guide::{Answers, Cushion, Horizon, Situation};

/// One chip.
#[derive(Debug, Clone)]
pub struct Preset {
    /// Stable, for links and tests: the English name in lowercase words.
    pub id: &'static str,
    /// The chip's name.
    pub name: Text,
    /// The situation in words, for its tip.
    pub explanation: Text,
    /// What it teaches, in a sentence.
    pub lesson: Text,
    pub answers: Answers,
    /// The page opens its dialog about where to find the fees of a portfolio
    /// the saver already has.
    pub opens_dialog: bool,
    /// The page shows the workbench, with every way to compare.
    pub workbench: bool,
}

fn answers(now: Decimal, monthly: Decimal, horizon: Horizon) -> Answers {
    Answers {
        situation: None,
        now,
        monthly,
        horizon,
        flexibility: None,
        cushion: None,
        needs_any_day: false,
        bank: None,
    }
}

const fn years(years: u32) -> Horizon {
    Horizon::Months(years * 12)
}

/// The situations, in the order of the chips.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the situations' data and texts")]
pub fn situations() -> Vec<Preset> {
    let preset = |id, name: Text, explanation: Text, lesson: Text, answers: Answers| Preset {
        id,
        name,
        explanation,
        lesson,
        answers,
        opens_dialog: false,
        workbench: false,
    };
    vec![
        preset(
            "salary",
            Text::new("Saving from your salary", "חיסכון מהמשכורת"),
            Text::new(
                "₪500 a month from your salary for 30 years. Over so long, what is charged every year on everything held matters far more than the fee on each purchase.",
                "₪500 בחודש מהמשכורת במשך 30 שנה. לאורך זמן כזה, מה שנגבה כל שנה מכל התיק חשוב הרבה יותר מהעמלה על כל קנייה.",
            ),
            Text::new(
                "A yearly fee compounds against you.",
                "עמלה שנתית פוגעת בצמיחה, שנה אחרי שנה.",
            ),
            answers(dec!(0), dec!(500), years(30)),
        ),
        preset(
            "child",
            Text::new("For a child", "לילד"),
            Text::new(
                "₪300 a month from a child's birth until they are 18: for their studies, a flat or a wedding. Deposits this small are where a minimum fee on each purchase, or a fixed monthly fee, takes a large share.",
                "₪300 בחודש מהלידה ועד גיל 18: ללימודים, לדירה או לחתונה. בהפקדות קטנות כאלה עמלת מינימום על כל קנייה, או דמי טיפול קבועים כל חודש, לוקחים חלק גדול.",
            ),
            Text::new(
                "With small deposits the fixed fees decide.",
                "בהפקדות קטנות העמלות הקבועות הן שמכריעות.",
            ),
            answers(dec!(0), dec!(300), years(18)),
        ),
        preset(
            "goal",
            Text::new("For a flat, or a goal on a date", "לדירה או למטרה בתאריך"),
            Text::new(
                "₪100,000 now and ₪2,000 a month for three years, towards a payment due on a day that can't move. When the date is fixed, what matters is knowing in advance what there will be, not what could be.",
                "₪100,000 עכשיו ועוד ₪2,000 בחודש במשך שלוש שנים, לתשלום שמגיע ביום שאי אפשר להזיז. כשהתאריך קבוע, מה שחשוב הוא לדעת מראש כמה יהיה, ולא כמה אולי יהיה.",
            ),
            Text::new(
                "A fixed date asks for a sum known in advance.",
                "תאריך קבוע מבקש סכום שידוע מראש.",
            ),
            answers(dec!(100_000), dec!(2000), years(3)),
        ),
        preset(
            "cushion",
            Text::new("A safety cushion", "כרית ביטחון"),
            Text::new(
                "₪30,000 put aside for a rainy day, which may be needed on any day. So where the money can be reached the next day matters as much as what it earns.",
                "₪30,000 בצד ליום סגריר, שאולי יהיה בהם צורך בכל יום. לכן חשוב איפה אפשר למשוך את הכסף למחרת, לא פחות ממה שהוא מרוויח.",
            ),
            Text::new(
                "Money you may need any day can't be locked.",
                "כסף שאולי תצטרכו בכל יום אי אפשר לנעול.",
            ),
            Answers {
                needs_any_day: true,
                ..answers(dec!(30_000), dec!(0), Horizon::Months(6))
            },
        ),
        preset(
            "bonus",
            Text::new("A bonus", "בונוס"),
            Text::new(
                "₪15,000 once, for 5 years. Five years is where a date that can move a year or two decides between a sum known almost in advance and growth that can also fall.",
                "₪15,000 חד\u{2011}פעמיים, ל\u{2011}5 שנים. בחמש שנים, תאריך שיכול לזוז בשנה\u{2011}שנתיים מכריע בין סכום שידוע כמעט מראש לבין צמיחה שיכולה גם לרדת.",
            ),
            Text::new(
                "Whether the date can move decides it.",
                "מה שמכריע הוא אם התאריך יכול לזוז.",
            ),
            answers(dec!(15_000), dec!(0), years(5)),
        ),
        preset(
            "inheritance",
            Text::new("An inheritance", "ירושה"),
            Text::new(
                "₪500,000 once, kept for 15 years. A sum this large is more than a provident fund for investment takes in a year, and a large gain pays an extra surtax: the accounts that fit a monthly saver don't all fit it.",
                "₪500,000 חד\u{2011}פעמיים, ל\u{2011}15 שנה. סכום גדול כזה הוא יותר ממה שקופת גמל להשקעה מקבלת בשנה, ורווח גדול משלם גם מס יסף: לא כל החשבונות שמתאימים לחוסך חודשי מתאימים לו.",
            ),
            Text::new(
                "A large sum meets the ceilings and the surtax.",
                "סכום גדול פוגש תקרות ומס יסף.",
            ),
            answers(dec!(500_000), dec!(0), years(15)),
        ),
        preset(
            "retirement",
            Text::new("For retirement", "לפרישה"),
            Text::new(
                "₪3,000 a month from age 45 to 65. Retirement money has a home of its own: taken as a pension from 60, the gain in a provident fund for investment pays no tax, where in a brokerage account it pays 25%.",
                "₪3,000 בחודש מגיל 45 עד 65. לכסף לפרישה יש בית משלו: נמשך כקצבה מגיל 60, הרווח בקופת גמל להשקעה לא משלם מס, ובחשבון מסחר הוא משלם 25%.",
            ),
            Text::new(
                "The way out decides the tax.",
                "הדרך לצאת מהכסף קובעת את המס.",
            ),
            answers(
                dec!(0),
                dec!(3000),
                Horizon::Retirement {
                    age: 45,
                    retire_at: 65,
                },
            ),
        ),
        preset(
            "unknown",
            Text::new("Don't know yet", "עוד לא יודעים"),
            Text::new(
                "₪3,000 a month with no goal and nothing set aside. A cushion of three months of expenses comes first, so a sudden expense doesn't force a sale in a fall; only then does the long term begin.",
                "₪3,000 בחודש בלי מטרה ובלי כסף בצד. קודם כרית של שלושה חודשי הוצאות, כדי שהוצאה פתאומית לא תכריח למכור בירידות, ורק אז מתחיל הטווח הארוך.",
            ),
            Text::new(
                "A cushion comes before investing.",
                "כרית ביטחון באה לפני ההשקעה.",
            ),
            Answers {
                cushion: Some(Cushion::HasNot {
                    monthly_expenses: Some(dec!(17_000)),
                }),
                ..answers(dec!(0), dec!(3000), Horizon::Unknown)
            },
        ),
        Preset {
            opens_dialog: true,
            ..preset(
                "existing",
                Text::new("I already have a portfolio at a bank", "כבר יש לי תיק בבנק"),
                Text::new(
                    "₪200,000 already invested at a bank, for 20 years. The bank's branch prices are compared with the rest, to see what moving would change: transfer the holdings as they are, don't sell them.",
                    "₪200,000 שכבר מושקעים בבנק, ל\u{2011}20 שנה. המחירים של הסניף בבנק מושווים לשאר, כדי לראות מה ישתנה אם עוברים: מעבירים את הניירות כמו שהם, לא מוכרים.",
                ),
                Text::new(
                    "Moving a portfolio can be cheaper than keeping it.",
                    "להעביר תיק יכול לעלות פחות מלהשאיר אותו.",
                ),
                Answers {
                    situation: Some(Situation::ExistingPortfolio { bank: None }),
                    ..answers(dec!(200_000), dec!(0), years(20))
                },
            )
        },
        Preset {
            workbench: true,
            ..preset(
                "workbench",
                Text::new(
                    "I know what I want to buy",
                    "אני יודע / יודעת מה אני רוצה לקנות",
                ),
                Text::new(
                    "The S&P 500 index, ₪10,000 now and ₪2,000 a month for 20 years, with every way to hold it side by side: an index fund or ETF in Tel Aviv, the US or Europe, and the provident funds. The one that leaves the most isn't always the one with the lowest fee.",
                    "מדד S&P\u{a0}500, ₪10,000 עכשיו ועוד ₪2,000 בחודש במשך 20 שנה, וכל הדרכים להחזיק אותו זו לצד זו: קרן מחקה או קרן סל בתל אביב, בארה״ב או באירופה, וקופות הגמל. זו שמשאירה הכי הרבה היא לא תמיד זו שדמי הניהול שלה הנמוכים ביותר.",
                ),
                Text::new(
                    "The way to hold the S&P 500 index can matter more than the broker.",
                    "הדרך להחזיק את מדד S&P\u{a0}500 יכולה להיות חשובה יותר מהבנק.",
                ),
                Answers {
                    situation: Some(Situation::KnowsWhatToBuy),
                    ..answers(dec!(10_000), dec!(2000), years(20))
                },
            )
        },
    ]
}
