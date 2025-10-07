use std::collections::HashMap;

use super::{IOMap, NodeMap};
use crate::graph::NodeExecKind;

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
    starting_nodes: Vec<usize>,
}

impl ExecMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            starting_nodes: vec![],
        }
    }

    pub fn set(&mut self, edge: ExecEdge) {
        self.map.insert(edge.out_node, edge);
    }

    pub fn get(&self, out_node: usize) -> Option<&ExecEdge> {
        self.map.get(&out_node)
    }

    pub fn values(&self) -> impl Iterator<Item = &ExecEdge> {
        self.map.values()
    }

    pub fn starting_nodes(&self) -> &Vec<usize> {
        &self.starting_nodes
    }

    pub fn print(&self) {
        println!("ExecMap:");
        for edge in self.map.values() {
            println!(
                "  Node {} (port {}) -> Node {} (port {})",
                edge.in_node, edge.in_port, edge.out_node, edge.out_port
            );
        }
        println!("Starting nodes: {:?}", self.starting_nodes);
    }

    pub fn fill_from_nodes(node_map: &NodeMap, exec_io_map: &IOMap) -> Self {
        let mut exec_map = Self::new();

        for (&(from_node, from_port), &(to_node, to_port)) in exec_io_map.io.iter() {
            if node_map.map.contains_key(&from_node) && node_map.map.contains_key(&to_node) {
                let edge = ExecEdge {
                    in_node: from_node,
                    in_port: from_port,
                    out_node: to_node,
                    out_port: to_port,
                };
                exec_map.set(edge);
            }
        }

        for node in node_map.values() {
            if matches!(node.desc.exec_kind, NodeExecKind::OutCondition(_)) {
                exec_map.starting_nodes.push(node.id);
            }
        }

        exec_map
    }
}
