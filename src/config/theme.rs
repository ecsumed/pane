use ratatui::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub collapse_borders: bool,
    pub show_state: bool,
    pub show_last_updated: bool,
    pub show_display_type: bool,
    pub show_status_bar: bool,
    pub palette: Palette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub border_active: Style,
    pub border_inactive: Style,
    pub border_label: Style,
    pub meta_label: Style,
    pub meta_value: Style,
    pub meta_highlight: Style,
    pub meta_secondary: Style,
    pub diff_add: Style,
    pub diff_remove: Style,
    pub search_match: Style,
    pub multiline_timestamp: Style,
    pub counter_key: Style,
    pub spark_line: Style,
    pub output: Style,
    pub error: Style,
}
