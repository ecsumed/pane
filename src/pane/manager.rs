use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use slotmap::SlotMap;

use super::node::{PaneNode, PaneKey};
use super::enums::{PaneNodeData, CardinalDirection};
use crate::logging::{info, debug, trace};
use crate::session::PaneKeyAsString;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaneManager {
    pub nodes: SlotMap<PaneKey, PaneNode>,
    pub active_pane_id: PaneKey,

    #[serde_as(as = "HashMap<PaneKeyAsString, _>")]
    pub pane_key_to_friendly_id: HashMap<PaneKey, usize>,
    pub id_counter: usize,
}

impl fmt::Display for PaneManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- Pane Tree Structure ---")?;
        let root_index = self.find_root();
        
        if let Some(index) = root_index {
            writeln!(f, "Root index: {:?}", index)?;
            writeln!(f, "Active pane id: {:?}", &self.active_pane_id)?;
            self.fmt_node_recursive(index, 0, f)?;
        } else {
            writeln!(f, "Error: No root node found.")?;
        }
        write!(f, "---------------------------")
    }
}

impl PaneManager {
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let mut pane_key_to_friendly_id = HashMap::new();
        let id_counter = 1;

        let root_key = nodes.insert(PaneNode {
            data: PaneNodeData::Single,
            parent: None,
            weight: 1,
        });

        pane_key_to_friendly_id.insert(root_key, id_counter);

        PaneManager {
            nodes,
            active_pane_id: root_key,
            pane_key_to_friendly_id,
            id_counter: id_counter + 1,
        }
    }
    
    fn find_root(&self) -> Option<PaneKey> {
        self.nodes.iter().find(|(_, node)| node.parent.is_none()).map(|(key, _)| key)
    }

    fn fmt_node_recursive(&self, node_key: PaneKey, depth: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(node) = self.nodes.get(node_key) {
            let indent = "  ".repeat(depth);
            let parent_id_str = node.parent.map(|p_key| format!("{:?}", p_key)).unwrap_or("None".to_string());
            
            let node_type_label = match node.data {
                PaneNodeData::Single { .. } => "Single", 
                PaneNodeData::Split { .. } => "Split", 
            };

            match &node.data {
                PaneNodeData::Single { .. } => { 
                    let pane_id = self.pane_key_to_friendly_id(&node_key).unwrap_or(0);
                    
                    writeln!(
                        f,
                        "{}[{:?}] {} ID: {}, Weight: {}, Parent: {}",
                        indent,
                        node_key,
                        node_type_label,
                        pane_id,
                        node.weight,
                        parent_id_str
                    )?;
                }
                PaneNodeData::Split { direction, children } => {
                    let direction_str = match direction {
                        Direction::Vertical => "Vertical",
                        Direction::Horizontal => "Horizontal",
                    };
                    writeln!(
                        f,
                        "{}[{:?}] {} ({}) Children: {}, Parent: {}",
                        indent,
                        node_key,
                        node_type_label,
                        direction_str,
                        children.len(),
                        parent_id_str
                    )?;

                    for &child_key in children {
                        self.fmt_node_recursive(child_key, depth + 1, f)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_root_pane_key(&self) -> Option<PaneKey> {
        self.nodes.iter().find(|(_, node)| node.parent.is_none()).map(|(key, _)| key)
    }
    
    pub fn pane_key_to_friendly_id(&self, node_key: &PaneKey) -> Option<usize> {
        self.pane_key_to_friendly_id
            .get(node_key)
            .copied()
    }

    pub fn get_all_pane_keys(&self) -> Vec<PaneKey> {
        let mut keys = Vec::new();
        if let Some(root_key) = self.find_root() {
            self.traverse_for_keys(root_key, &mut keys);
        }
        keys
    }

    fn traverse_for_keys(&self, node_key: PaneKey, keys: &mut Vec<PaneKey>) {
        if let Some(node) = self.nodes.get(node_key) {
            match &node.data {
                PaneNodeData::Single => {
                    keys.push(node_key);
                }
                PaneNodeData::Split { children, .. } => {
                    for &child_key in children {
                        self.traverse_for_keys(child_key, keys);
                    }
                }
            }
        }
    }

    pub fn split_pane(&mut self, direction: Direction) -> bool {
        info!("Splitting pane {:?}", self.active_pane_id);

        let Some(split_node) = self.nodes.get(self.active_pane_id) else {
            return false;
        };
        let active_id = self.active_pane_id;
        let parent_key: Option<PaneKey> = split_node.parent;
    
        let new_split_key = self.nodes.insert(PaneNode {
            data: PaneNodeData::Split {
                direction,
                children: vec![active_id],
            },
            parent: parent_key,
            weight: split_node.weight,
        });
    
        let new_single_key = self.nodes.insert(PaneNode {
            data: PaneNodeData::Single,
            parent: Some(new_split_key),
            weight: 1,
        });
    
        if let Some(new_split_node) = self.nodes.get_mut(new_split_key) {
            if let PaneNodeData::Split { children, .. } = &mut new_split_node.data {
                children.push(new_single_key);
            }
        }
    
        let original_node = self.nodes.get_mut(active_id).unwrap();
        original_node.parent = Some(new_split_key);
        original_node.weight = 1;
    
        if let Some(p_key) = parent_key {
            let parent_node = self.nodes.get_mut(p_key).unwrap();
            if let PaneNodeData::Split { children, .. } = &mut parent_node.data {
                if let Some(pos) = children.iter().position(|&c| c == active_id) {
                    children[pos] = new_split_key;
                }
            }
        } else {
            self.active_pane_id = new_split_key;
        }
    
        self.active_pane_id = new_single_key;

        // Maintain our user-readable ids
        self.pane_key_to_friendly_id.insert(new_single_key, self.id_counter);
        self.id_counter += 1;

        debug!("{}", self);

        true
    }
    
    pub fn cycle_panes(&mut self) {
        info!("Cycling pane");

        let pane_keys = self.get_all_pane_keys();
        
        if pane_keys.is_empty() {
            return;
        }
    
        if let Some(pos) = pane_keys
            .iter()
            .position(|&p_key| p_key == self.active_pane_id)
        {
            let next_pos = (pos + 1) % pane_keys.len();
            
            self.active_pane_id = pane_keys[next_pos];

        } else {
            self.active_pane_id = pane_keys[0];
        }
    
        debug!("{}", self);
    }

    fn replace_child_in_parent(
        nodes: &mut SlotMap<PaneKey, PaneNode>,
        parent_id: PaneKey,
        old_child_id: PaneKey,
        new_child_id: PaneKey,
    ) {
        if let Some(parent_node) = nodes.get_mut(parent_id) {
            if let PaneNodeData::Split { children, .. } = &mut parent_node.data {
                if let Some(pos) = children.iter().position(|&c| c == old_child_id) {
                    children[pos] = new_child_id;
                }
            }
        }
    }

    pub fn kill_pane(&mut self) -> bool {
        info!("Killing pane {:?}", self.active_pane_id);

        let active_id = self.active_pane_id;
        let Some(active_node) = self.nodes.get(active_id) else {
            return false;
        };
        let Some(parent_id) = active_node.parent else {
            info!("Can't kill last pane {:?}", self.active_pane_id);
            return self.nodes.len() > 1;
        };

        // Store cloned copies to avoid borrow checker issues with mutable borrows.
        let parent_id_clone = parent_id;
        let grand_parent_id = self.nodes.get(parent_id_clone).and_then(|p| p.parent);
        
        // 1. Remove the active_id from the parent's children list.
        let last_child_id_after_removal = if let Some(parent_node) = self.nodes.get_mut(parent_id_clone) {
            if let PaneNodeData::Split { children, .. } = &mut parent_node.data {
                children.retain(|&id| id != active_id);
                if children.len() == 1 {
                    children.pop()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // 2. If a single child remains, promote it.
        if let Some(promoted_child_id) = last_child_id_after_removal {
            if let Some(promoted_child_node) = self.nodes.get_mut(promoted_child_id) {
                promoted_child_node.parent = grand_parent_id;
            }

            // Replace the old parent with the promoted child in the grandparent's list.
            if let Some(gp_id) = grand_parent_id {
                PaneManager::replace_child_in_parent(&mut self.nodes, gp_id, parent_id_clone, promoted_child_id);
            }

            // The parent split node is no longer needed.
            self.nodes.remove(parent_id_clone);
        }
        
        // 3. Remove the active pane itself.
        self.nodes.remove(active_id);
        
        // 4. Update the active pane.
        self.active_pane_id = self.find_next_active();

        debug!("{}", self);
        true
    }

    fn find_next_active(&self) -> PaneKey {
        let keys = self.get_all_pane_keys();
        return keys[keys.len() - 1]
    }

    pub fn resize_pane(
        &mut self,
        // We only need the direction here.
        direction: &CardinalDirection, 
        // The amount should be passed as signed (i8/i16) as before
        amount: i8,
    ) -> bool {
        let active_id = self.active_pane_id;
        let mut current_id = active_id;
    
        // We iterate up the tree, checking each parent level.
        loop {
            let Some(parent_id) = self.nodes.get(current_id).and_then(|n| n.parent) else {
                return false; // Reached the root, cannot resize further in this direction.
            };
    
            // Check if the current parent's split direction matches the resize direction.
            let parent_node_data = self.nodes.get(parent_id).expect("Parent exists").data.clone();
            let (split_direction, children) = match parent_node_data {
                PaneNodeData::Split { direction, children } => (direction, children),
                _ => return false, // Parent must be a split.
            };
            
            let is_valid_split = matches!(
                (direction, &split_direction),
                (CardinalDirection::Left | CardinalDirection::Right, Direction::Vertical) |
                (CardinalDirection::Up | CardinalDirection::Down, Direction::Horizontal)
            );
    
            if is_valid_split {
                // Found the correct level to resize. Apply changes here.
                
                // Determine the sibling index using your logic
                let active_index = children.iter().position(|&id| id == current_id).expect("Active pane in parent's children");
                
                let sibling_index = if active_index == 0 { 1 } else { active_index - 1 };
                
                let Some(&sibling_id) = children.get(sibling_index) else { return false; };
    
                // Determine if we increase or decrease based on user input
                let final_amount = amount as i16;
                
                // Apply the updates with clamping. The logic is now much cleaner:
                
                let current_weight_active = self.nodes[current_id].weight as i16;
                let current_weight_sibling = self.nodes[sibling_id].weight as i16;
    
                let new_active_weight = (current_weight_active + final_amount).max(1) as u16;
                let new_sibling_weight = (current_weight_sibling - final_amount).max(1) as u16;
    
                // Apply updates only if the weights actually change
                if new_active_weight != self.nodes[current_id].weight || new_sibling_weight != self.nodes[sibling_id].weight {
                    self.nodes.get_mut(current_id).unwrap().weight = new_active_weight;
                    self.nodes.get_mut(sibling_id).unwrap().weight = new_sibling_weight;
                    return true;
                } else {
                    return false; // Weights didn't change (hit a boundary)
                }
            } else {
                // This is not the right level to apply the change (e.g., trying to resize horizontally in a horizontal split).
                // Continue up the tree to the next parent level.
                current_id = parent_id;
                continue;
            }
        }
    }
    
    // Private helper to perform the actual resize on a known split.
    fn resize_in_split(
        &mut self,
        split_id: PaneKey,
        active_id: PaneKey,
        amount: i8,
        direction: &CardinalDirection,
    ) -> bool {
        let parent_node = self.nodes.get(split_id).expect("Split exists");
        let children = parent_node.data.get_children().expect("Split must have children");
        let active_index = children.iter().position(|&id| id == active_id).expect("Active pane in split");
    
        let mut delta = 0;
        let mut sibling_id: Option<PaneKey> = None;
    
        match (direction, parent_node.data.get_direction()) {
            (CardinalDirection::Left, Some(Direction::Vertical)) if active_index > 0 => {
                sibling_id = Some(children[active_index - 1]);
                delta = -amount as i16;
            },
            (CardinalDirection::Right, Some(Direction::Vertical)) if active_index < children.len() - 1 => {
                sibling_id = Some(children[active_index + 1]);
                delta = amount as i16;
            },
            (CardinalDirection::Up, Some(Direction::Horizontal)) if active_index > 0 => {
                sibling_id = Some(children[active_index - 1]);
                delta = -amount as i16;
            },
            (CardinalDirection::Down, Some(Direction::Horizontal)) if active_index < children.len() - 1 => {
                sibling_id = Some(children[active_index + 1]);
                delta = amount as i16;
            },
            _ => return false,
        }
    
        if let Some(sibling_id) = sibling_id {
            let current_weight_active = self.nodes[active_id].weight as i16;
            let current_weight_sibling = self.nodes[sibling_id].weight as i16;
    
            let new_active_weight_signed = current_weight_active + delta;
            let new_sibling_weight_signed = current_weight_sibling - delta;
    
            let final_active_weight = new_active_weight_signed.max(1) as u16;
            let final_sibling_weight = new_sibling_weight_signed.max(1) as u16;
            
            if final_active_weight != current_weight_active as u16 || final_sibling_weight != current_weight_sibling as u16 {
                self.nodes.get_mut(active_id).unwrap().weight = final_active_weight;
                self.nodes.get_mut(sibling_id).unwrap().weight = final_sibling_weight;
                return true;
            }
        }
        false
    }
 
    pub fn get_pane_bounds(&self, total_size: Rect) -> BTreeMap<PaneKey, Rect> {
        let mut bounds_map = BTreeMap::new();
        // Assuming the root is the only node with no parent
        if let Some(root_key) = self.nodes.keys().find(|&key| self.nodes[key].parent.is_none()) {
            Self::calculate_bounds(&self.nodes, root_key, total_size, &mut bounds_map);
        }
        bounds_map
    }

    fn calculate_bounds(
        nodes: &SlotMap<PaneKey, PaneNode>,
        current_key: PaneKey,
        rect: Rect,
        bounds_map: &mut BTreeMap<PaneKey, Rect>,
    ) {
        let Some(node) = nodes.get(current_key) else { return; };

        match &node.data {
            PaneNodeData::Single => {
                bounds_map.insert(current_key, rect);
            }
            PaneNodeData::Split { direction, children } => {
                let total_weight: u16 = children.iter()
                    .filter_map(|&key| nodes.get(key))
                    .map(|n| n.weight)
                    .sum();

                let mut constraints: Vec<Constraint> = if total_weight > 0 {
                    children.iter()
                        .map(|&key| {
                            let child_node = nodes.get(key).expect("Child pane not found");
                            Constraint::Ratio(child_node.weight as u32, total_weight as u32)
                        })
                        .collect()
                } else {
                    // All children have a weight of 0, distribute evenly
                    let num_children = children.len() as u32;
                    if num_children == 0 { return; }
                    (0..num_children)
                        .map(|_| Constraint::Ratio(1, num_children))
                        .collect()
                };

                let direction = match direction {
                    Direction::Vertical => ratatui::layout::Direction::Vertical,
                    Direction::Horizontal => ratatui::layout::Direction::Horizontal,
                };
                
                let chunks = Layout::default()
                    .direction(direction)
                    .constraints(constraints)
                    .split(rect);

                for (i, &child_key) in children.iter().enumerate() {
                    if let Some(&child_rect) = chunks.get(i) {
                        Self::calculate_bounds(nodes, child_key, child_rect, bounds_map);
                    }
                }
            }
        }
    }
    pub fn change_active(&mut self, direction: &CardinalDirection, total_size: Rect) -> bool {
        let active_id = self.active_pane_id;
        // Calculate all bounds (you might want to cache this in your App struct)
        let bounds_map = self.get_pane_bounds(total_size);

        debug!("{:?}", bounds_map);
        
        let Some(active_bounds) = bounds_map.get(&active_id) else { return false; };
        
        let active_center = (
            active_bounds.x as f32 + active_bounds.width as f32 / 2.0,
            active_bounds.y as f32 + active_bounds.height as f32 / 2.0,
        );

        let next_active_id = self.find_next_pane_id_center_point(
            direction, 
            active_id, 
            active_center, 
            &bounds_map
        );

        if let Some(next_id) = next_active_id {
            self.active_pane_id = next_id;
            true
        } else {
            false
        }
    }

    fn find_next_pane_id_center_point(
        &self,
        direction: &CardinalDirection,
        active_id: PaneKey,
        active_center: (f32, f32),
        bounds_map: &BTreeMap<PaneKey, Rect>,
    ) -> Option<PaneKey> {
        let mut best_candidate: Option<(PaneKey, f32)> = None; // (Key, distance_squared)

        for (&candidate_key, &candidate_bounds) in bounds_map.iter() {
            if candidate_key == active_id { continue; }

            let candidate_center = (
                candidate_bounds.x as f32 + candidate_bounds.width as f32 / 2.0,
                candidate_bounds.y as f32 + candidate_bounds.height as f32 / 2.0,
            );

            let is_in_direction = match direction {
                CardinalDirection::Up    => candidate_center.1 < active_center.1,
                CardinalDirection::Down  => candidate_center.1 > active_center.1,
                CardinalDirection::Left  => candidate_center.0 < active_center.0,
                CardinalDirection::Right => candidate_center.0 > active_center.0,
            };

            if is_in_direction {
                // Calculate Euclidean distance squared (saves a sqrt call)
                let dx = candidate_center.0 - active_center.0;
                let dy = candidate_center.1 - active_center.1;
                let distance_sq = dx * dx + dy * dy;

                if best_candidate.is_none() || distance_sq < best_candidate.unwrap().1 {
                    best_candidate = Some((candidate_key, distance_sq));
                }
            }
        }

        best_candidate.map(|(key, _)| key)
    }
}
