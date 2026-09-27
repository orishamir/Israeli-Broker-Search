//! Small widgets and number formatting shared by the sidebar, the table and
//! the chart.

use broker_fees::{Exchange, Security};
use eframe::egui::{self, Color32, RichText};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// A titled box around a group of inputs.
pub fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).strong());
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        add_contents(ui);
    });
    ui.add_space(12.0);
}

/// egui's own checkbox, drawn in a plan's chart color: filled when checked,
/// just an outline when not.
pub fn colored_checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    text: &str,
    color: Color32,
) -> egui::Response {
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        // The checkbox takes its label's color from the same setting as its
        // tick, so keep the label in the normal text color.
        visuals.override_text_color = Some(visuals.text_color());
        let widgets = &mut visuals.widgets;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            state.bg_fill = if *checked {
                color
            } else {
                Color32::TRANSPARENT
            };
            state.bg_stroke = egui::Stroke::new(1.5, color);
            state.fg_stroke.color = readable_on(color); // the tick
        }
        ui.checkbox(checked, text)
    })
    .inner
}

/// A small square in a plan's color, matching its checkbox and chart line.
pub fn color_mark(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, color);
}

/// Black or white, whichever is easier to read on `background`.
pub fn readable_on(background: Color32) -> Color32 {
    if background.intensity() > 0.55 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

pub fn shekel_input(value: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .speed(100.0)
        .range(0.0..=100_000_000.0)
        .prefix("₪ ")
        .custom_formatter(|amount, _decimals| with_commas(amount.round() as u64))
        .custom_parser(|text| text.replace(',', "").trim().parse().ok())
}

pub fn rate_input(value: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .speed(0.01)
        .range(0.01..=100.0)
        .max_decimals(4)
        .prefix("₪ ")
}

/// "₪1.43M", "₪99k", "₪4.2k": short enough to fit next to the chart.
pub fn compact_shekels(amount: f64) -> String {
    let sign = if amount < 0.0 { "-" } else { "" };
    let amount = amount.abs();
    if amount >= 1_000_000.0 {
        format!("{sign}₪{:.2}M", amount / 1_000_000.0)
    } else if amount >= 10_000.0 {
        format!("{sign}₪{:.0}k", amount / 1_000.0)
    } else if amount >= 1_000.0 {
        format!("{sign}₪{:.1}k", amount / 1_000.0)
    } else {
        format!("{sign}₪{amount:.0}")
    }
}

/// "₪1,234,567", rounded to whole shekels.
pub fn shekels(amount: f64) -> String {
    let rounded = amount.round();
    let sign = if rounded < 0.0 { "-" } else { "" };
    format!("{sign}₪{}", with_commas(rounded.abs() as u64))
}

/// An amount as a tariff writes it: "$6,750", "$5.76", "₪3.5".
pub fn money_text(money: money2::Money) -> String {
    let symbol = match money.currency {
        money2::Currency::Ils => "₪",
        money2::Currency::Usd => "$",
        money2::Currency::Eur => "€",
        _ => "",
    };
    let text = money.amount.normalize().to_string();
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    let whole = with_commas(whole.parse().unwrap_or(0));
    if fraction.is_empty() {
        format!("{symbol}{whole}")
    } else {
        format!("{symbol}{whole}.{fraction}")
    }
}

/// "0.15%"
pub fn percent_text(percent: Decimal) -> String {
    format!("{}%", percent.normalize())
}

/// "1,234,567"
fn with_commas(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

pub fn to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or_default()
}

pub fn security_name(security: Security) -> &'static str {
    match security {
        Security::Etf => "ETF",
        Security::MutualFund => "Mutual fund",
        Security::Bond => "Bond",
        Security::Stock => "Stock",
    }
}

pub fn exchange_name(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Tlv => "Tel Aviv",
        Exchange::Usa => "USA",
        Exchange::Europe => "Europe",
    }
}
