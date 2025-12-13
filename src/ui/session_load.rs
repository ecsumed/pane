use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::utils::centered_rect;

pub fn draw_session_list(frame: &mut Frame, app: &mut App) {
    if let AppMode::SessionLoad { items, state } = &mut app.mode {
        let area = frame.area();

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut item = ListItem::new(s.as_str());

                if state.selected() == Some(i) {
                    item = item.style(Style::default().add_modifier(Modifier::BOLD));
                }
                item
            })
            .collect();

            let percent_x = 60;
            let popup_area = centered_rect(percent_x, area, 6);

        Clear.render(popup_area, frame.buffer_mut());

        let list_widget = List::new(list_items)
            .block(Block::default().title("Load Session").borders(Borders::ALL));

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}
