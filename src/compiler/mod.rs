mod compilation;
mod maps;
mod stages;

pub use compilation::Compilation;
pub use maps::*;
pub use stages::*;
// pub use writer::write;

use anyhow::{anyhow, Context, Result};
use tracing::{debug, info};

use crate::graph::{Connection, Node, PortVariant};

pub enum Stage {
    Maps {
        nodes: Vec<Node>,
        conns: Vec<Connection>,
    },
    ExecTraversal {
        node_map: NodeMap,
        io_map: IOMap,
        exec_map: ExecMap,
        join_map: JoinMap,
        types_map: TypesMap,
    },
    DepResolution {
        exec_traversal: ExecTraversal,
        node_map: NodeMap,
        io_map: IOMap,
    },
    CodeGen {
        exec_traversal: ExecTraversal,
        deps: Deps,
        node_map: NodeMap,
        io_map: IOMap,
    },
    Finished(String),
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
        let stage = Stage::Maps {
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
            Stage::Maps { nodes, conns } => {
                let node_map = NodeMap::new(nodes);
                let io_map = IOMap::new(conns, PortVariant::Simple);
                let exec_io_map = IOMap::new(conns, PortVariant::Execution);
                exec_io_map.print();
                let exec_map = ExecMap::fill_from_nodes(&node_map, &exec_io_map);
                exec_map.print();
                let join_map = JoinMap::fill_from_exec_map(&exec_map);
                let types_map = TypesMap::fill_from_nodes(&node_map);

                Ok(Stage::ExecTraversal {
                    node_map,
                    io_map,
                    exec_map,
                    join_map,
                    types_map,
                })
            }
            Stage::ExecTraversal {
                node_map,
                io_map,
                exec_map,
                join_map,
                types_map,
            } => {
                let mut exec_traversal = ExecTraversal::new();

                for node in exec_map.starting_nodes() {
                    exec_traversal.search(*node, node_map, io_map, exec_map, join_map, types_map);
                }

                exec_traversal.print();

                Ok(Stage::DepResolution {
                    exec_traversal,
                    io_map: io_map.clone(),
                    node_map: node_map.clone(),
                })
            }
            Stage::DepResolution {
                exec_traversal,
                io_map,
                node_map,
            } => {
                let mut deps = Deps::new();
                deps.build(exec_traversal, io_map);

                Ok(Stage::CodeGen {
                    exec_traversal: exec_traversal.clone(),
                    deps,
                    node_map: node_map.clone(),
                    io_map: io_map.clone(),
                })
            }
            Stage::CodeGen {
                exec_traversal,
                deps,
                node_map,
                io_map,
            } => {
                let codegen = CodeGen::new(
                    self.debug_info,
                    node_map.clone(),
                    io_map.clone(),
                    exec_traversal.clone(),
                    deps.clone(),
                );
                let code = codegen.generate()?;

                Ok(Stage::Finished(code))
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
