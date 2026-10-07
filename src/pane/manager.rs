use std::collections::HashMap;
use std::fmt;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use slotmap::SlotMap;

use super::navigation::neighbour;
use super::node::{PaneKey, PaneNode};
use super::node_data::{CardinalDirection, PaneNodeData};
use crate::logging::{debug, info};
use crate::session::PaneKeyAsString;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaneManager {
    pub nodes: SlotMap<PaneKey, PaneNode>,
    pub active_pane_id: PaneKey,

    #[serde_as(as = "HashMap<PaneKeyAsString, _>")]
    pub pane_key_to_friendly_id: HashMap<PaneKey, usize>,
    pub id_counter: usize,

    #[serde(skip)]
    previous_active: Option<PaneKey>,
}

const SHARE_TOTAL: u32 = 1000;
const RESIZE_STEP: u32 = 50;
const MIN_SHARE: u32 = 50;

fn axis(direction: CardinalDirection) -> Direction {
    match direction {
        CardinalDirection::Left | CardinalDirection::Right => Direction::Horizontal,
        CardinalDirection::Up | CardinalDirection::Down => Direction::Vertical,
    }
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
            previous_active: None,
        }
    }

    fn find_root(&self) -> Option<PaneKey> {
        self.nodes
            .iter()
            .find(|(_, node)| node.parent.is_none())
            .map(|(key, _)| key)
    }

    fn fmt_node_recursive(
        &self,
        node_key: PaneKey,
        depth: usize,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if let Some(node) = self.nodes.get(node_key) {
            let indent = "  ".repeat(depth);
            let parent_id_str = node
                .parent
                .map(|p_key| format!("{:?}", p_key))
                .unwrap_or("None".to_string());

            let node_type_label = match node.data {
                PaneNodeData::Single => "Single",
                PaneNodeData::Split { .. } => "Split",
            };

            match &node.data {
                PaneNodeData::Single => {
                    let pane_id = self.pane_key_to_friendly_id(&node_key).unwrap_or(0);

                    writeln!(
                        f,
                        "{}[{:?}] {} ID: {}, Weight: {}, Parent: {}",
                        indent, node_key, node_type_label, pane_id, node.weight, parent_id_str
                    )?;
                }
                PaneNodeData::Split {
                    direction,
                    children,
                } => {
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

    pub fn pane_key_to_friendly_id(&self, node_key: &PaneKey) -> Option<usize> {
        self.pane_key_to_friendly_id.get(node_key).copied()
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

        let active_id = self.active_pane_id;
        let Some(node) = self.nodes.get(active_id) else {
            return false;
        };
        let parent = node.parent;
        let weight = node.weight;

        let split = self.nodes.insert(PaneNode {
            data: PaneNodeData::Split {
                direction,
                children: Vec::new(),
            },
            parent,
            weight,
        });
        let new_pane = self.nodes.insert(PaneNode {
            data: PaneNodeData::Single,
            parent: Some(split),
            weight: 1,
        });

        if let PaneNodeData::Split { children, .. } = &mut self.nodes[split].data {
            children.extend([active_id, new_pane]);
        }
        let original = &mut self.nodes[active_id];
        original.parent = Some(split);
        original.weight = 1;

        if let Some(parent) = parent {
            self.replace_child(parent, active_id, split);
        }

        self.focus_pane(new_pane);
        self.pane_key_to_friendly_id
            .insert(new_pane, self.id_counter);
        self.id_counter += 1;

        debug!("{}", self);
        true
    }

    fn focus_pane(&mut self, key: PaneKey) {
        if key != self.active_pane_id {
            self.previous_active = Some(self.active_pane_id);
            self.active_pane_id = key;
        }
    }

    pub fn cycle_panes(&mut self) {
        let pane_keys = self.get_all_pane_keys();
        let next = pane_keys
            .iter()
            .position(|&key| key == self.active_pane_id)
            .map_or(0, |pos| (pos + 1) % pane_keys.len().max(1));
        if let Some(&key) = pane_keys.get(next) {
            self.focus_pane(key);
        }
    }

    fn replace_child(&mut self, parent: PaneKey, old: PaneKey, new: PaneKey) {
        if let Some(PaneNode {
            data: PaneNodeData::Split { children, .. },
            ..
        }) = self.nodes.get_mut(parent)
        {
            if let Some(pos) = children.iter().position(|&c| c == old) {
                children[pos] = new;
            }
        }
    }

    fn children(&self, key: PaneKey) -> &[PaneKey] {
        match self.nodes.get(key).map(|n| &n.data) {
            Some(PaneNodeData::Split { children, .. }) => children,
            _ => &[],
        }
    }

    fn edge_leaf(&self, mut key: PaneKey, first: bool) -> PaneKey {
        loop {
            let children = self.children(key);
            let next = if first {
                children.first()
            } else {
                children.last()
            };
            match next {
                Some(&child) => key = child,
                None => return key,
            }
        }
    }

    pub fn kill_pane(&mut self) -> bool {
        info!("Killing pane {:?}", self.active_pane_id);

        let active_id = self.active_pane_id;
        let Some(parent) = self.nodes.get(active_id).and_then(|n| n.parent) else {
            info!("Can't kill last pane {:?}", active_id);
            return false;
        };
        let removed_weight = self.nodes[active_id].weight;

        let PaneNodeData::Split { children, .. } = &mut self.nodes[parent].data else {
            return false;
        };
        let Some(index) = children.iter().position(|&c| c == active_id) else {
            return false;
        };
        children.remove(index);
        let neighbour_was_after = index < children.len();
        let neighbour = children[index.min(children.len() - 1)];
        let only_child = (children.len() == 1).then(|| children[0]);

        self.nodes[neighbour].weight += removed_weight;

        if let Some(promoted) = only_child {
            let grandparent = self.nodes[parent].parent;
            let parent_weight = self.nodes[parent].weight;
            let node = &mut self.nodes[promoted];
            node.parent = grandparent;
            node.weight = parent_weight;
            if let Some(grandparent) = grandparent {
                self.replace_child(grandparent, parent, promoted);
            }
            self.nodes.remove(parent);
        }

        self.nodes.remove(active_id);
        self.pane_key_to_friendly_id.remove(&active_id);
        self.active_pane_id = self.edge_leaf(neighbour, neighbour_was_after);
        self.previous_active = None;

        debug!("{}", self);
        true
    }

    fn levels_on_axis(&self, direction: Direction) -> Vec<(PaneKey, usize, usize)> {
        let mut levels = Vec::new();
        let mut current = self.active_pane_id;
        while let Some(parent) = self.nodes.get(current).and_then(|n| n.parent) {
            if let PaneNodeData::Split {
                direction: split, ..
            } = &self.nodes[parent].data
            {
                let children = self.children(parent);
                if *split == direction {
                    if let Some(index) = children.iter().position(|&c| c == current) {
                        levels.push((parent, index, children.len()));
                    }
                }
            }
            current = parent;
        }
        levels
    }

    fn normalize(&mut self, parent: PaneKey) {
        let children = self.children(parent).to_vec();
        let total: u32 = children.iter().map(|&c| self.nodes[c].weight as u32).sum();
        if total == SHARE_TOTAL || total == 0 {
            return;
        }
        let mut assigned = 0;
        for (i, &child) in children.iter().enumerate() {
            let share = if i + 1 == children.len() {
                SHARE_TOTAL - assigned
            } else {
                self.nodes[child].weight as u32 * SHARE_TOTAL / total
            };
            assigned += share;
            self.nodes[child].weight = share as u16;
        }
    }

    fn transfer(&mut self, parent: PaneKey, from: usize, to: usize) -> bool {
        self.normalize(parent);
        let children = self.children(parent);
        let (from, to) = (children[from], children[to]);
        let available = (self.nodes[from].weight as u32).saturating_sub(MIN_SHARE);
        let amount = available.min(RESIZE_STEP) as u16;
        if amount == 0 {
            return false;
        }
        self.nodes[from].weight -= amount;
        self.nodes[to].weight += amount;
        true
    }

    pub fn resize_active(&mut self, direction: Direction, grow: bool) -> bool {
        let Some(&(parent, index, len)) = self
            .levels_on_axis(direction)
            .iter()
            .find(|(_, _, len)| *len > 1)
        else {
            return false;
        };
        let other = if index + 1 < len {
            index + 1
        } else {
            index - 1
        };
        if grow {
            self.transfer(parent, other, index)
        } else {
            self.transfer(parent, index, other)
        }
    }

    pub fn move_border(&mut self, direction: CardinalDirection) -> bool {
        let forward = matches!(
            direction,
            CardinalDirection::Right | CardinalDirection::Down
        );
        let levels = self.levels_on_axis(axis(direction));

        let ahead = |&&(_, index, len): &&(PaneKey, usize, usize)| {
            if forward {
                index + 1 < len
            } else {
                index > 0
            }
        };
        let behind = |&&(_, index, len): &&(PaneKey, usize, usize)| {
            if forward {
                index > 0
            } else {
                index + 1 < len
            }
        };
        let step = |index: usize| if forward { index + 1 } else { index - 1 };
        let back = |index: usize| if forward { index - 1 } else { index + 1 };

        if let Some(&(parent, index, _)) = levels.iter().find(ahead) {
            return self.transfer(parent, step(index), index);
        }
        if let Some(&(parent, index, _)) = levels.iter().find(behind) {
            return self.transfer(parent, index, back(index));
        }
        false
    }

    pub fn equalize(&mut self) {
        let splits: Vec<PaneKey> = self
            .nodes
            .iter()
            .filter(|(_, n)| matches!(n.data, PaneNodeData::Split { .. }))
            .map(|(key, _)| key)
            .collect();
        for split in splits {
            for child in self.children(split).to_vec() {
                self.nodes[child].weight = 1;
            }
        }
    }

    pub fn layout(&self, area: Rect, collapse: bool) -> Vec<(PaneKey, Rect)> {
        let mut rects = Vec::new();
        if let Some(root) = self.find_root() {
            self.layout_node(root, area, collapse, &mut rects);
        }
        rects
    }

    fn layout_node(
        &self,
        key: PaneKey,
        area: Rect,
        collapse: bool,
        rects: &mut Vec<(PaneKey, Rect)>,
    ) {
        let Some(node) = self.nodes.get(key) else {
            return;
        };
        let PaneNodeData::Split {
            direction,
            children,
        } = &node.data
        else {
            rects.push((key, area));
            return;
        };

        let total: u32 = children.iter().map(|&c| self.nodes[c].weight as u32).sum();
        let constraints = children
            .iter()
            .map(|&c| Constraint::Ratio(self.nodes[c].weight as u32, total.max(1)));
        let chunks = Layout::default()
            .direction(*direction)
            .constraints(constraints)
            .spacing(if collapse { -1 } else { 0 })
            .split(area);

        for (&child, &chunk) in children.iter().zip(chunks.iter()) {
            self.layout_node(child, chunk, collapse, rects);
        }
    }

    pub fn focus(&mut self, direction: CardinalDirection, rects: &[(PaneKey, Rect)]) -> bool {
        match neighbour(rects, self.active_pane_id, direction, self.previous_active) {
            Some(key) => {
                self.focus_pane(key);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 40,
    };

    fn rect_of(manager: &PaneManager, key: PaneKey) -> Rect {
        manager
            .layout(AREA, false)
            .into_iter()
            .find(|(k, _)| *k == key)
            .unwrap()
            .1
    }

    fn side_by_side() -> (PaneManager, PaneKey, PaneKey) {
        let mut manager = PaneManager::new();
        let left = manager.active_pane_id;
        manager.split_pane(Direction::Horizontal);
        let right = manager.active_pane_id;
        (manager, left, right)
    }

    #[test]
    fn test_resizing_moves_in_even_steps_and_stops_at_the_minimum() {
        let (mut manager, _, right) = side_by_side();
        let mut widths = vec![rect_of(&manager, right).width];
        while manager.resize_active(Direction::Horizontal, true) {
            widths.push(rect_of(&manager, right).width);
        }

        assert_eq!(widths[..4], [50, 55, 60, 65]);
        assert_eq!(*widths.last().unwrap(), 95);
        assert!(widths.windows(2).all(|w| w[1] - w[0] == 5), "{widths:?}");

        for _ in 0..3 {
            manager.resize_active(Direction::Horizontal, false);
        }
        assert_eq!(rect_of(&manager, right).width, 80);
    }

    #[test]
    fn test_old_integer_weights_still_resize_smoothly() {
        let (mut manager, left, right) = side_by_side();
        manager.nodes[left].weight = 1;
        manager.nodes[right].weight = 3;

        assert_eq!(rect_of(&manager, right).width, 75);
        manager.resize_active(Direction::Horizontal, true);
        assert_eq!(rect_of(&manager, right).width, 80);
    }

    #[test]
    fn test_shift_arrows_move_the_nearest_border() {
        let (mut manager, left, right) = side_by_side();

        manager.move_border(CardinalDirection::Left);
        assert_eq!(rect_of(&manager, right).width, 55);

        manager.move_border(CardinalDirection::Right);
        manager.move_border(CardinalDirection::Right);
        assert_eq!(rect_of(&manager, right).width, 45);

        manager.focus_pane(left);
        manager.move_border(CardinalDirection::Right);
        assert_eq!(rect_of(&manager, left).width, 60);
        assert!(!manager.move_border(CardinalDirection::Up));
    }

    #[test]
    fn test_border_moves_find_the_split_that_owns_the_border() {
        let (mut manager, left, right) = side_by_side();
        manager.split_pane(Direction::Vertical);
        let bottom_right = manager.active_pane_id;

        manager.move_border(CardinalDirection::Left);
        assert_eq!(rect_of(&manager, left).width, 45);
        assert_eq!(rect_of(&manager, right).width, 55);
        assert_eq!(rect_of(&manager, bottom_right).width, 55);

        manager.move_border(CardinalDirection::Up);
        assert_eq!(rect_of(&manager, bottom_right).height, 22);
    }

    #[test]
    fn test_equalize_resets_every_split() {
        let (mut manager, _, right) = side_by_side();
        manager.resize_active(Direction::Horizontal, true);
        manager.resize_active(Direction::Horizontal, true);
        manager.equalize();
        assert_eq!(rect_of(&manager, right).width, 50);
    }

    #[test]
    fn test_focus_moves_to_adjacent_pane_and_back() {
        let (mut manager, left, top_right) = side_by_side();
        manager.split_pane(Direction::Vertical);
        let bottom_right = manager.active_pane_id;
        let rects = manager.layout(AREA, false);

        assert!(manager.focus(CardinalDirection::Left, &rects));
        assert_eq!(manager.active_pane_id, left);
        assert!(manager.focus(CardinalDirection::Right, &rects));
        assert_eq!(manager.active_pane_id, bottom_right);

        assert!(manager.focus(CardinalDirection::Up, &rects));
        assert_eq!(manager.active_pane_id, top_right);
        assert!(!manager.focus(CardinalDirection::Up, &rects));
    }

    #[test]
    fn test_closing_a_pane_gives_space_and_focus_to_its_neighbour() {
        let mut manager = PaneManager::new();
        let first = manager.active_pane_id;
        manager.split_pane(Direction::Horizontal);
        let second = manager.active_pane_id;
        manager.split_pane(Direction::Vertical);
        let third = manager.active_pane_id;
        for _ in 0..2 {
            manager.focus_pane(first);
            manager.resize_active(Direction::Horizontal, true);
        }
        assert_eq!(rect_of(&manager, first).width, 60);

        manager.focus_pane(third);
        assert!(manager.kill_pane());
        assert_eq!(manager.active_pane_id, second);
        assert_eq!(rect_of(&manager, second).width, 40);
        assert_eq!(rect_of(&manager, second).height, 40);
        assert!(!manager.pane_key_to_friendly_id.contains_key(&third));

        assert!(manager.kill_pane());
        assert_eq!(manager.active_pane_id, first);
        assert_eq!(rect_of(&manager, first), AREA);
        assert!(!manager.kill_pane());
    }
}
