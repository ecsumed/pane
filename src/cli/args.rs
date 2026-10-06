use std::time::Duration;

use clap::Parser;
use clap_verbosity_flag::{ErrorLevel, Verbosity};

use crate::config::utils::parse_duration;
use crate::ui::DisplayType;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Watch + tmux-resurrect = poor mans dashboard"
)]
pub struct Cli {
    /// Enable an audible beep if a command completes with a non-zero status code
    #[arg(short, long)]
    pub beep: bool,

    /// Highlight differences
    #[arg(
        short = 'd',
        long = "display",
        value_enum,
        default_missing_value = "diff-char", 
        num_args = 0..=1,
        require_equals = true,
    )]
    pub display: Option<DisplayType>,

    /// The interval to wait between executions, e.g. 2, 0.5, 500ms or 1m
    #[arg(short = 'n', long, value_name = "DURATION", value_parser = parse_duration)]
    pub interval: Option<Duration>,

    /// Kill a command if it runs longer than this, e.g. 30, 2.5 or 2m
    #[arg(short = 't', long, value_name = "DURATION", value_parser = parse_duration)]
    pub timeout: Option<Duration>,

    #[command(flatten)]
    pub verbose: Verbosity<ErrorLevel>,

    #[arg(num_args = 1..)]
    pub command: Vec<String>,

    /// Exit if command completes a non-zero status code
    #[arg(short = 'e', long = "err-exit")]
    pub err_exit: bool,

    /// Exit if output changes
    #[arg(short = 'g', long = "chg-exit")]
    pub chg_exit: bool,

    /// Max history to keep
    #[arg(short = 'm', long = "max-history", value_name = "COUNT")]
    pub max_history: Option<usize>,

    /// Disable line wrapping
    #[arg(short = 'w', long = "no-wrap")]
    pub no_wrap: bool,

    /// Zen (focus) mode: hides extra info
    #[arg(short = 'z', long = "zen")]
    pub zen: bool,

    /// Print the default configuration as TOML and exit
    #[arg(long)]
    pub print_default_config: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_and_timeout_accept_durations() {
        let cli = Cli::try_parse_from(["pane", "-n", "0.5", "-t", "2m", "date"]).unwrap();
        assert_eq!(cli.interval, Some(Duration::from_millis(500)));
        assert_eq!(cli.timeout, Some(Duration::from_secs(120)));
        assert!(Cli::try_parse_from(["pane", "-n", "0", "date"]).is_err());
    }
}
