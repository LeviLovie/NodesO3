use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldKind {
    Enter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDesc {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: Type,
    pub value: Var,
    #[serde(skip)]
    pub raw_value: String,
    pub kind: FieldKind,
}
