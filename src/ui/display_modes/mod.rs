use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;

pub mod big_text;
mod chart;
mod counter;
pub mod diff;
mod multiline;
mod raw_text;
mod sparkline;
pub mod types;

pub use diff::refresh_view;
pub use types::DisplayType;

pub fn render_command_output(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    command: &Command,
    display_type: DisplayType,
    block: Block,
) {
    let inner_area = block.inner(area);

    frame.render_widget(block.clone(), area);

    match display_type {
        DisplayType::RawText => {
            raw_text::render(frame, inner_area, config, command);
        }
        DisplayType::MultiLine | DisplayType::MultiLineTime | DisplayType::MultiLineDateTime => {
            multiline::render(frame, inner_area, config, command, display_type);
        }
        DisplayType::DiffChar | DisplayType::DiffWord | DisplayType::DiffLine => {
            diff::render(frame, inner_area, config, command, display_type);
        }
        DisplayType::Sparkline => {
            sparkline::render(frame, inner_area, config, command);
        }
        DisplayType::BigText => {
            big_text::render(frame, inner_area, config, command);
        }
        DisplayType::Counter => {
            counter::render(frame, inner_area, config, command);
        }
        DisplayType::LineChart | DisplayType::BarChart | DisplayType::ScatterChart => {
            chart::render(frame, inner_area, config, command, display_type);
        }
    }
}
