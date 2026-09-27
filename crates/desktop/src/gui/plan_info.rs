//! What brokers and plans are and what they charge, as read from their
//! tariffs: the preview card and details window of the sidebar. Also the fee
//! types' colors, for the fee breakdown view.

use broker_fees::describe::{self, FeesFor};
use broker_fees::simulation::Fees;
use broker_fees::{Broker, Exchange, Plan, Security};
use eframe::egui::{self, Color32, RichText};
use rust_decimal::Decimal;

use super::widgets::color_mark;

/// The kinds of fee, in the order they're stacked, with their colors: four
/// clearly different hues, readable on the dark background.
pub const FEE_TYPES: [(&str, Color32); 4] = [
    ("Purchases", Color32::from_rgb(91, 141, 239)),
    ("Conversions", Color32::from_rgb(46, 196, 182)),
    ("Custody", Color32::from_rgb(242, 143, 59)),
    ("Selling", Color32::from_rgb(224, 93, 123)),
];

/// `fees` in the order of `FEE_TYPES`.
pub fn by_type(fees: &Fees) -> [Decimal; 4] {
    [fees.purchases, fees.conversions, fees.custody, fees.selling]
}

// ─────────────────────────── Sidebar preview ───────────────────────────

/// A short preview of a plan, when hovering it in the sidebar.
pub fn plan_preview(ui: &mut egui::Ui, plan: &Plan, security: Security, exchange: Exchange) {
    ui.set_max_width(400.0);
    ui.strong(&plan.name);
    ui.label(&plan.description);
    ui.add_space(4.0);
    let fees = plan.describe_fees_for(security, exchange);
    fees_for_inputs(ui, &plan.name, &fees, security, exchange);
    ui.add_space(4.0);
    ui.label(
        RichText::new("ℹ shows the full details and caveats")
            .weak()
            .small(),
    );
}

// ─────────────────────────── Details window ───────────────────────────

/// Everything about one plan: what it is, what it charges, and what the
/// tariff leaves unclear.
pub fn plan_details(
    ui: &mut egui::Ui,
    broker: &Broker,
    plan: &Plan,
    color: Color32,
    security: Security,
    exchange: Exchange,
) {
    ui.horizontal(|ui| {
        color_mark(ui, color);
        ui.label(&broker.name);
        source_line(ui, broker);
    });
    ui.separator();
    ui.label(&plan.description);
    ui.add_space(8.0);
    let fees = broker.describe_fees_for(plan, security, exchange);
    fees_for_inputs(ui, &plan.name, &fees, security, exchange);
    notes(ui, &fees.caveats);
    ui.add_space(12.0);
    egui::CollapsingHeader::new("Full tariff").show(ui, |ui| full_tariff(ui, plan));
}

/// A broker, and each of its plans in a sentence.
pub fn broker_details(ui: &mut egui::Ui, broker: &Broker, colors: &[Color32]) {
    ui.horizontal(|ui| source_line(ui, broker));
    ui.separator();
    ui.label(&broker.description);
    notes(ui, broker.caveats.iter().map(|caveat| &caveat.text));
    ui.add_space(8.0);
    for (plan, color) in broker.plans.iter().zip(colors) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            color_mark(ui, *color);
            ui.strong(&plan.name);
        });
        ui.label(&plan.description);
    }
}

/// The tariff's date, and a link to it.
fn source_line(ui: &mut egui::Ui, broker: &Broker) {
    ui.label(RichText::new(broker.tariff_date_text().to_string()).weak());
    if let Some(url) = &broker.source_url {
        ui.hyperlink_to("Tariff (PDF)", url);
    }
}

/// The trade, custody and conversion fees that apply to `security` on
/// `exchange`.
fn fees_for_inputs(
    ui: &mut egui::Ui,
    plan_name: &str,
    fees: &FeesFor,
    security: Security,
    exchange: Exchange,
) {
    let purchase = describe::purchase(security, exchange);
    ui.label(RichText::new(format!("For {purchase}:")).weak());
    egui::Grid::new(("fees for inputs", plan_name))
        .num_columns(2)
        .spacing([12.0, 2.0])
        .show(ui, |ui| {
            for fee in &fees.fees {
                ui.label(&fee.name).on_hover_text(&fee.explanation);
                ui.label(&fee.price);
                ui.end_row();
            }
        });
}

pub fn notes<'a>(ui: &mut egui::Ui, notes: impl IntoIterator<Item = &'a String>) {
    for note in notes {
        ui.add_space(2.0);
        ui.horizontal_top(|ui| {
            ui.label("⚠");
            ui.add(egui::Label::new(RichText::new(note).weak()).wrap());
        });
    }
}

/// Every row of the plan's tariff, not just the ones that apply.
fn full_tariff(ui: &mut egui::Ui, plan: &Plan) {
    ui.strong("Buying and selling");
    egui::Grid::new(("tariff trading", &plan.name))
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            for row in &plan.trading {
                ui.label(row.coverage().to_string());
                ui.label(row.price.to_string());
                ui.end_row();
            }
        });

    ui.add_space(8.0);
    ui.strong("Custody");
    egui::Grid::new(("tariff custody", &plan.name))
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            for row in &plan.custody {
                ui.label(row.coverage().to_string());
                ui.label(row.to_string());
                ui.end_row();
            }
        });

    ui.add_space(8.0);
    ui.strong("Conversion");
    ui.label(format!(
        "{}; markup {}",
        plan.conversion, plan.conversion.markup
    ));
}
