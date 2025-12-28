mod history;
mod content;
mod search;

use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Widget};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::mode::{AppMode, DiffMode};
use crate::pane::PaneKey;
use crate::logging::debug;
use crate::ui::diffs;
use crate::ui::utils::{BlockExt, LayoutExt};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    commands: &HashMap<PaneKey, Command>,
    mode_state: &mut AppMode,
) {
    let [main_area, history_area] = Layout::horizontal([
        Constraint::Percentage(80),
        Constraint::Percentage(20),
    ])
    .collapse_if(config.theme.collapse_borders)
    .areas(area);

    let [search_area, content_area ] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .collapse_if(config.theme.collapse_borders)
    .areas(main_area);

    Clear.render(area, frame.buffer_mut());

    if let AppMode::Observe {
        active_id,
        selected_history_idx,
        last_history_len,
        diff_mode,
        search_input,
        history_list_state,
    } = mode_state {
        let Some(command) = commands.get(active_id) else { return; };
        
        let current_len = command.output_history.len();
        if current_len != *last_history_len {
            let diff = current_len.abs_diff(*last_history_len);
            if current_len > *last_history_len && *selected_history_idx > 0 {
                *selected_history_idx += diff;
            } else {
                *selected_history_idx = selected_history_idx.saturating_sub(diff);
            }
            *last_history_len = current_len;
        }

        // Render History
        let history_w = history::widget(config, command);
        history_list_state.select(Some(*selected_history_idx));
        frame.render_stateful_widget(history_w, history_area, history_list_state);

        // Render Content
        let content_w = content::widget(
            config, 
            command, 
            *selected_history_idx, 
            *diff_mode, 
            search_input.value()
        );
        frame.render_widget(content_w, content_area);

        // Render Search
        let search_w = search::widget(config, search_input.value());
        frame.render_widget(search_w, search_area);
    }
}
