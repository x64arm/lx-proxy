#!/bin/bash
# LX-Proxy 数据库管理脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 配置
DB_NAME="${DB_NAME:-lx_proxy}"
DB_USER="${DB_USER:-postgres}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
BACKUP_DIR="./data/backups"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示帮助
show_help() {
    echo "LX-Proxy 数据库管理脚本"
    echo ""
    echo "使用方式：$0 <command>"
    echo ""
    echo "命令列表:"
    echo "  backup          备份数据库"
    echo "  restore         恢复数据库"
    echo "  clean           清理过期数据"
    echo "  stats           显示数据库统计"
    echo "  size            显示表大小"
    echo "  connections     显示连接数"
    echo "  slow-queries    显示慢查询"
    echo "  vacuum          执行 VACUUM"
    echo "  analyze         执行 ANALYZE"
    echo "  refresh-views   刷新物化视图"
    echo "  migrate         运行数据库迁移"
    echo "  reset           重置数据库（危险！）"
    echo ""
}

# 备份数据库
backup_db() {
    log_info "备份数据库 $DB_NAME..."
    
    mkdir -p "$BACKUP_DIR"
    local backup_file="$BACKUP_DIR/${DB_NAME}_$(date +%Y%m%d_%H%M%S).sql.gz"
    
    PGPASSWORD="${DB_PASSWORD:-}" pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" | gzip > "$backup_file"
    
    log_success "备份完成：$backup_file"
    
    # 保留最近 7 个备份
    ls -t "$BACKUP_DIR"/${DB_NAME}_*.sql.gz 2>/dev/null | tail -n +8 | xargs -r rm
    log_info "已清理旧备份，保留最近 7 个"
}

# 恢复数据库
restore_db() {
    log_warning "恢复数据库将覆盖现有数据！"
    read -p "确定要继续吗？(y/n): " confirm
    
    if [ "$confirm" != "y" ]; then
        log_info "已取消"
        return
    fi
    
    echo "选择备份文件："
    ls -lt "$BACKUP_DIR"/${DB_NAME}_*.sql.gz 2>/dev/null | head -10
    
    read -p "输入备份文件名：" file
    
    if [ ! -f "$BACKUP_DIR/$file" ]; then
        log_error "文件不存在：$BACKUP_DIR/$file"
        exit 1
    fi
    
    log_info "恢复数据库..."
    gunzip -c "$BACKUP_DIR/$file" | PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME"
    
    log_success "恢复完成"
}

# 清理过期数据
clean_db() {
    log_info "清理过期数据..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
-- 清理 90 天前的流量日志
DELETE FROM traffic_logs WHERE recorded_at < NOW() - INTERVAL '90 days';

-- 清理 180 天前的操作日志
DELETE FROM operation_logs WHERE created_at < NOW() - INTERVAL '180 days';

-- 清理 90 天前的邮件日志
DELETE FROM email_logs WHERE created_at < NOW() - INTERVAL '90 days';

-- 清理过期会话
DELETE FROM login_sessions WHERE expires_at < NOW();

-- 显示清理后的统计
SELECT 'traffic_logs' as table_name, COUNT(*) as row_count FROM traffic_logs WHERE recorded_at < NOW() - INTERVAL '30 days'
UNION ALL
SELECT 'operation_logs', COUNT(*) FROM operation_logs WHERE created_at < NOW() - INTERVAL '90 days'
UNION ALL
SELECT 'email_logs', COUNT(*) FROM email_logs WHERE created_at < NOW() - INTERVAL '30 days';
EOF
    
    log_success "清理完成"
}

# 显示数据库统计
show_stats() {
    log_info "数据库统计信息..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
\\echo ''
\\echo '=== 表记录数统计 ==='
SELECT 
    schemaname as schema,
    relname as table_name,
    n_live_tup as row_count
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;

\\echo ''
\\echo '=== 流量使用统计 ==='
SELECT 
    '总用户数' as item, COUNT(*)::text as value FROM users
UNION ALL
SELECT '活跃用户', COUNT(*)::text FROM users WHERE enabled = true
UNION ALL
SELECT '入站配置', COUNT(*)::text FROM inbound_configs
UNION ALL
SELECT '启用的入站', COUNT(*)::text FROM inbound_configs WHERE enable = true
UNION ALL
SELECT '总流量使用', pg_size_pretty(COALESCE(SUM(traffic_used), 0)) FROM inbound_configs
UNION ALL
SELECT '今日流量', pg_size_pretty(COALESCE(SUM(upload + download), 0)) FROM traffic_logs WHERE recorded_at >= DATE_TRUNC('day', NOW());

\\echo ''
\\echo '=== 最近登录用户 ==='
SELECT username, role, last_login_at, last_login_ip
FROM users
WHERE last_login_at IS NOT NULL
ORDER BY last_login_at DESC
LIMIT 5;
EOF
}

# 显示表大小
show_size() {
    log_info "表大小统计..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT 
    relname as table_name,
    pg_size_pretty(pg_total_relation_size(relid)) as total_size,
    pg_size_pretty(pg_relation_size(relid)) as data_size,
    pg_size_pretty(pg_total_relation_size(relid) - pg_relation_size(relid)) as index_size
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;
EOF
}

# 显示连接数
show_connections() {
    log_info "数据库连接数..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT 
    count(*) as total_connections,
    count(*) FILTER (WHERE state = 'active') as active,
    count(*) FILTER (WHERE state = 'idle') as idle,
    count(*) FILTER (WHERE state = 'idle in transaction') as idle_in_transaction
FROM pg_stat_activity
WHERE datname = '$DB_NAME';

\\echo ''
\\echo '=== 连接详情 ==='
SELECT 
    pid,
    usename as username,
    state,
    query_start,
    LEFT(query, 50) as query_preview
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
ORDER BY query_start
LIMIT 10;
EOF
}

# 显示慢查询
show_slow_queries() {
    log_info "慢查询统计（>1 秒）..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT 
    calls,
    total_exec_time / 1000 as total_time_ms,
    mean_exec_time / 1000 as mean_time_ms,
    LEFT(query, 100) as query_preview
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = '$DB_NAME')
ORDER BY mean_exec_time DESC
LIMIT 10;
EOF
}

# 执行 VACUUM
vacuum_db() {
    log_info "执行 VACUUM..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "VACUUM VERBOSE;"
    
    log_success "VACUUM 完成"
}

# 执行 ANALYZE
analyze_db() {
    log_info "执行 ANALYZE..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "ANALYZE VERBOSE;"
    
    log_success "ANALYZE 完成"
}

# 刷新物化视图
refresh_views() {
    log_info "刷新物化视图..."
    
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_traffic;
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_monthly_traffic;
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_user_activity;
EOF
    
    log_success "物化视图刷新完成"
}

# 运行数据库迁移
run_migrate() {
    log_info "运行数据库迁移..."
    
    cd backend
    cargo install sqlx-cli --no-default-features --features postgres
    sqlx migrate run
    cd ..
    
    log_success "迁移完成"
}

# 重置数据库
reset_db() {
    log_error "警告：此操作将删除所有数据！"
    read -p "确定要重置数据库吗？(输入 YES 确认): " confirm
    
    if [ "$confirm" != "YES" ]; then
        log_info "已取消"
        return
    fi
    
    log_warning "删除所有表..."
    PGPASSWORD="${DB_PASSWORD:-}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO public;
EOF
    
    log_info "运行迁移..."
    run_migrate
    
    log_success "数据库重置完成"
}

# 主函数
main() {
    case "${1:-help}" in
        backup)
            backup_db
            ;;
        restore)
            restore_db
            ;;
        clean)
            clean_db
            ;;
        stats)
            show_stats
            ;;
        size)
            show_size
            ;;
        connections)
            show_connections
            ;;
        slow-queries)
            show_slow_queries
            ;;
        vacuum)
            vacuum_db
            ;;
        analyze)
            analyze_db
            ;;
        refresh-views)
            refresh_views
            ;;
        migrate)
            run_migrate
            ;;
        reset)
            reset_db
            ;;
        *)
            show_help
            ;;
    esac
}

main "$@"
