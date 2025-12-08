use std::time::Duration;
use tokio::time::{self, MissedTickBehavior};
use tokio::sync::mpsc;

use crate::logging::{info, warn};
use crate::pane::PaneKey;
use super::{CommandControl, CommandState};

impl super::Command {
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
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut is_paused = matches!(state, CommandState::Paused);

        loop {
            tokio::select! {
                Some(control) = control_rx.recv() => {
                    match control {
                        CommandControl::Stop => {
                            info!("Pane {:?} task received stop command.", id);
                            break;
                        }
                        // ... (rest of the match statement) ...
                        CommandControl::IncreaseInterval => { info!("Pane {:?} task received increase interval.", id); }
                        CommandControl::DecreaseInterval => { info!("Pane {:?} task received increase interval.", id); }
                        CommandControl::Pause => { info!("Pane {:?} paused", id); is_paused = true; }
                        CommandControl::Resume => {
                            info!("Pane {:?} resumed", id);
                            is_paused = false;
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} resume execution failed: {}", id, e);
                            }
                            interval.reset_at(time::Instant::now() + interval_duration);
                        }
                        CommandControl::Execute => {
                            info!("Pane {:?} received ad-hoc execution command.", id);
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} ad-hoc execution failed: {}", id, e);
                            }
                            interval.reset_at(time::Instant::now() + interval_duration);
                        }
                    }
                }
                _ = interval.tick(), if !is_paused => {
                    info!("Pane {:?} task running command: {}", id, exec);
                    if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                        warn!("Pane {:?} task failed to run command: {}", id, e);
                    }
                }
            }
        }
    }
}