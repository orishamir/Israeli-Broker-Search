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
    // Once the money goes to the index (the place type may be the
    // visitor's call first, by the data).
    let answer = answered(
        &Answers {
            flexibility: Some(Flexibility::CanWait),
            ..preset.answers
        },
        Choices::default(),
    );
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

/// Above the lines, where the rules put the money in the index, the data
/// decides how sure that is: the place type is the visitor's call (state C)
/// exactly when the index's worst past stretch, after fees and tax, ends
/// below the best safe way; otherwise it is stated, in state A.
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
        let best_safe = answer
            .outcome
            .rows
            .iter()
            .filter(|row| row.kind.arena() == Some(Arena::Safe))
            .map(|row| row.value.today)
            .max()
            .expect("a safe way is drawn");
        let asked = decision(&answer, LineKind::PlaceType).is_some();
        assert_eq!(
            asked,
            answer.confidence.state == ConfidenceState::C,
            "{answers:?}"
        );
        if asked {
            assert!(worst < best_safe, "{answers:?}");
        } else {
            assert!(worst >= best_safe, "{answers:?}");
            assert_eq!(answer.confidence.state, ConfidenceState::A, "{answers:?}");
            // The line says so in ₪, as the past: both figures, rounded.
            let Some(LineState::Stated { text, .. }) =
                line(&answer, LineKind::PlaceType).map(|line| &line.state)
            else {
                panic!("stated: {answers:?}");
            };
            for figure in [worst, best_safe] {
                let rounded = broker_fees::describe::card::about(figure);
                assert!(text.he.contains(&rounded), "{rounded} in {}", text.he);
            }
        }
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

/// The date `years` away as the page's years field sets it.
fn moved(answers: &Answers, row: &HorizonRow) -> Answers {
    Answers {
        horizon: match answers.horizon {
            Horizon::Retirement { age, .. } => Horizon::Retirement {
                age,
                retire_at: age + row.years,
            },
            Horizon::Months(_) | Horizon::Unknown => Horizon::Months(row.years * 12),
        },
        flexibility: row.flexibility.or(answers.flexibility),
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

/// §2.9's check: a horizon row is exactly what the answer shows once the
/// date moves there (and, for a row of one answer about the date, once
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
        assert!(!full.horizons.is_empty(), "{}", situation.id);
        for row in &full.horizons {
            let label = format!("{} at {} years", situation.id, row.years);
            assert_eq!(
                row.current,
                moved(&answers, row).horizon == answers.horizon,
                "{label}"
            );
            assert!(row.low.today <= row.high.today, "{label}");
            assert_eq!(
                row.condition.is_some(),
                row.flexibility.is_some(),
                "{label}"
            );
            let opened = answered(&moved(&answers, row), Choices::default());
            match opened.focus {
                Some(focus) => assert_eq!(row.kind, focus, "{label}"),
                None => assert!(
                    opened
                        .outcome
                        .rows
                        .iter()
                        .any(|shown| shown.kind == row.kind && shown.active),
                    "{label}: {:?}",
                    row.kind
                ),
            }
            let drawn: Vec<(Amount, Amount)> = opened.outcome.rows.iter().map(ends).collect();
            assert!(drawn.iter().any(|(low, _)| *low == row.low), "{label}: low");
            assert!(
                drawn.iter().any(|(_, high)| *high == row.high),
                "{label}: high"
            );
        }
    }
}

/// Where the visitor hasn't said whether the date can move, a date where it
/// decides the place has a row for each answer; once said, one row a date.
#[test]
fn a_date_where_flexibility_decides_has_both_rows_until_answered() {
    let full =
        |answers: &Answers| answer(answers, &Choices::default(), &settings(), &DATA).unwrap();
    let open = full(&bonus());
    for years in LADDER_YEARS {
        let rows = open
            .horizons
            .iter()
            .filter(|row| row.years == years)
            .count();
        let at = answered(
            &Answers {
                horizon: Horizon::Months(years * 12),
                ..bonus()
            },
            Choices::default(),
        );
        let decides = decision(&at, LineKind::PlaceType).is_some();
        assert_eq!(rows, if decides { 2 } else { 1 }, "{years} years");
    }
    let can_wait = full(&Answers {
        flexibility: Some(Flexibility::CanWait),
        ..bonus()
    });
    assert_eq!(can_wait.horizons.len(), LADDER_YEARS.len());
    assert!(
        can_wait
            .horizons
            .iter()
            .all(|row| row.flexibility.is_none())
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
            shown.extend([answer.outcome.deposited.today, answer.outcome.current_account.today]);
            for amount in body {
                assert!(
                    shown.iter().all(|other| (*other - amount).abs() > amount * dec!(0.01)),
                    "the body repeats {amount}: {}",
                    answer.body.he
                );
            }
        }
    }
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

/// For retirement the answer gives the monthly pension as a range with whose
/// coefficients it rests on, once; and the pension from work is said in the
/// body, not asked.
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
    for answers in [&preset.answers, &growth] {
        let answer = answered(answers, Choices::default());
        let annuity = answer.annuity.as_ref().expect("a monthly pension");
        assert!(!annuity.he.contains("[₪—]"), "{}", annuity.he);
        assert_eq!(amounts_in(&annuity.he).len(), 2, "{}", annuity.he);
        assert!(annuity.he.contains("מקדמי ההמרה"), "{}", annuity.he);
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
                        assert!(!search.sources.is_empty());
                    }
                    Kind::MoneyMarketFund => {
                        assert!(he.starts_with("כספית שקלית"), "{he}");
                        assert!(!he.contains("הזול"), "{he}");
                    }
                    Kind::Way(way) if way.vehicle == broker_fees::Vehicle::Brokerage => {
                        assert!(he.contains(broker_fees::examples::NOT_A_PICK.he.as_ref()), "{he}");
                        assert!(!search.sources.is_empty());
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
                let Some(places) = &how.where_to_buy else { continue };
                for row in &places.rows {
                    let name = row.name.he.as_ref();
                    assert_ne!(name, average);
                    let provider = name.split("\u{a0}· ").next().unwrap();
                    match how.row {
                        Kind::FixedDeposit | Kind::LinkedDeposit => {
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
/// no bank named, the dearest bank, said as such). Never more than three.
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
            None => assert!(bank_row.contains("היקר"), "{bank_row}"),
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
