use serde::{de, Deserialize, Serialize};
use std::{fmt, io};
use tokio::process::Command as SysCommand;
use tokio::sync::{mpsc, watch};
use tokio::time::{self, Duration};

use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};

use crate::logging::{debug, info, warn};
use crate::pane::PaneKey;
use crate::ui::DisplayType;

use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandControl {
    Resume,
    Stop,
    Pause,
    IncreaseInterval,
    DecreaseInterval,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CommandState {
    Running,
    Paused,
    Stopped,
}

impl fmt::Display for CommandState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandState::Running => write!(f, "RUNNING"),
            CommandState::Paused => write!(f, "PAUSED"),
            CommandState::Stopped => write!(f, "STOPPED"),
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CommandSerializableState {
    pub exec: String,
    pub interval: Duration,
    pub output_history: Vec<String>,
    pub last_output: String,
    pub state: CommandState,
    pub display_type: DisplayType,
}

#[derive(Debug)]
pub struct Command {
    pub exec: String,
    pub interval: Duration,
    pub output_history: Vec<String>,
    pub last_output: String,
    pub state: CommandState,
    pub display_type: DisplayType,
    pub task_handle: Option<JoinHandle<()>>,
    pub control_tx: mpsc::Sender<CommandControl>,
}

impl Command {
    pub async fn run_command_task(
        id: PaneKey,
        exec: String,
        interval: Duration,
        state: CommandState,
        mut control_rx: mpsc::Receiver<CommandControl>,
        output_tx: tokio::sync::mpsc::Sender<(PaneKey, String)>,
    ) {
        let interval_duration = interval;
        let mut interval = time::interval(interval);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        let mut is_paused = match state {
            CommandState::Paused => true,
            _ => false
        };

        loop {
            tokio::select! {
                Some(control) = control_rx.recv() => {
                    match control {
                        CommandControl::Stop => {
                            info!("Pane {:?} task received stop command.", id);
                            break;
                        }
                        // CommandControl::SetInterval(new_interval) => {
                        //     info!("Pane {} interval changed to {:?}", id, new_interval);
                        //     interval = time::interval(new_interval);
                        //     interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                        // }
                        CommandControl::IncreaseInterval => {
                            info!("Pane {:?} task received increase interval.", id);
                            // TODO: increase interval
                        }
                        CommandControl::DecreaseInterval => {
                            info!("Pane {:?} task received increase interval.", id);
                            // TODO: decrease interval
                        }
                        CommandControl::Pause => {
                            info!("Pane {:?} paused", id);
                            is_paused = true;
                        }
                        CommandControl::Resume => {
                            info!("Pane {:?} resumed", id);
                            is_paused = false;
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} resume execution failed: {}", id, e);
                            }
                            let now = time::Instant::now();
                            interval.reset_at(now + interval_duration);
                        }
                        CommandControl::Execute => {
                            info!("Pane {:?} received ad-hoc execution command.", id);
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} ad-hoc execution failed: {}", id, e);
                            }
                            let now = time::Instant::now();
                            interval.reset_at(now + interval_duration);
                        }
                    }
                }
                _ = interval.tick(), if !is_paused => {
                    info!("Pane {:?} task running command: {}", id, exec);
                    if let Err(e) = Self::run_and_send_output(
                        id,
                        &exec,
                        output_tx.clone(),
                    ).await {
                        warn!("Pane {:?} task failed to run command: {}", id, e);
                    }
                }
            }
        }
    }

    async fn run_and_send_output(
        id: PaneKey,
        exec: &str,
        output_tx: mpsc::Sender<(PaneKey, String)>,
    ) -> Result<(), io::Error> {
        let mut command = SysCommand::new("sh")
            .arg("-c")
            .arg(exec)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout_output = String::new();
        let mut stderr_output = String::new();

        if let Some(stdout) = command.stdout.take() {
            let mut reader = BufReader::new(stdout);
            reader.read_to_string(&mut stdout_output).await?;
        }
        if let Some(stderr) = command.stderr.take() {
            let mut reader = BufReader::new(stderr);
            reader.read_to_string(&mut stderr_output).await?;
        }

        let status = command.wait().await?;

        let output_message = if status.success() {
            stdout_output
        } else {
            format!(
                "Command failed with status: {}. Error: {}",
                status, stderr_output
            )
        };

        if let Err(e) = output_tx.send((id, output_message)).await {
            warn!("Failed to send output for pane {:?}: {}", id, e);
        }

        Ok(())
    }

    pub fn to_serializable_state(&self) -> CommandSerializableState {
        CommandSerializableState {
            exec: self.exec.clone(),
            interval: self.interval,
            output_history: self.output_history.clone(),
            last_output: self.last_output.clone(),
            state: self.state,
            display_type: self.display_type,
        }
    }
}
