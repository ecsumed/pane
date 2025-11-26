use ratatui::layout::Direction;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::logging::debug;
use crate::pane::Panes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaneManager {
    pub panes: Panes,
    active_pane_id: usize,
    pane_ids: Vec<usize>,
}

impl Default for PaneManager {
    fn default() -> Self {
        Self {
            panes: Panes::default(),
            active_pane_id: 1,
            pane_ids: vec![1],
        }
    }
}

impl PaneManager {
    pub fn cycle_panes(&mut self) {
        if self.pane_ids.is_empty() {
            return;
        }

        if let Some(pos) = self
            .pane_ids
            .iter()
            .position(|&p_id| p_id == self.active_pane_id)
        {
            let next_pos = (pos + 1) % self.pane_ids.len();
            self.active_pane_id = self.pane_ids[next_pos];
        } else {
            // Fallback to the first pane if the active pane is somehow missing.
            self.active_pane_id = self.pane_ids[0];
        }

        debug!("Total panes: {0}", self.pane_ids.len());
        debug!("Active pane id: {0}", self.active_pane_id);
        trace!("{:?}", self.panes);
        debug!("Pane ids: {:?}", self.pane_ids);
    }

    pub fn split_pane(&mut self, direction: Direction) -> bool {
        let new_id = self.generate_pane_id();
        let result = self
            .panes
            .split_pane(self.active_pane_id, direction, new_id);

        if result {
            self.pane_ids.push(new_id);
            self.pane_ids.sort();

            self.active_pane_id = new_id;
        } else {
            debug!("Split failed. Pane {} not found.", new_id);
        }

        debug!("Total panes: {0}", self.pane_ids.len());
        debug!("Active pane id: {0}", self.active_pane_id);
        debug!("{:?}", self.panes);
        debug!("Pane ids: {:?}", self.pane_ids);

        result
    }

    pub fn kill_pane(&mut self) -> bool {
        if self.pane_ids.len() == 1 {
            // TODO: quit here
            debug!("Cannot remove last pane.");
            return false;
        }

        let result = self.panes.kill_pane(self.active_pane_id);

        if result {
            self.pane_ids
                .retain(|&pane_id| pane_id != self.active_pane_id);

            self.active_pane_id = if let Some(&next_id) = self.pane_ids.get(0) {
                next_id
            } else {
                1 // Fallback to 1
            };
        }

        debug!("Total panes: {0}", self.pane_ids.len());
        debug!("Active pane id: {0}", self.active_pane_id);
        debug!("{:?}", self.panes);
        debug!("Pane ids: {:?}", self.pane_ids);

        result
    }

    fn generate_pane_id(&self) -> usize {
        let max_id = self.pane_ids.iter().max().unwrap_or(&0);
        max_id + 1
    }

    pub fn get_active_pane_id(&self) -> usize {
        self.active_pane_id
    }
}
