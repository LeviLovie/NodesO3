use eframe::egui::Pos2;
use serde::{Deserialize, Serialize};

use super::PortDesc;
use crate::graph::FieldDesc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub desc: NodeDesc,
    #[serde(skip)]
    pub stabilize_frames: usize,
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
    pub end: bool,
    pub desc: String,
    pub fields: Vec<FieldDesc>,
    pub inputs: Vec<PortDesc>,
    pub outputs: Vec<PortDesc>,
    pub impls: Vec<NodeImpl>,
}

impl Node {
    pub fn port_pos(&self, port_index: usize, output: bool) -> Pos2 {
        let y = self.pos.1 + 35.0 + port_index as f32 * 22.0;
        let x = if output {
            self.pos.0 + self.size.0 + 14.0
        } else {
            self.pos.0
        };
        Pos2 { x, y }
    }

    pub fn impl_for_lang(&self, lang: &str) -> Option<&NodeImpl> {
        self.desc.impls.iter().find(|ni| ni.lang == lang)
    }
}
