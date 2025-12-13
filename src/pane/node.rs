use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

use super::node_data::PaneNodeData;

new_key_type! {
    pub struct PaneKey;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaneNode {
    pub data: PaneNodeData,
    pub parent: Option<PaneKey>,
    pub weight: u16,
}
