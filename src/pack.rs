#![allow(dead_code)]

use std::io::Read;
use std::mem;
use std::num::NonZeroUsize;

use crate::fastcdc;
use crate::repo::{BlobKind, IndexBlobInfo, IndexPackInfo, Key, PackInfo, PackInfoEntry};

const CHUNK_MIN_SIZE: u32 = 512 * 1024;
const CHUNK_AVG_SIZE: u32 = 1024 * 1024;
const CHUNK_MAX_SIZE: u32 = 2 * 1024 * 1024;

const BLOB_COMPRESSION_THRESHOLD: usize = 100;

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

    #[allow(unused_variables)]
    pub fn finish(&mut self, key: &Key) -> (IndexPackInfo, Box<[u8]>) {
        let mut cursor = 0;
        let mut ies = Vec::new();

        for blob in &self.entries {
            let length = match blob.size_compressed {
                Some(size) => size.get(),
                None => blob.size_uncompressed,
            };

            let length_uncompressed = if blob.size_compressed.is_some() {
                let size_uncompressed = NonZeroUsize::new(blob.size_uncompressed).unwrap();
                Some(size_uncompressed)
            } else {
                None
            };

            let ib = IndexBlobInfo {
                id: blob.id,
                kind: blob.kind,
                offset: cursor,
                length,
                length_uncompressed,
            };

            cursor += length;
            ies.push(ib)
        }

        let info = PackInfo {
            blobs: mem::take(&mut self.entries),
        };

        let header = serde_cbor::to_vec(&info).unwrap();
        let header_len = (header.len() as u32).to_le_bytes();

        self.buffer.extend_from_slice(&header);
        self.buffer.extend_from_slice(&header_len);

        let data = mem::take(&mut self.buffer).into_boxed_slice();
        let id = blake3::hash(&data).into();

        let index = IndexPackInfo { id, blobs: ies };

        self.size = 0;
        (index, data)
    }
}

pub fn split_to_data_blobs(
    data: &mut dyn Read,
) -> impl Iterator<Item = (PackInfoEntry, Box<[u8]>)> {
    fastcdc::StreamCDC::new(data, CHUNK_MIN_SIZE, CHUNK_AVG_SIZE, CHUNK_MAX_SIZE).map(|chunk| {
        let chunk = chunk.unwrap();
        let id = blake3::hash(&chunk.data).into();

        let (kind, size_compressed, data) = if chunk.data.len() < BLOB_COMPRESSION_THRESHOLD {
            (BlobKind::Data, None, chunk.data)
        } else {
            let compressed =
                zstd::bulk::compress(&chunk.data, zstd::DEFAULT_COMPRESSION_LEVEL).unwrap();

            if compressed.len() < chunk.data.len() {
                (
                    BlobKind::DataZstd3,
                    Some(NonZeroUsize::new(compressed.len()).unwrap()),
                    compressed,
                )
            } else {
                (BlobKind::Data, None, chunk.data)
            }
        };

        let entry = PackInfoEntry {
            id,
            kind,
            size_uncompressed: data.len(),
            size_compressed,
        };

        (entry, data.into_boxed_slice())
    })
}
