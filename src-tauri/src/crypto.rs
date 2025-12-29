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
