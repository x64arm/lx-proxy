-- 邮件发送日志表
CREATE TABLE IF NOT EXISTS email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient VARCHAR(255) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    template VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 登录会话表
CREATE TABLE IF NOT EXISTS login_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_email_logs_recipient ON email_logs(recipient);
CREATE INDEX IF NOT EXISTS idx_email_logs_status ON email_logs(status);
CREATE INDEX IF NOT EXISTS idx_email_logs_sent_at ON email_logs(sent_at DESC);

CREATE INDEX IF NOT EXISTS idx_login_sessions_user_id ON login_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_login_sessions_token ON login_sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_login_sessions_expires ON login_sessions(expires_at);

-- 添加备注
COMMENT ON TABLE email_logs IS '邮件发送日志表';
COMMENT ON TABLE login_sessions IS '用户登录会话表';

-- 清理过期数据函数
CREATE OR REPLACE FUNCTION cleanup_expired_data()
RETURNS void AS $$
BEGIN
    -- 清理 90 天前的流量日志
    DELETE FROM traffic_logs WHERE recorded_at < NOW() - INTERVAL '90 days';
    
    -- 清理 180 天前的操作日志
    DELETE FROM operation_logs WHERE created_at < NOW() - INTERVAL '180 days';
    
    -- 清理 90 天前的邮件日志
    DELETE FROM email_logs WHERE created_at < NOW() - INTERVAL '90 days';
    
    -- 清理过期会话
    DELETE FROM login_sessions WHERE expires_at < NOW();
    
    RAISE NOTICE '清理完成：删除了过期的流量日志、操作日志、邮件日志和会话';
END;
$$ LANGUAGE plpgsql;

-- 添加用户最后登录信息
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_ip INET;
ALTER TABLE users ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT true;

-- 创建用户启用状态索引
CREATE INDEX IF NOT EXISTS idx_users_enabled ON users(enabled);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login_at DESC);

-- 添加入站配置累计流量字段
ALTER TABLE inbound_configs ADD COLUMN IF NOT EXISTS total_upload BIGINT NOT NULL DEFAULT 0;
ALTER TABLE inbound_configs ADD COLUMN IF NOT EXISTS total_download BIGINT NOT NULL DEFAULT 0;
ALTER TABLE inbound_configs ADD COLUMN IF NOT EXISTS reset_day INTEGER DEFAULT 0;

-- 创建入站配置累计流量索引
CREATE INDEX IF NOT EXISTS idx_inbounds_total_traffic ON inbound_configs(total_upload, total_download);

-- 添加操作日志状态和 UA 字段
ALTER TABLE operation_logs ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'success';
ALTER TABLE operation_logs ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- 创建操作日志状态索引
CREATE INDEX IF NOT EXISTS idx_operation_logs_status ON operation_logs(status);

-- 添加系统配置分类
ALTER TABLE system_configs ADD COLUMN IF NOT EXISTS category VARCHAR(50) DEFAULT 'general';

-- 创建系统配置分类索引
CREATE INDEX IF NOT EXISTS idx_system_configs_category ON system_configs(category);

-- 插入默认系统配置
INSERT INTO system_configs (key, value, description, category) VALUES
-- 基本设置
('web_title', '{"value": "LX-Proxy"}', '网站标题', 'basic'),
('web_subtitle', '{"value": "Xray 代理管理面板"}', '网站副标题', 'basic'),
('language', '{"value": "zh-CN"}', '界面语言', 'basic'),
('timezone', '{"value": "Asia/Shanghai"}', '时区', 'basic'),

-- 安全设置
('session_timeout', '{"value": 24}', '会话超时（小时）', 'security'),
('max_login_attempts', '{"value": 5}', '最大登录尝试', 'security'),
('ip_ban_duration', '{"value": 60}', 'IP 封禁时长（分钟）', 'security'),
('totp_required_for_admin', '{"value": false}', '管理员强制 TOTP', 'security'),

-- 流量设置
('traffic_retention_days', '{"value": 90}', '流量记录保留天数', 'traffic'),
('traffic_warning_threshold', '{"value": 80}', '流量告警阈值（%）', 'traffic'),

-- 邮件设置
('smtp_enabled', '{"value": false}', '启用邮件通知', 'email'),
('smtp_port', '{"value": 587}', 'SMTP 端口', 'email'),
('smtp_from_name', '{"value": "LX-Proxy"}', '发件人名称', 'email')
ON CONFLICT (key) DO NOTHING;
