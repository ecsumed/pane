use crate::{app::App, mode::AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
    Frame,
};

pub fn draw_display_type_select(frame: &mut Frame, app: &mut App) {
    if let AppMode::DisplayTypeSelect { items, state } = &mut app.mode {
        let area = frame.area();

        let list_items = items
            .iter()
            .map(|dt| ListItem::new(format!("{:?}", dt)))
            .collect::<Vec<_>>();

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
            .block(Block::default().title("Select Type").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}
