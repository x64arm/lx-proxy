use serde_json::json;
use std::path::Path;

/// 初始化 Xray 配置
pub fn init_xray_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::var("XRAY_CONFIG_PATH")
        .unwrap_or_else(|_| "/usr/local/etc/xray/config.json".to_string());

    // 创建默认配置
    let config = json!({
        "log": {
            "loglevel": "warning",
            "error": "/var/log/xray/error.log",
            "access": "/var/log/xray/access.log"
        },
        "inbounds": [],
        "outbounds": [{
            "tag": "direct",
            "protocol": "freedom",
            "settings": {}
        }, {
            "tag": "blocked",
            "protocol": "blackhole",
            "settings": {}
        }],
        "routing": {
            "rules": [
                {
                    "type": "field",
                    "ip": ["geoip:private"],
                    "outboundTag": "blocked"
                }
            ]
        }
    });

    // 确保目录存在
    if let Some(parent) = Path::new(&config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 写入配置文件
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    
    tracing::info!("Xray config initialized at {}", config_path);
    
    Ok(())
}

/// 生成入站配置
pub fn generate_inbound(
    tag: &str,
    protocol: &str,
    port: i32,
    settings: &serde_json::Value,
    stream_settings: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut inbound = json!({
        "tag": tag,
        "port": port,
        "protocol": protocol,
        "settings": settings
    });

    if let Some(stream) = stream_settings {
        inbound["streamSettings"] = stream.clone();
    }

    inbound
}

/// 生成 Vmess 配置
pub fn generate_vmess_config(
    user_id: &str,
    email: &str,
    alter_id: i32,
) -> serde_json::Value {
    json!({
        "clients": [{
            "id": user_id,
            "email": email,
            "alterId": alter_id,
            "level": 0
        }]
    })
}

/// 生成 Vless 配置
pub fn generate_vless_config(
    user_id: &str,
    email: &str,
) -> serde_json::Value {
    json!({
        "clients": [{
            "id": user_id,
            "email": email,
            "level": 0,
            "flow": ""
        }]
    })
}

/// 生成 Trojan 配置
pub fn generate_trojan_config(
    password: &str,
    email: &str,
) -> serde_json::Value {
    json!({
        "clients": [{
            "password": password,
            "email": email,
            "level": 0
        }]
    })
}

/// 生成 Shadowsocks 配置
pub fn generate_shadowsocks_config(
    password: &str,
    method: &str,
) -> serde_json::Value {
    json!({
        "clients": [{
            "method": method,
            "password": password,
            "email": ""
        }]
    })
}

/// 重启 Xray 服务
pub fn restart_xray() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("systemctl")
        .args(["restart", "xray"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to restart Xray: {}", stderr).into());
    }

    tracing::info!("Xray service restarted");
    
    Ok(())
}

/// 检查 Xray 服务状态
pub fn check_xray_status() -> bool {
    use std::process::Command;

    let output = Command::new("systemctl")
        .args(["is-active", "xray"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.trim() == "active"
        }
        Err(_) => false,
    }
}
