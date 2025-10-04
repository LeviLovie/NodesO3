use eframe::egui::Pos2;
use serde::{Deserialize, Serialize};

use super::PortDesc;
use crate::graph::FieldDesc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub desc: NodeDesc,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeDesc {
    pub title: String,
    pub desc: String,
    pub category: String,
    pub fields: Vec<FieldDesc>,
    pub inputs: Vec<PortDesc>,
    pub outputs: Vec<PortDesc>,
    pub py_impl: String,
}

impl Node {
    pub fn port_pos(&self, port_index: usize, output: bool) -> Pos2 {
        let y = self.pos.1 + 8.0 + port_index as f32 * 20.0;
        let x = if output {
            self.pos.0 + self.size.0 + 2.0
        } else {
            self.pos.0 - 2.0
        };
        Pos2 { x, y }
    }
}
