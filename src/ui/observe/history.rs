use std::collections::HashMap;
use std::time::Duration;

use chrono::NaiveDateTime;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};
use similar::{ChangeTag, TextDiff};

use crate::command::{Command, CommandOutput};
use crate::config::AppConfig;
use crate::ui::diffs::DIFF_TIMEOUT;
use crate::ui::picker::PickerStyle;
use crate::ui::utils::BlockExt;

pub type ChangeCounts = HashMap<NaiveDateTime, (usize, usize)>;

const ROW_WIDTH: usize = 25;

pub fn line_changes(previous: &str, current: &str) -> (usize, usize) {
    TextDiff::configure()
        .timeout(DIFF_TIMEOUT)
        .diff_lines(previous, current)
        .iter_all_changes()
        .fold((0, 0), |(added, removed), change| match change.tag() {
            ChangeTag::Insert => (added + 1, removed),
            ChangeTag::Delete => (added, removed + 1),
            ChangeTag::Equal => (added, removed),
        })
}

pub fn update_change_counts(command: &Command, counts: &mut ChangeCounts) {
    let entries: Vec<&CommandOutput> = command.output_history.iter().collect();
    counts.retain(|time, _| entries.iter().skip(1).any(|o| o.time == *time));
    for pair in entries.windows(2) {
        counts
            .entry(pair[1].time)
            .or_insert_with(|| line_changes(&pair[0].output, &pair[1].output));
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

fn describe_changes(changes: Option<(usize, usize)>) -> String {
    match changes {
        None => "first run".to_string(),
        Some((0, 0)) => "no change".to_string(),
        Some((added, 0)) => format!("+{added}"),
        Some((0, removed)) => format!("−{removed}"),
        Some((added, removed)) => format!("+{added} −{removed}"),
    }
}

fn row<'a>(
    out: &CommandOutput,
    is_latest: bool,
    changes: Option<(usize, usize)>,
    s: &PickerStyle,
) -> ListItem<'a> {
    let (status, detail, failed) = match out.exit_status {
        Some(0) => ("✓", describe_changes(changes), false),
        Some(code) => ("✗", format!("exit {code}"), true),
        None => ("⏱", "timed out".to_string(), true),
    };
    let (text, muted) = if failed {
        (s.error, s.error)
    } else {
        (s.text, s.muted)
    };

    let time = out.time.format("%H:%M:%S").to_string();
    let label = if is_latest {
        format!("Latest   {time}")
    } else {
        time
    };
    let padding = ROW_WIDTH.saturating_sub(label.chars().count() + 1);

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(label, text),
            Span::raw(" ".repeat(padding)),
            Span::styled(status, text),
        ]),
        Line::from(Span::styled(
            format!("{} · {detail}", format_duration(out.duration)),
            muted,
        )),
    ])
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
        .enumerate()
        .map(|(position, out)| {
            let changes = if Some(out.time) == oldest {
                None
            } else {
                change_counts.get(&out.time).copied()
            };
            row(out, position == 0, changes, &s)
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
                    format!(
                        " History ({}/{}) ",
                        command.output_history.len(),
                        config.max_history
                    ),
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
    fn test_line_changes_and_durations() {
        assert_eq!(line_changes("a\nb\nc\n", "a\nb\nc\n"), (0, 0));
        assert_eq!(line_changes("Mon 12:00:00\n", "Mon 12:00:10\n"), (1, 1));
        assert_eq!(line_changes("a\n", "a\nb\nc\n"), (2, 0));
        assert_eq!(describe_changes(Some((1, 1))), "+1 −1");
        assert_eq!(describe_changes(Some((2, 0))), "+2");
        assert_eq!(describe_changes(Some((0, 3))), "−3");
        assert_eq!(describe_changes(Some((0, 0))), "no change");
        assert_eq!(describe_changes(None), "first run");
        assert_eq!(format_duration(Duration::from_millis(1_480)), "1.5s");
        assert_eq!(format_duration(Duration::from_secs(17)), "17s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m05s");
    }
}
