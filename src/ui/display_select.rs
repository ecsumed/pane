use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, ListItem, Padding, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::command::Command;
use crate::mode::AppMode;
use crate::ui::display_modes::render_command_output;
use crate::ui::picker::{self, PickerStyle};

fn has_numbers(command: Option<&Command>) -> bool {
    command.is_some_and(|c| {
        c.output_history
            .iter()
            .any(|o| o.output.trim().parse::<f64>().is_ok())
    })
}

pub fn draw_display_type_select(frame: &mut Frame, app: &mut App) {
    let AppMode::DisplayTypeSelect { picker: state } = &app.mode else {
        return;
    };
    let s = PickerStyle::from_palette(&app.config.theme.palette);
    let command = app.tasks.get(&app.pane_manager.active_pane_id);
    let current = command.map(|c| c.display_type);
    let numeric = has_numbers(command);

    let mut rows = Vec::new();
    let mut selected_row = None;
    let mut last_group = None;
    let selected = state.selected_index();

    for (index, dt) in state.visible() {
        if last_group != Some(dt.group()) {
            last_group = Some(dt.group());
            let mut header = vec![Span::styled(dt.group().to_uppercase(), s.accent)];
            if dt.needs_numbers() && !numeric {
                header.push(Span::styled("  no numbers in output", s.disabled));
            }
            rows.push(ListItem::new(Line::from(header)));
        }

        let dimmed = dt.needs_numbers() && !numeric;
        let mut spans = vec![Span::styled(
            format!("  {}", dt.label()),
            if dimmed { s.disabled } else { s.text },
        )];
        if Some(*dt) == current {
            spans.push(Span::styled(" ●", s.marker));
        }

        if Some(index) == selected {
            selected_row = Some(rows.len());
        }
        rows.push(ListItem::new(Line::from(spans)));
    }

    let area = picker::popup_area(frame.area(), 120, (state.items().len() + 4).max(20) as u16);
    let areas = picker::draw_frame(frame, area, &s, "Display", &state.filter, Some(34));
    picker::draw_list(
        frame,
        areas.list,
        &s,
        rows,
        selected_row,
        "No matching display types",
    );
    if let (Some(preview_area), Some(dt)) = (areas.preview, state.selected()) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(s.border)
            .padding(Padding::left(1))
            .title(Span::styled(format!(" Preview: {} ", dt.label()), s.muted));
        let message = match command {
            None => Some(" No command in this pane"),
            Some(cmd) if cmd.output_history.is_empty() => Some(" No output yet"),
            Some(_) => None,
        };
        match (command, message) {
            (Some(cmd), None) => {
                render_command_output(frame, preview_area, &app.config, cmd, *dt, block)
            }
            (_, message) => frame.render_widget(
                Paragraph::new(Span::styled(message.unwrap_or_default(), s.muted)).block(block),
                preview_area,
            ),
        }
    }

    picker::draw_footer(
        frame,
        areas.footer,
        picker::hints(
            &s,
            &[
                ("↑↓", "move"),
                ("enter", "apply"),
                ("type", "filter"),
                ("esc", "cancel"),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::command::CommandOutput;
    use crate::config::AppConfig;
    use crate::ui::DisplayType;

    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw_display_type_select(frame, app))
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn test_all_display_types_visible() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select(None);

        let screen = render(&mut app, 100, 30);
        for item in DisplayType::iter() {
            assert!(screen.contains(item.label()), "{:?} not shown", item);
        }
        for group in ["TEXT", "DIFF", "CHARTS", "OTHER"] {
            assert!(screen.contains(group), "{group} header missing");
        }

        render(&mut app, 40, 5);
        render(&mut app, 10, 2);
    }

    #[test]
    fn test_filter_narrows_display_types() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select(None);
        if let AppMode::DisplayTypeSelect { picker } = &mut app.mode {
            for c in "chart".chars() {
                let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
                picker.handle_filter_event(&Event::Key(key));
            }
        }

        let screen = render(&mut app, 100, 30);
        assert!(screen.contains("Line chart"));
        assert!(screen.contains("CHARTS"));
        assert!(!screen.contains("Raw text"));
    }

    #[tokio::test]
    async fn test_current_type_marked_and_charts_dimmed() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let id = app.pane_manager.active_pane_id;
        app.set_command(id, "echo not-a-number".to_string()).await;
        if let Some(task) = app.tasks.get_mut(&id) {
            task.display_type = DisplayType::DiffWord;
            task.task_handle.take().unwrap().abort();
            task.record_output(
                CommandOutput {
                    output: "not-a-number".to_string(),
                    time: chrono::Local::now().naive_local(),
                    exit_status: Some(0),
                    duration: std::time::Duration::from_millis(5),
                },
                10,
            );
        }
        app.mode = AppMode::new_display_type_select(Some(DisplayType::DiffWord));

        let screen = render(&mut app, 100, 30);
        let marked = screen
            .lines()
            .find(|l| l.contains("Changed words ●"))
            .expect("current type not marked");
        assert!(marked.contains("> "), "current type not preselected");
        assert!(screen.contains("Preview: Changed words"));
        assert!(screen.contains("not-a-number"));

        let narrow = render(&mut app, 50, 30);
        assert!(!narrow.contains("Preview"));
        assert!(narrow.contains("Changed words ●"));
        let charts = screen.lines().find(|l| l.contains("CHARTS")).unwrap();
        assert!(charts.contains("no numbers"));
        assert!(screen.contains("esc cancel"));
    }
}

#[cfg(test)]
mod style_tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::config::AppConfig;
    use crate::ui::DisplayType;

    #[test]
    fn test_selected_row_uses_one_highlight_colour() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select(Some(DisplayType::Counter));
        let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();
        terminal
            .draw(|frame| draw_display_type_select(frame, &mut app))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let row = (0..buffer.area.height)
            .find(|&y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .contains("Counter")
            })
            .unwrap();
        let expected = app.config.theme.palette.search_match.bg;
        let start = (0..buffer.area.width)
            .find(|&x| buffer[(x, row)].symbol() == "C")
            .unwrap();
        for x in start..start + "Counter".len() as u16 {
            assert_eq!(buffer[(x, row)].bg, expected.unwrap());
        }
    }
}
