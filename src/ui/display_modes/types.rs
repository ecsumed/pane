use serde::{Deserialize, Serialize};
use strum::EnumIter;

use clap::ValueEnum;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize, ValueEnum,
)]
pub enum DisplayType {
    #[default]
    RawText,
    MultiLine,
    MultiLineTime,
    MultiLineDateTime,
    DiffChar,
    DiffLine,
    DiffWord,
    Sparkline,
    LineChart,
    BarChart,
    ScatterChart,
    Counter,
    BigText,
}

impl DisplayType {
    pub fn label(&self) -> &'static str {
        match self {
            DisplayType::RawText => "Raw text",
            DisplayType::MultiLine => "History",
            DisplayType::MultiLineTime => "History + time",
            DisplayType::MultiLineDateTime => "History + date/time",
            DisplayType::DiffChar => "Changed characters",
            DisplayType::DiffLine => "Changed lines",
            DisplayType::DiffWord => "Changed words",
            DisplayType::Sparkline => "Sparkline",
            DisplayType::LineChart => "Line chart",
            DisplayType::BarChart => "Bar chart",
            DisplayType::ScatterChart => "Scatter chart",
            DisplayType::Counter => "Counter",
            DisplayType::BigText => "Big text",
        }
    }

    pub fn group(&self) -> &'static str {
        match self {
            DisplayType::RawText
            | DisplayType::MultiLine
            | DisplayType::MultiLineTime
            | DisplayType::MultiLineDateTime => "Text",
            DisplayType::DiffChar | DisplayType::DiffLine | DisplayType::DiffWord => "Diff",
            DisplayType::Sparkline
            | DisplayType::LineChart
            | DisplayType::BarChart
            | DisplayType::ScatterChart => "Charts",
            DisplayType::Counter | DisplayType::BigText => "Other",
        }
    }

    pub fn needs_numbers(&self) -> bool {
        self.group() == "Charts"
    }
}
