mod cmd_input;
mod diffs;
mod display_modes;
mod display_select;
pub mod draw;
mod help_menu;
mod observe;
mod panes;
mod picker;
mod session_load;
mod session_save;
mod status_line;
pub mod utils;

pub use self::display_modes::{refresh_view, DisplayType};

pub fn big_text_largest_level(
    config: &crate::config::AppConfig,
    command: &crate::command::Command,
    pane: ratatui::layout::Rect,
) -> u8 {
    display_modes::big_text::largest_level(panes::content_area(config, command, pane))
}

pub fn big_text_auto_level(
    config: &crate::config::AppConfig,
    command: &crate::command::Command,
    pane: ratatui::layout::Rect,
) -> Option<u8> {
    let area = panes::content_area(config, command, pane);
    display_modes::big_text::auto_level(&display_modes::big_text::label_lines(command), area)
}
