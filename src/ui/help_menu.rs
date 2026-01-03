use std::collections::HashMap;

use crokey::KeyCombination;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::ui::utils::centered_rect;

pub fn build_keybinding_list<'a>(
    bindings_by_mode: &'a [(&'a KeyMode, &'a HashMap<KeyCombination, Action>)],
) -> Vec<ListItem<'a>> {
    let mut items: Vec<ListItem<'static>> = Vec::new();

    for (mode, bindings) in bindings_by_mode.into_iter() {
        if bindings.is_empty() {
            continue;
        }
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("--- {:?} Mode ---", mode),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )])));

        let mut sorted_actions: Vec<_> = bindings.iter().collect();
        sorted_actions.sort_by(|a, b| format!("{:?}", a.1).cmp(&format!("{:?}", b.1)));

        let mode_items: Vec<ListItem> = bindings
            .into_iter()
            .map(|(key, action)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{: <12}", key.to_string()), Color::Yellow),
                    Span::raw(" → "),
                    Span::styled(
                        format!("{:?}", action),
                        Style::default().italic().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        items.extend(mode_items);
        items.push(ListItem::new(Line::from(vec![])));
    }

    items
}

pub fn draw_help_menu(frame: &mut Frame, app: &mut App) {
    let _p = &app.config.theme.palette;

    let area = frame.area();

    let total_keybindings_count = app
        .config
        .keybindings
        .values()
        .map(|inner_map| inner_map.len())
        .sum::<usize>();
    let mode_header_count: usize = app.config.keybindings.len();
    let total_height = (total_keybindings_count + mode_header_count) as u16;

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
    ])
    .areas(inner_area);

    let c = &app.config;
    let settings: Vec<ListItem> = vec![
        format!("Beep: {}", c.beep),
        format!("Default Display: {:?}", c.default_display),
        format!("Exit on Change: {}", c.chg_exit),
        format!("Exit on Error: {}", c.err_exit),
        format!("Interval: {:?}", c.interval),
        format!("Log Level: {}", c.log_level.as_deref().unwrap_or("None")),
        format!("Logs Dir: {}", c.logs_dir.display()),
        format!("Max History: {}", c.max_history),
        format!("Sessions Dir: {}", c.sessions_dir.display()),
        format!("Snapshot Dir: {}", c.snapshot_dir.display()),
        format!("Wrap: {}", c.wrap),
        format!("Zen: {}", c.zen),
    ]
    .into_iter()
    .map(|s| ListItem::new(s).cyan())
    .collect();

    frame.render_widget(
        List::new(settings).block(Block::default().title(" Settings ")),
        left_area,
    );

    let separator_widget = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(separator_widget, sep_area);

    let mut bindings_ref_vec: Vec<_> = c.keybindings.iter().collect();
    bindings_ref_vec.sort_by_key(|(mode, _map)| format!("{:?}", mode));
    let key_items: Vec<ListItem> = build_keybinding_list(&bindings_ref_vec);

    frame.render_widget(
        List::new(key_items).block(Block::default().title(" Keybindings ")),
        right_area,
    );
}
