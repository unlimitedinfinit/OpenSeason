use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce, Key
};
use rand::RngCore;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Known plaintext sealed with the derived key and stored next to the salt.
pub const PASSWORD_VERIFIER_MAGIC: &[u8] = b"OpenSeason-password-verifier-v1";

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

pub fn create_password_verifier(key: &SessionKey) -> Result<Vec<u8>, String> {
    encrypt_blob(PASSWORD_VERIFIER_MAGIC, key)
}

pub fn verify_password_key(key: &SessionKey, verifier_blob: &[u8]) -> Result<(), String> {
    let plain = decrypt_blob(verifier_blob, key).map_err(|_| "Wrong password.".to_string())?;
    if plain.as_slice() != PASSWORD_VERIFIER_MAGIC {
        return Err("Wrong password.".to_string());
    }
    Ok(())
}

/// Unlock: reject a wrong password when a verifier exists. First unlock writes one.
pub fn unlock_vault_at(
    password: &str,
    salt: &str,
    verifier_path: &Path,
) -> Result<SessionKey, String> {
    if password.is_empty() {
        return Err("Password required.".to_string());
    }
    let key = derive_key(password, salt)?;
    if verifier_path.exists() {
        let blob = fs::read(verifier_path).map_err(|e| e.to_string())?;
        verify_password_key(&key, &blob)?;
    } else {
        if let Some(parent) = verifier_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let blob = create_password_verifier(&key)?;
        fs::write(verifier_path, blob).map_err(|e| e.to_string())?;
    }
    Ok(key)
}

/// Seal-time check. The verifier must already exist. Never call this after deleting plaintext.
pub fn confirm_password_for_seal(
    password: &str,
    salt: &str,
    verifier: &[u8],
) -> Result<SessionKey, String> {
    if password.is_empty() {
        return Err("Re-enter the vault password to seal. Plaintext is not deleted until the password matches.".to_string());
    }
    if verifier.is_empty() {
        return Err("No password verifier on disk. Unlock the vault once to set the password before sealing.".to_string());
    }
    let key = derive_key(password, salt)?;
    verify_password_key(&key, verifier)?;
    Ok(key)
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

    #[test]
    fn wrong_password_rejected_at_unlock() {
        let dir = std::env::temp_dir().join(format!("os-unlock-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let salt = generate_salt();
        let verifier_path = dir.join("master_verifier.bin");
        unlock_vault_at("correct-horse-battery", &salt, &verifier_path).unwrap();
        assert!(verifier_path.exists());

        let err = match unlock_vault_at("wrong-password-typed", &salt, &verifier_path) {
            Ok(_) => panic!("wrong password must not unlock"),
            Err(e) => e,
        };
        assert!(
            err.to_lowercase().contains("wrong password"),
            "wrong password must be rejected at unlock, got {err}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn first_unlock_writes_verifier() {
        let dir = std::env::temp_dir().join(format!("os-unlock-first-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let salt = generate_salt();
        let verifier_path = dir.join("master_verifier.bin");
        assert!(!verifier_path.exists());
        unlock_vault_at("first-time-password", &salt, &verifier_path).unwrap();
        assert!(verifier_path.exists());
        unlock_vault_at("first-time-password", &salt, &verifier_path).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}
