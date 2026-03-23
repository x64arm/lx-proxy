# LX-Proxy Makefile
# 简化管理命令

.PHONY: help setup up down logs backup update clean test build

# 默认目标
help:
	@echo "LX-Proxy 管理命令"
	@echo ""
	@echo "使用方式：make [command]"
	@echo ""
	@echo "命令列表:"
	@echo "  help       显示帮助信息"
	@echo "  setup      初始化环境（首次运行）"
	@echo "  up         启动所有服务"
	@echo "  down       停止所有服务"
	@echo "  restart    重启所有服务"
	@echo "  logs       查看所有日志"
	@echo "  logs-backend  查看后端日志"
	@echo "  logs-frontend 查看前端日志"
	@echo "  logs-db    查看数据库日志"
	@echo "  ps         查看服务状态"
	@echo "  backup     备份数据库"
	@echo "  restore    恢复数据库"
	@echo "  update     更新到最新版本"
	@echo "  clean      清理临时文件"
	@echo "  test       运行测试"
	@echo "  build      构建 Docker 镜像"
	@echo "  deploy     生产环境部署"
	@echo "  reset-admin 重置管理员密码"

# 初始化环境
setup:
	@echo "初始化环境..."
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "✅ .env 已创建，请编辑配置"; \
	else \
		echo "⚠️  .env 已存在"; \
	fi
	@if [ ! -f backend/.env ]; then \
		cp backend/.env.example backend/.env; \
		echo "✅ backend/.env 已创建"; \
	else \
		echo "⚠️  backend/.env 已存在"; \
	fi
	@echo "✅ 环境初始化完成"

# 启动服务
up:
	@echo "启动服务..."
	docker-compose up -d
	@echo "✅ 服务已启动"
	@echo "📌 访问：http://localhost:8080"

# 启动服务（带 Xray）
up-xray:
	docker-compose --profile xray up -d
	@echo "✅ 服务已启动（包含 Xray）"

# 启动服务（带自动备份）
up-backup:
	docker-compose --profile backup up -d
	@echo "✅ 服务已启动（包含自动备份）"

# 停止服务
down:
	@echo "停止服务..."
	docker-compose down
	@echo "✅ 服务已停止"

# 重启服务
restart:
	docker-compose restart

# 查看所有日志
logs:
	docker-compose logs -f

# 查看后端日志
logs-backend:
	docker-compose logs -f backend

# 查看前端日志
logs-frontend:
	docker-compose logs -f frontend

# 查看数据库日志
logs-db:
	docker-compose logs -f db

# 查看服务状态
ps:
	docker-compose ps

# 备份数据库
backup:
	@echo "备份数据库..."
	@mkdir -p ./data/backups
	@docker-compose exec -T db pg_dump -U postgres lx_proxy | gzip > ./data/backups/lx_proxy_$$(date +%Y%m%d_%H%M%S).sql.gz
	@echo "✅ 备份完成：./data/backups/lx_proxy_$$(date +%Y%m%d_%H%M%S).sql.gz"

# 恢复数据库
restore:
	@echo "选择最近的备份文件："
	@ls -lt ./data/backups/*.sql.gz | head -10
	@read -p "输入备份文件名：" file; \
	if [ -f "./data/backups/$$file" ]; then \
		echo "恢复数据库..."; \
		gunzip -c "./data/backups/$$file" | docker-compose exec -T db psql -U postgres lx_proxy; \
		echo "✅ 恢复完成"; \
	else \
		echo "❌ 文件不存在"; \
	fi

# 更新到最新版本
update:
	@echo "更新 LX-Proxy..."
	git pull
	docker-compose down
	docker-compose build --no-cache
	docker-compose up -d
	@echo "✅ 更新完成"

# 清理临时文件
clean:
	@echo "清理临时文件..."
	docker-compose down -v
	docker system prune -f
	@echo "✅ 清理完成"

# 运行测试
test:
	@echo "运行后端测试..."
	cd backend && cargo test
	@echo "✅ 测试完成"

# 构建 Docker 镜像
build:
	@echo "构建 Docker 镜像..."
	docker-compose build
	@echo "✅ 构建完成"

# 生产环境部署
deploy:
	@echo "生产环境部署..."
	@chmod +x ./scripts/deploy.sh
	./scripts/deploy.sh

# 重置管理员密码
reset-admin:
	@echo "重置管理员密码..."
	@read -p "输入新密码：" password; \
	docker-compose exec backend \
		psql -U postgres -d lx_proxy -c \
		"UPDATE users SET password_hash = '$$(echo -n $$password | sha256sum | cut -d' ' -f1)', updated_at = NOW() WHERE username = 'admin';"
	@echo "✅ 密码已重置"

# 运行数据库迁移
migrate:
	@echo "运行数据库迁移..."
	docker-compose exec backend sqlx migrate run
	@echo "✅ 迁移完成"

# 检查服务健康状态
health:
	@echo "检查服务健康状态..."
	@curl -f http://localhost:8080/health || echo "❌ 后端服务异常"
	@echo "✅ 健康检查完成"

# 查看资源使用
stats:
	docker stats --no-stream

# 进入后端容器
shell-backend:
	docker-compose exec backend bash

# 进入数据库容器
shell-db:
	docker-compose exec db bash

# 查看数据库连接
db-connect:
	docker-compose exec db psql -U postgres lx_proxy
