//! The redesigned answer (`guide::answer`, REDESIGN.md Part 2): the
//! principle-2 test (no decision whose options lead to the same outcome),
//! §2.5's three worked examples by structure, and the rules each part keeps.
//! The figures come from the core; the tests state the rules, never a winner.

use std::collections::HashSet;
use std::sync::LazyLock;

use broker_fees::guide::*;
use broker_fees::history;
use broker_fees::situations::situations;
use broker_fees::{ExchangeRates, Lang};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

static DATA: LazyLock<Data> = LazyLock::new(Data::listed);

fn settings() -> Settings {
    let mut settings = Settings::new(ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap());
    settings.today = Some(time::macros::date!(2026 - 10 - 01));
    settings
}

fn answers(now: Decimal, monthly: Decimal, months: u32) -> Answers {
    Answers {
        situation: None,
        now,
        monthly,
        horizon: Horizon::Months(months),
        flexibility: None,
        cushion: None,
        needs_any_day: false,
        bank: Some(Bank::Hapoalim),
    }
}

/// §2.5's three examples, with the visitor's bank.
fn apartment() -> Answers {
    answers(dec!(100_000), dec!(2000), 36)
}
fn bonus() -> Answers {
    answers(dec!(15_000), dec!(0), 60)
}
fn salary() -> Answers {
    answers(dec!(0), dec!(500), 360)
}

fn answered(answers: &Answers, choices: Choices) -> Answer {
    answer_with(answers, &choices, &settings(), &DATA, false).unwrap()
}

fn line(answer: &Answer, kind: LineKind) -> Option<&Line> {
    answer.lines.iter().find(|line| line.kind == kind)
}

fn decision(answer: &Answer, kind: LineKind) -> Option<&Decision> {
    match &line(answer, kind)?.state {
        LineState::Decision(decision) => Some(decision),
        _ => None,
    }
}

fn is_stated(answer: &Answer, kind: LineKind) -> bool {
    matches!(
        line(answer, kind).map(|line| &line.state),
        Some(LineState::Stated { .. })
    )
}

fn is_waiting(answer: &Answer, kind: LineKind) -> bool {
    matches!(
        line(answer, kind).map(|line| &line.state),
        Some(LineState::Waiting)
    )
}

/// The answers and choices with one option picked.
fn pick(answers: &Answers, choices: Choices, pick: Pick) -> (Answers, Choices) {
    let (mut answers, mut choices) = (answers.clone(), choices);
    match pick {
        Pick::Cushion(CushionPick::Has) => answers.cushion = Some(Cushion::Has),
        Pick::Cushion(CushionPick::HasNot) => {
            answers.cushion = Some(Cushion::HasNot {
                monthly_expenses: STAND_IN_MONTHLY_EXPENSES,
            });
        }
        Pick::Flexibility(flexibility) => answers.flexibility = Some(flexibility),
        Pick::SafeWay(way) => choices.safe_way = Some(way),
        Pick::Wrapper(wrapper) => choices.wrapper = Some(wrapper),
        Pick::Account(account) => choices.account = Some(account),
        Pick::Holdings(holdings) => choices.holdings = Some(holdings),
    }
    (answers, choices)
}

/// What the visitor sees, apart from which option is marked: the plan, the
/// lines' sentences and questions, the rows and which are active, the focus.
fn outcome_of(answer: &Answer) -> String {
    let lines: Vec<String> = answer
        .lines
        .iter()
        .map(|line| match &line.state {
            LineState::Stated { text, pill } => format!("{:?} {pill:?} {}", line.kind, text.en),
            LineState::Waiting => format!("{:?} waiting", line.kind),
            LineState::Decision(decision) => format!(
                "{:?} {:?} {} {:?}",
                line.kind,
                decision.id,
                decision.question.en,
                decision.difference.as_ref().map(|text| &text.en)
            ),
        })
        .collect();
    let rows: Vec<String> = answer
        .outcome
        .rows
        .iter()
        .map(|row| format!("{:?} {}", row.id, row.active))
        .collect();
    // "How to do it" follows the choices too.
    let steps: Vec<String> = answer
        .how
        .iter()
        .filter(|how| Some(how.row) == answer.focus)
        .flat_map(|how| how.steps.iter().map(|step| step.en.to_string()))
        .collect();
    format!(
        "{:?} | {lines:?} | {rows:?} | {:?} | {steps:?}",
        answer.plan.as_ref().map(|plan| &plan.en),
        answer.focus
    )
}

/// Every decision reachable from `answers` by picking options, at most
/// `depth` picks deep: each one's options must lead to different outcomes.
fn assert_every_decision_matters(start: &Answers, label: &str) {
    let mut seen: HashSet<String> = HashSet::new();
    // Each state is worked out once: as an option's outcome, then as a
    // state whose own decisions are tried.
    let mut worked: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut queue = vec![(start.clone(), Choices::default(), 0)];
    while let Some((answers, choices, depth)) = queue.pop() {
        let answer = answered(&answers, choices);
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            assert!(
                (2..=3).contains(&decision.options.len()),
                "{label}: {:?} has {} options",
                decision.id,
                decision.options.len()
            );
            let outcomes: Vec<String> = decision
                .options
                .iter()
                .map(|option| {
                    let (answers, choices) = pick(&answers, choices, option.pick);
                    let key = format!("{answers:?}{choices:?}");
                    let outcome = worked
                        .entry(key.clone())
                        .or_insert_with(|| outcome_of(&answered(&answers, choices)))
                        .clone();
                    if depth < 3 && seen.insert(key) {
                        queue.push((answers, choices, depth + 1));
                    }
                    outcome
                })
                .collect();
            let distinct: HashSet<&String> = outcomes.iter().collect();
            assert_eq!(
                distinct.len(),
                outcomes.len(),
                "{label}: the options of {:?} lead to the same outcome",
                decision.id
            );
        }
    }
}

// ─────────────────────────── Principle 2 ───────────────────────────

/// No question appears whose answers lead to the same outcome, in any
/// situation, the worked examples, or after any of their choices.
#[test]
fn every_decision_changes_the_outcome() {
    for preset in situations() {
        assert_every_decision_matters(&preset.answers, preset.id);
    }
    for (label, answers) in [
        ("apartment", apartment()),
        ("bonus", bonus()),
        ("salary", salary()),
    ] {
        assert_every_decision_matters(&answers, label);
    }
}

// ─────────────────────────── §2.5's examples ───────────────────────────

/// The apartment: the place type stated (no date question), the way a
/// decision between the fund and the bonds, no account question, where to
/// buy waiting on it; then stated for either choice.
#[test]
fn the_apartment_asks_only_the_way() {
    let answer = answered(&apartment(), Choices::default());
    assert!(line(&answer, LineKind::Before).is_none());
    assert!(is_stated(&answer, LineKind::PlaceType));
    let way = decision(&answer, LineKind::Way).expect("the way is the visitor's");
    assert_eq!(way.id, DecisionId::SafeWay);
    let picks: Vec<Pick> = way.options.iter().map(|option| option.pick).collect();
    assert!(picks.contains(&Pick::SafeWay(SafeWay::MoneyMarketFund)));
    assert!(picks.contains(&Pick::SafeWay(SafeWay::Bond)));
    // Monthly money: a deposit takes one sum only.
    assert!(!picks.contains(&Pick::SafeWay(SafeWay::Deposit)));
    // The bonds are one option, the other bond its variant.
    let bond = way
        .options
        .iter()
        .find(|option| option.pick == Pick::SafeWay(SafeWay::Bond))
        .unwrap();
    assert_eq!(bond.rows.len(), 2);
    assert!(is_waiting(&answer, LineKind::Where));
    assert_eq!(answer.open_decisions, 1);
    assert!(answer.plan.is_none());
    // Within the margin, or flipping with the rate: not a verdict.
    // (§2.5 gives B; the data agrees on 2026-10-19.)
    assert_ne!(answer.confidence.state, ConfidenceState::A);
    assert!(way.difference.is_some());
    // The stock row stays, dimmed, with its reason; its how says why and
    // gives no steps.
    let stock = answer
        .outcome
        .rows
        .iter()
        .find(|row| matches!(row.kind, Kind::Way(_)))
        .expect("the index is drawn beside");
    assert!(stock.excluded.is_some() && !stock.active);
    let how = answer.how.iter().find(|how| how.row == stock.id).unwrap();
    assert!(how.not_offered.is_some() && how.steps.is_empty());
    for choice in [SafeWay::MoneyMarketFund, SafeWay::Bond] {
        let chosen = answered(
            &apartment(),
            Choices {
                safe_way: Some(choice),
                ..Choices::default()
            },
        );
        assert!(
            is_stated(&chosen, LineKind::Where),
            "{choice:?}: no account question"
        );
        assert!(chosen.plan.is_some());
        assert_eq!(chosen.open_decisions, 0);
    }
}

/// The bonus: the place type is the visitor's call (state C); the way and
/// where wait on it; after "safe", which safe way (three options); after
/// "growth", the way is stated. No account question before growth.
#[test]
fn the_bonus_asks_safety_or_growth_first() {
    let answer = answered(&bonus(), Choices::default());
    assert_eq!(answer.confidence.state, ConfidenceState::C);
    let place = decision(&answer, LineKind::PlaceType).expect("safe or growth");
    assert_eq!(place.id, DecisionId::PlaceType);
    assert_eq!(place.options.len(), 2);
    assert!(is_waiting(&answer, LineKind::Way));
    assert!(is_waiting(&answer, LineKind::Where));
    assert!(
        line(&answer, LineKind::Before).is_none(),
        "one sum: no cushion"
    );

    let safe = answered(
        &Answers {
            flexibility: Some(Flexibility::Fixed),
            ..bonus()
        },
        Choices::default(),
    );
    let way = decision(&safe, LineKind::Way).expect("which safe way");
    assert_eq!(way.options.len(), 3);
    assert!(is_waiting(&safe, LineKind::Where));
    // The decision above keeps its options as they were.
    assert_eq!(
        decision(&safe, LineKind::PlaceType).unwrap().options,
        place.options
    );

    let growth = answered(
        &Answers {
            flexibility: Some(Flexibility::CanWait),
            ..bonus()
        },
        Choices::default(),
    );
    assert!(is_stated(&growth, LineKind::Way));
    assert_eq!(growth.confidence.state, ConfidenceState::C);
    assert_eq!(
        decision(&growth, LineKind::PlaceType).unwrap().options,
        place.options
    );
}

/// The salary: the cushion first, the index stated (state A), the way a
/// decision with the study fund, and the account only after the ETF.
#[test]
fn the_salary_asks_the_cushion_the_wrapper_then_the_account() {
    let answer = answered(&salary(), Choices::default());
    assert_eq!(answer.confidence.state, ConfidenceState::A);
    assert_eq!(
        decision(&answer, LineKind::Before).map(|d| d.id),
        Some(DecisionId::Cushion)
    );
    assert!(is_stated(&answer, LineKind::PlaceType));
    let way = decision(&answer, LineKind::Way).expect("ETF or study fund");
    assert_eq!(way.id, DecisionId::Wrapper);
    let study = way
        .options
        .iter()
        .find(|option| option.pick == Pick::Wrapper(Wrapper::StudyFund))
        .unwrap();
    assert!(study.fits_if.is_some(), "a study fund says who it fits");
    assert!(is_waiting(&answer, LineKind::Where));
    // The kinds come first in the lines' order.
    let kinds: Vec<LineKind> = answer.lines.iter().map(|line| line.kind).collect();
    assert_eq!(
        kinds,
        [
            LineKind::Before,
            LineKind::PlaceType,
            LineKind::Way,
            LineKind::Where
        ]
    );

    let etf = answered(
        &salary(),
        Choices {
            wrapper: Some(Wrapper::Anyone),
            ..Choices::default()
        },
    );
    assert_eq!(
        decision(&etf, LineKind::Where).map(|d| d.id),
        Some(DecisionId::Account)
    );
    assert!(etf.gain.is_some(), "fees decide the account");
    let study = answered(
        &salary(),
        Choices {
            wrapper: Some(Wrapper::StudyFund),
            ..Choices::default()
        },
    );
    assert!(is_stated(&study, LineKind::Where));
}

/// A bonus with no date: the place type is still the visitor's call, in the
/// same form.
#[test]
fn a_sum_with_no_date_asks_whether_it_can_wait() {
    let answer = answered(
        &Answers {
            horizon: Horizon::Unknown,
            ..bonus()
        },
        Choices::default(),
    );
    let place = decision(&answer, LineKind::PlaceType).expect("can it wait 5 years");
    assert_eq!(place.id, DecisionId::PlaceType);
    assert_eq!(answer.confidence.state, ConfidenceState::C);
}

/// A portfolio already at a bank: what it holds is asked on the איפה line.
#[test]
fn an_existing_portfolio_asks_what_it_holds() {
    let preset = situations()
        .into_iter()
        .find(|preset| preset.id == "existing")
        .unwrap();
    let answer = answered(&preset.answers, Choices::default());
    let holdings = decision(&answer, LineKind::Where).expect("what's in it");
    assert_eq!(holdings.id, DecisionId::Holdings);
    assert_eq!(holdings.options.len(), 3);
}

// ─────────────────────────── The rules of each part ───────────────────────────

/// A dependent decision appears only after its parent: until then its line
/// waits, and the plan waits for every line.
#[test]
fn a_dependent_decision_waits_for_its_parent() {
    for answers in [apartment(), bonus(), salary()] {
        let answer = answered(&answers, Choices::default());
        let mut open_above = false;
        for line in &answer.lines {
            match &line.state {
                LineState::Decision(decision) if decision.chosen.is_none() => open_above = true,
                LineState::Waiting => assert!(open_above, "{:?} waits on nothing", line.kind),
                _ => {}
            }
        }
        if answer
            .lines
            .iter()
            .any(|line| line.state == LineState::Waiting)
        {
            assert!(answer.plan.is_none());
        }
    }
}

/// Each option's figure is its row's: one number in one place, after tax and
/// in today's money.
#[test]
fn an_options_figure_is_its_rows() {
    for answers in [apartment(), bonus(), salary()] {
        let answer = answered(&answers, Choices::default());
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            for option in &decision.options {
                let (Some(value), Some(row)) = (option.value, option.rows.first()) else {
                    continue;
                };
                if matches!(
                    decision.id,
                    DecisionId::Account | DecisionId::Holdings | DecisionId::Cushion
                ) {
                    continue;
                }
                let row = answer.outcome.rows.iter().find(|r| r.id == *row).unwrap();
                assert_eq!(row.value, value, "{:?}", decision.id);
            }
        }
    }
}

/// The market's bar runs worst ≤ median ≤ best, and the deposits and the
/// current account are on every chart.
#[test]
fn the_chart_draws_the_markets_range_in_order() {
    for answers in [apartment(), bonus(), salary()] {
        let answer = answered(&answers, Choices::default());
        assert!(answer.outcome.deposited.today > Decimal::ZERO);
        assert!(answer.outcome.current_account.today <= answer.outcome.deposited.today);
        assert!(answer.outcome.rows.len() <= 5);
        let (low, high) = answer.outcome.axis;
        assert!(low < high);
        for row in &answer.outcome.rows {
            match &row.mark {
                Mark::Market {
                    worst,
                    median,
                    best,
                    ..
                } => {
                    assert!(worst.today <= median.today && median.today <= best.today);
                }
                Mark::RateRange { low, high } => {
                    assert!(low.today <= row.value.today && row.value.today <= high.today);
                }
                Mark::Known | Mark::Expected => {}
            }
        }
    }
}

/// Without a bank, nothing is assumed: the reference is the banks' range,
/// and where naming the bank would change what's shown, it is asked.
#[test]
fn without_a_bank_the_banks_are_a_range() {
    let answers = Answers {
        bank: None,
        ..salary()
    };
    let answer = answered(
        &answers,
        Choices {
            wrapper: Some(Wrapper::Anyone),
            ..Choices::default()
        },
    );
    let focus = answer.focus.expect("the ETF is chosen");
    let how = answer.how.iter().find(|how| how.row == focus).unwrap();
    let where_to_buy = how.where_to_buy.as_ref().unwrap();
    assert!(where_to_buy.rows.iter().all(|row| !row.yours));
    assert!(where_to_buy.bank_range.is_some());
}

/// Places within the providers' margin share a tie group, and the best's is
/// group 0.
#[test]
fn ties_are_ties() {
    let answer = answered(
        &apartment(),
        Choices {
            safe_way: Some(SafeWay::Bond),
            ..Choices::default()
        },
    );
    let focus = answer.focus.unwrap();
    let rows = &answer
        .how
        .iter()
        .find(|how| how.row == focus)
        .unwrap()
        .where_to_buy
        .as_ref()
        .unwrap()
        .rows;
    let margin = provider_margin(answer.outcome.deposited.today);
    let best = rows.first().unwrap();
    for row in rows {
        let within = best.left.today - row.left.today <= margin;
        assert_eq!(row.tie == 0, within, "{}", &row.name[Lang::En]);
    }
}

/// The horizons follow the visitor's answer about the date, or show both
/// where it's unanswered and the two differ.
#[test]
fn the_horizons_follow_the_date_answer() {
    let full =
        |answers: &Answers| answer(answers, &Choices::default(), &settings(), &DATA).unwrap();
    let open = full(&bonus());
    let five: Vec<&HorizonRow> = open.horizons.iter().filter(|row| row.years == 5).collect();
    assert_eq!(five.len(), 2, "5 years differs by the answer");
    let fixed = full(&Answers {
        flexibility: Some(Flexibility::Fixed),
        ..bonus()
    });
    assert_eq!(fixed.horizons.len(), LADDER_YEARS.len());
    assert!(
        fixed
            .horizons
            .iter()
            .all(|row| row.low.today <= row.high.today)
    );
}

/// The history run on one sum is the band's own: the money-weighted rate of
/// a single deposit is its yearly rate.
#[test]
fn one_sum_through_the_history_is_the_band() {
    for years in [1_u32, 5, 10, 30] {
        let band = history::band(Decimal::from(years)).unwrap();
        let mine = history::deposits_band(dec!(1000), dec!(0), years * 12).unwrap();
        assert!((band.worst.0 - mine.worst.0).abs() < dec!(0.05), "{years}");
        assert!((band.best.0 - mine.best.0).abs() < dec!(0.05), "{years}");
        assert_eq!(band.lower, mine.lower);
        assert_eq!(band.windows, mine.windows);
    }
}

/// The noise margins are the owner's: the larger of a sum and a share.
#[test]
fn the_margins_are_the_owners() {
    assert_eq!(kind_margin(dec!(50_000)), dec!(1000));
    assert_eq!(kind_margin(dec!(500_000)), dec!(5000));
    assert_eq!(provider_margin(dec!(50_000)), dec!(200));
    assert_eq!(provider_margin(dec!(400_000)), dec!(1000));
}
