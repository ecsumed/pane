use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{self, Interval, MissedTickBehavior};

use super::{Command, CommandControl, CommandState};
use crate::command::CommandEvent;
use crate::logging::{debug, info, warn};
use crate::pane::PaneKey;

enum Flow {
    Continue,
    Run,
    Stop,
}

impl Command {
    fn ticker(start: time::Instant, period: Duration) -> Interval {
        let mut tick_interval = time::interval_at(start, period);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        tick_interval
    }

    fn apply_control(
        id: PaneKey,
        control: CommandControl,
        interval: &mut Duration,
        tick_interval: &mut Interval,
        is_paused: &mut bool,
    ) -> Flow {
        match control {
            CommandControl::Stop => {
                info!("Pane {:?} task received stop command.", id);
                Flow::Stop
            }
            CommandControl::IntervalSet(duration) => {
                *interval = duration;
                *tick_interval = Self::ticker(time::Instant::now() + duration, duration);
                info!("Pane {:?} interval set to {:?}", id, duration);
                Flow::Continue
            }
            CommandControl::Pause => {
                info!("Pane {:?} paused", id);
                *is_paused = true;
                Flow::Continue
            }
            CommandControl::Resume => {
                info!("Pane {:?} resumed", id);
                *is_paused = false;
                Flow::Run
            }
            CommandControl::Execute => {
                info!("Pane {:?} received ad-hoc execution command.", id);
                Flow::Run
            }
            _ => Flow::Continue,
        }
    }

    pub async fn run_command_task(
        id: PaneKey,
        exec: String,
        interval: Duration,
        timeout: Duration,
        state: CommandState,
        mut control_rx: mpsc::UnboundedReceiver<CommandControl>,
        output_tx: mpsc::Sender<(PaneKey, CommandEvent)>,
    ) {
        let mut interval = interval;
        let mut tick_interval = Self::ticker(time::Instant::now(), interval);
        let mut is_paused = matches!(state, CommandState::Paused);

        loop {
            let manual = tokio::select! {
                control = control_rx.recv() => {
                    let Some(control) = control else { break };
                    match Self::apply_control(id, control, &mut interval, &mut tick_interval, &mut is_paused) {
                        Flow::Stop => break,
                        Flow::Continue => continue,
                        Flow::Run => true,
                    }
                }
                _ = tick_interval.tick(), if !is_paused => false,
            };

            debug!("Pane {:?} task running command: {}", id, exec);
            let run = Self::run_and_send_output(id, &exec, output_tx.clone(), timeout);
            tokio::pin!(run);

            loop {
                tokio::select! {
                    result = &mut run => {
                        if let Err(e) = result {
                            warn!("Pane {:?} task failed to run command: {}", id, e);
                        }
                        break;
                    }
                    control = control_rx.recv() => {
                        let Some(control) = control else { return };
                        if let Flow::Stop = Self::apply_control(id, control, &mut interval, &mut tick_interval, &mut is_paused) {
                            return;
                        }
                    }
                }
            }

            if manual {
                tick_interval.reset_at(time::Instant::now() + interval);
            }
        }
    }
}
