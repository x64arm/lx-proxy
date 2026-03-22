-- LX-Proxy 数据库初始化脚本
-- 自动创建表和初始数据

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100),
    role VARCHAR(20) DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT true
);

-- 入站配置表
CREATE TABLE IF NOT EXISTS inbounds (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    protocol VARCHAR(20) NOT NULL,
    port INTEGER NOT NULL,
    settings JSONB NOT NULL,
    total_gb BIGINT DEFAULT 0,
    used_gb BIGINT DEFAULT 0,
    expiry_time TIMESTAMP WITH TIME ZONE,
    enable BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 流量统计表
CREATE TABLE IF NOT EXISTS traffic_stats (
    id SERIAL PRIMARY KEY,
    inbound_id INTEGER REFERENCES inbounds(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    upload BIGINT DEFAULT 0,
    download BIGINT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(inbound_id, date)
);

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_config (
    id SERIAL PRIMARY KEY,
    key VARCHAR(100) UNIQUE NOT NULL,
    value TEXT,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_inbounds_protocol ON inbounds(protocol);
CREATE INDEX IF NOT EXISTS idx_inbounds_enable ON inbounds(enable);
CREATE INDEX IF NOT EXISTS idx_traffic_stats_inbound ON traffic_stats(inbound_id);
CREATE INDEX IF NOT EXISTS idx_traffic_stats_date ON traffic_stats(date);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- 插入默认管理员账号 (密码：admin123，使用 Argon2 哈希)
-- 注意：实际部署时应修改密码
INSERT INTO users (username, password_hash, email, role) 
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$Z0xQcm94eUFkbWluUGFzcw$example_hash_change_in_production', 'admin@lxproxy.local', 'admin')
ON CONFLICT (username) DO NOTHING;

-- 插入默认配置
INSERT INTO system_config (key, value, description) VALUES
    ('web_title', 'LX-Proxy', '网站标题'),
    ('web_subtitle', 'Xray 代理管理面板', '网站副标题'),
    ('timezone', 'Asia/Shanghai', '时区'),
    ('xray_path', '/usr/local/bin/xray', 'Xray 可执行文件路径'),
    ('xray_config_path', '/etc/xray/config.json', 'Xray 配置文件路径'),
    ('xray_port', '443', 'Xray 监听端口'),
    ('traffic_reset_day', '1', '每月流量重置日 (0=不重置)'),
    ('traffic_retention_days', '30', '流量统计保留天数'),
    ('session_timeout', '24', '会话超时时间 (小时)'),
    ('max_login_attempts', '5', '最大登录尝试次数'),
    ('ip_ban_duration', '60', 'IP 封禁时长 (分钟)')
ON CONFLICT (key) DO NOTHING;

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为各表添加更新时间触发器
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_inbounds_updated_at
    BEFORE UPDATE ON inbounds
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_system_config_updated_at
    BEFORE UPDATE ON system_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 创建视图：用户流量统计
CREATE OR REPLACE VIEW user_traffic_summary AS
SELECT 
    u.id,
    u.username,
    u.email,
    u.role,
    COALESCE(SUM(t.upload + t.download), 0) as total_traffic,
    COALESCE(SUM(t.upload), 0) as total_upload,
    COALESCE(SUM(t.download), 0) as total_download
FROM users u
LEFT JOIN inbounds i ON u.id = i.id
LEFT JOIN traffic_stats t ON i.id = t.inbound_id
GROUP BY u.id, u.username, u.email, u.role;

COMMENT ON TABLE users IS '用户表';
COMMENT ON TABLE inbounds IS '入站配置表';
COMMENT ON TABLE traffic_stats IS '流量统计表';
COMMENT ON TABLE system_config IS '系统配置表';
