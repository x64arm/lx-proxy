# LX-Proxy Docker 部署指南

本文档详细介绍如何使用 Docker 部署 LX-Proxy 代理管理面板。

## 📋 目录

- [快速开始](#快速开始)
- [环境配置](#环境配置)
- [服务管理](#服务管理)
- [高级配置](#高级配置)
- [备份与恢复](#备份与恢复)
- [故障排查](#故障排查)

---

## 🚀 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy
```

### 2. 初始化环境

```bash
# 使用 Makefile 初始化（推荐）
make setup

# 或手动初始化
cp .env.example .env
cp backend/.env.example backend/.env
mkdir -p data/backups data/postgres logs
```

### 3. 配置环境变量

编辑 `.env` 文件，**务必修改以下配置**：

```bash
# ⚠️ 生产环境必须修改！
JWT_SECRET=your_super_secret_key_here

# 数据库密码（建议修改）
DB_PASSWORD=your_secure_password
```

### 4. 启动服务

```bash
# 启动基本服务（前端 + 后端 + 数据库）
make up

# 或直接用 docker-compose
docker-compose up -d
```

### 5. 访问面板

打开浏览器访问：http://localhost

**默认账号：**
- 用户名：`admin`
- 密码：`admin123`

⚠️ **首次登录后请立即修改密码！**

---

## ⚙️ 环境配置

### 环境变量说明

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `JWT_SECRET` | JWT 密钥（生产环境必须修改） | `change_this_secret_key_in_production` |
| `DB_PASSWORD` | 数据库密码 | `lxproxy2026_secure_password` |
| `RUST_LOG` | 后端日志级别 | `info` |
| `FRONTEND_PORT` | 前端服务端口 | `80` |
| `BACKEND_PORT` | 后端服务端口 | `3000` |
| `POSTGRES_VERSION` | PostgreSQL 版本 | `15` |
| `ENABLE_XRAY` | 是否启用 Xray | `false` |

### 生成安全的 JWT 密钥

```bash
# 方法 1：使用 openssl
openssl rand -hex 32

# 方法 2：使用 head
head -c 32 /dev/urandom | base64

# 方法 3：使用 python
python3 -c "import secrets; print(secrets.token_hex(32))"
```

---

## 🛠️ 服务管理

### 使用 Makefile（推荐）

```bash
# 查看所有命令
make help

# 启动服务
make up

# 停止服务
make down

# 重启服务
make restart

# 查看日志
make logs

# 查看服务状态
make ps

# 构建镜像
make build

# 清理容器（保留数据）
make clean-containers

# 完全清理（删除所有数据）
make clean
```

### 使用 Docker Compose

```bash
# 启动
docker-compose up -d

# 停止
docker-compose down

# 重启
docker-compose restart

# 查看日志
docker-compose logs -f

# 查看状态
docker-compose ps

# 重新构建
docker-compose build --no-cache
```

### 启动可选服务

**启用 Xray 核心：**

```bash
# 使用 profile 启动
docker-compose --profile xray up -d

# 或 Makefile
make up-xray
```

**启用自动备份：**

```bash
docker-compose --profile backup up -d
```

**启用 Nginx Proxy Manager（HTTPS 管理）：**

```bash
docker-compose --profile proxy up -d
```

---

## 🔧 高级配置

### 1. HTTPS 配置

**方法一：使用 Nginx Proxy Manager（推荐）**

```bash
# 启动 Nginx Proxy Manager
docker-compose --profile proxy up -d

# 访问管理界面：http://localhost:81
# 默认账号：admin@example.com / changelog
```

在 Nginx Proxy Manager 中：
1. 添加 Proxy Host
2. 配置域名和 SSL 证书
3. 转发到 `frontend:80`

**方法二：手动配置 HTTPS**

编辑 `frontend/nginx.conf`，取消注释 HTTPS 配置段，然后：

```bash
# 使用 Let's Encrypt 获取证书
docker run --rm \
  -v ./data/letsencrypt:/etc/letsencrypt \
  certbot/certbot certonly --standalone -d your-domain.com
```

### 2. 数据库持久化

数据默认存储在 `./data/postgres` 目录：

```bash
# 查看数据目录
ls -la ./data/postgres

# 备份数据目录
tar -czf postgres_backup.tar.gz ./data/postgres
```

### 3. 日志管理

日志文件位置：

```
logs/
├── backend/    # 后端日志
├── frontend/   # Nginx 访问日志
├── nginx/      # Nginx 错误日志
└── xray/       # Xray 日志
```

日志轮转配置（每个服务）：
- 单文件最大：50MB
- 最大文件数：5 个

### 4. 网络配置

默认网络配置：
- 网络名称：`lx-proxy-network`
- 子网：`172.28.0.0/16`

如需修改，编辑 `docker-compose.yml` 中的网络配置。

---

## 💾 备份与恢复

### 手动备份

```bash
# 使用 Makefile
make backup

# 或手动执行
docker-compose exec db pg_dump -U lx_proxy lx_proxy | gzip > backup.sql.gz
```

### 自动备份

启用备份服务：

```bash
docker-compose --profile backup up -d
```

备份配置（在 `.env` 中）：

```bash
ENABLE_BACKUP=true
BACKUP_CRON=0 3 * * *  # 每天凌晨 3 点
BACKUP_RETENTION_DAYS=7  # 保留 7 天
```

备份文件位置：`./data/backups/`

### 恢复数据库

```bash
# 使用 Makefile（指定备份文件）
make restore FILE=./data/backups/lx_proxy_20260322_120000.sql.gz

# 或手动执行
gunzip -c backup.sql.gz | docker-compose exec -T db psql -U lx_proxy -d lx_proxy
```

### 重置管理员密码

```bash
# 使用 Makefile
make reset-admin

# 或手动执行
docker-compose exec db psql -U lx_proxy -d lx_proxy -c \
  "UPDATE users SET password_hash = '...' WHERE username = 'admin';"
```

---

## 🔍 故障排查

### 容器无法启动

```bash
# 查看容器状态
docker-compose ps

# 查看详细日志
docker-compose logs backend
docker-compose logs db

# 检查端口占用
netstat -tlnp | grep :80
netstat -tlnp | grep :3000
```

### 数据库连接失败

```bash
# 检查数据库是否就绪
docker-compose exec db pg_isready -U lx_proxy -d lx_proxy

# 查看数据库日志
docker-compose logs db

# 测试连接
docker-compose exec backend curl http://localhost:3000/api/health
```

### 前端页面空白

```bash
# 检查前端容器
docker-compose logs frontend

# 检查 Nginx 配置
docker-compose exec frontend nginx -t

# 检查 API 连接
curl http://localhost/api/health
```

### 内存不足

```bash
# 查看资源使用
docker stats

# 限制容器内存（编辑 docker-compose.yml）
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 512M
```

### 数据卷权限问题

```bash
# 修复权限
sudo chown -R 999:999 ./data/postgres

# 或重建数据卷
docker-compose down -v
docker-compose up -d
```

---

## 📊 监控与维护

### 查看系统状态

```bash
make status
```

### 定期维护

```bash
# 清理未使用的镜像
docker image prune -f

# 清理未使用的卷
docker volume prune -f

# 查看磁盘使用
docker system df
```

### 更新 LX-Proxy

```bash
# 使用 Makefile
make update

# 或手动更新
git pull
docker-compose build --no-cache
docker-compose down
docker-compose up -d
```

---

## 🆘 常见问题

### Q1: 修改配置后如何生效？

```bash
# 重启相关服务
docker-compose restart backend frontend

# 或重新构建
docker-compose up -d --build
```

### Q2: 如何查看实时日志？

```bash
# 所有服务日志
docker-compose logs -f

# 单个服务日志
docker-compose logs -f backend
```

### Q3: 如何进入容器内部？

```bash
# 后端容器
docker-compose exec backend sh

# 数据库容器
docker-compose exec db sh

# 前端容器
docker-compose exec frontend sh
```

### Q4: 如何重置所有数据？

⚠️ **警告：此操作会删除所有数据！**

```bash
make clean
make setup
make up
```

---

## 📞 技术支持

- GitHub Issues: https://github.com/x64arm/lx-proxy/issues
- 项目文档：https://github.com/x64arm/lx-proxy/tree/main/docs

---

*最后更新：2026-03-22*
