use crate::config::AppConfig;
use crate::mode::DiffMode;
use crate::ui::diffs;
use crate::{command::Command, ui::utils::scrollbar};
use ratatui::widgets::ScrollbarState;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub fn render<'a>(
    frame: &mut Frame,
    area: Rect,
    config: &'a AppConfig,
    command: &'a Command,
    selected_idx: usize,
    diff_mode: DiffMode,
    search_query: &'a str,
    scroll_offset: u16,
    max_scroll: &mut u16,
    scrollbar_state: &mut ScrollbarState,
    is_focused: bool,
) {
    let p = &config.theme.palette;
    let current_len = command.output_history.len();

    let data_idx = current_len.saturating_sub(1).saturating_sub(selected_idx);
    let prev_data_idx = data_idx.checked_sub(1);

    let current_output = command.output_history.get(data_idx);
    let previous_output = prev_data_idx.and_then(|idx| command.output_history.get(idx));

    let current_text = current_output.map_or("", |c| &c.output);
    let previous_text = previous_output.map_or("", |c| &c.output);

    let display_text = diffs::render_diff(
        &config.theme,
        current_text,
        previous_text,
        diff_mode,
        search_query,
    );

    let border_style = if is_focused {
        p.border_active
    } else {
        p.border_inactive
    };

    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::left(2));

    let inner_area = content_block.inner(area);

    let mut widget = Paragraph::new(display_text)
        .block(content_block)
        .scroll((scroll_offset, 0));

    if config.wrap {
        widget = widget.wrap(Wrap { trim: true });
    }

    let content_length: usize = widget.line_count(area.width);

    frame.render_widget(widget, area);
    scrollbar::widget(
        frame,
        inner_area,
        p,
        content_length as u16,
        max_scroll,
        scrollbar_state,
        &scroll_offset,
    );
}
