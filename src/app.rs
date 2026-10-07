use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::time::{Duration, Instant};

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
use crate::mode::{AppMode, DiffMode};
use crate::pane::{PaneKey, PaneManager};
use crate::ui::draw::draw_ui;
use crate::ui::DisplayType;

#[derive(Debug)]
pub enum AppControl {
    SetCommand(PaneKey, String),
    SendControl(PaneKey, CommandControl),
    SetDisplay(PaneKey, DisplayType),
}

const NOTICE_DURATION: Duration = Duration::from_secs(4);
const FRAME_INTERVAL: Duration = Duration::from_millis(100);

fn frame_due(dirty: bool, urgent: bool, since_last_draw: Duration) -> bool {
    dirty && (urgent || since_last_draw >= FRAME_INTERVAL)
}

#[derive(Debug)]
pub struct Notice {
    pub text: String,
    pub is_error: bool,
    shown_at: Instant,
}

pub struct App {
    pub pane_manager: PaneManager,
    pub tasks: HashMap<PaneKey, Command>,
    pub mode: AppMode,
    pub exit: bool,
    pub output_rx: mpsc::Receiver<(PaneKey, CommandEvent)>,
    pub output_tx: mpsc::Sender<(PaneKey, CommandEvent)>,
    pub app_control_tx: mpsc::UnboundedSender<AppControl>,
    pub app_control_rx: mpsc::UnboundedReceiver<AppControl>,
    pub config: AppConfig,
    pub pane_rects: Vec<(PaneKey, Rect)>,
    pub observe_diff_mode: DiffMode,
    pub notice: Option<Notice>,
}

impl App {
    pub fn new(config: AppConfig, command: Vec<String>) -> Self {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (app_control_tx, app_control_rx) = mpsc::unbounded_channel();

        let pane_manager = PaneManager::new();

        if !command.is_empty() {
            let command = command.join(" ");
            if let Err(e) =
                app_control_tx.send(AppControl::SetCommand(pane_manager.active_pane_id, command))
            {
                error!("Send failed: {}", e);
            }
        }

        Self {
            pane_manager,
            tasks: HashMap::new(),
            mode: AppMode::default(),
            exit: false,
            output_rx,
            output_tx,
            app_control_tx,
            app_control_rx,
            config,
            pane_rects: Vec::new(),
            observe_diff_mode: DiffMode::default(),
            notice: None,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()>
    where
        B: Backend,
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut tick_interval = interval(Duration::from_millis(250));
        let mut events = EventStream::new();
        let mut dirty = true;
        let mut urgent = true;
        let mut last_draw = Instant::now();

        loop {
            if frame_due(dirty, urgent, last_draw.elapsed()) {
                terminal.draw(|frame| draw_ui(self, frame))?;
                last_draw = Instant::now();
                dirty = false;
                urgent = false;
            }
            let next_frame = tokio::time::Instant::from_std(last_draw + FRAME_INTERVAL);

            tokio::select! {
                Some((id, event)) = self.output_rx.recv() => {
                    self.handle_command_event(id, event);
                    self.drain_command_events();
                    dirty = true;
                },
                Some(Ok(event)) = events.next().fuse() => {
                    controls::handle_event(self, event).await?;
                    dirty = true;
                    urgent = true;
                },
                Some(control) = self.app_control_rx.recv() => {
                    self.handle_app_control(control).await;
                    while let Ok(control) = self.app_control_rx.try_recv() {
                        self.handle_app_control(control).await;
                    }
                    dirty = true;
                    urgent = true;
                },
                _ = tokio::time::sleep_until(next_frame), if dirty => {},
                _ = tick_interval.tick() => {
                    dirty |= self.expire_notice();
                },
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn drain_command_events(&mut self) {
        while let Ok((id, event)) = self.output_rx.try_recv() {
            self.handle_command_event(id, event);
        }
    }

    async fn handle_app_control(&mut self, control: AppControl) {
        match control {
            AppControl::SetCommand(id, exec) => {
                self.set_command(id, exec).await;
            }
            AppControl::SendControl(id, cmd_ctrl) => {
                if let Some(command) = self.tasks.get_mut(&id) {
                    command.handle_control_signal(id, cmd_ctrl);
                }
            }
            AppControl::SetDisplay(id, display) => {
                if let Some(command) = self.tasks.get_mut(&id) {
                    command.update_display(display);
                    crate::ui::refresh_view(&self.config.theme, command);
                }
            }
        }
    }

    fn expire_notice(&mut self) -> bool {
        if self.notice.is_some() && self.current_notice().is_none() {
            self.notice = None;
            return true;
        }
        false
    }

    pub fn notify(&mut self, text: impl Into<String>, is_error: bool) {
        self.notice = Some(Notice {
            text: text.into(),
            is_error,
            shown_at: Instant::now(),
        });
    }

    pub fn current_notice(&self) -> Option<&Notice> {
        self.notice
            .as_ref()
            .filter(|notice| notice.shown_at.elapsed() < NOTICE_DURATION)
    }

    pub fn handle_command_event(&mut self, id: PaneKey, event: CommandEvent) {
        let CommandEvent::Output(out) = event else {
            if let Some(command) = self.tasks.get_mut(&id) {
                command.state = crate::command::CommandState::Executing;
            }
            return;
        };

        let failed = out.exit_status.is_some_and(|code| code != 0);
        if failed && self.config.beep {
            App::beep()
        }

        let Some(command) = self.tasks.get_mut(&id) else {
            return;
        };
        let changed = command
            .last_output()
            .is_some_and(|last| last.output != out.output);
        command.state = crate::command::CommandState::Idle;
        command.record_output(out, self.config.max_history);
        crate::ui::refresh_view(&self.config.theme, command);

        if failed && self.config.err_exit {
            info!("Exiting because err_exit was set.");
            self.exit();
        } else if changed && self.config.chg_exit {
            info!("Exiting because chg_exit was set and the output changed.");
            self.exit();
        }
    }

    pub fn exit(&mut self) {
        for pane_key in self.tasks.keys() {
            if let Err(e) = self
                .app_control_tx
                .send(AppControl::SendControl(*pane_key, CommandControl::Stop))
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
                self.config.timeout,
                self.output_tx.clone(),
            ),
        ) {
            if let Some(h) = old.task_handle {
                h.abort();
            }
        }
    }

    pub fn kill_active_pane(&mut self) {
        let id = self.pane_manager.active_pane_id;
        self.pane_manager.kill_pane();

        if self.pane_manager.nodes.contains_key(id) {
            return;
        }
        if let Some(command) = self.tasks.remove(&id) {
            if let Some(h) = command.task_handle {
                h.abort();
            }
        }
    }

    pub fn load_session(
        &mut self,
        pane_manager: PaneManager,
        tasks_state: HashMap<PaneKey, CommandSerializableState>,
    ) -> io::Result<()> {
        for (_, old) in self.tasks.drain() {
            if let Some(h) = old.task_handle {
                h.abort();
            }
        }

        let panes: HashSet<PaneKey> = pane_manager.get_all_pane_keys().into_iter().collect();
        let (tasks_state, orphans): (HashMap<_, _>, HashMap<_, _>) = tasks_state
            .into_iter()
            .partition(|(id, _)| panes.contains(id));
        for (id, task) in orphans {
            warn!("Skipping command without a pane {:?}: {}", id, task.exec);
        }

        let running_tasks =
            Command::restore_tasks(tasks_state, self.config.timeout, self.output_tx.clone());

        self.pane_manager = pane_manager;
        self.tasks = running_tasks;
        for command in self.tasks.values_mut() {
            crate::ui::refresh_view(&self.config.theme, command);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandControl, CommandState};
    use crate::config::AppConfig;
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
            .unwrap();

        while let Ok(control) = app.app_control_rx.try_recv() {
            println!("Processing: {:?}", control);
            if let AppControl::SendControl(id, cmd_ctrl) = control {
                if let Some(command) = app.tasks.get_mut(&id) {
                    command.handle_control_signal(root_pane, cmd_ctrl);
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
                        command.handle_control_signal(id, cmd_ctrl);
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
            .send(AppControl::SetDisplay(root_pane, DisplayType::Counter));
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
        app.pane_manager.resize_active(Direction::Horizontal, true);
        app.pane_manager.resize_active(Direction::Horizontal, true);

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

        app.config.theme.show_history_meter = false;
        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot(
            "normal_mode_no_history_meter",
            terminal.backend().to_string(),
        );

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
    async fn test_controls_do_not_block_during_long_run() {
        let (mut app, root_pane) = mock_app();

        app.set_command(root_pane, "sleep 5".to_string()).await;

        let started = timeout(Duration::from_secs(2), app.output_rx.recv()).await;
        assert!(matches!(started, Ok(Some((_, CommandEvent::Started)))));

        let command = app.tasks.get_mut(&root_pane).unwrap();
        let sent = timeout(Duration::from_millis(100), async {
            for _ in 0..5 {
                command.handle_control_signal(root_pane, CommandControl::IntervalIncrease);
            }
        })
        .await;
        assert!(
            sent.is_ok(),
            "Control signals blocked while command was running"
        );

        command.handle_control_signal(root_pane, CommandControl::Stop);
        let handle = command.task_handle.take().unwrap();
        let stopped = timeout(Duration::from_secs(1), handle).await;
        assert!(
            stopped.is_ok(),
            "Task did not stop while command was running"
        );
    }

    #[tokio::test]
    async fn test_load_session_stops_previous_tasks() {
        let (mut app, root_pane) = mock_app();

        app.set_command(root_pane, "sleep 5".to_string()).await;
        let old_task = app.tasks[&root_pane]
            .task_handle
            .as_ref()
            .unwrap()
            .abort_handle();

        app.load_session(PaneManager::new(), HashMap::new())
            .unwrap();
        tokio::task::yield_now().await;

        assert!(app.tasks.is_empty());
        assert!(
            old_task.is_finished(),
            "Previous session task still running"
        );
    }

    fn serialized(exec: &str) -> CommandSerializableState {
        CommandSerializableState {
            exec: exec.to_string(),
            interval: Duration::from_secs(60),
            output_history: Default::default(),
            state: CommandState::Paused,
            display_type: DisplayType::RawText,
        }
    }

    #[tokio::test]
    async fn test_closing_a_pane_stops_and_removes_its_command() {
        let (mut app, _) = mock_app();
        app.pane_manager.split_pane(Direction::Vertical);
        let closed = app.pane_manager.active_pane_id;
        app.set_command(closed, "sleep 6.17".to_string()).await;
        let task = app.tasks[&closed]
            .task_handle
            .as_ref()
            .unwrap()
            .abort_handle();
        tokio::time::sleep(Duration::from_millis(100)).await;

        app.kill_active_pane();
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert!(!app.tasks.contains_key(&closed));
        assert!(task.is_finished());
        let leftover = std::process::Command::new("pgrep")
            .args(["-f", "sleep 6.17"])
            .output()
            .unwrap();
        assert!(
            leftover.stdout.is_empty(),
            "closed pane's process still running"
        );
    }

    #[tokio::test]
    async fn test_closing_the_last_pane_keeps_its_command() {
        let (mut app, root_pane) = mock_app();
        app.set_command(root_pane, "sleep 5".to_string()).await;

        app.kill_active_pane();
        tokio::task::yield_now().await;

        let task = app.tasks.get(&root_pane).expect("command removed");
        assert!(!task.task_handle.as_ref().unwrap().is_finished());
        cleanup(app, root_pane);
    }

    #[tokio::test]
    async fn test_load_session_skips_commands_without_a_pane() {
        let (mut app, _) = mock_app();
        let pane_manager = PaneManager::new();
        let pane = pane_manager.active_pane_id;
        let orphan = PaneKey::from(slotmap::KeyData::from_ffi(12884901896));

        app.load_session(
            pane_manager,
            HashMap::from([(pane, serialized("kept")), (orphan, serialized("orphan"))]),
        )
        .unwrap();

        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks[&pane].exec, "kept");
        for task in app.tasks.values() {
            task.task_handle.as_ref().unwrap().abort();
        }
    }

    #[tokio::test]
    async fn test_save_session_skips_commands_without_a_pane() {
        let dir = std::env::temp_dir().join(format!("pane-save-orphans-{}", std::process::id()));
        let (mut app, root_pane) = mock_app();
        app.config.sessions_dir = dir.clone();
        let orphan = PaneKey::from(slotmap::KeyData::from_ffi(12884901896));
        app.set_command(root_pane, "sleep 5".to_string()).await;
        app.set_command(orphan, "sleep 5".to_string()).await;

        crate::session::save_session_by_name(&app, "orphans").unwrap();
        let preview = crate::session::load_session_preview(&app.config, "orphans.toml");
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(preview.unwrap().panes.len(), 1);
        for task in app.tasks.values() {
            task.task_handle.as_ref().unwrap().abort();
        }
    }

    fn output(text: &str, code: i32) -> CommandEvent {
        CommandEvent::Output(crate::command::CommandOutput {
            output: text.to_string(),
            time: chrono::Local::now().naive_local(),
            exit_status: Some(code),
            duration: Duration::from_millis(1),
        })
    }

    #[tokio::test]
    async fn test_chg_exit_exits_only_when_output_changes() {
        let (mut app, root_pane) = mock_app();
        app.config.chg_exit = true;
        app.set_command(root_pane, "sleep 5".to_string()).await;

        app.handle_command_event(root_pane, output("same", 0));
        app.handle_command_event(root_pane, output("same", 0));
        assert!(!app.exit, "exited without a change");

        app.handle_command_event(root_pane, output("different", 0));
        assert!(app.exit);
        assert_eq!(app.tasks[&root_pane].output_history.len(), 3);
        cleanup(app, root_pane);
    }

    #[tokio::test]
    async fn test_err_exit_still_exits_on_failure() {
        let (mut app, root_pane) = mock_app();
        app.config.err_exit = true;
        app.set_command(root_pane, "sleep 5".to_string()).await;

        app.handle_command_event(root_pane, output("fine", 0));
        assert!(!app.exit);
        app.handle_command_event(root_pane, output("boom", 1));
        assert!(app.exit);
        cleanup(app, root_pane);
    }

    async fn press_key(app: &mut App, c: char) {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
        let event = Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        controls::handle_event(app, event).await.unwrap();
    }

    fn status_line(app: &mut App) -> (String, Option<ratatui::style::Color>) {
        let mut terminal = Terminal::new(TestBackend::new(100, 10)).unwrap();
        render_terminal(&mut terminal, app);
        let buffer = terminal.backend().buffer();
        let y = buffer.area.height - 1;
        let text: String = (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect();
        let x = text.find("Saved").or(text.find("Couldn't")).unwrap_or(0) as u16;
        (text, Some(buffer[(x, y)].fg))
    }

    #[tokio::test]
    async fn test_saving_a_session_shows_a_notice() {
        let dir = std::env::temp_dir().join(format!("pane-notice-{}", std::process::id()));
        let (mut app, _) = mock_app();
        app.config.sessions_dir = dir.clone();

        press_key(&mut app, 's').await;
        let (line, _) = status_line(&mut app);
        std::fs::remove_dir_all(&dir).unwrap();

        assert!(line.contains("Saved session session-"), "{line}");
    }

    #[tokio::test]
    async fn test_failed_save_shows_an_error_notice() {
        let file = std::env::temp_dir().join(format!("pane-notice-file-{}", std::process::id()));
        std::fs::write(&file, "").unwrap();
        let (mut app, _) = mock_app();
        app.config.sessions_dir = file.join("sessions");

        press_key(&mut app, 's').await;
        let (line, colour) = status_line(&mut app);
        std::fs::remove_file(&file).unwrap();

        assert!(line.contains("Couldn't save session"), "{line}");
        assert_eq!(colour, app.config.theme.palette.error.fg);
    }

    #[test]
    fn test_notices_expire() {
        let (mut app, _) = mock_app();
        app.notify("hello", false);
        assert!(app.current_notice().is_some());

        app.notice.as_mut().unwrap().shown_at = Instant::now() - NOTICE_DURATION;
        assert!(app.current_notice().is_none());
    }

    #[test]
    fn test_command_output_redraws_at_most_ten_times_a_second() {
        let soon = FRAME_INTERVAL / 2;
        assert!(!frame_due(false, false, FRAME_INTERVAL));
        assert!(!frame_due(true, false, soon));
        assert!(frame_due(true, false, FRAME_INTERVAL));
        assert!(frame_due(true, true, Duration::ZERO));
    }

    #[test]
    fn test_tick_redraws_only_to_clear_an_expired_notice() {
        let (mut app, _) = mock_app();
        assert!(!app.expire_notice());

        app.notify("hello", false);
        assert!(!app.expire_notice());

        app.notice.as_mut().unwrap().shown_at = Instant::now() - NOTICE_DURATION;
        assert!(app.expire_notice());
        assert!(app.notice.is_none());
        assert!(!app.expire_notice());
    }

    #[tokio::test]
    async fn test_pending_command_events_are_handled_together() {
        let (mut app, root_pane) = mock_app();
        app.set_command(root_pane, "sleep 5".to_string()).await;
        for i in 0..5 {
            app.output_tx
                .send((root_pane, output(&format!("run {i}"), 0)))
                .await
                .unwrap();
        }

        app.drain_command_events();

        assert_eq!(app.tasks[&root_pane].output_history.len(), 5);
        cleanup(app, root_pane);
    }

    #[tokio::test]
    async fn test_pane_keys_move_focus_and_borders_using_the_drawn_layout() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let (mut app, left) = mock_app();
        app.pane_manager.split_pane(Direction::Horizontal);
        let right = app.pane_manager.active_pane_id;
        let mut terminal = mock_terminal();
        let width_of = |app: &App, key: PaneKey| {
            app.pane_rects
                .iter()
                .find(|(k, _)| *k == key)
                .unwrap()
                .1
                .width
        };
        let press = |code, modifiers| Event::Key(KeyEvent::new(code, modifiers));

        render_terminal(&mut terminal, &mut app);
        let start = width_of(&app, right);

        controls::handle_event(&mut app, press(KeyCode::Left, KeyModifiers::NONE))
            .await
            .unwrap();
        assert_eq!(app.pane_manager.active_pane_id, left);

        controls::handle_event(&mut app, press(KeyCode::Right, KeyModifiers::SHIFT))
            .await
            .unwrap();
        render_terminal(&mut terminal, &mut app);
        assert!(width_of(&app, right) < start);

        controls::handle_event(&mut app, press(KeyCode::Char('='), KeyModifiers::NONE))
            .await
            .unwrap();
        render_terminal(&mut terminal, &mut app);
        assert_eq!(width_of(&app, right), start);
    }

    #[tokio::test]
    async fn test_angle_brackets_move_the_border_the_way_they_point() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let (mut app, _) = mock_app();
        app.pane_manager.split_pane(Direction::Vertical);
        app.pane_manager.split_pane(Direction::Horizontal);
        let bottom_right = app.pane_manager.active_pane_id;
        let mut terminal = mock_terminal();
        let left_edge = |app: &App| {
            app.pane_rects
                .iter()
                .find(|(k, _)| *k == bottom_right)
                .unwrap()
                .1
                .x
        };
        let mut press = async |c: char| {
            let event = Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            controls::handle_event(&mut app, event).await.unwrap();
            render_terminal(&mut terminal, &mut app);
            left_edge(&app)
        };

        let start = press('=').await;
        let after_left = press('<').await;
        assert!(after_left < start, "< should move the border left");
        let after_right = press('>').await;
        assert_eq!(after_right, start, "> should move it back right");
        assert!(press('>').await > start);
    }

    fn view_text(app: &App, id: PaneKey) -> Option<String> {
        app.tasks[&id].diff_view.as_ref().map(|lines| {
            lines
                .iter()
                .flat_map(|l| l.spans.iter().map(|s| s.content.to_string()))
                .collect()
        })
    }

    #[tokio::test]
    async fn test_diff_view_is_computed_once_per_run_and_reused_when_drawing() {
        let (mut app, root_pane) = mock_app();
        app.set_command(root_pane, "sleep 5".to_string()).await;
        app.handle_app_control(AppControl::SetDisplay(root_pane, DisplayType::DiffWord))
            .await;
        assert_eq!(view_text(&app, root_pane).as_deref(), Some(""));

        app.handle_command_event(root_pane, output("pods: 3", 0));
        app.handle_command_event(root_pane, output("pods: 4", 0));
        assert_eq!(view_text(&app, root_pane).as_deref(), Some("pods: 4"));

        app.tasks.get_mut(&root_pane).unwrap().diff_view =
            Some(vec![ratatui::text::Line::from("CACHED-VIEW")]);
        let mut terminal = mock_terminal();
        render_terminal(&mut terminal, &mut app);
        assert!(terminal.backend().to_string().contains("CACHED-VIEW"));

        app.handle_app_control(AppControl::SetDisplay(root_pane, DisplayType::RawText))
            .await;
        assert_eq!(view_text(&app, root_pane), None);
        cleanup(app, root_pane);
    }

    #[tokio::test]
    async fn test_render_observe_mode() {
        let (mut app, root_pane) = mock_app();
        app.config.interval = Duration::from_secs(60);
        app.mode = AppMode::new_observing(&app);

        _ = app.set_command(root_pane, "ls".to_string()).await;

        let mut terminal = mock_terminal();

        let result = simulate_app(&mut app, 1);
        assert!(result.await.is_ok(), "Timed out waiting for CommandEvents");

        for out in app
            .tasks
            .get_mut(&root_pane)
            .unwrap()
            .output_history
            .iter_mut()
        {
            out.time = chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap();
            out.duration = Duration::from_millis(20);
        }

        render_terminal(&mut terminal, &mut app);
        assert_ui_snapshot("observe_mode", terminal.backend().to_string());

        cleanup(app, root_pane);
    }
}
