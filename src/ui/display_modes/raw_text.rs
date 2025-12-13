use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::DisplayType;
use crate::command::Command;

pub fn render(frame: &mut Frame, area: Rect, cmd: &Command) {
    let mut widget = Paragraph::new(cmd.last_output.as_str());

    if matches!(cmd.display_type, DisplayType::RawWrapped) {
        widget = widget.wrap(ratatui::widgets::Wrap { trim: true });
    }

    frame.render_widget(widget, area);
}
