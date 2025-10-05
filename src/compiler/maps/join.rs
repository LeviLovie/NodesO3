use std::collections::HashMap;

#[derive(Clone)]
pub struct JoinMap {
    map: HashMap<usize, u32>,
}

impl JoinMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, node_id: usize, count: u32) {
        self.map.insert(node_id, count);
    }

    pub fn get(&self, node_id: usize) -> Option<&u32> {
        self.map.get(&node_id)
    }
}
