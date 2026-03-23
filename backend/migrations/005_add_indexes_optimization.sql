-- 性能优化索引

-- 用户表索引优化
CREATE INDEX IF NOT EXISTS idx_users_role_enabled ON users(role, enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- 入站配置表索引优化
CREATE INDEX IF NOT EXISTS idx_inbounds_user_enable ON inbound_configs(user_id, enable) WHERE enable = true;
CREATE INDEX IF NOT EXISTS idx_inbounds_expire ON inbound_configs(expire_at, enable) WHERE enable = true AND expire_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inbounds_traffic_limit ON inbound_configs(traffic_used, traffic_limit) WHERE traffic_limit IS NOT NULL AND enable = true;
CREATE INDEX IF NOT EXISTS idx_inbounds_created_at ON inbound_configs(created_at DESC);

-- 流量日志表索引优化（复合索引）
CREATE INDEX IF NOT EXISTS idx_traffic_inbound_date ON traffic_logs(inbound_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_traffic_date_inbound ON traffic_logs(recorded_at DESC, inbound_id);

-- 操作日志表索引优化
CREATE INDEX IF NOT EXISTS idx_ops_user_date ON operation_logs(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ops_action_date ON operation_logs(action, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ops_resource ON operation_logs(resource, resource_id);

-- TOTP 配置表索引优化
CREATE INDEX IF NOT EXISTS idx_totp_enabled ON user_totp_configs(enabled);

-- 系统配置表索引优化
CREATE INDEX IF NOT EXISTS idx_system_configs_category ON system_configs(category);

-- 创建流量统计物化视图
DROP MATERIALIZED VIEW IF EXISTS mv_daily_traffic;
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_traffic AS
SELECT 
    DATE(recorded_at) as date,
    inbound_id,
    SUM(upload) as total_upload,
    SUM(download) as total_download,
    COUNT(*) as record_count
FROM traffic_logs
GROUP BY DATE(recorded_at), inbound_id
ORDER BY date DESC;

-- 物化视图索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_traffic_date_inbound ON mv_daily_traffic(date DESC, inbound_id);

-- 创建每月流量统计物化视图
DROP MATERIALIZED VIEW IF EXISTS mv_monthly_traffic;
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_monthly_traffic AS
SELECT 
    DATE_TRUNC('month', recorded_at) as month,
    inbound_id,
    SUM(upload) as total_upload,
    SUM(download) as total_download,
    COUNT(*) as record_count
FROM traffic_logs
GROUP BY DATE_TRUNC('month', recorded_at), inbound_id
ORDER BY month DESC;

-- 物化视图索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_monthly_traffic_month_inbound ON mv_monthly_traffic(month DESC, inbound_id);

-- 创建用户活动统计视图
DROP MATERIALIZED VIEW IF EXISTS mv_user_activity;
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_user_activity AS
SELECT 
    u.id as user_id,
    u.username,
    u.role,
    COUNT(DISTINCT ic.id) as inbound_count,
    COUNT(DISTINCT CASE WHEN ic.enable THEN ic.id END) as active_inbound_count,
    COALESCE(SUM(ic.traffic_used), 0) as total_traffic_used,
    MAX(u.last_login_at) as last_login_at
FROM users u
LEFT JOIN inbound_configs ic ON u.id = ic.user_id
GROUP BY u.id, u.username, u.role
ORDER BY total_traffic_used DESC;

-- 物化视图索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_user_activity_user_id ON mv_user_activity(user_id);

-- 刷新物化视图函数
CREATE OR REPLACE FUNCTION refresh_traffic_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_traffic;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_monthly_traffic;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_user_activity;
    RAISE NOTICE '物化视图刷新完成';
END;
$$ LANGUAGE plpgsql;

-- 添加数据库注释
COMMENT ON MATERIALIZED VIEW mv_daily_traffic IS '每日流量统计视图';
COMMENT ON MATERIALIZED VIEW mv_monthly_traffic IS '每月流量统计视图';
COMMENT ON MATERIALIZED VIEW mv_user_activity IS '用户活动统计视图';

-- 创建统计函数
CREATE OR REPLACE FUNCTION get_system_stats()
RETURNS TABLE (
    total_users BIGINT,
    active_users BIGINT,
    total_inbounds BIGINT,
    active_inbounds BIGINT,
    total_traffic_used BIGINT,
    total_upload BIGINT,
    total_download BIGINT,
    today_traffic BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        (SELECT COUNT(*) FROM users) as total_users,
        (SELECT COUNT(*) FROM users WHERE enabled = true) as active_users,
        (SELECT COUNT(*) FROM inbound_configs) as total_inbounds,
        (SELECT COUNT(*) FROM inbound_configs WHERE enable = true) as active_inbounds,
        (SELECT COALESCE(SUM(traffic_used), 0) FROM inbound_configs) as total_traffic_used,
        (SELECT COALESCE(SUM(total_upload), 0) FROM inbound_configs) as total_upload,
        (SELECT COALESCE(SUM(total_download), 0) FROM inbound_configs) as total_download,
        (SELECT COALESCE(SUM(upload + download), 0) FROM traffic_logs WHERE recorded_at >= DATE_TRUNC('day', NOW())) as today_traffic;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器：自动更新 updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为所有需要自动更新 updated_at 的表添加触发器
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_inbound_configs_updated_at ON inbound_configs;
CREATE TRIGGER update_inbound_configs_updated_at
    BEFORE UPDATE ON inbound_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_system_configs_updated_at ON system_configs;
CREATE TRIGGER update_system_configs_updated_at
    BEFORE UPDATE ON system_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_user_totp_configs_updated_at ON user_totp_configs;
CREATE TRIGGER update_user_totp_configs_updated_at
    BEFORE UPDATE ON user_totp_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 分析表以更新统计信息
ANALYZE users;
ANALYZE inbound_configs;
ANALYZE traffic_logs;
ANALYZE system_configs;
ANALYZE operation_logs;
ANALYZE user_totp_configs;
ANALYZE email_logs;
ANALYZE login_sessions;
