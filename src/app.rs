use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::watch;
use tokio::time::interval;

use crate::command::{Command, CommandControl, CommandSerializableState, CommandState};
use crate::config::AppConfig;
use crate::mode::InputState;
use crate::pane::{PaneKey, PaneManager};
use crate::{controls, ui};
use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use std::io::{self, Result};

use crate::logging::{debug, info, warn};

type DefaultTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug)]
pub enum AppControl {
    SetCommand(PaneKey, String),
    SendControl(PaneKey, CommandControl),
}

pub struct App {
    pub pane_manager: PaneManager,
    pub tasks: HashMap<PaneKey, Command>,
    pub input_state: InputState,
    pub exit: bool,
    pub output_rx: mpsc::Receiver<(PaneKey, String)>,
    pub output_tx: mpsc::Sender<(PaneKey, String)>,
    pub app_control_tx: mpsc::Sender<AppControl>,
    pub app_control_rx: mpsc::Receiver<AppControl>,
    pub config: AppConfig,
    pub pane_area: Rect,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (app_control_tx, app_control_rx) = mpsc::channel(10);
        Self {
            pane_manager: PaneManager::new(),
            tasks: HashMap::new(),
            input_state: InputState::default(),
            exit: false,
            output_rx,
            output_tx,
            app_control_tx,
            app_control_rx,
            config,
            pane_area: Rect::new(0, 0, 0, 0),
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut tick_interval = interval(Duration::from_millis(250));
        let mut events = EventStream::new();

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            ui::manage_cursor(self, terminal)?;

            tokio::select! {
                Some((id, output)) = self.output_rx.recv() => {
                    if let Some(command) = self.tasks.get_mut(&id) {
                        command.last_output = output;
                        command.output_history.push(command.last_output.clone());
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
                                match cmd_ctrl {
                                    CommandControl::Pause => command.state = CommandState::Paused,
                                    CommandControl::Resume => command.state = CommandState::Running,
                                    CommandControl::Stop => command.state = CommandState::Stopped,
                                    CommandControl::IncreaseInterval => {
                                        command.interval += Duration::from_secs(1);
                                    },
                                    CommandControl::DecreaseInterval => {
                                        command.interval -= Duration::from_secs(1);
                                    },
                                    // CommandControl::SetInterval(new_interval) => {
                                    //     command.interval = new_interval;
                                    // }
                                    _ => {}
                                }
                                if let Err(e) = command.control_tx.send(cmd_ctrl).await {
                                    warn!("Failed to send command control: {}", e);
                                }
                            }
                        },
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

    fn draw(&mut self, frame: &mut Frame) {
        self.pane_area = frame.area();
        ui::draw_main_ui(self, frame);
        ui::draw_input_popup(self, frame);
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub async fn set_command(&mut self, id: PaneKey, exec: String) {
        if let Some(mut existing_cmd) = self.tasks.remove(&id) {
            if let Some(handle) = existing_cmd.task_handle.take() {
                handle.abort();
            }
        }

        let (control_tx, control_rx) = mpsc::channel(1);
        let output_tx_clone = self.output_tx.clone();
        let interval = Duration::from_secs(5);

        let cmd = exec.clone();
        let task_handle = tokio::spawn(async move {
            Command::run_command_task(id, cmd, interval, CommandState::Running, control_rx, output_tx_clone).await;
        });

        info!("Adding new command: {}", &exec);
        let new_cmd = Command {
            exec,
            interval,
            output_history: Vec::new(),
            last_output: String::new(),
            state: CommandState::Running,
            task_handle: Some(task_handle),
            control_tx,
        };
        self.tasks.insert(id, new_cmd);
    }

    pub fn load_session(
        &mut self,
        pane_manager: PaneManager,
        tasks_state: HashMap<PaneKey, CommandSerializableState>,
    ) -> io::Result<()> {
        let mut running_tasks = HashMap::new();

        for (id, state) in tasks_state.into_iter() {
            let (control_tx, control_rx) = mpsc::channel(1);
            let output_tx_clone = self.output_tx.clone(); // Clone the sender for the new task
            let interval = state.interval;
            let cmd_exec = state.exec.clone();
            let cmd_state = state.state;
            let task_handle = tokio::spawn(async move {
                Command::run_command_task(id, cmd_exec, interval, cmd_state, control_rx, output_tx_clone)
                    .await;
            });

            info!("Restarting command for id: {:?}", id);
            let new_cmd = Command {
                exec: state.exec,
                interval,
                output_history: state.output_history,
                last_output: state.last_output,
                state: state.state,
                task_handle: Some(task_handle),
                control_tx,
            };
            running_tasks.insert(id, new_cmd);
        }

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
