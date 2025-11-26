// src/logging.rs

use std::io;
use tracing::Level;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, util::SubscriberInitExt};

pub use tracing::{debug, error, info, trace, warn};

// init_tracing now returns the WorkerGuard.
pub fn init_tracing() -> WorkerGuard {
    // Set up a rolling log file that rotates daily.
    let file_appender = rolling::daily("./logs", "ratatui-app.log");
    let (non_blocking_appender, guard) = non_blocking(file_appender);

    // Create the file-writing layer.
    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_filter(LevelFilter::TRACE);

    // Set up the registry.
    tracing_subscriber::registry().with(file_layer).init();

    info!("Tracing initialized.");

    // Return the guard so it stays in scope.
    guard
}
