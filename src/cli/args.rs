use clap::{Parser};
use clap_verbosity_flag::{ErrorLevel, Verbosity};

#[derive(Parser, Debug)]
#[command(author, version, about = "Watch + tmux-resurrect = poor mans grafana")]
pub struct Cli {
    /// Enable an audible beep if a task completes with a non-zero status code
    #[arg(short, long)]
    pub beep: bool,

    /// The interval to wait between executions
    #[arg(short = 'n', long, value_name = "SECONDS")]
    pub interval: Option<u64>,

    #[command(flatten)]
    pub verbose: Verbosity<ErrorLevel>,

    #[arg(num_args = 1..)]
    pub command: Vec<String>,
}