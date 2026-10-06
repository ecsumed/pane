use ratatui::prelude::*;

fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn find_matches(haystack: &[char], needle: &[char]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if haystack[i..i + needle.len()] == *needle {
            ranges.push((i, i + needle.len()));
            i += needle.len();
        } else {
            i += 1;
        }
    }
    ranges
}

fn split_line<'a>(
    line: Line<'a>,
    ranges: &[(usize, Style)],
    bounds: &[(usize, usize)],
) -> Line<'a> {
    let Line {
        style,
        alignment,
        spans: old_spans,
    } = line;

    let mut spans = Vec::new();
    let mut pos = 0;
    let mut r = 0;

    for span in old_spans {
        let chars: Vec<char> = span.content.chars().collect();
        let start = pos;
        let end = pos + chars.len();
        let mut cursor = start;

        while cursor < end {
            while r < bounds.len() && bounds[r].1 <= cursor {
                r += 1;
            }
            let (segment_end, highlight) = match bounds.get(r) {
                Some(&(range_start, range_end)) if range_start <= cursor => {
                    (range_end.min(end), Some(ranges[r].1))
                }
                Some(&(range_start, _)) => (range_start.min(end), None),
                None => (end, None),
            };
            let text: String = chars[cursor - start..segment_end - start].iter().collect();
            let segment_style = match highlight {
                Some(style) => span.style.patch(style),
                None => span.style,
            };
            spans.push(Span::styled(text, segment_style));
            cursor = segment_end;
        }
        pos = end;
    }

    Line {
        style,
        alignment,
        spans,
    }
}

pub fn highlight_lines<'a>(
    lines: Vec<Line<'a>>,
    query: &str,
    match_style: Style,
    current: Option<(usize, Style)>,
) -> (Vec<Line<'a>>, Vec<usize>) {
    let needle: Vec<char> = query.chars().map(fold).collect();
    if needle.is_empty() {
        return (lines, Vec::new());
    }

    let mut match_rows = Vec::new();
    let lines = lines
        .into_iter()
        .enumerate()
        .map(|(row, line)| {
            let haystack: Vec<char> = line
                .spans
                .iter()
                .flat_map(|s| s.content.chars())
                .map(fold)
                .collect();
            let bounds = find_matches(&haystack, &needle);
            if bounds.is_empty() {
                return line;
            }
            let styles: Vec<(usize, Style)> = bounds
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let index = match_rows.len() + i;
                    let style = match current {
                        Some((current, style)) if current == index => match_style.patch(style),
                        _ => match_style,
                    };
                    (index, style)
                })
                .collect();
            match_rows.extend(std::iter::repeat_n(row, bounds.len()));
            split_line(line, &styles, &bounds)
        })
        .collect();

    (lines, match_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};

    fn matched_text(line: &Line, match_style: Style) -> Vec<String> {
        line.spans
            .iter()
            .filter(|s| s.style == match_style)
            .map(|s| s.content.to_string())
            .collect()
    }

    fn joined(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn test_matches_across_spans_from_char_diff() {
        let found = Style::default().bg(Color::Yellow);
        let line = Line::from(
            "istio-ingress-7d9f8"
                .chars()
                .map(|c| Span::raw(c.to_string()))
                .collect::<Vec<_>>(),
        );

        let (lines, rows) = highlight_lines(vec![line], "istio-ingress", found, None);

        assert_eq!(rows, vec![0]);
        assert_eq!(joined(&lines[0]), "istio-ingress-7d9f8");
        assert_eq!(matched_text(&lines[0], found).concat(), "istio-ingress");
    }

    #[test]
    fn test_every_match_on_a_line_case_insensitive() {
        let found = Style::default().bg(Color::Yellow);
        let (lines, rows) = highlight_lines(
            vec![Line::from("Running running RUNNING"), Line::from("none")],
            "running",
            found,
            None,
        );

        assert_eq!(rows, vec![0, 0, 0]);
        assert_eq!(
            matched_text(&lines[0], found),
            vec!["Running", "running", "RUNNING"]
        );
        assert_eq!(joined(&lines[1]), "none");
    }

    #[test]
    fn test_keeps_original_span_styles_and_handles_non_ascii() {
        let found = Style::default().bg(Color::Yellow);
        let added = Style::default().fg(Color::Green);
        let line = Line::from(vec![Span::raw("İstanbul "), Span::styled("pod", added)]);

        let (lines, rows) = highlight_lines(vec![line], "od", found, None);

        assert_eq!(rows, vec![0]);
        assert_eq!(joined(&lines[0]), "İstanbul pod");
        let highlighted = lines[0].spans.iter().find(|s| s.content == "od").unwrap();
        assert_eq!(highlighted.style, added.patch(found));
    }

    #[test]
    fn test_current_match_gets_its_own_style() {
        let found = Style::default().bg(Color::Yellow);
        let current = Style::default().add_modifier(Modifier::REVERSED);
        let (lines, rows) = highlight_lines(
            vec![Line::from("pod pod"), Line::from("pod")],
            "pod",
            found,
            Some((1, current)),
        );

        assert_eq!(rows, vec![0, 0, 1]);
        let styles: Vec<Style> = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .filter(|s| s.content == "pod")
            .map(|s| s.style)
            .collect();
        assert_eq!(styles, vec![found, found.patch(current), found]);
    }

    #[test]
    fn test_empty_query_changes_nothing() {
        let (lines, rows) = highlight_lines(vec![Line::from("abc")], "", Style::default(), None);
        assert!(rows.is_empty());
        assert_eq!(joined(&lines[0]), "abc");
    }
}
