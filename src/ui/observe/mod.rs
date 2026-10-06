mod content;
mod history;
mod search;

use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Clear, Widget};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::mode::{AppMode, ObserveFocus};
use crate::pane::PaneKey;
use crate::ui::utils::LayoutExt;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    commands: &HashMap<PaneKey, Command>,
    mode_state: &mut AppMode,
) {
    let [main_area, history_area] =
        Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
            .collapse_if(config.theme.collapse_borders)
            .areas(area);

    let [search_area, content_area] = Layout::vertical([Constraint::Length(3), Constraint::Min(0)])
        .collapse_if(config.theme.collapse_borders)
        .areas(main_area);

    Clear.render(area, frame.buffer_mut());

    if let AppMode::Observe {
        active_id,
        selected_time,
        diff_mode,
        search_input,
        history_list_state,
        focus,
        scroll_offset,
        max_scroll,
        scrollbar_state,
    } = mode_state
    {
        let Some(command) = commands.get(active_id) else {
            return;
        };

        let selected_history_idx = command.history_position(*selected_time);
        if selected_time.is_some() {
            *selected_time = command.history_time_at(selected_history_idx);
        }

        // Render History
        let history_w = history::widget(config, command, *focus == ObserveFocus::History);
        history_list_state.select(Some(selected_history_idx));
        frame.render_stateful_widget(history_w, history_area, history_list_state);

        // Render Content
        content::render(
            frame,
            content_area,
            config,
            command,
            selected_history_idx,
            *diff_mode,
            search_input.value(),
            *scroll_offset,
            max_scroll,
            scrollbar_state,
            *focus == ObserveFocus::Content,
        );

        // Render Search
        let search_w = search::widget(config, search_input.value(), *focus == ObserveFocus::Search);
        frame.render_widget(search_w, search_area);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;

    use crate::app::App;
    use crate::command::CommandOutput;
    use crate::config::AppConfig;
    use crate::mode::AppMode;
    use crate::ui::draw::draw_ui;

    pub(crate) async fn observe_app(runs: &[(&str, Option<i32>)]) -> App {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let id = app.pane_manager.active_pane_id;
        app.set_command(id, "kubectl get pods".to_string()).await;
        let task = app.tasks.get_mut(&id).unwrap();
        task.task_handle.take().unwrap().abort();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        for (i, (output, code)) in runs.iter().enumerate() {
            task.record_output(
                CommandOutput {
                    output: output.to_string(),
                    time: start + chrono::Duration::seconds(10 * i as i64),
                    exit_status: *code,
                    duration: std::time::Duration::from_millis(1_500),
                },
                10,
            );
        }
        app.mode = AppMode::new_observing(&app);
        app
    }

    pub(crate) fn render(app: &mut App, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw_ui(app, frame)).unwrap();
        terminal.backend().buffer().clone()
    }

    pub(crate) fn row_containing(buffer: &Buffer, text: &str) -> (u16, String) {
        (0..buffer.area.height)
            .map(|y| {
                let row: String = (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect();
                (y, row)
            })
            .find(|(_, row)| row.contains(text))
            .unwrap_or_else(|| panic!("{text} not on screen"))
    }

    #[tokio::test]
    async fn test_selected_history_row_uses_palette_highlight() {
        let mut app = observe_app(&[("a", Some(0)), ("b", Some(0))]).await;
        let buffer = render(&mut app, 120, 20);

        let (y, row) = row_containing(&buffer, "▸");
        let x = row.chars().position(|c| c == '▸').unwrap() as u16;
        let expected = app.config.theme.palette.search_match.bg.unwrap();
        assert_eq!(buffer[(x + 2, y)].bg, expected);
    }
}
