use ratatui::{
    layout::{Margin, Rect},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::config::theme::Palette;
use crate::logging::debug;

pub fn widget(
    frame: &mut Frame,
    area: Rect,
    p: &Palette,
    content_length: u16,
    max_scroll: &mut u16,
    scrollbar_state: &mut ScrollbarState,
    scroll_offset: &u16,
) {
    let logical_max_scroll = (content_length as u16).saturating_sub(area.height);
    *max_scroll = logical_max_scroll;

    debug!("max scroll: {}", max_scroll);
    debug!("content_length: {}", content_length);
    debug!("scroll_offset: {}", scroll_offset);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▴"))
        .end_symbol(Some("▾"))
        .track_symbol(Some("░"))
        .thumb_symbol("█")
        .track_style(p.scroll_track)
        .thumb_style(p.scroll_bar);

    *scrollbar_state = scrollbar_state
        .content_length(logical_max_scroll as usize)
        .viewport_content_length(0)
        .position((*scroll_offset) as usize);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        scrollbar_state,
    );
}
