-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user', -- admin, user
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 入站配置表
CREATE TABLE IF NOT EXISTS inbound_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    tag VARCHAR(100) UNIQUE NOT NULL,
    protocol VARCHAR(50) NOT NULL, -- vmess, vless, trojan, shadowsocks, wireguard, http, tcp, ws
    port INTEGER NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}',
    stream_settings JSONB DEFAULT '{}',
    sniffing JSONB DEFAULT '{}',
    enable BOOLEAN NOT NULL DEFAULT true,
    traffic_used BIGINT NOT NULL DEFAULT 0,
    traffic_limit BIGINT, -- bytes, NULL for unlimited
    expire_at TIMESTAMPTZ, -- NULL for no expiration
    ip_limit INTEGER, -- NULL for unlimited
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 流量统计表
CREATE TABLE IF NOT EXISTS traffic_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inbound_id UUID NOT NULL REFERENCES inbound_configs(id) ON DELETE CASCADE,
    upload BIGINT NOT NULL DEFAULT 0,
    download BIGINT NOT NULL DEFAULT 0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_configs (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Xray 配置表
CREATE TABLE IF NOT EXISTS xray_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    config JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 操作日志表
CREATE TABLE IF NOT EXISTS operation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(100),
    resource_id UUID,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_inbound_configs_user_id ON inbound_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_inbound_configs_protocol ON inbound_configs(protocol);
CREATE INDEX IF NOT EXISTS idx_inbound_configs_enable ON inbound_configs(enable);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_inbound_id ON traffic_logs(inbound_id);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_recorded_at ON traffic_logs(recorded_at);
CREATE INDEX IF NOT EXISTS idx_operation_logs_user_id ON operation_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at);

-- 插入默认管理员账户 (密码：admin123)
INSERT INTO users (username, password_hash, role) VALUES
('admin', '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpams$abcdefghijklmnopqrstuvwxyz1234567890', 'admin')
ON CONFLICT (username) DO NOTHING;

-- 插入默认系统配置
INSERT INTO system_configs (key, value, description) VALUES
('panel_port', '{"value": 8080}', '面板端口'),
('panel_path', '{"value": "/app"}', '面板访问路径'),
('cert_enabled', '{"value": false}', '是否启用 SSL 证书'),
('traffic_reset_day', '{"value": 0}', '每月流量重置日期，0 表示不重置'),
('xray_bin', '{"value": "/usr/local/bin/xray"}', 'Xray 可执行文件路径'),
('xray_config_path', '{"value": "/usr/local/etc/xray/config.json"}', 'Xray 配置文件路径')
ON CONFLICT (key) DO NOTHING;
