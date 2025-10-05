use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortVariant {
    Execution,
    Simple,
}

impl PortVariant {
    pub fn color(&self) -> Color32 {
        match self {
            PortVariant::Execution => Color32::from_hex("#D0D0D0").unwrap(),
            PortVariant::Simple => Color32::from_hex("#5A9690").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDesc {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: Type,
    pub default: Option<Var>,
    pub variant: PortVariant,
}
