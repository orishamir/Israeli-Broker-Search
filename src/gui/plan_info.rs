//! What brokers and plans are and what they charge, as read from their
//! tariffs: the preview card and details window of the sidebar. Also the fee
//! types' colors, for the fee breakdown view.

use broker_fees::simulation::Fees;
use broker_fees::{Broker, ConversionFee, CustodyFee, Exchange, Period, Plan, Price, Security};
use eframe::egui::{self, Color32, RichText};
use rust_decimal::Decimal;

use super::widgets::{color_mark, exchange_name, money_text, percent_text, security_name};

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
    fees_for_inputs(ui, plan, security, exchange);
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
    fees_for_inputs(ui, plan, security, exchange);
    notes(ui, broker.notes.iter().chain(&plan.notes));
    ui.add_space(12.0);
    egui::CollapsingHeader::new("Full tariff").show(ui, |ui| full_tariff(ui, plan));
}

/// A broker, and each of its plans in a sentence.
pub fn broker_details(ui: &mut egui::Ui, broker: &Broker, colors: &[Color32]) {
    ui.horizontal(|ui| source_line(ui, broker));
    ui.separator();
    ui.label(&broker.description);
    notes(ui, &broker.notes);
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
    ui.label(RichText::new(tariff_date_text(broker)).weak());
    if let Some(url) = &broker.source_url {
        ui.hyperlink_to("Tariff (PDF)", url);
    }
}

/// "Tariff of 29/06/2026", from the ISO date the tariffs are stored with.
pub fn tariff_date_text(broker: &Broker) -> String {
    let date: Vec<&str> = broker.tariff_date.split('-').collect();
    match date[..] {
        [year, month, day] => format!("Tariff of {day}/{month}/{year}"),
        _ => format!("Tariff date {}", broker.tariff_date),
    }
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
