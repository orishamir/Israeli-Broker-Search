//! The chart under the results table: one line per plan over the years, with
//! values written on it so they can be read without hovering.
//!
//! The chart handles the mouse itself instead of using `egui_plot`'s built-in
//! zoom, which needs Ctrl held down and can't keep the view within the years
//! simulated. Here the plain wheel zooms the years only, and the value axis
//! always fits the lines in view, so zooming can't end up in empty space.

use std::collections::HashSet;

use broker_fees::simulation::Outcome;
use eframe::egui::{self, Color32};
use egui_plot::{Line, LineStyle, Plot, PlotPoint, PlotTransform};

use super::widgets::{compact_shekels, readable_on, shekels, to_f64};
use crate::{PlanInteraction, PlanKey, PlanResult};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChartView {
    #[default]
    Value,
    /// How much less each plan has than a zero-fee account, year by year.
    /// Spreads the lines apart, since the differences are small next to the
    /// total value.
    LostToFees,
}

/// What the chart shows, kept between frames.
#[derive(Default)]
pub struct ChartState {
    pub view: ChartView,
    /// The years in view. `None` shows them all.
    zoom: Option<YearRange>,
}

impl ChartState {
    pub fn reset_zoom(&mut self) {
        self.zoom = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YearRange {
    pub start: f64,
    pub end: f64,
}

impl YearRange {
    fn width(self) -> f64 {
        self.end - self.start
    }

    /// Zoomed by `factor` (above 1 zooms in) keeping `center` in place.
    fn zoomed(self, factor: f64, center: f64) -> Self {
        Self {
            start: center - (center - self.start) / factor,
            end: center + (self.end - center) / factor,
        }
    }

    fn shifted(self, years: f64) -> Self {
        Self {
            start: self.start + years,
            end: self.end + years,
        }
    }

    /// Moved and shrunk to fit within year 0 to `last_year`, and at least a
    /// year wide.
    fn fit_within(self, last_year: f64) -> Self {
        let width = self.width().clamp(1.0_f64.min(last_year), last_year);
        let start = self.start.clamp(0.0, last_year - width);
        Self {
            start,
            end: start + width,
        }
    }
}

/// Seconds the value axis takes to follow the lines as the view moves, so it
/// doesn't jump. Longer feels like the zoom is being held back.
const VALUE_AXIS_ANIMATION: f32 = 0.05;

/// Room to the right of the chart for the values at the lines' ends.
const END_LABELS_WIDTH: f32 = 64.0;

/// One line per plan, in the plan's color, with its value at the right edge
/// of the view. Highlighted plans also show their value every few years.
pub fn growth_chart(
    ui: &mut egui::Ui,
    state: &mut ChartState,
    no_fees: &Outcome,
    results: &[PlanResult],
    highlighted: &HashSet<PlanKey>,
) -> PlanInteraction {
    let (y_label, hover_suffix) = match state.view {
        ChartView::Value => ("Value held", ""),
        ChartView::LostToFees => ("Lost to fees", " lost to fees"),
    };
    let last_year = (no_fees.value_by_year.len() - 1) as f64;
    let all_years = YearRange {
        start: 0.0,
        end: last_year,
    };
    let visible = state
        .zoom
        .map_or(all_years, |zoom| zoom.fit_within(last_year));

    let lines = chart_lines(state.view, no_fees, results, highlighted);
    let (value_min, value_max) = value_range(&lines, visible);
    // Zoomed out, the value axis starts at zero; zoomed in, it fits the lines.
    let value_min = if state.zoom.is_none() {
        value_min.min(0.0)
    } else {
        value_min
    };
    let value_padding = ((value_max - value_min) * 0.05).max(1.0);
    // Each view animates separately, so switching views jumps rather than slides.
    let animate = |end: &str, target: f64| {
        let id = egui::Id::new(("value axis", end, state.view));
        f64::from(
            ui.ctx()
                .animate_value_with_time(id, target as f32, VALUE_AXIS_ANIMATION),
        )
    };
    let axis_min = animate("min", value_min - value_padding);
    let axis_max = animate("max", value_max + value_padding);
    let year_padding = visible.width() * 0.01;

    let chart_width = ui.available_width() - END_LABELS_WIDTH;
    let response = Plot::new(("growth", state.view))
        .width(chart_width)
        .x_axis_label("Years")
        .y_axis_label(y_label)
        .y_axis_formatter(|mark, _range| shekels(mark.value))
        .label_formatter(move |position| {
            let egui_plot::HoverPosition::NearDataPoint {
                plot_name,
                position,
                ..
            } = position
            else {
                return None;
            };
            Some(format!(
                "{plot_name}\nYear {:.0}: {}{hover_suffix}",
                position.x,
                shekels(position.y)
            ))
        })
        // The mouse is handled below instead.
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .allow_axis_zoom_drag(false)
        .show(ui, |plot| {
            plot.set_plot_bounds_x(visible.start - year_padding..=visible.end + year_padding);
            plot.set_plot_bounds_y(axis_min..=axis_max);
            for line in &lines {
                plot.line(plot_line(line));
            }
            view_after_mouse(plot, visible)
        });

    // The mouse's effect is drawn next frame; ask for that frame right away
    // rather than waiting for the next mouse event, so moving feels direct.
    let new_zoom = Some(response.inner.fit_within(last_year)).filter(|zoom| *zoom != all_years);
    if new_zoom != state.zoom {
        ui.ctx().request_repaint();
    }
    state.zoom = new_zoom;

    // Drawn over the chart rather than as chart items, so the values get a
    // solid background and stay readable where they cross other lines.
    let highlighted_lines: Vec<&ChartLine> = lines.iter().filter(|line| line.highlighted).collect();
    let label_every = years_between_labels(chart_width, visible.width());
    draw_yearly_values(
        ui,
        &response.transform,
        &highlighted_lines,
        visible,
        label_every,
    );
    draw_end_labels(ui, &response.transform, &lines, visible.end, last_year);

    let mut interaction = PlanInteraction::default();
    if let Some(hovered_item) = response.hovered_plot_item {
        interaction.hovered = lines
            .iter()
            .find(|line| egui::Id::new(line.key) == hovered_item)
            .and_then(|line| line.key);
        if interaction.hovered.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    if response.response.clicked() {
        interaction.clicked = interaction.hovered;
    }
    interaction
}

/// The years to show next frame, after this frame's wheel and drag over the
/// chart.
fn view_after_mouse(plot: &egui_plot::PlotUi<'_>, visible: YearRange) -> YearRange {
    let response = plot.response();
    let mut view = visible;
    if response.hovered() {
        // Plain wheel, and also Ctrl+wheel and touchpad pinch, which egui
        // reports as zoom rather than scroll.
        let (scroll, zoom) = plot
            .ctx()
            .input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
        let factor = f64::from(zoom) * (f64::from(scroll) * 0.003).exp();
        if let Some(pointer) = plot.pointer_coordinate()
            && factor != 1.0
        {
            view = view.zoomed(factor, pointer.x);
        }
    }
    if response.dragged() {
        view = view.shifted(-f64::from(plot.pointer_coordinate_drag_delta().x));
    }
    view
}

/// One line in the chart. `key` is `None` for the no-fees line.
struct ChartLine {
    key: Option<PlanKey>,
    name: String,
    /// One per year, starting at year 0.
    values: Vec<f64>,
    /// Already faded if other plans are highlighted.
    color: Color32,
    highlighted: bool,
}

fn chart_lines(
    view: ChartView,
    no_fees: &Outcome,
    results: &[PlanResult],
    highlighted: &HashSet<PlanKey>,
) -> Vec<ChartLine> {
    let mut lines = Vec::new();
    if view == ChartView::Value {
        lines.push(ChartLine {
            key: None,
            name: "No fees".to_owned(),
            values: no_fees.value_by_year.iter().copied().map(to_f64).collect(),
            color: Color32::GRAY,
            highlighted: false,
        });
    }
    for result in results {
        let Some(outcome) = &result.outcome else {
            continue;
        };
        let values = match view {
            ChartView::Value => outcome.value_by_year.iter().copied().map(to_f64).collect(),
            ChartView::LostToFees => no_fees
                .value_by_year
                .iter()
                .zip(&outcome.value_by_year)
                .map(|(no_fees, plan)| to_f64(no_fees - plan))
                .collect(),
        };
        let is_highlighted = highlighted.contains(&result.key);
        let fade = !highlighted.is_empty() && !is_highlighted;
        lines.push(ChartLine {
            key: Some(result.key),
            name: result.name.clone(),
            values,
            color: if fade {
                result.color.gamma_multiply(0.3)
            } else {
                result.color
            },
            highlighted: is_highlighted,
        });
    }
    lines
}

fn plot_line(line: &ChartLine) -> Line<'static> {
    let points: Vec<[f64; 2]> = line
        .values
        .iter()
        .enumerate()
        .map(|(year, value)| [year as f64, *value])
        .collect();
    let is_no_fees_line = line.key.is_none();
    Line::new(line.name.clone(), points)
        .id(egui::Id::new(line.key))
        .color(line.color)
        .width(if line.highlighted { 3.5 } else { 2.0 })
        .style(if is_no_fees_line {
            LineStyle::dashed_loose()
        } else {
            LineStyle::Solid
        })
}

/// The lowest and highest value of any line within `years`. Uses the lines'
/// exact values at the view's edges, so the range changes smoothly as the
/// view moves.
fn value_range(lines: &[ChartLine], years: YearRange) -> (f64, f64) {
    let whole_years_inside = years.start.ceil() as usize..=years.end.floor() as usize;
    lines
        .iter()
        .flat_map(|line| {
            let inside = whole_years_inside
                .clone()
                .filter_map(|year| line.values.get(year).copied());
            let edges = [
                value_at(&line.values, years.start),
                value_at(&line.values, years.end),
            ];
            inside.chain(edges)
        })
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

/// A line's value at any point in time, between its yearly values in a
/// straight line, as the chart draws it.
fn value_at(values: &[f64], year: f64) -> f64 {
    let last = values.len() - 1;
    let year = year.clamp(0.0, last as f64);
    let before = (year.floor() as usize).min(last);
    let after = (before + 1).min(last);
    let fraction = year - before as f64;
    values[before] + (values[after] - values[before]) * fraction
}

/// Each line's value at `year` (the right edge of the view), in the margin
/// right of the chart and joined to the line. Labels are nudged apart when
/// the lines are close together. Lines that are above or below the view at
/// that year get no label.
fn draw_end_labels(
    ui: &egui::Ui,
    transform: &PlotTransform,
    lines: &[ChartLine],
    year: f64,
    last_year: f64,
) {
    let frame = *transform.frame();
    let label_x = frame.right() + 10.0;
    let painter = ui.painter();

    // When the view stops before the last year, say which year the labels are for.
    let mut labels_top = frame.top();
    if year < last_year - 0.05 {
        painter.text(
            egui::pos2(label_x, frame.top()),
            egui::Align2::LEFT_TOP,
            format!("Year {year:.1}"),
            egui::FontId::proportional(11.0),
            ui.visuals().weak_text_color(),
        );
        labels_top += 22.0;
    }

    let mut labels: Vec<(egui::Pos2, &ChartLine)> = lines
        .iter()
        .filter_map(|line| {
            let point = PlotPoint::new(year, value_at(&line.values, year));
            let position = transform.position_from_point(&point);
            frame
                .y_range()
                .contains(position.y)
                .then_some((position, line))
        })
        .collect();
    labels.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    let mut label_ys: Vec<f32> = labels.iter().map(|(position, _)| position.y).collect();
    spread_apart(&mut label_ys, 15.0, labels_top, frame.bottom());

    for ((point, line), label_y) in labels.iter().zip(label_ys) {
        let label_start = egui::pos2(label_x, label_y);
        painter.line_segment(
            [
                *point + egui::vec2(3.0, 0.0),
                label_start - egui::vec2(3.0, 0.0),
            ],
            egui::Stroke::new(1.0, line.color.gamma_multiply(0.6)),
        );
        let text = compact_shekels(value_at(&line.values, year));
        if line.highlighted {
            value_tag(
                painter,
                label_start,
                egui::Align2::LEFT_CENTER,
                text,
                line.color,
            );
        } else {
            painter.text(
                label_start,
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::proportional(12.0),
                line.color,
            );
        }
    }
}

/// A dot every `every` years on each highlighted line, with the value in a
/// tag above it. Tags of different lines in the same year are stacked so they
/// don't cover each other. Years right at the view's edge are left out: the
/// end labels show them.
fn draw_yearly_values(
    ui: &egui::Ui,
    transform: &PlotTransform,
    lines: &[&ChartLine],
    visible: YearRange,
    every: usize,
) {
    const TAG_HEIGHT: f32 = 17.0;
    let painter = ui.painter().with_clip_rect(*transform.frame());
    // The first multiple of `every` in view, skipping year 0 (the first deposit).
    let first_year = (visible.start.max(1.0) / every as f64).ceil() as usize * every;

    let before_edge = (visible.end - 0.25).max(0.0).ceil() as usize;
    for year in (first_year..before_edge).step_by(every) {
        let mut dots: Vec<(egui::Pos2, f64, Color32)> = lines
            .iter()
            .map(|line| {
                let value = line.values[year];
                let dot = transform.position_from_point(&PlotPoint::new(year as f64, value));
                (dot, value, line.color)
            })
            .collect();
        // Lowest on screen first, so each tag goes just above its dot or,
        // if that's taken, just above the previous tag.
        dots.sort_by(|a, b| b.0.y.total_cmp(&a.0.y));
        // All dots before any tag, so no dot covers another line's tag.
        for (dot, _, color) in &dots {
            painter.circle_filled(*dot, 4.0, *color);
        }
        let mut highest_free_y = f32::INFINITY;
        for (dot, value, color) in dots {
            let tag_bottom = (dot.y - 8.0).min(highest_free_y);
            value_tag(
                &painter,
                egui::pos2(dot.x, tag_bottom),
                egui::Align2::CENTER_BOTTOM,
                compact_shekels(value),
                color,
            );
            highest_free_y = tag_bottom - TAG_HEIGHT - 2.0;
        }
    }
}

/// `text` on a small rounded background of `color`, placed at `anchor`.
fn value_tag(
    painter: &egui::Painter,
    anchor: egui::Pos2,
    align: egui::Align2,
    text: String,
    color: Color32,
) {
    let padding = egui::vec2(4.0, 1.0);
    let galley = painter.layout_no_wrap(text, egui::FontId::proportional(12.0), readable_on(color));
    let rect = align.anchor_size(anchor, galley.size() + 2.0 * padding);
    painter.rect_filled(rect, 3.0, color);
    painter.galley(rect.min + padding, galley, color);
}

/// Moves the sorted positions `ys` apart until neighbours are at least
/// `min_gap` apart, staying between `top` and `bottom` where there's room.
fn spread_apart(ys: &mut [f32], min_gap: f32, top: f32, bottom: f32) {
    // Push down whatever is too close to the one above it...
    for i in 1..ys.len() {
        ys[i] = ys[i].max(ys[i - 1] + min_gap);
    }
    // ...then, if that ran past the bottom, push back up from there.
    if let Some(last) = ys.last_mut() {
        *last = last.min(bottom);
    }
    for i in (0..ys.len().saturating_sub(1)).rev() {
        ys[i] = ys[i].min(ys[i + 1] - min_gap);
    }
    for y in ys.iter_mut() {
        *y = y.max(top);
    }
}

/// Every how many years to put a value on a highlighted line so the values
/// don't overlap: 1, 2, 5 or 10.
fn years_between_labels(chart_width: f32, years_in_view: f64) -> usize {
    const LABEL_WIDTH: f32 = 56.0;
    let pixels_per_year = chart_width / years_in_view.max(1.0) as f32;
    [1, 2, 5, 10]
        .into_iter()
        .find(|&step| step as f32 * pixels_per_year >= LABEL_WIDTH)
        .unwrap_or(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zooming_keeps_the_point_under_the_mouse() {
        let view = YearRange {
            start: 0.0,
            end: 20.0,
        };
        let zoomed = view.zoomed(2.0, 15.0);
        assert_eq!(
            zoomed,
            YearRange {
                start: 7.5,
                end: 17.5
            }
        );
    }

    #[test]
    fn view_stays_within_the_years() {
        let panned_too_far = YearRange {
            start: 18.0,
            end: 28.0,
        };
        assert_eq!(
            panned_too_far.fit_within(20.0),
            YearRange {
                start: 10.0,
                end: 20.0
            }
        );

        let zoomed_too_far = YearRange {
            start: 5.0,
            end: 5.1,
        };
        assert_eq!(zoomed_too_far.fit_within(20.0).width(), 1.0);

        let everything = YearRange {
            start: f64::NEG_INFINITY,
            end: f64::INFINITY,
        };
        assert_eq!(
            everything.fit_within(20.0),
            YearRange {
                start: 0.0,
                end: 20.0
            }
        );
    }

    #[test]
    fn value_between_years_is_on_the_line() {
        let values = [0.0, 10.0, 30.0];
        assert_eq!(value_at(&values, 1.5), 20.0);
        assert_eq!(value_at(&values, 2.0), 30.0);
        assert_eq!(value_at(&values, 5.0), 30.0);
    }

    #[test]
    fn spread_apart_keeps_order_and_gap() {
        let mut ys = [100.0, 101.0, 102.0, 300.0];
        spread_apart(&mut ys, 15.0, 0.0, 500.0);
        assert_eq!(ys, [100.0, 115.0, 130.0, 300.0]);
    }
}
