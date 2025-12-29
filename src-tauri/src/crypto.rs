use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use std::sync::{Arc, Mutex};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Constant for the salt length (16 bytes is standard)
// Note: In production Argon2, salt is usually stored with the hash.
// For our "Session-Only" derived key that decrypts the vault, we need a consistent salt
// OR we store the salt in the vault header.
// Given the "Master Password is never stored" rule, we usually derive the Key from
// (Password + Salt). If the Salt is random every time, the Key is different every time,
// which means we can't decrypt data encrypted with a previous session's key.
// Therefore, the Salt *must* be stored alongside the encrypted data (e.g., in a header or distinct file),
// OR the user provides the Salt (unlikely), OR we use a mechanism where the Vault itself 
// has a "Lockbox" header containing the actual Data Encryption Key (DEK) encrypted by a Key Encryption Key (KEK).
// The KEK is derived from Password + Random Salt (stored in config).
// However, specifically for this "Zero-Trust" local app, usually:
// 1. User sets Master PW -> Generate Random Salt -> Save Salt to disk (cleartext is fine for salt).
// 2. Derive KEK = Argon2(PW, Salt).
//
// For simplicity in this specific "Hunter" app context where we might want plausible deniability:
// If the prompt says "Encryption is fixed to XChaCha20Poly1305 using a random 192-bit nonce",
// it implies per-file or per-record encryption.
//
// Let's implement the standard KEK workflow:
// - `VaultConfig` stores the Argon2 Salt.
// - Application loads Salt on startup.
// - User admits PW -> Derive SessionKey.

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
    // Note: PasswordHash usually produces a string PHC format. 
    // We want raw bytes for the XChaCha20 key.
    // So we use `hash_password_custom` or simply hash to a 32-byte output buffer.
    
    let mut output_key_material = [0u8; 32];
    argon2.hash_password_into(
        password.as_bytes(),
        salt.as_salt(),
        &mut output_key_material
    ).map_err(|e| e.to_string())?;

    Ok(SessionKey(output_key_material))
}

pub fn encrypt_data(data: &[u8], key: &SessionKey) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    
    // Generate random 192-bit (24-byte) nonce
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| e.to_string())?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce_bytes: &[u8], key: &SessionKey) -> Result<Vec<u8>, String> {
    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    let nonce = XNonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(plaintext)
}
