//! The page's sentences about money in the answer and the questions: what
//! was chosen (in the terms of the bank's app), the amount's notes, the
//! follow-up cards, the bank stand-in note and the arenas' names. They live
//! here, not in the page's texts, because each says something the guide
//! computed (CLAUDE.md: keep the logic in Rust).
//!
//! A sentence the page may want to emphasise comes as [`Span`]s, so the page
//! never has to parse markup out of a string.

use super::reasons::{shekels, span};
use crate::guide::{AlsoPossible, Amount, Arena, Choice, FollowUp, Horizon, Kind, MovePortfolio};
use crate::ways::Way;
use crate::{Lang, Named, Product, Vehicle};

/// A piece of a sentence, and whether it is the thing named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Span {
            text: text.into(),
            bold: false,
        }
    }

    pub fn bold(text: impl Into<String>) -> Self {
        Span {
            text: text.into(),
            bold: true,
        }
    }
}

/// What the place is called after the kind: "(the cheapest: X · Y)". A name
/// that only repeats the kind, as a fund's `plan` does, is left out.
fn place(kind: Kind, choice: &Choice, lang: Lang) -> String {
    let provider = &choice.provider[lang];
    let plan = &choice.plan[lang];
    let named = if plan.is_empty() || *plan == *kind.name(lang) || plan == provider {
        provider.to_owned()
    } else {
        format!("{provider}{}· {plan}", lang.pick(" ", "\u{a0}"))
    };
    // A fund's or a bank's name says which it is already ("הקרן הזולה: ..."),
    // a broker's account is the cheapest of several.
    match kind {
        Kind::GovernmentBond { .. } => lang.pick("the cheapest: ", "הזול: ").to_owned() + &named,
        Kind::Way(way) if way.vehicle == Vehicle::Brokerage => {
            lang.pick("the cheapest: ", "הזול: ").to_owned() + &named
        }
        // A fund's or a policy's name is the kind's again, with its fee.
        Kind::Way(_) => String::new(),
        _ => named,
    }
}

/// What was chosen, in the terms the saver meets in the bank's app and in
/// ads: the kind first (bold), then what it means for this money.
#[must_use]
pub fn what(kind: Kind, choice: &Choice, months: u32, horizon: &Horizon, lang: Lang) -> Vec<Span> {
    let placed = place(kind, choice, lang);
    let (head, rest) = match kind {
        Kind::MoneyMarketFund => (
            lang.pick("Money market fund (קרן כספית)", "קרן כספית").to_owned(),
            lang.pick(
                ", which you can sell on any trading day, and whose interest follows the Bank of Israel's",
                ", שאפשר למכור בכל יום מסחר, והריבית שלה עוקבת אחרי ריבית בנק ישראל",
            )
            .to_owned(),
        ),
        Kind::FixedDeposit | Kind::LinkedDeposit => (
            kind.name(lang).to_owned(),
            match (kind, lang) {
                (Kind::LinkedDeposit, Lang::He) => format!(
                    " ל{}, שהקרן שלו צמודה למדד המחירים לצרכן, ושהכסף נעול בו עד סופו",
                    span(months, lang)
                ),
                (Kind::LinkedDeposit, Lang::En) => format!(
                    " for {}, whose principal follows the consumer price index, with the money locked in until the end",
                    span(months, lang)
                ),
                (_, Lang::He) => format!(
                    " ל{}, שהבנק קובע את הריבית שלו מראש ושהכסף נעול בו עד סופו",
                    span(months, lang)
                ),
                (_, Lang::En) => format!(
                    " for {}, whose rate the bank fixes in advance, with the money locked in until the end",
                    span(months, lang)
                ),
            },
        ),
        Kind::GovernmentBond { .. } => (
            kind.name(lang).to_owned(),
            lang.pick(
                " held until it is repaid, in ",
                " שמחזיקים עד שהוא נפדה, בעוד ",
            )
            .to_owned()
                + &span(months, lang)
                + lang.pick(", through a brokerage account", ", דרך חשבון מסחר"),
        ),
        Kind::Way(way) => way_sentence(way, horizon, lang),
        Kind::CurrentAccount => (
            kind.name(lang).to_owned(),
            lang.pick(", where it earns nothing", ", שבו הוא לא מרוויח כלום")
                .to_owned(),
        ),
    };
    let mut spans = vec![Span::bold(head), Span::plain(rest)];
    if placed.is_empty() {
        spans.push(Span::plain("."));
    } else {
        spans.push(Span::plain(format!(" ({placed}).")));
    }
    spans
}

fn way_sentence(way: Way, horizon: &Horizon, lang: Lang) -> (String, String) {
    match (way.product, way.vehicle) {
        (Product::IsraeliIndexFund, Vehicle::Brokerage) => (
            lang.pick(
                "Index fund (קרן מחקה) on a broad stock index",
                "קרן מחקה מדד מניות רחב",
            )
            .to_owned(),
            lang.pick(
                ", as it appears in the bank's app, on the Tel Aviv exchange",
                ", כמו שהיא מופיעה באפליקציה של הבנק, בבורסה בתל אביב",
            )
            .to_owned(),
        ),
        (Product::IsraeliEtf, Vehicle::Brokerage) => (
            lang.pick(
                "ETF (קרן סל) on a broad stock index",
                "קרן סל על מדד מניות רחב",
            )
            .to_owned(),
            lang.pick(", on the Tel Aviv exchange", ", בבורסה בתל אביב")
                .to_owned(),
        ),
        (Product::ForeignEtfInTelAviv, Vehicle::Brokerage) => (
            lang.pick(
                "ETF (קרן סל) on a broad stock index",
                "קרן סל על מדד מניות רחב",
            )
            .to_owned(),
            lang.pick(
                ", registered in Ireland and traded in shekels in Tel Aviv",
                ", רשומה באירלנד ונסחרת בשקלים בתל אביב",
            )
            .to_owned(),
        ),
        (Product::UsFund, Vehicle::Brokerage) => (
            lang.pick(
                "US ETF (קרן סל) on a broad stock index",
                "קרן סל אמריקאית על מדד מניות רחב",
            )
            .to_owned(),
            lang.pick(", on the US exchange", ", בבורסה בארה״ב")
                .to_owned(),
        ),
        (Product::IrishFund, Vehicle::Brokerage) => (
            lang.pick(
                "Irish ETF (קרן סל) on a broad stock index",
                "קרן סל אירית על מדד מניות רחב",
            )
            .to_owned(),
            lang.pick(", on a European exchange", ", בבורסה באירופה")
                .to_owned(),
        ),
        _ => {
            // A fund or a policy: the track is named by `Way::kind_name`.
            let pension = match (*horizon, way.vehicle) {
                (Horizon::Retirement { retire_at, .. }, Vehicle::InvestmentGemel)
                    if retire_at >= 60 =>
                {
                    match lang {
                        Lang::He => format!(", שתימשך כקצבה חודשית מגיל {retire_at}"),
                        Lang::En => format!(
                            ", which can be taken as a monthly pension from age {retire_at}"
                        ),
                    }
                }
                _ => String::new(),
            };
            (way.kind_name(lang).to_owned(), pension)
        }
    }
}

/// The plan that leaves more but may cost more, said beside the answer: who,
/// how much more, and why it was not the one chosen, in the caveat's own
/// summary (so the sentence says how sure the price is).
#[must_use]
pub fn also_possible_line(also: &AlsoPossible, lang: Lang) -> String {
    let dot = lang.pick(" · ", "\u{a0}· ");
    let who = format!(
        "{}{dot}{}",
        &also.choice.provider[lang], &also.choice.plan[lang]
    );
    // A difference of under a shekel is not worth a number.
    let more = if also.ahead_by.today < rust_decimal::Decimal::ONE {
        lang.pick("a little more", "מעט יותר").to_owned()
    } else {
        match lang {
            Lang::En => format!("about {} more", shekels(also.ahead_by.today)),
            Lang::He => format!("כ־{} יותר", shekels(also.ahead_by.today)),
        }
    };
    match lang {
        Lang::En => format!(
            "Also possible: {who}, which leaves {more}, but \u{26a0} {}.",
            // "May cost more: ..." in the middle of a sentence.
            also.warning
                .en
                .replacen("May cost more", "may cost more", 1)
        ),
        Lang::He => format!(
            "אפשר גם: {who}, שמשאירה {more}, אבל \u{26a0} {}.",
            &also.warning[lang]
        ),
    }
}

/// What moving a portfolio from the saver's bank to the plan named saves,
/// and what selling first would cost, in words. The saving is the engines':
/// the same money in the bank's branch plan and in the plan named. The tax is
/// an upper bound (a sale pays tax on the gain only, and the gain is not
/// known here), and a transfer in kind is described as the banks' own pages
/// describe it (see [`move_portfolio_sources`]), not as the Tax Authority's.
#[must_use]
pub fn move_portfolio_line(m: &MovePortfolio, months: u32, lang: Lang) -> String {
    let dot = lang.pick(" · ", "\u{a0}· ");
    let to = format!("{}{dot}{}", &m.to.provider[lang], &m.to.plan[lang]);
    match lang {
        Lang::En => format!(
            "Moving the portfolio from {} to {to} leaves about {} more after {} (in today's \
             money, the same money in both). Securities can be moved as they are, a transfer \
             of securities between houses (ניוד), to an account in your own name, which the \
             banks' pages say is not a sale, so nothing is taxed when it moves. Selling first \
             does pay tax on the gain: at most {} of tax on {} (25% of the gain, and more with \
             the surtax; the gain is not the whole value), and if you know what the securities \
             cost, the tax is a quarter of the difference. Not every security can be moved, \
             mostly those held abroad: the house you move to says which.",
            &m.from.provider[lang],
            shekels(m.saves.today),
            span(months, lang),
            shekels(m.sale_tax_at_most),
            shekels(m.value),
        ),
        Lang::He => format!(
            "העברת התיק מ{} אל {to} משאירה בערך {} יותר אחרי {} (בכסף של היום, אותו כסף בשניהם). אפשר להעביר את הניירות כמו שהם, העברת ניירות ערך בין בתי השקעות (ניוד), לחשבון על שם אותו בעלים, ולפי האתרים של הבנקים זו לא מכירה, ולכן אין מס כשהם עוברים. מכירה לפני כן כן משלמת מס על הרווח: לכל היותר {} מס על {} (25% מהרווח, ויותר עם מס יסף; הרווח הוא לא כל הסכום), ומי שיודע כמה שילם על הניירות משלם רבע מההפרש. לא כל נייר אפשר להעביר, בעיקר ניירות שמוחזקים בחו״ל: הבית שמעבירים אליו אומר איזה.",
            &m.from.provider[lang],
            shekels(m.saves.today),
            span(months, lang),
            shekels(m.sale_tax_at_most),
            shekels(m.value),
        ),
    }
}

/// The pages the transfer rule rests on: banks' and houses' own pages (the
/// Tax Authority's own guidance was not found; `policies/sources.md` says so).
#[must_use]
pub fn move_portfolio_sources() -> Vec<crate::Page> {
    let page = |en: &'static str, he: &'static str, url: &str| crate::Page {
        name: crate::Text::new(en, he),
        url: url.to_owned(),
    };
    vec![
        page(
            "Bank Leumi: transferring securities between accounts",
            "בנק לאומי: העברת ניירות ערך בין חשבונות",
            "https://www.leumi.co.il/he/account-to-account-transfer",
        ),
        page(
            "A-S Invest: portfolio transfer FAQ",
            "אס-אינוסט: שאלות נפוצות על ניוד תיק",
            "https://www.as-invest.co.il/trade/faq_transfer/",
        ),
    ]
}

/// The note under the amount: what the saver is shown, and the same sum in
/// the money of the year it arrives, when that differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftNote {
    pub today: String,
    pub nominal: Option<String>,
}

#[must_use]
pub fn left_note(left: Amount, months: u32, this_year: Option<i32>, lang: Lang) -> LeftNote {
    let differs = left.nominal.round_dp(0) != left.today.round_dp(0);
    let nominal = this_year.filter(|_| differs).map(|year| {
        let arrives = year + i32::try_from(months.div_ceil(12)).unwrap_or(0);
        match lang {
            Lang::En => format!("{} in {arrives}'s shekels", shekels(left.nominal)),
            Lang::He => format!("{} בשקלים של {arrives}", shekels(left.nominal)),
        }
    });
    LeftNote {
        today: lang
            .pick(
                "You would have this, in today's money, after tax",
                "יישארו לכם בכסף של היום, אחרי מס",
            )
            .to_owned(),
        nominal,
    }
}

/// What the same deposits come to if the money sits in a current account.
#[must_use]
pub fn current_account_line(amount: Amount, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "If the money sat in a current account: {} in today's money",
            shekels(amount.today)
        ),
        Lang::He => format!("אם הכסף יישב בעו״ש: {} בכסף של היום", shekels(amount.today)),
    }
}

/// What "another bank" is priced as.
#[must_use]
pub fn bank_stand_in_note(lang: Lang) -> &'static str {
    lang.pick(
        "For another bank, the dearest branch price list we found is shown, so as not to show less than you would pay.",
        "לבנק אחר מוצג מחירון הסניף היקר שמצאנו, כדי לא להציג פחות ממה שתשלמו.",
    )
}

/// The two arenas, as the saver chooses between them.
#[must_use]
pub fn arena_name(arena: Arena, lang: Lang) -> &'static str {
    match arena {
        Arena::Safe => lang.pick("A safer sum", "סכום בטוח יותר"),
        Arena::Index => lang.pick("An index, for the long run", "מדד, לטווח ארוך"),
    }
}

/// A follow-up question as a card: its title, why it is asked, the answers
/// offered and the number it takes, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowCard {
    pub title: String,
    pub text: String,
    /// The buttons, in the order of the answers they stand for.
    pub answers: Vec<String>,
    /// The label of the number field the card holds.
    pub field: Option<String>,
}

#[must_use]
pub fn follow_card(follow_up: FollowUp, lang: Lang) -> FollowCard {
    let card = |title: &str, text: &str, answers: &[&str], field: Option<&str>| FollowCard {
        title: title.to_owned(),
        text: text.to_owned(),
        answers: answers.iter().map(|&each| each.to_owned()).collect(),
        field: field.map(str::to_owned),
    };
    match follow_up {
        FollowUp::Flexibility => card(
            lang.pick(
                "If the market is down right then, could you wait another year or two?",
                "אם הבורסה תהיה בירידה בדיוק אז, תוכלו לחכות עוד שנה-שנתיים?",
            ),
            lang.pick(
                "When you need the money on a fixed date, like a payment on an apartment, a 20% fall in the market that year is a problem. When you can wait, it usually passes.",
                "כשצריך את הכסף בתאריך קבוע, כמו תשלום על דירה, ירידה של 20% בבורסה באותה שנה היא בעיה. כשאפשר לחכות, היא בדרך כלל חולפת.",
            ),
            &[
                lang.pick("No, exactly then", "לא, צריך בדיוק אז"),
                lang.pick("Yes, I can wait", "כן, אפשר לחכות"),
            ],
            None,
        ),
        FollowUp::AgeAndRetirement => card(
            lang.pick("For retirement: how old are you?", "לפרישה: בני כמה אתם?"),
            lang.pick(
                "Your age sets when the money can be taken as a monthly pension, and so how many years it can grow.",
                "הגיל קובע מתי אפשר למשוך את הכסף כקצבה חודשית, ולכן כמה שנים הוא יכול לצמוח.",
            ),
            &[],
            None,
        ),
        FollowUp::Cushion => card(
            lang.pick(
                "Do you already have money set aside for 3 to 6 months of expenses?",
                "יש לכם כבר כסף בצד ל־3–6 חודשי הוצאות?",
            ),
            lang.pick(
                "A cushion comes first: money within reach for a layoff or a sudden expense, so you never have to sell an investment while the market is down.",
                "כרית ביטחון היא הדבר הראשון: כסף זמין למקרה של פיטורים או הוצאה פתאומית, כדי שלא תצטרכו למכור השקעה בזמן ירידות.",
            ),
            &[
                lang.pick("Yes", "כן"),
                lang.pick("Not yet", "עוד לא"),
            ],
            None,
        ),
        FollowUp::MonthlyExpenses => card(
            "",
            "",
            &[],
            Some(lang.pick("My expenses in a month, about", "ההוצאות שלי בחודש, בערך")),
        ),
    }
}
