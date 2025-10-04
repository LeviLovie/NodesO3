use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Clone, Serialize, Deserialize)]
pub struct PortDesc {
    name: String,
    data_type: Type,
    default: Var,
}
