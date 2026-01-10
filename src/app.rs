use std::collections::HashMap;
use std::io::{self};
use std::time::Duration;

use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;
use ratatui::prelude::Backend;
use ratatui::Terminal;
use tokio::sync::mpsc::{self};
use tokio::time::interval;

use crate::command::{Command, CommandControl, CommandEvent, CommandSerializableState};
use crate::config::AppConfig;
use crate::controls;
use crate::logging::{error, info, warn};
use crate::mode::AppMode;
use crate::pane::{PaneKey, PaneManager};
use crate::ui::draw::draw_ui;
use crate::ui::DisplayType;

#[derive(Debug)]
pub enum AppControl {
    SetCommand(PaneKey, String),
    SendControl(PaneKey, CommandControl),
    SetDisplay(PaneKey, DisplayType),
}

pub struct App {
    pub pane_manager: PaneManager,
    pub tasks: HashMap<PaneKey, Command>,
    pub mode: AppMode,
    pub exit: bool,
    pub output_rx: mpsc::Receiver<(PaneKey, CommandEvent)>,
    pub output_tx: mpsc::Sender<(PaneKey, CommandEvent)>,
    pub app_control_tx: mpsc::Sender<AppControl>,
    pub app_control_rx: mpsc::Receiver<AppControl>,
    pub config: AppConfig,
    pub pane_area: Rect,
}

impl App {
    pub fn new(config: AppConfig, command: Vec<String>) -> Self {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (app_control_tx, app_control_rx) = mpsc::channel(10);

        let pane_manager = PaneManager::new();

        if !command.is_empty() {
            let command = command.join(" ");
            if let Err(e) = app_control_tx
                .try_send(AppControl::SetCommand(pane_manager.active_pane_id, command))
            {
                error!("Send failed: {}", e);
            }
        }

        Self {
            pane_manager: pane_manager,
            tasks: HashMap::new(),
            mode: AppMode::default(),
            exit: false,
            output_rx,
            output_tx,
            app_control_tx,
            app_control_rx,
            config,
            pane_area: Rect::new(0, 0, 0, 0),
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()>
    where
        B: Backend,
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut tick_interval = interval(Duration::from_millis(250));
        let mut events = EventStream::new();

        loop {
            terminal.draw(|frame| {
                self.pane_area = frame.area();

                draw_ui(self, frame);
            })?;

            tokio::select! {
                Some((id, event)) = self.output_rx.recv() => {
                    match event {
                        CommandEvent::Started => {
                            if let Some(command) = self.tasks.get_mut(&id) {
                                command.state = crate::command::CommandState::Executing;
                            }
                        }
                        CommandEvent::Output(out) => {
                            if let Some(code) = out.exit_status {
                                if code != 0 && self.config.beep {
                                    App::beep()
                                }
                                if code != 0 && self.config.err_exit {
                                    info!("Exiting because err_exit was set.");
                                    self.exit();
                                }
                            }

                            if let Some(command) = self.tasks.get_mut(&id) {
                                command.state = crate::command::CommandState::Idle;
                                command.record_output(out, self.config.max_history);
                            }
                        }
                    }
                },
                Some(Ok(event)) = events.next().fuse() => {
                    controls::handle_event(self, event).await?;
                },

                Some(control) = self.app_control_rx.recv() => {
                    match control {
                        AppControl::SetCommand(id, exec) => {
                            self.set_command(id, exec).await;
                        },
                        AppControl::SendControl(id, cmd_ctrl) => {
                            if let Some(command) = self.tasks.get_mut(&id) {
                                command.handle_control_signal(id, cmd_ctrl).await;
                            }
                        }
                        AppControl::SetDisplay(id, display) => {
                            if let Some(command) = self.tasks.get_mut(&id) {
                                command.update_display(display);
                            }
                        }
                    }
                },
                _ = tick_interval.tick() => {
                },
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    pub fn exit(&mut self) {
        for (pane_key, _cmd) in &self.tasks {
            if let Err(e) = self
                .app_control_tx
                .try_send(AppControl::SendControl(*pane_key, CommandControl::Stop))
            {
                warn!("Failed to send AppControl::SendControl: {}", e);
            }
        }
        self.exit = true;
    }

    fn beep() {
        print!("\x07");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }

    pub async fn set_command(&mut self, id: PaneKey, exec: String) {
        if let Some(old) = self.tasks.insert(
            id,
            Command::spawn(
                id,
                exec,
                self.config.default_display,
                self.config.interval,
                self.output_tx.clone(),
            ),
        ) {
            if let Some(h) = old.task_handle {
                h.abort();
            }
        }
    }

    pub fn load_session(
        &mut self,
        pane_manager: PaneManager,
        tasks_state: HashMap<PaneKey, CommandSerializableState>,
    ) -> io::Result<()> {
        let running_tasks = Command::restore_tasks(tasks_state, self.output_tx.clone());

        self.pane_manager = pane_manager;
        self.tasks = running_tasks;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandControl, CommandState};
    use crate::config::AppConfig;
    use crate::pane::CardinalDirection;
    use crate::ui;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Direction;
    use tokio::time::error::Elapsed;
    use tokio::time::timeout;

    fn mock_config() -> AppConfig {
        std::env::set_var("PANE__CONFIG_PATH", "tests/fixtures/empty_config.toml");
        AppConfig::load().expect("")
    }

    fn mock_app() -> (App, PaneKey) {
        let config = mock_config();
        let command = ["echo", "test"].map(String::from).to_vec();
        let mut app = App::new(config, command);

        // These are dynamic and thus hard to test with insta snap
        app.config.theme.show_last_updated = false;
        app.config.theme.show_state = false;

        let pane_key = app.pane_manager.get_all_pane_keys()[0];

        (app, pane_key)
    }

    fn mock_terminal() -> Terminal<TestBackend> {
        let backend = TestBackend::new(140, 40);
        Terminal::new(backend).unwrap()
    }

    fn cleanup(app: App, root_pane: PaneKey) {
        app.tasks
            .get(&root_pane)
            .unwrap()
            .task_handle
            .as_ref()
            .unwrap()
            .abort();
    }

    #[test]
    fn test_app_new_initializes_correctly() {
        let (app, _root_pane) = mock_app();

        assert!(!app.exit);
        assert!(app.tasks.is_empty());
    }

    #[test]
    fn test_app_exit_sets_flag() {
        let (mut app, _root_pane) = mock_app();

        app.exit();
        assert!(app.exit);
    }

    #[tokio::test]
    async fn test_set_command_adds_new_task() {
        let (mut app, root_pane) = mock_app();

        let exec = "echo hello";

        app.set_command(root_pane, exec.to_string()).await;

        assert_eq!(app.tasks.len(), 1);
        let command = app.tasks.get(&root_pane).unwrap();
        assert_eq!(command.exec, exec);
        assert!(matches!(
            command.state,
            CommandState::Executing | CommandState::Idle
        ));
        assert!(command.task_handle.is_some());

        if let Some(handle) = command.task_handle.as_ref() {
            handle.abort();
        }
    }

    #[tokio::test]
    async fn test_set_command_replaces_existing_task() {
        let (mut app, root_pane) = mock_app();

        let initial_exec = "sleep 5";
        let new_exec = "echo replaced";

        app.set_command(root_pane, initial_exec.to_string()).await;

        app.set_command(root_pane, new_exec.to_string()).await;

        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks.get(&root_pane).unwrap().exec, new_exec);

        cleanup(app, root_pane);
    }

    #[tokio::test]
    async fn test_app_control_pause() {
        let (mut app, root_pane) = mock_app();

        app.set_command(root_pane, "sleep 1".to_string()).await;

        app.app_control_tx
            .send(AppControl::SendControl(root_pane, CommandControl::Pause))
            .await
            .unwrap();

        while let Ok(control) = app.app_control_rx.try_recv() {
            println!("Processing: {:?}", control);
            if let AppControl::SendControl(id, cmd_ctrl) = control {
                if let Some(command) = app.tasks.get_mut(&id) {
                    command.handle_control_signal(root_pane, cmd_ctrl).await;
                    println!("Processed SendControl for ID: {:?}", id);
                }
            }
        }

        assert_eq!(
            app.tasks.get(&root_pane).unwrap().state,
            CommandState::Paused
        );

        cleanup(app, root_pane);
    }

    async fn simulate_app(app: &mut App, tasks: i32) -> Result<(), Elapsed> {
        while let Ok(control) = app.app_control_rx.try_recv() {
            println!("Processing: {:?}", control);
            match control {
                AppControl::SendControl(id, cmd_ctrl) => {
                    if let Some(command) = app.tasks.get_mut(&id) {
                        command.handle_control_signal(id, cmd_ctrl).await;
                    }
                }
                AppControl::SetDisplay(id, display) => {
                    if let Some(command) = app.tasks.get_mut(&id) {
                        command.update_display(display);
                    }
                }
                _ => (),
            }
        }

        let mut output_count = 0;
        let target_outputs = tasks;

        timeout(Duration::from_secs(5), async {
            while output_count < target_outputs {
                if let Some((id, event)) = app.output_rx.recv().await {
                    match event {
                        CommandEvent::Started => {
                            if let Some(command) = app.tasks.get_mut(&id) {
                                command.state = crate::command::CommandState::Executing;
                            }
                        }
                        CommandEvent::Output(out) => {
                            if let Some(command) = app.tasks.get_mut(&id) {
                                command.state = crate::command::CommandState::Idle;
                                command.record_output(out, app.config.max_history);
                                output_count += 1;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        })
        .await
    }

    fn render_terminal(terminal: &mut Terminal<TestBackend>, app: &mut App) {
        terminal
            .draw(|frame| ui::draw::draw_ui(app, frame))
            .expect("failed to draw UI");
    }

    pub fn assert_ui_snapshot(name: &str, content: String) {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path("../tests/snapshots");
        settings.bind(|| {
            insta::assert_snapshot!(name, content);
        });
    }

    #[tokio::test]
    async fn test_render_app() {
        let (mut app, root_pane) = mock_app();
        app.config.interval = Duration::from_millis(100);

        let mut terminal = mock_terminal();

        // 1. Set command "ls"
        _ = app.set_command(root_pane, "ls".to_string()).await;
        println!("{:?}", app.tasks);

        // 2. Set counter display
        _ = app
            .app_control_tx
            .send(AppControl::SetDisplay(root_pane, DisplayType::Counter))
            .await;
        println!("{:?}", app.tasks);

        // 3. Split vertically
        app.pane_manager.split_pane(Direction::Vertical);

        // 4. Set command echo "test"
        _ = app
            .set_command(app.pane_manager.active_pane_id, "echo test".to_string())
            .await;
        println!("{:?}", app.tasks);

        // 5. Split horizontally and increase size
        app.pane_manager.split_pane(Direction::Horizontal);
        app.pane_manager.resize_pane(&CardinalDirection::Up, 2);

        // 6. Set command "ls"
        _ = app
            .set_command(app.pane_manager.active_pane_id, "ls".to_string())
            .await;

        // 7. Enter zen mode so that the dynamic part (the date) is not visible
        app.config.zen = true;

        let result = simulate_app(&mut app, 3);
        assert!(result.await.is_ok(), "Timed out waiting for CommandEvents");

        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("zen_mode", terminal.backend().to_string());

        app.config.zen = false;
        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("normal_mode", terminal.backend().to_string());

        app.config.theme.show_display_type = false;
        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot(
            "normal_mode_no_display_type",
            terminal.backend().to_string(),
        );

        app.config.theme.show_status_bar = false;
        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("normal_mode_no_status_bar", terminal.backend().to_string());
    
        cleanup(app, root_pane);
    }

    #[test]
    fn test_render_help_menu() {
        let (mut app, _) = mock_app();
        let mut terminal = mock_terminal();

        app.mode = AppMode::new_help();

        render_terminal(&mut terminal, &mut app);
        // assert_ui_snapshot("help_mode_top", terminal.backend().to_string());

        app.mode._scroll_bottom();
        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("help_mode_bottom", terminal.backend().to_string());
    }

    #[tokio::test]
    async fn test_render_observe_mode() {
        let (mut app, root_pane) = mock_app();
        app.config.interval = Duration::from_secs(60);
        app.mode = AppMode::new_observing(&app);

        _ = app
            .set_command(root_pane, "ls".to_string())
            .await;

        let mut terminal = mock_terminal();

        let result = simulate_app(&mut app, 1);
        assert!(result.await.is_ok(), "Timed out waiting for CommandEvents");

        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("observe_mode", terminal.backend().to_string());
    
        cleanup(app, root_pane);
    }
}
