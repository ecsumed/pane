use std::collections::HashMap;

use crokey::KeyCombination;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Widget};
use ratatui::Frame;

use crate::config::theme::Palette;
use crate::config::AppConfig;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::mode::AppMode;
use crate::settings_line;
use crate::ui::utils::centered_rect2;
use crate::ui::utils::formatting::ToSettingsString;
use crate::ui::utils::scrollbar;

pub fn build_keybinding_list<'a>(
    keybindings: &'a HashMap<KeyMode, HashMap<KeyCombination, Action>>,
    p: &Palette,
    area: Rect,
) -> Vec<Line<'a>> {
    let mut items = Vec::new();

    let mut bindings_by_mode: Vec<_> = keybindings.iter().collect();
    bindings_by_mode.sort_by_key(|(mode, _map)| format!("{:?}", mode));

    for (mode, bindings) in bindings_by_mode.into_iter() {
        if bindings.is_empty() {
            continue;
        }

        items.extend(vec![
            Line::from(""),
            Line::from(Span::styled(format!(" KEYBINDINGS ({:?})", mode), p.h1)),
            Line::from(Span::raw("─".repeat(area.width as usize))),
        ]);

        let mut sorted_actions: Vec<_> = bindings.iter().collect();
        sorted_actions.sort_by(|a, b| format!("{:?}", a.1).cmp(&format!("{:?}", b.1)));

        items.extend(settings_line!(p, 20, @list sorted_actions));
    }

    items
}

pub fn draw_help_menu(frame: &mut Frame, c: &AppConfig, mode: &mut AppMode) {
    let p = &c.theme.palette;

    let area = frame.area();

    let popup_area = centered_rect2(75, 80, area);

    if let AppMode::Help {
        scroll_offset,
        max_scroll,
        scrollbar_state,
    } = mode
    {
        let [_, centered_area, _] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .areas(popup_area);

        Clear.render(centered_area, frame.buffer_mut());

        let mut settings = vec![
            Line::from(Span::styled(" CONFIGURATION ", p.h1)),
            Line::from(Span::raw("─".repeat(area.width as usize))),
        ];

        settings.extend(settings_line!(
            p,
            20,
            "Beep" => c.beep,
            "Default Display" => format!("{:?}", c.default_display),
            "Exit on Change" => c.chg_exit,
            "Exit on Error" => c.err_exit,
            "Interval" => format!("{:?}", c.interval),
            "Log Level" => c.log_level.as_deref().unwrap_or("None"),
            "Logs Dir" => c.logs_dir.display(),
            "Max History" => c.max_history,
            "Sessions Dir" => c.sessions_dir.display(),
            "Snapshot Dir" => c.snapshot_dir.display(),
            "Wrap" => c.wrap,
            "Zen" => c.zen,
        ));

        settings.extend(vec![
            Line::from(""),
            Line::from(Span::styled(" THEME ", p.h1)),
            Line::from(Span::raw("─".repeat(area.width as usize))),
        ]);

        settings.extend(settings_line!(
            p,
            20,
            "Collapse Borders" => c.theme.collapse_borders,
            "Show State" => c.theme.show_state,
            "Show Last Update" => c.theme.show_last_updated,
            "Show Display Type" => c.theme.show_display_type,
            "Show Status Bar" => c.theme.show_status_bar,
        ));

        settings.extend(build_keybinding_list(&c.keybindings, p, centered_area));

        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(2));

        let inner_area = block.inner(centered_area);
        let content_length = settings.len();

        let widget = Paragraph::new(Text::from(settings))
            .block(block)
            .scroll((*scroll_offset, 0));

        frame.render_widget(widget, centered_area);
        scrollbar::widget(
            frame,
            inner_area,
            p,
            content_length as u16,
            max_scroll,
            scrollbar_state,
            scroll_offset,
        );
    }
}
