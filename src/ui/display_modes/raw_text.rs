use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, cmd: &Command) {
    let p = &config.theme.palette;

    let lines: Vec<Line> = match cmd.last_output() {
        Some(out) => out
            .output
            .lines()
            .iter()
            .map(|line| Line::from(line.as_ref()))
            .collect(),
        None => vec![Line::from("N/A")],
    };
    let mut widget = Paragraph::new(lines).style(p.output);

    if config.wrap {
        widget = widget.wrap(Wrap { trim: false });
    }

    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::app::App;
    use crate::command::CommandOutput;
    use crate::config::AppConfig;
    use crate::ui::draw::draw_ui;

    #[tokio::test]
    async fn test_wrapped_output_keeps_indentation() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        assert!(app.config.wrap);
        let id = app.pane_manager.active_pane_id;
        app.set_command(id, "cat config.yaml".to_string()).await;
        let task = app.tasks.get_mut(&id).unwrap();
        task.task_handle.take().unwrap().abort();
        task.record_output(
            CommandOutput {
                output: "root:\n  child: 1\n    leaf: 2\n".into(),
                time: chrono::Local::now().naive_local(),
                exit_status: Some(0),
                duration: std::time::Duration::from_millis(1),
            },
            10,
        );

        let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
        terminal.draw(|frame| draw_ui(&mut app, frame)).unwrap();
        let screen = terminal.backend().to_string();

        let column = |text: &str| {
            screen
                .lines()
                .find_map(|l| l.find(text))
                .unwrap_or_else(|| panic!("{text} missing\n{screen}"))
        };
        assert_eq!(column("child"), column("root") + 2);
        assert_eq!(column("leaf"), column("root") + 4);
    }
}
