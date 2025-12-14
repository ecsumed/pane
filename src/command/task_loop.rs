use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};

use super::{Command, CommandControl, CommandState};
use crate::command::CommandOutput;
use crate::logging::{info, debug, warn};
use crate::pane::PaneKey;

impl Command {
    pub async fn run_command_task(
        id: PaneKey,
        exec: String,
        interval: Duration,
        state: CommandState,
        mut control_rx: mpsc::Receiver<CommandControl>,
        output_tx: tokio::sync::mpsc::Sender<(PaneKey, CommandOutput)>,
    ) {
        let interval_duration = interval;
        let mut tick_interval = time::interval(interval);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut is_paused = matches!(state, CommandState::Paused);

        loop {
            tokio::select! {
                Some(control) = control_rx.recv() => {
                    match control {
                        CommandControl::Stop => {
                            info!("Pane {:?} task received stop command.", id);
                            break;
                        }
                        CommandControl::IntervalSet(duration) => {
                            info!("Pane {:?} task received set interval.", id);
                            tick_interval = time::interval(duration);
                            tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                            info!("Pane {:?} interval set to {:?}", id, duration);
                        }

                        CommandControl::Pause => { info!("Pane {:?} paused", id); is_paused = true; }
                        CommandControl::Resume => {
                            info!("Pane {:?} resumed", id);
                            is_paused = false;
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} resume execution failed: {}", id, e);
                            }
                            tick_interval.reset_at(time::Instant::now() + interval_duration);
                        }
                        CommandControl::Execute => {
                            info!("Pane {:?} received ad-hoc execution command.", id);
                            if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                                warn!("Pane {:?} ad-hoc execution failed: {}", id, e);
                            }
                            tick_interval.reset_at(time::Instant::now() + interval_duration);
                        }
                        _ => ()
                    }
                }
                _ = tick_interval.tick(), if !is_paused => {
                    debug!("Pane {:?} task running command: {}", id, exec);
                    if let Err(e) = Self::run_and_send_output(id, &exec, output_tx.clone()).await {
                        warn!("Pane {:?} task failed to run command: {}", id, e);
                    }
                }
            }
        }
    }
}
