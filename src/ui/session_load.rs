use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;

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

        const POPUP_HEIGHT: u16 = 6;
        const POPUP_WIDTH: u16 = 60;

        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - POPUP_HEIGHT) / 2),
                Constraint::Length(POPUP_HEIGHT),
                Constraint::Percentage((100 - POPUP_HEIGHT) / 2),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - POPUP_WIDTH) / 2),
                Constraint::Length(POPUP_WIDTH),
                Constraint::Percentage((100 - POPUP_WIDTH) / 2),
            ])
            .split(popup_area)[1];

        Clear.render(popup_area, frame.buffer_mut());

        let list_widget = List::new(list_items)
            .block(Block::default().title("Load Session").borders(Borders::ALL));

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}
