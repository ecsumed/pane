use super::node_data::PaneNodeData;
use ratatui::layout::Direction;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct PaneKey;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaneNode {
    pub data: PaneNodeData,
    pub parent: Option<PaneKey>,
    pub weight: u16,
}

impl PaneNodeData {
    pub fn get_children(&self) -> Option<&Vec<PaneKey>> {
        if let PaneNodeData::Split { children, .. } = self {
            Some(children)
        } else {
            None
        }
    }
    pub fn get_direction(&self) -> Option<Direction> {
        if let PaneNodeData::Split { direction, .. } = self {
            Some(*direction)
        } else {
            None
        }
    }
}
