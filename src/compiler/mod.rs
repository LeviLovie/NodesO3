mod compilation;
mod iomap;
mod node_map;
mod traversal;
mod type_map;
mod writer;

pub use compilation::Compilation;
pub use iomap::IOMap;
pub use node_map::NodeMap;
pub use traversal::UpstreamTraversal;
pub use type_map::TypeMap;
pub use writer::write;

use anyhow::{anyhow, Context, Result};
use tracing::{debug, info};

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

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Raw { .. } => write!(f, "Raw"),
            Stage::Maps { .. } => write!(f, "Maps"),
            Stage::Traversal { .. } => write!(f, "Traversal"),
            Stage::Finished(_) => write!(f, "Finished"),
        }
    }
}

pub struct Compiler {
    final_node: usize,
    debug_info: bool,
    stage: Stage,
    compilation: Compilation,
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
            compilation: Compilation::new(),
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

    #[tracing::instrument(skip_all)]
    pub fn compile(&mut self) -> Result<Compilation> {
        debug!("Starting compilation");

        loop {
            debug!(stage = ?self.stage.to_string(), "Execution compile stage");
            let start = std::time::Instant::now();

            self.step().context("Compilation step failed")?;

            let duration = start.elapsed();
            self.compilation
                .add_elapsed_time(&self.stage.to_string(), duration);

            if let Stage::Finished(r) = &self.stage {
                info!("Compilation finished");
                self.compilation.set_code(r.clone());
                return Ok(self.compilation.clone());
            }
        }
    }
}
