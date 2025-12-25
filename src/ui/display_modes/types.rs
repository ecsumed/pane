use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum DisplayType {
    #[default]
    RawText,
    MultiLine,
    MultiLineTime,
    MultiLineDateTime,
    DiffChar,
    DiffLine,
    DiffWord,
    // LineChart,
    // Counter,
    // PieChart,
}
