// src/components.rs: the game's own scene components, used by both binaries.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Door {
    pub leads_to: String,
}
