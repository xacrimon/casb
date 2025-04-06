use std::mem;

use serde::{Deserialize, Serialize};

use crate::repo::{BlobKind, IndexBlobInfo, IndexPackInfo, Key, PackInfo, PackInfoEntry};

const PACK_SIZE_MIN: usize = 4 * 1024 * 1024;
const PACK_SIZE_TARGET: usize = 8 * 1024 * 1024;
const PACK_SIZE_MAX: usize = 16 * 1024 * 1024;

pub struct Packer {
    entries: Vec<PackInfoEntry>,
    buffer: Vec<u8>,
    size: usize,
}

impl Packer {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            buffer: Vec::new(),
            size: 0,
        }
    }

    pub fn should_pack(&self) -> bool {
        self.size >= PACK_SIZE_TARGET
    }

    pub fn free_space(&self) -> usize {
        PACK_SIZE_MAX - self.size
    }

    pub fn add_blob(&mut self, entry: PackInfoEntry, data: &[u8]) {
        if self.size + data.len() > PACK_SIZE_MAX {
            panic!();
        }

        self.entries.push(entry);
        self.size += data.len();
        self.buffer.extend_from_slice(data);
    }

    pub fn finish(&mut self, key: &Key) -> (IndexPackInfo, Box<[u8]>) {
        let mut cursor = 0;
        let mut ies = Vec::new();

        for blob in &self.entries {
            let ib = IndexBlobInfo {
                id: blob.id,
                kind: blob.kind,
                offset: cursor,
                length: blob.size_compressed,
                length_uncompressed: blob.size_uncompressed,
            };

            cursor += blob.size_compressed;
            ies.push(ib)
        }

        let info = PackInfo {
            blobs: mem::take(&mut self.entries),
        };

        let header = serde_cbor::to_vec(&info).unwrap();
        let mac = blake3::keyed_hash(&key.mac, &header);
        let header_len = header.len() + blake3::OUT_LEN;

        self.buffer.extend_from_slice(&header);
        self.buffer.extend_from_slice(mac.as_bytes());
        self.buffer
            .extend_from_slice(&(header_len as u32).to_le_bytes());

        let data = mem::take(&mut self.buffer).into_boxed_slice();
        let id = blake3::hash(&self.buffer);

        let index = IndexPackInfo { id, blobs: ies };

        self.size = 0;
        (index, data)
    }
}
