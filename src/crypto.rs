use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::RngCore, SaltString};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::{QRCryptError, Result};

const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedData {
    pub nonce: String,
    pub salt: String,
    pub ciphertext: String,
    pub version: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LayeredData {
    pub decoy_layer: EncryptedData,
    pub hidden_layer: Option<EncryptedData>,
    pub version: u8,
    pub decoy_hint: Option<String>, // Optional hint to make decoy believable
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretData {
    pub data: String,
}

impl SecretData {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

pub struct Crypto {
    argon2: Argon2<'static>,
}

impl Default for Crypto {
    fn default() -> Self {
        Self::new()
    }
}

impl Crypto {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    pub fn encrypt(&self, data: &SecretData, password: &str) -> Result<EncryptedData> {
        // Generate salt for password derivation
        let salt = SaltString::generate(&mut OsRng);
        
        // Derive key from password using Argon2
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| QRCryptError::Encryption(format!("Password hashing failed: {}", e)))?;
        
        // Extract the key bytes (first 32 bytes of the hash)
        let hash = password_hash.hash
            .ok_or_else(|| QRCryptError::Encryption("Failed to extract key from hash".to_string()))?;
        let key_bytes = hash.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err(QRCryptError::Encryption("Derived key too short".to_string()));
        }
        
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, data.data.as_bytes())
            .map_err(|e| QRCryptError::Encryption(format!("AES-GCM encryption failed: {}", e)))?;
        
        Ok(EncryptedData {
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            salt: salt.to_string(),
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            version: 1,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData, password: &str) -> Result<SecretData> {
        if encrypted.version != 1 {
            return Err(QRCryptError::Decryption(format!("Unsupported version: {}", encrypted.version)));
        }
        
        // Parse salt
        let salt = SaltString::from_b64(&encrypted.salt)
            .map_err(|e| QRCryptError::Decryption(format!("Invalid salt: {}", e)))?;
        
        // Derive key from password
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| QRCryptError::Decryption(format!("Password hashing failed: {}", e)))?;
        
        let hash = password_hash.hash
            .ok_or_else(|| QRCryptError::Decryption("Failed to extract key from hash".to_string()))?;
        let key_bytes = hash.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err(QRCryptError::Decryption("Derived key too short".to_string()));
        }
        
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        // Decode nonce and ciphertext
        let nonce_bytes = general_purpose::STANDARD.decode(&encrypted.nonce)?;
        let ciphertext_bytes = general_purpose::STANDARD.decode(&encrypted.ciphertext)?;
        
        if nonce_bytes.len() != NONCE_SIZE {
            return Err(QRCryptError::Decryption("Invalid nonce size".to_string()));
        }
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext_bytes.as_ref())
            .map_err(|e| QRCryptError::Decryption(format!("AES-GCM decryption failed: {}", e)))?;
        
        let data = String::from_utf8(plaintext)
            .map_err(|e| QRCryptError::Decryption(format!("Invalid UTF-8: {}", e)))?;
        
        Ok(SecretData::new(data))
    }

    pub fn verify_password(&self, encrypted: &EncryptedData, password: &str) -> Result<bool> {
        let salt = SaltString::from_b64(&encrypted.salt)
            .map_err(|e| QRCryptError::Decryption(format!("Invalid salt: {}", e)))?;
        
        let _password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| QRCryptError::Decryption(format!("Password hashing failed: {}", e)))?;
        
        // Try to decrypt a small portion to verify password
        match self.decrypt(encrypted, password) {
            Ok(_) => Ok(true),
            Err(QRCryptError::Decryption(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    // Plausible deniability: create layered encryption with decoy data
    pub fn encrypt_with_decoy(&self, real_data: &SecretData, real_password: &str, 
                              decoy_data: &SecretData, decoy_password: &str,
                              decoy_hint: Option<String>) -> Result<LayeredData> {
        let decoy_layer = self.encrypt(decoy_data, decoy_password)?;
        let hidden_layer = Some(self.encrypt(real_data, real_password)?);
        
        Ok(LayeredData {
            decoy_layer,
            hidden_layer,
            version: 1,
            decoy_hint,
        })
    }

    // Try to decrypt layered data with given password
    pub fn decrypt_layered(&self, layered: &LayeredData, password: &str) -> Result<(SecretData, bool)> {
        // First try the decoy layer
        if let Ok(decoy_data) = self.decrypt(&layered.decoy_layer, password) {
            return Ok((decoy_data, false)); // false = this is decoy data
        }
        
        // Then try the hidden layer if it exists
        if let Some(hidden_layer) = &layered.hidden_layer {
            if let Ok(real_data) = self.decrypt(hidden_layer, password) {
                return Ok((real_data, true)); // true = this is real data
            }
        }
        
        Err(QRCryptError::Decryption("Password does not match any layer".to_string()))
    }
}