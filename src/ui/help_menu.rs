use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::ui::utils::centered_rect;

pub fn draw_help_menu(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    
    let settings_count = 4;
    let bindings_count = app.config.keybindings.len();
    let content_height = std::cmp::max(settings_count, bindings_count) as u16;
    let total_height = (content_height + 4).min(area.height - 2);

    let popup_area = centered_rect(75, area, total_height);

    Clear.render(popup_area, frame.buffer_mut());
    
    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" Help & Settings ")
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner_area = main_block.inner(popup_area);
    frame.render_widget(main_block, popup_area);

    let [_, left_area, sep_area, _, right_area] = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Percentage(45),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Percentage(54),
    ]).areas(inner_area);

    let config = &app.config;
    let settings: Vec<ListItem> = vec![
        format!("Interval:    {:?}", config.interval),
        format!("Max History: {}", config.max_history),
        format!("Log Level:   {}", config.log_level.as_deref().unwrap_or("None")),
        format!("Logs Dir:    {}", config.logs_dir.display()),
    ]
    .into_iter()
    .map(|s| ListItem::new(s).cyan())
    .collect();

    frame.render_widget(List::new(settings).block(Block::default().title(" Settings ")), left_area);

    let separator_widget = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(separator_widget, sep_area);

    let mut bindings: Vec<_> = config.keybindings.iter().collect();
    bindings.sort_by(|a, b| format!("{:?}", a.1).cmp(&format!("{:?}", b.1)));

    let key_items: Vec<ListItem> = bindings
        .into_iter()
        .map(|(key, action)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{: <12}", key.to_string()), Color::Yellow),
                Span::raw(" → "),
                Span::styled(format!("{:?}", action), Style::default().italic()),
            ]))
        })
        .collect();

    frame.render_widget(List::new(key_items).block(Block::default().title(" Keybindings ")), right_area);
}
