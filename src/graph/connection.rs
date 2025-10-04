use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: (usize, usize),
    pub to: (usize, usize),
}
