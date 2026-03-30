-- P13 订阅链接优化 - 数据库迁移

-- 订阅令牌表（用于加密订阅链接）
CREATE TABLE IF NOT EXISTS subscription_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inbound_id UUID NOT NULL REFERENCES inbound_configs(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscription_tokens_token ON subscription_tokens(token);
CREATE INDEX IF NOT EXISTS idx_subscription_tokens_inbound_id ON subscription_tokens(inbound_id);
CREATE INDEX IF NOT EXISTS idx_subscription_tokens_expires_at ON subscription_tokens(expires_at);

-- 订阅访问日志表（用于访问统计）
CREATE TABLE IF NOT EXISTS subscription_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inbound_id UUID NOT NULL REFERENCES inbound_configs(id) ON DELETE CASCADE,
    client_ip INET NOT NULL,
    user_agent TEXT,
    country VARCHAR(100),
    accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subscription_access_inbound_id ON subscription_access_logs(inbound_id);
CREATE INDEX IF NOT EXISTS idx_subscription_access_accessed_at ON subscription_access_logs(accessed_at);
CREATE INDEX IF NOT EXISTS idx_subscription_access_client_ip ON subscription_access_logs(client_ip);

-- 为 inbound_configs 添加访问统计字段
ALTER TABLE inbound_configs 
ADD COLUMN IF NOT EXISTS access_count BIGINT DEFAULT 0,
ADD COLUMN IF NOT EXISTS last_accessed_at TIMESTAMP WITH TIME ZONE;

-- 创建访问统计视图
CREATE OR REPLACE VIEW subscription_stats AS
SELECT 
    ic.id AS inbound_id,
    ic.remark AS inbound_name,
    ic.protocol,
    ic.access_count,
    ic.last_accessed_at,
    COUNT(sal.id) AS total_access,
    COUNT(DISTINCT sal.client_ip) AS unique_ips,
    COUNT(CASE WHEN sal.accessed_at > NOW() - INTERVAL '24 hours' THEN 1 END) AS last_24h_access,
    COUNT(CASE WHEN sal.accessed_at > NOW() - INTERVAL '7 days' THEN 1 END) AS last_7d_access
FROM inbound_configs ic
LEFT JOIN subscription_access_logs sal ON ic.id = sal.inbound_id
GROUP BY ic.id, ic.remark, ic.protocol, ic.access_count, ic.last_accessed_at;

-- 添加注释
COMMENT ON TABLE subscription_tokens IS '订阅链接加密令牌';
COMMENT ON TABLE subscription_access_logs IS '订阅链接访问日志';
COMMENT ON VIEW subscription_stats IS '订阅链接访问统计视图';
