# LX-Proxy

🚀 基于 Rust 的 Xray 代理管理面板（参考 3x-ui）

[![License](https://img.shields.io/github/license/x64arm/lx-proxy)](https://github.com/x64arm/lx-proxy/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue)](https://www.docker.com)

## 技术栈

- **后端：** Rust + Axum + SQLx + PostgreSQL
- **前端：** Vue 3 + TypeScript + Vite + Element Plus
- **认证：** JWT + Argon2 密码哈希
- **代理：** Xray-core

## 功能特性

- 👥 **用户管理** - 多用户支持、角色权限
- 📡 **协议支持** - Vmess, Vless, Trojan, Shadowsocks
- 📊 **流量统计** - 实时流量监控、历史记录、ECharts 图表
- ⏱️ **流量限制** - 流量上限、到期时间
- 🔐 **IP 限制** - 基于 IP 的访问控制
- 📈 **系统监控** - CPU、内存、Xray 状态
- 🔗 **订阅链接** - 一键生成订阅链接、二维码
- ⚙️ **系统设置** - 可视化配置管理

## 快速开始

### 🐳 Docker 部署（推荐）

#### 一键部署

```bash
# 1. 克隆项目
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy

# 2. 初始化环境
make setup

# 3. 启动服务
make up

# 4. 访问面板
# 打开浏览器访问：http://localhost
```

#### 详细步骤

**1. 配置环境变量**

```bash
# 复制配置文件
cp .env.example .env
cp backend/.env.example backend/.env

# 编辑 .env，务必修改 JWT_SECRET
vim .env
```

**2. 启动服务**

```bash
# 基本服务（前端 + 后端 + 数据库）
docker-compose up -d

# 带 Xray 核心
docker-compose --profile xray up -d

# 带自动备份
docker-compose --profile backup up -d
```

**3. 访问面板**

- 地址：http://localhost
- 用户名：`admin`
- 密码：`admin123`

⚠️ **首次登录后请立即修改密码！**

**4. 常用命令**

```bash
make help      # 查看所有命令
make logs      # 查看日志
make ps        # 查看状态
make down      # 停止服务
make backup    # 备份数据库
make update    # 更新 LX-Proxy
```

### 💻 手动部署

详见 [Docker 部署指南](docs/Docker 部署指南.md)

## 项目结构

```
lx-proxy/
├── backend/              # Rust 后端
│   ├── src/
│   │   ├── main.rs       # 入口
│   │   ├── handlers.rs   # API 处理器
│   │   ├── handlers/
│   │   │   └── traffic.rs # 流量统计模块
│   │   ├── models.rs     # 数据模型
│   │   ├── db.rs         # 数据库连接
│   │   ├── auth.rs       # JWT 认证
│   │   └── xray.rs       # Xray 配置
│   ├── migrations/       # 数据库迁移
│   ├── Dockerfile
│   └── Cargo.toml
├── frontend/             # Vue 3 前端
│   ├── src/
│   │   ├── views/        # 页面组件
│   │   ├── components/   # 通用组件
│   │   ├── api/          # API 封装
│   │   └── main.ts       # 入口
│   ├── nginx.conf        # Nginx 配置
│   ├── Dockerfile
│   └── package.json
├── docs/                 # 文档
│   ├── 开发日志.md
│   ├── 开发计划.md
│   ├── 部署指南.md
│   └── Docker 部署指南.md
├── scripts/              # 脚本
│   └── backup.sh         # 数据库备份
├── docker-compose.yml    # Docker 编排
├── .env.example          # 环境变量示例
├── Makefile              # 构建脚本
└── README.md
```

## API 文档

### 认证

- `POST /api/auth/login` - 用户登录
- `POST /api/auth/logout` - 用户登出
- `GET /api/auth/me` - 获取当前用户

### 用户管理

- `GET /api/users` - 用户列表
- `POST /api/users` - 创建用户
- `GET /api/users/{id}` - 获取用户
- `PUT /api/users/{id}` - 更新用户
- `DELETE /api/users/{id}` - 删除用户

### 入站配置

- `GET /api/inbounds` - 入站列表
- `POST /api/inbounds` - 创建入站
- `GET /api/inbounds/{id}` - 获取入站
- `PUT /api/inbounds/{id}` - 更新入站
- `DELETE /api/inbounds/{id}` - 删除入站
- `POST /api/inbounds/{id}/reset` - 重置流量
- `GET /api/inbounds/{id}/links` - 订阅链接

### 流量统计

- `GET /api/traffic` - 流量统计（支持筛选）
- `GET /api/traffic/{inbound_id}` - 入站流量
- `GET /api/traffic/summary` - 流量汇总
- `POST /api/traffic/log` - 记录流量日志

### 系统

- `GET /api/stats` - 统计数据
- `GET /api/system/status` - 系统状态
- `GET /api/config` - 系统配置
- `PUT /api/config` - 更新配置

## 开发

### 后端开发

```bash
cd backend

# 运行
cargo run

# 测试
cargo test

# 编译
cargo build --release
```

### 前端开发

```bash
cd frontend

# 安装依赖
npm install

# 开发模式
npm run dev

# 生产构建
npm run build
```

## 文档

- [开发日志](docs/开发日志.md) - 开发过程记录
- [开发计划](docs/开发计划.md) - 功能规划
- [部署指南](docs/部署指南.md) - 手动部署教程
- [Docker 部署指南](docs/Docker 部署指南.md) - Docker 部署教程

## 常见问题

### Q: 如何修改默认密码？

登录后在系统设置页面修改，或使用命令重置：

```bash
make reset-admin
```

### Q: 如何备份数据库？

```bash
make backup
```

备份文件保存在 `./data/backups/`

### Q: 如何查看日志？

```bash
# 所有服务日志
make logs

# 单个服务日志
make logs-backend
make logs-frontend
make logs-db
```

### Q: 如何更新 LX-Proxy？

```bash
make update
```

### Q: 如何启用 HTTPS？

详见 [Docker 部署指南](docs/Docker 部署指南.md#1-https-配置)

## 安全建议

1. ⚠️ **修改 JWT_SECRET** - 生产环境必须修改
2. 🔒 **修改默认密码** - 首次登录后立即修改
3. 🛡️ **启用 HTTPS** - 使用反向代理配置 SSL
4. 🔑 **定期备份** - 启用自动备份服务
5. 📝 **监控日志** - 定期检查异常访问

## 技术细节

- **密码哈希：** Argon2id（2015 年密码哈希竞赛获胜者）
- **JWT 令牌：** HS256 算法，可配置过期时间
- **数据库：** PostgreSQL 15+，支持 ACID
- **API 框架：** Axum 0.8（Tokio 生态）
- **UI 框架：** Element Plus（Vue 3 组件库）
- **图表库：** ECharts 5（百度开源）

## License

MIT License

## 致谢

- [3x-ui](https://github.com/vaxilu/x-ui) - 灵感来源
- [Xray-core](https://github.com/XTLS/Xray-core) - 代理核心
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [Element Plus](https://element-plus.org) - Vue 3 组件库
- [ECharts](https://echarts.apache.org) - 图表库

---

**🎉 感谢使用 LX-Proxy！**

如有问题，请提交 [Issue](https://github.com/x64arm/lx-proxy/issues)

*最后更新：2026-03-22*
