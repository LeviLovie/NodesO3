use std::collections::HashSet;

use super::super::{ExecMap, IOMap, JoinMap, NodeMap, TypesMap};

#[derive(Debug, Clone)]
pub struct ExecPath {
    pub path_node_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ExecTraversal {
    pub paths: Vec<ExecPath>,
    pub current_path: Vec<usize>,
    pub visited: HashSet<usize>,
}

impl ExecTraversal {
    pub fn new() -> Self {
        Self {
            paths: vec![],
            current_path: vec![],
            visited: HashSet::new(),
        }
    }

    pub fn search(
        &mut self,
        node_id: usize,
        node_map: &NodeMap,
        io_map: &IOMap,
        exec_map: &ExecMap,
        join_map: &JoinMap,
        types_map: &TypesMap,
    ) {
        println!("Searching from node {}", node_id);
        if self.visited.contains(&node_id) {
            println!("Already visited node {}, skipping to avoid cycle", node_id);
            return;
        }

        self.visited.insert(node_id);
        self.current_path.push(node_id);

        println!("At node {}", node_id);

        if let Some(edge) = exec_map.get(node_id) {
            println!(
                "Traversing from node {} to node {}",
                edge.out_node, edge.in_node
            );
            self.search(
                edge.in_node,
                node_map,
                io_map,
                exec_map,
                join_map,
                types_map,
            );
        } else {
            println!("Reached end of path at node {}", node_id);
            self.paths.push(ExecPath {
                path_node_ids: self.current_path.clone(),
            });
        }

        self.current_path.pop();
        self.visited.remove(&node_id);
    }

    pub fn print(&self) {
        for (i, path) in self.paths.iter().enumerate() {
            println!("Path {}: {:?}", i, path.path_node_ids);
        }
    }

    pub fn get_paths(&self) -> &[ExecPath] {
        &self.paths
    }

    pub fn execution_order(&self) -> Vec<usize> {
        let mut order = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for path in &self.paths {
            for &node_id in &path.path_node_ids {
                if seen.insert(node_id) {
                    order.push(node_id);
                }
            }
        }

        order
    }
}
