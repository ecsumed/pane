use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::config::AppConfig;
use crate::logging::trace;
use crate::{command::Command, ui::DisplayType};

use chrono::NaiveDateTime;

fn datetime_to_f64(dt: NaiveDateTime) -> f64 {
    let seconds = dt.and_utc().timestamp() as f64;
    let millis = dt.and_utc().timestamp_subsec_millis() as f64 / 1000.0;
    seconds + millis
}

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let p = &config.theme.palette;

    let data_result: Result<Vec<(f64, f64)>, &str> = command
        .output_history
        .iter()
        .map(|entry| {
            let x = datetime_to_f64(entry.time);

            let y = entry
                .output
                .trim()
                .parse::<f64>()
                .map_err(|_| "Conversion failed")?;

            Ok((x, y))
        })
        .collect();

    match data_result {
        Err(_) => {
            let msg = Paragraph::new("No numeric data found in history to graph.")
                .style(p.error)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::empty()));
            frame.render_widget(msg, area);
        }
        Ok(chart_data) if chart_data.is_empty() => {
            let msg = Paragraph::new("No data.")
                .style(p.error)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::empty()));
            frame.render_widget(msg, area);
        }
        Ok(chart_data) => {
            let max_data_points = area.width as usize;

            let display_data = if chart_data.len() > max_data_points {
                &chart_data[chart_data.len() - max_data_points..]
            } else {
                &chart_data[..]
            };

            let (graph_type, marker, style) = match command.display_type {
                DisplayType::LineChart => (GraphType::Line, symbols::Marker::Braille, p.chart_line),
                DisplayType::BarChart => (GraphType::Bar, symbols::Marker::Quadrant, p.chart_bar),
                _ => (GraphType::Scatter, symbols::Marker::Dot, p.chart_scatter),
            };

            let datasets = vec![Dataset::default()
                .marker(marker)
                .style(style)
                .graph_type(graph_type)
                .data(display_data)];

            let x_min = display_data.first().map(|(x, _)| *x).unwrap_or(0.0);
            let x_max = display_data.last().map(|(x, _)| *x).unwrap_or(0.0);

            let y_min = display_data
                .iter()
                .map(|(_, y)| *y)
                .fold(f64::INFINITY, f64::min);
            let y_max = display_data
                .iter()
                .map(|(_, y)| *y)
                .fold(f64::NEG_INFINITY, f64::max);
            let y_padding = ((y_max - y_min) * 0.1).max(y_min * 0.1);

            trace!("data: {:?}", display_data);
            trace!("x min: {x_min}, x_max: {x_max}");
            trace!("y min: {y_min}, y_max: {y_max}, y padding: {y_padding}");

            let chart = Chart::new(datasets)
                .x_axis(
                    Axis::default()
                        .style(Style::default().gray())
                        .bounds([x_min, x_max]),
                )
                .y_axis(
                    Axis::default()
                        .style(Style::default().gray())
                        .bounds([y_min - y_padding, y_max + y_padding])
                        .labels([y_min.to_string(), y_max.to_string()]),
                );

            frame.render_widget(chart, area);
        }
    }
}
