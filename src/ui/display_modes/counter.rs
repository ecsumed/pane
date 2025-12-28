use std::collections::HashMap;
use ratatui::widgets::{List, ListItem};

use ratatui::{Frame, layout::Rect, style::{Color, Style}, widgets::{Block, Borders, Paragraph, Sparkline}};
use ratatui::text::{Line, Span, Text};

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::DisplayType;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let p = &config.theme.palette;
    
    let mut counts = HashMap::new();
    for entry in &command.output_history {
        *counts.entry(&entry.output).or_insert(0) += 1;
    }

    let mut sorted_counts: Vec<_> = counts.into_iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));

    let items: Vec<ListItem> = sorted_counts
        .into_iter()
        .map(|(text, count)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>4} ", count), p.counter_key),
                Span::styled(text, p.output),
            ]))
        })
        .collect();

    let widget = List::new(items)
        .block(Block::default().borders(Borders::empty()));

    frame.render_widget(widget, area);
}