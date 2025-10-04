use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDesc {
    name: String,
    #[serde(rename = "type")]
    data_type: Type,
    default: Var,
}
