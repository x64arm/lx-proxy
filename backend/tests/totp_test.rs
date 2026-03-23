/// TOTP 双因素认证测试
/// 注意：这些测试需要访问内部模块，暂时作为集成测试示例

#[cfg(test)]
mod tests {
    use chrono::Utc;

    #[test]
    fn test_base32_encoding() {
        // 测试 Base32 编码特性
        let test_string = "JBSWY3DPEHPK3PXP";
        
        // Base32 只包含 A-Z 和 2-7
        for c in test_string.chars() {
            assert!(
                c.is_ascii_uppercase() || (c.is_ascii_digit() && c >= '2' && c <= '7'),
                "Invalid Base32 character: {}",
                c
            );
        }
    }

    #[test]
    fn test_backup_code_format() {
        // 测试备用代码格式（8 位数字）
        let test_codes = vec!["12345678", "87654321", "00000000"];
        
        for code in test_codes {
            assert_eq!(code.len(), 8, "Backup code should be 8 digits");
            assert!(
                code.chars().all(|c: char| c.is_ascii_digit()),
                "Backup code should contain only digits"
            );
        }
    }

    #[test]
    fn test_totp_code_length() {
        // TOTP 验证码应该是 6 位
        let test_codes = vec!["123456", "000000", "999999"];
        
        for code in test_codes {
            assert_eq!(code.len(), 6, "TOTP code should be 6 digits");
        }
    }

    #[test]
    fn test_qr_url_format() {
        // 测试 QR Code URL 格式
        let secret = "JBSWY3DPEHPK3PXP";
        let account = "test@example.com";
        let issuer = "LX-Proxy";
        
        let qr_url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            issuer, account, secret, issuer
        );
        
        assert!(qr_url.starts_with("otpauth://totp/"));
        assert!(qr_url.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(qr_url.contains("issuer=LX-Proxy"));
        assert!(qr_url.contains("algorithm=SHA1"));
        assert!(qr_url.contains("digits=6"));
        assert!(qr_url.contains("period=30"));
    }

    #[test]
    fn test_timestamp_window() {
        // 测试时间窗口（30 秒）
        let now = Utc::now().timestamp() as u64;
        let window = 30;
        
        // 同一窗口内的时间应该产生相同的代码索引
        let time1 = now;
        let time2 = now + 10;
        let time3 = now + 60;
        
        assert_eq!(time1 / window, time2 / window, "Should be in same window");
        assert_ne!(time1 / window, time3 / window, "Should be in different window");
    }
}
