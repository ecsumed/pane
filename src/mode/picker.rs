use crossterm::event::Event;
use ratatui::widgets::ListState;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug)]
pub struct Picker<T> {
    items: Vec<T>,
    labels: Vec<String>,
    visible: Vec<usize>,
    pub filter: Input,
    pub state: ListState,
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle.chars().all(|n| chars.any(|h| h == n))
}

impl<T> Picker<T> {
    pub fn new(items: Vec<T>, label: impl Fn(&T) -> String) -> Self {
        let mut picker = Self {
            items: Vec::new(),
            labels: Vec::new(),
            visible: Vec::new(),
            filter: Input::default(),
            state: ListState::default(),
        };
        picker.set_items(items, label);
        picker
    }

    pub fn set_items(&mut self, items: Vec<T>, label: impl Fn(&T) -> String) {
        let position = self.state.selected().unwrap_or(0);
        self.labels = items.iter().map(label).collect();
        self.items = items;
        self.apply_filter();
        if !self.visible.is_empty() {
            self.state
                .select(Some(position.min(self.visible.len() - 1)));
        }
    }

    fn apply_filter(&mut self) {
        let query = self.filter.value().to_lowercase();
        let mut contains = Vec::new();
        let mut fuzzy = Vec::new();

        for (i, label) in self.labels.iter().enumerate() {
            let label = label.to_lowercase();
            if label.contains(&query) {
                contains.push(i);
            } else if is_subsequence(&query, &label) {
                fuzzy.push(i);
            }
        }

        contains.extend(fuzzy);
        self.visible = contains;
        self.state.select(if self.visible.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    pub fn handle_filter_event(&mut self, event: &Event) {
        if let Some(change) = self.filter.handle_event(event) {
            if change.value {
                self.apply_filter();
            }
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter.reset();
        self.apply_filter();
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn visible(&self) -> impl Iterator<Item = (usize, &T)> {
        self.visible.iter().map(|&i| (i, &self.items[i]))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state
            .selected()
            .and_then(|pos| self.visible.get(pos).copied())
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected_index().map(|i| &self.items[i])
    }

    pub fn select_item(&mut self, index: usize) {
        if let Some(pos) = self.visible.iter().position(|&i| i == index) {
            self.state.select(Some(pos));
        }
    }

    pub fn move_up(&mut self) {
        let len = self.visible.len();
        if len == 0 {
            return;
        }
        let pos = self.state.selected().unwrap_or(0);
        self.state.select(Some((pos + len - 1) % len));
    }

    pub fn move_down(&mut self) {
        let len = self.visible.len();
        if len == 0 {
            return;
        }
        let pos = self.state.selected().unwrap_or(0);
        self.state.select(Some((pos + 1) % len));
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    fn picker(items: &[&str]) -> Picker<String> {
        Picker::new(items.iter().map(|s| s.to_string()).collect(), |s| s.clone())
    }

    fn type_filter(picker: &mut Picker<String>, text: &str) {
        for c in text.chars() {
            let event = Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            picker.handle_filter_event(&event);
        }
    }

    fn visible(picker: &Picker<String>) -> Vec<&str> {
        picker.visible().map(|(_, s)| s.as_str()).collect()
    }

    #[test]
    fn test_filter_ranks_substring_before_fuzzy() {
        let mut p = picker(&["Sparkline", "Changed words", "Raw text", "Line chart"]);
        type_filter(&mut p, "line");
        assert_eq!(visible(&p), vec!["Sparkline", "Line chart"]);

        let mut p = picker(&["changed words", "chart", "cw"]);
        type_filter(&mut p, "cw");
        assert_eq!(visible(&p), vec!["cw", "changed words"]);
        assert_eq!(p.selected().map(String::as_str), Some("cw"));
    }

    #[test]
    fn test_selection_wraps_and_handles_empty() {
        let mut p = picker(&["a", "b", "c"]);
        p.move_up();
        assert_eq!(p.selected().map(String::as_str), Some("c"));
        p.move_down();
        assert_eq!(p.selected().map(String::as_str), Some("a"));

        type_filter(&mut p, "zzz");
        assert_eq!(p.selected(), None);
        p.move_down();
        assert_eq!(p.selected(), None);
    }

    #[test]
    fn test_set_items_keeps_position_in_range() {
        let mut p = picker(&["a", "b", "c"]);
        p.state.select(Some(2));
        p.set_items(vec!["a".to_string(), "b".to_string()], |s| s.clone());
        assert_eq!(p.selected().map(String::as_str), Some("b"));
    }
}
