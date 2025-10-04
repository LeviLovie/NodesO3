use eframe::egui::{Pos2, Vec2};
use serde::{Deserialize, Serialize};

use crate::graph::FieldDesc;

use super::PortDesc;

pub struct Node {
    pub id: usize,
    pub pos: Pos2,
    pub size: Vec2,
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
