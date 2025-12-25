use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;

pub mod diff;
pub mod multiline;
pub mod raw_text;
pub mod types;
mod utils;

pub use types::DisplayType;

pub fn render_command_output(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command, block: Block) {
    let inner_area = block.inner(area);

    frame.render_widget(block.clone(), area);

    match command.display_type {
        DisplayType::RawText => {
            raw_text::render(frame, inner_area, config, command);
        }
        DisplayType::MultiLine | DisplayType::MultiLineTime | DisplayType::MultiLineDateTime => {
            multiline::render(frame, inner_area, config, command);
        }
        DisplayType::DiffChar | DisplayType::DiffWord | DisplayType::DiffLine => {
            diff::render(frame, inner_area, config, command);
        }
    }
}
