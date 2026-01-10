use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding};

use crate::config::AppConfig;
use crate::ui::panes::node_info::NodeInfo;
use crate::ui::utils::BlockExt;

pub fn create_pane_block<'a>(config: &AppConfig, ni: NodeInfo<'a>) -> Block<'a> {
    let p = &config.theme.palette;

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(if ni.is_active {
            p.border_active
        } else {
            p.border_inactive
        })
        .padding(Padding::uniform(1))
        .merge_if(config.theme.collapse_borders && config.zen);

    if !config.zen {
        let title_top_left = Line::from(vec![
            Span::styled("Every ", p.meta_label),
            Span::styled(ni.interval_secs_str, p.meta_value),
            Span::styled(": ", p.meta_label),
            Span::styled(ni.exec_str, p.meta_highlight),
        ]);
        block = block.title(title_top_left);

        if config.theme.show_state {
            let title_top_right =
                Line::from(Span::styled(ni.state_str, p.meta_value)).right_aligned();
            block = block.title(title_top_right);
        }

        if config.theme.show_last_updated {
            let title_bottom_left = Line::from(vec![
                Span::styled("Updated: ", p.meta_label),
                Span::styled(ni.last_exec_time, p.meta_value),
                Span::styled(format!(" ({})", ni.duration), p.meta_value),
            ]);
            block = block.title_bottom(title_bottom_left);
        }

        if config.theme.show_display_type {
            let title_bottom_right =
                Line::from(Span::styled(ni.display_type_str, p.meta_value)).right_aligned();
            block = block.title_bottom(title_bottom_right);
        }

        if config.theme.show_history_meter {
            let title_bottom_centered =
                Line::from(Span::styled(ni.history_limit, p.meta_meter)).left_aligned();
            block = block.title_bottom(title_bottom_centered);
        }
    }
    block
}
