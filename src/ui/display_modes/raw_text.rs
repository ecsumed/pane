use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::DisplayType;
use crate::command::Command;
use crate::ui::display_modes::utils;

pub fn render(frame: &mut Frame, area: Rect, cmd: &Command) {
    let last_output = utils::formatted_last_output(cmd);
    let mut widget = Paragraph::new(last_output);

    if matches!(cmd.display_type, DisplayType::RawWrapped) {
        widget = widget.wrap(ratatui::widgets::Wrap { trim: true });
    }

    frame.render_widget(widget, area);
}
