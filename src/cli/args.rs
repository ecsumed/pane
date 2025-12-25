use clap::{Parser};
use clap_verbosity_flag::{ErrorLevel, Verbosity};

#[derive(Parser, Debug)]
#[command(author, version, about = "Watch + tmux-resurrect = poor mans grafana")]
pub struct Cli {
    /// Enable an audible beep if a command completes with a non-zero status code
    #[arg(short, long)]
    pub beep: bool,

    /// The interval to wait between executions
    #[arg(short = 'n', long, value_name = "SECONDS")]
    pub interval: Option<u64>,

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
}