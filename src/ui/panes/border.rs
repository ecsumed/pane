use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding};

use crate::config::AppConfig;
use crate::ui::utils::BlockExt;

pub fn create_pane_block<'a>(
    config: &AppConfig,
    is_active: bool,
    exec_str: &'a str,
    interval_secs_str: &'a str,
    last_exec_time: &'a str,
    duration: &'a str,
    state_str: &'a str,
    display_type_str: &'a str,
) -> Block<'a> {
    let p = &config.theme.palette;

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active {
            p.border_active
        } else {
            p.border_inactive
        })
        .padding(Padding::uniform(1))
        .merge_if(config.theme.collapse_borders && config.zen);

    if !config.zen {
        let title_top_left = Line::from(vec![
            Span::styled("Every ", p.meta_label),
            Span::styled(interval_secs_str, p.meta_value),
            Span::styled(": ", p.meta_label),
            Span::styled(exec_str, p.meta_highlight),
        ]);
        block = block.title(title_top_left);

        if config.theme.show_state {
            let title_top_right = Line::from(Span::styled(state_str, p.meta_value)).right_aligned();
            block = block.title(title_top_right);
        }

        if config.theme.show_last_updated {
            let title_bottom_left = Line::from(vec![
                Span::styled("Updated: ", p.meta_label),
                Span::styled(last_exec_time, p.meta_value),
                Span::styled(format!(" ({duration})"), p.meta_value),
            ]);
            block = block.title_bottom(title_bottom_left);
        }

        if config.theme.show_display_type {
            let title_bottom_right =
                Line::from(Span::styled(display_type_str, p.meta_value)).right_aligned();
            block = block.title_bottom(title_bottom_right);
        }
    }
    block
}
