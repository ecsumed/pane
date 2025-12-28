use ratatui::{Frame, layout::Rect, style::{Color, Style}, widgets::{Block, Borders, Paragraph, Sparkline}};
use ratatui::text::{Line, Span, Text};

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::DisplayType;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let p = &config.theme.palette;

    let numeric_data: Vec<Option<u64>> = command
        .output_history
        .iter()
        .map(|entry| entry.output.trim().parse::<f64>().ok().map(|v| v as u64))
        .collect();

    if numeric_data.iter().all(|val| val.is_none()) {
        let msg = Paragraph::new("No numeric data found in history to graph.")
            .style(Style::default().fg(Color::Red))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::empty()));
        frame.render_widget(msg, area);
        return;
    }

    let data: Vec<u64> = numeric_data.into_iter().flatten().collect();

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::empty()))
        .data(&data)
        .style(p.meta_highlight);

    frame.render_widget(sparkline, area);
}