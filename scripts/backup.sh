#!/bin/sh
# LX-Proxy 数据库备份脚本

set -e

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/lx_proxy_${DATE}.sql.gz"
RETENTION_DAYS=${BACKUP_RETENTION_DAYS:-7}

echo "[$(date)] 开始备份数据库..."

# 创建备份
pg_dump -h db -U "${POSTGRES_USER}" "${POSTGRES_DB}" | gzip > "${BACKUP_FILE}"

if [ $? -eq 0 ]; then
    echo "[$(date)] 备份成功：${BACKUP_FILE}"
    ls -lh "${BACKUP_FILE}"
else
    echo "[$(date)] 备份失败！" >&2
    exit 1
fi

# 清理旧备份
echo "[$(date)] 清理 ${RETENTION_DAYS} 天前的备份..."
find "${BACKUP_DIR}" -name "lx_proxy_*.sql.gz" -type f -mtime +${RETENTION_DAYS} -delete

echo "[$(date)] 备份完成"
