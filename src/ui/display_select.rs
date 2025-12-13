use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::utils::centered_rect;

pub fn draw_display_type_select(frame: &mut Frame, app: &mut App) {
    if let AppMode::DisplayTypeSelect { items, state } = &mut app.mode {
        let area = frame.area();

        let list_items = items
            .iter()
            .map(|dt| ListItem::new(format!("{:?}", dt)))
            .collect::<Vec<_>>();

        let percent_x = 60;
        let popup_area = centered_rect(percent_x, area, 7);

        Clear.render(popup_area, frame.buffer_mut());

        let list_widget = List::new(list_items)
            .block(Block::default().title("Select Type").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}
