use crate::event::router::NodeId;

/// Manages focus order and active focus target.
#[derive(Default)]
pub struct FocusManager {
    order: Vec<NodeId>,
    active: Option<usize>, // index into order
}

impl FocusManager {
    pub fn new() -> Self { Self { order: Vec::new(), active: None } }

    /// Add a focusable node to the end of the order if not already present.
    pub fn add(&mut self, id: NodeId) {
        if !self.order.contains(&id) { self.order.push(id); }
        if self.active.is_none() { self.active = Some(0); }
    }

    /// Remove a focusable node. Adjust active index if needed.
    pub fn remove(&mut self, id: NodeId) {
        if let Some(pos) = self.order.iter().position(|n| *n == id) {
            self.order.remove(pos);
            if let Some(ai) = self.active {
                if self.order.is_empty() { self.active = None; }
                else if pos <= ai { self.active = Some(ai.saturating_sub(1).min(self.order.len()-1)); }
            }
        }
    }

    /// Set the active node explicitly if present in order.
    pub fn set_active(&mut self, id: NodeId) {
        if let Some(pos) = self.order.iter().position(|n| *n == id) {
            self.active = Some(pos);
        }
    }

    /// Move focus to next node and return it.
    pub fn next(&mut self) -> Option<NodeId> {
        if self.order.is_empty() { return None; }
        let idx = match self.active { Some(i) => (i + 1) % self.order.len(), None => 0 };
        self.active = Some(idx);
        self.order.get(idx).cloned()
    }

    /// Move focus to previous node and return it.
    pub fn prev(&mut self) -> Option<NodeId> {
        if self.order.is_empty() { return None; }
        let idx = match self.active { Some(i) => if i==0 { self.order.len()-1 } else { i-1 }, None => 0 };
        self.active = Some(idx);
        self.order.get(idx).cloned()
    }

    /// Get the active focused node id
    pub fn active(&self) -> Option<NodeId> {
        self.active.and_then(|i| self.order.get(i).cloned())
    }

    /// Replace focus order wholesale
    pub fn set_order(&mut self, order: Vec<NodeId>) {
        self.order = order;
        self.active = if self.order.is_empty() { None } else { Some(0) };
    }

    pub fn clear(&mut self) { self.order.clear(); self.active = None; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traversal_wraps() {
        let mut fm = FocusManager::new();
        let ids = [NodeId::new(), NodeId::new(), NodeId::new()];
        for id in ids { fm.add(id); }
        assert_eq!(fm.active(), Some(ids[0]));
        assert_eq!(fm.next(), Some(ids[1]));
        assert_eq!(fm.next(), Some(ids[2]));
        assert_eq!(fm.next(), Some(ids[0]));
        assert_eq!(fm.prev(), Some(ids[2]));
    }
}

