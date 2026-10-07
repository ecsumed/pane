use std::io;
use std::process::Stdio;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDateTime};
use humantime::format_duration;
use tokio::process::Command as SysCommand;
use tokio::sync::mpsc;
use tokio::time;

use crate::command::{CommandEvent, CommandOutput};
use crate::logging::warn;
use crate::pane::PaneKey;

struct ProcessGroup(Option<i32>);

impl ProcessGroup {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        if let Some(pgid) = self.0 {
            unsafe {
                libc::killpg(pgid, libc::SIGKILL);
            }
        }
    }
}

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

        let command = SysCommand::new("sh")
            .arg("-c")
            .arg(exec)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .process_group(0)
            .spawn()?;

        let mut group = ProcessGroup(command.id().map(|pid| pid as i32));
        let collected = time::timeout(timeout, command.wait_with_output()).await;
        if collected.is_ok() {
            group.disarm();
        }
        drop(group);

        let duration = start.elapsed();

        let (output_message, exit_status) = match collected {
            Ok(result) => {
                let output = result?;
                let message = if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                } else {
                    format!(
                        "Command failed with status: {}. Error: {}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    )
                };
                (message, output.status.code())
            }
            Err(_) => (
                format!("Command timed out after {}", format_duration(timeout)),
                None,
            ),
        };

        let now_datetime: NaiveDateTime = Local::now().naive_local();
        let cmd_output = CommandOutput {
            output: output_message.into(),
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
        assert_eq!(out.output.to_string(), "done\n");
        assert_eq!(out.exit_status, Some(0));
    }

    #[tokio::test]
    async fn test_large_stderr_does_not_deadlock() {
        let exec = "head -c 200000 /dev/zero | tr '\\0' x >&2; echo done";
        let out = run(exec, Duration::from_secs(3)).await;
        assert_eq!(out.output.to_string(), "done\n");
    }

    #[tokio::test]
    async fn test_timeout_kills_whole_pipeline() {
        let out = run("sleep 7.31 | cat", Duration::from_millis(200)).await;
        assert!(
            out.output.to_string().contains("timed out"),
            "{}",
            out.output
        );

        time::sleep(Duration::from_millis(200)).await;
        let leftover = std::process::Command::new("pgrep")
            .args(["-f", "sleep 7.31"])
            .output()
            .unwrap();
        assert!(
            leftover.stdout.is_empty(),
            "Pipeline child still running after timeout"
        );
    }

    #[tokio::test]
    async fn test_command_times_out() {
        let out = run("sleep 10", Duration::from_millis(200)).await;
        assert!(
            out.output.to_string().contains("timed out"),
            "{}",
            out.output
        );
        assert_eq!(out.exit_status, None);
        assert!(out.duration < Duration::from_secs(2));
    }
}
