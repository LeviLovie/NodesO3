use std::collections::HashMap;

use super::ExecMap;

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

    pub fn fill_from_exec_map(exec_map: &ExecMap) -> Self {
        let mut join_map = Self::new();

        for edge in exec_map.values() {
            let counter = join_map.map.entry(edge.out_node).or_insert(0);
            *counter += 1;
        }

        join_map
    }
}
