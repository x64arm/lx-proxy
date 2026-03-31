-- P15 多节点管理 - 数据库迁移

-- 节点配置表
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    api_url TEXT NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    status VARCHAR(20) DEFAULT 'offline',  -- online, offline, error
    location VARCHAR(100),  -- 节点位置
    version VARCHAR(50),  -- Xray 版本
    cpu_usage REAL DEFAULT 0,
    memory_usage REAL DEFAULT 0,
    disk_usage REAL DEFAULT 0,
    bandwidth_upload BIGINT DEFAULT 0,
    bandwidth_download BIGINT DEFAULT 0,
    connection_count INTEGER DEFAULT 0,
    last_seen TIMESTAMP WITH TIME ZONE,
    is_primary BOOLEAN DEFAULT FALSE,  -- 是否为主节点
    is_active BOOLEAN DEFAULT TRUE,  -- 是否启用
    sync_status VARCHAR(20) DEFAULT 'pending',  -- pending, syncing, synced, failed
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nodes_status ON nodes(status);
CREATE INDEX IF NOT EXISTS idx_nodes_active ON nodes(is_active);
CREATE INDEX IF NOT EXISTS idx_nodes_primary ON nodes(is_primary);
CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON nodes(last_seen);

-- 节点配置同步历史表
CREATE TABLE IF NOT EXISTS node_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    sync_type VARCHAR(50) NOT NULL,  -- full, incremental, config, user
    status VARCHAR(20) NOT NULL,  -- success, failed, partial
    items_synced INTEGER DEFAULT 0,
    error_message TEXT,
    duration_ms INTEGER,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_node_sync_history_node_id ON node_sync_history(node_id);
CREATE INDEX IF NOT EXISTS idx_node_sync_history_status ON node_sync_history(status);
CREATE INDEX IF NOT EXISTS idx_node_sync_history_started_at ON node_sync_history(started_at);

-- 节点健康检查日志表
CREATE TABLE IF NOT EXISTS node_health_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL,  -- healthy, unhealthy, error
    response_time_ms INTEGER,
    cpu_usage REAL,
    memory_usage REAL,
    disk_usage REAL,
    connection_count INTEGER,
    error_message TEXT,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_node_health_logs_node_id ON node_health_logs(node_id);
CREATE INDEX IF NOT EXISTS idx_node_health_logs_checked_at ON node_health_logs(checked_at);

-- 为 inbound_configs 添加节点关联
ALTER TABLE inbound_configs 
ADD COLUMN IF NOT EXISTS node_id UUID REFERENCES nodes(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS sync_status VARCHAR(20) DEFAULT 'pending';

CREATE INDEX IF NOT EXISTS idx_inbound_configs_node_id ON inbound_configs(node_id);

-- 节点统计视图
CREATE OR REPLACE VIEW node_stats AS
SELECT 
    n.id,
    n.name,
    n.status,
    n.is_primary,
    n.is_active,
    COUNT(DISTINCT ic.id) as inbound_count,
    COUNT(DISTINCT CASE WHEN ic.enable THEN ic.id END) as enabled_inbound_count,
    COALESCE(SUM(ic.traffic_used), 0) as total_traffic_used,
    COALESCE(SUM(ic.connection_count), 0) as total_connections,
    n.last_seen,
    n.last_sync_at
FROM nodes n
LEFT JOIN inbound_configs ic ON n.id = ic.node_id
GROUP BY n.id, n.name, n.status, n.is_primary, n.is_active, n.last_seen, n.last_sync_at;

-- 添加注释
COMMENT ON TABLE nodes IS 'Xray 节点配置表';
COMMENT ON TABLE node_sync_history IS '节点配置同步历史';
COMMENT ON TABLE node_health_logs IS '节点健康检查日志';
COMMENT ON VIEW node_stats IS '节点统计视图';
