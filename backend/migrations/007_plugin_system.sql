-- P14 插件系统 - 数据库迁移

-- 插件配置表
CREATE TABLE IF NOT EXISTS plugin_configs (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255),
    type VARCHAR(50) NOT NULL,  -- notification, statistics, authentication, extension
    config_schema JSONB NOT NULL DEFAULT '{}',
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 插件日志表
CREATE TABLE IF NOT EXISTS plugin_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id VARCHAR(100) NOT NULL REFERENCES plugin_configs(id) ON DELETE CASCADE,
    level VARCHAR(20) NOT NULL,  -- info, warning, error, debug
    message TEXT NOT NULL,
    data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_plugin_logs_plugin_id ON plugin_logs(plugin_id);
CREATE INDEX IF NOT EXISTS idx_plugin_logs_created_at ON plugin_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_plugin_logs_level ON plugin_logs(level);

-- 插入内置插件配置
INSERT INTO plugin_configs (id, name, description, version, author, type, config_schema, enabled) VALUES
(
    'telegram_notification',
    'Telegram 通知',
    '通过 Telegram Bot 发送系统通知、告警和报表',
    '1.0.0',
    'LX-Proxy Team',
    'notification',
    '{
        "type": "object",
        "required": ["bot_token", "chat_id"],
        "properties": {
            "bot_token": {
                "type": "string",
                "title": "Bot Token",
                "description": "从 @BotFather 获取的 Telegram Bot Token"
            },
            "chat_id": {
                "type": "string",
                "title": "Chat ID",
                "description": "接收消息的聊天 ID（用户 ID 或群组 ID）"
            },
            "enabled": {
                "type": "boolean",
                "title": "启用",
                "default": true
            }
        }
    }'::jsonb,
    FALSE
),
(
    'dingtalk_notification',
    '钉钉通知',
    '通过钉钉机器人发送系统通知、告警和报表',
    '1.0.0',
    'LX-Proxy Team',
    'notification',
    '{
        "type": "object",
        "required": ["webhook_url"],
        "properties": {
            "webhook_url": {
                "type": "string",
                "title": "Webhook URL",
                "description": "钉钉机器人 Webhook 地址"
            },
            "secret": {
                "type": "string",
                "title": "加签密钥",
                "description": "可选，用于消息加签"
            },
            "enabled": {
                "type": "boolean",
                "title": "启用",
                "default": true
            }
        }
    }'::jsonb,
    FALSE
),
(
    'advanced_statistics',
    '高级统计',
    '提供增强的流量统计、用户行为分析和报表导出功能',
    '1.0.0',
    'LX-Proxy Team',
    'statistics',
    '{
        "type": "object",
        "properties": {
            "enabled": {
                "type": "boolean",
                "title": "启用",
                "default": true
            },
            "retention_days": {
                "type": "integer",
                "title": "数据保留天数",
                "default": 90,
                "minimum": 7,
                "maximum": 365
            },
            "auto_export": {
                "type": "boolean",
                "title": "自动导出报表",
                "default": false
            }
        }
    }'::jsonb,
    FALSE
)
ON CONFLICT (id) DO NOTHING;

-- 添加注释
COMMENT ON TABLE plugin_configs IS '插件配置表';
COMMENT ON TABLE plugin_logs IS '插件运行日志表';
