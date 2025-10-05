use std::collections::HashMap;

use crate::graph::Type;

#[derive(Clone, Debug)]
pub struct TypeMap<T>
where
    T: std::cmp::Eq + std::hash::Hash,
{
    map: HashMap<T, Type>,
}

impl<T> TypeMap<T>
where
    T: std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, from: T, ty: Type) {
        self.map.insert(from, ty);
    }

    pub fn print(&self) {
        println!("TypeMap:");
        for (k, v) in &self.map {
            println!("  {:?} => {:?}", k, v);
        }
    }

    pub fn get(&self, from: T) -> Option<&Type> {
        self.map.get(&from)
    }
}
