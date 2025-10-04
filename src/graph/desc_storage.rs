use anyhow::{Context, Result};

use super::NodeDesc;

pub struct DescStorage {
    pub descs: Vec<NodeDesc>,
    pub categories: Vec<String>,
}

impl DescStorage {
    pub fn from(yaml: String) -> Result<Self> {
        let descs: Vec<NodeDesc> = serde_yaml_ng::from_str(&yaml)
            .context("Failed to parse node descriptions from YAML")?;
        let mut categories: Vec<String> = descs
            .iter()
            .map(|desc| desc.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();

        Ok(Self { descs, categories })
    }
}
