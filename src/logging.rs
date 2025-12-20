use std::path::PathBuf;

pub use tracing::{debug, error, trace, info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing(log_level_filter: LevelFilter, logs_dir: &PathBuf) -> WorkerGuard {
    let file_appender = rolling::daily(logs_dir, "ratatui-app.log");
    let (non_blocking_appender, guard) = non_blocking(file_appender);

    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_filter(log_level_filter);

    tracing_subscriber::registry().with(file_layer).init();

    info!("Tracing initialized.");

    guard
}

pub fn get_log_level_filter(log_level_str: Option<&str>) -> LevelFilter {
    match log_level_str {
        Some("error") => LevelFilter::ERROR,
        Some("warn") => LevelFilter::WARN,
        Some("info") => LevelFilter::INFO,
        Some("debug") => LevelFilter::DEBUG,
        Some("trace") => LevelFilter::TRACE,
        _ => LevelFilter::OFF,
    }
}
