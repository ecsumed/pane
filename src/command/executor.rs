use std::io;
use std::process::Stdio;
use std::time::Instant;

use chrono::{Local, NaiveDateTime};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command as SysCommand;
use tokio::sync::mpsc;

use crate::command::{CommandEvent, CommandOutput};
use crate::logging::warn;
use crate::pane::PaneKey;

impl super::Command {
    pub async fn run_and_send_output(
        id: PaneKey,
        exec: &str,
        output_tx: mpsc::Sender<(PaneKey, CommandEvent)>,
    ) -> Result<(), io::Error> {
        if let Err(e) = output_tx.send((id, CommandEvent::Started)).await {
            warn!("Failed to send output for pane {:?}: {}", id, e);
        }

        let start = Instant::now();

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

        let duration = start.elapsed();

        let output_message = if status.success() {
            stdout_output
        } else {
            format!(
                "Command failed with status: {}. Error: {}",
                status, stderr_output
            )
        };

        let now_datetime: NaiveDateTime = Local::now().naive_local();
        let cmd_output = CommandOutput {
            output: output_message,
            time: now_datetime,
            exit_status: status.code(),
            duration: duration,
        };

        if let Err(e) = output_tx.send((id, CommandEvent::Output(cmd_output))).await {
            warn!("Failed to send output for pane {:?}: {}", id, e);
        }

        Ok(())
    }
}
