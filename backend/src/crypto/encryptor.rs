// 数据加密器
// 使用 AES-256-GCM 加密敏感数据

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use std::sync::Arc;

/// 数据加密器
pub struct DataEncryptor {
    cipher: Arc<Aes256Gcm>,
}

impl DataEncryptor {
    /// 创建新的加密器
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Arc::new(Aes256Gcm::new_from_slice(key).expect("Invalid key length")),
        }
    }

    /// 从环境变量创建加密器
    pub fn from_env() -> Result<Self, String> {
        let key_hex = std::env::var("ENCRYPTION_KEY")
            .map_err(|_| "ENCRYPTION_KEY environment variable not set".to_string())?;

        let key_bytes = hex::decode(&key_hex)
            .map_err(|e| format!("Invalid hex key: {}", e))?;

        if key_bytes.len() != 32 {
            return Err("ENCRYPTION_KEY must be 32 bytes (256 bits)".to_string());
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);

        Ok(Self::new(&key))
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, String> {
        // 生成随机 nonce (12 bytes for GCM)
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // 组合 nonce + ciphertext 并 base64 编码
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    /// 解密数据
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<Vec<u8>, String> {
        // base64 解码
        let data = general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid ciphertext: too short".to_string());
        }

        // 提取 nonce 和 ciphertext
        let nonce_bytes = &data[..12];
        let ciphertext = &data[12..];

        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// 加密字符串
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, String> {
        self.encrypt(plaintext.as_bytes())
    }

    /// 解密为字符串
    pub fn decrypt_string(&self, ciphertext_b64: &str) -> Result<String, String> {
        let plaintext = self.decrypt(ciphertext_b64)?;
        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

/// 生成随机加密密钥（32 字节 / 256 位）
pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// 将密钥转换为十六进制字符串
pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex::encode(key)
}

/// 从十六进制字符串解析密钥
pub fn key_from_hex(hex: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex)
        .map_err(|e| format!("Invalid hex: {}", e))?;

    if bytes.len() != 32 {
        return Err("Key must be 32 bytes".to_string());
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_encryption_key();
        let encryptor = DataEncryptor::new(&key);

        let plaintext = b"Hello, World!";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = generate_encryption_key();
        let encryptor = DataEncryptor::new(&key);

        let plaintext = "Sensitive API Key: sk-1234567890abcdef";
        let ciphertext = encryptor.encrypt_string(plaintext).unwrap();
        let decrypted = encryptor.decrypt_string(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = generate_encryption_key();
        let encryptor = DataEncryptor::new(&key);

        let plaintext = b"Same plaintext";
        let ciphertext1 = encryptor.encrypt(plaintext).unwrap();
        let ciphertext2 = encryptor.encrypt(plaintext).unwrap();

        // 每次加密应该产生不同的密文（因为 nonce 不同）
        assert_ne!(ciphertext1, ciphertext2);

        // 但都应该能正确解密
        let decrypted1 = encryptor.decrypt(&ciphertext1).unwrap();
        let decrypted2 = encryptor.decrypt(&ciphertext2).unwrap();

        assert_eq!(plaintext, &decrypted1[..]);
        assert_eq!(plaintext, &decrypted2[..]);
    }
}
