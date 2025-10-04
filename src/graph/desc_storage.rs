use anyhow::{Context, Result};

use super::NodeDesc;

pub struct DescStorage {
    pub descs: Vec<NodeDesc>,
}

impl DescStorage {
    pub fn from(yaml: String) -> Result<Self> {
        let descs: Vec<NodeDesc> = serde_yaml_ng::from_str(&yaml)
            .context("Failed to parse node descriptions from YAML")?;
        Ok(Self { descs })
    }
}
