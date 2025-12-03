use slotmap::{SlotMap, new_key_type};

pub mod manager;
pub mod node;
pub mod enums;
pub mod serialization;

pub use self::manager::PaneManager;
pub use self::node::{PaneNode, PaneKey};
pub use self::enums::{CardinalDirection, PaneNodeData};