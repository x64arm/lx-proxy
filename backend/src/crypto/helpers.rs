// 加密辅助函数
// 用于数据库字段加密/解密

use sqlx::{PgPool, Type};
use serde::{Serialize, Deserialize};

/// 加密文本字段（用于数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedText {
    pub ciphertext: String,
}

impl EncryptedText {
    /// 创建加密文本
    pub fn new(plaintext: &str, encryptor: &crate::crypto::DataEncryptor) -> Result<Self, String> {
        let ciphertext = encryptor.encrypt_string(plaintext)?;
        Ok(Self { ciphertext })
    }

    /// 解密文本
    pub fn decrypt(&self, encryptor: &crate::crypto::DataEncryptor) -> Result<String, String> {
        encryptor.decrypt_string(&self.ciphertext)
    }

    /// 从密文创建（用于从数据库加载）
    pub fn from_ciphertext(ciphertext: String) -> Self {
        Self { ciphertext }
    }
}

/// 加密 API 密钥
pub fn encrypt_api_key(
    api_key: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.encrypt_string(api_key)
}

/// 解密 API 密钥
pub fn decrypt_api_key(
    encrypted_key: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.decrypt_string(encrypted_key)
}

/// 加密密码（用于数据库存储，注意：登录时仍需哈希验证）
pub fn encrypt_password(
    password: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.encrypt_string(password)
}

/// 解密密码
pub fn decrypt_password(
    encrypted_password: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.decrypt_string(encrypted_password)
}

/// 加密 Token
pub fn encrypt_token(
    token: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.encrypt_string(token)
}

/// 解密 Token
pub fn decrypt_token(
    encrypted_token: &str,
    encryptor: &crate::crypto::DataEncryptor,
) -> Result<String, String> {
    encryptor.decrypt_string(encrypted_token)
}

/// 敏感数据模型（自动加密/解密）
pub struct SensitiveField<T> {
    encrypted_value: Option<String>,
    decrypted_value: Option<T>,
}

impl<T> SensitiveField<T>
where
    T: ToString + std::str::FromStr,
{
    /// 从明文创建
    pub fn from_plaintext(value: T, encryptor: &crate::crypto::DataEncryptor) -> Result<Self, String> {
        let encrypted = encryptor.encrypt_string(&value.to_string())?;
        Ok(Self {
            encrypted_value: Some(encrypted),
            decrypted_value: Some(value),
        })
    }

    /// 从密文加载
    pub fn from_ciphertext(encrypted: String) -> Self {
        Self {
            encrypted_value: Some(encrypted),
            decrypted_value: None,
        }
    }

    /// 获取明文（需要时解密）
    pub fn plaintext(&mut self, encryptor: &crate::crypto::DataEncryptor) -> Result<&T, String> {
        if self.decrypted_value.is_none() {
            if let Some(encrypted) = &self.encrypted_value {
                let decrypted = encryptor.decrypt_string(encrypted)?;
                self.decrypted_value = Some(
                    decrypted
                        .parse()
                        .map_err(|_| "Failed to parse decrypted value".to_string())?,
                );
            }
        }
        self.decrypted_value
            .as_ref()
            .ok_or_else(|| "No value available".to_string())
    }

    /// 获取密文（用于存储）
    pub fn ciphertext(&self) -> Option<&String> {
        self.encrypted_value.as_ref()
    }
}
