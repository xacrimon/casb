use chacha20::ChaCha12;
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

use crate::repo::types::{
    BlobKind, Config, Index, IndexBlobInfo, IndexPackInfo, Kdf, Key, Node, NodeKind, PackInfo,
    PackInfoEntry, Recipe, RepositoryVersion, Snapshot, Tree, UnpackedEncoding,
};

const ENCRYPTION_CONTEXT: &str = "encryption";
const AUTHENTICATION_CONTEXT: &str = "authentication";

const ENCRYPTION_KEY_SIZE: usize = 32;
const AUTHENTICATION_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 32;
const CIPHER_NONCE_SIZE: usize = 12;
const MAC_SIZE: usize = 32;

fn derive_encryption_key(key: &Key) -> [u8; 32] {
    blake3::derive_key(ENCRYPTION_CONTEXT, &key.bytes)
}

fn derive_authentication_key(key: &Key, nonce: &[u8; NONCE_SIZE]) -> [u8; 32] {
    let mut auth_material = Vec::with_capacity(NONCE_SIZE + ENCRYPTION_KEY_SIZE);
    auth_material.extend_from_slice(nonce);
    auth_material.extend_from_slice(&key.bytes);
    blake3::derive_key(AUTHENTICATION_CONTEXT, &auth_material)
}

fn derive_cipher_nonce(nonce: &[u8; NONCE_SIZE]) -> [u8; 12] {
    let mut cipher_nonce = [0u8; 12];
    cipher_nonce.copy_from_slice(&nonce[..CIPHER_NONCE_SIZE]);
    cipher_nonce
}

pub fn seal_blob(plain: &[u8], key: &Key) -> Vec<u8> {
    let mut nonce = [0u8; 32];
    getrandom::fill(&mut nonce).unwrap();

    let e_key = derive_encryption_key(key);
    let a_key = derive_authentication_key(key, &nonce);
    let cipher_nonce = derive_cipher_nonce(&nonce);

    let mut cipher = ChaCha12::new(&e_key.into(), &cipher_nonce.into());

    let mut buf = Vec::with_capacity(NONCE_SIZE + plain.len() + MAC_SIZE);
    buf.extend_from_slice(&nonce);
    buf.extend_from_slice(plain);
    cipher.apply_keystream(buf.as_mut_slice());

    let mac = blake3::keyed_hash(&a_key, &buf);
    buf.extend_from_slice(mac.as_bytes());

    buf
}

pub fn unseal_blob(data: &[u8], key: &Key) -> Vec<u8> {
    let data_nonce = &data[..NONCE_SIZE];
    let data_ciphertext = &data[(data.len() - MAC_SIZE)..MAC_SIZE];
    let data_mac = blake3::Hash::from_slice(&data[(data.len() - MAC_SIZE)..]).unwrap();

    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(data_nonce);

    let e_key = derive_encryption_key(key);
    let a_key = derive_authentication_key(key, &nonce);
    let cipher_nonce = derive_cipher_nonce(&nonce);

    let mut cipher = ChaCha12::new(&e_key.into(), &cipher_nonce.into());

    let mac = blake3::keyed_hash(&a_key, &data);
    if mac != data_mac {
        panic!();
    }

    let mut buf = Vec::with_capacity(data.len() - NONCE_SIZE - MAC_SIZE);
    buf.extend_from_slice(data_ciphertext);
    cipher.apply_keystream(buf.as_mut_slice());

    buf
}
