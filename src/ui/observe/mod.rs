mod content;
mod history;

use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Clear, Widget};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::mode::{AppMode, ObserveFocus};
use crate::pane::PaneKey;
use crate::ui::utils::LayoutExt;

const HISTORY_WIDTH: u16 = 30;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    commands: &HashMap<PaneKey, Command>,
    mode_state: &mut AppMode,
) {
    let [content_area, history_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(HISTORY_WIDTH)])
            .collapse_if(config.theme.collapse_borders)
            .areas(area);

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
        change_counts,
        current_match,
        match_count,
        jump_to_match,
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
        history::update_change_counts(command, change_counts);
        let history_w = history::widget(
            config,
            command,
            change_counts,
            *focus == ObserveFocus::History,
        );
        history_list_state.select(Some(selected_history_idx));
        frame.render_stateful_widget(history_w, history_area, history_list_state);

        content::render(
            frame,
            content_area,
            config,
            command,
            content::ContentView {
                selected_idx: selected_history_idx,
                diff_mode: *diff_mode,
                search: search_input,
                search_focused: *focus == ObserveFocus::Search,
                focused: *focus == ObserveFocus::Content,
                scroll_offset,
                max_scroll,
                scrollbar_state,
                current_match,
                match_count,
                jump_to_match,
            },
        );
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
    use crate::ui::observe::history;

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
    async fn test_history_rows_show_status_duration_and_changes() {
        let mut app = observe_app(&[
            ("a\nb\n", Some(0)),
            ("a\nb\n", Some(0)),
            ("Command failed", Some(1)),
            ("a\nB\n", None),
            ("a\nb\nc\n", Some(0)),
        ])
        .await;
        let buffer = render(&mut app, 120, 24);

        row_containing(&buffer, "History (5/10)");
        let (y, latest) = row_containing(&buffer, "Latest   12:00:40");
        assert!(latest.contains('✓'), "{latest}");
        assert_eq!(row_containing(&buffer, "1.5s · +2 −1").0, y + 1);

        let (y, timed_out) = row_containing(&buffer, "12:00:30  ");
        assert!(timed_out.contains('⏱'), "{timed_out}");
        assert_eq!(row_containing(&buffer, "timed out").0, y + 1);

        let (y, failed) = row_containing(&buffer, "12:00:20  ");
        let x = failed.chars().position(|c| c == '✗').unwrap() as u16;
        assert_eq!(
            buffer[(x, y)].fg,
            app.config.theme.palette.error.fg.unwrap()
        );
        assert_eq!(row_containing(&buffer, "exit 1").0, y + 1);

        let (y, _) = row_containing(&buffer, "12:00:10  ");
        assert_eq!(row_containing(&buffer, "no change").0, y + 1);
        let (y, _) = row_containing(&buffer, "12:00:00  ");
        assert_eq!(row_containing(&buffer, "first run").0, y + 1);
    }

    #[tokio::test]
    async fn test_change_counts_forget_evicted_runs() {
        let app = observe_app(&[("a", Some(0)), ("b", Some(0))]).await;
        let command = app.tasks.values().next().unwrap();
        let stale = chrono::NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let mut counts = history::ChangeCounts::from([(stale, (9, 9))]);

        history::update_change_counts(command, &mut counts);

        assert!(!counts.contains_key(&stale));
        assert_eq!(counts.len(), 1);
    }

    #[tokio::test]
    async fn test_content_title_names_command_and_compared_runs() {
        let mut app = observe_app(&[("a", Some(0)), ("b", Some(0))]).await;
        let buffer = render(&mut app, 140, 20);
        let (_, title) = row_containing(&buffer, "kubectl get pods");
        assert!(title.contains("every 5s"), "{title}");
        assert!(
            title.contains("12:00:10 vs 12:00:00 · Word diff"),
            "{title}"
        );

        if let AppMode::Observe { diff_mode, .. } = &mut app.mode {
            *diff_mode = crate::mode::DiffMode::None;
        }
        let buffer = render(&mut app, 140, 20);
        let (_, title) = row_containing(&buffer, "kubectl get pods");
        assert!(title.contains("12:00:10 · Plain"), "{title}");
    }

    async fn press(app: &mut App, code: crossterm::event::KeyCode) {
        use crossterm::event::{Event, KeyEvent, KeyModifiers};
        let event = Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
        crate::controls::handle_event(app, event).await.unwrap();
    }

    #[tokio::test]
    async fn test_search_strip_counts_and_jumps_between_matches() {
        use crossterm::event::KeyCode;

        let output: String = (0..60)
            .map(|i| match i {
                5 => "needle first\n".to_string(),
                45 => "needle second\n".to_string(),
                i => format!("line {i}\n"),
            })
            .collect();
        let mut app = observe_app(&[(output.as_str(), Some(0))]).await;

        let buffer = render(&mut app, 120, 20);
        assert!((0..buffer.area.height).all(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            !row.contains("│  / ")
        }));

        press(&mut app, KeyCode::Char('/')).await;
        for c in "needle".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }
        press(&mut app, KeyCode::Enter).await;
        let buffer = render(&mut app, 120, 20);
        row_containing(&buffer, "match 1 of 2");
        row_containing(&buffer, "needle first");

        press(&mut app, KeyCode::Char('n')).await;
        let buffer = render(&mut app, 120, 20);
        row_containing(&buffer, "match 2 of 2");
        row_containing(&buffer, "needle second");

        press(&mut app, KeyCode::Char('n')).await;
        let buffer = render(&mut app, 120, 20);
        row_containing(&buffer, "match 1 of 2");
        row_containing(&buffer, "needle first");

        for c in ['/', 'z', 'z', 'z'] {
            press(&mut app, KeyCode::Char(c)).await;
        }
        let buffer = render(&mut app, 120, 20);
        row_containing(&buffer, "no matches");
    }

    #[tokio::test]
    async fn test_key_hints_follow_focus() {
        use crossterm::event::KeyCode;

        let mut app = observe_app(&[("a", Some(0))]).await;
        let buffer = render(&mut app, 140, 20);
        let (_, hints) = row_containing(&buffer, "select run");
        assert!(hints.contains("esc back"), "{hints}");

        press(&mut app, KeyCode::Left).await;
        let buffer = render(&mut app, 140, 20);
        let (_, hints) = row_containing(&buffer, "↑↓ scroll");
        assert!(!hints.contains("n/N"), "{hints}");

        press(&mut app, KeyCode::Char('/')).await;
        let buffer = render(&mut app, 140, 20);
        row_containing(&buffer, "enter keep");

        press(&mut app, KeyCode::Char('a')).await;
        press(&mut app, KeyCode::Enter).await;
        let buffer = render(&mut app, 140, 20);
        let (_, hints) = row_containing(&buffer, "↑↓ scroll");
        assert!(hints.contains("n/N match"), "{hints}");
        assert!(hints.contains("esc clear "), "{hints}");
    }

    #[tokio::test]
    async fn test_history_panel_has_fixed_width() {
        let mut app = observe_app(&[("a", Some(0))]).await;
        for width in [100, 200] {
            let buffer = render(&mut app, width, 20);
            let (_, row) = row_containing(&buffer, "History");
            let chars: Vec<char> = row.chars().collect();
            let start = chars.iter().rposition(|&c| c == '┌').unwrap();
            assert_eq!(chars.len() - start, 30);
        }
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
