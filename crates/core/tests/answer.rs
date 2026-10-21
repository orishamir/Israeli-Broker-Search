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
                monthly_expenses: None,
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
    // Only options the visitor could choose get a row (§4.6): no stock
    // row; the place type's line says why, with the past's count (§4.4).
    assert!(
        answer
            .outcome
            .rows
            .iter()
            .all(|row| !matches!(row.kind, Kind::Way(_)))
    );
    let Some(LineState::Stated { text, .. }) =
        line(&answer, LineKind::PlaceType).map(|line| &line.state)
    else {
        panic!("the place type is stated");
    };
    assert!(text.he.starts_with("בטוח."), "{}", text.he);
    assert!(
        text.he.contains("מתוך") && text.he.contains("מאז 1926"),
        "{}",
        text.he
    );
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
    // §2.5: where is stated after growth, a tie, with no account question:
    // the bank is within the margin between kinds (finding 8).
    assert!(is_stated(&growth, LineKind::Where), "no account question");
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

/// A portfolio already at a bank (finding 2): the visitor asks what it
/// costs, so the place type isn't asked; the first decision is what it
/// holds, then the move is stated (as it is, no sale), against the bank's
/// branch plan, or every bank's while none is named; and the steps move
/// it, they don't buy.
#[test]
fn an_existing_portfolio_asks_what_it_holds() {
    let preset = situations()
        .into_iter()
        .find(|preset| preset.id == "existing")
        .unwrap();
    for bank in [None, Some(Bank::Discount)] {
        let start = Answers {
            situation: Some(Situation::ExistingPortfolio { bank }),
            ..preset.answers.clone()
        };
        let answer = answered(&start, Choices::default());
        assert!(is_stated(&answer, LineKind::PlaceType), "no place question");
        assert!(decision(&answer, LineKind::PlaceType).is_none());
        let first = answer
            .lines
            .iter()
            .find_map(|line| match &line.state {
                LineState::Decision(decision) => Some(decision.id),
                _ => None,
            })
            .expect("a decision");
        assert_eq!(first, DecisionId::Holdings);
        assert_eq!(decision(&answer, LineKind::Way).unwrap().options.len(), 3);
        assert!(is_waiting(&answer, LineKind::Where));
        // No safe way: it would mean selling.
        assert!(
            answer
                .outcome
                .rows
                .iter()
                .all(|row| row.kind.arena() == Some(Arena::Index))
        );
        let moved = answered(
            &start,
            Choices {
                holdings: Some(Holdings::IndexFunds),
                ..Choices::default()
            },
        );
        let Some(LineState::Stated { text, .. }) =
            line(&moved, LineKind::Where).map(|line| &line.state)
        else {
            panic!("the move is stated");
        };
        assert!(text.he.contains("כמו שהם"), "{}", text.he);
        let how = moved
            .how
            .iter()
            .find(|how| Some(how.row) == moved.focus)
            .unwrap();
        let steps: String = how.steps.iter().map(|step| step.he.to_string()).collect();
        assert!(steps.contains("להעביר"), "{steps}");
        assert!(!steps.contains("קונים"), "{steps}");
        assert!(how.search.is_none(), "nothing to buy");
        let where_to_buy = how.where_to_buy.as_ref().unwrap();
        match bank {
            None => {
                assert!(where_to_buy.rows.iter().all(|row| !row.yours));
                assert!(where_to_buy.bank_range.is_some(), "the branch plans' range");
                assert!(
                    !he_texts(&moved)
                        .iter()
                        .any(|text| text.contains("הבנק שלכם"))
                );
            }
            Some(_) => assert!(where_to_buy.rows.iter().any(|row| row.yours)),
        }
        assert!(moved.plan.is_some());
    }
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

/// Every bar the chart draws says what it spans (principle 10): the fund's
/// rate bar has its note, in every situation and with the cushion asked
/// either way, and the index's bar has its losses note.
#[test]
fn every_bar_says_what_it_spans() {
    let mut starts: Vec<Answers> = situations().into_iter().map(|s| s.answers).collect();
    starts.extend([apartment(), bonus(), salary()]);
    for start in starts {
        for cushion in [
            None,
            Some(Cushion::Has),
            Some(Cushion::HasNot {
                monthly_expenses: Some(dec!(8000)),
            }),
        ] {
            let answers = Answers {
                cushion,
                ..start.clone()
            };
            let Ok(answer) = answer_with(&answers, &Choices::default(), &settings(), &DATA, false)
            else {
                continue;
            };
            for row in &answer.outcome.rows {
                match &row.mark {
                    Mark::RateRange { .. } => {
                        let note = row.note.as_ref().expect("a rate bar's note");
                        assert!(note[Lang::He].contains("ריבית בנק ישראל"), "{}", note.he);
                    }
                    Mark::Market { losses, .. } => assert!(!losses.he.is_empty()),
                    Mark::Known | Mark::Expected => {}
                }
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
    let five = open
        .horizons
        .iter()
        .find(|row| row.months == 60)
        .expect("5 years");
    assert_eq!(five.fits.len(), 2, "5 years differs by the answer");
    let fixed = full(&Answers {
        flexibility: Some(Flexibility::Fixed),
        ..bonus()
    });
    assert!(fixed.horizons.iter().all(|row| row.fits.len() == 1));
    assert!(
        fixed
            .horizons
            .iter()
            .flat_map(|row| &row.fits)
            .all(|fit| fit.low.today <= fit.high.today)
    );
}

/// The dates of "ואם התאריך יזוז?" are built around the visitor's (§4.10):
/// the list's closest date before it, 5 years (or 1) when earlier still,
/// the visitor's own, and the next date after it, or from 20 years on the
/// visitor's date plus 5 years. Dates the engine didn't compute before,
/// such as 35 years, are worked out too.
#[test]
fn the_horizon_dates_are_built_around_the_visitors() {
    let months_of = |months: u32| -> Vec<(u32, bool)> {
        answer(
            &answers(dec!(10_000), dec!(500), months),
            &Choices::default(),
            &settings(),
            &DATA,
        )
        .unwrap()
        .horizons
        .iter()
        .map(|row| (row.months, row.current))
        .collect()
    };
    assert_eq!(
        months_of(360),
        [(60, false), (240, false), (360, true), (420, false)]
    );
    assert_eq!(
        months_of(120),
        [(12, false), (60, false), (120, true), (240, false)]
    );
    assert_eq!(months_of(36), [(12, false), (36, true), (60, false)]);
    assert_eq!(
        months_of(44),
        [(12, false), (36, false), (44, true), (60, false)]
    );
    // 35 years is past the history: its figure is the expected return's.
    let far = answer(
        &answers(dec!(10_000), dec!(500), 360),
        &Choices::default(),
        &settings(),
        &DATA,
    )
    .unwrap();
    let last = far.horizons.last().unwrap();
    assert_eq!(last.months, 420);
    assert!(last.fits.iter().all(|fit| fit.low.today > Decimal::ZERO));
    assert!(last.deposited.today > far.outcome.deposited.today);
    // The visitor's own row is marked, and only it.
    for row in &far.horizons {
        assert_eq!(row.label.is_some(), row.current);
    }
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

/// Above the lines, where the rules put the money in the index, the data
/// decides how sure that is (finding 4): the place type is stated, in state
/// A, only when the index's worst past stretch, after fees and tax, beats
/// the safe ways' best version (the rate half a point up) by more than the
/// margin between kinds; otherwise it is the visitor's call (state C).
#[test]
fn above_the_lines_the_worst_past_decides_the_state() {
    let mut grid: Vec<Answers> = Vec::new();
    for months in [96, 120, 180, 240, 360] {
        for (now, monthly) in [
            (dec!(20_000), dec!(0)),
            (dec!(0), dec!(1000)),
            (dec!(50_000), dec!(2000)),
        ] {
            grid.push(Answers {
                bank: None,
                ..answers(now, monthly, months)
            });
        }
    }
    grid.push(Answers {
        horizon: Horizon::Retirement {
            age: 45,
            retire_at: 65,
        },
        ..answers(dec!(0), dec!(3000), 0)
    });
    grid.push(Answers {
        horizon: Horizon::Unknown,
        cushion: Some(Cushion::Has),
        ..answers(dec!(0), dec!(3000), 0)
    });
    // The workbench's 20 years at ₪2,000 a month, which the review found
    // stated by ₪2,000 under a ₪4,900 margin.
    grid.push(Answers {
        bank: None,
        ..answers(dec!(10_000), dec!(2000), 240)
    });
    let mut stated = 0;
    for answers in &grid {
        let answer = answered(answers, Choices::default());
        let worst = answer
            .outcome
            .rows
            .iter()
            .find_map(|row| match (&row.mark, row.kind) {
                (Mark::Market { worst, .. }, Kind::Way(_)) => Some(worst.today),
                _ => None,
            })
            .expect("the index's range is drawn");
        // The safe side's best version: the top of each row's mark.
        let best_safe = answer
            .outcome
            .rows
            .iter()
            .filter(|row| row.kind.arena() == Some(Arena::Safe))
            .map(|row| ends(row).1.today)
            .max()
            .expect("a safe way is drawn");
        let margin = kind_margin(answer.outcome.deposited.today);
        let asked = decision(&answer, LineKind::PlaceType).is_some();
        assert_eq!(
            asked,
            answer.confidence.state == ConfidenceState::C,
            "{answers:?}"
        );
        if asked {
            assert!(worst - best_safe <= margin, "{answers:?}");
        } else {
            stated += 1;
            assert!(worst - best_safe > margin, "{answers:?}");
            assert_eq!(answer.confidence.state, ConfidenceState::A, "{answers:?}");
            // The line says so as the past, on the figures' basis; the
            // figures are the chart's (each number once).
            let Some(LineState::Stated { text, .. }) =
                line(&answer, LineKind::PlaceType).map(|line| &line.state)
            else {
                panic!("stated: {answers:?}");
            };
            assert!(text.he.contains("אחרי עמלות ומס"), "{}", text.he);
            assert!(amounts_in(&text.he).is_empty(), "{}", text.he);
        }
    }
    assert!(stated > 0, "some of the grid is clear");
}

/// The noise margins are the owner's: the larger of a sum and a share.
#[test]
fn the_margins_are_the_owners() {
    assert_eq!(kind_margin(dec!(50_000)), dec!(1000));
    assert_eq!(kind_margin(dec!(500_000)), dec!(5000));
    assert_eq!(provider_margin(dec!(50_000)), dec!(200));
    assert_eq!(provider_margin(dec!(400_000)), dec!(1000));
}

/// The date `months` away as the page's fields set it, with the answer about
/// the date a fit is for.
fn moved(answers: &Answers, months: u32, flexibility: Option<Flexibility>) -> Answers {
    Answers {
        horizon: match answers.horizon {
            Horizon::Retirement { age, .. } => Horizon::Retirement {
                age,
                retire_at: age + months / 12,
            },
            Horizon::Months(_) | Horizon::Unknown => Horizon::Months(months),
        },
        flexibility: flexibility.or(answers.flexibility),
        ..answers.clone()
    }
}

/// The lower and upper ends a row of the outcome chart draws.
fn ends(row: &Row) -> (Amount, Amount) {
    match &row.mark {
        Mark::Known | Mark::Expected => (row.value, row.value),
        Mark::RateRange { low, high } => (*low, *high),
        Mark::Market { worst, median, .. } => (*worst, *median),
    }
}

/// A horizon row names each way once: what's in a portfolio keeps the same
/// fund whatever the answer, and it read "קרן מחקה…, קרן מחקה… או קרן
/// מחקה…" (the review of 2026-10-21).
#[test]
fn a_horizon_fit_names_each_way_once() {
    for situation in situations() {
        let full = answer(&situation.answers, &Choices::default(), &settings(), &DATA).unwrap();
        for row in &full.horizons {
            for fit in &row.fits {
                let name = &fit.name.he;
                let parts: Vec<&str> = name
                    .split(", ")
                    .flat_map(|part| part.split(" או "))
                    .collect();
                let mut unique = parts.clone();
                unique.sort_unstable();
                unique.dedup();
                assert_eq!(unique.len(), parts.len(), "{}: {name}", situation.id);
            }
        }
    }
}

/// §2.9's check: a horizon row is exactly what the answer shows once the
/// date moves there (and, for a fit of one answer about the date, once
/// that answer is given): the way it settles on, or one it leaves to the
/// visitor, and figures its chart draws.
#[test]
fn a_horizon_row_opens_exactly_what_it_showed() {
    for situation in situations() {
        let answers = Answers {
            bank: Some(Bank::Hapoalim),
            ..situation.answers.clone()
        };
        let full = answer(&answers, &Choices::default(), &settings(), &DATA).unwrap();
        // Money that may be needed any day has no date to move (the
        // review's N10): no rows, and the page holds no place for them.
        assert_eq!(full.moves, !answers.needs_any_day, "{}", situation.id);
        assert_eq!(
            full.horizons.is_empty(),
            answers.needs_any_day,
            "{}",
            situation.id
        );
        for row in &full.horizons {
            assert_eq!(
                row.current,
                moved(&answers, row.months, None).horizon == answers.horizon,
                "{} at {} months",
                situation.id,
                row.months
            );
            for fit in &row.fits {
                let label = format!(
                    "{} at {} months, {:?}",
                    situation.id, row.months, fit.flexibility
                );
                assert!(fit.low.today <= fit.high.today, "{label}");
                assert_eq!(
                    fit.condition.is_some(),
                    fit.flexibility.is_some(),
                    "{label}"
                );
                let opened = answered(
                    &moved(&answers, row.months, fit.flexibility),
                    Choices::default(),
                );
                match opened.focus {
                    Some(focus) => assert_eq!(fit.kind, focus, "{label}"),
                    None => assert!(
                        opened
                            .outcome
                            .rows
                            .iter()
                            .any(|shown| shown.kind == fit.kind && shown.active),
                        "{label}: {:?}",
                        fit.kind
                    ),
                }
                let rows = &opened.outcome.rows;
                let open_way = decision(&opened, LineKind::Way).filter(|way| way.chosen.is_none());
                let spanned: Vec<&Row> = match (opened.focus, open_way) {
                    (Some(focus), _) => rows.iter().filter(|shown| shown.id == focus).collect(),
                    (None, Some(way)) => way
                        .options
                        .iter()
                        .filter(|option| option.pick != Pick::Wrapper(Wrapper::StudyFund))
                        .flat_map(|option| &option.rows)
                        .filter_map(|id| rows.iter().find(|shown| shown.id == *id))
                        .collect(),
                    (None, None) => rows
                        .iter()
                        .filter(|shown| shown.active && shown.excluded.is_none())
                        .filter(|shown| shown.kind.arena() == fit.kind.arena())
                        .collect(),
                };
                let low = spanned.iter().map(|shown| ends(shown).0.today).min();
                let high = spanned.iter().map(|shown| ends(shown).1.today).max();
                assert_eq!(low, Some(fit.low.today), "{label}: the lowest it spans");
                assert_eq!(high, Some(fit.high.today), "{label}: the highest it spans");
                assert_eq!(row.deposited, opened.outcome.deposited, "{label}");
            }
        }
    }
}

/// Where the visitor hasn't said whether the date can move, a date where it
/// decides the place has a fit for each answer; once said, one a date.
#[test]
fn a_date_where_flexibility_decides_has_both_rows_until_answered() {
    let full =
        |answers: &Answers| answer(answers, &Choices::default(), &settings(), &DATA).unwrap();
    let open = full(&bonus());
    for row in &open.horizons {
        let at = answered(
            &Answers {
                horizon: Horizon::Months(row.months),
                ..bonus()
            },
            Choices::default(),
        );
        let decides = decision(&at, LineKind::PlaceType).is_some();
        assert_eq!(
            row.fits.len(),
            if decides { 2 } else { 1 },
            "{} months",
            row.months
        );
    }
    let can_wait = full(&Answers {
        flexibility: Some(Flexibility::CanWait),
        ..bonus()
    });
    assert!(
        can_wait
            .horizons
            .iter()
            .flat_map(|row| &row.fits)
            .all(|fit| fit.flexibility.is_none())
    );
}
/// The takeaway names the part that makes most of the difference, as the
/// bars' own figures say: here fees for the account, tax for the study fund.
#[test]
fn the_gain_takeaway_names_the_bigger_part() {
    let etf = Choices {
        wrapper: Some(Wrapper::Anyone),
        ..Choices::default()
    };
    for choices in [Choices::default(), etf] {
        let gain = answer(&salary(), &choices, &settings(), &DATA)
            .unwrap()
            .gain
            .expect("fees or tax decide something");
        // The takeaway is about the last decision's two bars.
        let [.., first, second] = gain.rows.as_slice() else {
            panic!("two bars at least");
        };
        let (better, worse) = if first.kept.today >= second.kept.today {
            (first, second)
        } else {
            (second, first)
        };
        let fees = |row: &GainRow| row.fees.today + row.product.today;
        let by_fees = fees(worse) - fees(better) >= worse.tax.today - better.tax.today;
        let says = &gain.takeaway[Lang::En];
        assert_eq!(says.contains("from fees"), by_fees, "{says}");
        assert_eq!(says.contains("from tax"), !by_fees, "{says}");
    }
}

// ─────────────────────────── The cleanup's rules ───────────────────────────

/// Every start the rules below are checked on: the situations and the
/// worked examples, each with the date's answer given both ways, with and
/// without the visitor's bank.
fn starts() -> Vec<Answers> {
    let mut all: Vec<Answers> = situations().into_iter().map(|s| s.answers).collect();
    all.extend([apartment(), bonus(), salary()]);
    let mut out = Vec::new();
    for start in all {
        for flexibility in [None, Some(Flexibility::Fixed), Some(Flexibility::CanWait)] {
            for bank in [start.bank, None] {
                out.push(Answers {
                    flexibility,
                    bank,
                    ..start.clone()
                });
            }
        }
    }
    out
}

/// The choices that open every line, and none.
fn every_choice() -> [Choices; 4] {
    [
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
            holdings: Some(Holdings::IndexFunds),
        },
        Choices {
            safe_way: Some(SafeWay::MoneyMarketFund),
            ..Choices::default()
        },
    ]
}

/// The ₪ amounts written in a text: "₪12,345" is 12345.
fn amounts_in(text: &str) -> Vec<Decimal> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices('₪') {
        let digits: String = text[at + '₪'.len_utf8()..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ',')
            .filter(char::is_ascii_digit)
            .collect();
        if let Ok(amount) = digits.parse::<Decimal>() {
            out.push(amount);
        }
    }
    out
}

/// A typed month's expenses are the visitor's: only an empty field is the
/// stand-in, and only then does the option say so.
#[test]
fn only_an_empty_expenses_field_is_a_stand_in() {
    let fact = |monthly_expenses| {
        let answers = Answers {
            cushion: Some(Cushion::HasNot { monthly_expenses }),
            ..salary()
        };
        let answer = answered(&answers, Choices::default());
        let cushion = decision(&answer, LineKind::Before).expect("the cushion is asked");
        let not_yet = cushion
            .options
            .iter()
            .find(|option| option.pick == Pick::Cushion(CushionPick::HasNot))
            .unwrap();
        not_yet
            .facts
            .iter()
            .map(|fact| fact.he.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let typed = fact(Some(STAND_IN_MONTHLY_EXPENSES));
    assert!(!typed.contains("הערכה"), "{typed}");
    assert!(typed.contains("ההוצאות שלכם"), "{typed}");
    let empty = fact(None);
    assert!(empty.contains("הערכה"), "{empty}");
}

/// The safe way's difference names what a bank would sell: makam where the
/// bond's place is makam (money needed any day), never "אג״ח" for it.
#[test]
fn the_difference_names_makam_as_makam() {
    let mut seen = 0;
    for start in starts().into_iter().filter(|start| start.needs_any_day) {
        for bank in [None, Some(Bank::Hapoalim)] {
            let answer = answered(
                &Answers {
                    bank,
                    ..start.clone()
                },
                Choices::default(),
            );
            let Some(way) =
                decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::SafeWay)
            else {
                continue;
            };
            let said = way.difference.as_ref().map_or("", |text| &text.he);
            assert!(!said.contains("אג״ח"), "{said}");
            seen += usize::from(said.contains("מק״מ"));
        }
    }
    assert!(seen > 0, "makam is in a difference");
}

/// The plan says "a new account" only where the visitor chose one: a place
/// the site states may be where they already have an account (principle
/// 5; the review of 2026-10-21, the apartment's bond).
#[test]
fn the_plan_says_new_account_only_when_chosen() {
    let mut stated = 0;
    for start in starts() {
        for choices in every_choice() {
            let account = choices.account;
            let answer = answered(&start, choices);
            let Some(plan) = &answer.plan else { continue };
            let asked =
                decision(&answer, LineKind::Where).is_some_and(|d| d.id == DecisionId::Account);
            if plan.he.contains("בחשבון חדש") {
                assert!(asked && account == Some(Account::NewAccount), "{}", plan.he);
            }
            stated += usize::from(!asked && plan.he.contains("בחשבון ניירות ערך"));
        }
    }
    assert!(stated > 0, "a stated place is a securities account");
}

/// The visitor's bank, in the tie the line states, is named as theirs:
/// the any-day deposit's tie listed Hapoalim as one more bank (principle 15).
#[test]
fn the_visitors_bank_in_a_tie_is_named_as_theirs() {
    let cushion = situations()
        .into_iter()
        .find(|situation| situation.id == "cushion")
        .unwrap();
    let answers = Answers {
        bank: Some(Bank::Hapoalim),
        ..cushion.answers
    };
    let answer = answered(
        &answers,
        Choices {
            safe_way: Some(SafeWay::Deposit),
            ..Choices::default()
        },
    );
    let Some(Line {
        state: LineState::Stated { text, .. },
        ..
    }) = line(&answer, LineKind::Where)
    else {
        panic!("the place is stated");
    };
    assert!(text.he.contains("הבנק שלכם (הפועלים)"), "{}", text.he);
    assert!(!text.he.contains(", הפועלים"), "{}", text.he);
}

/// "דרך בנק" is one row, a reference, only beside a place it is a
/// reference for: where only banks hold it, the banks are listed.
#[test]
fn the_bank_reference_stands_beside_another_place() {
    let mut listed = 0;
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            for how in &answer.how {
                let Some(places) = &how.where_to_buy else {
                    continue;
                };
                assert!(
                    places.reference.is_none() || !places.rows.is_empty(),
                    "{:?}: a reference with nothing beside it",
                    how.row
                );
                listed += usize::from(places.reference.is_none() && !places.rows.is_empty());
            }
        }
    }
    assert!(listed > 0);
}

/// Where another safe way's figure lies inside the money market fund's bar
/// (the rate ±0.5 point), the order flips within the versions: the decision
/// then says the choice hangs on the rate, not on locking or an account.
#[test]
fn a_switch_inside_the_rate_bar_means_the_order_flips() {
    let mut flipped = 0;
    for start in starts() {
        let answer = answered(&start, Choices::default());
        let Some(way) = decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::SafeWay)
        else {
            continue;
        };
        let Some((low, high)) = answer.outcome.rows.iter().find_map(|row| match row.mark {
            Mark::RateRange { low, high } => Some((low.today, high.today)),
            _ => None,
        }) else {
            continue;
        };
        let inside = way.options.iter().any(|option| {
            option.pick != Pick::SafeWay(SafeWay::MoneyMarketFund)
                && option
                    .value
                    .is_some_and(|value| low <= value.today && value.today <= high)
        });
        if inside {
            flipped += 1;
            // Where the fund and the way beside it trade places is said, as
            // a rate (principle 6), when the difference measures that pair.
            let fund = Pick::SafeWay(SafeWay::MoneyMarketFund);
            let pair = &way.options[..2.min(way.options.len())];
            let other_inside = pair.iter().any(|option| option.pick == fund)
                && pair.iter().any(|option| {
                    option.pick != fund
                        && option
                            .value
                            .is_some_and(|value| low <= value.today && value.today <= high)
                });
            let said = way.difference.as_ref().map_or("", |text| &text.he);
            assert!(
                !other_inside || said.contains("בממוצע מעל כ") || said.contains("בממוצע מתחת לכ"),
                "{:?}: {said}",
                start.horizon
            );
            assert!(
                way.why.he.contains("בריבית שאף אחד לא יודע"),
                "{:?}: {}",
                start.horizon,
                way.why.he
            );
        }
    }
    assert!(flipped > 0, "the apartment's case, at least, flips");
}

/// The body says the direction and what's left; its figures are the lines',
/// the options' and the chart's, so none of theirs is repeated in it
/// (principle 13).
#[test]
fn the_body_repeats_no_figure_shown_elsewhere() {
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            let body = amounts_in(&answer.body.he);
            if body.is_empty() {
                continue;
            }
            let mut shown: Vec<Decimal> = Vec::new();
            let json = serde_json::to_value(&answer.lines).unwrap();
            let mut texts = Vec::new();
            hebrew_in(&json, &mut texts);
            for text in texts {
                shown.extend(amounts_in(&text));
            }
            for row in &answer.outcome.rows {
                shown.push(row.value.today);
            }
            shown.extend([
                answer.outcome.deposited.today,
                answer.outcome.current_account.today,
            ]);
            for amount in body {
                assert!(
                    shown
                        .iter()
                        .all(|other| (*other - amount).abs() > amount * dec!(0.01)),
                    "the body repeats {amount}: {}",
                    answer.body.he
                );
            }
        }
    }
}

/// Every Hebrew text of an answer.
fn he_texts(answer: &Answer) -> Vec<String> {
    let mut texts = Vec::new();
    hebrew_in(&serde_json::to_value(answer).unwrap(), &mut texts);
    texts
}

fn hebrew_in(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(fields) => {
            if let Some(serde_json::Value::String(he)) = fields.get("he") {
                out.push(he.clone());
            }
            fields.values().for_each(|field| hebrew_in(field, out));
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| hebrew_in(item, out)),
        _ => {}
    }
}

/// For retirement the answer gives the monthly pension as a range, once:
/// the growth option's second amount while the place type is the
/// visitor's, the card's otherwise. Whose coefficients it rests on, and on
/// what terms, is in the provident fund's steps, not in the option (the
/// review's N7). The pension from work is said in the body, not asked.
#[test]
fn retirement_gives_the_pension_and_counts_the_one_from_work() {
    let preset = situations()
        .into_iter()
        .find(|preset| preset.id == "retirement")
        .unwrap();
    let growth = Answers {
        flexibility: Some(Flexibility::CanWait),
        ..preset.answers.clone()
    };
    // While the place type is open, the pension is the growth option's,
    // not the answer's (an option's result isn't shown before it's chosen).
    let open = answered(&preset.answers, Choices::default());
    if let Some(place) = decision(&open, LineKind::PlaceType) {
        assert!(open.annuity.is_none(), "not before growth is chosen");
        let growth_option = place
            .options
            .iter()
            .find(|option| option.pick == Pick::Flexibility(Flexibility::CanWait))
            .unwrap();
        let also = growth_option
            .also
            .as_ref()
            .expect("the pension is in the growth option");
        assert_eq!(amounts_in(&also.he).len(), 2, "{}", also.he);
        assert!(!also.he.contains("[₪—]"), "{}", also.he);
        assert!(
            growth_option
                .facts
                .iter()
                .all(|fact| !fact.he.contains("מקדמי ההמרה") && !fact.he.contains("קצבה חודשית")),
            "the terms aren't in the option"
        );
    }
    // Retirement draws the other accounts beside the provident fund, and
    // shows where the gain goes, since the way out decides the tax.
    let grown = answered(&growth, Choices::default());
    let vehicles: Vec<broker_fees::Vehicle> = grown
        .outcome
        .rows
        .iter()
        .filter_map(|row| match row.kind {
            Kind::Way(way) => Some(way.vehicle),
            _ => None,
        })
        .collect();
    for vehicle in [
        broker_fees::Vehicle::InvestmentGemel,
        broker_fees::Vehicle::Brokerage,
        broker_fees::Vehicle::StudyFund,
    ] {
        assert!(vehicles.contains(&vehicle), "{vehicle:?} is drawn");
    }
    assert!(grown.gain.is_some(), "the gain split");
    let steps: String = grown
        .how
        .iter()
        .filter(|how| Some(how.row) == grown.focus)
        .flat_map(|how| how.steps.iter().map(|step| step.he.to_string()))
        .collect();
    assert!(
        !steps.contains("מול המעסיק") && steps.contains("קופת גמל להשקעה"),
        "{steps}"
    );
    assert!(
        steps.contains("מקדמי ההמרה"),
        "the terms are in the steps: {steps}"
    );
    // Once: the option still shows it after the choice, so the card doesn't.
    if let Some(place) = decision(&grown, LineKind::PlaceType) {
        assert!(grown.annuity.is_none(), "said in the option");
        assert!(place.options.iter().any(|option| option.also.is_some()));
    } else {
        let annuity = grown.annuity.as_ref().expect("a monthly pension");
        assert!(!annuity.he.contains("[₪—]"), "{}", annuity.he);
        assert_eq!(amounts_in(&annuity.he).len(), 2, "{}", annuity.he);
        assert!(!annuity.he.contains("מקדמי ההמרה"), "{}", annuity.he);
    }
    for answer in [&open, &grown] {
        assert!(answer.body.he.contains("קרן הפנסיה"), "{}", answer.body.he);
        assert!(
            answer
                .lines
                .iter()
                .all(|line| !matches!(&line.state, LineState::Decision(d) if d.question.he.contains("פנסיה"))),
            "the pension from work is no question"
        );
    }
    let safe = answered(
        &Answers {
            flexibility: Some(Flexibility::Fixed),
            ..preset.answers.clone()
        },
        Choices::default(),
    );
    assert!(safe.annuity.is_none(), "a safe way pays no pension");
    let not_retiring = answered(&salary(), Choices::default());
    assert!(not_retiring.annuity.is_none());
}

/// What to type in the app: a bond's series by its number, from the data;
/// the funds' examples marked as examples; the money market funds as a tie,
/// never "the cheapest"; and a placeholder only where the data has nothing,
/// saying why.
#[test]
fn the_search_slots_come_from_the_data() {
    let numbers: Vec<String> = broker_fees::examples::government_bonds()
        .iter()
        .map(|bond| bond.number.to_string())
        .collect();
    let mut bonds = 0;
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            for how in &answer.how {
                let Some(search) = &how.search else { continue };
                let he = &search.text.he;
                if search.placeholder {
                    assert!(he.contains("____") && he.contains('('), "{he}");
                    continue;
                }
                assert!(!he.contains("____") && !he.contains("[—]"), "{he}");
                match how.row {
                    Kind::GovernmentBond { .. } => {
                        bonds += 1;
                        assert!(
                            numbers.iter().any(|number| he.starts_with(number.as_str())),
                            "{he}"
                        );
                        assert_ne!(search.sources, Vec::new());
                    }
                    Kind::MoneyMarketFund => {
                        assert!(he.starts_with("כספית שקלית"), "{he}");
                        assert!(!he.contains("הזול"), "{he}");
                    }
                    Kind::Way(way) if way.vehicle == broker_fees::Vehicle::Brokerage => {
                        assert!(
                            he.contains(broker_fees::examples::NOT_A_PICK.he.as_ref()),
                            "{he}"
                        );
                        assert_ne!(search.sources, Vec::new());
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(bonds > 0);
}

/// Where to buy names each provider one way: a broker's plan as "short name ·
/// plan", a bank's deposit by the bank's short name, and never the banks'
/// average, which is no place to buy.
#[test]
fn where_to_buy_names_each_provider_one_way() {
    let brokers: Vec<String> = DATA
        .brokers
        .iter()
        .map(|broker| broker.short_name.he.to_string())
        .collect();
    let deposits: Vec<String> = DATA
        .safe
        .iter()
        .flat_map(|kind| &kind.places)
        .map(|place| place.short_name.he.to_string())
        .collect();
    let average = broker_fees::short_term::deposits::AVERAGE.he.to_string();
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            for how in &answer.how {
                let Some(places) = &how.where_to_buy else {
                    continue;
                };
                for row in &places.rows {
                    let name = row.name.he.as_ref();
                    assert_ne!(name, average);
                    let provider = name.split("\u{a0}· ").next().unwrap();
                    match how.row {
                        Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit => {
                            assert!(deposits.iter().any(|bank| bank == name), "{name}");
                        }
                        _ => assert!(
                            name.contains("\u{a0}· ") && brokers.iter().any(|b| b == provider),
                            "{name}"
                        ),
                    }
                }
            }
        }
    }
}

/// The gain keeps the option not chosen (principle 4): before the way is
/// chosen, the wrapper's two options; after the ETF, the study fund beside
/// the ETF at the place that leaves most and at the visitor's bank (or, with
/// no bank named, the bank that leaves most, said as such, as every other
/// comparison measures from it: the review's N4). Never more than three.
#[test]
fn the_gain_keeps_the_option_not_chosen() {
    let study = Kind::Way(
        broker_fees::ways::Way::for_index()
            .into_iter()
            .find(|way| way.vehicle == broker_fees::Vehicle::StudyFund)
            .unwrap(),
    );
    let study_name = |answer: &Answer| {
        answer
            .outcome
            .rows
            .iter()
            .find(|row| row.kind == study)
            .map(|row| row.name.he.to_string())
    };
    let before = answered(&salary(), Choices::default());
    let gain = before.gain.as_ref().expect("tax decides the wrapper");
    assert_eq!(gain.rows.len(), 2);
    let etf = Choices {
        wrapper: Some(Wrapper::Anyone),
        ..Choices::default()
    };
    for bank in [Some(Bank::Hapoalim), None] {
        let answers = Answers { bank, ..salary() };
        let after = answered(&answers, etf);
        let gain = after.gain.as_ref().expect("fees decide the account");
        assert_eq!(gain.rows.len(), 3);
        let name = study_name(&after).expect("the study fund is drawn");
        assert_eq!(gain.rows[0].name.he, name, "the study fund stays");
        let bank_row = &gain.rows[2].name.he;
        match bank {
            Some(_) => assert!(bank_row.contains("הבנק שלכם"), "{bank_row}"),
            None => assert!(bank_row.contains("הבנק הזול"), "{bank_row}"),
        }
    }
    for start in starts() {
        for choices in every_choice() {
            if let Some(gain) = answered(&start, choices).gain {
                assert!((2..=3).contains(&gain.rows.len()));
            }
        }
    }
}

/// What every option would leave is drawn beside what the visitor
/// deposited and what the current account would leave (principle 12), and
/// the other arena is drawn beside the one the rules chose: a safe answer
/// shows what the index might do, an index answer what safety leaves.
#[test]
fn the_outcome_holds_the_current_account_and_both_arenas() {
    for start in starts() {
        // Money already invested has no safe option: selling isn't one.
        let existing = matches!(start.situation, Some(Situation::ExistingPortfolio { .. }));
        for choices in every_choice() {
            let answer = answered(&start, choices);
            let outcome = &answer.outcome;
            assert!(
                outcome.current_account.today > Decimal::ZERO,
                "{start:?}: the current account is the yardstick"
            );
            if existing {
                continue;
            }
            assert_eq!(
                outcome.current_account.nominal, outcome.deposited.nominal,
                "{start:?}: a current account leaves the deposits as they are"
            );
            // Both lines are drawn, with their amounts (§4.6).
            assert_eq!(outcome.lines.len(), 2);
            assert_eq!(outcome.lines[0].value, outcome.deposited);
            assert_eq!(outcome.lines[1].value, outcome.current_account);
            // Only options the visitor could choose get a row (§4.6): the
            // safe side always, the stocks unless the rules leave them out.
            assert!(outcome.rows.iter().all(|row| row.excluded.is_none()));
            assert!(
                outcome
                    .rows
                    .iter()
                    .any(|row| row.kind.arena() == Some(Arena::Safe)),
                "{start:?} {choices:?}: the safe side is drawn"
            );
            let stocks_left_out = start.needs_any_day
                || matches!(
                    line(&answer, LineKind::PlaceType).map(|line| &line.state),
                    Some(LineState::Stated { text, .. }) if text.en.starts_with("Safe")
                );
            let stocks_drawn = outcome
                .rows
                .iter()
                .any(|row| row.kind.arena() == Some(Arena::Index));
            if !stocks_left_out {
                assert!(stocks_drawn, "{start:?} {choices:?}: stocks are drawn");
            }
            if start.needs_any_day {
                assert!(
                    !stocks_drawn,
                    "{start:?}: no stock row for money needed any day"
                );
            }
        }
    }
}

/// The horizons, with the date that can wait: short money is safe, long
/// money is the index's, the place type changes once, and within one arena
/// more years never leave less.
#[test]
fn the_horizons_grow_with_the_years_and_change_arena_once() {
    for start in [
        answers(dec!(0), dec!(3000), 60),
        answers(dec!(15_000), dec!(0), 60),
    ] {
        let start = Answers {
            flexibility: Some(Flexibility::CanWait),
            ..start
        };
        let horizons = answer(&start, &Choices::default(), &settings(), &DATA)
            .unwrap()
            .horizons;
        assert!(horizons.iter().all(|row| row.fits.len() == 1), "{start:?}");
        let arenas: Vec<Option<Arena>> = horizons
            .iter()
            .map(|row| row.fits[0].kind.arena())
            .collect();
        assert_eq!(arenas.first(), Some(&Some(Arena::Safe)), "{start:?}");
        assert_eq!(arenas.last(), Some(&Some(Arena::Index)), "{start:?}");
        assert!(arenas.windows(2).filter(|pair| pair[0] != pair[1]).count() <= 1);
        for pair in horizons.windows(2) {
            let (before, after) = (&pair[0].fits[0], &pair[1].fits[0]);
            if before.kind.arena() == after.kind.arena() {
                assert!(
                    after.low.today >= before.low.today && after.high.today >= before.high.today,
                    "{} months leaves less than {}",
                    pair[1].months,
                    pair[0].months
                );
            }
        }
    }
}

// ─────────────────────────── The review's findings ───────────────────────────

/// The starts the review's rules are checked on, and their answers with
/// every choice: worked out once for all the tests that read them, since
/// the grid takes seconds (and `cargo mutants` runs it for every mutant).
fn every_answer() -> Vec<(Answers, Choices, Answer)> {
    static ALL: LazyLock<Vec<(Answers, Choices, Answer)>> = LazyLock::new(work_out_every_answer);
    ALL.clone()
}

fn work_out_every_answer() -> Vec<(Answers, Choices, Answer)> {
    let mut out = Vec::new();
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            out.push((start.clone(), choices, answer));
        }
    }
    out
}

/// What the index's rows leave, in today's money.
fn index_values(answer: &Answer) -> Vec<Decimal> {
    answer
        .outcome
        .rows
        .iter()
        .filter(|row| matches!(row.kind, Kind::Way(_)))
        .map(|row| row.value.today)
        .collect()
}

/// Finding 1: "not yet" on the cushion is one path, the monthly money in a
/// money market fund until the cushion is aside, then in the index, so it
/// changes the index's figures whenever the cushion takes a month or more;
/// and its option says what it leaves, as the chart does once it's chosen.
#[test]
fn the_cushions_answer_moves_the_figures() {
    let mut grid: Vec<Answers> = situations().into_iter().map(|s| s.answers).collect();
    grid.push(salary());
    for months in [120, 240, 360] {
        grid.push(answers(dec!(0), dec!(1000), months));
        grid.push(answers(dec!(10_000), dec!(3000), months));
    }
    let mut asked = 0;
    for start in grid {
        for flexibility in [None, Some(Flexibility::CanWait)] {
            let start = Answers {
                flexibility,
                ..start.clone()
            };
            let with = |cushion| {
                answered(
                    &Answers {
                        cushion: Some(cushion),
                        ..start.clone()
                    },
                    Choices::default(),
                )
            };
            let has = with(Cushion::Has);
            let Some(cushion) = decision(&has, LineKind::Before) else {
                continue;
            };
            asked += 1;
            let has_not = with(Cushion::HasNot {
                monthly_expenses: None,
            });
            assert_ne!(
                index_values(&has),
                index_values(&has_not),
                "{start:?}: not yet moves the index's figures"
            );
            let value = |answer: &Answer, pick| {
                decision(answer, LineKind::Before)
                    .unwrap()
                    .options
                    .iter()
                    .find(|option| option.pick == Pick::Cushion(pick))
                    .and_then(|option| option.value)
                    .map(|value| value.today)
            };
            assert_ne!(
                value(&has, CushionPick::Has),
                value(&has, CushionPick::HasNot),
                "{start:?}: each option says what it leaves"
            );
            assert!(
                cushion
                    .difference
                    .as_ref()
                    .is_some_and(|text| !amounts_in(&text.he).is_empty())
            );
            // Chosen, the chart's lead row is the option's figure.
            let lead = |answer: &Answer| {
                answer
                    .outcome
                    .rows
                    .iter()
                    .find(|row| matches!(row.kind, Kind::Way(_)) && row.active)
                    .map(|row| row.value.today)
            };
            assert_eq!(
                lead(&has_not),
                value(&has_not, CushionPick::HasNot),
                "{start:?}"
            );
            assert_eq!(lead(&has), value(&has, CushionPick::Has), "{start:?}");
        }
    }
    assert!(asked > 0);
}

/// Finding 3: every statement about the past beside a figure is on that
/// figure's basis: the bar's note counts the stretches that would have left
/// less than was deposited after fees and tax, so it says "none" exactly
/// when the bar's worst is at or above the deposits; and no dollar figure is
/// shown, so no sentence is "in dollars, before fees and tax".
#[test]
fn the_past_is_said_on_the_figures_basis() {
    for (start, _, answer) in every_answer() {
        for text in he_texts(&answer) {
            assert!(
                !text.contains("בדולרים, לפני עמלות ומס"),
                "{start:?}: {text}"
            );
        }
        for row in &answer.outcome.rows {
            if let Mark::Market { worst, losses, .. } = &row.mark {
                let none = losses.he.starts_with("באף אחת");
                assert_eq!(
                    none,
                    worst.today >= answer.outcome.deposited.today,
                    "{start:?}: {}",
                    losses.he
                );
            }
        }
        // The question's growth option never says it never lost, beside a
        // bar whose worst is below the deposits.
        if let Some(place) = decision(&answer, LineKind::PlaceType) {
            let worst = answer.outcome.rows.iter().find_map(|row| match &row.mark {
                Mark::Market { worst, .. } if row.active || row.excluded.is_none() => {
                    Some(worst.today)
                }
                _ => None,
            });
            let growth = place
                .options
                .iter()
                .find(|option| option.pick == Pick::Flexibility(Flexibility::CanWait))
                .unwrap();
            let says_never = growth
                .facts
                .iter()
                .any(|fact| fact.he.contains("באף אחת מהן לא פחות ממה שהופקד"));
            if says_never && let Some(worst) = worst {
                assert!(worst >= answer.outcome.deposited.today, "{start:?}");
            }
        }
    }
}

/// Finding 5: a figure that rests on an assumption (the index at a yearly
/// return, a money market fund at a rate) is said to three significant
/// figures wherever the answer words it: the rows, the options, the gaps.
#[test]
fn assumed_figures_are_rounded_to_three() {
    let three = |amount: Decimal| broker_fees::describe::card::three_figures(amount) == amount;
    for (start, _, answer) in every_answer() {
        for row in &answer.outcome.rows {
            if matches!(row.kind, Kind::MoneyMarketFund | Kind::Way(_)) {
                for amount in amounts_in(&row.amount.he) {
                    assert!(three(amount), "{start:?}: {}", row.amount.he);
                }
            }
        }
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            for option in &decision.options {
                // Where to buy a bond is a choice between sums known to the
                // shekel, said so (the review's N6).
                let known = !option.rows.is_empty()
                    && option.rows.iter().all(|row| {
                        matches!(
                            row,
                            Kind::GovernmentBond { .. } | Kind::FixedDeposit | Kind::LinkedDeposit
                        )
                    });
                let assumed = matches!(
                    option.pick,
                    Pick::Wrapper(_)
                        | Pick::Account(_)
                        | Pick::Cushion(_)
                        | Pick::Flexibility(_)
                        | Pick::SafeWay(SafeWay::MoneyMarketFund)
                ) && !(known && matches!(option.pick, Pick::Account(_)));
                if assumed {
                    for amount in amounts_in(&option.amount.he) {
                        assert!(three(amount), "{start:?}: {}", option.amount.he);
                    }
                }
            }
        }
        // The plan repeats no option's figure.
        if let Some(plan) = &answer.plan {
            let options: Vec<Decimal> = answer
                .lines
                .iter()
                .filter_map(|line| match &line.state {
                    LineState::Decision(decision) => Some(decision),
                    _ => None,
                })
                .flat_map(|decision| {
                    decision
                        .options
                        .iter()
                        .flat_map(|o| amounts_in(&o.amount.he))
                })
                .collect();
            for amount in amounts_in(&plan.he) {
                assert!(
                    !options.contains(&amount) || amount < dec!(100_000),
                    "{start:?}: {}",
                    plan.he
                );
            }
        }
    }
}

/// Finding 6: a money market fund is never "known almost in advance"; the
/// safe option says each kind it holds by how sure its sum is.
#[test]
fn each_safe_kind_is_said_by_its_own_certainty() {
    for (start, _, answer) in every_answer() {
        for text in he_texts(&answer) {
            assert!(!text.contains("ידוע כמעט מראש"), "{start:?}: {text}");
        }
        let Some(place) = decision(&answer, LineKind::PlaceType) else {
            continue;
        };
        let safe = &place.options[0];
        let kinds = &safe.facts[0].he;
        for row in answer
            .outcome
            .rows
            .iter()
            .filter(|row| place.options[0].rows.contains(&row.kind))
        {
            let sure = match row.kind {
                Kind::MoneyMarketFund => "הריבית משתנה",
                Kind::GovernmentBond { linked: true } => "כוח הקנייה ידוע מראש",
                _ => "ידוע מראש",
            };
            assert!(kinds.contains(sure), "{start:?}: {kinds}");
        }
    }
}

/// Finding 7 (principle 8): a decision that appears once another is
/// answered comes after it in the lines.
#[test]
fn a_dependent_decision_comes_after_its_parent() {
    let mut starts: Vec<Answers> = situations().into_iter().map(|s| s.answers).collect();
    starts.extend([apartment(), bonus(), salary()]);
    let ids = |answer: &Answer| -> Vec<DecisionId> {
        answer
            .lines
            .iter()
            .filter_map(|line| match &line.state {
                LineState::Decision(decision) => Some(decision.id),
                _ => None,
            })
            .collect()
    };
    let mut checked = 0;
    for start in starts {
        let before = answered(&start, Choices::default());
        for (at, line) in before.lines.iter().enumerate() {
            let LineState::Decision(parent) = &line.state else {
                continue;
            };
            for option in &parent.options {
                let (answers, choices) = pick(&start, Choices::default(), option.pick);
                let after = answered(&answers, choices);
                let was = ids(&before);
                for (later, line) in after.lines.iter().enumerate() {
                    if let LineState::Decision(child) = &line.state
                        && !was.contains(&child.id)
                    {
                        checked += 1;
                        assert!(
                            later > at,
                            "{:?}: {:?} appears above {:?}, which it depends on",
                            start.situation,
                            child.id,
                            parent.id
                        );
                    }
                }
            }
        }
    }
    assert!(checked > 0);
}

/// Finding 8: "open a new account, or buy through your bank?" is a decision
/// only when the bank is behind the best place by more than the margin
/// between kinds; within it, the איפה line states it.
#[test]
fn the_account_is_asked_only_beyond_the_kinds_margin() {
    let mut both_ways = (0, 0);
    for (start, choices, answer) in every_answer() {
        if choices.wrapper != Some(Wrapper::Anyone) {
            continue;
        }
        let Some(focus) = answer.focus else { continue };
        let Some(Kind::Way(way)) = Some(focus) else {
            continue;
        };
        if way.vehicle != broker_fees::Vehicle::Brokerage
            || matches!(start.situation, Some(Situation::ExistingPortfolio { .. }))
        {
            continue;
        }
        let how = answer.how.iter().find(|how| how.row == focus).unwrap();
        let places = how.where_to_buy.as_ref().unwrap();
        let Some(best) = places.rows.first() else {
            continue;
        };
        let reference = places
            .rows
            .iter()
            .find(|row| row.yours)
            .map(|row| row.left.today)
            .or(places.bank_range.map(|(_, high)| high.today));
        let Some(reference) = reference else { continue };
        let gap = best.left.today - reference;
        let asked = decision(&answer, LineKind::Where).is_some_and(|d| d.id == DecisionId::Account);
        let margin = kind_margin(answer.outcome.deposited.today);
        assert_eq!(asked, gap > margin, "{start:?}: gap {gap}, margin {margin}");
        if asked {
            both_ways.0 += 1;
        } else {
            both_ways.1 += 1;
        }
    }
    assert!(both_ways.0 > 0 && both_ways.1 > 0, "{both_ways:?}");
}

/// Finding 9: nothing is "your bank" until the visitor names one; with one
/// named, the apartment's bond through it is said against the money market
/// fund, and without, the banks' range is.
#[test]
fn nothing_is_yours_until_named() {
    for (start, _, answer) in every_answer() {
        let named = start.bank.is_some()
            || matches!(
                start.situation,
                Some(Situation::ExistingPortfolio { bank: Some(_) })
            );
        if !named {
            for text in he_texts(&answer) {
                assert!(!text.contains("הבנק שלכם"), "{start:?}: {text}");
                assert!(!text.contains("בבנק שלכם"), "{start:?}: {text}");
            }
        }
    }
    for (bank, says) in [(Some(Bank::Hapoalim), "דרך הבנק שלכם"), (None, "דרך בנק,")]
    {
        let answer = answered(
            &Answers {
                bank,
                ..apartment()
            },
            Choices::default(),
        );
        let way = decision(&answer, LineKind::Way).unwrap();
        let difference = &way.difference.as_ref().unwrap().he;
        assert!(difference.contains(says), "{difference}");
    }
}

/// Finding 10: one tie rule. The tie group is every place within the
/// providers' margin of the best, and the note under where to buy names
/// that group's size and its spread, its highest less its lowest.
#[test]
fn a_tie_note_measures_its_own_group() {
    for (start, _, answer) in every_answer() {
        let margin = provider_margin(answer.outcome.deposited.today);
        for how in &answer.how {
            let Some(places) = &how.where_to_buy else {
                continue;
            };
            let tied: Vec<&Provider> = places.rows.iter().filter(|row| row.tie == 0).collect();
            let Some(best) = places.rows.first() else {
                continue;
            };
            for row in &tied {
                assert!(
                    (best.left.today - row.left.today).abs() <= margin,
                    "{start:?}: {} is no tie",
                    row.name.he
                );
            }
            let Some(note) = &places.note else { continue };
            let high = tied.iter().map(|row| row.left.today).max().unwrap();
            let low = tied.iter().map(|row| row.left.today).min().unwrap();
            assert!(note.he.contains(&format!("{}", tied.len())), "{}", note.he);
            if high - low >= Decimal::ONE {
                let said = amounts_in(&note.he);
                assert_eq!(said, vec![(high - low).round()], "{start:?}: {}", note.he);
            }
        }
    }
}

/// Finding 11: with no date of the visitor's, no date is made up; and a row
/// the rules exclude doesn't stretch the axis, drawn as a tail beyond it.
#[test]
fn no_invented_date_and_no_axis_from_an_excluded_row() {
    let months = [
        "ינואר",
        "פברואר",
        "מרץ",
        "אפריל",
        "מאי",
        "יוני",
        "יולי",
        "אוגוסט",
        "ספטמבר",
        "אוקטובר",
        "נובמבר",
        "דצמבר",
    ];
    for (start, _, answer) in every_answer() {
        if start.horizon == Horizon::Unknown || start.needs_any_day {
            let dated = months
                .iter()
                .any(|month| answer.outcome.title.he.contains(month));
            assert!(!dated, "{start:?}: {}", answer.outcome.title.he);
            assert!(answer.outcome.title.he.contains("לדוגמה"));
            if let Some(plan) = &answer.plan {
                assert!(
                    !months.iter().any(|month| plan.he.contains(month)),
                    "{}",
                    plan.he
                );
            }
        }
        let (low, high) = answer.outcome.axis;
        for row in &answer.outcome.rows {
            if row.excluded.is_some() {
                let outside = row.value.today > high || row.value.today < low;
                assert_eq!(row.tail, outside, "{start:?}");
            } else {
                assert!(!row.tail);
                assert!(
                    row.value.today <= high && row.value.today >= low,
                    "{start:?}"
                );
            }
        }
    }
}

/// The review's Hebrew (X16): a fund's value rises and falls; the situations
/// put the article on the second noun.
#[test]
fn the_reviews_hebrew_is_fixed() {
    for preset in situations() {
        assert!(
            !preset.explanation.he.contains("הדמי ניהול"),
            "{}",
            preset.explanation.he
        );
    }
    for (_, _, answer) in every_answer() {
        for row in &answer.outcome.rows {
            assert!(row.tags.iter().all(|tag| tag.he != "עולה ויורד"));
        }
        for text in he_texts(&answer) {
            assert!(!text.contains("למדד S&P"), "{text}");
            assert!(!text.contains("(אם אין לכם, פותחים אותו שם)") || text.contains("באפליקציה"));
        }
    }
}

// ─────────────────────────── The second pass's findings ───────────────────────────

/// Whether a start is a portfolio already at a bank, whose place is a move.
fn is_existing(start: &Answers) -> bool {
    matches!(start.situation, Some(Situation::ExistingPortfolio { .. }))
}

/// N2: the visitor's bank, behind the best by no more than the margin that
/// would make staying a question, is in the stated place and in the plan,
/// named first, and the line never sends the visitor elsewhere alone; a
/// bank further behind isn't. The bond is left out: through a bank it can
/// leave no more than the money market fund in the same app.
#[test]
fn a_bank_within_the_margin_joins_the_stated_place() {
    let mut joined = 0;
    for (start, _, answer) in every_answer() {
        if start.bank.is_none() || is_existing(&start) {
            continue;
        }
        let Some(Line {
            state: LineState::Stated { text, .. },
            ..
        }) = line(&answer, LineKind::Where)
        else {
            continue;
        };
        let Some(focus) = answer.focus else { continue };
        let bought = matches!(focus, Kind::FixedDeposit)
            || matches!(focus, Kind::Way(way) if way.vehicle == broker_fees::Vehicle::Brokerage);
        if !bought {
            continue;
        }
        let how = answer.how.iter().find(|how| how.row == focus).unwrap();
        let rows = &how.where_to_buy.as_ref().unwrap().rows;
        let (Some(best), Some(yours)) = (rows.first(), rows.iter().find(|row| row.yours)) else {
            continue;
        };
        let margin = kind_margin(answer.outcome.deposited.today);
        let joins = yours.tie != 0 && best.left.today - yours.left.today <= margin;
        assert_eq!(
            text.he.contains("הבנק שלכם ("),
            joins,
            "{start:?}: {}",
            text.he
        );
        if joins {
            joined += 1;
            if let Some(plan) = &answer.plan {
                assert!(plan.he.contains("בבנק שלכם ("), "{start:?}: {}", plan.he);
            }
        }
    }
    assert!(joined > 0, "the rule is checked on a bank that joins");
}

/// N4: until a bank is named, every comparison measures from the banks'
/// range, and where one bank must stand in (a gain bar), it is the one that
/// leaves most, as the difference's lower end is.
#[test]
fn without_a_bank_every_part_measures_from_the_same_banks() {
    for (start, _, answer) in every_answer() {
        if start.bank.is_some() {
            continue;
        }
        for text in he_texts(&answer) {
            assert!(!text.contains("היקר מבין"), "{start:?}: {text}");
            assert!(!text.contains("לעומת בנק: בערך"), "{start:?}: {text}");
            assert!(!text.contains("לעומת הבנק הזול"), "{start:?}: {text}");
        }
        if let Some(decision) = decision(&answer, LineKind::Where)
            && decision.id == DecisionId::Account
        {
            let difference = decision.difference.as_ref().unwrap();
            assert!(difference.he.contains("לפי הבנק"), "{}", difference.he);
        }
    }
}

/// N5: a one-letter preposition before Latin letters takes a hyphen ("ב‑IBI"),
/// a range ends its sentence, and a provident fund is no קרן.
#[test]
fn the_second_passs_hebrew_is_fixed() {
    for (start, _, answer) in every_answer() {
        for text in he_texts(&answer) {
            let chars: Vec<char> = text.chars().collect();
            for at in 1..chars.len() {
                let starts_word = at == 1 || !chars[at - 2].is_alphanumeric();
                assert!(
                    !(starts_word
                        && "בלמוהכש".contains(chars[at - 1])
                        && chars[at].is_ascii_alphabetic()),
                    "{start:?}: {text}"
                );
            }
            assert!(!text.contains("לפי הבנק יותר"), "{start:?}: {text}");
            assert!(!text.contains("לפחות בערך"), "{start:?}: {text}");
            assert!(!text.contains("בבנק: בבנק"), "{start:?}: {text}");
        }
        let Some(place) = decision(&answer, LineKind::PlaceType) else {
            continue;
        };
        for option in &place.options {
            if option.title.he.contains("קופת גמל") {
                let said: Vec<&str> = option
                    .facts
                    .iter()
                    .map(|fact| fact.he.as_ref())
                    .chain(place.difference.iter().map(|text| text.he.as_ref()))
                    .collect();
                for text in said {
                    assert!(!text.contains("הקרן הייתה"), "{text}");
                    assert!(!text.contains("קרן על המדד"), "{text}");
                }
            }
        }
    }
}

/// N6: an option for a sum known in advance gives it as its row does, to
/// the shekel.
#[test]
fn a_known_sum_has_one_precision() {
    let mut checked = 0;
    for (start, _, answer) in every_answer() {
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            for option in &decision.options {
                let known: Vec<&Row> = option
                    .rows
                    .iter()
                    .filter_map(|id| answer.outcome.rows.iter().find(|row| row.id == *id))
                    .filter(|row| {
                        matches!(row.kind, Kind::GovernmentBond { .. } | Kind::FixedDeposit)
                    })
                    .collect();
                if known.is_empty() || known.len() != option.rows.len() {
                    continue;
                }
                assert!(
                    !option.amount.he.contains("בערך"),
                    "{start:?}: {}",
                    option.amount.he
                );
                if option.pick == Pick::Account(Account::NewAccount) {
                    checked += 1;
                    assert_eq!(
                        amounts_in(&option.amount.he),
                        amounts_in(&known[0].amount.he),
                        "{start:?}"
                    );
                }
            }
        }
    }
    assert!(checked > 0, "an account for a bond is checked");
}

/// N7: an option's second amount is one short line, in its own unit.
#[test]
fn an_options_second_amount_is_short() {
    for (_, _, answer) in every_answer() {
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            for option in &decision.options {
                if let Some(also) = &option.also {
                    assert!(also.he.chars().count() <= 60, "{}", also.he);
                    assert_eq!(
                        amounts_in(&also.he).len().clamp(1, 2),
                        amounts_in(&also.he).len()
                    );
                }
            }
        }
    }
}

// ─────────────────────────── The mutants' rules ───────────────────────────

/// Small portfolios already at a bank, where moving saves little: every
/// figure lies within the margins, so the rules' other side shows.
fn small_portfolios() -> Vec<(Answers, Choices, Answer)> {
    static ALL: LazyLock<Vec<(Answers, Choices, Answer)>> = LazyLock::new(|| {
        let mut out = Vec::new();
        for (now, months, bank) in [
            (dec!(20_000), 36, None),
            (dec!(20_000), 36, Some(Bank::Leumi)),
            (dec!(5000), 24, None),
            (dec!(5000), 24, Some(Bank::Discount)),
        ] {
            let start = Answers {
                situation: Some(Situation::ExistingPortfolio { bank }),
                bank: None,
                ..answers(now, dec!(0), months)
            };
            for holdings in [None, Some(Holdings::IndexFunds)] {
                let choices = Choices {
                    holdings,
                    ..Choices::default()
                };
                let answer = answered(&start, choices);
                out.push((start.clone(), choices, answer));
            }
        }
        out
    });
    ALL.clone()
}

/// What a nominal figure is to its today's one at the chart's date: the
/// current account's, which keeps the deposits as they are.
fn price_level(answer: &Answer) -> Decimal {
    let account = answer.outcome.current_account;
    account.nominal / account.today
}

/// Every figure on the chart's date has its nominal one at that date's
/// prices: the rows and their marks, the options, the places and their
/// differences, the banks' range.
#[test]
fn every_figure_at_the_date_has_its_nominal_at_the_date() {
    for (start, choices, answer) in every_answer() {
        // The index is worked out for whole years: beside a date that isn't
        // one, its figures are at its own (`the_charts_deposits_are_over_the_rows_months`).
        if matches!(start.horizon, Horizon::Months(months) if months % 12 != 0) {
            continue;
        }
        let level = price_level(&answer);
        let check = |amount: &Amount, what: &str| {
            let expected = amount.today * level;
            assert!(
                (amount.nominal - expected).abs() <= Decimal::ONE + expected.abs() / dec!(100_000),
                "{start:?} {choices:?} {what}: {amount:?} at {level}"
            );
        };
        for row in &answer.outcome.rows {
            check(&row.value, "a row");
            match &row.mark {
                Mark::RateRange { low, high } => {
                    check(low, "the rate bar's low");
                    check(high, "the rate bar's high");
                }
                Mark::Market {
                    worst,
                    median,
                    assumed,
                    best,
                    ..
                } => {
                    for (amount, what) in [
                        (worst, "worst"),
                        (median, "median"),
                        (assumed, "assumed"),
                        (best, "best"),
                    ] {
                        check(amount, what);
                    }
                }
                Mark::Known | Mark::Expected => {}
            }
        }
        for line in &answer.lines {
            if let LineState::Decision(decision) = &line.state {
                for value in decision.options.iter().filter_map(|option| option.value) {
                    check(&value, "an option");
                }
            }
        }
        for place in answer
            .how
            .iter()
            .filter_map(|how| how.where_to_buy.as_ref())
        {
            for row in &place.rows {
                check(&row.left, "a place");
                if let Some(versus) = &row.versus_yours {
                    check(versus, "against yours");
                }
            }
            if let Some((low, high)) = &place.bank_range {
                check(low, "the banks' low");
                check(high, "the banks' high");
            }
        }
    }
}

/// The deposits the chart draws are the visitor's own over the months its
/// rows are worked out for: the date, or for the index the whole years it
/// runs (70 months is 6 years), and a retirement's years.
#[test]
fn the_charts_deposits_are_over_the_rows_months() {
    let mut all: Vec<(Answers, Answer)> = every_answer()
        .into_iter()
        .map(|(start, _, answer)| (start, answer))
        .collect();
    // A date that isn't whole years, which can wait: the index's arena.
    let odd = Answers {
        flexibility: Some(Flexibility::CanWait),
        ..answers(dec!(10_000), dec!(1000), 66)
    };
    all.push((odd.clone(), answered(&odd, Choices::default())));
    for (start, answer) in &all {
        let deposits = |months: u32| start.now + start.monthly * Decimal::from(months);
        let drawn = answer.outcome.deposited.nominal;
        match start.horizon {
            Horizon::Months(months) => {
                let index = answer
                    .outcome
                    .rows
                    .iter()
                    .any(|row| row.active && matches!(row.kind, Kind::Way(_)));
                let whole = (months + 6) / 12 * 12;
                let expected = if index && !start.needs_any_day {
                    deposits(whole)
                } else {
                    deposits(months)
                };
                assert_eq!(drawn, expected, "{start:?}");
            }
            Horizon::Retirement { age, retire_at } => {
                assert_eq!(drawn, deposits((retire_at - age) * 12), "{start:?}");
            }
            Horizon::Unknown => {}
        }
    }
}

/// The plan is shown once every decision is made and no line waits.
#[test]
fn the_plan_comes_once_nothing_is_open() {
    let mut shown = 0;
    for (start, choices, answer) in every_answer() {
        let waiting = answer
            .lines
            .iter()
            .any(|line| line.state == LineState::Waiting);
        let open = answer.open_decisions > 0 || waiting;
        assert_eq!(answer.plan.is_some(), !open, "{start:?} {choices:?}");
        shown += usize::from(!open);
    }
    assert!(shown > 0);
}

/// Where to buy: the best's tie group and the next, four at most, and the
/// visitor's bank always, as the reference the others are measured from;
/// a plan that may cost more is warned once, on its row or under them;
/// and the bank is asked only where naming it would move the reference by
/// more than the providers' margin.
#[test]
fn where_to_buy_shows_the_ties_and_measures_from_your_bank() {
    let (mut warned, mut unlocks, mut asked, mut not_asked) = (0, 0, 0, 0);
    for (start, choices, answer) in every_answer().into_iter().chain(small_portfolios()) {
        let lead = answer
            .outcome
            .rows
            .iter()
            .find(|row| matches!(row.kind, Kind::Way(_)))
            .map(|row| row.id);
        let margin = provider_margin(answer.outcome.deposited.today);
        for how in &answer.how {
            let Some(place) = &how.where_to_buy else {
                continue;
            };
            let label = format!("{start:?} {choices:?} {:?}", how.row);
            let others: Vec<&Provider> = place.rows.iter().filter(|row| !row.yours).collect();
            assert!(others.len() <= 4, "{label}");
            assert!(others.iter().all(|row| row.tie <= 1), "{label}");
            let yours = place.rows.iter().find(|row| row.yours);
            let existing = is_existing(&start);
            assert_eq!(
                place.title,
                broker_fees::describe::card::where_title(yours.is_some(), existing, &how.name),
                "{label}: the title says what the rows are measured from"
            );
            for row in &place.rows {
                let expected = yours.filter(|_| !row.yours).map(|yours| Amount {
                    today: row.left.today - yours.left.today,
                    nominal: row.left.nominal - yours.left.nominal,
                });
                assert_eq!(row.versus_yours, expected, "{label}");
            }
            let warnings = place
                .rows
                .iter()
                .filter(|row| row.warning.is_some())
                .count();
            assert!(warnings <= 1, "{label}");
            assert!(warnings == 0 || place.may_cost_more.is_none(), "{label}");
            warned += usize::from(warnings > 0 || place.may_cost_more.is_some());
            unlocks += usize::from(place.unlock.is_some());
            // What a bigger first deposit or a plan that may cost more would
            // do is said of the way the rules lead with, under its places.
            if Some(how.row) != lead {
                assert!(warnings == 0 && place.may_cost_more.is_none(), "{label}");
                assert!(place.unlock.is_none(), "{label}");
            }
            if how.row == Kind::MoneyMarketFund {
                assert!(place.rows.is_empty() && place.note.is_some(), "{label}");
            }
            // The banks are one row until one is named (§4.7).
            if let Some(reference) = &place.reference {
                assert!(place.rows.iter().all(|row| !row.yours), "{label}");
                assert_eq!(
                    Some((reference.low, reference.high)),
                    place.bank_range,
                    "{label}"
                );
            }
            let best = place
                .rows
                .first()
                .map(|row| row.left.today)
                .into_iter()
                .chain(
                    place
                        .reference
                        .as_ref()
                        .map(|reference| reference.high.today),
                )
                .max();
            let widens = start.bank.is_none()
                && how.row != Kind::MoneyMarketFund
                && best
                    .zip(place.bank_range)
                    .is_some_and(|(best, (low, _))| best - low.today > margin);
            assert_eq!(place.asks_bank, widens, "{label}");
            asked += usize::from(place.asks_bank);
            not_asked += usize::from(start.bank.is_none() && place.bank_range.is_some() && !widens);
        }
    }
    assert!(
        warned > 0 && unlocks > 0 && asked > 0 && not_asked > 0,
        "{warned} {unlocks} {asked} {not_asked}"
    );
}

/// Every row of a kind bought at places has its places, the row's own
/// figure among them: the best place's.
#[test]
fn every_rows_places_hold_its_figure() {
    for (start, choices, answer) in every_answer() {
        for how in &answer.how {
            let Some(row) = answer.outcome.rows.iter().find(|row| row.id == how.row) else {
                continue;
            };
            if !matches!(
                row.kind,
                Kind::Way(_) | Kind::GovernmentBond { .. } | Kind::FixedDeposit
            ) {
                continue;
            }
            let place = how.where_to_buy.as_ref().unwrap();
            let places = &place.rows;
            let ends = place
                .reference
                .as_ref()
                .map(|reference| [reference.low.today, reference.high.today]);
            assert!(
                places
                    .iter()
                    .map(|place| place.left.today)
                    .chain(ends.into_iter().flatten())
                    .any(|left| (left - row.value.today).abs() < dec!(0.01)),
                "{start:?} {choices:?} {:?}: {} among {:?}",
                row.kind,
                row.value.today,
                places
                    .iter()
                    .map(|place| place.left.today)
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// The rows follow the choices (§2.6): before a way is chosen every
/// option's rows are lit, once chosen only its own; a row the rules exclude
/// is never lit; and only the money market fund's bar carries the rate's
/// note.
#[test]
fn the_rows_light_with_the_choices() {
    for (start, choices, answer) in every_answer() {
        let label = format!("{start:?} {choices:?}");
        let active = |id: &RowId| {
            answer
                .outcome
                .rows
                .iter()
                .find(|row| row.id == *id)
                .map(|row| row.active)
        };
        for row in &answer.outcome.rows {
            assert!(
                row.excluded.is_none() || !row.active,
                "{label} {:?}",
                row.kind
            );
            if matches!(row.mark, Mark::Known) && row.kind.arena() == Some(Arena::Safe) {
                assert!(row.note.is_none(), "{label} {:?}", row.kind);
            }
        }
        for line in &answer.lines {
            let LineState::Decision(decision) = &line.state else {
                continue;
            };
            if !matches!(decision.id, DecisionId::SafeWay | DecisionId::Wrapper) {
                continue;
            }
            let chosen = decision
                .options
                .iter()
                .find(|option| Some(option.pick) == decision.chosen);
            for option in &decision.options {
                for id in &option.rows {
                    let lit = match chosen {
                        None => true,
                        Some(chosen) => chosen.rows.contains(id),
                    };
                    if let Some(active) = active(id) {
                        assert_eq!(active, lit, "{label} {:?} {id:?}", option.pick);
                    }
                }
            }
        }
    }
}

/// A study fund is offered beside the way anyone can open only when it
/// leaves more by more than the margin between kinds.
#[test]
fn a_study_fund_is_asked_only_beyond_the_margin() {
    let mut asked = 0;
    for (start, choices, answer) in every_answer() {
        let Some(wrapper) =
            decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::Wrapper)
        else {
            continue;
        };
        asked += 1;
        let value = |pick| {
            wrapper
                .options
                .iter()
                .find(|option| option.pick == Pick::Wrapper(pick))
                .unwrap()
        };
        let (anyone, study) = (value(Wrapper::Anyone), value(Wrapper::StudyFund));
        let gap = study.value.unwrap().today - anyone.value.unwrap().today;
        let margin = kind_margin(answer.outcome.deposited.today);
        assert!(gap > margin, "{start:?} {choices:?}: {gap} under {margin}");
        assert!(
            study
                .rows
                .iter()
                .all(|row| matches!(row, Kind::Way(way) if way.vehicle == broker_fees::Vehicle::StudyFund)),
            "{start:?}"
        );
    }
    assert!(asked > 0);
}

/// A portfolio already at a bank: it is moved as it is, an index fund in
/// Tel Aviv; the chip says "hard to go wrong" exactly when moving saves
/// more than the margin between kinds, against the named bank or the bank
/// that leaves most, and the move is "clear" only then and with one place
/// to move to, and the plan moves it only then; for managed funds or holdings not known, the gain isn't
/// drawn, since no fee decides anything there.
#[test]
fn a_portfolio_moves_when_it_saves_beyond_the_margin() {
    let (mut cases, mut states) = (0, HashSet::new());
    let small: Vec<Answers> = small_portfolios()
        .into_iter()
        .map(|(start, _, _)| start)
        .collect();
    for start in starts().into_iter().filter(is_existing).chain(small) {
        for holdings in [None, Some(Holdings::IndexFunds)] {
            let answer = answered(
                &start,
                Choices {
                    holdings,
                    ..Choices::default()
                },
            );
            let lead = &answer.outcome.rows[0];
            let Kind::Way(way) = lead.kind else {
                panic!("{start:?}: the lead is the index")
            };
            assert_eq!(way.vehicle, broker_fees::Vehicle::Brokerage);
            assert_eq!(way.product, broker_fees::Product::IsraeliIndexFund);
            let place = answer
                .how
                .iter()
                .find(|how| how.row == lead.id)
                .and_then(|how| how.where_to_buy.as_ref())
                .unwrap();
            let best = place.rows[0].left.today;
            let reference = place
                .rows
                .iter()
                .find(|row| row.yours)
                .map(|row| row.left.today)
                .or(place.bank_range.map(|(_, high)| high.today))
                .unwrap();
            let margin = kind_margin(answer.outcome.deposited.today);
            let worth = best - reference > margin;
            assert_eq!(
                answer.confidence.state == ConfidenceState::A,
                worth,
                "{start:?}: {best} from {reference}"
            );
            states.insert(worth);
            if holdings.is_some() {
                let one = place.rows.iter().filter(|row| row.tie == 0).count() == 1;
                let Some(LineState::Stated { pill, .. }) =
                    line(&answer, LineKind::Where).map(|line| &line.state)
                else {
                    panic!("{start:?}: the move is stated");
                };
                assert_eq!(*pill == Pill::Clear, worth && one, "{start:?}");
                // The plan moves it only when that saves (finding 2).
                let plan = answer.plan.as_ref().expect("the move's plan");
                assert_eq!(
                    plan.en.contains("Keep the portfolio where it is"),
                    !worth,
                    "{start:?}: {}",
                    plan.en
                );
            }
            cases += 1;
        }
        for holdings in [Holdings::Managed, Holdings::DontKnow] {
            let answer = answered(
                &start,
                Choices {
                    holdings: Some(holdings),
                    ..Choices::default()
                },
            );
            assert!(answer.gain.is_none(), "{start:?} {holdings:?}");
        }
    }
    assert!(cases > 0 && states.len() == 2, "{states:?}");
}

/// The cushion's options (finding 1): each says what it leaves whichever is
/// chosen, and that is what the chart's lit row leaves once it is chosen;
/// "not yet" leaves less, the money waiting in a fund first; and it says
/// the month's expenses it is sized by. Where the cushion takes all the
/// time but a year, it is still a path; past that, every shekel goes to it.
#[test]
fn the_cushions_options_keep_their_figures() {
    let starts = [
        (answers(dec!(0), dec!(500), 360), None),
        (answers(dec!(20_000), dec!(2000), 360), None),
        // 60 months to fill, a year left for the index: a path.
        (answers(dec!(0), dec!(400), 72), Some(Flexibility::CanWait)),
        // 80 months to fill: all of it goes to the cushion.
        (answers(dec!(0), dec!(300), 72), Some(Flexibility::CanWait)),
    ];
    for (start, flexibility) in starts {
        let start = Answers {
            flexibility,
            ..start
        };
        let with = |cushion| {
            answered(
                &Answers {
                    cushion,
                    ..start.clone()
                },
                Choices::default(),
            )
        };
        let open = with(None);
        let has = with(Some(Cushion::Has));
        let has_not = with(Some(Cushion::HasNot {
            monthly_expenses: None,
        }));
        let option = |answer: &Answer, pick| {
            decision(answer, LineKind::Before)
                .unwrap_or_else(|| panic!("{start:?}: the cushion is asked"))
                .options
                .iter()
                .find(|option| option.pick == Pick::Cushion(pick))
                .cloned()
                .unwrap()
        };
        let asked = decision(&open, LineKind::Before).unwrap();
        let texts: String = [&asked.question, &asked.why]
            .into_iter()
            .chain(asked.options.iter().map(|option| &option.amount))
            .chain(asked.options.iter().flat_map(|option| &option.facts))
            .map(|text| text.he.to_string())
            .collect();
        assert!(
            amounts_in(&texts).contains(&STAND_IN_MONTHLY_EXPENSES),
            "{texts}"
        );
        for (pick, chosen) in [(CushionPick::Has, &has), (CushionPick::HasNot, &has_not)] {
            let amount = option(&open, pick).value.unwrap();
            let at_the_date = amount.today * price_level(&open);
            assert!(
                (amount.nominal - at_the_date).abs() < Decimal::ONE,
                "{start:?} {amount:?}"
            );
            let value = amount.today;
            for answer in [&has, &has_not] {
                assert_eq!(
                    option(answer, pick).value.unwrap().today,
                    value,
                    "{start:?} {pick:?}"
                );
            }
            let first = option(chosen, pick).rows[0];
            let row = chosen
                .outcome
                .rows
                .iter()
                .find(|row| row.id == first)
                .unwrap();
            assert!(row.active, "{start:?} {pick:?}");
            assert_eq!(row.value.today, value, "{start:?} {pick:?}");
        }
        assert!(
            option(&open, CushionPick::HasNot).value.unwrap().today
                < option(&open, CushionPick::Has).value.unwrap().today,
            "{start:?}"
        );
        let takes_all = start.monthly == dec!(300);
        let index_lit = has_not
            .outcome
            .rows
            .iter()
            .any(|row| row.active && matches!(row.kind, Kind::Way(_)));
        assert_eq!(index_lit, !takes_all, "{start:?}");
    }
}

/// The past said beside the growth option is on its figures' basis: "even
/// less than was put in" exactly when the index's worst stretch leaves less
/// than the deposits.
#[test]
fn even_less_than_was_put_in_only_when_it_was() {
    let mut said = [0, 0];
    for (start, choices, answer) in every_answer() {
        if decision(&answer, LineKind::PlaceType).is_none() {
            continue;
        }
        let Some(worst) = answer.outcome.rows.iter().find_map(|row| match &row.mark {
            Mark::Market { worst, .. } => Some(worst.today),
            _ => None,
        }) else {
            continue;
        };
        let below = worst < answer.outcome.deposited.today;
        let place = decision(&answer, LineKind::PlaceType).unwrap();
        let texts: String = [&answer.body, &place.question, &place.why]
            .into_iter()
            .chain(&place.difference)
            .chain(place.options.iter().map(|option| &option.amount))
            .chain(place.options.iter().flat_map(|option| &option.facts))
            .map(|text| text.he.to_string())
            .collect();
        let says = texts.contains("ואפילו פחות ממה ש");
        assert_eq!(says, below, "{start:?} {choices:?}");
        said[usize::from(below)] += 1;
    }
    assert!(said[0] > 0 && said[1] > 0, "{said:?}");
}

/// A safe answer the rules state is "hard to go wrong" exactly when its way
/// is stated too: one way leads clearly, in every version, or there is one.
#[test]
fn a_safe_answer_is_clear_when_its_way_is() {
    let mut clear = 0;
    // Monthly money for 10 months: a fund alone (a deposit takes one sum).
    let short = answers(dec!(0), dec!(1000), 10);
    let extra = (
        short.clone(),
        Choices::default(),
        answered(&short, Choices::default()),
    );
    for (start, choices, answer) in every_answer().into_iter().chain([extra]) {
        let safe = answer
            .outcome
            .rows
            .iter()
            .filter(|row| row.active)
            .all(|row| row.kind.arena() == Some(Arena::Safe));
        if !safe || !is_stated(&answer, LineKind::PlaceType) || is_existing(&start) {
            continue;
        }
        let stated = is_stated(&answer, LineKind::Way);
        assert_eq!(
            answer.confidence.state == ConfidenceState::A,
            stated,
            "{start:?} {choices:?}"
        );
        clear += usize::from(stated);
    }
    assert!(clear > 0);
}

/// For retirement, once the rules state the index, the card gives the
/// monthly pension the provident fund's sum would pay, as a range.
#[test]
fn a_stated_retirement_gives_the_pension() {
    let start = Answers {
        horizon: Horizon::Retirement {
            age: 30,
            retire_at: 67,
        },
        cushion: Some(Cushion::Has),
        ..answers(dec!(0), dec!(3000), 0)
    };
    let answer = answered(&start, Choices::default());
    assert!(
        is_stated(&answer, LineKind::PlaceType),
        "{:?}",
        answer.lines
    );
    let annuity = answer.annuity.expect("the pension");
    assert_eq!(amounts_in(&annuity.he).len(), 2, "{}", annuity.he);
}

/// "How much of the gain stays" is drawn only where fees or tax decide
/// something on the page (§2.7): the account or the move (fees), the study
/// fund (tax), or retirement's way out (tax), and then for what that
/// decision weighs: the study fund beside the account only once it was
/// offered, and for retirement the brokerage account beside the provident
/// fund.
#[test]
fn the_gain_is_drawn_only_where_fees_or_tax_decide() {
    let mut retirements = 0;
    for (start, choices, answer) in every_answer().into_iter().chain(small_portfolios()) {
        let Some(gain) = &answer.gain else { continue };
        let label = format!("{start:?} {choices:?}");
        let asked = |id| {
            answer
                .lines
                .iter()
                .any(|line| matches!(&line.state, LineState::Decision(d) if d.id == id))
        };
        let moved = is_existing(&start) && choices.holdings == Some(Holdings::IndexFunds);
        let index_lit = answer
            .outcome
            .rows
            .iter()
            .any(|row| row.active && matches!(row.kind, Kind::Way(_)));
        let retirement = matches!(start.horizon, Horizon::Retirement { .. }) && index_lit;
        assert!(
            asked(DecisionId::Account) || asked(DecisionId::Wrapper) || moved || retirement,
            "{label}"
        );
        assert!(gain.rows.len() <= 3, "{label}");
        if (asked(DecisionId::Account) || moved) && !asked(DecisionId::Wrapper) {
            assert_eq!(gain.rows.len(), 2, "{label}");
        }
        if retirement && !asked(DecisionId::Account) && !asked(DecisionId::Wrapper) {
            retirements += 1;
            let name = |vehicle| {
                answer.outcome.rows.iter().find_map(|row| match row.kind {
                    Kind::Way(way) if way.vehicle == vehicle => Some(row.name.clone()),
                    _ => None,
                })
            };
            let names: Vec<broker_fees::Text> =
                gain.rows.iter().map(|row| row.name.clone()).collect();
            assert_eq!(
                names,
                [
                    name(broker_fees::Vehicle::Brokerage).unwrap(),
                    name(broker_fees::Vehicle::InvestmentGemel).unwrap()
                ],
                "{label}"
            );
        }
    }
    assert!(retirements > 0);
}

// ─────────────────────────── Part 4 ───────────────────────────

/// The lines' names (§4.1).
#[test]
fn the_lines_are_named_by_what_they_decide() {
    let answer = answered(&salary(), Choices::default());
    let labels: Vec<&str> = answer.lines.iter().map(|line| &*line.label.he).collect();
    for label in &labels {
        assert!(
            ["כסף לחירום", "מניות או בטוח", "המוצר", "איפה קונים"].contains(label),
            "{label}"
        );
    }
    assert!(labels.contains(&"מניות או בטוח"));
}

/// Money that may be needed any day (§4.9) compares every place it can be
/// taken from the next day, ranked like any safe answer, over a year as an
/// example: no stock row, no term the visitor never gave.
#[test]
fn money_needed_any_day_compares_what_can_be_taken_out_the_next_day() {
    for bank in [None, Some(Bank::Hapoalim)] {
        let any_day = Answers {
            horizon: Horizon::Unknown,
            needs_any_day: true,
            bank,
            ..answers(dec!(30_000), dec!(0), 0)
        };
        let answer = answered(&any_day, Choices::default());
        let kinds: Vec<Kind> = answer.outcome.rows.iter().map(|row| row.kind).collect();
        assert!(kinds.len() >= 2, "{kinds:?}");
        for kind in &kinds {
            assert!(
                [
                    Kind::MoneyMarketFund,
                    Kind::AnyDayDeposit,
                    Kind::TreasuryBill
                ]
                .contains(kind),
                "{kind:?}"
            );
        }
        assert!(kinds.contains(&Kind::MoneyMarketFund));
        // Ranked: the rows go by what they leave.
        let values: Vec<Decimal> = answer
            .outcome
            .rows
            .iter()
            .map(|row| row.value.today)
            .collect();
        assert!(
            values.windows(2).all(|pair| pair[0] >= pair[1]),
            "{values:?}"
        );
        assert!(
            answer.outcome.title.he.contains("בעוד שנה"),
            "{}",
            answer.outcome.title.he
        );
        assert!(!answer.outcome.caption.he.contains("S&P"));
        for line in &answer.lines {
            if let LineState::Stated { text, .. } = &line.state {
                assert!(!text.he.contains("לטווח"), "{}", text.he);
            }
        }
        // The title and the chip follow the numbers.
        let stated = is_stated(&answer, LineKind::Way);
        assert_eq!(stated, answer.confidence.state == ConfidenceState::A);
        // Each says what else differs: where it's kept, what it asks.
        for row in &answer.outcome.rows {
            assert!(!row.facts.he.is_empty(), "{:?}", row.kind);
        }
    }
}

/// The plan (§4.5): once every decision is made, its amount, the date and
/// how sure; for stocks, what the figures were measured on.
#[test]
fn the_plan_has_its_amount() {
    for (start, choices, answer) in every_answer() {
        assert_eq!(
            answer.plan.is_some(),
            answer.plan_amount.is_some(),
            "{start:?} {choices:?}"
        );
        let Some(amount) = &answer.plan_amount else {
            continue;
        };
        let stocks = answer
            .focus
            .is_some_and(|focus| matches!(focus, Kind::Way(_)));
        assert_eq!(amount.example.is_some(), stocks, "{start:?} {choices:?}");
        assert!(amount.basis.he.contains("בכסף של היום, אחרי מס"));
        assert!(amount.value.today > Decimal::ZERO);
    }
}

/// "איך הגעתם לזה" (§4.5): a step for each line, in the answer's order;
/// a decided step opens its decision again, a stated one carries its pill;
/// "איזה מדד" follows the place type wherever the money goes to stocks,
/// the visitor's call outside the site.
#[test]
fn the_path_is_the_lines_in_order() {
    let mut why = 0;
    for (start, choices, answer) in every_answer() {
        let label = format!("{start:?} {choices:?}");
        let open = answer
            .path
            .iter()
            .filter(|step| step.kind == StepKind::Open)
            .count();
        assert_eq!(open, answer.open_decisions as usize, "{label}");
        // Every step has its words, an open one too (its question and why).
        for step in &answer.path {
            assert!(
                !step.choice.he.trim().is_empty() && !step.hinge.he.trim().is_empty(),
                "{label}: {step:?}"
            );
        }
        if answer.plan.is_none() {
            continue;
        }
        let lines: Vec<LineKind> = answer.lines.iter().map(|line| line.kind).collect();
        let stepped: Vec<LineKind> = answer.path.iter().filter_map(|step| step.line).collect();
        assert_eq!(stepped, lines, "{label}");
        for step in &answer.path {
            let line = step.line.and_then(|kind| line(&answer, kind));
            match (step.kind, line.map(|line| &line.state)) {
                (StepKind::Visitor, Some(LineState::Decision(decision))) => {
                    assert_eq!(step.reopens, Some(decision.id), "{label}");
                    assert!(decision.chosen.is_some(), "{label}");
                }
                (StepKind::Site, Some(LineState::Stated { pill, .. })) => {
                    assert_eq!(step.pill, Some(*pill), "{label}");
                    assert!(step.reopens.is_none());
                }
                (StepKind::Outside, None) => assert_eq!(step.label.he, "איזה מדד"),
                (kind, state) => panic!("{label}: {kind:?} for {state:?}"),
            }
            assert!(
                !step.choice.he.is_empty() && !step.hinge.he.is_empty(),
                "{label}: {step:?}"
            );

            if let Some(clear) = &step.why_clear {
                why += 1;
                assert_eq!(step.kind, StepKind::Site);
                assert_eq!(step.line, Some(LineKind::PlaceType));
                assert_eq!(clear.amounts.len(), 4, "{label}");
                assert!(clear.count.he.contains("מאז 1926"), "{}", clear.count.he);
                assert!(clear.past.he.contains("לא הבטחה"));
                assert!(clear.past.he.contains("S&P"));
            }
        }
        let stocks = answer.lines.iter().any(|line| line.kind == LineKind::Way)
            && answer
                .focus
                .is_some_and(|focus| matches!(focus, Kind::Way(_)))
            && !is_existing(&start);
        let outside = answer
            .path
            .iter()
            .position(|step| step.kind == StepKind::Outside);
        assert_eq!(outside.is_some(), stocks, "{label}");
        if let Some(at) = outside {
            assert_eq!(
                answer.path[at - 1].line,
                Some(LineKind::PlaceType),
                "{label}"
            );
        }
    }
    assert!(why > 0, "some place type is stated with its explanation");
}

/// "למה זה ברור?" for 30 years of stocks (§4.5): every period since 1926,
/// none ended below what was deposited, and the worst left more than the
/// safe way; the four amounts in order, from the engine.
#[test]
fn why_stocks_are_clear_over_thirty_years() {
    let thirty = Answers {
        cushion: Some(Cushion::Has),
        bank: None,
        ..salary()
    };
    let answer = answered(
        &thirty,
        Choices {
            wrapper: Some(Wrapper::Anyone),
            account: Some(Account::NewAccount),
            ..Choices::default()
        },
    );
    let step = answer
        .path
        .iter()
        .find(|step| step.why_clear.is_some())
        .expect("the place type is stated");
    let clear = step.why_clear.as_ref().unwrap();
    let values: Vec<Decimal> = clear
        .amounts
        .iter()
        .map(|figure| figure.value.today)
        .collect();
    // Deposited < the safe way < the worst stretch < the yearly return.
    assert!(
        values.windows(2).all(|pair| pair[0] < pair[1]),
        "{values:?}"
    );
    let stock = answer
        .outcome
        .rows
        .iter()
        .find(|row| matches!(row.kind, Kind::Way(_)))
        .unwrap();
    let Mark::Market { worst, assumed, .. } = &stock.mark else {
        panic!("the market's range");
    };
    assert_eq!(values[2], worst.today);
    assert_eq!(values[3], assumed.today);
    assert!(clear.count.he.contains("810"), "{}", clear.count.he);
    assert_eq!(step.hinge.he.replace('\u{a0}', " "), "הכסף מחכה 30 שנים.");
}

/// The outcome chart (§4.6): each mark on a stock band labelled with its
/// amount, lowest first; the reference lines with theirs; the facts as one
/// line; the caption names the S&P 500 only where stocks are drawn.
#[test]
fn the_chart_labels_every_mark() {
    for (start, choices, answer) in every_answer() {
        let label = format!("{start:?} {choices:?}");
        let stocks = answer
            .outcome
            .rows
            .iter()
            .any(|row| matches!(row.kind, Kind::Way(_)));
        assert_eq!(answer.outcome.caption.he.contains("S&P"), stocks, "{label}");
        for row in &answer.outcome.rows {
            assert!(!row.facts.he.is_empty() || row.tags.is_empty(), "{label}");
            match &row.mark {
                Mark::Market { .. } => {
                    assert_eq!(row.marks.len(), 4, "{label}");
                    let values: Vec<Decimal> =
                        row.marks.iter().map(|mark| mark.value.today).collect();
                    assert!(values.windows(2).all(|pair| pair[0] <= pair[1]), "{label}");
                    assert!(row.marks.iter().all(|mark| mark.amount.he.starts_with('₪')));
                }
                _ => assert!(row.marks.is_empty(), "{label}"),
            }
        }
    }
}

/// Where to buy (§4.7): until a bank is named, "דרך בנק" is one row from the
/// dearest bank to the cheapest, and the difference is said against it.
#[test]
fn where_to_buy_measures_from_the_banks_as_one_row() {
    let mut said = 0;
    for (start, choices, answer) in every_answer() {
        for how in &answer.how {
            let Some(place) = &how.where_to_buy else {
                continue;
            };
            if let Some(reference) = &place.reference {
                assert!(start.bank.is_none(), "{start:?}");
                assert!(reference.low.today <= reference.high.today);
                assert_eq!(reference.name.he, "דרך בנק");
            }
            if let Some(difference) = &place.difference {
                said += 1;
                let against = if start.bank.is_some() {
                    "בבנק שלכם"
                } else {
                    "דרך בנק"
                };
                assert!(
                    difference.he.contains(against),
                    "{start:?} {choices:?}: {}",
                    difference.he
                );
            }
        }
    }
    assert!(said > 0);
}

/// The gain's parts (§4.7): each with its name and amount, the fund's own
/// cost a part of its own; together the gain.
#[test]
fn the_gain_parts_are_named_and_add_up() {
    let mut seen = 0;
    for (_, _, answer) in every_answer() {
        let Some(gain) = &answer.gain else { continue };
        for row in &gain.rows {
            seen += 1;
            let sum: Decimal = row.parts.iter().map(|part| part.value.today).sum();
            let whole = row.kept.today + row.tax.today + row.fees.today + row.product.today;
            assert_eq!(sum, whole);
            assert_eq!(row.note.is_some(), row.tax.today.is_zero());
            assert!(
                row.parts
                    .iter()
                    .any(|part| part.which == GainPartKind::Fund)
            );
        }
    }
    assert!(seen > 0);
}

/// The facts in one line (§4.3), from the answers alone.
#[test]
fn the_facts_fold_into_one_line() {
    let line = |answers: &Answers| {
        broker_fees::describe::card::facts_line(answers)
            .he
            .replace('\u{a0}', " ")
    };
    assert_eq!(
        line(&apartment()),
        "₪100,000 עכשיו, ₪2,000 בחודש, בעוד 3 שנים"
    );
    assert_eq!(line(&salary()), "₪500 בחודש, בעוד 30 שנים");
    let any_day = Answers {
        horizon: Horizon::Unknown,
        needs_any_day: true,
        ..answers(dec!(30_000), dec!(0), 0)
    };
    assert_eq!(line(&any_day), "₪30,000 עכשיו, לשעת חירום");
    let retire = Answers {
        horizon: Horizon::Retirement {
            age: 30,
            retire_at: 67,
        },
        ..salary()
    };
    assert_eq!(line(&retire), "₪500 בחודש, עד גיל 67");
}

// ─────────────────────────── The phone's table (§4.3) ───────────────────────────

/// What an option asks is some of its own facts, each once; its "מתאים אם"
/// for the table's row of that name is the same sentence without those
/// words; and the decisions about effort each say what they ask.
#[test]
fn an_options_asks_are_its_facts_and_its_fits_is_its_fits_if() {
    let mut asked: Vec<DecisionId> = Vec::new();
    for start in starts() {
        for choices in every_choice() {
            let answer = answered(&start, choices);
            for line in &answer.lines {
                let LineState::Decision(decision) = &line.state else {
                    continue;
                };
                for option in &decision.options {
                    for asks in &option.asks {
                        assert!(
                            option.facts.contains(asks),
                            "{:?}: {asks:?} isn't one of its facts",
                            decision.id
                        );
                    }
                    let unique: HashSet<&str> = option.asks.iter().map(|asks| &*asks.he).collect();
                    assert_eq!(unique.len(), option.asks.len(), "{:?}", decision.id);
                    match (&option.fits_if, &option.fits) {
                        (Some(fits_if), Some(fits)) => {
                            assert_eq!(fits_if.he, format!("מתאים אם: {}", fits.he));
                            assert_eq!(fits_if.en, format!("Fits if: {}", fits.en));
                        }
                        (None, None) => {}
                        other => panic!("{:?}: {other:?}", decision.id),
                    }
                    if !option.asks.is_empty() && !asked.contains(&decision.id) {
                        asked.push(decision.id);
                    }
                }
            }
        }
    }
    for id in [
        DecisionId::Cushion,
        DecisionId::PlaceType,
        DecisionId::SafeWay,
        DecisionId::Wrapper,
        DecisionId::Account,
    ] {
        assert!(asked.contains(&id), "{id:?} never says what it asks");
    }
}

// ─────────────────────────── Mutants missed on 2026-10-21 ───────────────────────────

/// What the lead index row's worst past leaves, where the chart draws one.
fn lead_worst(answer: &Answer) -> Option<Decimal> {
    let place = decision(answer, LineKind::PlaceType)?;
    let lead = place
        .options
        .iter()
        .flat_map(|option| &option.rows)
        .find(|row| matches!(row, Kind::Way(_)))?;
    let row = answer.outcome.rows.iter().find(|row| row.id == *lead)?;
    match &row.mark {
        Mark::Market { worst, .. } => Some(worst.today),
        _ => None,
    }
}

/// "Even less than was put in" is said, in the place's decision and in the
/// body above it, exactly when the lead row's worst past ends below the
/// deposits the chart draws.
#[test]
fn less_than_was_put_in_is_said_only_below_the_deposits() {
    let mut seen = HashSet::new();
    for (start, _, answer) in every_answer() {
        let Some(worst) = lead_worst(&answer) else {
            continue;
        };
        let below = worst < answer.outcome.deposited.today;
        let difference = &decision(&answer, LineKind::PlaceType)
            .unwrap()
            .difference
            .as_ref()
            .expect("the place's difference")
            .en;
        assert_eq!(
            difference.contains("even less than was put in"),
            below,
            "{start:?}: {worst} against {}: {difference}",
            answer.outcome.deposited.today
        );
        if answer
            .body
            .en
            .contains("in the worst stretches of the past")
        {
            assert_eq!(
                answer.body.en.contains("even less than was put in"),
                below,
                "{start:?}: {}",
                answer.body.en
            );
            seen.insert((below, true));
        }
        seen.insert((below, false));
    }
    assert!(
        seen.contains(&(true, true)) && seen.contains(&(false, false)),
        "{seen:?}"
    );
}

/// Once the place is stated safe, the way is stated exactly when the chip
/// says it's hard to go wrong (one way leads clearly in every version, or
/// there is one), whatever rows the chart lights; and a stated safe way
/// names its term, except for money that may be needed any day.
#[test]
fn a_stated_safe_place_states_its_way_when_it_leads_clearly() {
    // Where the rules state the place: a lump sum for months, money that
    // may be needed any day, and monthly money for months (a fund alone).
    let extras = [
        answers(dec!(300_000), dec!(0), 10),
        answers(dec!(50_000), dec!(0), 6),
        answers(dec!(20_000), dec!(1000), 6),
        Answers {
            needs_any_day: true,
            ..answers(dec!(50_000), dec!(0), 24)
        },
        Answers {
            horizon: Horizon::Unknown,
            needs_any_day: true,
            ..answers(dec!(50_000), dec!(0), 0)
        },
    ]
    .map(|start| {
        let answer = answered(&start, Choices::default());
        (start, Choices::default(), answer)
    });
    let mut seen = HashSet::new();
    for (start, choices, answer) in every_answer().into_iter().chain(extras) {
        if is_existing(&start) || !is_stated(&answer, LineKind::PlaceType) {
            continue;
        }
        let way = line(&answer, LineKind::Way).unwrap();
        let safe = match &way.state {
            LineState::Decision(decision) => decision.id == DecisionId::SafeWay,
            LineState::Stated { .. } => answer
                .focus
                .is_some_and(|focus| focus.arena() == Some(Arena::Safe)),
            LineState::Waiting => false,
        };
        if !safe {
            continue;
        }
        let stated = is_stated(&answer, LineKind::Way);
        assert_eq!(
            answer.confidence.state == ConfidenceState::A,
            stated,
            "{start:?} {choices:?}"
        );
        if let LineState::Stated { text, .. } = &way.state {
            assert_eq!(
                text.en.contains(": for "),
                !start.needs_any_day,
                "{start:?}: {}",
                text.en
            );
            // "The only safe way" only where the chart has no other.
            let others = answer
                .outcome
                .rows
                .iter()
                .filter(|row| row.kind.arena() == Some(Arena::Safe) && row.excluded.is_none())
                .count();
            assert_eq!(
                text.en.contains("the only safe way"),
                others == 1,
                "{start:?}: {others} safe rows: {}",
                text.en
            );
            seen.insert((start.needs_any_day, others == 1));
        }
    }
    assert_eq!(seen.len(), 3, "{seen:?}");
}

/// The rate a safe way's difference names, where the fund trades places.
fn switch_in(text: &str) -> Option<Decimal> {
    let at = ["averages above about ", "averages below about "]
        .iter()
        .find_map(|words| text.find(words).map(|at| at + words.len()))?;
    let number: String = text[at..].chars().take_while(|c| *c != '%').collect();
    number.parse().ok()
}

/// The switch point the safe ways' difference names is where the fund's
/// figure meets the way it's measured against: drawn through the fund's
/// rate bar (its figures at the rate ±0.5 point), the fund reaches the
/// other way's figure at about that rate. And only the fund's option says
/// how its sum moves with the rate, wherever its row draws the bar.
#[test]
fn the_switch_point_is_where_the_fund_meets_the_other_way() {
    let rate = settings().rate.0;
    let fund = Pick::SafeWay(SafeWay::MoneyMarketFund);
    let mut switches = 0;
    for (start, choices, answer) in every_answer() {
        let Some(way) = decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::SafeWay)
        else {
            continue;
        };
        let Some((low, high)) = answer.outcome.rows.iter().find_map(|row| match row.mark {
            Mark::RateRange { low, high } => Some((low.today, high.today)),
            _ => None,
        }) else {
            continue;
        };
        for option in &way.options {
            let says = option
                .facts
                .iter()
                .any(|fact| fact.en.contains("moves with the Bank of Israel's rate"));
            assert_eq!(says, option.pick == fund, "{start:?} {choices:?}");
        }
        let Some(switch) = way.difference.as_ref().and_then(|text| switch_in(&text.en)) else {
            continue;
        };
        let value = |pick: &Opt| pick.value.unwrap().today;
        let fund_now = value(
            way.options
                .iter()
                .find(|option| option.pick == fund)
                .unwrap(),
        );
        let other = value(
            way.options
                .iter()
                .find(|option| option.pick != fund)
                .unwrap(),
        );
        // A point of the rate is worth (high − low) to the fund.
        let expected = rate + (other - fund_now) / (high - low);
        assert!(
            (switch - expected).abs() <= dec!(0.15),
            "{start:?}: said {switch}%, the bar gives {expected}%"
        );
        switches += 1;
    }
    assert!(switches > 0);
}

/// "At a rate nobody knows" is the safe ways' reason only where the order
/// can flip: some sums for a few months, with no way inside the fund's rate
/// bar, are decided without it (the fund's own figure is always inside its
/// bar, so it never counts as a way that trades places with it).
#[test]
fn the_rate_decides_only_where_the_order_can_flip() {
    let fund = Pick::SafeWay(SafeWay::MoneyMarketFund);
    let mut without = 0;
    for now in [dec!(3000), dec!(10_000), dec!(30_000)] {
        for months in [7, 12] {
            let start = Answers {
                flexibility: Some(Flexibility::Fixed),
                cushion: Some(Cushion::Has),
                bank: None,
                ..answers(now, dec!(0), months)
            };
            let answer = answered(&start, Choices::default());
            let Some(way) =
                decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::SafeWay)
            else {
                continue;
            };
            let Some((low, high)) = answer.outcome.rows.iter().find_map(|row| match row.mark {
                Mark::RateRange { low, high } => Some((low.today, high.today)),
                _ => None,
            }) else {
                continue;
            };
            if !way.options.iter().any(|option| option.pick == fund) {
                continue;
            }
            let inside = way.options.iter().any(|option| {
                option.pick != fund
                    && option
                        .value
                        .is_some_and(|value| low <= value.today && value.today <= high)
            });
            let said = way.why.en.contains("nobody knows");
            assert!(!inside || said, "{start:?}: {}", way.why.en);
            without += usize::from(!said);
        }
    }
    assert!(without > 0);
}

/// The study fund's difference says it's "mostly from the tax exemption"
/// exactly when the tax it saves is more than any fees it adds, as the gain
/// section draws them for the two ways.
#[test]
fn the_study_fund_is_mostly_tax_when_the_tax_outweighs_the_fees() {
    let mut seen = HashSet::new();
    for (start, choices, answer) in every_answer() {
        let Some(wrapper) =
            decision(&answer, LineKind::Way).filter(|d| d.id == DecisionId::Wrapper)
        else {
            continue;
        };
        if decision(&answer, LineKind::Where).is_some() {
            continue;
        }
        let Some(gain) = &answer.gain else { continue };
        let [lead, study] = gain.rows.as_slice() else {
            panic!("{start:?}: the two ways")
        };
        let tax_saved = lead.tax.today - study.tax.today;
        let fees_added = study.fees.today - lead.fees.today;
        // The bars are rounded as the headline amounts are: skip a near tie.
        if (tax_saved - fees_added).abs() <= dec!(0.02) * tax_saved.abs().max(fees_added.abs()) {
            continue;
        }
        let said = wrapper
            .difference
            .as_ref()
            .is_some_and(|text| text.en.contains("mostly from the tax exemption"));
        assert_eq!(
            said,
            tax_saved > fees_added,
            "{start:?} {choices:?}: tax saved {tax_saved}, fees added {fees_added}"
        );
        seen.insert(said);
    }
    assert!(!seen.is_empty(), "{seen:?}");
}

/// A stated line's pill says what its words say: the index's way names the
/// way that leaves about the same exactly when it's a tie; a deposit's
/// place is a tie exactly when it names several banks; a fund company is
/// clear only as the visitor's own study fund, and otherwise the rules'
/// (an average fee).
#[test]
fn a_stated_lines_pill_is_what_its_words_say() {
    let mut seen = HashSet::new();
    for (start, choices, answer) in every_answer() {
        let ways = [
            "Index fund",
            "ETF",
            "Provident fund",
            "Study fund",
            "Savings policy",
        ];
        if let Some(LineState::Stated { text, pill }) =
            line(&answer, LineKind::Way).map(|line| &line.state)
            && ways.iter().any(|way| text.en.starts_with(way))
        {
            let tie = text.en.contains("About the same is left by");
            assert_eq!(
                tie,
                *pill == Pill::Tie,
                "{start:?} {choices:?}: {}",
                text.en
            );
            seen.insert(("way", tie));
        }
        let Some(LineState::Stated { text, pill }) =
            line(&answer, LineKind::Where).map(|line| &line.state)
        else {
            continue;
        };
        let focus = answer.focus;
        if focus == Some(Kind::FixedDeposit) {
            let tie = text.en.starts_with("A tie:");
            assert_eq!(
                *pill == Pill::Tie,
                tie,
                "{start:?} {choices:?}: {}",
                text.en
            );
            assert!(tie || *pill == Pill::Rules, "{start:?}: {pill:?}");
            seen.insert(("deposit", tie));
        }
        if text.en.contains("on a stock track") {
            let own = text.en.starts_with("In your study fund");
            assert_eq!(
                *pill,
                if own { Pill::Clear } else { Pill::Rules },
                "{start:?} {choices:?}: {}",
                text.en
            );
            seen.insert(("fund", own));
        }
    }
    assert_eq!(seen.len(), 6, "{seen:?}");
}

/// A deposit's place, from the visitor's bank: "At your bank, …: ₪N less"
/// is what the places list under it says the bank is behind the best.
#[test]
fn a_deposit_at_your_bank_is_behind_by_its_places() {
    let mut said = 0;
    let deposit = Choices {
        safe_way: Some(SafeWay::Deposit),
        ..Choices::default()
    };
    let mut all = every_answer();
    for (now, months) in [(dec!(200_000), 24), (dec!(100_000), 12), (dec!(50_000), 18)] {
        for bank in [Bank::Hapoalim, Bank::Leumi, Bank::Discount] {
            let start = Answers {
                flexibility: Some(Flexibility::Fixed),
                bank: Some(bank),
                ..answers(now, dec!(0), months)
            };
            let answer = answered(&start, deposit);
            all.push((start, deposit, answer));
        }
    }
    for (start, choices, answer) in all {
        let Some(LineState::Stated { text, .. }) =
            line(&answer, LineKind::Where).map(|line| &line.state)
        else {
            continue;
        };
        let Some(at) = text.en.find("At your bank, ") else {
            continue;
        };
        let behind = amounts_in(&text.en[at..])[0];
        let places = answer
            .how
            .iter()
            .find(|how| Some(how.row) == answer.focus)
            .and_then(|how| how.where_to_buy.as_ref())
            .expect("the deposit's places");
        let best = places.rows.iter().map(|row| row.left.today).max().unwrap();
        let yours = places.rows.iter().find(|row| row.yours).unwrap().left.today;
        assert!(
            (behind - (best - yours)).abs() <= Decimal::ONE,
            "{start:?} {choices:?}: said {behind}, the places {best} and {yours}"
        );
        said += 1;
    }
    assert!(said > 0);
}

/// A place the site states is "clear" only when one place leads: where
/// several are within the providers' margin of the best, it's a tie.
#[test]
fn a_clear_place_is_one_place() {
    let mut clear = 0;
    for (start, choices, answer) in every_answer() {
        let Some(LineState::Stated { text, pill }) =
            line(&answer, LineKind::Where).map(|line| &line.state)
        else {
            continue;
        };
        let Some(places) = answer
            .how
            .iter()
            .find(|how| Some(how.row) == answer.focus)
            .and_then(|how| how.where_to_buy.as_ref())
        else {
            continue;
        };
        if *pill != Pill::Clear || places.rows.is_empty() {
            continue;
        }
        let leading = places.rows.iter().filter(|row| row.tie == 0).count();
        assert_eq!(leading, 1, "{start:?} {choices:?}: {}", text.en);
        clear += 1;
    }
    assert!(clear > 0);
}

/// A securities account's stated place asks for a new account only where
/// the visitor's bank doesn't join it: a bank behind the best by no more
/// than the margin between kinds is as good as moving, so the steps start
/// from an account at a bank or a house, not from opening one.
#[test]
fn a_new_account_is_asked_only_where_your_bank_doesnt_join() {
    let mut seen = HashSet::new();
    for (start, choices, answer) in every_answer() {
        if is_existing(&start) || !is_stated(&answer, LineKind::Where) {
            continue;
        }
        let Some(Kind::Way(way)) = answer.focus else {
            continue;
        };
        if way.vehicle != broker_fees::Vehicle::Brokerage {
            continue;
        }
        let how = answer
            .how
            .iter()
            .find(|how| Some(how.row) == answer.focus)
            .unwrap();
        let places = how.where_to_buy.as_ref().unwrap();
        let best = places.rows.iter().map(|row| row.left.today).max().unwrap();
        let margin = kind_margin(answer.outcome.deposited.today);
        let joins = places
            .rows
            .iter()
            .any(|row| row.yours && row.tie != 0 && best - row.left.today <= margin);
        let opens = how.steps[0]
            .en
            .starts_with("Open an account at one of the places below");
        assert_eq!(opens, !joins, "{start:?} {choices:?}: {}", how.steps[0].en);
        seen.insert(joins);
    }
    assert_eq!(seen.len(), 2, "{seen:?}");
}

/// What moving a portfolio saves, while no bank is named, runs from the
/// dearest bank's branch plan to the cheapest's: the best place less the
/// banks' range, as the places under it draw them.
#[test]
fn moving_a_portfolio_saves_the_banks_range() {
    let mut said = 0;
    let all = every_answer().into_iter().chain(small_portfolios());
    for (start, choices, answer) in all {
        if !is_existing(&start) {
            continue;
        }
        let Some(holdings) = decision(&answer, LineKind::Way) else {
            continue;
        };
        let index_funds = &holdings.options[0];
        assert_eq!(index_funds.pick, Pick::Holdings(Holdings::IndexFunds));
        let amounts = amounts_in(&index_funds.amount.en);
        let lead = &answer.outcome.rows[0];
        let places = answer
            .how
            .iter()
            .find(|how| how.row == lead.id)
            .and_then(|how| how.where_to_buy.as_ref())
            .unwrap();
        let (Some((low, high)), false) =
            (places.bank_range, places.rows.iter().any(|row| row.yours))
        else {
            continue;
        };
        let [from, to] = amounts.as_slice() else {
            continue;
        };
        let best = places.rows[0].left.today;
        for (said, worked_out) in [(from, best - high.today), (to, best - low.today)] {
            assert!(
                (*said - worked_out).abs() <= dec!(0.01) * worked_out.abs() + dec!(10),
                "{start:?} {choices:?}: said {said}, the places give {worked_out}: {}",
                index_funds.amount.en
            );
        }
        said += 1;
    }
    assert!(said > 0);
}

/// How to do the index, with no cushion yet, starts by putting the cushion
/// aside in a money market fund; a safe way's steps never do (the money is
/// already safe), nor do the index's once the cushion is there.
#[test]
fn the_cushion_comes_first_only_in_the_indexs_steps() {
    let mut first = 0;
    let mut all = every_answer();
    for start in [
        salary(),
        answers(dec!(20_000), dec!(2000), 360),
        apartment(),
    ] {
        let start = Answers {
            cushion: Some(Cushion::HasNot {
                monthly_expenses: None,
            }),
            flexibility: Some(Flexibility::CanWait),
            ..start
        };
        for choices in every_choice() {
            let answer = answered(&start, choices);
            all.push((start.clone(), choices, answer));
        }
    }
    for (start, choices, answer) in all {
        let Some(how) = answer.how.iter().find(|how| Some(how.row) == answer.focus) else {
            continue;
        };
        let starts_with_cushion = how
            .steps
            .first()
            .is_some_and(|step| step.en.starts_with("First:"));
        // The plan says it as the steps do.
        if let Some(plan) = &answer.plan {
            assert_eq!(
                plan.en.starts_with("First "),
                starts_with_cushion,
                "{start:?} {choices:?}: {}",
                plan.en
            );
        }
        let index = how.row.arena() == Some(Arena::Index);
        let no_cushion = matches!(start.cushion, Some(Cushion::HasNot { .. }));
        if !index || !no_cushion {
            assert!(
                !starts_with_cushion,
                "{start:?} {choices:?}: {:?}",
                how.steps
            );
        } else if start.monthly > Decimal::ZERO && line(&answer, LineKind::Before).is_some() {
            assert!(
                starts_with_cushion,
                "{start:?} {choices:?}: {:?}",
                how.steps
            );
            first += 1;
        }
    }
    assert!(first > 0);
}

/// What to search for a money market fund names the funds that tie for the
/// most, each with its page as the source; no other row's search is about
/// a money market fund.
#[test]
fn a_money_market_funds_search_names_the_tied_funds() {
    let mut named = 0;
    for (start, choices, answer) in every_answer() {
        for how in &answer.how {
            let Some(search) = &how.search else { continue };
            if how.row == Kind::MoneyMarketFund {
                assert!(
                    !search.sources.is_empty(),
                    "{start:?} {choices:?}: {}",
                    search.text.en
                );
                named += 1;
            } else {
                assert!(
                    !search.text.en.contains("money market"),
                    "{start:?} {choices:?} {:?}: {}",
                    how.row,
                    search.text.en
                );
            }
        }
    }
    assert!(named > 0);
}

/// Until the visitor names a bank, the banks are one row, the reference
/// point, wherever a house takes the money too; but a deposit's places stay
/// the banks themselves, and a portfolio is measured against every bank's
/// branch price list, row by row.
#[test]
fn the_banks_are_one_row_except_for_deposits_and_portfolios() {
    let mut as_one = 0;
    for (start, choices, answer) in every_answer().into_iter().chain(small_portfolios()) {
        for how in &answer.how {
            let Some(places) = &how.where_to_buy else {
                continue;
            };
            let deposit = matches!(
                how.row,
                Kind::FixedDeposit | Kind::LinkedDeposit | Kind::AnyDayDeposit
            );
            if is_existing(&start) || deposit {
                assert!(
                    places.reference.is_none(),
                    "{start:?} {choices:?} {:?}",
                    how.row
                );
            } else if places.reference.is_some() {
                as_one += 1;
            }
        }
    }
    assert!(as_one > 0);
}

/// The gain's rows, while no bank is named, call only the reference "the
/// cheapest bank": the place that leaves most is named as itself. For a
/// portfolio, the reference is the cheapest branch price list.
#[test]
fn the_gain_calls_only_the_reference_the_cheapest_bank() {
    let mut seen = HashSet::new();
    for (start, choices, answer) in every_answer().into_iter().chain(small_portfolios()) {
        let Some(gain) = &answer.gain else { continue };
        let named = start.bank.is_some()
            || matches!(
                start.situation,
                Some(Situation::ExistingPortfolio { bank: Some(_) })
            );
        if named {
            continue;
        }
        let cheapest = |words: &str| {
            gain.rows
                .iter()
                .filter(|row| row.name.en.contains(words))
                .count()
        };
        let (bank, branch) = (
            cheapest("The cheapest bank"),
            cheapest("The cheapest branch"),
        );
        assert!(bank + branch <= 1, "{start:?} {choices:?}: {:?}", gain.rows);
        if is_existing(&start) {
            assert_eq!(bank, 0, "{start:?} {choices:?}: {:?}", gain.rows);
        }
        seen.insert((is_existing(&start), bank + branch));
    }
    assert!(
        seen.contains(&(true, 1)) && seen.contains(&(false, 1)),
        "{seen:?}"
    );
}
