use std::io;
use std::process::Stdio;
use tokio::process::Command as SysCommand;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::sync::mpsc;

use crate::logging::warn;
use crate::pane::PaneKey;

impl super::Command {
    pub async fn run_and_send_output(
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
}