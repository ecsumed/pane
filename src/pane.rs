use crate::logging::{info, trace};
use ratatui::layout::Direction;
use serde::{Deserialize, Serialize};

mod direction_serialization {
    use ratatui::layout::Direction;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(direction: &Direction, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match direction {
            Direction::Vertical => "Vertical",
            Direction::Horizontal => "Horizontal",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Direction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Vertical" => Ok(Direction::Vertical),
            "Horizontal" => Ok(Direction::Horizontal),
            _ => Err(serde::de::Error::custom("Invalid Direction string")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Panes {
    Split {
        #[serde(with = "direction_serialization")]
        direction: Direction,
        children: Vec<Panes>,
    },
    Single {
        id: usize,
    },
}

impl Default for Panes {
    fn default() -> Self {
        Panes::Single { id: 1 }
    }
}

impl Panes {
    /// Finds the total number of single panes.
    // pub fn count_panes(&self) -> usize {
    //     match self {
    //         Panes::Single { .. } => 1,
    //         Panes::Split { children, .. } => children.iter().map(|c| c.count_panes()).sum(),
    //     }
    // }

    pub fn split_pane(&mut self, id: usize, direction: Direction, new_id: usize) -> bool {
        info!("Splitting pane with id: {id}");

        match self {
            Panes::Single { id: single_id, .. } if *single_id == id => {
                *self = Panes::Split {
                    direction,
                    children: vec![Panes::Single { id }, Panes::Single { id: new_id }],
                };
                true
            }
            Panes::Split { children, .. } => {
                for child in children {
                    if child.split_pane(id, direction, new_id) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn kill_pane(&mut self, id: usize) -> bool {
        trace!("Attempting to kill pane {id}");

        match self {
            Panes::Single { id: single_id, .. } if *single_id == id => {
                // don't kill the last pane
                false
            }

            Panes::Split { children, .. } => {
                let mut found = false;

                // First pass: Find and remove the child.
                children.retain(|child| {
                    if let Panes::Single { id: child_id, .. } = child {
                        if *child_id == id {
                            found = true;
                            return false; // Remove this child.
                        }
                    }
                    true // Keep other children.
                });

                // Second pass (recursive):
                if !found {
                    for child in children.iter_mut() {
                        if child.kill_pane(id) {
                            found = true;
                            break; // Stop after finding the first one.
                        }
                    }
                }

                // If only one child remains, promote it.
                if children.len() == 1 {
                    *self = children.remove(0);
                }

                found // Return the final found status
            }
            _ => false,
        }
    }
}
