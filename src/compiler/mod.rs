mod compilation;
mod maps;
mod writer;

pub use compilation::Compilation;
pub use maps::*;
// pub use writer::write;

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
        exec_map: ExecMap,
        join_map: JoinMap,
        types_map: TypesMap<String>,
    },
    ExecTraversal {
        exec_map: ExecMap,
        join_map: JoinMap,
        io_map: IOMap,
        node_map: NodeMap,
    },
    DepResolution {
        io_map: IOMap,
        node_map: NodeMap,
    },
    CodeGen {
        node_map: NodeMap,
    },
    Finished(String),
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Raw { .. } => write!(f, "Raw"),
            Stage::Maps { .. } => write!(f, "Maps"),
            Stage::ExecTraversal { .. } => write!(f, "ExecTraversal"),
            Stage::DepResolution { .. } => write!(f, "DepResolution"),
            Stage::CodeGen { .. } => write!(f, "CodeGen"),
            Stage::Finished(_) => write!(f, "Finished"),
        }
    }
}

pub struct Compiler {
    debug_info: bool,
    stage: Stage,
    compilation: Compilation,
}

impl Compiler {
    pub fn new(debug_info: bool, nodes: Vec<Node>, conns: Vec<Connection>) -> Self {
        let stage = Stage::Raw {
            nodes: nodes.clone(),
            conns: conns.clone(),
        };

        Self {
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
            Stage::Maps { .. } => Err(anyhow!("Not implemented")),
            Stage::ExecTraversal { .. } => Err(anyhow!("Not implemented")),
            Stage::DepResolution { .. } => Err(anyhow!("Not implemented")),
            Stage::CodeGen { .. } => Err(anyhow!("Not implemented")),
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
