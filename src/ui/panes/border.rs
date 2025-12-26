use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::command::Command;
use crate::config::AppConfig;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::ui::display_modes::render_command_output;
use crate::ui::DisplayType;
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
        .border_style(
            if is_active { 
                p.border_active
            } else { 
                p.border_inactive
            }
        )
        .padding(Padding::uniform(1))
        .merge_if(config.theme.collapse_borders && config.zen);

    if !config.zen {
        let title_top_left = Line::from(vec![
            Span::styled("Every ", p.meta_label),
            Span::styled(interval_secs_str, p.meta_value),
            Span::styled(": ", p.meta_label),
            Span::styled(exec_str, p.meta_highlight),
        ]);

        let title_top_right = Line::from(vec![
            Span::styled(state_str, p.meta_value),
        ]).right_aligned();
    
        let title_bottom_left = Line::from(vec![
            Span::styled("Updated: ", p.meta_label),
            Span::styled(last_exec_time, p.meta_value),
            Span::styled(format!(" ({duration})"), p.meta_value),
        ]);
        let title_bottom_right = Line::from(
            Span::styled(display_type_str, p.meta_value),
        ).right_aligned();

        block = block
            .title(title_top_left)
            .title(title_top_right)
            .title_bottom(title_bottom_left)
            .title_bottom(title_bottom_right)
    }
    block
}