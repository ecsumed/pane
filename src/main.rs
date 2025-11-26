#![allow(unused_imports)]

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

use crate::app::App;
use crate::config::AppConfig;
use crate::logging::info;

mod app;
mod command;
mod config;
mod controls;
mod logging;
mod mode;
mod pane;
mod pane_manager;
mod session;
mod ui;

pub type DefaultTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

fn init() -> io::Result<DefaultTerminal> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore() -> io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let _guard = logging::init_tracing();

    info!("Application starting up.");

    let config = match config::AppConfig::load() {
        Ok(cfg) => {
            info!("{}", cfg);
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let mut terminal = init()?;

    let app_result = {
        let mut app = App::new(config);
        app.run(&mut terminal).await
    };

    restore()?;

    info!("Application shutting down.");

    app_result
}
