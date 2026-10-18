//! The rules.

use super::{Answers, Data, InvalidAnswers, Recommendation, Rung, Settings};

/// What fits money like this.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn recommend(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
) -> Result<Recommendation, InvalidAnswers> {
    let _ = (answers, settings, data);
    unimplemented!("first commit: types only")
}

/// The same money at the ladder's horizons.
///
/// # Errors
///
/// When the answers can't be worked on.
pub fn ladder(
    answers: &Answers,
    settings: &Settings,
    data: &Data,
) -> Result<Vec<Rung>, InvalidAnswers> {
    let _ = (answers, settings, data);
    unimplemented!("first commit: types only")
}
