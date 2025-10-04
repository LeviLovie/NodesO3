use std::collections::HashSet;

use crate::compiler::{IOMap, NodeMap};

const MAX_TRAVERSAL_ITER: usize = 1000;

pub struct UpstreamTraversal {
    visited: HashSet<usize>,
    traversal: Vec<usize>,
}

impl UpstreamTraversal {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            traversal: Vec::new(),
        }
    }

    fn traverse_recursive(
        &mut self,
        iter: usize,
        node_id: usize,
        node_map: &NodeMap,
        io_map: &IOMap,
    ) {
        if iter > MAX_TRAVERSAL_ITER {
            panic!("Max traversal iterations reached");
        }
        if self.visited.contains(&node_id) {
            return;
        }
        self.visited.insert(node_id);

        if let Some(node) = node_map.get(node_id) {
            for (port_id, _) in node.desc.inputs.iter().enumerate() {
                if let Some(&(from_node, _from_port)) = io_map.get((node_id, port_id)) {
                    self.traverse_recursive(iter + 1, from_node, node_map, io_map);
                }
            }
        } else {
            panic!("Node ID {} not found in NodeMap", node_id);
        }

        self.traversal.push(node_id);
    }

    pub fn traverse(&mut self, node_id: usize, node_map: &NodeMap, io_map: &IOMap) {
        self.traverse_recursive(0, node_id, node_map, io_map);
    }

    pub fn execution_order(&self) -> &Vec<usize> {
        &self.traversal
    }
}
