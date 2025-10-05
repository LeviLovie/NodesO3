use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDesc {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: Type,
    pub default: Option<Var>,
}
