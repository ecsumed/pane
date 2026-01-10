use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::command::Command;
use crate::config::AppConfig;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let p = &config.theme.palette;

    let numeric_data: Vec<Option<u64>> = command
        .output_history
        .iter()
        .map(|entry| entry.output.trim().parse::<f64>().ok().map(|v| v as u64))
        .collect();

    if numeric_data.iter().all(|val| val.is_none()) {
        let msg = Paragraph::new("No numeric data found in history to graph.")
            .style(p.error)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::empty()));
        frame.render_widget(msg, area);
        return;
    }

    let data: Vec<u64> = numeric_data.into_iter().flatten().collect();

    let max_data_points = area.width as usize;

    let display_data = if data.len() > max_data_points {
        &data[data.len() - max_data_points..]
    } else {
        &data[..]
    };

    let sparkline = Sparkline::default()
        // .bar_set(symbols::bar::NINE_LEVELS)
        .data(display_data)
        .style(p.spark_line);

    frame.render_widget(sparkline, area);
}
