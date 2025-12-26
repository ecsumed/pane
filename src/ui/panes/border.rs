use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
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
    state_str: &'a str,
    display_type_str: &'a str,
) -> Block<'a> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active { Style::default().fg(Color::Green).bold() } else { Style::default() })
        .padding(Padding::uniform(1))
        .merge_if(config.theme.collapse_borders && config.zen);

    if !config.zen {
        let title_top_left = Line::from(vec![
            "Every ".into(),
            interval_secs_str.dark_gray().bold(),
            ": ".dark_gray().bold(),
            exec_str.blue().bold(),
        ]);
        let title_top_right = Line::from(vec![" State ".into(), state_str.blue().bold()]).right_aligned();
    
        let title_bottom_left = Line::from(vec![last_exec_time.blue().bold()]);
        let title_bottom_right = Line::from(display_type_str).right_aligned();

        block = block
            .title(title_top_left)
            .title(title_top_right)
            .title_bottom(title_bottom_left)
            .title_bottom(title_bottom_right)
    }
    block
}