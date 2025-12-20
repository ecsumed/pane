pub mod args;
pub use args::Cli;
use clap::Parser;

pub fn parse() -> Cli {
    Cli::parse()
}