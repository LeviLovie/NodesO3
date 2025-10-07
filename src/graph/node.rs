use eframe::egui::Pos2;
use serde::{Deserialize, Serialize};

use super::PortDesc;
use crate::graph::{FieldDesc, PortVariant, Type};

const SIMPLE_OFFSET: f32 = 44.0;
const EXECUTION_OFFSET: f32 = 16.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeExecKind {
    In,
    OutCondition(String),
    InOut,
    InOutChoose((usize, usize)),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortLocation {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub desc: NodeDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeImpl {
    pub lang: String,
    pub required: Option<Vec<String>>,
    pub type_check: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDesc {
    pub title: String,
    pub exec_kind: NodeExecKind,
    pub desc: String,
    pub fields: Vec<FieldDesc>,
    pub inputs: Vec<PortDesc>,
    pub outputs: Vec<PortDesc>,
    pub impls: Vec<NodeImpl>,
}

impl Node {
    pub fn port_pos(&self, port_index: usize, output: bool, offset: f32) -> Pos2 {
        let y = self.pos.1 + offset + port_index as f32 * 22.0;
        let x = if output {
            self.pos.0 + self.size.0 + 14.0
        } else {
            self.pos.0
        };
        Pos2 { x, y }
    }

    pub fn ports(&self) -> Vec<(PortDesc, Pos2, PortLocation)> {
        let (exec_ins, exec_outs) = match self.desc.exec_kind {
            NodeExecKind::In => (1, 0),
            NodeExecKind::OutCondition(_) => (0, 1),
            NodeExecKind::InOut => (1, 1),
            NodeExecKind::InOutChoose((ins, outs)) => (ins, outs),
        };

        let mut ports = Vec::new();

        let mut simple_y_offset = 0;

        for i in 0..exec_ins {
            ports.push((
                PortDesc {
                    name: format!("exec_in_{}", i),
                    data_type: Type::Int,
                    default: None,
                    variant: PortVariant::Execution,
                },
                self.port_pos(i, false, EXECUTION_OFFSET),
                PortLocation::Input,
            ));
            simple_y_offset = simple_y_offset.max(i + 1);
        }

        for i in 0..exec_outs {
            ports.push((
                PortDesc {
                    name: format!("exec_out_{}", i),
                    data_type: Type::Int,
                    default: None,
                    variant: PortVariant::Execution,
                },
                self.port_pos(i, true, EXECUTION_OFFSET),
                PortLocation::Output,
            ));
            simple_y_offset = simple_y_offset.max(i + 1);
        }

        for (i, output) in self.desc.outputs.iter().enumerate() {
            ports.push((
                output.clone(),
                self.port_pos(i + simple_y_offset, true, SIMPLE_OFFSET),
                PortLocation::Output,
            ));
            simple_y_offset += 1;
        }

        for (i, input) in self.desc.inputs.iter().enumerate() {
            ports.push((
                input.clone(),
                self.port_pos(i + simple_y_offset, false, SIMPLE_OFFSET),
                PortLocation::Input,
            ));
        }

        ports
    }

    pub fn impl_for_lang(&self, lang: &str) -> Option<&NodeImpl> {
        self.desc.impls.iter().find(|ni| ni.lang == lang)
    }
}
