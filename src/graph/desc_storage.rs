use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::NodeDesc;

#[derive(Clone, Serialize, Deserialize)]
pub struct DescLib {
    pub category: String,
    pub lib: String,
    #[serde(rename = "nodes")]
    pub descs: Vec<NodeDesc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DescStorage {
    pub libs: Vec<DescLib>,
}

impl DescStorage {
    pub fn new() -> Self {
        Self { libs: Vec::new() }
    }

    pub fn categories(&self) -> Vec<String> {
        self.libs.iter().map(|lib| lib.category.clone()).collect()
    }

    pub fn descs_category(&self, category: &str) -> Option<&Vec<NodeDesc>> {
        self.libs
            .iter()
            .find(|lib| lib.category == category)
            .map(|lib| &lib.descs)
    }

    pub fn desc(&self, category: &str, title: &str) -> Option<&NodeDesc> {
        self.libs
            .iter()
            .find(|lib| lib.category == category)
            .and_then(|lib| lib.descs.iter().find(|desc| desc.title == title))
    }

    pub fn desc_count(&self) -> usize {
        self.libs.iter().map(|lib| lib.descs.len()).sum()
    }

    pub fn lib_exists(&self, category: &str) -> bool {
        self.libs.iter().any(|lib| lib.category == category)
    }

    #[tracing::instrument(skip_all)]
    pub fn import(&mut self, yaml_path: PathBuf, upgrade: bool) -> Result<()> {
        debug!(?yaml_path, "Importing desc lib");
        let yaml =
            std::fs::read_to_string(&yaml_path).context("Failed to read desc lib YAML file")?;
        let lib: DescLib =
            serde_yaml_ng::from_str(&yaml).context("Failed to parse desc lib from YAML")?;
        debug!(category=%lib.category, lib=%lib.lib, desc_count=%lib.descs.len(), "Parsed desc lib");

        if self.lib_exists(&lib.category) {
            if !upgrade {
                warn!(
                    "A desc lib with category '{}' already exists, skipping import",
                    lib.category
                );
                bail!("A desc lib with category '{}' already exists", lib.category);
            } else {
                warn!(
                    "A desc lib with category '{}' already exists, upgrading",
                    lib.category
                );
                let original_count = self.desc_count();
                self.libs.retain(|l| l.category != lib.category);
                let removed_count = original_count - self.desc_count();
                debug!(removed_count, "Removed descs from existing lib");
            }
        }

        self.libs.push(lib.clone());
        info!(
            category=%lib.category,
            lib=%lib.lib,
            desc_count=%lib.descs.len(),
            "Imported desc lib successfully"
        );

        Ok(())
    }
}
