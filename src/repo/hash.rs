use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash {
    #[serde(with = "serde_bytes")]
    pub bytes: [u8; 32],
}

impl Hash {
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }
}

impl From<blake3::Hash> for Hash {
    fn from(hash: blake3::Hash) -> Self {
        Hash {
            bytes: *hash.as_bytes(),
        }
    }
}
