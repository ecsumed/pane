use crate::command::Command;

pub fn formatted_last_output(cmd: &Command) -> &str {
    cmd.last_output()
        .map(|c| c.output.as_str())
        .unwrap_or("N/A")
}