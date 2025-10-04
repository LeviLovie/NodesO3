use serde::{Deserialize, Serialize};

use super::{Type, Var};

#[derive(Clone, Serialize, Deserialize)]
pub enum FieldKind {
    Enter,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FieldDesc {
    name: String,
    data_type: Type,
    value: Var,
    kind: FieldKind,
}
