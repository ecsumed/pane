use crate::logging::{info, debug, trace};
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
        children: Vec<(u16, Panes)>,
    },
    Single {
        id: usize,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum ResizeDirection {
    Up,
    Down,
    Left,
    Right,
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
                    children: vec![
                        (1, Panes::Single { id }),
                        (1, Panes::Single { id: new_id }),
                    ],
                };
                true
            }
            Panes::Split { children, .. } => {
                for (weight, child) in children {
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
        info!("Attempting to kill pane {id}");

        match self {
            Panes::Single { id: single_id, .. } if *single_id == id => {
                // don't kill the last pane
                false
            }

            Panes::Split { children, .. } => {
                let mut found = false;

                // First pass: Find and remove the child.
                let mut temp_children = Vec::new();
                for (weight, child) in children.drain(..) {
                    if let Panes::Single { id: child_id, .. } = child {
                        if child_id == id {
                            found = true;
                            continue; // Skip this child
                        }
                    }
                    temp_children.push((weight, child));
                }
                *children = temp_children;

                // Second pass (recursive):
                if !found {
                    for (_weight, child) in children.iter_mut() {
                        if child.kill_pane(id) {
                            found = true;
                            break; // Stop after finding the first one.
                        }
                    }
                }

                // If only one child remains, promote it.
                if children.len() == 1 {
                    *self = children.remove(0).1; 
                }

                found // Return the final found status
            }
            _ => false,
        }
    }

    pub fn resize_pane(
        &mut self,
        id: usize,
        amount: i32,
        resize_direction: &ResizeDirection,
    ) -> bool {
        let change_amount = amount.abs() as u16;
    
        match self {
            Panes::Single { .. } => false,
    
            Panes::Split { direction, children } => {
                let mut target_index = None;
                for (i, (_, child)) in children.iter().enumerate() {
                    if child.find_pane_id(id) {
                        target_index = Some(i);
                        break;
                    }
                }
    
                if let Some(index) = target_index {
                    let is_vertical_resize = matches!(resize_direction, ResizeDirection::Up | ResizeDirection::Down);
                    let is_horizontal_resize = matches!(resize_direction, ResizeDirection::Left | ResizeDirection::Right);
    
                    // Check if this split's direction aligns with the resize direction
                    if (is_vertical_resize && *direction == Direction::Vertical) ||
                       (is_horizontal_resize && *direction == Direction::Horizontal)
                    {
                        // This is the correct level to apply the change.
                        // Find the sibling index:
                        let sibling_index = Panes::get_sibling_index(index, children.len());
    
                        if let Some(s_index) = sibling_index {
                            // Apply the weight changes directly and return true (no recursion needed after a change)
                            if amount > 0 { // Increase target, decrease sibling
                                children[index].0 = children[index].0.saturating_add(change_amount);
                                children[s_index].0 = children[s_index].0.saturating_sub(change_amount).max(1);
                            } else { // Decrease target, increase sibling
                                children[index].0 = children[index].0.saturating_sub(change_amount).max(1);
                                children[s_index].0 = children[s_index].0.saturating_add(change_amount);
                            }
                            return true;
                        }
                    }
                    
                    // If this wasn't the correct split level, or the resize didn't happen (e.g., edge of the screen),
                    // recurse down into the child containing the active pane to find the right split level.
                    if children[index].1.resize_pane(id, amount, resize_direction) {
                        return true;
                    }
                }
                false
            }
        }
    }
            
    pub fn find_pane_id(&self, id: usize) -> bool {
        match self {
            Panes::Single { id: single_id } => *single_id == id,
            Panes::Split { children, .. } => children.iter().any(|(_, child)| child.find_pane_id(id)),
        }
    }

    fn get_sibling_index(target_index: usize, children_len: usize) -> Option<usize> {
        // The index of the last element
        let last_index = children_len - 1;
    
        let sibling_index = if target_index == 0 {
            // First index becomes second index
            target_index.checked_add(1)
        } else if target_index == last_index {
            // Last index becomes the one twice before it
            // this is so the controls act naturally
            target_index.checked_sub(1)
        } else {
            // Otherwise, add one (favor the right/bottom sibling)
            target_index.checked_add(1)
        };
    
        // Filter the result to ensure the sibling index is within bounds
        sibling_index.filter(|&i| i < children_len)
    }

    fn find_next_pane_recursive(&self, current_id: usize, direction: &ResizeDirection) -> Option<usize> {
        match self {
            Panes::Single { id } if *id == current_id => None, // Active pane is a single leaf, so search starts at its parent
            Panes::Single { .. } => None,
            Panes::Split { direction: split_direction, children } => {
                let is_vertical = matches!(direction, ResizeDirection::Up | ResizeDirection::Down);
                let is_horizontal = matches!(direction, ResizeDirection::Left | ResizeDirection::Right);

                for (i, (_, child)) in children.iter().enumerate() {
                    if let Some(next_id) = child.find_next_pane_recursive(current_id, direction) {
                        return Some(next_id);
                    }
                    if child.find_pane_id(current_id) {
                        if (is_vertical && *split_direction == Direction::Vertical) ||
                           (is_horizontal && *split_direction == Direction::Horizontal)
                        {
                            let next_index = match direction {
                                ResizeDirection::Up | ResizeDirection::Left => i.checked_sub(1),
                                ResizeDirection::Down | ResizeDirection::Right => i.checked_add(1).filter(|&i| i < children.len()),
                            };
                            if let Some(next_i) = next_index {
                                return children[next_i].1.get_first_pane_id();
                            }
                        }
                        // If direction doesn't align or no sibling found, propagate None upwards.
                        return None;
                    }
                }
                None
            }
        }
    }

    pub fn find_next_pane(&self, current_id: usize, direction: &ResizeDirection) -> Option<usize> {
        self.find_next_pane_recursive(current_id, direction)
    }

    pub fn get_first_pane_id(&self) -> Option<usize> {
        match self {
            Panes::Single { id } => Some(*id),
            Panes::Split { children, .. } => {
                if children.is_empty() {
                    None
                } else {
                    children[0].1.get_first_pane_id()
                }
            }
        }
    }
}

//     if (is_vertical_resize && *direction == Direction::Vertical) ||
//        (is_horizontal_resize && *direction == Direction::Horizontal)
//     {
//         let sibling_index = match resize_direction {
//             ResizeDirection::Up | ResizeDirection::Left if amount > 0 => target_index.checked_sub(1),
//             ResizeDirection::Down | ResizeDirection::Right if amount < 0 => target_index.checked_sub(1),
//             ResizeDirection::Up | ResizeDirection::Left if amount < 0 => target_index.checked_add(1).filter(|&i| i < children.len()),
//             ResizeDirection::Down | ResizeDirection::Right if amount > 0 => target_index.checked_add(1).filter(|&i| i < children.len()),
//             _ => None,
//         };

//         if let Some(s_index) = sibling_index {
//             // Modify weights after determining the indices
//             if amount > 0 { // Increase target, decrease sibling
//                 children[target_index].0 = children[target_index].0.saturating_add(change_amount);
//                 children[s_index].0 = children[s_index].0.saturating_sub(change_amount).max(1);
//             } else { // Decrease target, increase sibling
//                 children[target_index].0 = children[target_index].0.saturating_sub(change_amount).max(1);
//                 children[s_index].0 = children[s_index].0.saturating_add(change_amount);
//             }
//             return true;
//         }
//     }
// }

