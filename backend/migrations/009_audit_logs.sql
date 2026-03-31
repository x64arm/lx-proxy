-- P16 审计日志系统 - 数据库迁移

-- 审计日志表
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    username VARCHAR(100),
    action VARCHAR(50) NOT NULL,  -- create, update, delete, login, logout, sync, etc.
    resource_type VARCHAR(50),  -- user, inbound, node, config, etc.
    resource_id UUID,
    ip_address INET,
    user_agent TEXT,
    request_method VARCHAR(10),
    request_path TEXT,
    request_body JSONB,
    response_status INTEGER,
    response_body JSONB,
    duration_ms INTEGER,
    status VARCHAR(20) DEFAULT 'success',  -- success, failure, error
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_username ON audit_logs(username);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_status ON audit_logs(status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_ip_address ON audit_logs(ip_address);

-- 登录日志表
CREATE TABLE IF NOT EXISTS login_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    username VARCHAR(100),
    ip_address INET NOT NULL,
    user_agent TEXT,
    status VARCHAR(20) NOT NULL,  -- success, failed, blocked
    failure_reason VARCHAR(100),  -- invalid_password, totp_failed, account_locked, etc.
    session_id UUID,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_login_logs_user_id ON login_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_login_logs_username ON login_logs(username);
CREATE INDEX IF NOT EXISTS idx_login_logs_ip_address ON login_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_login_logs_created_at ON login_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_login_logs_status ON login_logs(status);

-- IP 封禁表
CREATE TABLE IF NOT EXISTS ip_bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    reason VARCHAR(255),
    banned_until TIMESTAMP WITH TIME ZONE NOT NULL,
    banned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    banned_by UUID REFERENCES users(id) ON DELETE SET NULL,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_ip_bans_ip_address ON ip_bans(ip_address);
CREATE INDEX IF NOT EXISTS idx_ip_bans_active ON ip_bans(is_active);
CREATE INDEX IF NOT EXISTS idx_ip_bans_until ON ip_bans(banned_until);

-- 配置变更历史表
CREATE TABLE IF NOT EXISTS config_change_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    username VARCHAR(100),
    config_type VARCHAR(50) NOT NULL,  -- system, xray, inbound, user, etc.
    config_id UUID,
    action VARCHAR(20) NOT NULL,  -- create, update, delete
    old_value JSONB,
    new_value JSONB,
    changes_summary TEXT,  -- 人类可读的变更描述
    ip_address INET,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_config_change_history_user_id ON config_change_history(user_id);
CREATE INDEX IF NOT EXISTS idx_config_change_history_config_type ON config_change_history(config_type);
CREATE INDEX IF NOT EXISTS idx_config_change_history_config_id ON config_change_history(config_id);
CREATE INDEX IF NOT EXISTS idx_config_change_history_created_at ON config_change_history(created_at);

-- 审计日志视图 - 最近 7 天活动
CREATE OR REPLACE VIEW audit_logs_recent AS
SELECT 
    DATE_TRUNC('hour', created_at) as hour,
    action,
    status,
    COUNT(*) as count
FROM audit_logs
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY DATE_TRUNC('hour', created_at), action, status
ORDER BY hour DESC, action, status;

-- 审计日志视图 - 用户活动统计
CREATE OR REPLACE VIEW user_activity_stats AS
SELECT 
    COALESCE(user_id, '00000000-0000-0000-0000-000000000000'::uuid) as user_id,
    COALESCE(username, 'unknown') as username,
    COUNT(*) as total_actions,
    COUNT(DISTINCT DATE(created_at)) as active_days,
    COUNT(CASE WHEN status = 'success' THEN 1 END) as successful_actions,
    COUNT(CASE WHEN status = 'failure' THEN 1 END) as failed_actions,
    MAX(created_at) as last_activity,
    MIN(created_at) as first_activity
FROM audit_logs
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY user_id, username
ORDER BY total_actions DESC;

-- 添加注释
COMMENT ON TABLE audit_logs IS '系统审计日志表';
COMMENT ON TABLE login_logs IS '用户登录日志表';
COMMENT ON TABLE ip_bans IS 'IP 封禁表';
COMMENT ON TABLE config_change_history IS '配置变更历史表';
COMMENT ON VIEW audit_logs_recent IS '最近 7 天审计日志统计';
COMMENT ON VIEW user_activity_stats IS '用户活动统计视图';
