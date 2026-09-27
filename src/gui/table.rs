//! The results table, best plan first. Each plan's colored mark makes it the
//! chart's legend.

use std::collections::HashSet;

use broker_fees::simulation::Outcome;
use eframe::egui::{self, RichText};
use rust_decimal::Decimal;

use super::widgets::{color_mark, shekels, to_f64};
use crate::{PlanInteraction, PlanKey, PlanResult};

/// Wide enough for a two-digit rank.
const RANK_WIDTH: f32 = 20.0;

/// `results` must be sorted best first.
pub fn results_table(
    ui: &mut egui::Ui,
    no_fees: &Outcome,
    results: &[PlanResult],
    highlighted: &HashSet<PlanKey>,
) -> PlanInteraction {
    let mut interaction = PlanInteraction::default();
    egui::Grid::new("results")
        .striped(true)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            header_row(ui);
            for (index, result) in results.iter().enumerate() {
                let rank = result.outcome.is_some().then_some(index + 1);
                let row = plan_row(ui, rank, result, no_fees, highlighted);
                interaction.hovered = interaction.hovered.or(row.hovered);
                interaction.clicked = interaction.clicked.or(row.clicked);
            }
        });
    interaction
}

fn header_row(ui: &mut egui::Ui) {
    // The rank shares the broker's cell, so it doesn't get a whole column's spacing.
    ui.horizontal(|ui| {
        rank_label(ui, RichText::new("#").strong());
        ui.strong("Broker");
    });
    ui.strong("Plan");
    ui.strong("Value held")
        .on_hover_text("What the investment is worth at the end, without selling.");
    ui.strong("Value if sold").on_hover_text(
        "What you'd get in shekels by selling everything at the end, \
         after the sell fee and converting back. Before tax.",
    );
    ui.strong("Fees paid").on_hover_text(
        "Every fee charged while investing: purchases, conversions, custody. \
         Not including selling.",
    );
    ui.strong("Lost to fees").on_hover_text(
        "How much less you end up with than with no fees at all, after \
         selling. Bigger than the fees paid, because money paid in fees \
         stops growing.",
    );
    ui.end_row();
}

/// One plan's row. `rank` is `None` for plans that don't offer the trade.
/// The whole row can be hovered and clicked, not just its text.
fn plan_row(
    ui: &mut egui::Ui,
    rank: Option<usize>,
    result: &PlanResult,
    no_fees: &Outcome,
    highlighted: &HashSet<PlanKey>,
) -> PlanInteraction {
    // Filled in once the row's size is known, but drawn behind it.
    let background = ui.painter().add(egui::Shape::Noop);
    let offered = result.outcome.is_some();
    let weak_unless_offered = |text: &str| {
        let text = RichText::new(text);
        if offered { text } else { text.weak() }
    };

    let rank = rank.map_or("–".to_owned(), |rank| rank.to_string());
    let mut cells = vec![
        ui.horizontal(|ui| {
            rank_label(ui, weak_unless_offered(&rank));
            ui.label(weak_unless_offered(&result.broker_name).weak());
        })
        .response,
    ];
    let mark_color = if offered {
        result.color
    } else {
        result.color.gamma_multiply(0.3)
    };
    // Bold rather than colored, so it isn't mistaken for a plan's color.
    let is_best = rank == "1";
    let plan_name = weak_unless_offered(&result.plan_name);
    cells.push(
        ui.horizontal(|ui| {
            color_mark(ui, mark_color);
            ui.label(if is_best {
                plan_name.strong()
            } else {
                plan_name
            });
        })
        .response,
    );

    let Some(outcome) = &result.outcome else {
        ui.label(RichText::new("doesn't offer this security on this exchange").weak());
        ui.end_row();
        return PlanInteraction::default();
    };
    cells.push(amount_cell(ui, outcome.held));
    cells.push(amount_cell(ui, outcome.after_selling));
    cells.push(amount_cell(ui, outcome.fees_paid));
    cells.push(amount_cell(
        ui,
        no_fees.after_selling - outcome.after_selling,
    ));
    ui.end_row();

    let row_rect = cells
        .iter()
        .map(|cell| cell.rect)
        .reduce(|row, cell| row.union(cell))
        .expect("a row has cells")
        .expand2(egui::vec2(8.0, 2.0));
    let row = ui.interact(
        row_rect,
        egui::Id::new(("result row", result.key)),
        egui::Sense::click(),
    );

    let mut interaction = PlanInteraction::default();
    if row.hovered() {
        interaction.hovered = Some(result.key);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if row.clicked() {
        interaction.clicked = Some(result.key);
    }
    if highlighted.contains(&result.key) {
        ui.painter().set(
            background,
            egui::Shape::rect_filled(row_rect, 3.0, result.color.gamma_multiply(0.25)),
        );
    }
    interaction
}

/// A label padded to `RANK_WIDTH`, so the broker names after it line up.
fn rank_label(ui: &mut egui::Ui, text: RichText) {
    let width = ui.label(text).rect.width();
    ui.add_space((RANK_WIDTH - width - ui.spacing().item_spacing.x).max(0.0));
}

/// Padded in a monospace font, so columns of numbers line up on the right.
fn amount_cell(ui: &mut egui::Ui, amount: Decimal) -> egui::Response {
    ui.monospace(format!("{:>12}", shekels(to_f64(amount))))
}
