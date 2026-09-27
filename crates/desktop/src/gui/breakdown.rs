//! The "Fee breakdown" view: one bar per plan, split by fee type, all on the
//! same ₪ scale, so plans compare by what they pay and why.
//!
//! - A year slider shows the fees paid up to that year, so you can watch
//!   which fees build up over time. The scale stays that of the last year, so
//!   bars visibly grow.
//! - Focusing a fee type moves it to the start of every bar and sorts the
//!   plans by it: in a stacked bar only the first part starts at the same
//!   place, so only it can be compared by eye.
//!
//! Drawn with egui's painter rather than egui_plot, which can't put plan
//! names on its axis or write amounts inside the bars.

use std::collections::HashSet;

use broker_fees::simulation::Outcome;
use eframe::egui::{self, RichText};

use super::plan_info::{FEE_TYPES, by_type};
use super::widgets::{color_mark, compact_shekels, readable_on, shekels, to_f64};
use crate::{PlanInteraction, PlanKey, PlanResult};

#[derive(Default)]
pub struct BreakdownState {
    /// The fee type (index into `FEE_TYPES`) lined up at the start of every bar.
    focus: Option<usize>,
    /// Show the fees paid up to the end of this year. `None` is the last year.
    up_to_year: Option<usize>,
}

const NAME_WIDTH: f32 = 240.0;
const TOTAL_WIDTH: f32 = 150.0;
const ROW_HEIGHT: f32 = 36.0;
const BAR_HEIGHT: f32 = 18.0;

pub fn fee_breakdown(
    ui: &mut egui::Ui,
    state: &mut BreakdownState,
    results: &[PlanResult],
    highlighted: &HashSet<PlanKey>,
) -> PlanInteraction {
    let offered: Vec<(&PlanResult, &Outcome)> = results
        .iter()
        .filter_map(|result| Some((result, result.outcome.as_ref()?)))
        .collect();
    let last_year = offered
        .first()
        .map_or(0, |(_, outcome)| outcome.fees_by_year.len());
    let year = state.up_to_year.unwrap_or(last_year).min(last_year);

    controls(ui, state, year, last_year);
    ui.add_space(8.0);

    // Rows in the order of the last year, so they don't jump while the slider moves.
    let mut rows: Vec<(&PlanResult, [f64; 4], [f64; 4])> = offered
        .iter()
        .map(|(result, outcome)| {
            (
                *result,
                fees_up_to(outcome, year),
                fees_up_to(outcome, last_year),
            )
        })
        .collect();
    rows.sort_by(|a, b| sort_key(&a.2, state.focus).total_cmp(&sort_key(&b.2, state.focus)));
    let scale = rows
        .iter()
        .map(|(_, _, final_fees)| final_fees.iter().sum::<f64>())
        .fold(0.0, f64::max);

    let mut interaction = PlanInteraction::default();
    let bar_width = (ui.available_width() - NAME_WIDTH - TOTAL_WIDTH).max(100.0);
    for (result, fees, _) in &rows {
        let row = plan_row(ui, result, fees, scale, bar_width, state.focus, highlighted);
        if row.hovered() {
            interaction.hovered = Some(result.key);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if row.clicked() {
            interaction.clicked = Some(result.key);
        }
    }
    scale_axis(ui, scale, bar_width);

    let not_offered: Vec<&str> = results
        .iter()
        .filter(|result| result.outcome.is_none())
        .map(|result| result.name.as_str())
        .collect();
    if !not_offered.is_empty() {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "Not offered for this security: {}",
                not_offered.join(", ")
            ))
            .weak(),
        );
    }
    interaction
}

/// The year slider and the fee type toggles.
fn controls(ui: &mut egui::Ui, state: &mut BreakdownState, mut year: usize, last_year: usize) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Fees paid up to year");
        if ui
            .add(egui::Slider::new(&mut year, 1..=last_year.max(1)))
            .changed()
        {
            state.up_to_year = (year < last_year).then_some(year);
        }
        if year == last_year {
            ui.label(RichText::new("(including selling at the end)").weak());
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Compare one fee:");
        for (index, (name, color)) in FEE_TYPES.iter().enumerate() {
            let focused = state.focus == Some(index);
            color_mark(ui, *color);
            if ui.selectable_label(focused, *name).clicked() {
                state.focus = if focused { None } else { Some(index) };
            }
        }
    });
    let hint = match state.focus {
        Some(index) => format!(
            "{} is lined up at the start of every bar and the plans are sorted by it. \
             Click it again to go back to totals.",
            FEE_TYPES[index].0
        ),
        None => "Click a fee type to line it up at the start of every bar and compare it \
                 across plans. Click a plan to pin it."
            .to_owned(),
    };
    ui.label(RichText::new(hint).weak().small());
}

/// Fees by type (in `FEE_TYPES` order) paid up to the end of `year`. Selling
/// happens at the end, so it only counts in the last year.
fn fees_up_to(outcome: &Outcome, year: usize) -> [f64; 4] {
    let mut total = [0.0; 4];
    for fees in outcome.fees_by_year.iter().take(year) {
        for (sum, amount) in total.iter_mut().zip(by_type(fees)) {
            *sum += to_f64(amount);
        }
    }
    if year >= outcome.fees_by_year.len() {
        total[3] += to_f64(outcome.fees.selling);
    }
    total
}

/// Plans are sorted by the focused fee type, or by their total.
fn sort_key(fees: &[f64; 4], focus: Option<usize>) -> f64 {
    focus.map_or_else(|| fees.iter().sum(), |index| fees[index])
}

/// One plan: its name, its bar and its total. The whole row can be hovered
/// and clicked.
fn plan_row(
    ui: &mut egui::Ui,
    result: &PlanResult,
    fees: &[f64; 4],
    scale: f64,
    bar_width: f32,
    focus: Option<usize>,
    highlighted: &HashSet<PlanKey>,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::click());
    let painter = ui.painter();
    if highlighted.contains(&result.key) {
        painter.rect_filled(rect, 3.0, result.color.gamma_multiply(0.25));
    }

    // The plan's name, with the broker's small above it so the name column
    // stays narrow and the bars get the room.
    let name_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 26.0, rect.top()),
        egui::pos2(rect.left() + NAME_WIDTH - 8.0, rect.bottom()),
    );
    let name_painter = painter.with_clip_rect(name_rect);
    name_painter.text(
        egui::pos2(name_rect.left(), rect.center().y - 1.0),
        egui::Align2::LEFT_BOTTOM,
        &result.broker_name,
        egui::FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );
    let plan_name = name_painter.text(
        egui::pos2(name_rect.left(), rect.center().y - 1.0),
        egui::Align2::LEFT_TOP,
        &result.plan.name,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );
    let mark = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 12.0, plan_name.center().y),
        egui::Vec2::splat(11.0),
    );
    painter.rect_filled(mark, 2.0, result.color);

    // The bar: the focused type first, then the rest in their usual order.
    let order: Vec<usize> = focus
        .into_iter()
        .chain((0..FEE_TYPES.len()).filter(|index| Some(*index) != focus))
        .collect();
    let bar_left = rect.left() + NAME_WIDTH;
    let mut left = bar_left;
    for index in order {
        let (_, color) = FEE_TYPES[index];
        let faded = focus.is_some_and(|focused| focused != index);
        let color = if faded {
            color.gamma_multiply(0.3)
        } else {
            color
        };
        let width = if scale > 0.0 {
            (fees[index] / scale) as f32 * bar_width
        } else {
            0.0
        };
        let segment = egui::Rect::from_min_size(
            egui::pos2(left, rect.center().y - BAR_HEIGHT / 2.0),
            egui::vec2(width, BAR_HEIGHT),
        );
        painter.rect_filled(segment, 2.0, color);
        // The amount, if it fits inside the segment and isn't faded.
        let text = compact_shekels(fees[index]);
        let galley =
            painter.layout_no_wrap(text, egui::FontId::proportional(12.0), readable_on(color));
        if !faded && galley.size().x + 8.0 < width {
            painter.galley(segment.center() - galley.size() / 2.0, galley, color);
        }
        left += width;
    }

    // The total after the bar.
    let total: f64 = fees.iter().sum();
    let total_text = match focus {
        Some(index) => format!(
            "{} of {}",
            compact_shekels(fees[index]),
            compact_shekels(total)
        ),
        None => shekels(total),
    };
    painter.text(
        egui::pos2(left + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        total_text,
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );

    response.on_hover_ui(|ui| {
        ui.strong(&result.name);
        egui::Grid::new("breakdown tooltip")
            .num_columns(3)
            .min_col_width(0.0)
            .show(ui, |ui| {
                for ((name, color), amount) in FEE_TYPES.iter().zip(fees) {
                    color_mark(ui, *color);
                    ui.label(*name);
                    ui.label(shekels(*amount));
                    ui.end_row();
                }
            });
    })
}

/// ₪ marks under the bars, at round amounts.
fn scale_axis(ui: &mut egui::Ui, scale: f64, bar_width: f32) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), egui::Sense::hover());
    if scale <= 0.0 {
        return;
    }
    let painter = ui.painter();
    let bar_left = rect.left() + NAME_WIDTH;
    let step = round_step(scale / 4.0);
    let mut amount = 0.0;
    while amount <= scale {
        let x = bar_left + (amount / scale) as f32 * bar_width;
        painter.vline(
            x,
            rect.top()..=rect.top() + 4.0,
            egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
        );
        painter.text(
            egui::pos2(x, rect.top() + 5.0),
            egui::Align2::CENTER_TOP,
            compact_shekels(amount),
            egui::FontId::proportional(11.0),
            ui.visuals().weak_text_color(),
        );
        amount += step;
    }
}

/// The nearest 1, 2 or 5 times a power of ten at or above `rough`.
fn round_step(rough: f64) -> f64 {
    let power = 10f64.powf(rough.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * power)
        .find(|step| *step >= rough)
        .unwrap_or(10.0 * power)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_round() {
        assert_eq!(round_step(23_000.0), 50_000.0);
        assert_eq!(round_step(18_000.0), 20_000.0);
        assert_eq!(round_step(1_000.0), 1_000.0);
    }
}
