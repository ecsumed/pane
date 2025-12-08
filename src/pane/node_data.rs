use ratatui::layout::Direction;
use serde::{Deserialize, Serialize};

use super::node::PaneKey;

#[derive(Debug, Copy, Clone)]
pub enum CardinalDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PaneNodeData {
    Split {
        #[serde(with = "super::serialization::direction_serialization")]
        direction: Direction,
        children: Vec<PaneKey>,
    },
    Single,
}
