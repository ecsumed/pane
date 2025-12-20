use std::io::{self, stdout};
use std::panic::{set_hook, take_hook};

use crokey::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crokey::crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::logging::info;

mod app;
mod cli;
mod command;
mod config;
mod controls;
mod shell_history;
mod logging;
mod mode;
mod pane;
mod session;
mod ui;

pub type DefaultTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        hook(panic_info);
    }));
}

fn init() -> io::Result<DefaultTerminal> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    init_panic_hook();
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
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli_args = cli::parse();


    let mut config = match config::AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    config.merge_cli(&cli_args);

    let log_level_filter = logging::get_log_level_filter(config.log_level.as_deref());
    let _guard = logging::init_tracing(log_level_filter, &config.logs_dir);

    info!("Application starting up.");
    info!("{}", config);

    let mut terminal = init()?;

    let app_result = {
        let mut app = App::new(config, cli_args.command);
        app.run(&mut terminal).await
    };

    restore()?;

    info!("Application shutting down.");

    app_result
}
