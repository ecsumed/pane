use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::mode::{AppMode, DiffMode};
use crate::pane::PaneKey;
use crate::logging::debug;
use crate::ui::diffs;
use crate::ui::utils::{BlockExt, LayoutExt};

pub fn draw_observe_mode(
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
        let p = &config.theme.palette;

        // This is crazy, probably needs to be simplified.

        ////////////////////////////////////////
        // HISTORY
        ////////////////////////////////////////
        let Some(command) = commands.get(active_id) else { return; };
        let current_len = command.output_history.len();

        if current_len > *last_history_len {
            let diff = current_len - *last_history_len;
            
            if *selected_history_idx > 0 {
                *selected_history_idx += diff;
            }
            
            *last_history_len = current_len;
        } else if current_len < *last_history_len {
            let diff = *last_history_len - current_len;
            *selected_history_idx = selected_history_idx.saturating_sub(diff);
            *last_history_len = current_len;
        }
        
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

        let data_idx = current_len.saturating_sub(1).saturating_sub(*selected_history_idx);
        let prev_data_idx = data_idx.checked_sub(1);

        let current_output = command.output_history.get(data_idx);
        let previous_output = prev_data_idx.and_then(|idx| command.output_history.get(idx));
        
        // debug!("UI Selected: {}, Vector Current: {}, Vector Prev: {:?}", selected_history_idx, data_idx, prev_data_idx);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .merge_if(config.theme.collapse_borders)
                    .title("History")
                )
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        history_list_state.select(Some(*selected_history_idx));
        frame.render_stateful_widget(list, history_area, history_list_state);

        ////////////////////////////////////////
        // CONTENT
        ////////////////////////////////////////
        let current_text = current_output.unwrap().output.to_string();
        let previous_text = previous_output.as_ref().map_or("", |c| &c.output);

        let display_text = diffs::render_diff(
            &config.theme,
            &current_text,
            &previous_text,
            *diff_mode, 
            search_input.value()
        );

        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_style(p.border_active);
        let widget = Paragraph::new(display_text).block(content_block);
        frame.render_widget(widget, content_area);
    
        ////////////////////////////////////////
        // SEARCH
        ////////////////////////////////////////
        let prefix = Span::styled(" / ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
        let search_text = Span::raw(search_input.value());
        
        let search_content = Line::from(vec![prefix, search_text]);
        
        let search_block = Block::default()
            .borders(Borders::ALL)
            .merge_if(config.theme.collapse_borders)
            .border_style(Style::default().fg(Color::DarkGray))
            .title_alignment(ratatui::layout::Alignment::Right)
            .title(Span::styled(" SEARCH ", Style::default().fg(Color::Yellow).bg(Color::Black)));
        
        // 4. Render the widget
        let search_input_widget = Paragraph::new(search_content)
            .block(search_block);
        
        frame.render_widget(search_input_widget, search_area);
    }
}

