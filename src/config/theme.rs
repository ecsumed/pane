use ratatui::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub collapse_borders: bool,
    pub palette: Palette,
    pub show_display_type: bool,
    pub show_history_meter: bool,
    pub show_inline_deletions: bool,
    pub show_last_updated: bool,
    pub show_state: bool,
    pub show_status_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub border_active: Style,
    pub border_inactive: Style,
    pub border_label: Style,
    pub chart_bar: Style,
    pub chart_line: Style,
    pub chart_scatter: Style,
    pub counter_key: Style,
    pub diff_add: Style,
    pub diff_remove: Style,
    pub error: Style,
    pub h1: Style,
    pub h2: Style,
    pub meta_highlight: Style,
    pub meta_label: Style,
    pub meta_meter: Style,
    pub meta_secondary: Style,
    pub meta_value: Style,
    pub multiline_timestamp: Style,
    pub output: Style,
    pub scroll_bar: Style,
    pub scroll_track: Style,
    pub search_match: Style,
    pub spark_line: Style,
}
