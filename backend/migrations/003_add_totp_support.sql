-- 用户 TOTP 配置表
CREATE TABLE IF NOT EXISTS user_totp_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    secret VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    verified BOOLEAN NOT NULL DEFAULT false,
    backup_codes JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_user_totp_configs_user_id ON user_totp_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_totp_configs_enabled ON user_totp_configs(enabled);

-- 添加备注
COMMENT ON TABLE user_totp_configs IS '用户双因素认证（TOTP）配置';
COMMENT ON COLUMN user_totp_configs.secret IS 'TOTP 密钥（Base32 编码）';
COMMENT ON COLUMN user_totp_configs.enabled IS '是否启用 TOTP';
COMMENT ON COLUMN user_totp_configs.verified IS '是否已验证';
COMMENT ON COLUMN user_totp_configs.backup_codes IS '备用代码（JSON 数组）';

-- 添加 TOTP 相关系统配置
INSERT INTO system_configs (key, value, description) VALUES
('totp_issuer', '{"value": "LX-Proxy"}', 'TOTP 发行者名称'),
('totp_required_for_admin', '{"value": false}', '管理员是否必须启用 TOTP'),
('totp_grace_period_minutes', '{"value": 5}', '登录时 TOTP 宽限期（分钟）')
ON CONFLICT (key) DO NOTHING;
