# 数据库迁移说明

**数据库：** PostgreSQL 15+  
**迁移工具：** SQLx Migrate

---

## 📋 迁移历史

| 版本 | 文件 | 日期 | 说明 |
|------|------|------|------|
| 001 | `001_init.sql` | 2026-03-06 | 初始数据库结构 |
| 002 | `002_add_email_to_users.sql` | 2026-03-23 | 添加用户邮箱字段 |
| 003 | `003_add_totp_support.sql` | 2026-03-23 | 添加 TOTP 双因素认证支持 |
| 004 | `004_add_email_and_sessions.sql` | 2026-03-23 | 添加邮件日志和会话表 |
| 005 | `005_add_indexes_optimization.sql` | 2026-03-23 | 性能优化索引和物化视图 |

---

## 🚀 运行迁移

### 使用 Cargo

```bash
cd backend

# 运行所有迁移
sqlx migrate run

# 检查迁移状态
sqlx migrate info

# 添加新迁移
sqlx migrate add <migration_name>
```

### 使用 Docker

```bash
# 自动运行迁移（启动时）
docker-compose up -d

# 手动运行迁移
docker-compose exec backend sqlx migrate run
```

### 使用 Makefile

```bash
make migrate
```

---

## 📊 数据库表

### 核心表
- `users` - 用户账户表
- `inbound_configs` - 入站配置表
- `traffic_logs` - 流量日志表

### 配置表
- `system_configs` - 系统配置表
- `xray_configs` - Xray 配置表

### 日志表
- `operation_logs` - 操作日志表
- `email_logs` - 邮件发送日志表

### 安全表
- `user_totp_configs` - TOTP 配置表
- `login_sessions` - 登录会话表

---

## 🔧 数据库管理

### 备份数据库

```bash
# 使用脚本
./scripts/db_manage.sh backup

# 或使用 Makefile
make backup

# 手动备份
pg_dump -U postgres lx_proxy | gzip > backup.sql.gz
```

### 恢复数据库

```bash
# 使用脚本
./scripts/db_manage.sh restore

# 手动恢复
gunzip -c backup.sql.gz | psql -U postgres lx_proxy
```

### 清理过期数据

```bash
# 使用脚本
./scripts/db_manage.sh clean

# 或手动执行
psql -U postgres lx_proxy -c "SELECT cleanup_expired_data();"
```

### 查看统计信息

```bash
# 数据库统计
./scripts/db_manage.sh stats

# 表大小
./scripts/db_manage.sh size

# 连接数
./scripts/db_manage.sh connections
```

### 性能优化

```bash
# 刷新物化视图
./scripts/db_manage.sh refresh-views

# 执行 VACUUM
./scripts/db_manage.sh vacuum

# 执行 ANALYZE
./scripts/db_manage.sh analyze
```

---

## 📈 物化视图

### mv_daily_traffic
每日流量统计视图，按日期和入站 ID 聚合。

### mv_monthly_traffic
每月流量统计视图，按月份和入站 ID 聚合。

### mv_user_activity
用户活动统计视图，包含用户入站数量和流量使用统计。

### 刷新视图

```sql
-- 手动刷新
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_traffic;
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_monthly_traffic;
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_user_activity;

-- 或使用函数
SELECT refresh_traffic_views();
```

---

## 🔐 安全建议

### 1. 修改默认密码

```sql
-- 修改 postgres 用户密码
ALTER USER postgres WITH PASSWORD 'strong-password';

-- 修改应用用户密码
ALTER USER lx_proxy WITH PASSWORD 'strong-password';
```

### 2. 限制远程访问

```sql
-- 编辑 postgresql.conf
ALTER SYSTEM SET listen_addresses = 'localhost';
SELECT pg_reload_conf();
```

### 3. 启用 SSL

```sql
-- 编辑 postgresql.conf
ALTER SYSTEM SET ssl = on;
ALTER SYSTEM SET ssl_cert_file = '/path/to/server.crt';
ALTER SYSTEM SET ssl_key_file = '/path/to/server.key';
SELECT pg_reload_conf();
```

---

## 📝 添加新迁移

```bash
# 创建新迁移
cd backend
sqlx migrate add "add_new_feature"

# 编辑迁移文件
vim migrations/<timestamp>_add_new_feature.sql

# 运行迁移
sqlx migrate run
```

### 迁移文件命名规范

- 使用下划线分隔单词
- 使用动词开头（add_, create_, update_, drop_）
- 描述清晰明确

示例：
- `add_email_to_users.sql`
- `create_notification_table.sql`
- `update_traffic_indexes.sql`

---

## 🔍 故障排查

### 迁移失败

```bash
# 查看迁移状态
sqlx migrate info

# 强制重新运行
sqlx migrate run --ignore-missing
```

### 连接问题

```bash
# 检查数据库是否运行
docker-compose ps db

# 查看数据库日志
docker-compose logs db

# 测试连接
psql -h localhost -p 5432 -U postgres -d lx_proxy
```

### 性能问题

```sql
-- 查看慢查询
SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;

-- 查看锁
SELECT * FROM pg_locks WHERE NOT granted;

-- 查看表大小
SELECT relname, pg_size_pretty(pg_total_relation_size(relid))
FROM pg_statio_user_tables ORDER BY pg_total_relation_size DESC;
```

---

## 📚 参考文档

- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [SQLx 文档](https://docs.rs/sqlx/)
- [数据库设计文档](../../docs/数据库设计文档.md)
- [性能优化指南](../../docs/性能优化指南.md)

---

*最后更新：2026-03-23*
