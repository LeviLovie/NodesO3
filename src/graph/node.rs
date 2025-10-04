use iced::{Point, Size};
use serde::{Deserialize, Serialize};

use super::PortDesc;

pub struct Node {
    pub id: usize,
    pub pos: Point,
    pub size: Size,
    pub desc: NodeDesc,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeDesc {
    pub title: String,
    pub category: String,
    pub inputs: Vec<PortDesc>,
    pub outputs: Vec<PortDesc>,
    pub py_impl: String,
}
