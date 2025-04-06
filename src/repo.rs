use std::collections::BTreeSet;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::upath::UPath;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackInfo {
    pub blobs: Vec<PackInfoEntry>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PackInfoEntry {
    #[serde(rename = "i")]
    pub id: blake3::Hash,
    #[serde(rename = "k")]
    pub kind: BlobKind,
    #[serde(rename = "c")]
    pub size_compressed: usize,
    #[serde(rename = "u", skip_serializing_if = "Option::is_none")]
    pub size_uncompressed: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "i32", from = "i32")]
#[repr(i32)]
pub enum BlobKind {
    Tree = 1,
    Data = 2,
}

impl From<BlobKind> for i32 {
    fn from(value: BlobKind) -> Self {
        value as i32
    }
}

impl From<i32> for BlobKind {
    fn from(value: i32) -> Self {
        match value {
            v if v == BlobKind::Tree as i32 => BlobKind::Tree,
            v if v == BlobKind::Data as i32 => BlobKind::Data,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub nodes: BTreeSet<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: UPath,
    pub mode: u32,
    pub mtime: i64,
    pub atime: i64,
    pub ctime: i64,
    pub uid: u32,
    pub gid: u32,
    pub user: String,
    pub inode: u64,
    #[serde(flatten)]
    pub kind: NodeKind,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeKind {
    File { content: Vec<blake3::Hash> },
    Dir { subtree: blake3::Hash },
    Symlink { link_target: UPath, links: u64 },
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum UnpackedEncoding {
    V1 = 1,
}

impl From<UnpackedEncoding> for i32 {
    fn from(value: UnpackedEncoding) -> Self {
        value as i32
    }
}

impl From<i32> for UnpackedEncoding {
    fn from(value: i32) -> Self {
        match value {
            1 => UnpackedEncoding::V1,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub supersedes: Vec<blake3::Hash>,
    pub packs: Vec<IndexPackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPackInfo {
    pub id: blake3::Hash,
    pub blobs: Vec<IndexBlobInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexBlobInfo {
    pub id: blake3::Hash,
    pub kind: BlobKind,
    pub offset: usize,
    pub length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length_uncompressed: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub hostname: String,
    pub username: String,
    #[serde(flatten)]
    pub kdf: Kdf,
    pub created: i64,
    pub data: Vec<u8>,
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kdf", rename_all = "lowercase")]
pub enum Kdf {
    Scrypt {
        #[serde(rename = "N")]
        n: i32,
        r: i32,
        p: i32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Key {
    pub mac: [u8; 32],
    pub encrypt: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub time: i64,
    pub tree: blake3::Hash,
    pub paths: Vec<UPath>,
    pub hostname: String,
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<blake3::Hash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: RepositoryVersion,
    pub id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i32)]
pub enum RepositoryVersion {
    V1 = 1,
}

impl From<RepositoryVersion> for i32 {
    fn from(value: RepositoryVersion) -> Self {
        value as i32
    }
}

impl From<i32> for RepositoryVersion {
    fn from(value: i32) -> Self {
        match value {
            1 => RepositoryVersion::V1,
            _ => panic!(),
        }
    }
}
