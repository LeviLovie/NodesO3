mod iomap;
mod node_map;
mod traversal;
mod writer;

pub use iomap::IOMap;
pub use node_map::NodeMap;
pub use traversal::UpstreamTraversal;
pub use writer::write;

use anyhow::{anyhow, Context, Result};

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
        io_map: IOMap,
        traversal: UpstreamTraversal,
    },
    Finished(String),
}

pub struct Compiler {
    final_node: usize,
    debug_info: bool,
    stage: Stage,
}

impl Compiler {
    pub fn new(
        debug_info: bool,
        nodes: Vec<Node>,
        conns: Vec<Connection>,
        final_node: usize,
    ) -> Self {
        let stage = Stage::Raw {
            nodes: nodes.clone(),
            conns: conns.clone(),
        };
        Self {
            final_node,
            debug_info,
            stage,
        }
    }

    pub fn step(&mut self) -> Result<()> {
        let result = match &self.stage {
            Stage::Raw { nodes, conns } => {
                let node_map = NodeMap::new(nodes);
                let io_map = IOMap::new(conns);

                Ok(Stage::Maps { node_map, io_map })
            }
            Stage::Maps { node_map, io_map } => {
                let mut traversal = UpstreamTraversal::new();
                traversal.traverse(self.final_node, node_map, io_map);

                Ok(Stage::Traversal {
                    node_map: node_map.clone(),
                    io_map: io_map.clone(),
                    traversal,
                })
            }
            Stage::Traversal {
                traversal,
                node_map,
                io_map,
            } => {
                let output = write(
                    self.debug_info,
                    node_map.clone(),
                    io_map.clone(),
                    traversal.clone(),
                )
                .context("Failed to write output")?;
                Ok(Stage::Finished(output))
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
        let mut output = String::new();

        loop {
            match self.step() {
                Ok(_) => {}
                Err(_) => break,
            }

            if let Stage::Finished(r) = &self.stage {
                output = r.clone();
                break;
            }
        }

        std::fs::write("output.py", output)
            .context("Failed to write output.py")
            .unwrap();
        println!("Wrote output.py");
    }
}
