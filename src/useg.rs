use std::borrow::Borrow;
use std::ffi::OsStr;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UPath {
    #[serde(with = "serde_bytes")]
    buffer: Box<[u8]>,
    #[serde(with = "serde_bytes")]
    splits: Box<[u8]>,
}

impl UPath {
    pub fn from_path(path: &Path) -> Self {
        let mut buf = Vec::new();
        let mut splits = Vec::new();
        for component in path.components() {
            buf.extend_from_slice(normalize_osstr(component.as_os_str()));
            splits.push(buf.len() as u8);
        }

        let buffer = buf.into_boxed_slice();
        let splits = splits.into_boxed_slice();
        Self { buffer, splits }
    }

    pub fn parent(&self) -> Self {
        let mut new_splits = Vec::new();
        for i in 0..self.splits.len() - 1 {
            new_splits.push(self.splits[i]);
        }

        let cut = new_splits.last().unwrap();
        let buffer = &self.buffer[..*cut as usize];
        let buffer = buffer.to_vec().into_boxed_slice();
        let new_splits = new_splits.into_boxed_slice();
        Self {
            buffer,
            splits: new_splits,
        }
    }

    pub fn last_segment(&self) -> &[u8] {
        let last = self.splits.last().unwrap();
        let start = self.splits.get(self.splits.len() - 2).unwrap_or(&0);
        &self.buffer[*start as usize..*last as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct USeg {
    #[serde(with = "serde_bytes")]
    raw: Box<[u8]>,
}

impl USeg {
    pub fn from_segment_bytes(bytes: &[u8]) -> Self {
        let raw = bytes.to_vec().into_boxed_slice();
        Self { raw }
    }
}

impl Borrow<[u8]> for USeg {
    fn borrow(&self) -> &[u8] {
        &self.raw
    }
}

fn normalize_osstr(s: &OsStr) -> &[u8] {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::ffi::OsStrExt;
        s.as_bytes()
    }

    #[cfg(not(target_family = "unix"))]
    s.to_str().expect("found bad byte in path").as_bytes()
}
