use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UPath {
    segments: Vec<USeg>,
}

impl UPath {
    pub fn segments(&self) -> &[USeg] {
        &self.segments
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct USeg {
    raw: Vec<u8>,
}
