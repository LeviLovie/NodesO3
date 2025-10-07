use std::collections::HashMap;

use super::NodeMap;
use crate::graph::Type;

pub type TypesMapType = (usize, usize);

#[derive(Clone, Debug)]
pub struct TypesMap {
    map: HashMap<TypesMapType, Type>,
}

impl TypesMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, from: TypesMapType, ty: Type) {
        self.map.insert(from, ty);
    }

    pub fn print(&self) {
        println!("TypeMap:");
        for (k, v) in &self.map {
            println!("  {:?} => {:?}", k, v);
        }
    }

    pub fn get(&self, from: TypesMapType) -> Option<&Type> {
        self.map.get(&from)
    }

    pub fn fill_from_nodes(node_map: &NodeMap) -> Self {
        let mut types_map = Self::new();

        for node in node_map.values() {
            for (i, output) in node.desc.outputs.iter().enumerate() {
                types_map.set((node.id, i), output.data_type.clone());
            }
        }

        types_map
    }
}
