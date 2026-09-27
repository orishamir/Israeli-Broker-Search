//! A window for comparing what each broker's fees do to an investment over the
//! years. The numbers come from `broker_fees::simulation`; this file is only
//! the user interface.

use std::sync::mpsc::{self, Receiver};

use broker_fees::simulation::{self, Outcome, Scenario};
use broker_fees::{Broker, Exchange, Security, exchange_rates, tariffs};
use eframe::egui::{self, Color32, RichText};
use egui_plot::{Legend, Line, LineStyle, Plot};
use money2::{Currency, ExchangeRates};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Broker fee comparison")
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "broker_fees",
        options,
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}

struct App {
    inputs: Inputs,
    brokers: Vec<BrokerChoice>,
    rates: RatesInput,
}

/// The user's inputs, as the widgets edit them.
struct Inputs {
    security: Security,
    exchange: Exchange,
    first_deposit: f64,
    monthly_deposit: f64,
    yearly_return_percent: f64,
    years: u32,
    buy_every_months: u32,
    /// In the exchange's currency.
    share_price: f64,
}

struct BrokerChoice {
    broker: Broker,
    selected: bool,
}

/// Shekel prices of a dollar and a euro. Start as defaults, get replaced by
/// the ECB's rates once they download, and the user can edit them any time.
struct RatesInput {
    ils_per_usd: f64,
    ils_per_eur: f64,
    status: RatesStatus,
    download: Option<Receiver<Result<(f64, f64), String>>>,
}

enum RatesStatus {
    Downloading,
    Downloaded,
    Failed(String),
}

/// One plan's result, ready to display.
struct PlanResult {
    name: String,
    /// `None` if the plan has no price for the chosen security and exchange.
    outcome: Option<Outcome>,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        use_font_with_shekel_sign(ctx);
        Self {
            inputs: Inputs {
                security: Security::Etf,
                exchange: Exchange::Usa,
                first_deposit: 10_000.0,
                monthly_deposit: 2_000.0,
                yearly_return_percent: 10.0,
                years: 20,
                buy_every_months: 1,
                share_price: 500.0,
            },
            brokers: [tariffs::altshuler(), tariffs::leumi()]
                .into_iter()
                .map(|broker| BrokerChoice {
                    broker,
                    selected: true,
                })
                .collect(),
            rates: RatesInput {
                ils_per_usd: 3.7,
                ils_per_eur: 4.3,
                status: RatesStatus::Downloading,
                download: Some(download_rates(ctx.clone())),
            },
        }
    }

    /// Picks up the downloaded rates once they arrive.
    fn check_rates_download(&mut self) {
        let Some(download) = &self.rates.download else {
            return;
        };
        let Ok(result) = download.try_recv() else {
            return; // still downloading
        };
        match result {
            Ok((ils_per_usd, ils_per_eur)) => {
                self.rates.ils_per_usd = ils_per_usd;
                self.rates.ils_per_eur = ils_per_eur;
                self.rates.status = RatesStatus::Downloaded;
            }
            Err(error) => self.rates.status = RatesStatus::Failed(error),
        }
        self.rates.download = None;
    }

    fn scenario(&self) -> Scenario {
        let i = &self.inputs;
        Scenario {
            security: i.security,
            exchange: i.exchange,
            first_deposit: decimal(i.first_deposit),
            monthly_deposit: decimal(i.monthly_deposit),
            yearly_return_percent: decimal(i.yearly_return_percent),
            years: i.years,
            buy_every_months: i.buy_every_months,
            share_price: decimal(i.share_price),
        }
    }

    /// Runs the simulation for every plan of every selected broker, plus a
    /// plan with no fees to compare against.
    fn compare(&self) -> (Outcome, Vec<PlanResult>) {
        let scenario = self.scenario();
        let rates = exchange_rates(
            decimal(self.rates.ils_per_usd),
            decimal(self.rates.ils_per_eur),
        );
        let no_fees = simulation::simulate(&simulation::free_plan(), &scenario, &rates)
            .expect("the free plan covers every trade");

        let mut results: Vec<PlanResult> = self
            .brokers
            .iter()
            .filter(|choice| choice.selected)
            .flat_map(|choice| {
                choice.broker.plans.iter().map(|plan| PlanResult {
                    name: format!("{} – {}", choice.broker.name, plan.name),
                    outcome: simulation::simulate(plan, &scenario, &rates),
                })
            })
            .collect();

        // Best first; plans that don't offer this trade go last.
        results.sort_by_key(|r| std::cmp::Reverse(r.outcome.as_ref().map(|o| o.after_selling)));
        (no_fees, results)
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.check_rates_download();

        egui::Panel::left("inputs")
            .resizable(false)
            .exact_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.inputs_ui(ui));
            });

        egui::CentralPanel::default().show(ui, |ui| self.results_ui(ui));
    }
}

// ─────────────────────────── Inputs panel ───────────────────────────

impl App {
    fn inputs_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.heading("Your investment");
        ui.add_space(8.0);

        section(ui, "What you buy", |ui| {
            ui.horizontal_wrapped(|ui| {
                for security in [
                    Security::Etf,
                    Security::MutualFund,
                    Security::Bond,
                    Security::Stock,
                ] {
                    ui.selectable_value(
                        &mut self.inputs.security,
                        security,
                        security_name(security),
                    );
                }
            });
            ui.add_space(4.0);
            ui.label("Traded on");
            ui.horizontal_wrapped(|ui| {
                for exchange in [Exchange::Tlv, Exchange::Usa, Exchange::Europe] {
                    ui.selectable_value(
                        &mut self.inputs.exchange,
                        exchange,
                        exchange_name(exchange),
                    );
                }
            });
        });

        section(ui, "Deposits", |ui| {
            egui::Grid::new("deposits")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("First deposit");
                    ui.add(shekel_input(&mut self.inputs.first_deposit));
                    ui.end_row();

                    ui.label("Every month");
                    ui.add(shekel_input(&mut self.inputs.monthly_deposit));
                    ui.end_row();

                    ui.label("Buy every").on_hover_text(
                        "Deposits wait as cash until the next purchase. Buying less \
                         often means paying fewer minimum fees, but the cash doesn't \
                         grow while it waits.",
                    );
                    egui::ComboBox::from_id_salt("buy_every")
                        .selected_text(months_name(self.inputs.buy_every_months))
                        .show_ui(ui, |ui| {
                            for months in [1, 2, 3, 6, 12] {
                                ui.selectable_value(
                                    &mut self.inputs.buy_every_months,
                                    months,
                                    months_name(months),
                                );
                            }
                        });
                    ui.end_row();
                });
        });

        section(ui, "Expectations", |ui| {
            egui::Grid::new("expectations")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Yearly return").on_hover_text(
                        "In the security's own currency. The S&P 500 has averaged about 10%.",
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.inputs.yearly_return_percent)
                            .speed(0.1)
                            .range(-50.0..=50.0)
                            .max_decimals(1)
                            .suffix(" %"),
                    );
                    ui.end_row();

                    ui.label("Years");
                    ui.add(egui::Slider::new(&mut self.inputs.years, 1..=50));
                    ui.end_row();

                    // Only US plans charge per share.
                    if self.inputs.exchange == Exchange::Usa {
                        ui.label("Share price").on_hover_text(
                            "Today's price of one share. Only matters for plans that \
                             charge per share. It grows with the yearly return.",
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.inputs.share_price)
                                .speed(1.0)
                                .range(0.01..=100_000.0)
                                .prefix("$ "),
                        );
                        ui.end_row();
                    }
                });
        });

        if self.inputs.exchange != Exchange::Tlv {
            section(ui, "Exchange rates", |ui| self.rates_ui(ui));
        }

        section(ui, "Brokers to compare", |ui| {
            for choice in &mut self.brokers {
                let plans = choice.broker.plans.len();
                ui.checkbox(
                    &mut choice.selected,
                    format!("{}  ({plans} plans)", choice.broker.name),
                );
            }
        });
    }

    fn rates_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("rates")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("$1 =");
                ui.add(rate_input(&mut self.rates.ils_per_usd));
                ui.end_row();

                ui.label("€1 =");
                ui.add(rate_input(&mut self.rates.ils_per_eur));
                ui.end_row();
            });

        let status = match &self.rates.status {
            RatesStatus::Downloading => "Downloading today's rates…".to_owned(),
            RatesStatus::Downloaded => "Today's European Central Bank rates.".to_owned(),
            RatesStatus::Failed(error) => {
                format!("Couldn't download today's rates ({error}). Check these defaults.")
            }
        };
        ui.label(RichText::new(status).small().weak());
    }
}

// ─────────────────────────── Results ───────────────────────────

impl App {
    fn results_ui(&self, ui: &mut egui::Ui) {
        let (no_fees, results) = self.compare();

        ui.add_space(8.0);
        ui.heading(format!("After {} years", self.inputs.years));
        let deposited = self.inputs.first_deposit
            + self.inputs.monthly_deposit * 12.0 * f64::from(self.inputs.years);
        ui.label(format!(
            "You deposit {} in total. With no fees at all it would grow to {}.",
            shekels(deposited),
            shekels(to_f64(no_fees.after_selling)),
        ));
        ui.add_space(12.0);

        if results.is_empty() {
            ui.label("Tick a broker on the left to compare.");
            return;
        }

        results_table(ui, &no_fees, &results);
        ui.add_space(16.0);
        growth_chart(ui, &no_fees, &results);
    }
}

fn results_table(ui: &mut egui::Ui, no_fees: &Outcome, results: &[PlanResult]) {
    egui::Grid::new("results")
        .striped(true)
        .num_columns(5)
        .spacing([24.0, 6.0])
        .show(ui, |ui| {
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

            for (rank, result) in results.iter().enumerate() {
                let name = RichText::new(&result.name);
                let Some(outcome) = &result.outcome else {
                    ui.label(name.weak());
                    ui.label(RichText::new("doesn't offer this security on this exchange").weak());
                    ui.end_row();
                    continue;
                };
                let is_best = rank == 0;
                ui.label(if is_best {
                    name.strong().color(Color32::from_rgb(60, 170, 90))
                } else {
                    name
                });
                amount_cell(ui, outcome.held);
                amount_cell(ui, outcome.after_selling);
                amount_cell(ui, outcome.fees_paid);
                amount_cell(ui, no_fees.after_selling - outcome.after_selling);
                ui.end_row();
            }
        });
}

fn growth_chart(ui: &mut egui::Ui, no_fees: &Outcome, results: &[PlanResult]) {
    Plot::new("growth")
        .legend(Legend::default())
        .x_axis_label("Years")
        .y_axis_label("Value held")
        .y_axis_formatter(|mark, _range| shekels(mark.value))
        .label_formatter(|position| {
            let egui_plot::HoverPosition::NearDataPoint {
                plot_name,
                position,
                ..
            } = position
            else {
                return None;
            };
            Some(format!(
                "{plot_name}\nYear {:.0}: {}",
                position.x,
                shekels(position.y)
            ))
        })
        .include_y(0.0)
        .allow_scroll(false)
        .show(ui, |plot| {
            plot.line(
                Line::new("No fees", yearly_points(no_fees))
                    .style(LineStyle::dashed_loose())
                    .color(Color32::GRAY),
            );
            for result in results {
                if let Some(outcome) = &result.outcome {
                    plot.line(Line::new(result.name.clone(), yearly_points(outcome)).width(2.0));
                }
            }
        });
}

fn yearly_points(outcome: &Outcome) -> Vec<[f64; 2]> {
    outcome
        .value_by_year
        .iter()
        .enumerate()
        .map(|(year, value)| [year as f64, to_f64(*value)])
        .collect()
}

// ─────────────────────────── Widgets and formatting ───────────────────────────

/// A titled box around a group of inputs.
fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).strong());
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        add_contents(ui);
    });
    ui.add_space(12.0);
}

fn shekel_input(value: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .speed(100.0)
        .range(0.0..=100_000_000.0)
        .prefix("₪ ")
        .custom_formatter(|amount, _decimals| with_commas(amount.round() as u64))
        .custom_parser(|text| text.replace(',', "").trim().parse().ok())
}

fn rate_input(value: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .speed(0.01)
        .range(0.01..=100.0)
        .max_decimals(4)
        .prefix("₪ ")
}

/// Padded in a monospace font, so columns of numbers line up on the right.
fn amount_cell(ui: &mut egui::Ui, amount: Decimal) {
    ui.monospace(format!("{:>12}", shekels(to_f64(amount))));
}

/// "₪1,234,567", rounded to whole shekels.
fn shekels(amount: f64) -> String {
    let rounded = amount.round();
    let sign = if rounded < 0.0 { "-" } else { "" };
    format!("{sign}₪{}", with_commas(rounded.abs() as u64))
}

/// "1,234,567"
fn with_commas(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

fn security_name(security: Security) -> &'static str {
    match security {
        Security::Etf => "ETF",
        Security::MutualFund => "Mutual fund",
        Security::Bond => "Bond",
        Security::Stock => "Stock",
    }
}

fn exchange_name(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Tlv => "Tel Aviv",
        Exchange::Usa => "USA",
        Exchange::Europe => "Europe",
    }
}

fn months_name(months: u32) -> String {
    match months {
        1 => "month".to_owned(),
        12 => "year".to_owned(),
        n => format!("{n} months"),
    }
}

fn decimal(value: f64) -> Decimal {
    Decimal::try_from(value).unwrap_or_default().round_dp(4)
}

fn to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or_default()
}

/// egui's default text font has no ₪; its monospace font (Hack) does, so
/// text falls back to it for that one character.
fn use_font_with_shekel_sign(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("Hack".to_owned());
    ctx.set_fonts(fonts);
}

/// Downloads today's rates on a background thread. The receiver gets
/// (₪ per $, ₪ per €) or an error message.
fn download_rates(ctx: egui::Context) -> Receiver<Result<(f64, f64), String>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())
            .and_then(|runtime| {
                runtime
                    .block_on(ExchangeRates::new())
                    .map_err(|e| e.to_string())
            })
            .and_then(|rates| {
                let ils_per = |currency| {
                    rates
                        .get(&currency, &Currency::Ils)
                        .map(to_f64)
                        .ok_or("the ECB's rates are missing the shekel".to_owned())
                };
                Ok((ils_per(Currency::Usd)?, ils_per(Currency::Eur)?))
            });
        let _ = sender.send(result);
        ctx.request_repaint(); // so the window shows the new rates right away
    });
    receiver
}
