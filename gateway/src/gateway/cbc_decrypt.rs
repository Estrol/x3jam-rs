use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
use des::Des;
use md5::{Digest, Md5};

type DesCbcDec = cbc::Decryptor<Des>;

pub struct DesKey {
    pub key: [u8; 8],
    pub iv: [u8; 8],
}

pub struct Md5AndDes;

impl Md5AndDes {
    pub fn derive_key(password: &[u8], iteration_count: Option<u32>) -> Option<DesKey> {
        if password.len() != 16 {
            return None;
        }

        let iterations = iteration_count.unwrap_or(16);
        let mut current_hash: [u8; 16] = password.try_into().unwrap();

        for _ in 0..iterations {
            let mut hasher = Md5::new();
            hasher.update(&current_hash);
            current_hash = hasher.finalize().into();
        }

        let mut iv = [0u8; 8];
        let mut key = [0u8; 8];

        iv.copy_from_slice(&current_hash[0..8]);
        key.copy_from_slice(&current_hash[8..16]);

        Some(DesKey { key, iv })
    }

    pub fn decrypt_with_key(
        input: &[u8],
        key: &DesKey,
    ) -> Result<Vec<u8>, cbc::cipher::inout::PadError> {
        Self::decrypt(input, &key.key, &key.iv)
    }

    pub fn decrypt(
        input: &[u8],
        key: &[u8; 8],
        iv: &[u8; 8],
    ) -> Result<Vec<u8>, cbc::cipher::inout::PadError> {
        let decryptor = DesCbcDec::new(key.into(), iv.into());

        // Decrypt the input without padding.
        // NOTE: Because padding is NoPadding, input.len() MUST be a multiple of 8,
        // otherwise this method will return an error.
        let mut output = input.to_vec();
        let _ = decryptor.decrypt_padded_mut::<NoPadding>(&mut output);
        Ok(output)
    }
}
