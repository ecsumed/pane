use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, Wrap};
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};

use crate::command::Command;
use crate::config::AppConfig;

const GLYPH: u16 = 8;
pub const MAX_LEVEL: u8 = 12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Pixels { columns: u16, rows: u16 },
    Small(PixelSize),
}

impl Size {
    fn cell(self) -> (u16, u16) {
        match self {
            Size::Pixels { columns, rows } => (GLYPH * columns, GLYPH * rows),
            Size::Small(PixelSize::Quadrant) => (GLYPH / 2, GLYPH / 2),
            Size::Small(_) => (GLYPH, GLYPH / 2),
        }
    }
}

pub struct Layout {
    pub size: Size,
    pub rows: Vec<String>,
    pub width: u16,
    pub height: u16,
}

fn wrap(lines: &[String], columns: usize, split_words: bool) -> Option<Vec<String>> {
    if columns == 0 {
        return None;
    }
    let mut rows = Vec::new();
    for line in lines {
        let mut row = String::new();
        for word in line.split_whitespace() {
            let mut word: Vec<char> = word.chars().collect();
            while word.len() > columns {
                if !split_words {
                    return None;
                }
                if !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                }
                rows.push(word.drain(..columns).collect());
            }
            let word: String = word.into_iter().collect();
            let needed = row.chars().count() + usize::from(!row.is_empty()) + word.chars().count();
            if needed > columns && !row.is_empty() {
                rows.push(std::mem::take(&mut row));
            }
            if !row.is_empty() {
                row.push(' ');
            }
            row.push_str(&word);
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    Some(rows)
}

fn layout_with(lines: &[String], area: Rect, size: Size, split_words: bool) -> Option<Layout> {
    let (cell_width, cell_height) = size.cell();
    let mut columns = (area.width / cell_width) as usize;
    if split_words {
        columns = columns.max(1);
    }
    let rows = wrap(lines, columns, split_words)?;
    let widest = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as u16;
    Some(Layout {
        size,
        width: widest * cell_width,
        height: rows.len() as u16 * cell_height,
        rows,
    })
}

pub fn layout(lines: &[String], area: Rect, level: u8, split_words: bool) -> Option<Layout> {
    let fits = |l: &Layout| !l.rows.is_empty() && l.height <= area.height;
    match level {
        0 | 1 => layout_with(lines, area, Size::Small(PixelSize::Quadrant), split_words),
        2 => layout_with(lines, area, Size::Small(PixelSize::HalfHeight), split_words),
        level => {
            let rows = (level - 2) as u16;
            (rows..=rows * 2)
                .rev()
                .filter_map(|columns| {
                    layout_with(lines, area, Size::Pixels { columns, rows }, false)
                })
                .find(|l| fits(l) || split_words)
                .or_else(|| {
                    split_words
                        .then(|| {
                            layout_with(
                                lines,
                                area,
                                Size::Pixels {
                                    columns: rows,
                                    rows,
                                },
                                true,
                            )
                        })
                        .flatten()
                })
        }
    }
}

pub fn largest_level(area: Rect) -> u8 {
    let rows = area.width.min(area.height) / GLYPH;
    (rows as u8 + 2).clamp(1, MAX_LEVEL)
}

pub fn auto_level(lines: &[String], area: Rect) -> Option<u8> {
    (1..=MAX_LEVEL).rev().find(|&level| {
        layout(lines, area, level, false)
            .is_some_and(|l| !l.rows.is_empty() && l.height <= area.height)
    })
}

pub fn label_lines(command: &Command) -> Vec<String> {
    command
        .last_output()
        .map(|out| {
            out.output
                .lines()
                .iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn big_text(rows: Vec<String>, size: PixelSize, style: Style) -> BigText<'static> {
    BigText::builder()
        .pixel_size(size)
        .style(style)
        .lines(rows.into_iter().map(Line::from).collect::<Vec<_>>())
        .centered()
        .build()
}

fn render_pixels(
    buf: &mut Buffer,
    clip: Rect,
    target: Rect,
    rows: Vec<String>,
    (pixel_width, pixel_height): (u16, u16),
    style: Style,
) {
    let width = target.width / pixel_width;
    let height = target.height / pixel_height;
    let mut glyphs = Buffer::empty(Rect::new(0, 0, width, height));
    big_text(rows, PixelSize::Full, style).render(glyphs.area, &mut glyphs);

    for y in 0..height {
        for x in 0..width {
            if glyphs[(x, y)].symbol() == " " {
                continue;
            }
            let pixel = Rect::new(
                target.x + x * pixel_width,
                target.y + y * pixel_height,
                pixel_width,
                pixel_height,
            )
            .intersection(clip);
            for py in pixel.top()..pixel.bottom() {
                for px in pixel.left()..pixel.right() {
                    buf[(px, py)].set_symbol("█").set_style(style);
                }
            }
        }
    }
}

fn render_plain(frame: &mut Frame, area: Rect, lines: Vec<String>, style: Style) {
    let text: Vec<Line> = lines.into_iter().map(Line::from).collect();
    let top = area.height.saturating_sub(text.len() as u16) / 2;
    frame.render_widget(
        Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        Rect::new(area.x, area.y + top, area.width, area.height - top),
    );
}

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let style = config.theme.palette.output.add_modifier(Modifier::BOLD);
    let lines = label_lines(command);
    if lines.is_empty() {
        return;
    }

    let chosen = match command.text_size {
        Some(level) => layout(&lines, area, level, true),
        None => auto_level(&lines, area).and_then(|level| layout(&lines, area, level, false)),
    };
    let Some(Layout {
        size,
        rows,
        width,
        height,
    }) = chosen
    else {
        render_plain(frame, area, lines, style);
        return;
    };

    let target = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.max(1),
        height.max(1),
    );
    match size {
        Size::Pixels {
            columns,
            rows: pixel_rows,
        } => render_pixels(
            frame.buffer_mut(),
            area,
            target,
            rows,
            (columns, pixel_rows),
            style,
        ),
        Size::Small(pixel_size) => {
            let row = Rect::new(area.x, target.y, area.width, height).intersection(area);
            frame.render_widget(big_text(rows, pixel_size, style), row);
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::app::App;
    use crate::command::CommandOutput;
    use crate::ui::draw::draw_ui;
    use crate::ui::DisplayType;

    async fn screen(output: Option<&str>, width: u16, height: u16) -> String {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let id = app.pane_manager.active_pane_id;
        app.set_command(id, "kubectl get pods --no-headers | wc -l".to_string())
            .await;
        let task = app.tasks.get_mut(&id).unwrap();
        task.task_handle.take().unwrap().abort();
        task.display_type = DisplayType::BigText;
        if let Some(output) = output {
            task.record_output(
                CommandOutput {
                    output: output.into(),
                    time: chrono::Local::now().naive_local(),
                    exit_status: Some(0),
                    duration: std::time::Duration::from_millis(1),
                },
                10,
            );
        }
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw_ui(&mut app, frame)).unwrap();
        terminal.backend().to_string()
    }

    #[tokio::test]
    async fn test_renders_big_centred_text_or_falls_back() {
        let big = screen(Some("  42\n"), 80, 24).await;
        let block_rows: Vec<usize> = big
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains('█'))
            .map(|(i, _)| i)
            .collect();
        assert!(block_rows.len() >= 6, "{big}");
        let middle = block_rows[block_rows.len() / 2];
        assert!((9..=14).contains(&middle), "not vertically centred:\n{big}");

        let fallback = screen(Some("this-label-is-far-too-long-to-draw-big"), 40, 12).await;
        assert!(!fallback.contains('█'));
        assert!(fallback.contains("this-label"));

        let empty = screen(None, 40, 12).await;
        assert!(!empty.contains('█'));
    }

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_wraps_on_spaces_and_only_splits_words_when_asked() {
        let text = lines(&["deployment rollout done"]);
        assert_eq!(
            wrap(&text, 12, false).unwrap(),
            vec!["deployment", "rollout done"]
        );
        assert_eq!(wrap(&lines(&["READY"]), 2, false), None);
        assert_eq!(
            wrap(&lines(&["READY"]), 2, true).unwrap(),
            vec!["RE", "AD", "Y"]
        );
    }

    #[test]
    fn test_auto_size_grows_taller_when_width_is_the_limit() {
        let wide = Rect::new(0, 0, 175, 43);
        let time = lines(&["10:23:45"]);
        let level = auto_level(&time, wide).unwrap();
        let chosen = layout(&time, wide, level, false).unwrap();
        assert_eq!(chosen.rows, vec!["10:23:45"]);
        assert_eq!((chosen.width, chosen.height), (128, 16));
        assert_eq!(
            chosen.size,
            Size::Pixels {
                columns: 2,
                rows: 2
            }
        );
    }

    #[test]
    fn test_auto_size_uses_the_height_of_narrow_panes() {
        let narrow = Rect::new(0, 0, 40, 30);
        let label = lines(&["12:34 READY"]);
        assert_eq!(auto_level(&label, narrow), Some(3));
        let chosen = layout(&label, narrow, 3, false).unwrap();
        assert_eq!(chosen.rows, vec!["12:34", "READY"]);
        assert_eq!((chosen.width, chosen.height), (40, 16));

        assert_eq!(
            auto_level(&lines(&["42"]), Rect::new(0, 0, 80, 24)),
            Some(5)
        );
        assert_eq!(
            auto_level(&lines(&["READY"]), Rect::new(0, 0, 22, 30)),
            Some(1)
        );
        assert_eq!(
            auto_level(
                &lines(&["a-very-long-unbroken-label"]),
                Rect::new(0, 0, 20, 10)
            ),
            None
        );
    }
}

#[cfg(test)]
mod size_tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Direction;
    use ratatui::Terminal;

    use crate::app::App;
    use crate::command::CommandOutput;
    use crate::config::AppConfig;
    use crate::pane::PaneKey;
    use crate::ui::draw::draw_ui;
    use crate::ui::DisplayType;

    async fn big_text_pane(app: &mut App, text: &str) -> PaneKey {
        let id = app.pane_manager.active_pane_id;
        app.set_command(id, format!("echo {text}")).await;
        let task = app.tasks.get_mut(&id).unwrap();
        task.task_handle.take().unwrap().abort();
        task.display_type = DisplayType::BigText;
        task.record_output(
            CommandOutput {
                output: text.into(),
                time: chrono::Local::now().naive_local(),
                exit_status: Some(0),
                duration: std::time::Duration::from_millis(1),
            },
            10,
        );
        id
    }

    fn draw(app: &mut App) -> ratatui::buffer::Buffer {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| draw_ui(app, frame)).unwrap();
        terminal.backend().buffer().clone()
    }

    async fn press(app: &mut App, c: char) {
        let event = Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        crate::controls::handle_event(app, event).await.unwrap();
    }

    #[tokio::test]
    async fn test_larger_stops_at_the_biggest_size_that_shows_a_character() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let id = big_text_pane(&mut app, "42").await;
        draw(&mut app);
        for _ in 0..20 {
            press(&mut app, ']').await;
        }
        let largest =
            crate::ui::big_text_largest_level(&app.config, &app.tasks[&id], app.pane_rects[0].1);
        assert_eq!(app.tasks[&id].text_size, Some(largest));
        assert!(largest < crate::ui::display_modes::big_text::MAX_LEVEL);
    }

    #[tokio::test]
    async fn test_size_keys_step_from_auto_and_reset() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let id = big_text_pane(&mut app, "123456").await;
        draw(&mut app);
        let auto =
            crate::ui::big_text_auto_level(&app.config, &app.tasks[&id], app.pane_rects[0].1)
                .unwrap();

        press(&mut app, ']').await;
        assert_eq!(app.tasks[&id].text_size, Some(auto + 1));
        press(&mut app, '[').await;
        press(&mut app, '[').await;
        assert_eq!(app.tasks[&id].text_size, Some(auto - 1));
        press(&mut app, '0').await;
        assert_eq!(app.tasks[&id].text_size, None);

        app.tasks.get_mut(&id).unwrap().display_type = DisplayType::RawText;
        press(&mut app, ']').await;
        assert_eq!(app.tasks[&id].text_size, None);
        assert!(app.current_notice().is_some_and(|n| n.is_error));
    }

    #[tokio::test]
    async fn test_oversized_text_never_draws_over_the_next_pane() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let left = big_text_pane(&mut app, "8").await;
        app.pane_manager.split_pane(Direction::Horizontal);
        app.tasks.get_mut(&left).unwrap().text_size =
            Some(crate::ui::display_modes::big_text::MAX_LEVEL);
        let buffer = draw(&mut app);

        let right = app
            .pane_rects
            .iter()
            .find(|(key, _)| *key != left)
            .unwrap()
            .1;
        for y in right.top()..right.bottom() {
            for x in right.left()..right.right() {
                assert_ne!(
                    buffer[(x, y)].symbol(),
                    "█",
                    "drew into the next pane at {x},{y}"
                );
            }
        }
        let left_rect = app
            .pane_rects
            .iter()
            .find(|(key, _)| *key == left)
            .unwrap()
            .1;
        assert!(
            (left_rect.top()..left_rect.bottom())
                .any(|y| (left_rect.left()..left_rect.right())
                    .any(|x| buffer[(x, y)].symbol() == "█"))
        );
    }

    #[test]
    fn test_chosen_size_is_saved_with_the_session() {
        let state = crate::command::CommandSerializableState {
            exec: "date".into(),
            interval: std::time::Duration::from_secs(1),
            output_history: Default::default(),
            state: crate::command::CommandState::Idle,
            display_type: DisplayType::BigText,
            text_size: Some(7),
        };
        let saved = toml::to_string(&state).unwrap();
        let loaded: crate::command::CommandSerializableState = toml::from_str(&saved).unwrap();
        assert_eq!(loaded.text_size, Some(7));
    }
}
