mod iomap;
mod node_map;
mod traversal;

pub use iomap::IOMap;
pub use node_map::NodeMap;
pub use traversal::UpstreamTraversal;

use anyhow::{anyhow, Result};

use crate::graph::{Connection, Node};

pub enum Stage {
    Raw {
        nodes: Vec<Node>,
        conns: Vec<Connection>,
    },
    Maps {
        node_map: NodeMap,
        io_map: IOMap,
    },
    Traversal {
        node_map: NodeMap,
        traversal: UpstreamTraversal,
    },
    Finished(String),
}

pub struct Compiler {
    final_node: usize,
    stage: Stage,
}

impl Compiler {
    pub fn new(nodes: Vec<Node>, conns: Vec<Connection>, final_node: usize) -> Self {
        let stage = Stage::Raw {
            nodes: nodes.clone(),
            conns: conns.clone(),
        };
        Self { final_node, stage }
    }

    pub fn step(&mut self) -> Result<()> {
        let result = match &self.stage {
            Stage::Raw { nodes, conns } => {
                println!("Building maps...");
                let node_map = NodeMap::new(nodes);
                let io_map = IOMap::new(conns);

                Ok(Stage::Maps { node_map, io_map })
            }
            Stage::Maps { node_map, io_map } => {
                println!("Performing upstream traversal...");
                let mut traversal = UpstreamTraversal::new();
                traversal.traverse(self.final_node, node_map, io_map);

                println!("Traversal order: {:?}", traversal.execution_order());
                Ok(Stage::Traversal {
                    node_map: node_map.clone(),
                    traversal,
                })
            }
            Stage::Traversal {
                traversal,
                node_map,
            } => {
                let mut dumped = String::new();
                for node_id in traversal.execution_order() {
                    if let Some(node) = node_map.get(*node_id) {
                        dumped.push_str(&format!("{:#?}\n", node));
                    } else {
                        return Err(anyhow!("Node ID {} not found during compilation", node_id));
                    }
                }

                Ok(Stage::Finished(dumped))
            }
            Stage::Finished(_) => Err(anyhow!("Already finished")),
        };

        match result {
            Ok(stage) => {
                self.stage = stage;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn compile(&mut self) {
        println!("Starting compilation...");
        loop {
            match self.step() {
                Ok(_) => {}
                Err(_) => break,
            }

            if let Stage::Finished(r) = &self.stage {
                println!("Compilation finished:\n{}", r);
                break;
            }
        }
    }
}
