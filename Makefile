# LX-Proxy Docker Makefile
# 简化 Docker 部署和管理操作

.PHONY: help build up down restart logs clean backup restore

# 默认目标
help:
	@echo "LX-Proxy Docker 管理命令"
	@echo ""
	@echo "用法：make [命令]"
	@echo ""
	@echo "命令:"
	@echo "  build       构建所有 Docker 镜像"
	@echo "  up          启动所有服务"
	@echo "  down        停止所有服务"
	@echo "  restart     重启所有服务"
	@echo "  logs        查看日志"
	@echo "  logs-backend  查看后端日志"
	@echo "  logs-frontend 查看前端日志"
	@echo "  logs-db     查看数据库日志"
	@echo "  ps          查看服务状态"
	@echo "  clean       清理所有容器和数据卷（危险！）"
	@echo "  backup      备份数据库"
	@echo "  restore     恢复数据库"
	@echo "  shell-backend 进入后端容器"
	@echo "  shell-db    进入数据库容器"
	@echo "  status      查看系统状态"
	@echo "  setup       初始化部署环境"

# 构建镜像
build:
	docker-compose build --no-cache

build-quick:
	docker-compose build

# 启动服务
up:
	docker-compose up -d

# 启动服务（带 Xray）
up-xray:
	docker-compose --profile xray up -d

# 启动服务（带备份）
up-backup:
	docker-compose --profile backup up -d

# 启动所有服务
up-all:
	docker-compose --profile xray --profile backup up -d

# 停止服务
down:
	docker-compose down

# 重启服务
restart:
	docker-compose restart

# 查看日志
logs:
	docker-compose logs -f

logs-backend:
	docker-compose logs -f backend

logs-frontend:
	docker-compose logs -f frontend

logs-db:
	docker-compose logs -f db

# 查看服务状态
ps:
	docker-compose ps

# 清理（删除容器和数据卷）
clean:
	@echo "警告：此操作将删除所有容器和数据卷！"
	@echo "按 Ctrl+C 取消，或等待 5 秒后继续..."
	@sleep 5
	docker-compose down -v
	docker system prune -f

# 只清理容器，保留数据卷
clean-containers:
	docker-compose down

# 备份数据库
backup:
	@echo "正在备份数据库..."
	@mkdir -p ./data/backups
	docker-compose exec -T db pg_dump -U lx_proxy lx_proxy | gzip > ./data/backups/lx_proxy_$$(date +%Y%m%d_%H%M%S).sql.gz
	@echo "备份完成！"
	@ls -lh ./data/backups/

# 恢复数据库（需要指定备份文件）
restore:
	@if [ -z "$(FILE)" ]; then \
		echo "错误：请指定备份文件，例如：make restore FILE=./data/backups/lx_proxy_20260322_120000.sql.gz"; \
		exit 1; \
	fi
	@echo "正在恢复数据库：$(FILE)"
	gunzip -c $(FILE) | docker-compose exec -T db psql -U lx_proxy -d lx_proxy
	@echo "恢复完成！"

# 进入后端容器
shell-backend:
	docker-compose exec backend sh

# 进入前端容器
shell-frontend:
	docker-compose exec frontend sh

# 进入数据库容器
shell-db:
	docker-compose exec db sh

# 进入数据库 psql
psql:
	docker-compose exec db psql -U lx_proxy -d lx_proxy

# 查看系统状态
status:
	@echo "=== Docker 容器状态 ==="
	docker-compose ps
	@echo ""
	@echo "=== 磁盘使用情况 ==="
	du -sh ./data/* 2>/dev/null || echo "无数据目录"
	@echo ""
	@echo "=== 最近日志 ==="
	docker-compose logs --tail=20 backend

# 初始化部署环境
setup:
	@echo "初始化 LX-Proxy 部署环境..."
	@cp -n .env.example .env || true
	@cp -n backend/.env.example backend/.env || true
	@mkdir -p ./data/backups
	@mkdir -p ./data/postgres
	@mkdir -p ./data/nginx-proxy
	@mkdir -p ./data/letsencrypt
	@mkdir -p ./logs/backend
	@mkdir -p ./logs/frontend
	@mkdir -p ./logs/nginx
	@mkdir -p ./logs/xray
	@echo "环境初始化完成！"
	@echo ""
	@echo "下一步："
	@echo "1. 编辑 .env 文件，修改 JWT_SECRET 等配置"
	@echo "2. 运行 'make up' 启动服务"
	@echo "3. 访问 http://localhost"

# 更新服务
update:
	@echo "正在更新 LX-Proxy..."
	git pull
	docker-compose build --no-cache
	docker-compose down
	docker-compose up -d
	@echo "更新完成！"

# 重置管理员密码
reset-admin:
	@echo "重置管理员密码..."
	@docker-compose exec db psql -U lx_proxy -d lx_proxy -c \
		"UPDATE users SET password_hash = '\$$argon2id\$$v=19\$$m=19456,t=2,p=1\$$YWJjZGVmZ2hpams\$$abcdefghijklmnopqrstuvwxyz1234567890' WHERE username = 'admin';"
	@echo "密码已重置为：admin123"
