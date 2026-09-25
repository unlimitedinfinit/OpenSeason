use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce, Key
};
use rand::RngCore;
use std::sync::{Arc, Mutex};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKey(pub [u8; 32]);

pub struct AppState {
    pub key: Arc<Mutex<Option<SessionKey>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            key: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_key(&self, key: SessionKey) {
        let mut guard = self.key.lock().unwrap();
        *guard = Some(key);
    }

    pub fn clear_key(&self) {
        let mut guard = self.key.lock().unwrap();
        *guard = None; // ZeroizeOnDrop handles the cleanup of the old value
    }
    
    pub fn get_key(&self) -> Option<SessionKey> {
        let guard = self.key.lock().unwrap();
        guard.clone()
    }
}

pub fn generate_salt() -> String {
    let salt = SaltString::generate(&mut OsRng);
    salt.as_str().to_string()
}

pub fn derive_key(password: &str, salt_str: &str) -> Result<SessionKey, String> {
    // Parse the stored salt
    let salt = SaltString::from_b64(salt_str).map_err(|e| e.to_string())?;

    // Argon2id configuration (adjust params for security vs performance as needed)
    let argon2 = Argon2::default();

    // Hash password to get the derived key material
    let mut output_key_material = [0u8; 32];
    argon2.hash_password_into(
        password.as_bytes(),
        salt.as_str().as_bytes(),
        &mut output_key_material
    ).map_err(|e| e.to_string())?;

    Ok(SessionKey(output_key_material))
}

pub const NONCE_LEN: usize = 24;

/// Encrypt and prefix the 24-byte nonce so a file can be decrypted without a sidecar.
pub fn encrypt_blob(data: &[u8], key: &SessionKey) -> Result<Vec<u8>, String> {
    let (ciphertext, nonce) = encrypt_data(data, key)?;
    if nonce.len() != NONCE_LEN {
        return Err(format!("Unexpected nonce length {}", nonce.len()));
    }
    let mut out = nonce;
    out.extend(ciphertext);
    Ok(out)
}

pub fn decrypt_blob(blob: &[u8], key: &SessionKey) -> Result<Vec<u8>, String> {
    if blob.len() < NONCE_LEN + 1 {
        return Err("Encrypted file is too short to contain a nonce and ciphertext.".to_string());
    }
    decrypt_data(&blob[NONCE_LEN..], &blob[..NONCE_LEN], key)
}

pub fn encrypt_data(data: &[u8], key: &SessionKey) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key.0));
    
    // Generate random 192-bit (24-byte) nonce
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| e.to_string())?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce_bytes: &[u8], key: &SessionKey) -> Result<Vec<u8>, String> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key.0));
    let nonce = XNonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(plaintext)
}

pub fn strip_jpeg_metadata(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return data.to_vec();
    }

    let mut output = Vec::with_capacity(data.len());
    output.push(0xFF);
    output.push(0xD8);

    let mut i = 2;
    while i < data.len() {
        if i + 1 >= data.len() {
            output.extend_from_slice(&data[i..]);
            break;
        }

        if data[i] == 0xFF {
            let marker = data[i + 1];
            if marker == 0xD9 {
                output.push(0xFF);
                output.push(0xD9);
                break;
            }

            if marker == 0x00 || (marker >= 0xD0 && marker <= 0xD7) {
                output.push(0xFF);
                output.push(marker);
                i += 2;
                continue;
            }

            if i + 3 >= data.len() {
                output.extend_from_slice(&data[i..]);
                break;
            }

            let len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
            
            if marker == 0xE1 || marker == 0xFE {
                i += 2 + len;
            } else {
                if i + 2 + len <= data.len() {
                    output.extend_from_slice(&data[i..i + 2 + len]);
                    i += 2 + len;
                } else {
                    output.extend_from_slice(&data[i..]);
                    break;
                }
            }
        } else {
            output.push(data[i]);
            i += 1;
        }
    }

    output
}

pub fn strip_png_metadata(data: &[u8]) -> Vec<u8> {
    let png_signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 8 || data[0..8] != png_signature {
        return data.to_vec();
    }

    let mut output = Vec::with_capacity(data.len());
    output.extend_from_slice(&png_signature);

    let mut i = 8;
    while i + 8 <= data.len() {
        let length = ((data[i] as u32) << 24
            | (data[i + 1] as u32) << 16
            | (data[i + 2] as u32) << 8
            | (data[i + 3] as u32)) as usize;
        
        let chunk_type = &data[i + 4..i + 8];

        let is_metadata = chunk_type == b"eXIf" 
            || chunk_type == b"tEXt" 
            || chunk_type == b"zTXt" 
            || chunk_type == b"iTXt";

        let total_chunk_len = 4 + 4 + length + 4;
        if i + total_chunk_len <= data.len() {
            if !is_metadata {
                output.extend_from_slice(&data[i..i + total_chunk_len]);
            }
            i += total_chunk_len;
        } else {
            output.extend_from_slice(&data[i..]);
            break;
        }
    }

    output
}

pub fn strip_metadata(data: &[u8]) -> Vec<u8> {
    if data.len() >= 4 && data[0] == 0xFF && data[1] == 0xD8 {
        strip_jpeg_metadata(data)
    } else if data.len() >= 8 && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        strip_png_metadata(data)
    } else {
        data.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_blob_round_trip() {
        let salt = generate_salt();
        let key = derive_key("fixture-password-not-used-elsewhere", &salt).unwrap();
        let plain = b"sample evidence bytes for Jordan Example";
        let blob = encrypt_blob(plain, &key).unwrap();
        assert_ne!(&blob[NONCE_LEN..], plain.as_slice());
        let out = decrypt_blob(&blob, &key).unwrap();
        assert_eq!(out, plain);
    }
}
