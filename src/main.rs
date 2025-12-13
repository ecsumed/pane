use crokey::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crokey::crossterm::ExecutableCommand;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

use crate::app::App;
use crate::logging::info;

mod app;
mod command;
mod config;
mod controls;
mod logging;
mod mode;
mod pane;
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
    let config = match config::AppConfig::load() {
        Ok(cfg) => {
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let log_level_filter = logging::get_log_level_filter(config.log_level.as_deref()); 
    let _guard = logging::init_tracing(log_level_filter, &config.logs_dir);

    info!("Application starting up.");
    info!("{}", config);

    let mut terminal = init()?;

    let app_result = {
        let mut app = App::new(config);
        app.run(&mut terminal).await
    };

    restore()?;

    info!("Application shutting down.");

    app_result
}
