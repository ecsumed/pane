use slotmap::{SlotMap, new_key_type};

pub mod manager;
pub mod node;
pub mod node_data;
pub mod serialization;

pub use self::manager::PaneManager;
pub use self::node::{PaneNode, PaneKey};
pub use self::node_data::{CardinalDirection, PaneNodeData};