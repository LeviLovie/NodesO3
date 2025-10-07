use std::collections::HashMap;

use super::{super::IOMap, ExecTraversal};

#[derive(Clone, Debug)]
pub struct Deps {
    map: HashMap<usize, Vec<usize>>,
}

impl Deps {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn build(&mut self, exec: &ExecTraversal, io_map: &IOMap) {
        for path in &exec.paths {
            for &node_id in path.path_node_ids.iter() {
                let deps = io_map
                    .inputs(node_id)
                    .iter()
                    .map(|(_, (from_node, _from_port))| *from_node)
                    .collect::<Vec<_>>();
                self.map.insert(node_id, deps);
            }
        }
    }

    pub fn dependents(&self, node_id: usize) -> Option<&Vec<usize>> {
        self.map.get(&node_id)
    }
}
