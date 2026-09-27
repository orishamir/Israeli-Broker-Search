//! A window for comparing what each broker's fees do to an investment over the
//! years. The numbers come from `broker_fees::simulation`; this binary is only
//! the user interface. This file has the app's state and the sidebar; the
//! results table and chart are in `src/gui/`.

mod gui {
    pub mod chart;
    pub mod plan_info;
    pub mod table;
    pub mod widgets;
}

use std::collections::HashSet;

use broker_fees::simulation::{self, Outcome, Scenario};
use broker_fees::{Broker, Exchange, Plan, Security, exchange_rates, tariffs};
use eframe::egui::{self, Color32, RichText};
use gui::chart::{ChartState, ChartView, growth_chart};
use gui::plan_info::{details_contents, hover_card, notes, tariff_date_text};
use gui::table::results_table;
use gui::widgets::{
    colored_checkbox, exchange_name, rate_input, section, security_name, shekel_input, shekels,
    to_f64,
};
use money2::{Currency, ExchangeRates};
use poll_promise::Promise;
use rust_decimal::Decimal;

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
    chart: ChartState,
    /// Plans the user clicked in the table or the chart. They stay
    /// highlighted until clicked again.
    pinned_plans: HashSet<PlanKey>,
    /// The plan under the mouse, in the sidebar, the table or the chart.
    /// Highlighted like the pinned plans while the mouse is there. Found while
    /// drawing a frame, so it takes effect on the next one.
    hovered_plan: Option<PlanKey>,
    /// The plan shown in the details window, if it's open.
    details_plan: Option<PlanKey>,
}

/// What the mouse did to the plans in the table or the chart.
#[derive(Default)]
struct PlanInteraction {
    hovered: Option<PlanKey>,
    clicked: Option<PlanKey>,
    /// Asked for the plan's details window.
    details: Option<PlanKey>,
}

/// Which plan of which broker, by position in `App::brokers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PlanKey {
    broker: usize,
    plan: usize,
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
    /// One per plan, in the same order as `broker.plans`.
    plans: Vec<PlanChoice>,
}

struct PlanChoice {
    selected: bool,
    /// The plan's line in the chart, and its swatch next to the checkbox.
    color: Color32,
}

/// The Okabe-Ito palette: distinguishable with color blindness, and readable
/// on both dark and light backgrounds.
const PLAN_COLORS: [Color32; 8] = [
    Color32::from_rgb(86, 180, 233),
    Color32::from_rgb(230, 159, 0),
    Color32::from_rgb(0, 158, 115),
    Color32::from_rgb(204, 121, 167),
    Color32::from_rgb(240, 228, 66),
    Color32::from_rgb(213, 94, 0),
    Color32::from_rgb(0, 114, 178),
    Color32::from_rgb(170, 120, 255),
];

/// Shekel prices of a dollar and a euro. Start as defaults, get replaced by
/// the ECB's rates once they download, and the user can edit them any time.
struct RatesInput {
    ils_per_usd: f64,
    ils_per_eur: f64,
    status: RatesStatus,
}

enum RatesStatus {
    /// Gives (₪ per $, ₪ per €) or an error message when done.
    Downloading(Promise<Result<(f64, f64), String>>),
    Downloaded,
    Failed(String),
}

/// One plan's result, ready to display.
struct PlanResult {
    key: PlanKey,
    broker_name: String,
    /// Caveats that apply to every plan of the broker.
    broker_notes: Vec<String>,
    plan: Plan,
    /// "Broker – Plan"
    name: String,
    color: Color32,
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
            brokers: broker_choices(vec![tariffs::altshuler(), tariffs::leumi()]),
            rates: RatesInput {
                ils_per_usd: 3.7,
                ils_per_eur: 4.3,
                status: RatesStatus::Downloading(download_rates(ctx.clone())),
            },
            chart: ChartState::default(),
            pinned_plans: HashSet::new(),
            hovered_plan: None,
            details_plan: None,
        }
    }

    /// Picks up the downloaded rates once they arrive.
    fn check_rates_download(&mut self) {
        let RatesStatus::Downloading(download) = &self.rates.status else {
            return;
        };
        let Some(result) = download.ready() else {
            return; // still downloading
        };
        self.rates.status = match result.clone() {
            Ok((ils_per_usd, ils_per_eur)) => {
                self.rates.ils_per_usd = ils_per_usd;
                self.rates.ils_per_eur = ils_per_eur;
                RatesStatus::Downloaded
            }
            Err(error) => RatesStatus::Failed(error),
        };
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

        let mut results = Vec::new();
        for (broker_index, choice) in self.brokers.iter().enumerate() {
            let plans = choice.broker.plans.iter().zip(&choice.plans);
            for (plan_index, (plan, plan_choice)) in plans.enumerate() {
                if !plan_choice.selected {
                    continue;
                }
                results.push(PlanResult {
                    key: PlanKey {
                        broker: broker_index,
                        plan: plan_index,
                    },
                    broker_name: choice.broker.name.clone(),
                    broker_notes: choice.broker.notes.clone(),
                    plan: plan.clone(),
                    name: format!("{} – {}", choice.broker.name, plan.name),
                    color: plan_choice.color,
                    outcome: simulation::simulate(plan, &scenario, &rates),
                });
            }
        }

        // Best first; plans that don't offer this trade go last.
        results.sort_by_key(|r| std::cmp::Reverse(r.outcome.as_ref().map(|o| o.after_selling)));
        (no_fees, results)
    }
}

impl App {
    /// R or Home resets the chart's zoom and Esc closes the details window,
    /// unless a text field is being typed in.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let no_modifiers = egui::Modifiers::NONE;
        if ctx.input_mut(|input| {
            input.consume_key(no_modifiers, egui::Key::R)
                || input.consume_key(no_modifiers, egui::Key::Home)
        }) {
            self.chart.reset_zoom();
        }
        if ctx.input_mut(|input| input.consume_key(no_modifiers, egui::Key::Escape)) {
            self.details_plan = None;
        }
    }

    /// A window with everything about one plan, over the chart so the
    /// results keep their width.
    fn details_window(&mut self, ctx: &egui::Context, key: PlanKey) {
        let choice = &self.brokers[key.broker];
        let plan = &choice.broker.plans[key.plan];
        let rates = exchange_rates(
            decimal(self.rates.ils_per_usd),
            decimal(self.rates.ils_per_eur),
        );
        let outcome = simulation::simulate(plan, &self.scenario(), &rates);

        let mut open = true;
        egui::Window::new(format!("{} – {}", choice.broker.name, plan.name))
            .id(egui::Id::new("plan details")) // the same window for every plan
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .vscroll(true)
            .default_width(420.0)
            .default_height(600.0)
            .default_pos(ctx.content_rect().right_top() + egui::vec2(-460.0, 60.0))
            .show(ctx, |ui| {
                details_contents(
                    ui,
                    &choice.broker,
                    plan,
                    choice.plans[key.plan].color,
                    self.inputs.security,
                    self.inputs.exchange,
                    outcome.as_ref(),
                );
            });
        if !open {
            self.details_plan = None;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.check_rates_download();
        self.handle_shortcuts(ui.ctx());
        let last_hovered = self.hovered_plan.take();
        let mut highlighted = self.pinned_plans.clone();
        highlighted.extend(last_hovered);

        egui::Panel::left("inputs")
            .resizable(false)
            .exact_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.inputs_ui(ui));
            });

        egui::CentralPanel::default().show(ui, |ui| self.results_ui(ui, &highlighted));

        if let Some(key) = self.details_plan {
            self.details_window(ui.ctx(), key);
        }

        // The hover found this frame is drawn next frame; don't wait for the
        // mouse to move again to draw it.
        if self.hovered_plan != last_hovered {
            ui.ctx().request_repaint();
        }
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
            let inputs = (self.inputs.security, self.inputs.exchange);
            for (broker_index, choice) in self.brokers.iter_mut().enumerate() {
                if let Some(plan_index) = broker_checkboxes(ui, choice, inputs) {
                    self.hovered_plan = Some(PlanKey {
                        broker: broker_index,
                        plan: plan_index,
                    });
                }
                ui.add_space(4.0);
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
            RatesStatus::Downloading(_) => "Downloading today's rates…".to_owned(),
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
    /// `highlighted`: the plans to make stand out, in the table and the chart.
    fn results_ui(&mut self, ui: &mut egui::Ui, highlighted: &HashSet<PlanKey>) {
        let (no_fees, results) = self.compare();
        // Forget pins on plans that were unticked since.
        self.pinned_plans
            .retain(|key| results.iter().any(|result| result.key == *key));

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

        let inputs = (self.inputs.security, self.inputs.exchange);
        let table = results_table(ui, &no_fees, &results, highlighted, inputs);
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.chart.view, ChartView::Value, "Value over time");
            ui.selectable_value(&mut self.chart.view, ChartView::LostToFees, "Lost to fees")
                .on_hover_text(
                    "How much less each plan has than an account with no fees, \
                     year by year. Shows where plans overtake each other.",
                );
            ui.add_space(16.0);
            if self.pinned_plans.is_empty() {
                ui.label(RichText::new("Click a row or a line to pin it").weak());
            } else if ui.small_button("Unpin all").clicked() {
                self.pinned_plans.clear();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("Wheel: zoom years · Drag: move · R: reset").weak());
            });
        });
        let chart = growth_chart(ui, &mut self.chart, &no_fees, &results, highlighted);

        self.hovered_plan = self.hovered_plan.or(table.hovered).or(chart.hovered);
        if table.details.is_some() {
            self.details_plan = table.details;
        }
        for clicked in [table.clicked, chart.clicked].into_iter().flatten() {
            // Clicking a pinned plan unpins it.
            if !self.pinned_plans.remove(&clicked) {
                self.pinned_plans.insert(clicked);
            }
        }
    }
}

// ─────────────────────────── Sidebar helpers ───────────────────────────

/// Gives every plan of every broker its own color, in order.
fn broker_choices(brokers: Vec<Broker>) -> Vec<BrokerChoice> {
    // Cycles if there are ever more plans than colors.
    let mut colors = PLAN_COLORS.iter().copied().cycle();
    brokers
        .into_iter()
        .map(|broker| BrokerChoice {
            plans: broker
                .plans
                .iter()
                .map(|_| PlanChoice {
                    selected: true,
                    color: colors.next().expect("cycle never ends"),
                })
                .collect(),
            broker,
        })
        .collect()
}

/// A checkbox for the broker, which ticks or unticks all its plans, and an
/// indented checkbox per plan with the plan's chart color. Hovering a plan
/// shows what it charges for `inputs`. Returns the index of the plan under
/// the mouse.
fn broker_checkboxes(
    ui: &mut egui::Ui,
    choice: &mut BrokerChoice,
    (security, exchange): (Security, Exchange),
) -> Option<usize> {
    let all = choice.plans.iter().all(|plan| plan.selected);
    let some = choice.plans.iter().any(|plan| plan.selected);
    let mut checked = all;
    let broker_box = egui::Checkbox::new(&mut checked, RichText::new(&choice.broker.name).strong())
        .indeterminate(some && !all);
    if ui.add(broker_box).changed() {
        for plan in &mut choice.plans {
            plan.selected = checked;
        }
    }
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tariff_date_text(&choice.broker))
                .weak()
                .small(),
        );
        if !choice.broker.notes.is_empty() {
            ui.label(RichText::new("⚠").weak().small())
                .on_hover_ui(|ui| notes(ui, &choice.broker.notes));
        }
    });

    let mut hovered = None;
    ui.indent(&choice.broker.name, |ui| {
        for (index, (plan, plan_choice)) in choice
            .broker
            .plans
            .iter()
            .zip(&mut choice.plans)
            .enumerate()
        {
            let response =
                colored_checkbox(ui, &mut plan_choice.selected, &plan.name, plan_choice.color)
                    .on_hover_ui(|ui| {
                        let name = format!("{} – {}", choice.broker.name, plan.name);
                        let broker_notes = &choice.broker.notes;
                        hover_card(ui, &name, broker_notes, plan, security, exchange, None);
                    });
            if response.hovered() {
                hovered = Some(index);
            }
        }
    });
    hovered
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

/// Downloads today's rates on a background thread.
fn download_rates(ctx: egui::Context) -> Promise<Result<(f64, f64), String>> {
    let (sender, promise) = Promise::new();
    std::thread::spawn(move || {
        sender.send(fetch_rates());
        ctx.request_repaint(); // so the window shows the new rates right away
    });
    promise
}

/// (₪ per $, ₪ per €) from the European Central Bank.
fn fetch_rates() -> Result<(f64, f64), String> {
    // money2 downloads with reqwest, which needs a tokio runtime.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let rates = runtime
        .block_on(ExchangeRates::new())
        .map_err(|e| e.to_string())?;
    let ils_per = |currency| {
        rates
            .get(&currency, &Currency::Ils)
            .map(to_f64)
            .ok_or("the ECB's rates are missing the shekel")
    };
    Ok((ils_per(Currency::Usd)?, ils_per(Currency::Eur)?))
}
