use std::collections::HashMap;

use crate::graph::Node;

#[derive(Clone)]
pub struct NodeMap {
    pub map: HashMap<usize, Node>,
}

impl NodeMap {
    pub fn new(nodes: &Vec<Node>) -> Self {
        let mut map = HashMap::new();
        for node in nodes {
            map.insert(node.id, node.clone());
        }
        Self { map }
    }

    pub fn get(&self, id: usize) -> Option<&Node> {
        self.map.get(&id)
    }

    pub fn values(&self) -> impl Iterator<Item = &Node> {
        self.map.values()
    }
}
