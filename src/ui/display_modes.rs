use ratatui::{Frame, layout::Rect, widgets::Block};
use crate::command::Command;

pub mod types;
pub mod raw_text;
pub mod multiline;

pub use types::DisplayType;

pub fn render_command_output(frame: &mut Frame, area: Rect, command: &Command, block: Block) {
    let inner_area = block.inner(area); 

    frame.render_widget(block.clone(), area);

    match command.display_type {
        DisplayType::RawText | DisplayType::RawWrapped => {
            raw_text::render(frame, inner_area, command);
        }
        DisplayType::MultiLine => {
            multiline::render(frame, inner_area, command);
        }
    }
}
