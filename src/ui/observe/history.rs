use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};
use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::utils::BlockExt;

pub fn widget<'a>(
    config: &'a AppConfig,
    command: &'a Command,
) -> List<'a> {    
    let items: Vec<ListItem> = command.output_history
        .iter()
        .rev()
        .enumerate()
        .map(|(ui_idx, out)| {
            let label = if ui_idx == 0 {
                "Latest".to_string()
            } else {
                out.time.format("%H:%M:%S").to_string()
            };
            ListItem::new(label)
        })
        .collect();

    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .merge_if(config.theme.collapse_borders)
                .title("History")
        )
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
}
