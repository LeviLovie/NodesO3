use serde::{Deserialize, Serialize};

use super::PortVariant;

#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
    pub variant: PortVariant,
    pub from: (usize, usize),
    pub to: (usize, usize),
}
