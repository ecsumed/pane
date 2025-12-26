use std::collections::HashMap;
use std::io::{self, Result};
use std::time::Duration;

use crokey::Combiner;
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use tokio::sync::mpsc::{self};
use tokio::time::interval;

use crate::command::{Command, CommandControl, CommandOutput, CommandSerializableState};
use crate::config::AppConfig;
use crate::controls;
use crate::mode::AppMode;
use crate::pane::{PaneKey, PaneManager};
use crate::logging::{info, error, warn};
use crate::ui::draw::draw_ui;
use crate::ui::DisplayType;

type DefaultTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

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
    pub output_rx: mpsc::Receiver<(PaneKey, CommandOutput)>,
    pub output_tx: mpsc::Sender<(PaneKey, CommandOutput)>,
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
            if let Err(e) = app_control_tx.try_send(AppControl::SetCommand(pane_manager.active_pane_id, command)) {
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

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let mut tick_interval = interval(Duration::from_millis(250));
        let mut events = EventStream::new();

        loop {
            terminal.draw(|frame| {
                self.pane_area = frame.area();

                draw_ui(self, frame);
            })?;

            tokio::select! {
                Some((id, output)) = self.output_rx.recv() => {
                    if let Some(code) = output.exit_status {
                        if code != 0 && self.config.beep {
                            App::beep()
                        }
                        if code != 0 && self.config.err_exit {
                            info!("Exiting because err_exit was set.");
                            self.exit();
                        }
                    }

                    if let Some(command) = self.tasks.get_mut(&id) {
                        command.record_output(output, self.config.max_history);
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
            if let Err(e) = self.app_control_tx
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
            Command::spawn(id, exec, self.config.default_display, self.config.interval, self.output_tx.clone()),
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::command::{CommandControl, CommandState};
//     use crate::config::AppConfig;
//     use ratatui::backend::TestBackend;
//     use std::time::Duration;

//     fn mock_config() -> AppConfig {
//         AppConfig::load().expect("")
//     }

//     #[test]
//     fn test_app_new_initializes_correctly() {
//         let config = mock_config();
//         let app = App::new(config);

//         assert!(!app.exit);
//         assert!(app.tasks.is_empty());
//         // assert_eq!(app.pane_manager.active_pane_id, 1);
//     }

//     #[test]
//     fn test_app_exit_sets_flag() {
//         let config = mock_config();
//         let mut app = App::new(config);

//         app.exit();
//         assert!(app.exit);
//     }

//     #[tokio::test]
//     async fn test_set_command_adds_new_task() {
//         let config = mock_config();
//         let mut app = App::new(config);

//         // Get the key for the initial root pane
//         let pane_key = app.pane_manager.get_all_pane_keys()[0];

//         let exec = "echo hello";

//         app.set_command(pane_key, exec.to_string()).await;

//         assert_eq!(app.tasks.len(), 1);
//         let command = app.tasks.get(&pane_key).unwrap();
//         assert_eq!(command.exec, exec);
//         assert_eq!(command.state, CommandState::Running);
//         assert!(command.task_handle.is_some());

//         if let Some(handle) = command.task_handle.as_ref() {
//             handle.abort();
//         }
//     }

//     #[tokio::test]
//     async fn test_set_command_replaces_existing_task() {
//         let config = mock_config();
//         let mut app = App::new(config);
//         let id = 1;
//         let initial_exec = "sleep 5";
//         let new_exec = "echo replaced";

//         app.set_command(id, initial_exec.to_string()).await;

//         app.set_command(id, new_exec.to_string()).await;

//         assert_eq!(app.tasks.len(), 1);
//         assert_eq!(app.tasks.get(&id).unwrap().exec, new_exec);

//         app.tasks
//             .get(&id)
//             .unwrap()
//             .task_handle
//             .as_ref()
//             .unwrap()
//             .abort();
//     }

//     #[test]
//     fn test_draw_calls_ui_functions() {
//         let config = mock_config();
//         let app = App::new(config);
//         let backend = TestBackend::new(50, 10);
//         let mut terminal = Terminal::new(backend).unwrap();

//         terminal
//             .draw(|frame| app.draw(frame))
//             .expect("draw should complete without panicking");
//     }

//     #[tokio::test]
//     async fn test_app_control_pause_resume_stop() {
//         let config = mock_config();
//         let mut app = App::new(config);
//         let id = 1;

//         app.set_command(id, "sleep 1".to_string()).await;

//         let control_msg = AppControl::SendControl(id, CommandControl::Pause);
//         app.app_control_tx.send(control_msg).await.unwrap();

//         if let Some(control) = app.app_control_rx.recv().await {
//             if let AppControl::SendControl(id, cmd_ctrl) = control {
//                 if let Some(command) = app.tasks.get_mut(&id) {
//                     match cmd_ctrl {
//                         CommandControl::Pause => command.state = CommandState::Paused,
//                         _ => {}
//                     }
//                 }
//             }
//         }

//         assert_eq!(app.tasks.get(&id).unwrap().state, CommandState::Paused);

//         app.tasks
//             .get(&id)
//             .unwrap()
//             .task_handle
//             .as_ref()
//             .unwrap()
//             .abort();
//     }
// }
