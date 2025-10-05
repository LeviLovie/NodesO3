use std::collections::HashMap;

use crate::graph::Node;

#[derive(Clone)]
pub struct NodeMap {
    node_map: HashMap<usize, Node>,
}

impl NodeMap {
    pub fn new(nodes: &Vec<Node>) -> Self {
        let mut node_map = HashMap::new();
        for node in nodes {
            node_map.insert(node.id, node.clone());
        }
        Self { node_map }
    }

    pub fn get(&self, id: usize) -> Option<&Node> {
        self.node_map.get(&id)
    }
}
