use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::NodeDesc;

#[derive(Clone, Serialize, Deserialize)]
pub struct DescStorage {
    pub descs: Vec<NodeDesc>,
    pub categories: Vec<String>,
}

impl DescStorage {
    pub fn new() -> Self {
        Self {
            descs: Vec::new(),
            categories: Vec::new(),
        }
    }

    pub fn load(&mut self, yaml: String) -> Result<()> {
        let descs: Vec<NodeDesc> = serde_yaml_ng::from_str(&yaml)
            .context("Failed to parse node descriptions from YAML")?;
        self.descs.extend(descs.clone());

        let categories: Vec<String> = descs
            .iter()
            .map(|desc| desc.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        self.categories.extend(categories);
        self.categories.dedup();
        self.categories.sort();

        Ok(())
    }
}
