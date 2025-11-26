// src/task_manager.rs
use std::collections::HashMap;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tracing::{info, warn};
use tokio::time::Duration;
use crate::app::{AppControl, TaskStatus};
use crate::command::{CommandActor, CommandControl, CommandState};

// Command is now a simpler struct for sending to the App
#[derive(Debug, Clone)]
pub struct Command {
    pub id: usize,
    pub exec: String,
    pub interval: Duration,
    pub output_history: Vec<String>,
    pub last_output: String,
    pub state: CommandState,
    pub control_tx: mpsc::Sender<CommandControl>,
    pub pause_tx: watch::Sender<bool>,
    pub(crate) task_handle: Option<JoinHandle<()>>,
}

// Internal to TaskManager, this holds non-clonable data
#[derive(Debug)]
struct TaskMetadata {
    pub task_handle: JoinHandle<()>,
    pub control_tx: mpsc::Sender<CommandControl>,
    pub pause_tx: watch::Sender<bool>,
}

#[derive(Debug)]
pub struct TaskManager {
    pub tasks: HashMap<usize, TaskMetadata>,
    task_status_tx: mpsc::Sender<TaskStatus>,
}

impl TaskManager {
    pub fn new(task_status_tx: mpsc::Sender<TaskStatus>) -> Self {
        Self {
            tasks: HashMap::new(),
            task_status_tx,
        }
    }

    pub async fn run(&mut self, mut app_control_rx: mpsc::Receiver<AppControl>) {
        info!("Task manager running.");

        loop {
            tokio::select! {
                Some(app_control) = app_control_rx.recv() => {
                    match app_control {
                        AppControl::SetCommand(id, exec) => {
                            self.set_command(id, exec);
                        },
                        AppControl::SendControl(id, control) => {
                            self.send_control(id, control).await; // <-- Needs to be awaited
                        }
                    }
                }
            }
        }
    }

    pub async fn send_control(&mut self, id: usize, control: CommandControl) {
        if let Some(metadata) = self.tasks.get_mut(&id) {
            match control {
                CommandControl::Pause => {
                    if let Err(e) = metadata.pause_tx.send(true) {
                        warn!("Failed to send pause signal to pane {}: {}", id, e);
                    }
                    if let Err(e) = self.task_status_tx.send(TaskStatus::TaskStateChanged(id, CommandState::Paused)).await {
                        warn!("Failed to send state change status: {}", e);
                    }
                }
                CommandControl::Resume => {
                    if let Err(e) = metadata.pause_tx.send(false) {
                        warn!("Failed to send resume signal to pane {}: {}", id, e);
                    }
                    if let Err(e) = self.task_status_tx.send(TaskStatus::TaskStateChanged(id, CommandState::Running)).await {
                        warn!("Failed to send state change status: {}", e);
                    }
                }
                CommandControl::Stop => {
                    if let Err(e) = metadata.control_tx.try_send(control) {
                        warn!("Failed to send control message to pane {}: {}", id, e);
                    }
                    if let Err(e) = self.task_status_tx.send(TaskStatus::TaskStateChanged(id, CommandState::Stopped)).await {
                        warn!("Failed to send state change status: {}", e);
                    }
                }
                _ => {
                    if let Err(e) = metadata.control_tx.try_send(control) {
                        warn!("Failed to send control message to pane {}: {}", id, e);
                    }
                }
            }
        }
    }

    pub async fn set_command(&mut self, id: usize, exec: String) {
        if let Some(metadata) = self.tasks.remove(&id) {
            metadata.task_handle.abort();
        }

        let (command_control_tx, command_control_rx) = mpsc::channel(1);
        let (pause_tx, pause_rx) = watch::channel(false);
        let interval = Duration::from_secs(5);
        
        let new_actor = CommandActor::new(
            id,
            exec.clone(),
            interval,
            pause_rx,
            self.task_status_tx.clone(),
        );

        let task_handle = tokio::spawn(async move {
            new_actor.run(command_control_rx).await;
        });

        let new_cmd_for_app = Command {
            id,
            exec: exec.clone(),
            interval,
            output_history: Vec::new(),
            last_output: String::new(),
            state: CommandState::Running,
            control_tx: command_control_tx.clone(),
            pause_tx: pause_tx.clone(),
        };

        // Send a message to the App indicating the task was created
        if let Err(e) = self.task_status_tx.send(TaskStatus::TaskCreated(id, new_cmd_for_app)).await {
            warn!("Failed to send task creation status: {}", e);
        }

        // Store metadata internally for TaskManager's use
        self.tasks.insert(id, TaskMetadata {
            task_handle,
            control_tx: command_control_tx,
            pause_tx,
        });
        info!("Pane {} task spawned.", id);
    }
}
