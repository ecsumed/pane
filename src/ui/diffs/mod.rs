use std::time::Duration;

use ratatui::text::Line;

use crate::{config::theme::Theme, mode::DiffMode};

pub mod char;
pub mod line;
mod plain;
pub mod word;

pub const DIFF_TIMEOUT: Duration = Duration::from_millis(25);

pub fn render_diff<'a>(
    theme: &Theme,
    current: &'a str,
    previous: &'a str,
    mode: DiffMode,
) -> Vec<Line<'a>> {
    match mode {
        DiffMode::None => plain::render(theme, current),
        DiffMode::Line => line::render(theme, current, previous),
        DiffMode::Word => word::render(theme, current, previous),
        DiffMode::Char => char::render(theme, current, previous),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    fn noise(seed: u64, len: usize) -> String {
        let mut state = seed;
        (0..len)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                if i % 80 == 79 {
                    '\n'
                } else {
                    (b'a' + (state >> 59) as u8) as char
                }
            })
            .collect()
    }

    #[test]
    fn test_huge_unrelated_outputs_diff_within_the_timeout() {
        let theme = crate::config::AppConfig::default().theme;
        let (previous, current) = (noise(1, 60_000), noise(2, 60_000));

        for mode in [DiffMode::Char, DiffMode::Word, DiffMode::Line] {
            let start = Instant::now();
            let lines = render_diff(&theme, &current, &previous, mode);
            let elapsed = start.elapsed();
            assert!(!lines.is_empty());
            assert!(elapsed < Duration::from_millis(500), "{mode}: {elapsed:?}");
        }
    }
}
