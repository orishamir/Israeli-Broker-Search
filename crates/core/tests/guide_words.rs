//! The guide's words (`describe/reasons.rs`, `describe/steps.rs`): every
//! answer the guide can give has text in both languages with no placeholder
//! left in it, and every kind has steps. Wording itself is not tested.

use std::sync::LazyLock;

use broker_fees::describe::answer::{current_account_line, left_note, what};
use broker_fees::describe::reasons::{
    basis_sentence, cushion_text, history_sentence, safety_note, safety_tag, steps_for,
    unlock_text, what_safety_costs,
};
use broker_fees::guide::*;
use broker_fees::ways::Way;
use broker_fees::{ExchangeRates, Lang};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

static DATA: LazyLock<Data> = LazyLock::new(Data::listed);

fn settings() -> Settings {
    Settings::new(ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap())
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

/// Answers that between them reach every row of the table.
fn many_answers() -> Vec<Answers> {
    let mut all = Vec::new();
    for months in [6, 24, 48, 60, 84, 120, 204] {
        for flexibility in [Flexibility::Fixed, Flexibility::CanWait] {
            for (now, monthly) in [(dec!(50_000), dec!(0)), (dec!(10_000), dec!(2_000))] {
                let mut a = answers(now, monthly, Horizon::Months(months));
                a.flexibility = Some(flexibility);
                all.push(a);
            }
        }
    }
    let mut any_day = answers(dec!(30_000), dec!(0), Horizon::Months(6));
    any_day.needs_any_day = true;
    all.push(any_day);
    for (age, retire_at) in [(40, 60), (40, 55)] {
        all.push(answers(
            dec!(0),
            dec!(1_500),
            Horizon::Retirement { age, retire_at },
        ));
    }
    let mut cushion = answers(dec!(0), dec!(3_000), Horizon::Unknown);
    cushion.cushion = Some(Cushion::HasNot {
        monthly_expenses: Some(dec!(17_000)),
    });
    all.push(cushion);
    let mut has = answers(dec!(0), dec!(3_000), Horizon::Unknown);
    has.cushion = Some(Cushion::Has);
    all.push(has);
    all.push(answers(dec!(15_000), dec!(0), Horizon::Unknown));
    let mut portfolio = answers(dec!(200_000), dec!(0), Horizon::Months(240));
    portfolio.situation = Some(Situation::ExistingPortfolio {
        bank: Some(Bank::Leumi),
    });
    all.push(portfolio);
    let mut knows = answers(dec!(10_000), dec!(2_000), Horizon::Months(240));
    knows.situation = Some(Situation::KnowsWhatToBuy);
    all.push(knows);
    all
}

fn clean(text: &str, what: &str) {
    assert!(!text.trim().is_empty(), "{what} is empty");
    assert!(
        !text.contains('{') && !text.contains('}'),
        "{what} has a placeholder left: {text}"
    );
}

#[test]
fn every_reason_and_step_has_text_in_both_languages() {
    for answers in many_answers() {
        let rec = recommend(&answers, &settings(), &DATA).unwrap();
        for reason in &rec.reasons {
            for lang in [Lang::En, Lang::He] {
                clean(&reason.text[lang], &format!("{:?}", reason.from));
            }
        }
        assert!(!rec.steps.is_empty(), "{answers:?}");
        for step in &rec.steps {
            for lang in [Lang::En, Lang::He] {
                clean(&step.title[lang], "a step's title");
                clean(&step.text[lang], "a step");
            }
        }
        for lang in [Lang::En, Lang::He] {
            let spans = what(rec.kind, &rec.choice, rec.months, &answers.horizon, lang);
            assert!(spans.iter().any(|part| part.bold), "{answers:?}");
            for part in &spans {
                clean(&part.text, "the what-sentence");
            }
            let note = left_note(rec.left, rec.months, Some(2026), lang);
            clean(&note.today, "the amount's note");
            clean(
                &current_account_line(rec.current_account, lang),
                "the base line",
            );
        }
        if let Some(unlock) = &rec.unlock {
            for lang in [Lang::En, Lang::He] {
                clean(&unlock_text(unlock, lang), "the unlock line");
            }
        }
        if let Some(plan) = &rec.two_parts {
            for lang in [Lang::En, Lang::He] {
                let (first, second) = cushion_text(plan, dec!(3_000), lang);
                for part in first.iter().chain(&second) {
                    clean(&part.text, "the cushion");
                }
            }
        }
        if let Some(band) = &rec.band {
            for lang in [Lang::En, Lang::He] {
                clean(&history_sentence(band, lang), "the history");
            }
        }
    }
}

#[test]
fn every_kind_has_steps() {
    let base = recommend(
        &answers(dec!(50_000), dec!(1_000), Horizon::Months(60)),
        &settings(),
        &DATA,
    )
    .unwrap();
    let mut kinds = vec![
        Kind::MoneyMarketFund,
        Kind::FixedDeposit,
        Kind::LinkedDeposit,
        Kind::GovernmentBond { linked: false },
        Kind::GovernmentBond { linked: true },
        Kind::CurrentAccount,
    ];
    kinds.extend(Way::for_index().into_iter().map(Kind::Way));
    for kind in kinds {
        let mut rec = base.clone();
        rec.kind = kind;
        for lang in [Lang::En, Lang::He] {
            let steps = steps_for(&rec, lang);
            assert!(!steps.is_empty(), "{kind:?}");
            for step in steps {
                clean(&step.title, "a title");
                clean(&step.text, "a step");
            }
        }
    }
}

#[test]
fn every_safety_has_a_tag_and_a_note() {
    for safety in [
        Safety::SumKnown,
        Safety::RateChanges,
        Safety::RealValueKnown,
        Safety::CanFall,
    ] {
        for lang in [Lang::En, Lang::He] {
            clean(safety_tag(safety, lang), "a tag");
            clean(safety_note(safety, lang), "a note");
        }
    }
    let amount = |today| Amount {
        today,
        nominal: today,
    };
    for (safe, index) in [(dec!(100), dec!(200)), (dec!(200), dec!(100))] {
        for lang in [Lang::En, Lang::He] {
            clean(
                &what_safety_costs(amount(safe), amount(index), 120, lang),
                "what safety costs",
            );
        }
    }
}

/// The word מדד, wherever it stands alone: "המדד", "למדד", but not
/// "מדד S&P 500", "מדד המחירים לצרכן", a stock index as a kind ("מדד
/// מניות"), other indices as such ("מדדים", "מדד אחר"), a name quoted as
/// the bank shows it (״צמוד מדד״), or "נמדד" (measured).
/// One sentence is a unit of its own: a bare מדד may refer back to the one
/// index named earlier in it ([`bare_index_in_unit`]).
fn bare_index(text: &str) -> Option<String> {
    bare_index_in_unit(&[text])
}

/// The same rule over one unit (REDESIGN.md §2.8, amended 2026-10-20): an
/// option card, a stated line, a chart row with its label, an identity line.
/// There a bare מדד may refer back to an index named earlier in the unit,
/// but only when exactly one index (the S&P 500 or the consumer price
/// index) is named there; anywhere else it stands alone.
fn bare_index_in_unit(parts: &[&str]) -> Option<String> {
    const WORDS: [&str; 10] = [
        "מדד",
        "המדד",
        "למדד",
        "במדד",
        "ממדד",
        "ומדד",
        "והמדד",
        "שהמדד",
        "כשהמדד",
        "שבמדד",
    ];
    // A stock index named as a kind ("מדד מניות רחב", "מדד המניות הרחב",
    // "מדד רחב"): the S&P 500 is its example (principle 17), so either
    // names the stock index a bare מדד may refer back to.
    const STOCKS: [&str; 4] = ["מניות", "המניות", "רחב", "S&P"];
    // Named, but not an index the words can refer back to.
    const OTHER: [&str; 1] = ["אחר"];
    let text = parts.join(" ");
    let hebrew_letter = |c: char| ('\u{5d0}'..='\u{5ea}').contains(&c);
    let (mut sp500, mut prices) = (false, false);
    let mut quoted = false;
    let mut asked_which = false;
    let mut word = String::new();
    let mut chars = text.char_indices().peekable();
    let mut previous = ' ';
    while let Some((at, c)) = chars.next() {
        // A quote mark, not the gershayim inside a word ("אג״ח").
        let inside_a_word =
            hebrew_letter(previous) && chars.peek().is_some_and(|(_, next)| hebrew_letter(*next));
        previous = c;
        if c == '״' && !inside_a_word {
            quoted = !quoted;
        }
        if !hebrew_letter(c) {
            continue;
        }
        word.push(c);
        if chars.peek().is_some_and(|(_, next)| hebrew_letter(*next)) {
            continue;
        }
        let after = text[at + c.len_utf8()..].trim_start_matches([' ', '\u{a0}']);
        // "איזה מדד": the question which index, about indexes as such.
        let which = std::mem::replace(&mut asked_which, word == "איזה");
        if !quoted && !which && WORDS.contains(&word.as_str()) {
            if STOCKS.iter().any(|name| after.starts_with(name))
                || after.starts_with("ה\u{2011}S&P")
            {
                sp500 = true;
            } else if after.starts_with("המחירים") {
                prices = true;
            } else if !OTHER.iter().any(|name| after.starts_with(name)) && sp500 == prices {
                // Neither named yet, or both: nothing to refer back to.
                return Some(word);
            }
        }
        word.clear();
    }
    None
}

/// The units of an answer, each as its Hebrew strings in order: the lines'
/// sentences, each option card, each chart row with its label, each
/// identity line; every other string is a unit of its own.
fn units(answer: &Answer) -> Vec<Vec<String>> {
    let he = |text: &broker_fees::Text| text.he.to_string();
    let mut units: Vec<Vec<String>> = Vec::new();
    // The plan and its amount are one unit (§4.5): the sentence names the
    // index the amount's "המדד" refers back to.
    let mut plan: Vec<String> = answer.plan.iter().map(he).collect();
    if let Some(amount) = &answer.plan_amount {
        plan.extend([he(&amount.amount), he(&amount.when), he(&amount.basis)]);
        plan.extend(amount.example.iter().map(he));
    }
    units.push(plan);
    for step in &answer.path {
        units.push(vec![he(&step.label), he(&step.choice), he(&step.hinge)]);
        if let Some(why) = &step.why_clear {
            let mut unit = vec![he(&why.lead), he(&why.count), he(&why.crises)];
            unit.extend(
                why.amounts
                    .iter()
                    .flat_map(|figure| [he(&figure.label), he(&figure.amount)]),
            );
            unit.push(he(&why.past));
            units.push(unit);
        }
    }
    for line in &answer.lines {
        match &line.state {
            LineState::Stated { text, .. } => units.push(vec![he(text)]),
            LineState::Decision(decision) => {
                units.push(vec![he(&decision.question)]);
                units.push(vec![he(&decision.why)]);
                units.extend(decision.difference.iter().map(|text| vec![he(text)]));
                for option in &decision.options {
                    let mut card = vec![he(&option.title), he(&option.amount)];
                    card.extend(option.basis.iter().map(he));
                    card.extend(option.facts.iter().map(he));
                    card.extend(option.fits_if.iter().map(he));
                    card.push(he(&option.button));
                    units.push(card);
                }
            }
            LineState::Waiting => {}
        }
    }
    for row in &answer.outcome.rows {
        let mut unit = vec![he(&row.name)];
        unit.extend(row.tags.iter().map(he));
        unit.extend(row.note.iter().map(he));
        // A stock row's return field is said in the row's own words: "אם
        // המדד יעלה" only where its name names the index (§4.6).
        if matches!(row.id, Kind::Way(_)) {
            unit.push(he(&broker_fees::describe::outcome::return_lead(row.id)));
        }
        units.push(unit);
        let mut rest = Vec::new();
        hebrew_strings(&serde_json::to_value(&row.mark).unwrap(), &mut rest);
        hebrew_strings(&serde_json::to_value(&row.excluded).unwrap(), &mut rest);
        units.extend(rest.into_iter().map(|text| vec![text]));
    }
    let mut rest = Vec::new();
    for how in &answer.how {
        units.push(how.identity.iter().map(he).collect());
        units.push(facts_unit(how));
        let mut how = how.clone();
        how.identity.clear();
        how.facts.clear();
        hebrew_strings(&serde_json::to_value(&how).unwrap(), &mut rest);
    }
    let mut others = answer.clone();
    others.lines.clear();
    others.outcome.rows.clear();
    others.how.clear();
    others.plan = None;
    others.plan_amount = None;
    others.path.clear();
    hebrew_strings(&serde_json::to_value(&others).unwrap(), &mut rest);
    units.extend(rest.into_iter().map(|text| vec![text]));
    units
}

/// A product's facts (§4.8) are one unit, under its name: "כמו המדד" in
/// "כמה מרוויחה" refers back to the index "מה זה" names.
fn facts_unit(how: &broker_fees::guide::How) -> Vec<String> {
    std::iter::once(how.name.he.to_string())
        .chain(
            how.facts
                .iter()
                .flat_map(|fact| [fact.question.he.to_string(), fact.answer.he.to_string()]),
        )
        .collect()
}

/// Every Hebrew string inside a serialized value: a `Text`'s `he`, or a
/// string written in the page's language.
fn hebrew_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text)
            if text.chars().any(|c| ('\u{5d0}'..='\u{5ea}').contains(&c)) =>
        {
            out.push(text.clone());
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| hebrew_strings(item, out)),
        serde_json::Value::Object(fields) => {
            for (key, field) in fields {
                // A `Text`'s English may name the Hebrew term in brackets.
                if key != "en" {
                    hebrew_strings(field, out);
                }
            }
        }
        _ => {}
    }
}

fn no_bare_index(value: &serde_json::Value, what: &str) {
    let mut texts = Vec::new();
    hebrew_strings(value, &mut texts);
    for text in texts {
        assert!(
            bare_index(&text).is_none(),
            "{what} says a bare מדד: {text}"
        );
    }
}

#[test]
fn the_check_finds_a_bare_index() {
    assert_eq!(
        bare_index("לא נוגעים. אם המדד בירידה, מחכים.").as_deref(),
        Some("המדד")
    );
    assert_eq!(bare_index("קרן מחקה מדד מניות רחב").as_deref(), None);
    assert_eq!(
        bare_index("קרן סל על מדד, בתל אביב").as_deref(),
        Some("מדד")
    );
    assert_eq!(bare_index("צמודה למדד המחירים לצרכן"), None);
    assert_eq!(bare_index("קרן על מדד S&P\u{a0}500"), None);
    assert_eq!(bare_index("״פיקדון צמוד מדד״, 3 שנים"), None);
    assert_eq!(bare_index("מה שאבד לעמלות נמדד לפני מס"), None);
    // Within a unit: a reference back to the one index named before it…
    assert_eq!(
        bare_index_in_unit(&[
            "צמיחה: קרן על מדד S&P\u{a0}500",
            "₪19,925 אם המדד יעלה 10% בשנה"
        ]),
        None
    );
    // …but not before it is named, nor where two indexes are.
    assert_eq!(
        bare_index_in_unit(&["אם המדד יעלה 10% בשנה", "קרן על מדד S&P\u{a0}500"]).as_deref(),
        Some("המדד")
    );
    assert_eq!(
        bare_index_in_unit(&[
            "אג״ח צמודה למדד המחירים לצרכן",
            "קרן על מדד S&P\u{a0}500",
            "אם המדד יעלה"
        ])
        .as_deref(),
        Some("המדד")
    );
    assert_eq!(
        bare_index_in_unit(&["קרן סל על מדד, בתל אביב"]).as_deref(),
        Some("מדד")
    );
}

/// The redesigned answers the checks below read: every row of the safe
/// and the index tables, with and without the visitor's bank, and with
/// choices that open every line.
fn many_redesigned_answers() -> &'static [Answer] {
    // Worked out once for both tests that read them: it takes seconds.
    static ALL: LazyLock<Vec<Answer>> = LazyLock::new(work_out_many_redesigned_answers);
    &ALL
}

fn work_out_many_redesigned_answers() -> Vec<Answer> {
    let mut dated = settings();
    dated.today = Some(time::macros::date!(2026 - 10 - 01));
    let picks = [
        Choices::default(),
        Choices {
            safe_way: Some(SafeWay::Bond),
            wrapper: Some(Wrapper::Anyone),
            account: Some(Account::YourBank),
            holdings: None,
        },
        Choices {
            safe_way: Some(SafeWay::Deposit),
            wrapper: Some(Wrapper::StudyFund),
            account: Some(Account::NewAccount),
            holdings: None,
        },
    ];
    let mut all = Vec::new();
    for mut answers in many_answers() {
        for bank in [None, Some(Bank::Hapoalim)] {
            answers.bank = bank;
            for choices in picks {
                all.push(answer_with(&answers, &choices, &dated, &DATA, true).unwrap());
            }
        }
    }
    all
}

/// "מדד" never stands alone (REDESIGN.md §2.8, principle 14): the guide
/// says which, the S&P 500 or the consumer price index. Here: "how to do
/// it" and where to buy for every row, the previous answer while it is
/// still shown, the kinds' names and the learn cards.
#[test]
fn the_guide_names_the_index_it_means() {
    for answers in many_answers() {
        let rec = recommend(&answers, &settings(), &DATA).unwrap();
        let lang = Lang::He;
        for reason in &rec.reasons {
            assert!(
                bare_index(&reason.text[lang]).is_none(),
                "a reason: {}",
                &reason.text[lang]
            );
        }
        for line in &rec.ranked {
            let name = &line.kind.text()[lang];
            assert!(bare_index(name).is_none(), "a kind: {name}");
        }
        for step in steps_for(&rec, lang) {
            assert!(bare_index(&step.text).is_none(), "a step: {}", step.text);
            assert!(bare_index(&step.title).is_none(), "a step: {}", step.title);
        }
        for part in what(rec.kind, &rec.choice, rec.months, &answers.horizon, lang) {
            assert!(
                bare_index(&part.text).is_none(),
                "the what-sentence: {}",
                part.text
            );
        }
        if let Some(band) = &rec.band {
            assert!(bare_index(&history_sentence(band, lang)).is_none());
            assert!(bare_index(basis_sentence(lang)).is_none());
        }
    }
    for answer in many_redesigned_answers() {
        for how in &answer.how {
            let unit = facts_unit(how);
            let parts: Vec<&str> = unit.iter().map(String::as_str).collect();
            assert!(bare_index_in_unit(&parts).is_none(), "the facts: {unit:?}");
            let mut rest = how.clone();
            rest.facts.clear();
            no_bare_index(&serde_json::to_value(&rest).unwrap(), "how to do it");
        }
    }
    for kind in Way::for_index().into_iter().map(Kind::Way) {
        assert!(bare_index(&kind.text().he).is_none(), "{}", kind.text().he);
    }
    no_bare_index(
        &serde_json::to_value(broker_fees::describe::glossary(Lang::He)).unwrap(),
        "a learn card",
    );
}

/// The same rule over the answer card's words (`describe/card.rs`,
/// `describe/outcome.rs`) and the chips (`situations.rs`): never "אם המדד
/// יעלה 10% בשנה", "המדד נגמר בהפסד" or "עוברים למדד".
#[test]
fn the_answer_card_names_the_index_it_means() {
    for answer in many_redesigned_answers() {
        for unit in units(answer) {
            let parts: Vec<&str> = unit.iter().map(String::as_str).collect();
            assert!(
                bare_index_in_unit(&parts).is_none(),
                "the answer says a bare מדד: {unit:?}"
            );
        }
        // What to buy is a broad stock index; the S&P 500 is named only
        // where it is what was measured, as an example (principle 17, §2.13
        // item 1): never as a product's index or track. "How to do it"
        // keeps its search box (§4.11), which names examples on it.
        let mut card = answer.clone();
        card.how.clear();
        let value = serde_json::to_value(&card).unwrap();
        let mut texts = Vec::new();
        hebrew_strings(&value, &mut texts);
        for text in texts {
            for bought in [
                "על מדד S&P",
                "מסלול מדד S&P",
                "S&P\u{a0}500, לטווח",
                "S&P\u{a0}500 ל\u{2011}",
            ] {
                assert!(
                    !text.contains(bought),
                    "the answer names the S&P 500 as bought: {text}"
                );
            }
        }
    }
    for preset in broker_fees::situations::situations() {
        no_bare_index(&serde_json::to_value(&preset.name).unwrap(), "a chip");
        no_bare_index(
            &serde_json::to_value(&preset.explanation).unwrap(),
            "a chip's tip",
        );
        no_bare_index(
            &serde_json::to_value(&preset.lesson).unwrap(),
            "a chip's lesson",
        );
    }
}

/// The sentence about moving a portfolio has both languages, no placeholder,
/// names the sale's tax as an upper bound, and rests on linked pages.
#[test]
fn the_move_sentence_says_the_saving_the_transfer_and_the_sale() {
    use broker_fees::describe::answer::{move_portfolio_line, move_portfolio_sources};
    let rec = recommend(
        &Answers {
            situation: Some(Situation::ExistingPortfolio {
                bank: Some(Bank::Leumi),
            }),
            ..answers(dec!(200_000), dec!(0), Horizon::Months(240))
        },
        &settings(),
        &DATA,
    )
    .unwrap();
    let moved = rec.move_portfolio.unwrap();
    for lang in [Lang::En, Lang::He] {
        let line = move_portfolio_line(&moved, rec.months, lang);
        clean(&line, "the move sentence");
        assert!(line.contains("ניוד"), "{line}");
        assert!(line.contains("25%"), "{line}");
        assert!(!line.contains("S&P"), "{line}");
    }
    let sources = move_portfolio_sources();
    assert!(sources.len() >= 2);
    assert!(sources.iter().all(|page| page.url.starts_with("https://")));
}
