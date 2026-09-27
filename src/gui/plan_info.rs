//! What a plan charges and where a plan's fees went: the hover card, the
//! details panel, and the fee split bar in the results table.

use broker_fees::simulation::{Fees, Outcome};
use broker_fees::{Broker, ConversionFee, CustodyFee, Exchange, Period, Plan, Price, Security};
use eframe::egui::{self, Color32, RichText};
use egui_plot::{AxisHints, Bar, BarChart, Plot};
use rust_decimal::Decimal;

use super::widgets::{
    color_mark, compact_shekels, exchange_name, money_text, percent_text, security_name, shekels,
    to_f64,
};

/// The kinds of fee, in the order they're stacked, with their colors. Muted,
/// so they aren't mistaken for the plans' colors.
const FEE_TYPES: [(&str, Color32); 4] = [
    ("Purchases", Color32::from_rgb(91, 143, 201)),
    ("Conversions", Color32::from_rgb(171, 128, 212)),
    ("Custody", Color32::from_rgb(219, 150, 72)),
    ("Selling", Color32::from_rgb(150, 150, 150)),
];

/// `fees` in the order of `FEE_TYPES`.
fn by_type(fees: &Fees) -> [Decimal; 4] {
    [fees.purchases, fees.conversions, fees.custody, fees.selling]
}

// ─────────────────────────── Hover card ───────────────────────────

/// A short summary of a plan: the fees it charges for what the user buys,
/// what they came to in this simulation, and the tariff's caveats.
pub fn hover_card(
    ui: &mut egui::Ui,
    name: &str,
    broker_notes: &[String],
    plan: &Plan,
    security: Security,
    exchange: Exchange,
    outcome: Option<&Outcome>,
) {
    ui.set_max_width(440.0);
    ui.strong(name);
    ui.add_space(4.0);
    fees_for_inputs(ui, plan, security, exchange);
    if let Some(outcome) = outcome {
        ui.add_space(4.0);
        ui.label(format!(
            "Fees paid: {}",
            shekels(to_f64(outcome.fees.total()))
        ));
        fee_legend(ui, &outcome.fees);
    }
    notes(ui, broker_notes.iter().chain(&plan.notes));
    ui.add_space(4.0);
    ui.label(
        RichText::new("ℹ in the table shows the full tariff")
            .weak()
            .small(),
    );
}

/// The trade, custody and conversion fees that apply to `security` on
/// `exchange`.
fn fees_for_inputs(ui: &mut egui::Ui, plan: &Plan, security: Security, exchange: Exchange) {
    ui.label(RichText::new(format!("For {}:", purchase_phrase(security, exchange))).weak());
    egui::Grid::new(("fees for inputs", &plan.name))
        .num_columns(2)
        .spacing([12.0, 2.0])
        .show(ui, |ui| {
            ui.label("Buy or sell");
            ui.label(plan.trade_row(security, exchange).map_or_else(
                || "not offered".to_owned(),
                |row| describe_price(&row.price),
            ));
            ui.end_row();

            ui.label("Custody");
            ui.label(
                plan.custody_row(exchange)
                    .map_or_else(|| "none".to_owned(), describe_custody),
            );
            ui.end_row();

            if exchange != Exchange::Tlv {
                ui.label("Conversion");
                ui.label(describe_conversion(&plan.conversion));
                ui.end_row();
            }
        });
}

/// "an ETF bought in the USA", "a mutual fund bought in Tel Aviv".
fn purchase_phrase(security: Security, exchange: Exchange) -> String {
    let security = match security {
        Security::Etf => "an ETF",
        Security::MutualFund => "a mutual fund",
        Security::Bond => "a bond",
        Security::Stock => "a stock",
    };
    let place = match exchange {
        Exchange::Tlv => "Tel Aviv",
        Exchange::Usa => "the USA",
        Exchange::Europe => "Europe",
    };
    format!("{security} bought in {place}")
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

// ─────────────────────────── Details panel ───────────────────────────

/// Everything about one plan, for the details window.
pub fn details_contents(
    ui: &mut egui::Ui,
    broker: &Broker,
    plan: &Plan,
    color: Color32,
    security: Security,
    exchange: Exchange,
    outcome: Option<&Outcome>,
) {
    ui.horizontal(|ui| {
        color_mark(ui, color);
        ui.label(&broker.name);
        ui.label(RichText::new(tariff_date_text(broker)).weak());
        if let Some(url) = &broker.source_url {
            ui.hyperlink_to("Tariff (PDF)", url);
        }
    });
    ui.separator();

    fees_for_inputs(ui, plan, security, exchange);
    notes(ui, broker.notes.iter().chain(&plan.notes));

    if let Some(outcome) = outcome {
        ui.add_space(12.0);
        ui.strong("Where the fees went");
        ui.horizontal(|ui| {
            fee_donut(ui, &outcome.fees);
            ui.add_space(12.0);
            fee_legend(ui, &outcome.fees);
        });

        ui.add_space(12.0);
        ui.strong("Fees each year");
        ui.label(
            RichText::new("Selling at the end isn't included.")
                .weak()
                .small(),
        );
        yearly_fees_chart(ui, plan, &outcome.fees_by_year);
    } else {
        ui.add_space(8.0);
        ui.label(RichText::new("This plan doesn't offer what you're buying.").weak());
    }

    ui.add_space(12.0);
    egui::CollapsingHeader::new("Full tariff").show(ui, |ui| full_tariff(ui, plan));
}

/// "Tariff of 29/06/2026", from the ISO date the tariffs are stored with.
pub fn tariff_date_text(broker: &Broker) -> String {
    let date: Vec<&str> = broker.tariff_date.split('-').collect();
    match date[..] {
        [year, month, day] => format!("Tariff of {day}/{month}/{year}"),
        _ => format!("Tariff date {}", broker.tariff_date),
    }
}

/// A ring split by fee type, with the total in the middle.
fn fee_donut(ui: &mut egui::Ui, fees: &Fees) {
    const RADIUS: f32 = 48.0;
    const THICKNESS: f32 = 18.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::Vec2::splat(2.0 * RADIUS + 4.0), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let total = to_f64(fees.total());

    // Each part is drawn as a thick arc, starting at the top and going clockwise.
    let mut angle = -std::f32::consts::FRAC_PI_2;
    for ((_, color), amount) in FEE_TYPES.iter().zip(by_type(fees)) {
        if total <= 0.0 {
            break;
        }
        let sweep = (to_f64(amount) / total) as f32 * std::f32::consts::TAU;
        let steps = (sweep * 20.0).ceil().max(1.0) as usize;
        let ring_radius = RADIUS - THICKNESS / 2.0;
        let points: Vec<egui::Pos2> = (0..=steps)
            .map(|step| {
                let a = angle + sweep * step as f32 / steps as f32;
                center + ring_radius * egui::vec2(a.cos(), a.sin())
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(THICKNESS, *color),
        ));
        angle += sweep;
    }
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        compact_shekels(total),
        egui::FontId::proportional(14.0),
        ui.visuals().strong_text_color(),
    );
}

/// Each fee type's color, amount and share of the total.
fn fee_legend(ui: &mut egui::Ui, fees: &Fees) {
    let total = to_f64(fees.total());
    egui::Grid::new(("fee legend", ui.next_auto_id()))
        .num_columns(4)
        .min_col_width(0.0)
        .spacing([8.0, 2.0])
        .show(ui, |ui| {
            for ((name, color), amount) in FEE_TYPES.iter().zip(by_type(fees)) {
                let amount = to_f64(amount);
                color_mark(ui, *color);
                ui.label(*name);
                ui.label(shekels(amount));
                let share = if total > 0.0 {
                    amount / total * 100.0
                } else {
                    0.0
                };
                ui.label(RichText::new(format!("{share:.0}%")).weak());
                ui.end_row();
            }
        });
}

/// The fees of each year, stacked by type.
fn yearly_fees_chart(ui: &mut egui::Ui, plan: &Plan, fees_by_year: &[Fees]) {
    let mut charts: Vec<BarChart> = Vec::new();
    for (type_index, (name, color)) in FEE_TYPES.iter().enumerate().take(3) {
        let bars = fees_by_year
            .iter()
            .enumerate()
            .map(|(year, fees)| {
                Bar::new((year + 1) as f64, to_f64(by_type(fees)[type_index])).fill(*color)
            })
            .collect();
        let others: Vec<&BarChart> = charts.iter().collect();
        let chart = BarChart::new(*name, bars)
            .color(*color)
            .width(0.7)
            .element_formatter(Box::new(move |bar, _chart| {
                format!("{name}\nYear {}: {}", bar.argument, shekels(bar.value))
            }))
            .stack_on(&others);
        charts.push(chart);
    }
    // egui_plot puts value labels at powers of ten and hides ones closer than
    // 20 points apart, which can leave a small chart with only "₪0".
    let value_axis = AxisHints::new_y()
        .formatter(|mark, _range| compact_shekels(mark.value))
        .label_spacing(12.0..=20.0)
        .min_thickness(48.0);
    Plot::new(("yearly fees", &plan.name))
        .height(200.0)
        .x_axis_label("Year")
        .custom_y_axes(vec![value_axis])
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show(ui, |plot| {
            for chart in charts {
                plot.bar_chart(chart);
            }
        });
}

/// Every row of the plan's tariff, not just the ones that apply.
fn full_tariff(ui: &mut egui::Ui, plan: &Plan) {
    ui.strong("Buying and selling");
    egui::Grid::new(("tariff trading", &plan.name))
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            for row in &plan.trading {
                ui.label(format!(
                    "{} on {}",
                    list_or_any(&row.securities, security_name, "Anything"),
                    list_or_any(&row.exchanges, exchange_name, "any exchange"),
                ));
                ui.label(describe_price(&row.price));
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
                ui.label(list_or_any(&row.exchanges, exchange_name, "Any exchange"));
                ui.label(describe_custody(row));
                ui.end_row();
            }
        });

    ui.add_space(8.0);
    ui.strong("Conversion");
    ui.label(describe_conversion(&plan.conversion));
}

/// "ETF, Stock", or `any` when the tariff row doesn't limit it.
fn list_or_any<T: Copy>(items: &[T], name: fn(T) -> &'static str, any: &str) -> String {
    if items.is_empty() {
        any.to_owned()
    } else {
        items
            .iter()
            .map(|item| name(*item))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// ─────────────────────────── Fee split bar ───────────────────────────

/// A bar split by fee type, as long as `fees` are relative to `largest`,
/// so the bars of different plans can be compared.
pub fn fee_split_bar(ui: &mut egui::Ui, fees: &Fees, largest: Decimal, max_width: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(max_width, 10.0), egui::Sense::hover());
    let largest = to_f64(largest);
    if largest <= 0.0 {
        return;
    }
    let mut left = rect.left();
    for ((_, color), amount) in FEE_TYPES.iter().zip(by_type(fees)) {
        let width = (to_f64(amount) / largest) as f32 * max_width;
        let part = egui::Rect::from_min_size(
            egui::pos2(left, rect.top()),
            egui::vec2(width, rect.height()),
        );
        ui.painter().rect_filled(part, 0.0, *color);
        left += width;
    }
}

/// The fee split bar's legend, for the table header.
pub fn fee_types_legend(ui: &mut egui::Ui) {
    for (name, color) in FEE_TYPES {
        ui.horizontal(|ui| {
            color_mark(ui, color);
            ui.label(name);
        });
    }
}

// ─────────────────────────── Describing fees ───────────────────────────

/// "0.3%, min $24, max $6,750", "$0.01 per share, min $9", "$4 per order".
fn describe_price(price: &Price) -> String {
    match price {
        Price::Percent { percent, min, max } => with_bounds(percent_text(*percent), *min, *max),
        Price::PerShare {
            per_share,
            min,
            max,
        } => with_bounds(format!("{} per share", money_text(*per_share)), *min, *max),
        Price::Flat(amount) => format!("{} per order", money_text(*amount)),
    }
}

/// "0.15% a quarter (0.6% a year)", "0.15% a year, charged monthly, min ₪75 a month".
fn describe_custody(row: &CustodyFee) -> String {
    let mut text = format!("{} a {}", percent_text(row.percent), period_name(row.per));
    if row.per != Period::Year {
        text += &format!(" ({} a year)", percent_text(row.percent_per_year()));
    }
    if row.billed != row.per {
        text += &format!(", charged {}", billing_name(row.billed));
    }
    if let Some(min) = row.min {
        text += &format!(", min {} a {}", money_text(min), period_name(row.billed));
    }
    text
}

/// "0.16%, min $5.76, max $2,400", "no fee, but up to 0.7% worse exchange rate".
fn describe_conversion(conversion: &ConversionFee) -> String {
    let fee = if conversion.percent.is_zero() {
        "no fee".to_owned()
    } else {
        with_bounds(
            percent_text(conversion.percent),
            conversion.min,
            conversion.max,
        )
    };
    if conversion.spread_percent.is_zero() {
        fee
    } else {
        format!(
            "{fee}, but up to {} worse exchange rate",
            percent_text(conversion.spread_percent)
        )
    }
}

fn with_bounds(text: String, min: Option<money2::Money>, max: Option<money2::Money>) -> String {
    let mut text = text;
    if let Some(min) = min {
        text += &format!(", min {}", money_text(min));
    }
    if let Some(max) = max {
        text += &format!(", max {}", money_text(max));
    }
    text
}

fn period_name(period: Period) -> &'static str {
    match period {
        Period::Month => "month",
        Period::Quarter => "quarter",
        Period::Year => "year",
    }
}

fn billing_name(period: Period) -> &'static str {
    match period {
        Period::Month => "monthly",
        Period::Quarter => "quarterly",
        Period::Year => "yearly",
    }
}
