-- P19 高可用集群支持
-- 创建时间：2026-04-02

-- 集群节点表
CREATE TABLE IF NOT EXISTS cluster_nodes (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address INET NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'follower',
    status VARCHAR(50) NOT NULL DEFAULT 'starting',
    load_avg REAL DEFAULT 0,
    last_heartbeat TIMESTAMPTZ,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON TABLE cluster_nodes IS '集群节点注册表';
COMMENT ON COLUMN cluster_nodes.id IS '节点唯一标识';
COMMENT ON COLUMN cluster_nodes.name IS '节点名称（用户友好）';
COMMENT ON COLUMN cluster_nodes.address IS '节点地址（IP:Port）';
COMMENT ON COLUMN cluster_nodes.role IS '节点角色（leader/follower/candidate）';
COMMENT ON COLUMN cluster_nodes.status IS '节点状态（starting/running/stopping/unreachable）';
COMMENT ON COLUMN cluster_nodes.load_avg IS '节点平均负载（0.0-1.0）';
COMMENT ON COLUMN cluster_nodes.last_heartbeat IS '最后心跳时间';
COMMENT ON COLUMN cluster_nodes.started_at IS '节点启动时间';

CREATE INDEX IF NOT EXISTS idx_cluster_nodes_status ON cluster_nodes(status);
CREATE INDEX IF NOT EXISTS idx_cluster_nodes_role ON cluster_nodes(role);
CREATE INDEX IF NOT EXISTS idx_cluster_nodes_heartbeat ON cluster_nodes(last_heartbeat);

-- 配置变更日志表
CREATE TABLE IF NOT EXISTS config_changes (
    id BIGSERIAL PRIMARY KEY,
    version BIGINT NOT NULL,
    config_type VARCHAR(100) NOT NULL,
    config_data JSONB NOT NULL,
    changed_by UUID REFERENCES users(id),
    node_id UUID REFERENCES cluster_nodes(id),
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    checksum VARCHAR(64) NOT NULL,
    applied BOOLEAN DEFAULT false
);

COMMENT ON TABLE config_changes IS '配置变更日志表';
COMMENT ON COLUMN config_changes.version IS '配置版本号';
COMMENT ON COLUMN config_changes.config_type IS '配置类型';
COMMENT ON COLUMN config_changes.config_data IS '配置数据（JSON）';
COMMENT ON COLUMN config_changes.changed_by IS '变更发起用户';
COMMENT ON COLUMN config_changes.node_id IS '变更发起节点';
COMMENT ON COLUMN config_changes.checksum IS '配置数据校验和';
COMMENT ON COLUMN config_changes.applied IS '是否已应用';

CREATE UNIQUE INDEX IF NOT EXISTS idx_config_version_type ON config_changes(config_type, version);
CREATE INDEX IF NOT EXISTS idx_config_applied ON config_changes(applied) WHERE NOT applied;
CREATE INDEX IF NOT EXISTS idx_config_changed_at ON config_changes(changed_at);

-- 集群事件日志表
CREATE TABLE IF NOT EXISTS cluster_events (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    node_id UUID REFERENCES cluster_nodes(id),
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON TABLE cluster_events IS '集群事件日志表';
COMMENT ON COLUMN cluster_events.event_type IS '事件类型';
COMMENT ON COLUMN cluster_events.node_id IS '相关节点 ID';
COMMENT ON COLUMN cluster_events.details IS '事件详情（JSON）';

CREATE INDEX IF NOT EXISTS idx_cluster_events_type ON cluster_events(event_type);
CREATE INDEX IF NOT EXISTS idx_cluster_events_node ON cluster_events(node_id);
CREATE INDEX IF NOT EXISTS idx_cluster_events_created ON cluster_events(created_at);

-- 集群健康检查记录表
CREATE TABLE IF NOT EXISTS cluster_health_checks (
    id BIGSERIAL PRIMARY KEY,
    node_id UUID REFERENCES cluster_nodes(id),
    healthy BOOLEAN NOT NULL,
    etcd_healthy BOOLEAN NOT NULL,
    db_healthy BOOLEAN NOT NULL,
    xray_healthy BOOLEAN NOT NULL,
    response_time_ms BIGINT NOT NULL,
    error_message TEXT,
    checked_at TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON TABLE cluster_health_checks IS '集群健康检查记录表';
COMMENT ON COLUMN cluster_health_checks.node_id IS '节点 ID';
COMMENT ON COLUMN cluster_health_checks.healthy IS '是否健康';
COMMENT ON COLUMN cluster_health_checks.etcd_healthy IS 'etcd 连接状态';
COMMENT ON COLUMN cluster_health_checks.db_healthy IS '数据库连接状态';
COMMENT ON COLUMN cluster_health_checks.xray_healthy IS 'Xray 服务状态';
COMMENT ON COLUMN cluster_health_checks.response_time_ms IS '响应时间（毫秒）';
COMMENT ON COLUMN cluster_health_checks.error_message IS '错误信息';

CREATE INDEX IF NOT EXISTS idx_health_checks_node ON cluster_health_checks(node_id);
CREATE INDEX IF NOT EXISTS idx_health_checks_healthy ON cluster_health_checks(healthy);
CREATE INDEX IF NOT EXISTS idx_health_checks_created ON cluster_health_checks(checked_at);

-- 插入初始集群事件
INSERT INTO cluster_events (event_type, details)
VALUES (
    'cluster_init',
    jsonb_build_object(
        'message',
        'P19 cluster support initialized',
        'version',
        '0.1.0',
        'timestamp',
        NOW()
    )
);

-- 添加函数：自动更新 updated_at 时间戳
CREATE OR REPLACE FUNCTION update_cluster_nodes_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
DROP TRIGGER IF EXISTS trigger_update_cluster_nodes_updated_at ON cluster_nodes;
CREATE TRIGGER trigger_update_cluster_nodes_updated_at
    BEFORE UPDATE ON cluster_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_cluster_nodes_updated_at();
