use ratatui::prelude::*;

pub fn highlight_query<'a>(
    content: &'a str,
    query: &str,
    base_style: Style,
    match_style: Style,
) -> Vec<Span<'a>> {
    if query.is_empty() {
        return vec![Span::styled(content, base_style)];
    }

    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();

    if let Some(index) = content_lower.find(&query_lower) {
        let start = index;
        let end = index + query.len();

        let prefix = &content[..start];
        let matched = &content[start..end];
        let suffix = &content[end..];

        let mut spans = Vec::with_capacity(3);
        if !prefix.is_empty() {
            spans.push(Span::styled(prefix, base_style));
        }
        spans.push(Span::styled(matched, match_style));
        if !suffix.is_empty() {
            spans.push(Span::styled(suffix, base_style));
        }

        spans
    } else {
        vec![Span::styled(content, base_style)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn test_highlight_query_match_middle() {
        let base_style = Style::default().fg(Color::White);
        let match_style = Style::default().fg(Color::Yellow);
        let result = highlight_query("Hello World", "lo", base_style, match_style);

        // Should return 3 spans: "Hel", "lo", " World"
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].content, "Hel");
        assert_eq!(result[0].style, base_style);
        assert_eq!(result[1].content, "lo");
        assert_eq!(result[1].style, match_style);
        assert_eq!(result[2].content, " World");
        assert_eq!(result[2].style, base_style);
    }

    #[test]
    fn test_highlight_query_no_match() {
        let base_style = Style::default().fg(Color::White);
        let result = highlight_query("Hello", "abc", base_style, Style::default());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello");
        assert_eq!(result[0].style, base_style);
    }

    #[test]
    fn test_highlight_query_empty_search() {
        let base_style = Style::default().fg(Color::White);
        let result = highlight_query("Hello", "", base_style, Style::default());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello");
    }

    #[test]
    fn test_highlight_query_case_insensitive() {
        let base_style = Style::default().fg(Color::White);
        let match_style = Style::default().fg(Color::Yellow);
        
        // Query is lowercase 'hello', content is 'HELLO'
        let result = highlight_query("HELLO", "hello", base_style, match_style);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "HELLO");
        assert_eq!(result[0].style, match_style);
    }
}

