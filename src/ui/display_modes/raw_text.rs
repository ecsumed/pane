use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use super::DisplayType;
use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::display_modes::utils;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, cmd: &Command) {
    let p = &config.theme.palette;

    let last_output = utils::formatted_last_output(cmd);
    let mut widget = Paragraph::new(last_output).style(p.output);

    if config.wrap {
        widget = widget.wrap(Wrap { trim: true });
    }

    frame.render_widget(widget, area);
}
