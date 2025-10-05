use std::collections::HashMap;

#[derive(Clone)]
pub struct ExecEdge {
    pub in_node: usize,
    pub in_port: usize,
    pub out_node: usize,
    pub out_port: usize,
}

#[derive(Clone)]
pub struct ExecMap {
    map: HashMap<usize, ExecEdge>,
}

impl ExecMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, edge: ExecEdge) {
        self.map.insert(edge.in_node, edge);
    }

    pub fn get(&self, in_node: usize) -> Option<&ExecEdge> {
        self.map.get(&in_node)
    }
}
