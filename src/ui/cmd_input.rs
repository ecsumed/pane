use ratatui::prelude::Frame;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::mode::AppMode;
use crate::ui::picker::{self, PickerStyle, Prompt};
use crate::App;

const SUGGESTION_ROWS: u16 = 10;

pub fn draw_input_popup(frame: &mut Frame, app: &mut App) {
    let AppMode::CmdEdit {
        input,
        state,
        suggestions,
        ..
    } = &app.mode
    else {
        return;
    };
    let s = PickerStyle::from_palette(&app.config.theme.palette);

    let pane = app
        .pane_manager
        .pane_key_to_friendly_id(&app.pane_manager.active_pane_id)
        .map_or(String::new(), |id| format!(" for pane {id}"));
    let title = format!("Command{pane}");

    let typed = input.value();
    let rows: Vec<ListItem> = suggestions
        .iter()
        .map(|suggestion| {
            let (matched, rest) = match suggestion.strip_prefix(typed) {
                Some(rest) => (typed, rest),
                None => ("", suggestion.as_str()),
            };
            ListItem::new(Line::from(vec![
                Span::styled(matched.to_string(), s.accent),
                Span::styled(rest.to_string(), s.text),
            ]))
        })
        .collect();

    let empty_message = if typed.is_empty() {
        "Type a command. Matches from your shell history appear here."
    } else {
        "No matches in shell history. Enter runs it as typed."
    };

    let area = picker::popup_area(frame.area(), 100, SUGGESTION_ROWS);
    let prompt = Prompt {
        symbol: "$",
        input,
        focused: true,
        placeholder: "",
    };
    let areas = picker::draw_frame(frame, area, &s, &title, prompt, None);
    picker::draw_list(frame, areas.list, &s, rows, state.selected(), empty_message);
    picker::draw_footer(
        frame,
        areas.footer,
        picker::hints(
            &s,
            &[
                ("enter", "run"),
                ("↑↓", "pick from history"),
                ("tab", "complete"),
                ("esc", "cancel"),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::ListState;
    use ratatui::Terminal;
    use tui_input::Input;

    use super::*;
    use crate::config::AppConfig;
    use crate::shell_history::ShellHistoryManager;

    #[test]
    fn test_suggestions_do_not_shift_when_one_is_selected() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let column = |app: &mut App| {
            let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
            terminal.draw(|frame| draw_input_popup(frame, app)).unwrap();
            let screen = terminal.backend().to_string();
            screen
                .lines()
                .find_map(|l| l.find("kubectl get nodes"))
                .unwrap()
        };

        app.mode = AppMode::CmdEdit {
            input: Input::default().with_value("kubectl".to_string()),
            state: ListState::default(),
            suggestions: vec!["kubectl get pods".into(), "kubectl get nodes".into()],
            history: ShellHistoryManager::from_commands(Vec::new()),
        };
        let unselected = column(&mut app);
        if let AppMode::CmdEdit { state, .. } = &mut app.mode {
            state.select(Some(0));
        }
        assert_eq!(column(&mut app), unselected);
    }

    #[test]
    fn test_suggestions_fit_short_terminal() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::CmdEdit {
            input: Input::default().with_value("kubectl".to_string()),
            state: ListState::default(),
            suggestions: (0..10).map(|i| format!("kubectl get pods {i}")).collect(),
            history: ShellHistoryManager::from_commands(Vec::new()),
        };

        for height in [1, 3, 10, 15] {
            let mut terminal = Terminal::new(TestBackend::new(80, height)).unwrap();
            terminal
                .draw(|frame| draw_input_popup(frame, &mut app))
                .unwrap();
        }
    }
}
