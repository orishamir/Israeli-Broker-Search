//! The guide's words (`describe/reasons.rs`, `describe/steps.rs`): every
//! answer the guide can give has text in both languages with no placeholder
//! left in it, and every kind has steps. Wording itself is not tested.

use std::sync::LazyLock;

use broker_fees::describe::answer::{current_account_line, follow_card, left_note, what};
use broker_fees::describe::reasons::{
    basis_sentence, cushion_text, history_caveat, history_sentence, safety_note, safety_tag,
    steps_for, unlock_text, what_safety_costs,
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
        monthly_expenses: dec!(17_000),
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
fn every_reason_why_not_and_step_has_text_in_both_languages() {
    for answers in many_answers() {
        let rec = recommend(&answers, &settings(), &DATA).unwrap();
        for reason in &rec.reasons {
            for lang in [Lang::En, Lang::He] {
                clean(&reason.text[lang], &format!("{:?}", reason.from));
            }
        }
        for alternative in &rec.alternatives {
            for lang in [Lang::En, Lang::He] {
                clean(
                    &alternative.why_not.text[lang],
                    &format!("{:?} / {:?}", alternative.kind, alternative.why_not.from),
                );
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
            for follow_up in &rec.follow_ups {
                let card = follow_card(*follow_up, lang);
                if *follow_up != FollowUp::MonthlyExpenses {
                    clean(&card.title, "a follow-up's title");
                    clean(&card.text, "a follow-up's text");
                }
                assert_eq!(
                    card.field.is_some(),
                    *follow_up == FollowUp::MonthlyExpenses
                );
            }
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
                clean(&history_caveat(band, lang), "the history's caveat");
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

/// The site names no index, fund or ticker as a pick (CLAUDE.md, "What
/// this site is for"): what the guide says about its choice, the kinds it
/// didn't choose, the steps and the chips never mention one. The S&P 500
/// appears once, as the basis of the history and the costs
/// (`basis_sentence`, in the history's caveat).
#[test]
fn the_guide_names_no_index_fund_or_ticker() {
    const NAMED: [&str; 7] = [
        "S&P", "VOO", "CSPX", "VTI", "Vanguard", "iShares", "Invesco",
    ];
    let named = |text: &str, what: &str| {
        for word in NAMED {
            assert!(!text.contains(word), "{what} names {word}: {text}");
        }
    };
    for answers in many_answers() {
        let rec = recommend(&answers, &settings(), &DATA).unwrap();
        for lang in [Lang::En, Lang::He] {
            for reason in &rec.reasons {
                named(&reason.text[lang], "a reason");
            }
            for alternative in &rec.alternatives {
                named(&alternative.why_not.text[lang], "a why-not");
                named(&alternative.kind.text()[lang], "a kind's name");
            }
            for step in steps_for(&rec, lang) {
                named(&step.title, "a step's title");
                named(&step.text, "a step");
            }
            named(&rec.kind.text()[lang], "the chosen kind");
            for part in what(rec.kind, &rec.choice, rec.months, &answers.horizon, lang) {
                named(&part.text, "the what-sentence");
            }
            if let Some(band) = &rec.band {
                named(&history_sentence(band, lang), "the history sentence");
                // The basis is said here, and once.
                let caveat = history_caveat(band, lang);
                assert_eq!(caveat.matches("S&P").count(), 1, "{caveat}");
                assert!(caveat.contains(basis_sentence(lang)));
            }
        }
    }
    for kind in Way::for_index().into_iter().map(Kind::Way) {
        for lang in [Lang::En, Lang::He] {
            named(&kind.text()[lang], "a way's guide name");
        }
    }
    for preset in broker_fees::situations::situations() {
        for lang in [Lang::En, Lang::He] {
            named(&preset.name[lang], "a chip");
            named(&preset.explanation[lang], "a chip's tip");
            named(&preset.lesson[lang], "a chip's lesson");
        }
    }
}
