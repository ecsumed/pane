use std::time::SystemTime;

use ratatui::prelude::Frame;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::mode::AppMode;
use crate::ui::help_menu::home_relative;
use crate::ui::picker::{self, PickerStyle, Prompt};
use crate::ui::session_load::format_age;
use crate::App;

const BODY_ROWS: u16 = 10;

pub fn draw_session_save_popup(frame: &mut Frame, app: &App) {
    let AppMode::SessionSave { input, existing } = &app.mode else {
        return;
    };
    let s = PickerStyle::from_palette(&app.config.theme.palette);
    let name = input.value().trim().trim_end_matches(".toml");
    let folder = home_relative(&app.config.sessions_dir);

    let mut rows = vec![if name.is_empty() {
        ListItem::new(Span::styled(
            "Leave empty to save with a timestamped name",
            s.muted,
        ))
    } else {
        ListItem::new(Line::from(vec![
            Span::styled("Saves as ", s.muted),
            Span::styled(format!("{name}.toml"), s.text),
            Span::styled(format!(" in {folder}"), s.muted),
        ]))
    }];

    if name.contains(['/', '\\']) {
        rows.push(ListItem::new(Span::styled(
            "Names can't contain slashes",
            s.error,
        )));
    } else if existing.iter().any(|entry| entry.name == name) {
        rows.push(ListItem::new(Span::styled(
            format!("A session named {name} already exists and will be overwritten"),
            s.error,
        )));
    }

    let query = name.to_lowercase();
    let matching: Vec<_> = existing
        .iter()
        .filter(|entry| entry.name.to_lowercase().contains(&query))
        .collect();
    if !matching.is_empty() {
        rows.push(ListItem::new(""));
        rows.push(ListItem::new(Span::styled("Existing sessions", s.accent)));
        let now = SystemTime::now();
        rows.extend(matching.into_iter().map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<32}", entry.name), s.text),
                Span::styled(format_age(entry.modified, now), s.muted),
            ]))
        }));
    }

    let area = picker::popup_area(frame.area(), 80, BODY_ROWS);
    let prompt = Prompt {
        symbol: "name",
        input,
        focused: true,
        placeholder: "",
    };
    let areas = picker::draw_frame(frame, area, &s, "Save session", prompt, None);
    picker::draw_list(frame, areas.list, &s, rows, None, "");
    picker::draw_footer(
        frame,
        areas.footer,
        picker::hints(&s, &[("enter", "save"), ("esc", "cancel")]),
    );
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::config::AppConfig;

    fn app_with_sessions(test: &str, names: &[&str]) -> (App, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("pane-save-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for name in names {
            fs::write(dir.join(format!("{name}.toml")), "").unwrap();
        }
        let config = AppConfig {
            sessions_dir: dir.clone(),
            ..AppConfig::default()
        };
        let mut app = App::new(config, Vec::new());
        app.mode = AppMode::new_session_save(&app);
        (app, dir)
    }

    async fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            let event = Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            crate::controls::handle_event(app, event).await.unwrap();
        }
    }

    fn render(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal
            .draw(|frame| draw_session_save_popup(frame, app))
            .unwrap();
        terminal.backend().to_string()
    }

    #[tokio::test]
    async fn test_shows_target_and_warns_before_overwriting() {
        let (mut app, dir) = app_with_sessions("overwrite", &["istio", "loki-pods"]);

        let screen = render(&mut app);
        assert!(screen.contains("Leave empty to save with a timestamped name"));
        assert!(screen.contains("istio") && screen.contains("loki-pods"));

        type_text(&mut app, "istio").await;
        let screen = render(&mut app);
        assert!(screen.contains("istio.toml"));
        assert!(screen.contains("already exists and will be overwritten"));
        assert!(!screen.contains("loki-pods"));

        type_text(&mut app, "/x").await;
        assert!(render(&mut app).contains("can't contain slashes"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn test_empty_name_saves_with_a_timestamp_and_slashes_are_rejected() {
        let (mut app, dir) = app_with_sessions("empty", &[]);
        let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        crate::controls::handle_event(&mut app, enter.clone())
            .await
            .unwrap();
        let saved: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(saved.len(), 1);
        assert!(saved[0].starts_with("session-"), "{saved:?}");

        app.mode = AppMode::new_session_save(&app);
        type_text(&mut app, "../escape").await;
        crate::controls::handle_event(&mut app, enter)
            .await
            .unwrap();
        assert!(app.current_notice().is_some_and(|n| n.is_error));
        assert!(!dir.join("../escape.toml").exists());

        fs::remove_dir_all(&dir).unwrap();
    }
}
