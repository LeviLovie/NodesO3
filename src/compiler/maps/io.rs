use std::collections::HashMap;

use crate::{graph::PortVariant, Connection};

#[derive(Clone)]
pub struct IOMap {
    pub io: HashMap<(usize, usize), (usize, usize)>,
}

impl IOMap {
    pub fn new(conns: &Vec<Connection>, variant: PortVariant) -> Self {
        let mut io = HashMap::new();

        for conn in conns {
            if conn.variant != variant {
                continue;
            }

            io.insert(conn.to, conn.from);
        }

        Self { io }
    }

    pub fn inputs(&self, node_id: usize) -> Vec<((usize, usize), (usize, usize))> {
        self.io
            .iter()
            .filter_map(|(&(to_node, to_port), &(from_node, from_port))| {
                if to_node == node_id {
                    Some(((to_node, to_port), (from_node, from_port)))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn print(&self) {
        println!("IOMap:");
        for (k, v) in &self.io {
            println!("  {:?} => {:?}", k, v);
        }
    }

    pub fn get(&self, from: (usize, usize)) -> Option<&(usize, usize)> {
        self.io.get(&from)
    }

    pub fn values(&self) -> Vec<&(usize, usize)> {
        self.io.values().collect()
    }
}
