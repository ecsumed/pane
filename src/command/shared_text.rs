use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Default)]
pub struct SharedText {
    lines: Arc<[Arc<str>]>,
    trailing_newline: bool,
}

impl SharedText {
    pub fn lines(&self) -> &[Arc<str>] {
        &self.lines
    }

    pub fn shares_storage_with(&self, other: &SharedText) -> bool {
        Arc::ptr_eq(&self.lines, &other.lines)
    }

    pub fn share_with(self, previous: &SharedText) -> SharedText {
        if self == *previous {
            return previous.clone();
        }

        let known: HashMap<&str, &Arc<str>> = previous
            .lines
            .iter()
            .map(|line| (line.as_ref(), line))
            .collect();
        let lines = self
            .lines
            .iter()
            .map(|line| {
                known
                    .get(line.as_ref())
                    .map_or(line.clone(), |&shared| shared.clone())
            })
            .collect();

        SharedText {
            lines,
            trailing_newline: self.trailing_newline,
        }
    }

    pub fn number(&self) -> Option<f64> {
        match self.lines.as_ref() {
            [line] => line.trim().parse().ok(),
            _ => self.to_string().trim().parse().ok(),
        }
    }
}

impl PartialEq for SharedText {
    fn eq(&self, other: &Self) -> bool {
        self.shares_storage_with(other)
            || (self.trailing_newline == other.trailing_newline && self.lines == other.lines)
    }
}

impl Eq for SharedText {}

impl std::hash::Hash for SharedText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lines.hash(state);
        self.trailing_newline.hash(state);
    }
}

impl From<&str> for SharedText {
    fn from(text: &str) -> Self {
        SharedText {
            lines: text.lines().map(Arc::from).collect(),
            trailing_newline: text.ends_with('\n'),
        }
    }
}

impl From<String> for SharedText {
    fn from(text: String) -> Self {
        SharedText::from(text.as_str())
    }
}

impl fmt::Display for SharedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            f.write_str(line)?;
        }
        if self.trailing_newline {
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Serialize for SharedText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SharedText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(SharedText::from(String::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trips_text_exactly() {
        for text in ["", "a", "a\n", "a\nb", "a\nb\n", "\n\n"] {
            assert_eq!(SharedText::from(text).to_string(), text, "{text:?}");
        }
    }

    #[test]
    fn test_identical_runs_share_everything() {
        let first = SharedText::from("NAME READY\nistiod 1/1\n");
        let second = SharedText::from("NAME READY\nistiod 1/1\n").share_with(&first);
        assert!(second.shares_storage_with(&first));
    }

    #[test]
    fn test_unchanged_lines_are_shared_between_runs() {
        let first = SharedText::from("NAME READY\nistiod 1/1\ningress 0/1\n");
        let second = SharedText::from("NAME READY\nistiod 1/1\ningress 1/1\n").share_with(&first);

        assert!(!second.shares_storage_with(&first));
        assert!(Arc::ptr_eq(&second.lines()[0], &first.lines()[0]));
        assert!(Arc::ptr_eq(&second.lines()[1], &first.lines()[1]));
        assert!(!Arc::ptr_eq(&second.lines()[2], &first.lines()[2]));
        assert_eq!(second.to_string(), "NAME READY\nistiod 1/1\ningress 1/1\n");
    }

    #[test]
    fn test_numbers_and_equality() {
        assert_eq!(SharedText::from(" 42.5\n").number(), Some(42.5));
        assert_eq!(SharedText::from("pods").number(), None);
        assert_eq!(SharedText::from("a\n"), SharedText::from("a\n"));
        assert_ne!(SharedText::from("a\n"), SharedText::from("a"));
    }
}
