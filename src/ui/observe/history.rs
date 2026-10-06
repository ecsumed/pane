use std::collections::HashMap;
use std::time::Duration;

use chrono::NaiveDateTime;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};
use similar::{ChangeTag, TextDiff};

use crate::command::{Command, CommandOutput};
use crate::config::AppConfig;
use crate::ui::picker::PickerStyle;
use crate::ui::utils::BlockExt;

pub type ChangeCounts = HashMap<NaiveDateTime, usize>;

pub fn changed_lines(previous: &str, current: &str) -> usize {
    TextDiff::from_lines(previous, current)
        .iter_all_changes()
        .filter(|change| change.tag() != ChangeTag::Equal)
        .count()
}

pub fn update_change_counts(command: &Command, counts: &mut ChangeCounts) {
    let entries: Vec<&CommandOutput> = command.output_history.iter().collect();
    for pair in entries.windows(2) {
        counts
            .entry(pair[1].time)
            .or_insert_with(|| changed_lines(&pair[0].output, &pair[1].output));
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    match duration.as_secs() {
        0..10 => format!("{secs:.1}s"),
        10..60 => format!("{}s", duration.as_secs()),
        total => format!("{}m{:02}s", total / 60, total % 60),
    }
}

fn row<'a>(out: &CommandOutput, changes: Option<usize>, s: &PickerStyle) -> ListItem<'a> {
    let (status, failed) = match out.exit_status {
        Some(0) => ("✓", false),
        Some(_) => ("✗", true),
        None => ("⏱", true),
    };
    let changes = match changes {
        Some(0) => "=".to_string(),
        Some(n) => format!("±{n}"),
        None => String::new(),
    };

    let (text, muted) = if failed {
        (s.error, s.error)
    } else {
        (s.text, s.muted)
    };

    ListItem::new(Line::from(vec![
        Span::styled(out.time.format("%H:%M:%S").to_string(), text),
        Span::styled(format!("  {status}"), text),
        Span::styled(format!(" {:>6}", format_duration(out.duration)), muted),
        Span::styled(format!(" {changes:>5}"), muted),
    ]))
}

pub fn widget<'a>(
    config: &'a AppConfig,
    command: &'a Command,
    change_counts: &ChangeCounts,
    is_focused: bool,
) -> List<'a> {
    let p = &config.theme.palette;
    let s = PickerStyle::from_palette(p);
    let oldest = command.output_history.front().map(|o| o.time);

    let items: Vec<ListItem> = command
        .output_history
        .iter()
        .rev()
        .map(|out| {
            let changes = if Some(out.time) == oldest {
                None
            } else {
                change_counts.get(&out.time).copied()
            };
            row(out, changes, &s)
        })
        .collect();

    let border_style = if is_focused {
        p.border_active
    } else {
        p.border_inactive
    };

    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .merge_if(config.theme.collapse_borders)
                .title(Span::styled(
                    format!(" History ({}) ", command.output_history.len()),
                    s.title,
                )),
        )
        .highlight_style(s.selected)
        .highlight_symbol("▸ ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changed_lines_and_durations() {
        assert_eq!(changed_lines("a\nb\nc\n", "a\nb\nc\n"), 0);
        assert_eq!(changed_lines("a\nb\nc\n", "a\nB\nc\n"), 2);
        assert_eq!(format_duration(Duration::from_millis(1_480)), "1.5s");
        assert_eq!(format_duration(Duration::from_secs(17)), "17s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m05s");
    }
}
