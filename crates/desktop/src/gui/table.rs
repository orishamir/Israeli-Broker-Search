//! The results table, best plan first. Each plan's colored mark makes it the
//! chart's legend.

use std::collections::HashSet;

use broker_fees::simulation::Outcome;
use eframe::egui::{self, Align, Layout, RichText};
use egui_extras::{Column, TableBuilder, TableRow};
use rust_decimal::Decimal;

use super::widgets::{color_mark, shekels, to_f64};
use crate::{PlanInteraction, PlanKey, PlanResult};

/// `results` must be sorted best first. Rows can be hovered and clicked as a
/// whole; highlighted plans get a tint of their own color. The fees paid are
/// links to the fee breakdown view.
pub fn results_table(
    ui: &mut egui::Ui,
    no_fees: &Outcome,
    results: &[PlanResult],
    highlighted: &HashSet<PlanKey>,
) -> PlanInteraction {
    let mut interaction = PlanInteraction::default();
    let row_height = ui.text_style_height(&egui::TextStyle::Body) + 8.0;

    ui.scope(|ui| {
        // Selectable text would take the mouse from the rows it's in.
        ui.style_mut().interaction.selectable_labels = false;

        TableBuilder::new(ui)
            .id_salt("results")
            .striped(true)
            .sense(egui::Sense::click())
            .vscroll(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::exact(20.0)) // rank
            .column(Column::auto().at_most(150.0).clip(true)) // broker
            .column(Column::auto()) // plan
            .columns(Column::auto().at_least(90.0), 4) // amounts
            .header(row_height, header_row)
            .body(|mut body| {
                for (index, result) in results.iter().enumerate() {
                    // A highlighted row is drawn as "selected", in the plan's
                    // color rather than the theme's selection color.
                    let visuals = body.ui_mut().visuals_mut();
                    visuals.selection.bg_fill = result.color.gamma_multiply(0.25);
                    visuals.selection.stroke.color = visuals.text_color();

                    body.row(row_height, |mut row| {
                        row.set_selected(highlighted.contains(&result.key));
                        if plan_row(&mut row, index + 1, result, no_fees) {
                            interaction.show_breakdown = true;
                        }

                        let response = row.response();
                        if response.hovered() {
                            interaction.hovered = Some(result.key);
                            response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if response.clicked() {
                            interaction.clicked = Some(result.key);
                        }
                    });
                }
            });
    });
    interaction
}

fn header_row(mut header: TableRow<'_, '_>) {
    header.col(|ui| {
        ui.strong("#");
    });
    header.col(|ui| {
        ui.strong("Broker");
    });
    header.col(|ui| {
        ui.strong("Plan");
    });
    let amounts = [
        (
            "Value held",
            "What the investment is worth at the end, without selling.",
        ),
        (
            "Value if sold",
            "What you'd get in shekels by selling everything at the end, \
             after the sell fee and converting back. Before tax.",
        ),
        (
            "Fees paid",
            "Every fee charged: purchases, conversions, custody, and selling \
             at the end. Click an amount to see what it went to.",
        ),
        (
            "Lost to fees",
            "How much less you end up with than with no fees at all, after \
             selling. Bigger than the fees paid, because money paid in fees \
             stops growing.",
        ),
    ];
    for (title, explanation) in amounts {
        header.col(|ui| {
            right_aligned(ui, |ui| ui.strong(title).on_hover_text(explanation));
        });
    }
}

/// Plans that don't offer the trade are greyed out, without a rank or amounts.
/// Returns true if the fees paid link was clicked.
fn plan_row(
    row: &mut TableRow<'_, '_>,
    rank: usize,
    result: &PlanResult,
    no_fees: &Outcome,
) -> bool {
    let offered = result.outcome.is_some();
    let mut fees_clicked = false;

    row.col(|ui| {
        if offered {
            ui.label(rank.to_string());
        } else {
            ui.weak("–");
        }
    });
    row.col(|ui| {
        ui.weak(&result.broker_name);
    });
    row.col(|ui| {
        let (color, name) = if offered {
            // Bold rather than colored, so it isn't mistaken for a plan's color.
            let name = RichText::new(&result.plan.name);
            (result.color, if rank == 1 { name.strong() } else { name })
        } else {
            (
                result.color.gamma_multiply(0.3),
                RichText::new(&result.plan.name).weak(),
            )
        };
        color_mark(ui, color);
        ui.label(name);
    });

    let Some(outcome) = &result.outcome else {
        for _ in 0..4 {
            row.col(|ui| {
                right_aligned(ui, |ui| ui.weak("–"));
            });
        }
        return false;
    };
    row.col(|ui| amount_cell(ui, outcome.held));
    row.col(|ui| amount_cell(ui, outcome.after_selling));
    row.col(|ui| {
        // A link, to say there's more behind the number.
        let fees = format!("{} ›", shekels(to_f64(outcome.fees.total())));
        fees_clicked = right_aligned(ui, |ui| ui.link(fees))
            .on_hover_text("See what these fees went to")
            .clicked();
    });
    row.col(|ui| amount_cell(ui, no_fees.after_selling - outcome.after_selling));
    fees_clicked
}

fn amount_cell(ui: &mut egui::Ui, amount: Decimal) {
    right_aligned(ui, |ui| ui.label(shekels(to_f64(amount))));
}

fn right_aligned<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.with_layout(Layout::right_to_left(Align::Center), add_contents)
        .inner
}
