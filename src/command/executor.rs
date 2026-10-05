use std::io;
use std::process::Stdio;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDateTime};
use humantime::format_duration;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command as SysCommand;
use tokio::sync::mpsc;
use tokio::time;

use crate::command::{CommandEvent, CommandOutput};
use crate::logging::warn;
use crate::pane::PaneKey;

impl super::Command {
    pub async fn run_and_send_output(
        id: PaneKey,
        exec: &str,
        output_tx: mpsc::Sender<(PaneKey, CommandEvent)>,
        timeout: Duration,
    ) -> Result<(), io::Error> {
        if let Err(e) = output_tx.send((id, CommandEvent::Started)).await {
            warn!("Failed to send output for pane {:?}: {}", id, e);
        }

        let start = Instant::now();

        let mut command = SysCommand::new("sh")
            .arg("-c")
            .arg(exec)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let collected = time::timeout(timeout, async {
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
            Ok::<_, io::Error>((status, stdout_output, stderr_output))
        })
        .await;

        let duration = start.elapsed();

        let (output_message, exit_status) = match collected {
            Ok(result) => {
                let (status, stdout_output, stderr_output) = result?;
                let message = if status.success() {
                    stdout_output
                } else {
                    format!(
                        "Command failed with status: {}. Error: {}",
                        status, stderr_output
                    )
                };
                (message, status.code())
            }
            Err(_) => (
                format!("Command timed out after {}", format_duration(timeout)),
                None,
            ),
        };

        let now_datetime: NaiveDateTime = Local::now().naive_local();
        let cmd_output = CommandOutput {
            output: output_message,
            time: now_datetime,
            exit_status,
            duration,
        };

        if let Err(e) = output_tx.send((id, CommandEvent::Output(cmd_output))).await {
            warn!("Failed to send output for pane {:?}: {}", id, e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    async fn run(exec: &str, timeout: Duration) -> CommandOutput {
        let (tx, mut rx) = mpsc::channel(10);
        let result = time::timeout(
            Duration::from_secs(5),
            Command::run_and_send_output(PaneKey::default(), exec, tx, timeout),
        )
        .await;
        assert!(result.is_ok(), "Command did not finish in time");

        while let Some((_, event)) = rx.recv().await {
            if let CommandEvent::Output(out) = event {
                return out;
            }
        }
        panic!("No output received");
    }

    #[tokio::test]
    async fn test_stdin_is_not_inherited() {
        let out = run("cat; echo done", Duration::from_secs(5)).await;
        assert_eq!(out.output, "done\n");
        assert_eq!(out.exit_status, Some(0));
    }

    #[tokio::test]
    async fn test_command_times_out() {
        let out = run("sleep 10", Duration::from_millis(200)).await;
        assert!(out.output.contains("timed out"), "{}", out.output);
        assert_eq!(out.exit_status, None);
        assert!(out.duration < Duration::from_secs(2));
    }
}
