/// 双因素认证模块（TOTP）
/// 基于时间的一次性密码算法（RFC 6238）

use base32;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use totp_rs::{Algorithm, TOTP};
use tracing::info;

/// TOTP 配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    pub secret: String,
    pub enabled: bool,
    pub verified: bool,
    pub backup_codes: Vec<String>,
    pub created_at: chrono::DateTime<Utc>,
}

/// TOTP 设置请求
#[derive(Debug, Deserialize)]
pub struct SetupTotpRequest {
    pub user_id: Uuid,
}

/// TOTP 设置响应
#[derive(Debug, Serialize)]
pub struct SetupTotpResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

/// TOTP 验证请求
#[derive(Debug, Deserialize)]
pub struct VerifyTotpRequest {
    pub user_id: Uuid,
    pub code: String,
}

/// TOTP 验证响应
#[derive(Debug, Serialize)]
pub struct VerifyTotpResponse {
    pub success: bool,
    pub message: String,
}

/// 生成 TOTP 密钥
pub fn generate_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes)
}

/// 创建 TOTP 实例
pub fn create_totp(secret: &str, account_name: &str, issuer: &str) -> TOTP {
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
    )
    .expect("Failed to create TOTP")
}

/// 生成 QR Code URL（用于 Google Authenticator 等）
pub fn generate_qr_code_url(secret: &str, account_name: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer,
        account_name,
        secret,
        issuer
    )
}

/// 验证 TOTP 代码
pub fn verify_code(secret: &str, code: &str) -> bool {
    let totp = create_totp(secret, "temp", "temp");
    let time = Utc::now().timestamp() as u64;
    totp.check(code, time)
}

/// 生成备用代码
pub fn generate_backup_codes(count: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            (0..8)
                .map(|_| rng.gen_range(0..10).to_string())
                .collect::<String>()
        })
        .collect()
}

/// 为用户设置 TOTP
pub async fn setup_totp(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
) -> Result<SetupTotpResponse, sqlx::Error> {
    let secret = generate_secret();
    let backup_codes = generate_backup_codes(10);
    let qr_code_url = generate_qr_code_url(&secret, username, "LX-Proxy");

    // 保存 TOTP 配置到数据库（未验证状态）
    let totp_config = TotpConfig {
        secret: secret.clone(),
        enabled: false,
        verified: false,
        backup_codes: backup_codes.clone(),
        created_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO user_totp_configs (user_id, secret, enabled, verified, backup_codes, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id) DO UPDATE SET
            secret = $2,
            enabled = false,
            verified = false,
            backup_codes = $5,
            created_at = $6,
            updated_at = NOW()
        "#
    )
    .bind(user_id)
    .bind(&secret)
    .bind(false)
    .bind(false)
    .bind(&serde_json::to_value(&backup_codes).unwrap())
    .bind(Utc::now())
    .execute(pool)
    .await?;

    info!("🔐 TOTP setup initiated for user: {}", username);

    Ok(SetupTotpResponse {
        secret,
        qr_code_url,
        backup_codes,
    })
}

/// 验证并启用 TOTP
pub async fn enable_totp(
    pool: &PgPool,
    user_id: Uuid,
    code: &str,
) -> Result<VerifyTotpResponse, sqlx::Error> {
    // 获取用户的 TOTP 配置
    let totp_config = sqlx::query_as::<_, (String, bool, bool)>(
        r#"
        SELECT secret, enabled, verified
        FROM user_totp_configs
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (secret, enabled, verified) = match totp_config {
        Some(config) => config,
        None => {
            return Ok(VerifyTotpResponse {
                success: false,
                message: "TOTP 未设置".to_string(),
            });
        }
    };

    if enabled && verified {
        return Ok(VerifyTotpResponse {
            success: false,
            message: "TOTP 已启用".to_string(),
        });
    }

    // 验证代码
    if !verify_code(&secret, code) {
        return Ok(VerifyTotpResponse {
            success: false,
            message: "验证码错误".to_string(),
        });
    }

    // 启用 TOTP
    sqlx::query(
        r#"
        UPDATE user_totp_configs
        SET enabled = true, verified = true, updated_at = NOW()
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    info!("✅ TOTP enabled for user: {}", user_id);

    Ok(VerifyTotpResponse {
        success: true,
        message: "TOTP 已启用".to_string(),
    })
}

/// 禁用 TOTP
pub async fn disable_totp(
    pool: &PgPool,
    user_id: Uuid,
    _password: &str,
) -> Result<VerifyTotpResponse, sqlx::Error> {
    // 验证密码（简化处理，实际应该由调用者验证）
    // 这里假设密码已经验证通过

    sqlx::query(
        r#"
        UPDATE user_totp_configs
        SET enabled = false, updated_at = NOW()
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    info!("🔓 TOTP disabled for user: {}", user_id);

    Ok(VerifyTotpResponse {
        success: true,
        message: "TOTP 已禁用".to_string(),
    })
}

/// 使用备用代码验证
pub async fn verify_backup_code(
    pool: &PgPool,
    user_id: Uuid,
    backup_code: &str,
) -> Result<bool, sqlx::Error> {
    let backup_codes: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT backup_codes
        FROM user_totp_configs
        WHERE user_id = $1 AND enabled = true
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or(serde_json::json!([]));

    let codes: Vec<String> = serde_json::from_value(backup_codes).unwrap_or_default();

    if codes.contains(&backup_code.to_string()) {
        // 移除已使用的备用代码
        let new_codes: Vec<String> = codes
            .into_iter()
            .filter(|c| c != backup_code)
            .collect();

        sqlx::query(
            r#"
            UPDATE user_totp_configs
            SET backup_codes = $1, updated_at = NOW()
            WHERE user_id = $2
            "#
        )
        .bind(&serde_json::to_value(&new_codes).unwrap())
        .bind(user_id)
        .execute(pool)
        .await?;

        info!("🔑 Backup code used for user: {}", user_id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 检查用户是否启用了 TOTP
pub async fn is_totp_enabled(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let enabled: Option<bool> = sqlx::query_scalar(
        r#"
        SELECT enabled
        FROM user_totp_configs
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(enabled.unwrap_or(false))
}

/// 获取用户 TOTP 状态
pub async fn get_totp_status(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(bool, bool), sqlx::Error> {
    let status = sqlx::query_as::<_, (bool, bool)>(
        r#"
        SELECT enabled, verified
        FROM user_totp_configs
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(status.unwrap_or((false, false)))
}
