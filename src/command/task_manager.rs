use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use tokio::sync::mpsc;

use crate::command::{Command, CommandControl, CommandOutput, CommandSerializableState, CommandState};
use crate::logging::{info, warn};
use crate::pane::PaneKey;
use crate::ui::DisplayType;

impl Command {
    pub fn spawn(
        id: PaneKey,
        exec: String,
        interval: Duration,
        output_tx: mpsc::Sender<(PaneKey, CommandOutput)>,
    ) -> Self {
        let (control_tx, control_rx) = mpsc::channel(1);

        let cmd = exec.clone();
        let task_handle = tokio::spawn(async move {
            Command::run_command_task(
                id,
                cmd,
                interval,
                CommandState::Running,
                control_rx,
                output_tx,
            )
            .await;
        });

        info!("Adding new command: {}", &exec);
        Command {
            exec,
            interval,
            output_history: VecDeque::new(),
            state: CommandState::Running,
            display_type: DisplayType::default(),
            task_handle: Some(task_handle),
            control_tx,
        }
    }

    pub fn restore_tasks(
        tasks_state: HashMap<PaneKey, CommandSerializableState>,
        output_tx: mpsc::Sender<(PaneKey, CommandOutput)>,
    ) -> HashMap<PaneKey, Command> {
        let mut running_tasks = HashMap::new();

        for (id, state) in tasks_state {
            let new_cmd = Self::spawn_from_state(id, state, output_tx.clone());

            info!("Restarting command for id: {:?}", id);
            running_tasks.insert(id, new_cmd);
        }

        running_tasks
    }

    pub fn spawn_from_state(
        id: PaneKey,
        state: CommandSerializableState,
        output_tx: mpsc::Sender<(PaneKey, CommandOutput)>,
    ) -> Command {
        let (control_tx, control_rx) = mpsc::channel(1);

        let interval = state.interval;
        let exec = state.exec.clone();
        let cmd_state = state.state;

        let task_handle = tokio::spawn(async move {
            Self::run_command_task(id, exec, interval, cmd_state, control_rx, output_tx).await;
        });

        Command {
            exec: state.exec,
            interval: state.interval,
            output_history: state.output_history,
            state: state.state,
            display_type: state.display_type,
            task_handle: Some(task_handle),
            control_tx,
        }
    }

    pub async fn handle_control_signal(&mut self, id: PaneKey, cmd_ctrl: CommandControl) {
        let worker_instruction = match cmd_ctrl {
            CommandControl::IntervalIncrease => {
                self.interval += Duration::from_secs(1);
                CommandControl::IntervalSet(self.interval)
            }
            CommandControl::IntervalDecrease => {
                self.interval = self.interval.saturating_sub(Duration::from_secs(1));
                CommandControl::IntervalSet(self.interval)
            }
            _ => cmd_ctrl,
        };

        match &worker_instruction {
            CommandControl::Pause => self.state = CommandState::Paused,
            CommandControl::Resume => self.state = CommandState::Running,
            CommandControl::Stop => self.state = CommandState::Stopped,
            _ => {}
        }

        if let Err(e) = self.control_tx.send(worker_instruction).await {
            warn!(
                "Task {:?} is no longer running. Cannot send {:?}: {e}",
                id, worker_instruction
            );
        }
    }

    pub fn record_output(&mut self, new_output: CommandOutput, max_history: usize) {
        if self.output_history.len() >= max_history {
            self.output_history.pop_front();
        }
        self.output_history.push_back(new_output);
    }

    pub fn last_output(&self) -> Option<&CommandOutput> {
        self.output_history.back()
    }

    pub fn update_display(&mut self, display: DisplayType) {
        self.display_type = display;
    }
}
