//! The guide (`guide.rs`): one test for each row of REDESIGN.md §3's table,
//! the follow-up questions, the cushion's arithmetic, the ladder, the
//! situations, and the questions of §8 as the core answers them. The numbers
//! come from the core; the tests state the rule.

use std::sync::LazyLock;

use broker_fees::guide::*;
use broker_fees::simulation::{Scenario, free_plan, simulate};
use broker_fees::situations::situations;
use broker_fees::{
    Exchange, ExchangeRates, Lang, Named, Percent, Product, Security, Vehicle, Withdrawal,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

static DATA: LazyLock<Data> = LazyLock::new(Data::listed);

/// €1 = $1.25 = ₪4.625, so $1 = ₪3.70. Not real rates.
fn rates() -> ExchangeRates {
    ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap()
}

fn settings() -> Settings {
    Settings::new(rates())
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

fn years(years: u32) -> Horizon {
    Horizon::Months(years * 12)
}

fn fixed(mut answers: Answers) -> Answers {
    answers.flexibility = Some(Flexibility::Fixed);
    answers
}

fn can_wait(mut answers: Answers) -> Answers {
    answers.flexibility = Some(Flexibility::CanWait);
    answers
}

fn recommended(answers: &Answers) -> Recommendation {
    recommend(answers, &settings(), &DATA).unwrap()
}

/// Whether some reason names a circumstance of the wanted kind.
fn names(rec: &Recommendation, wanted: fn(&Circumstance) -> bool) -> bool {
    rec.reasons.iter().any(|reason| wanted(&reason.from))
}

fn way(kind: Kind) -> broker_fees::ways::Way {
    match kind {
        Kind::Way(way) => way,
        other => panic!("not a way: {other:?}"),
    }
}

fn is_safe_kind(kind: Kind) -> bool {
    kind.arena() == Some(Arena::Safe)
}

fn is_study_fund(kind: Kind) -> bool {
    matches!(kind, Kind::Way(way) if way.vehicle == Vehicle::StudyFund)
}

/// A kind of the other arena is drawn beside every answer, and so is the
/// current account.
fn assert_draws_the_other_arena(rec: &Recommendation) {
    assert!(
        rec.alternatives
            .iter()
            .any(|alternative| alternative.kind == Kind::CurrentAccount),
        "the current account is the yardstick"
    );
    let other = match rec.arena {
        Arena::Safe => Arena::Index,
        Arena::Index => Arena::Safe,
    };
    assert!(
        rec.alternatives
            .iter()
            .any(|alternative| alternative.kind.arena() == Some(other)),
        "{:?} should be drawn beside {:?}",
        other,
        rec.arena
    );
}

// ─────────────────────────── The rows of §3 ───────────────────────────

/// Row 1: money that may be needed any day goes where it is paid out the
/// next day: the money market fund, even where a deposit leaves more.
#[test]
fn row_1_money_needed_any_day_is_in_a_money_market_fund() {
    let mut cushion = answers(dec!(30_000), dec!(0), Horizon::Months(6));
    cushion.needs_any_day = true;
    let rec = recommended(&cushion);
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
    assert!(names(&rec, |c| matches!(c, Circumstance::NeedsAnyDay)));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::EarnsBankRate { .. }
    )));
    // The deposit is drawn, and it is locked.
    let deposit = rec
        .alternatives
        .iter()
        .find(|alternative| alternative.kind == Kind::FixedDeposit)
        .expect("the deposit is drawn");
    assert!(matches!(
        deposit.why_not.from,
        Circumstance::DepositLocked { .. }
    ));
    assert_draws_the_other_arena(&rec);
}

/// Row 1: a year or less, with no need to reach it any day: whichever of
/// the money market fund and a deposit leaves more.
#[test]
fn row_1_a_year_or_less_takes_the_better_of_the_fund_and_a_deposit() {
    let sum = answers(dec!(15_000), dec!(0), Horizon::Months(12));
    let rec = recommended(&sum);
    assert_eq!(rec.arena, Arena::Safe);
    assert!(matches!(
        rec.kind,
        Kind::MoneyMarketFund | Kind::FixedDeposit
    ));
    let best = rec
        .ranked
        .iter()
        .filter(|line| matches!(line.kind, Kind::MoneyMarketFund | Kind::FixedDeposit))
        .max_by_key(|line| line.left.nominal.round_dp(0))
        .unwrap();
    assert_eq!(rec.left.nominal.round_dp(0), best.left.nominal.round_dp(0));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::ShortHorizon { months: 12 }
    )));
    // Monthly deposits: a deposit takes one sum, so the fund it is.
    let monthly = recommended(&answers(dec!(0), dec!(2000), Horizon::Months(12)));
    assert_eq!(monthly.kind, Kind::MoneyMarketFund);
    // And "any day" makes it the fund however the numbers fall.
    let mut any_day = sum;
    any_day.needs_any_day = true;
    assert_eq!(recommended(&any_day).kind, Kind::MoneyMarketFund);
}

/// Row 2: 13 to 36 months on a known date: safe, whichever of the fixed
/// deposit, the fund and the bonds leaves more after tax; a stock index
/// fund is drawn with its band.
#[test]
fn row_2_a_known_date_within_three_years_is_safe() {
    let rec = recommended(&answers(dec!(50_000), dec!(0), Horizon::Months(24)));
    assert_eq!(rec.arena, Arena::Safe);
    assert!(is_safe_kind(rec.kind));
    assert_eq!(
        rec.ranked.first().map(|line| line.kind),
        Some(rec.kind),
        "the answer leads the ranking"
    );
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::KnownDate { months: 24 }
    )));
    assert!(names(&rec, |c| matches!(c, Circumstance::OneSum)));
    let kinds: Vec<Kind> = rec.ranked.iter().map(|line| line.kind).collect();
    assert!(kinds.contains(&Kind::FixedDeposit));
    assert!(kinds.contains(&Kind::MoneyMarketFund));
    assert!(kinds.contains(&Kind::GovernmentBond { linked: false }));
    let stock = rec
        .alternatives
        .iter()
        .find(|alternative| matches!(alternative.kind, Kind::Way(_)))
        .expect("a stock index fund is drawn");
    assert!(stock.band.is_some());
    assert_draws_the_other_arena(&rec);
}

/// Row 3: 37 to 84 months with the date fixed: only the kinds whose sum is
/// known in advance are ranked; the fund is an alternative whose rate isn't
/// promised.
#[test]
fn row_3_a_fixed_date_beyond_three_years_takes_a_sum_known_in_advance() {
    let rec = recommended(&fixed(answers(dec!(50_000), dec!(0), years(5))));
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.kind.safety(), Safety::SumKnown);
    assert!(
        rec.ranked
            .iter()
            .all(|line| line.kind.safety() == Safety::SumKnown)
    );
    assert!(
        rec.ranked
            .iter()
            .all(|line| line.kind != Kind::GovernmentBond { linked: true }),
        "a linked bond's sum in shekels is not known in advance"
    );
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::FixedDate { months: 60 }
    )));
    assert!(names(&rec, |c| matches!(c, Circumstance::OneSum)));
    let fund = rec
        .alternatives
        .iter()
        .find(|alternative| alternative.kind == Kind::MoneyMarketFund)
        .expect("the fund is an alternative");
    assert!(matches!(
        fund.why_not.from,
        Circumstance::RateNotPromised { .. }
    ));
    let stock = rec
        .alternatives
        .iter()
        .find(|alternative| matches!(alternative.kind, Kind::Way(_)))
        .expect("a stock index fund is drawn");
    let band = stock.band.expect("with the band of five years");
    assert_eq!(band.years, dec!(5));
    assert!(
        band.lower > 0 && band.lower < band.windows,
        "N of M periods"
    );
    assert_draws_the_other_arena(&rec);
}

/// Row 3 at its edge: seven years fixed is still safe, and a month more is
/// not.
#[test]
fn row_3_ends_at_seven_years() {
    let safe = recommended(&fixed(answers(dec!(50_000), dec!(0), years(7))));
    assert_eq!(safe.arena, Arena::Safe);
    let index = recommended(&fixed(answers(dec!(50_000), dec!(0), Horizon::Months(85))));
    assert_eq!(index.arena, Arena::Index);
}

/// Row 4: 37 to 59 months with a date that can move: safe as when fixed,
/// and the reason says that from five years stocks become reasonable.
#[test]
fn row_4_a_date_that_can_move_below_five_years_is_still_safe() {
    let rec = recommended(&can_wait(answers(dec!(50_000), dec!(0), years(4))));
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.kind.safety(), Safety::SumKnown);
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::FlexibleDate { months: 48 }
    )));
    assert_draws_the_other_arena(&rec);
}

/// Row 5: 60 to 84 months with a date that can move: the index kinds,
/// "if it's down then, you wait", the history, and the safe kinds drawn.
#[test]
fn row_5_five_years_that_can_move_is_the_index_arena() {
    let rec = recommended(&can_wait(answers(dec!(50_000), dec!(0), years(5))));
    assert_eq!(rec.arena, Arena::Index);
    assert!(matches!(rec.kind, Kind::Way(_)));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::IfItsDownYouWait { months: 60 }
    )));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::HistoryOfTheHorizon { .. }
    )));
    assert!(names(&rec, |c| matches!(c, Circumstance::WhatSafetyCosts)));
    let safe: Vec<&Alternative> = rec
        .alternatives
        .iter()
        .filter(|alternative| is_safe_kind(alternative.kind))
        .collect();
    assert!(safe.iter().any(|a| a.kind == Kind::MoneyMarketFund));
    assert!(safe.iter().all(|a| a.band.is_some()));
    assert_draws_the_other_arena(&rec);
}

/// Row 6: seven years and more: the index kinds, ranked by what is left
/// after tax, and the cheapest way is named with its yearly cost.
#[test]
fn row_6_long_horizons_rank_the_ways_after_tax() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(17)));
    assert_eq!(rec.arena, Arena::Index);
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::LongHorizon { months: 204 }
    )));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::CheapestAfterTax { .. }
    )));
    let lefts: Vec<Decimal> = rec.ranked.iter().map(|line| line.left.nominal).collect();
    assert!(lefts.windows(2).all(|pair| pair[0] >= pair[1]));
    assert!(rec.ranked.len() >= 5, "every way that can be opened");
    assert_draws_the_other_arena(&rec);
}

/// Row 7: retirement: the provident fund for investment as a pension when
/// the saver is 60 at the end, the tax-free pension against 25%, the
/// ceiling; the study fund drawn with who can open one.
#[test]
fn row_7_retirement_takes_the_provident_fund_as_a_pension() {
    let retire = answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 60,
        },
    );
    let rec = recommended(&retire);
    assert_eq!(rec.arena, Arena::Index);
    assert_eq!(way(rec.kind).vehicle, Vehicle::InvestmentGemel);
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::TaxFreePension { age_at_the_end: 60 }
    )));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::DepositCeiling { .. }
    )));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::Retirement {
            age: 40,
            retire_at: 60
        }
    )));
    let vehicles: Vec<Vehicle> = rec
        .alternatives
        .iter()
        .filter_map(|alternative| match alternative.kind {
            Kind::Way(way) => Some(way.vehicle),
            _ => None,
        })
        .collect();
    for wanted in [
        Vehicle::Brokerage,
        Vehicle::StudyFund,
        Vehicle::SavingsPolicy,
    ] {
        assert!(vehicles.contains(&wanted), "{wanted:?} is drawn");
    }
    let study = rec
        .alternatives
        .iter()
        .find(|alternative| is_study_fund(alternative.kind))
        .unwrap();
    assert_eq!(study.why_not.from, Circumstance::StudyFundNeedsEligibility);
    assert_draws_the_other_arena(&rec);

    // Who retires before 60 can't take the tax-free pension yet, and the
    // answer says so instead.
    let early = recommended(&answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 55,
        },
    ));
    assert!(names(&early, |c| matches!(
        c,
        Circumstance::PensionNeedsSixty { age_at_the_end: 55 }
    )));
    assert!(!names(&early, |c| matches!(
        c,
        Circumstance::TaxFreePension { .. }
    )));
}

/// The study fund can be ahead of the answer in the ranking, but it isn't
/// named as the answer: not everyone can open one.
#[test]
fn the_study_fund_is_never_the_answer_unless_chosen() {
    let sum = answers(dec!(15_000), dec!(0), years(20));
    let rec = recommended(&sum);
    assert!(!is_study_fund(rec.kind));
    let study = rec
        .ranked
        .iter()
        .find(|line| is_study_fund(line.kind))
        .expect("it is ranked");
    assert!(study.left.nominal >= rec.left.nominal, "and leaves more");
    assert!(
        rec.alternatives
            .iter()
            .any(|a| a.why_not.from == Circumstance::StudyFundNeedsEligibility)
    );
    // The expert can choose it.
    let mut chosen = settings();
    chosen.overrides.kind = Some(study.kind);
    let rec = recommend(&sum, &chosen, &DATA).unwrap();
    assert!(is_study_fund(rec.kind));
    assert!(names(&rec, |c| matches!(c, Circumstance::ChosenByYou)));
}

/// Row 8: no date, money every month, no cushion: the cushion first, in a
/// money market fund; then an index kind that can be opened any time.
#[test]
fn row_8_no_date_and_no_cushion_is_a_two_part_answer() {
    let mut unknown = answers(dec!(0), dec!(3000), Horizon::Unknown);
    unknown.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(17_000),
    });
    let rec = recommended(&unknown);
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
    assert!(names(&rec, |c| matches!(c, Circumstance::NoCushion { .. })));
    let plan = rec.two_parts.as_ref().expect("two parts");
    assert_eq!(plan.then, plan.then_ranked[0].kind);
    let then = way(plan.then);
    assert!(
        matches!(then.vehicle, Vehicle::Brokerage | Vehicle::InvestmentGemel),
        "a place that can be opened any time"
    );
    assert_draws_the_other_arena(&rec);
}

/// Row 8: with a cushion, the same money goes to an index kind that can be
/// opened any time, and the answer is not in two parts.
#[test]
fn row_8_with_a_cushion_the_money_can_grow() {
    let mut unknown = answers(dec!(0), dec!(3000), Horizon::Unknown);
    unknown.cushion = Some(Cushion::Has);
    let rec = recommended(&unknown);
    assert_eq!(rec.arena, Arena::Index);
    assert!(matches!(
        way(rec.kind).vehicle,
        Vehicle::Brokerage | Vehicle::InvestmentGemel
    ));
    assert!(rec.two_parts.is_none());
    assert!(names(&rec, |c| matches!(c, Circumstance::HasCushion)));
    assert!(names(&rec, |c| matches!(c, Circumstance::NoDate)));
    assert_draws_the_other_arena(&rec);
}

/// Row 9: no date, one sum: the sum waits where it can be reached any day,
/// and the ladder says what the date would change.
#[test]
fn row_9_no_date_and_one_sum_waits_in_the_fund_and_has_a_ladder() {
    let bonus = answers(dec!(15_000), dec!(0), Horizon::Unknown);
    let rec = recommended(&bonus);
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
    assert!(names(&rec, |c| matches!(c, Circumstance::NoDate)));
    assert!(names(&rec, |c| matches!(c, Circumstance::OneSum)));
    assert_draws_the_other_arena(&rec);
    let rungs = ladder(&bonus, &settings(), &DATA).unwrap();
    assert_eq!(rungs.len(), 6, "any day, then 1, 3, 5, 10 and 20 years");
    assert!(rungs[0].any_day);
}

/// Row 10: a portfolio the saver already has: the bank's branch plan is
/// compared with the rest, and marked as theirs.
#[test]
fn row_10_a_portfolio_at_a_bank_compares_its_branch_plan() {
    let existing = |bank| Answers {
        situation: Some(Situation::ExistingPortfolio { bank }),
        ..answers(dec!(200_000), dec!(0), years(20))
    };
    let yours = |rec: &Recommendation| -> Option<Decimal> {
        rec.plans
            .iter()
            .find(|plan| plan.yours)
            .map(|plan| plan.left.nominal)
    };
    let leumi = recommended(&existing(Some(Bank::Leumi)));
    assert!(names(&leumi, |c| matches!(
        c,
        Circumstance::ExistingPortfolio
    )));
    let own = leumi.plans.iter().find(|plan| plan.yours).expect("ranked");
    assert!(own.name.en.starts_with("Leumi"));
    assert!(own.name.en.contains("Branch"));
    // Without the saver's bank, the branch plan that leaves the least stands
    // in for it: not cheaper than any bank's could be shown to be.
    let per_bank: Vec<Decimal> = [Bank::Leumi, Bank::MizrahiTefahot, Bank::OtsarHahayal]
        .into_iter()
        .map(|bank| yours(&recommended(&existing(Some(bank)))).expect("ranked"))
        .collect();
    let stand_in = yours(&recommended(&existing(None))).expect("ranked");
    assert_eq!(stand_in, per_bank.into_iter().min().unwrap());
}

/// Row 11: "I know what I want to buy": the index arena with every way, even
/// for a horizon the rules would call safe.
#[test]
fn row_11_knowing_what_to_buy_opens_every_way() {
    let know = Answers {
        situation: Some(Situation::KnowsWhatToBuy),
        ..answers(dec!(10_000), dec!(2000), Horizon::Months(24))
    };
    let rec = recommended(&know);
    assert_eq!(rec.arena, Arena::Index);
    assert!(names(&rec, |c| matches!(c, Circumstance::KnowsWhatToBuy)));
    assert!(rec.ranked.len() >= 5);
}

/// The expert's choice of arena or kind wins over the rules, and says so.
#[test]
fn the_expert_can_choose_the_arena_and_the_kind() {
    let sum = fixed(answers(dec!(50_000), dec!(0), years(5)));
    let mut settings = settings();
    settings.overrides.arena = Some(Arena::Index);
    let rec = recommend(&sum, &settings, &DATA).unwrap();
    assert_eq!(rec.arena, Arena::Index);
    assert!(names(&rec, |c| matches!(c, Circumstance::ChosenByYou)));
    settings.overrides = Overrides {
        arena: None,
        kind: Some(Kind::MoneyMarketFund),
    };
    let rec = recommend(&sum, &settings, &DATA).unwrap();
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
}

// ─────────────────────────── The questions ───────────────────────────

#[test]
fn follow_ups_are_asked_only_when_the_answer_needs_them() {
    let nothing: [FollowUp; 0] = [];
    // Flexibility: more than 3 years, up to 7.
    for months in [12, 36] {
        assert_eq!(
            follow_ups(&answers(dec!(1), dec!(0), Horizon::Months(months))),
            nothing
        );
    }
    for months in [37, 60, 84] {
        assert_eq!(
            follow_ups(&answers(dec!(1), dec!(0), Horizon::Months(months))),
            [FollowUp::Flexibility]
        );
    }
    assert_eq!(
        follow_ups(&answers(dec!(1), dec!(0), Horizon::Months(85))),
        nothing
    );
    // Ages: for retirement.
    assert_eq!(
        follow_ups(&answers(
            dec!(1),
            dec!(0),
            Horizon::Retirement {
                age: 40,
                retire_at: 65
            }
        )),
        [FollowUp::AgeAndRetirement]
    );
    // The cushion: no date and money every month; its expenses once there is none.
    assert_eq!(
        follow_ups(&answers(dec!(1), dec!(0), Horizon::Unknown)),
        nothing
    );
    let mut unknown = answers(dec!(0), dec!(3000), Horizon::Unknown);
    assert_eq!(follow_ups(&unknown), [FollowUp::Cushion]);
    unknown.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(17_000),
    });
    assert_eq!(
        follow_ups(&unknown),
        [FollowUp::Cushion, FollowUp::MonthlyExpenses]
    );
    unknown.cushion = Some(Cushion::Has);
    assert_eq!(follow_ups(&unknown), [FollowUp::Cushion]);
    // They come with the answer too.
    let rec = recommended(&answers(dec!(1000), dec!(0), Horizon::Months(60)));
    assert_eq!(rec.follow_ups, [FollowUp::Flexibility]);
}

/// Three months of expenses is the target, and the months to fill it are
/// the target (less what is there) over the month's saving, rounded up.
#[test]
fn the_cushion_is_three_months_of_expenses_filled_by_saving() {
    let cushion = |now, monthly, expenses| {
        let mut answers = answers(now, monthly, Horizon::Unknown);
        answers.cushion = Some(Cushion::HasNot {
            monthly_expenses: expenses,
        });
        recommended(&answers).two_parts.expect("two parts")
    };
    let plan = cushion(dec!(0), dec!(3000), dec!(17_000));
    assert_eq!(plan.target, dec!(51_000));
    assert_eq!(plan.months_to_fill, 17);
    // What is already put aside shortens it: (51,000 − 6,000) / 3,000.
    assert_eq!(
        cushion(dec!(6000), dec!(3000), dec!(17_000)).months_to_fill,
        15
    );
    // A part of a month is a month: 51,000 / 5,000 = 10.2.
    assert_eq!(
        cushion(dec!(0), dec!(5000), dec!(17_000)).months_to_fill,
        11
    );
    // A cushion already full needs no first part.
    let mut full = answers(dec!(60_000), dec!(3000), Horizon::Unknown);
    full.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(17_000),
    });
    let rec = recommended(&full);
    assert!(rec.two_parts.is_none());
    assert_eq!(rec.arena, Arena::Index);
}

#[test]
fn impossible_answers_say_why() {
    let bad = |answers: &Answers| recommend(answers, &settings(), &DATA).unwrap_err();
    assert_eq!(
        bad(&answers(dec!(1), dec!(0), Horizon::Months(0))),
        InvalidAnswers::NoHorizon
    );
    assert_eq!(
        bad(&answers(dec!(0), dec!(0), years(5))),
        InvalidAnswers::NoMoney
    );
    assert_eq!(
        bad(&answers(dec!(-1), dec!(0), years(5))),
        InvalidAnswers::NegativeAmount
    );
    assert_eq!(
        bad(&answers(dec!(1), dec!(0), Horizon::Months(601))),
        InvalidAnswers::HorizonTooLong
    );
    assert_eq!(
        bad(&answers(
            dec!(1),
            dec!(0),
            Horizon::Retirement {
                age: 60,
                retire_at: 60
            }
        )),
        InvalidAnswers::RetirementNotAhead
    );
    assert_eq!(
        bad(&answers(dec!(1e13), dec!(0), years(5))),
        InvalidAnswers::TooLarge
    );
    let mut expenses = answers(dec!(0), dec!(3000), Horizon::Unknown);
    expenses.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(0),
    });
    assert_eq!(bad(&expenses), InvalidAnswers::NoExpenses);
    for why in [
        InvalidAnswers::NoHorizon,
        InvalidAnswers::NoMoney,
        InvalidAnswers::NegativeAmount,
        InvalidAnswers::HorizonTooLong,
        InvalidAnswers::RetirementNotAhead,
        InvalidAnswers::NoExpenses,
        InvalidAnswers::TooLarge,
    ] {
        assert!(!why.text(Lang::En).is_empty() && !why.text(Lang::He).is_empty());
    }
}

// ─────────────────────────── The ladder ───────────────────────────

/// The rungs go 1, 3, 5, 10 and 20 years; with a sum and nothing monthly
/// "any day" comes first. Within an arena, more years leave more.
#[test]
fn the_ladder_has_five_rungs_and_grows_with_the_horizon() {
    let monthly = answers(dec!(0), dec!(3000), Horizon::Unknown);
    let sum = answers(dec!(15_000), dec!(0), Horizon::Unknown);

    let rungs = ladder(&monthly, &settings(), &DATA).unwrap();
    let months: Vec<u32> = rungs.iter().map(|rung| rung.months).collect();
    assert_eq!(months, [12, 36, 60, 120, 240]);
    assert!(rungs.iter().all(|rung| !rung.any_day));

    let with_a_sum = ladder(&sum, &settings(), &DATA).unwrap();
    assert_eq!(with_a_sum.len(), 6);
    assert!(with_a_sum[0].any_day);
    assert_eq!(with_a_sum[0].kind, Kind::MoneyMarketFund);

    for rungs in [rungs, with_a_sum] {
        // Not the "any day" rung, which shares its year with the next.
        let by_horizon: Vec<&Rung> = rungs.iter().filter(|rung| !rung.any_day).collect();
        for pair in by_horizon.windows(2) {
            if pair[0].arena == pair[1].arena {
                assert!(
                    pair[1].left.today >= pair[0].left.today,
                    "{} months leaves less than {}",
                    pair[1].months,
                    pair[0].months
                );
            }
        }
        // Short money is safe, long money is the index's, and it only
        // changes once.
        let arenas: Vec<Arena> = by_horizon.iter().map(|rung| rung.arena).collect();
        assert_eq!(arenas.first(), Some(&Arena::Safe));
        assert_eq!(arenas.last(), Some(&Arena::Index));
        assert!(arenas.windows(2).filter(|pair| pair[0] != pair[1]).count() <= 1);
    }
}

// ─────────────────────────── The answer's figures ───────────────────────────

/// Every answer, over a grid of horizons: the other arena and the current
/// account are drawn, the answer leads its arena's ranking, the reasons
/// have words in both languages, and the figures are in today's money.
#[test]
fn every_answer_is_drawn_beside_the_other_arena_with_reasons() {
    let mut grid: Vec<Answers> = Vec::new();
    for months in [
        1, 6, 12, 13, 24, 36, 37, 48, 59, 60, 72, 84, 85, 120, 240, 360,
    ] {
        for flexibility in [None, Some(Flexibility::Fixed), Some(Flexibility::CanWait)] {
            grid.push(Answers {
                flexibility,
                ..answers(dec!(20_000), dec!(0), Horizon::Months(months))
            });
        }
    }
    grid.push(answers(dec!(0), dec!(1500), Horizon::Months(120)));
    grid.push(answers(dec!(15_000), dec!(500), Horizon::Unknown));
    grid.push(answers(dec!(15_000), dec!(0), Horizon::Unknown));
    for answers in &grid {
        let rec = recommended(answers);
        assert_draws_the_other_arena(&rec);
        assert!(!rec.reasons.is_empty(), "{answers:?}");
        for reason in &rec.reasons {
            assert!(!reason.text.en.is_empty() && !reason.text.he.is_empty());
        }
        for alternative in &rec.alternatives {
            let why = &alternative.why_not.text;
            assert!(!why.en.is_empty() && !why.he.is_empty());
            assert_eq!(alternative.arena, alternative.kind.arena());
        }
        assert_ne!(rec.steps.len(), 0);
        // The answer leads its ranking, but for the study fund, which is
        // drawn with who can open one.
        let first_open = rec
            .ranked
            .iter()
            .find(|line| !is_study_fund(line.kind))
            .map(|line| line.kind);
        assert_eq!(first_open, Some(rec.kind), "{answers:?}");
        // Today's money is never more than the nominal, with prices rising.
        assert!(rec.left.today <= rec.left.nominal);
        assert!(rec.current_account.today < rec.current_account.nominal);
        assert_eq!(rec.current_account.nominal, rec.put_in.nominal);
    }
}

/// The current account is the deposits themselves, in today's money; the
/// answer's today's money is the simulation's own (`in_todays_money`).
#[test]
fn the_figures_are_in_todays_money_as_the_simulation_states_them() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    let inflation = Percent(dec!(2));
    let scenario = Scenario {
        security: Security::IndexFund,
        exchange: Exchange::Tlv,
        first_deposit: dec!(10_000),
        monthly_deposit: dec!(2000),
        deposit_growth: Percent(dec!(0)),
        yearly_return: Percent(dec!(10)),
        product: None,
        years: 20,
        buy_every_months: 1,
        share_price: dec!(500),
        sell_at_end: true,
        inflation,
        age: 30,
        withdrawal: Withdrawal::LumpSum,
        redeems_at_maturity: false,
    };
    let near = |a: Decimal, b: Decimal| (a - b).abs() < dec!(0.01);
    // What the deposits are worth, each at its own month's prices.
    assert!(near(
        rec.put_in.today,
        scenario.deposited_in_todays_money(inflation)
    ));
    assert_eq!(rec.put_in.nominal, scenario.deposited());
    // The prices at the end, as the simulation divides by them.
    let outcome = simulate(&free_plan(), &scenario, &rates()).unwrap();
    let level = outcome.after_selling / outcome.in_todays_money(inflation).after_selling;
    assert!(near(
        rec.current_account.today,
        scenario.deposited() / level
    ));
    assert!(near(rec.left.today, rec.left.nominal / level));
}

/// The breakdown names where the money went: what was left, the fees and
/// the tax.
#[test]
fn the_breakdown_names_where_the_money_went() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    let split = &rec.breakdown;
    assert_eq!(split.left, rec.left.today);
    assert!(split.fees > Decimal::ZERO && split.tax > Decimal::ZERO);
    assert!(split.left + split.fees + split.tax > split.put_in);
}

/// The product's own cost is a slice of its own, and every alternative
/// that has a figure carries the breakdown that ends at it (the current
/// account's is only a line).
#[test]
fn the_products_cost_is_apart_and_alternatives_carry_breakdowns() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    assert!(rec.breakdown.product > Decimal::ZERO, "an index fund costs");
    let mut with_breakdown = 0;
    for alternative in &rec.alternatives {
        match (&alternative.breakdown, alternative.left) {
            (Some(split), Some(left)) => {
                assert!(near(split.left, left.today), "{:?}", alternative.kind);
                with_breakdown += 1;
            }
            (None, _) => assert!(
                alternative.kind == Kind::CurrentAccount || alternative.left.is_none(),
                "{:?}",
                alternative.kind
            ),
            (Some(_), None) => panic!("a breakdown without a figure"),
        }
    }
    assert!(with_breakdown > 0);
}

// ─────────────────────────── §8: the user's questions ───────────────────────────

/// a1: ₪50,000 now, 5 years, fixed: a government bond or the 5-year
/// deposit, whichever leaves more, with the brokers ranked on bonds. With
/// the data as it is the answer is the unlinked bond or the deposit; the
/// test states the rule, not which of them the data favours.
#[test]
fn a1_fixed_five_years_is_a_bond_or_a_deposit() {
    let rec = recommended(&fixed(answers(dec!(50_000), dec!(0), years(5))));
    assert_eq!(rec.arena, Arena::Safe);
    assert!(matches!(
        rec.kind,
        Kind::GovernmentBond { .. } | Kind::FixedDeposit
    ));
    let best = rec
        .ranked
        .iter()
        .max_by_key(|line| line.left.nominal.round_dp(0))
        .unwrap();
    assert_eq!(best.kind, rec.kind);
    assert!(names(&rec, |c| matches!(c, Circumstance::FixedDate { .. })));
    assert!(names(&rec, |c| matches!(c, Circumstance::OneSum)));
    if matches!(rec.kind, Kind::GovernmentBond { .. }) {
        assert!(rec.plans.len() >= 2, "the brokers ranked on bonds");
    }
    assert_ne!(rec.steps.len(), 0);
    assert!(rec.breakdown.tax > Decimal::ZERO);
}

/// a2: the same with a date that can move: the index arena, an index fund
/// in Tel Aviv, the five-year history, the safe kinds drawn.
#[test]
fn a2_five_years_that_can_move_is_an_index_fund_in_tel_aviv() {
    let rec = recommended(&can_wait(answers(dec!(50_000), dec!(0), years(5))));
    assert_eq!(rec.arena, Arena::Index);
    assert_eq!(way(rec.kind).product, Product::IsraeliIndexFund);
    assert_eq!(way(rec.kind).vehicle, Vehicle::Brokerage);
    let band = rec.band.expect("five years of history");
    assert_eq!(band.years, dec!(5));
    assert!(
        rec.alternatives
            .iter()
            .any(|a| is_safe_kind(a.kind) && a.band.is_some())
    );
}

/// c: ₪3,000 a month, no date, no cushion, ₪17,000 expenses: ₪51,000 in a
/// money market fund over 17 months, then an index kind; five rungs.
#[test]
fn c_the_cushion_comes_first_and_the_ladder_has_five_rungs() {
    let mut unknown = answers(dec!(0), dec!(3000), Horizon::Unknown);
    unknown.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(17_000),
    });
    let rec = recommended(&unknown);
    let plan = rec.two_parts.as_ref().unwrap();
    assert_eq!((plan.target, plan.months_to_fill), (dec!(51_000), 17));
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
    assert_eq!(rec.months, 17);
    assert!(matches!(plan.then, Kind::Way(_)));
    assert_eq!(ladder(&unknown, &settings(), &DATA).unwrap().len(), 5);
}

/// d: ₪1,500 a month, age 40, retire at 60: the provident fund for
/// investment as a pension, and the tax-free reason.
#[test]
fn d_retirement_is_the_provident_fund_for_investment() {
    let rec = recommended(&answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 60,
        },
    ));
    assert_eq!(way(rec.kind).vehicle, Vehicle::InvestmentGemel);
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::TaxFreePension { .. }
    )));
}

/// e: ₪10,000 + ₪2,000 a month, 17 years: the index arena, the history
/// sentence for 17 years, and what safety costs.
#[test]
fn e_seventeen_years_is_the_index_with_its_history() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(17)));
    assert_eq!(rec.arena, Arena::Index);
    assert_eq!(rec.band.map(|band| band.years), Some(dec!(17)));
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::HistoryOfTheHorizon { years } if *years == dec!(17)
    )));
    let money_fund = rec
        .alternatives
        .iter()
        .find(|a| a.kind == Kind::MoneyMarketFund)
        .unwrap();
    assert!(money_fund.left.unwrap().today < rec.left.today);
}

/// T1: ₪15,000 once, no date: the ladder; tapping 20 years gives the index
/// arena, where a tie in the shekel goes to the plan without a "may cost
/// more" caveat (Excellence before Altshuler), or the ⚠ shows on the card
/// when Altshuler is strictly ahead.
#[test]
fn t1_the_bonus_ladder_ends_in_the_index_with_the_tie_rule() {
    let bonus = answers(dec!(15_000), dec!(0), Horizon::Unknown);
    let rungs = ladder(&bonus, &settings(), &DATA).unwrap();
    let twenty = rungs.iter().find(|rung| rung.months == 240).unwrap();
    assert_eq!(twenty.arena, Arena::Index);

    let rec = recommended(&answers(dec!(15_000), dec!(0), years(20)));
    assert_eq!(rec.arena, Arena::Index);
    let (first, second) = (&rec.plans[0], &rec.plans[1]);
    if first.left.nominal.round_dp(0) == second.left.nominal.round_dp(0) {
        assert!(
            !first.may_cost_more || second.may_cost_more,
            "a tie goes to the plan without a ⚠"
        );
    } else {
        assert!(first.left.nominal > second.left.nominal);
    }
    let position = |name: &str| {
        rec.plans
            .iter()
            .position(|plan| plan.name.en.starts_with(name))
    };
    if let (Some(excellence), Some(altshuler)) = (position("Excellence"), position("Altshuler")) {
        let left = |at: usize| rec.plans[at].left.nominal.round_dp(0);
        if left(excellence) == left(altshuler) {
            assert!(
                excellence < altshuler,
                "Excellence before Altshuler at equal shekels"
            );
        } else if altshuler < excellence {
            assert!(left(altshuler) > left(excellence));
        }
    }
}

/// T2: ₪500 a month for 30 years: the index arena, and the unlock line
/// names the plan a ₪5,000 first deposit opens. The plan expected the
/// provident fund for investment as the answer; with these rates it is the
/// brokerage account's US ETF when taken as a lump sum, and the fund only
/// when taken as a pension from 60 (the next test).
#[test]
fn t2_thirty_years_of_five_hundred_names_what_a_first_deposit_opens() {
    let rec = recommended(&answers(dec!(0), dec!(500), years(30)));
    assert_eq!(rec.arena, Arena::Index);
    let unlock = rec
        .unlock
        .expect("a bigger first deposit opens a better plan");
    assert_eq!(unlock.first_deposit, dec!(5000));
    assert!(unlock.after_tax.today > unlock.instead_of.today);
    assert!(!unlock.plan.provider.en.is_empty());
}

#[test]
fn t2_taken_as_a_pension_at_sixty_the_fund_is_ahead_of_the_brokerage() {
    let rec = recommended(&answers(
        dec!(0),
        dec!(500),
        Horizon::Retirement {
            age: 30,
            retire_at: 60,
        },
    ));
    assert_eq!(way(rec.kind).vehicle, Vehicle::InvestmentGemel);
}

/// T3: ₪100,000 + ₪2,000 a month, 3 years, fixed: safe; the deposit can't
/// take monthly deposits, and says why; the stock fund is drawn with its
/// band. (The plan expected the money market fund to lead; with today's
/// yield curve a CPI-linked bond leaves more, and the test states only the
/// rule.)
#[test]
fn t3_a_flat_in_three_years_is_safe_and_the_deposit_says_why_not() {
    let rec = recommended(&fixed(answers(dec!(100_000), dec!(2000), years(3))));
    assert_eq!(rec.arena, Arena::Safe);
    let deposit = rec
        .alternatives
        .iter()
        .find(|a| a.kind == Kind::FixedDeposit)
        .expect("the deposit is drawn, with why not");
    assert!(deposit.left.is_none());
    assert_eq!(deposit.why_not.from, Circumstance::MonthlyDeposits);
    let stock = rec
        .alternatives
        .iter()
        .find(|a| matches!(a.kind, Kind::Way(_)))
        .unwrap();
    assert!(stock.band.is_some());
    assert_eq!(rec.ranked[0].kind, rec.kind);
}

/// T4: a portfolio at Leumi, ₪200,000, 20 years: the branch plan is among
/// the plans compared, marked as the saver's.
#[test]
fn t4_the_branch_plan_is_ticked_and_ranked() {
    let rec = recommended(&Answers {
        situation: Some(Situation::ExistingPortfolio {
            bank: Some(Bank::Leumi),
        }),
        ..answers(dec!(200_000), dec!(0), years(20))
    });
    assert!(rec.plans.iter().any(|plan| plan.yours));
    let without = recommended(&answers(dec!(200_000), dec!(0), years(20)));
    assert!(!without.plans.iter().any(|plan| plan.yours));
}

/// T5: the defaults: the figures are in today's money, with the nominal
/// beside them, and the current account answers "do I need to care".
#[test]
fn t5_the_answer_is_in_todays_money_with_the_yardstick() {
    let rec = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    assert!(rec.left.today < rec.left.nominal);
    assert!(rec.left.today > rec.current_account.today);
}

// ─────────────────────────── The situations ───────────────────────────

#[test]
fn every_situation_can_be_run_and_teaches_something_of_its_own() {
    let all = situations();
    let names: Vec<&str> = all.iter().map(|preset| &*preset.name.en).collect();
    assert_eq!(
        names,
        [
            "Saving from your salary",
            "For a child",
            "For a flat, or a goal on a date",
            "A safety cushion",
            "A bonus",
            "An inheritance",
            "For retirement",
            "Don't know yet",
            "I already have a portfolio at a bank",
            "I know what I want to buy",
        ]
    );
    for preset in &all {
        let rec = recommended(&preset.answers);
        assert!(!rec.reasons.is_empty(), "{}", preset.id);
        ladder(&preset.answers, &settings(), &DATA).unwrap();
        for text in [&preset.name, &preset.explanation, &preset.lesson] {
            assert!(!text.en.is_empty() && !text.he.is_empty());
        }
    }
    let distinct = |texts: Vec<&str>| {
        let mut texts = texts;
        texts.sort_unstable();
        texts.dedup();
        texts.len()
    };
    let count = all.len();
    assert_eq!(
        distinct(all.iter().map(|p| &*p.explanation.en).collect()),
        count,
        "each explains something different"
    );
    assert_eq!(
        distinct(all.iter().map(|p| &*p.lesson.en).collect()),
        count,
        "each teaches a different lesson"
    );
    assert_eq!(distinct(all.iter().map(|p| p.id).collect()), count);
    let find = |id: &str| all.iter().find(|p| p.id == id).unwrap();
    assert!(find("existing").opens_dialog);
    assert!(find("workbench").workbench);
    assert_eq!(find("goal").answers.flexibility, Some(Flexibility::Fixed));
    assert!(find("cushion").answers.needs_any_day);
}

/// Every kind has a name in both languages, the bank's words.
#[test]
fn every_kind_has_its_name() {
    let kinds = [
        Kind::MoneyMarketFund,
        Kind::FixedDeposit,
        Kind::LinkedDeposit,
        Kind::GovernmentBond { linked: false },
        Kind::GovernmentBond { linked: true },
        Kind::CurrentAccount,
    ];
    for kind in kinds {
        assert!(!kind.name(Lang::En).is_empty() && !kind.name(Lang::He).is_empty());
        assert_eq!(kind.text().he, kind.name(Lang::He));
    }
}

// ─────────────────────────── Rules the mutants asked for ───────────────────────────

fn near(a: Decimal, b: Decimal) -> bool {
    (a - b).abs() < dec!(0.01)
}

/// How much prices have risen by the end, read off an answer's own figures.
fn level_of(left: Amount) -> Decimal {
    left.nominal / left.today
}

/// The longest horizon and the largest amounts are the edges of what the
/// guide works out.
#[test]
fn the_edges_of_what_the_guide_works_out() {
    assert!(check(&answers(dec!(1), dec!(0), Horizon::Months(600))).is_ok());
    assert_eq!(
        check(&answers(dec!(1), dec!(0), Horizon::Months(601))),
        Err(InvalidAnswers::HorizonTooLong)
    );
    let retiring_in = |years: u32| {
        check(&answers(
            dec!(1),
            dec!(0),
            Horizon::Retirement {
                age: 20,
                retire_at: 20 + years,
            },
        ))
    };
    assert!(retiring_in(50).is_ok());
    assert_eq!(retiring_in(51), Err(InvalidAnswers::HorizonTooLong));
    let trillion = dec!(1_000_000_000_000);
    assert!(check(&answers(trillion, dec!(0), years(5))).is_ok());
    assert!(check(&answers(dec!(0), trillion, years(5))).is_ok());
    assert_eq!(
        check(&answers(trillion + dec!(1), dec!(0), years(5))),
        Err(InvalidAnswers::TooLarge)
    );
    assert_eq!(
        check(&answers(dec!(0), trillion + dec!(1), years(5))),
        Err(InvalidAnswers::TooLarge)
    );
}

#[test]
fn every_name_and_every_error_says_something_different() {
    let kinds = [
        Kind::MoneyMarketFund,
        Kind::FixedDeposit,
        Kind::LinkedDeposit,
        Kind::GovernmentBond { linked: false },
        Kind::GovernmentBond { linked: true },
        Kind::CurrentAccount,
    ];
    for lang in [Lang::En, Lang::He] {
        let mut names: Vec<&str> = kinds.iter().map(|kind| kind.name(lang)).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), kinds.len());
        let errors = [
            InvalidAnswers::NoHorizon,
            InvalidAnswers::HorizonTooLong,
            InvalidAnswers::NoMoney,
            InvalidAnswers::NegativeAmount,
            InvalidAnswers::TooLarge,
            InvalidAnswers::NoExpenses,
            InvalidAnswers::RetirementNotAhead,
        ];
        let mut texts: Vec<&str> = errors.iter().map(|why| why.text(lang)).collect();
        texts.sort_unstable();
        texts.dedup();
        assert_eq!(texts.len(), errors.len());
    }
}

/// A deposit is ranked when the money isn't needed any day, and left out
/// when it is.
#[test]
fn a_deposit_is_ranked_only_when_the_money_is_not_needed_any_day() {
    let year = answers(dec!(15_000), dec!(0), Horizon::Months(12));
    let has_deposit =
        |rec: &Recommendation| rec.ranked.iter().any(|l| l.kind == Kind::FixedDeposit);
    assert!(has_deposit(&recommended(&year)));
    let mut any_day = year;
    any_day.needs_any_day = true;
    assert!(!has_deposit(&recommended(&any_day)));
}

/// Choosing the arena the rules already chose changes nothing but the
/// reason; choosing the other one is worked for the same years.
#[test]
fn choosing_the_same_arena_keeps_the_rule() {
    let retire = answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 60,
        },
    );
    let mut chosen = settings();
    chosen.overrides.arena = Some(Arena::Index);
    let rec = recommend(&retire, &chosen, &DATA).unwrap();
    assert_eq!(way(rec.kind).vehicle, Vehicle::InvestmentGemel);
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::TaxFreePension { .. }
    )));
    chosen.overrides.arena = Some(Arena::Safe);
    let rec = recommend(&retire, &chosen, &DATA).unwrap();
    assert_eq!(rec.arena, Arena::Safe);
    assert_eq!(rec.months, 240);
}

/// The pension's reasons are for the fund only: whoever chose another way
/// isn't told it is tax-free.
#[test]
fn the_pension_is_said_only_of_the_provident_fund() {
    let mut chosen = settings();
    let brokerage = Kind::Way(
        broker_fees::ways::Way::for_index()
            .into_iter()
            .find(|way| way.vehicle == Vehicle::Brokerage)
            .unwrap(),
    );
    chosen.overrides.kind = Some(brokerage);
    for retire_at in [55, 60, 65] {
        let rec = recommend(
            &answers(
                dec!(0),
                dec!(1500),
                Horizon::Retirement { age: 40, retire_at },
            ),
            &chosen,
            &DATA,
        )
        .unwrap();
        assert!(!names(&rec, |c| matches!(
            c,
            Circumstance::TaxFreePension { .. }
        )));
        assert_eq!(
            names(&rec, |c| matches!(
                c,
                Circumstance::PensionNeedsSixty { .. }
            )),
            retire_at < 60,
            "retiring at {retire_at}"
        );
    }
}

/// With money coming every month the reason says so; with one sum it says
/// that.
#[test]
fn the_reason_says_whether_it_is_one_sum_or_money_every_month() {
    let rec = recommended(&fixed(answers(dec!(100_000), dec!(2000), years(3))));
    assert!(names(&rec, |c| matches!(c, Circumstance::MonthlyDeposits)));
    assert!(!names(&rec, |c| matches!(c, Circumstance::OneSum)));
    let rec = recommended(&fixed(answers(dec!(100_000), dec!(0), years(3))));
    assert!(names(&rec, |c| matches!(c, Circumstance::OneSum)));
    assert!(!names(&rec, |c| matches!(c, Circumstance::MonthlyDeposits)));
}

/// The cushion's reason repeats the arithmetic of the plan.
#[test]
fn the_cushion_reason_and_plan_agree() {
    let mut unknown = answers(dec!(0), dec!(3000), Horizon::Unknown);
    unknown.cushion = Some(Cushion::HasNot {
        monthly_expenses: dec!(17_000),
    });
    let rec = recommended(&unknown);
    let plan = rec.two_parts.as_ref().unwrap();
    assert!(names(&rec, |c| matches!(
        c,
        Circumstance::NoCushion { monthly_expenses, target }
            if *monthly_expenses == dec!(17_000) && *target == dec!(51_000)
    )));
    // The first part is worked for the months it takes to fill; the second
    // for ten years, in today's money at 2% a year.
    assert_eq!(rec.months, plan.months_to_fill);
    let ten_years = Decimal::try_from(1.02_f64.powi(10)).unwrap();
    for line in &plan.then_ranked {
        assert!((level_of(line.left) - ten_years).abs() < dec!(0.001));
    }
}

/// The breakdown of a safe answer is the engine's own, in today's money.
#[test]
fn the_breakdown_of_a_safe_answer_is_the_engines_in_todays_money() {
    use broker_fees::short_term::{self, Pays};
    let mut any_day = answers(dec!(100_000), dec!(0), Horizon::Months(12));
    any_day.needs_any_day = true;
    let rec = recommended(&any_day);
    assert_eq!(rec.kind, Kind::MoneyMarketFund);
    let kinds = short_term::kinds();
    let funds: Vec<&short_term::Place> = kinds
        .iter()
        .flat_map(|kind| &kind.places)
        .filter(|place| matches!(place.pays, Pays::TheRateLess(_)))
        .collect();
    let scenario = short_term::Scenario {
        first_deposit: dec!(100_000),
        monthly_deposit: dec!(0),
        months: 12,
        rate: short_term::TODAYS_RATE,
        inflation: Percent(dec!(2)),
    };
    let comparison = short_term::compare(&funds, &scenario);
    let outcome = comparison.places[0].outcome.as_ref().unwrap();
    let level = level_of(rec.left);
    let split = &rec.breakdown;
    assert!(outcome.tax > Decimal::ZERO);
    assert!(near(split.tax, outcome.tax / level));
    assert!(near(split.kept_by_the_place, outcome.kept / level));
    assert!(near(split.put_in, dec!(100_000) / level));
    assert!(near(split.left, outcome.after_tax / level));
    assert_eq!(split.fees, Decimal::ZERO);
    // The same of an index answer: the deposits in today's money.
    let index = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    assert!(near(
        index.breakdown.put_in,
        index.put_in.nominal / level_of(index.left)
    ));
}

/// The two bonds are the engine's, the linked one not swapped for the other.
#[test]
fn each_bond_is_the_engines_own() {
    use broker_fees::short_term::government_bonds::Linkage;
    use broker_fees::short_term::{self, Pays};
    let rec = recommended(&answers(dec!(50_000), dec!(0), years(3)));
    let kinds = short_term::kinds();
    let scenario = short_term::Scenario {
        first_deposit: dec!(50_000),
        monthly_deposit: dec!(0),
        months: 36,
        rate: short_term::TODAYS_RATE,
        inflation: Percent(dec!(2)),
    };
    for (linkage, linked) in [(Linkage::Unlinked, false), (Linkage::CpiLinked, true)] {
        let place = kinds
            .iter()
            .flat_map(|kind| &kind.places)
            .find(|place| place.pays == Pays::GovernmentBond(linkage))
            .unwrap();
        let outcome = short_term::simulate(place, &scenario).unwrap();
        let line = rec
            .ranked
            .iter()
            .find(|line| line.kind == Kind::GovernmentBond { linked })
            .unwrap();
        assert_eq!(line.left.nominal, outcome.after_tax, "linked: {linked}");
    }
}

/// A place's "may cost more" mark is the place's own.
#[test]
fn a_places_price_mark_is_its_own() {
    let mut any_day = answers(dec!(30_000), dec!(0), Horizon::Months(6));
    any_day.needs_any_day = true;
    let rec = recommended(&any_day);
    let kinds = broker_fees::short_term::kinds();
    let places: Vec<&broker_fees::short_term::Place> =
        kinds.iter().flat_map(|kind| &kind.places).collect();
    assert_ne!(rec.plans.len(), 0);
    for line in &rec.plans {
        let place = places.iter().find(|p| p.name == line.name).unwrap();
        let marked = place
            .caveats
            .iter()
            .any(|caveat| caveat.may_cost_more_summary().is_some());
        assert_eq!(line.may_cost_more, marked, "{}", line.name.en);
    }
}

/// The stock fund drawn beside a safe answer is worked over the same years,
/// so its today's money divides by the same prices.
#[test]
fn the_stock_fund_beside_a_safe_answer_is_in_the_same_todays_money() {
    let rec = recommended(&fixed(answers(dec!(50_000), dec!(0), years(5))));
    let stock = rec
        .alternatives
        .iter()
        .find(|a| matches!(a.kind, Kind::Way(_)))
        .unwrap();
    let left = stock.left.unwrap();
    assert!((level_of(left) - level_of(rec.left)).abs() < dec!(0.0001));
}

/// The other ways are drawn beside an index answer only for a retirement
/// saver, once each: the study fund (with who can open one) is always
/// there.
#[test]
fn only_a_retirement_saver_sees_the_other_ways_drawn() {
    let ways = |rec: &Recommendation| -> Vec<Vehicle> {
        rec.alternatives
            .iter()
            .filter_map(|a| match a.kind {
                Kind::Way(way) => Some(way.vehicle),
                _ => None,
            })
            .collect()
    };
    let long = recommended(&answers(dec!(10_000), dec!(2000), years(17)));
    assert_eq!(ways(&long), [Vehicle::StudyFund]);
    let retire = recommended(&answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 60,
        },
    ));
    let drawn = ways(&retire);
    for vehicle in [
        Vehicle::Brokerage,
        Vehicle::StudyFund,
        Vehicle::SavingsPolicy,
    ] {
        assert_eq!(
            drawn.iter().filter(|&&v| v == vehicle).count(),
            1,
            "{vehicle:?}"
        );
    }
}

/// A deposit is drawn with "why not" when money comes every month, but only
/// where a deposit could have been (up to five years).
#[test]
fn the_deposit_is_drawn_only_where_it_could_exist() {
    let deposit_of = |rec: &Recommendation| {
        rec.alternatives
            .iter()
            .find(|a| a.kind == Kind::FixedDeposit)
            .cloned()
    };
    let five = deposit_of(&recommended(&fixed(answers(
        dec!(1000),
        dec!(500),
        years(5),
    ))))
    .unwrap();
    assert!(five.left.is_none());
    assert!(
        deposit_of(&recommended(&fixed(answers(
            dec!(1000),
            dec!(500),
            years(6)
        ))))
        .is_none()
    );
    assert!(deposit_of(&recommended(&fixed(answers(dec!(1000), dec!(0), years(6))))).is_none());
    // And why not is the locked term when the money is needed any day.
    let mut any_day = answers(dec!(1000), dec!(0), Horizon::Months(12));
    any_day.needs_any_day = true;
    let locked = deposit_of(&recommended(&any_day)).unwrap();
    assert!(matches!(
        locked.why_not.from,
        Circumstance::DepositLocked { .. }
    ));
    let year = deposit_of(&recommended(&answers(
        dec!(1000),
        dec!(0),
        Horizon::Months(24),
    )));
    if let Some(deposit) = year {
        assert!(!matches!(
            deposit.why_not.from,
            Circumstance::DepositLocked { .. }
        ));
    }
}

/// The reasons of a chosen safe kind are about that kind.
#[test]
fn a_safe_answer_gives_the_reasons_of_its_own_kind() {
    let mut seen = Vec::new();
    for months in [6, 12, 24, 36, 60, 84] {
        for flexibility in [Flexibility::Fixed, Flexibility::CanWait] {
            let rec = recommended(&Answers {
                flexibility: Some(flexibility),
                ..answers(dec!(50_000), dec!(0), Horizon::Months(months))
            });
            if rec.arena != Arena::Safe {
                continue;
            }
            seen.push(rec.kind);
            match rec.kind {
                Kind::FixedDeposit => {
                    assert!(names(&rec, |c| matches!(
                        c,
                        Circumstance::DepositLocked { .. }
                    )));
                    assert!(names(&rec, |c| matches!(
                        c,
                        Circumstance::SumKnownInAdvance
                    )));
                }
                Kind::MoneyMarketFund => {
                    assert!(names(&rec, |c| matches!(
                        c,
                        Circumstance::EarnsBankRate { .. }
                    )));
                }
                Kind::GovernmentBond { .. } => {
                    assert!(names(&rec, |c| matches!(c, Circumstance::BondCanBeSold)));
                }
                _ => panic!("not a safe kind"),
            }
        }
    }
    assert!(seen.contains(&Kind::FixedDeposit), "{seen:?}");
}

/// The ladder asks the same money without the date, the situation or the
/// flexibility of the answers that opened it.
#[test]
fn the_ladder_ignores_what_only_the_answer_had() {
    let mut sum = answers(dec!(15_000), dec!(0), Horizon::Unknown);
    sum.situation = Some(Situation::KnowsWhatToBuy);
    sum.flexibility = Some(Flexibility::Fixed);
    let rungs = ladder(&sum, &settings(), &DATA).unwrap();
    let by_months = |months| {
        rungs
            .iter()
            .find(|r| r.months == months && !r.any_day)
            .unwrap()
    };
    assert_eq!(
        by_months(12).arena,
        Arena::Safe,
        "not every way, as the situation said"
    );
    assert_eq!(by_months(60).arena, Arena::Index, "its date can move");
}

/// What a bigger first deposit opens is a plan of the same kind of account
/// as its way, held against the answer at that same deposit.
#[test]
fn the_unlock_line_is_held_against_the_answer_at_the_same_deposit() {
    let base = answers(dec!(0), dec!(500), years(30));
    let rec = recommended(&base);
    let unlock = rec.unlock.as_ref().unwrap();
    // A brokerage way, opened with a broker's plan.
    assert_eq!(way(unlock.kind).vehicle, Vehicle::Brokerage);
    let broker = DATA
        .brokers
        .iter()
        .find(|broker| broker.short_name == unlock.plan.provider)
        .unwrap();
    assert_ne!(broker.kind, broker_fees::BrokerKind::Funds);
    // With that first deposit the ranking says the same.
    let with = recommended(&Answers {
        now: unlock.first_deposit,
        ..base
    });
    let line = |kind| with.ranked.iter().find(|l| l.kind == kind).unwrap();
    assert_eq!(line(unlock.kind).left.nominal, unlock.after_tax.nominal);
    assert!(unlock.instead_of.nominal <= line(rec.kind).left.nominal);
    assert!(
        unlock.instead_of.nominal > rec.left.nominal,
        "the deposit itself adds"
    );
}

/// A deposit that was not chosen, with the money not needed any day, is not
/// said to be locked: it is the date that decides.
#[test]
fn a_deposit_beside_a_fixed_date_is_not_blamed_for_being_locked() {
    let mut other = settings();
    other.overrides.kind = Some(Kind::GovernmentBond { linked: false });
    let rec = recommend(
        &fixed(answers(dec!(50_000), dec!(0), years(5))),
        &other,
        &DATA,
    )
    .unwrap();
    let deposit = rec
        .alternatives
        .iter()
        .find(|a| a.kind == Kind::FixedDeposit)
        .unwrap();
    assert!(deposit.left.is_some());
    assert!(matches!(
        deposit.why_not.from,
        Circumstance::FixedDate { months: 60 }
    ));
}

/// A CPI-linked bond tells the truth about its safety: its real value is
/// known, its sum in shekels is not. It is ranked where a known sum is not
/// required, and drawn beside the answer where it is.
#[test]
fn a_linked_bond_is_not_a_sum_known_in_advance() {
    assert_eq!(
        Kind::GovernmentBond { linked: true }.safety(),
        Safety::RealValueKnown
    );
    assert_eq!(
        Kind::GovernmentBond { linked: false }.safety(),
        Safety::SumKnown
    );
    let fixed_date = recommended(&fixed(answers(dec!(50_000), dec!(0), years(5))));
    assert_ne!(fixed_date.kind, Kind::GovernmentBond { linked: true });
    assert!(
        fixed_date
            .alternatives
            .iter()
            .any(|a| a.kind == Kind::GovernmentBond { linked: true }
                && a.safety == Safety::RealValueKnown)
    );
    let known_date = recommended(&answers(dec!(50_000), dec!(0), Horizon::Months(24)));
    assert!(
        known_date
            .ranked
            .iter()
            .any(|l| l.kind == Kind::GovernmentBond { linked: true })
    );
}

// ─────────────────────────── The alternatives' lines ───────────────────────────

/// The chart's lines are the alternatives' figures drawn whole: each ends at
/// the amount `recommend` states for the kind, and the first is the answer.
#[test]
fn the_alternatives_lines_end_at_the_figures_of_the_answer() {
    for preset in situations() {
        let rec = recommended(&preset.answers);
        let lines = alternatives(&preset.answers, &settings(), &DATA).unwrap();
        assert_eq!(lines.series[0].kind, rec.kind, "{}", preset.id);
        assert!(lines.series[0].recommended);
        assert_eq!(lines.series[0].left, rec.left, "{}", preset.id);
        for line in &lines.series {
            assert_eq!(
                line.value_by_month.last().copied(),
                Some(line.left.today),
                "{} {:?}",
                preset.id,
                line.kind
            );
            assert!(line.value_by_month.len() > 1);
            // Month 0 means the same for every line: the first sum alone.
            assert_eq!(
                line.value_by_month[0], preset.answers.now,
                "{} {:?} starts at the first sum",
                preset.id, line.kind
            );
        }
        if let Some(band) = &lines.band {
            assert_eq!(band.worst[0], preset.answers.now, "{}", preset.id);
            assert_eq!(band.best[0], preset.answers.now, "{}", preset.id);
        }
        // The current account is always drawn: the yardstick.
        assert!(
            lines
                .series
                .iter()
                .any(|line| line.kind == Kind::CurrentAccount),
            "{}",
            preset.id
        );
    }
}

/// The band's lines are the deposits grown at the history's worst and best
/// yearly rates: worst never above best, both starting at the first deposit
/// and the first month's.
#[test]
fn the_bands_lines_bracket_the_deposits() {
    let ten_years = answers(dec!(10_000), dec!(1_000), years(10));
    let lines = alternatives(&can_wait(ten_years), &settings(), &DATA).unwrap();
    let band = lines.band.expect("ten years are inside the history");
    assert_eq!(band.worst.len(), 121);
    assert_eq!(band.worst[0], dec!(10_000));
    assert_eq!(band.best[0], dec!(10_000));
    assert!(
        band.worst
            .iter()
            .zip(&band.best)
            .all(|(worst, best)| worst <= best)
    );
}

/// The columns of the list within an arena: both rank by what is left and
/// show the tax; only the index arena's plans show the fees. Every row has an
/// amount for each column of its arena's list.
#[test]
fn the_columns_follow_the_arena() {
    let ids = |arena| {
        columns_for(arena)
            .into_iter()
            .map(|column| column.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ids(Arena::Safe),
        [ColumnId::LeftAfterTax, ColumnId::CostAYear, ColumnId::Tax]
    );
    assert_eq!(
        ids(Arena::Index),
        [
            ColumnId::LeftAfterTax,
            ColumnId::CostAYear,
            ColumnId::Fees,
            ColumnId::Tax
        ]
    );
    let safe = recommended(&fixed(answers(dec!(100_000), dec!(0), Horizon::Months(8))));
    assert_eq!(safe.arena, Arena::Safe);
    assert!(safe.plans.iter().any(|place| place.tax.is_some()));
    let index = recommended(&answers(dec!(10_000), dec!(2000), years(20)));
    assert!(
        index
            .plans
            .iter()
            .all(|plan| plan.tax.is_some() && plan.fees.is_some())
    );
}

/// Row d: the workbench's ticks drive its own comparison only. With a single
/// broker ticked the guide still draws the brokerage account, the study fund
/// (with who can open one) and the savings policy, and answers the same.
#[test]
fn the_alternatives_do_not_depend_on_the_ticked_plans() {
    let sum = answers(
        dec!(0),
        dec!(1500),
        Horizon::Retirement {
            age: 40,
            retire_at: 60,
        },
    );
    let usual = recommended(&sum);
    let mut one = settings();
    let leumi = DATA
        .brokers
        .iter()
        .position(|broker| broker.short_name.en == "Leumi")
        .unwrap();
    one.ticked = Some(vec![broker_fees::guide::PlanRef {
        broker: leumi,
        plan: DATA.brokers[leumi].new_customer_plan,
    }]);
    let rec = recommend(&sum, &one, &DATA).unwrap();
    assert_eq!(rec.kind, usual.kind);
    assert_eq!(rec.choice, usual.choice);
    for wanted in [
        Vehicle::Brokerage,
        Vehicle::StudyFund,
        Vehicle::SavingsPolicy,
    ] {
        assert!(
            rec.alternatives
                .iter()
                .any(|a| matches!(a.kind, Kind::Way(w) if w.vehicle == wanted)),
            "{wanted:?} is drawn"
        );
    }
    assert!(
        rec.alternatives
            .iter()
            .any(|a| a.why_not.from == Circumstance::StudyFundNeedsEligibility)
    );
}

/// The ticked brokers are the ones compared: a broker that is not ticked is
/// not among the plans, though every kind of fund always is drawn.
#[test]
fn only_the_ticked_brokers_are_compared() {
    let sum = answers(dec!(15_000), dec!(0), years(20));
    let leumi = DATA
        .brokers
        .iter()
        .position(|broker| broker.short_name.en == "Leumi")
        .unwrap();
    let mut one = settings();
    one.ticked = Some(vec![broker_fees::guide::PlanRef {
        broker: leumi,
        plan: DATA.brokers[leumi].new_customer_plan,
    }]);
    let rec = recommend(&sum, &one, &DATA).unwrap();
    assert_eq!(rec.plans.len(), 1, "{:?}", rec.plans);
}

// ─────────────────────────── A plan that may cost more ───────────────────────────

/// The answer names the best plan whose price can't be too low. A plan that
/// leaves more but carries a may-cost-more caveat is "also possible": said
/// beside the answer with its caveat, and in the steps.
#[test]
fn the_plan_named_is_the_best_without_a_may_cost_more_caveat() {
    let rec = recommended(&answers(dec!(10_000), dec!(2_000), years(20)));
    let also = rec
        .also_possible
        .as_ref()
        .expect("a plan ahead of the answer carries a may-cost-more caveat in the listed data");
    assert!(!rec.plans[0].may_cost_more, "the answer is flagged");
    assert_eq!(rec.plans[0].left, rec.left);
    assert_eq!(
        rec.choice.provider.en,
        rec.plans[0].name.en.split(" · ").next().unwrap()
    );
    // The plan beside it leaves more, by what the line says, and is the
    // flagged one.
    let flagged = &rec.plans[1];
    assert!(flagged.may_cost_more);
    assert_eq!(flagged.left, also.left);
    assert!(also.left.today > rec.left.today);
    assert_eq!(
        also.ahead_by.today.round_dp(6),
        (also.left.today - rec.left.today).round_dp(6)
    );
    assert!(
        also.warning.en.starts_with("May cost more: "),
        "{:?}",
        also.warning
    );
    assert!(
        also.warning.he.starts_with("עשוי לעלות יותר: "),
        "{:?}",
        also.warning
    );
    // No other plan that is ranked higher without a caveat was passed over.
    assert!(
        rec.plans
            .iter()
            .skip(2)
            .all(|plan| plan.may_cost_more || plan.left.today <= rec.left.today || !plan.fits)
    );
}

#[test]
fn the_flagged_plan_is_said_in_the_line_and_in_the_steps_with_its_own_caveat() {
    use broker_fees::describe::answer::also_possible_line;
    let rec = recommended(&answers(dec!(10_000), dec!(2_000), years(20)));
    let also = rec.also_possible.as_ref().unwrap();
    for lang in [Lang::En, Lang::He] {
        let line = also_possible_line(also, lang);
        // It names the plan and the caveat's own summary, and marks it.
        assert!(line.contains(&also.choice.provider[lang]), "{line}");
        let summary = also.warning[lang]
            .split_once(": ")
            .map(|(_, summary)| summary.to_owned())
            .unwrap();
        assert!(line.contains(&summary), "{line}");
        assert!(line.contains('\u{26a0}'), "{line}");
        let steps = broker_fees::describe::reasons::steps_for(&rec, lang);
        assert!(
            steps.iter().any(|step| step.text.contains(&summary)),
            "the steps leave out the caveat"
        );
    }
}

/// With no flagged plan ahead, nothing is added and the answer is the best.
#[test]
fn nothing_is_said_when_the_best_plan_has_no_caveat() {
    let rec = recommended(&answers(dec!(50_000), dec!(0), years(10)));
    assert!(rec.also_possible.is_none());
    assert!(!rec.plans[0].may_cost_more);
}
