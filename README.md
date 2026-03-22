# LX-Proxy

🚀 基于 Rust 的 Xray 代理管理面板（参考 3x-ui）

## 技术栈

- **后端：** Rust + Axum + SQLx + PostgreSQL
- **前端：** Vue 3 + TypeScript + Vite
- **认证：** JWT + Argon2 密码哈希
- **代理：** Xray-core

## 功能特性

- 👥 **用户管理** - 多用户支持、角色权限
- 📡 **协议支持** - Vmess, Vless, Trojan, Shadowsocks
- 📊 **流量统计** - 实时流量监控、历史记录
- ⏱️ **流量限制** - 流量上限、到期时间
- 🔐 **IP 限制** - 基于 IP 的访问控制
- 📈 **系统监控** - CPU、内存、Xray 状态
- 🔗 **订阅链接** - 一键生成订阅链接

## 快速开始

### 方式一：Docker 部署（推荐）

```bash
# 克隆项目
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy

# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 访问 http://localhost
```

### 方式二：手动部署

#### 环境要求

- Rust 1.70+
- PostgreSQL 14+
- Node.js 18+
- Xray-core

#### 后端启动

```bash
cd backend

# 复制环境配置
cp .env.example .env

# 编辑 .env 配置数据库连接

# 编译运行
cargo run
```

#### 前端启动

```bash
cd frontend

# 安装依赖
npm install

# 开发模式
npm run dev

# 生产构建
npm run build
```

#### 数据库初始化

```bash
# 创建数据库
createdb lx_proxy

# 创建用户
createuser lx_proxy
psql -c "ALTER USER lx_proxy WITH PASSWORD 'lxproxy2026';"

# 授予权限
psql -c "GRANT ALL PRIVILEGES ON DATABASE lx_proxy TO lx_proxy;"

# 运行初始化脚本
psql -d lx_proxy -f init.sql
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

### 系统

- `GET /api/stats` - 统计数据
- `GET /api/system/status` - 系统状态
- `GET /api/config` - 系统配置

## 项目结构

```
lx-proxy/
├── backend/
│   ├── src/
│   │   ├── main.rs       # 入口
│   │   ├── handlers.rs   # API 处理器
│   │   ├── models.rs     # 数据模型
│   │   ├── db.rs         # 数据库连接
│   │   ├── auth.rs       # JWT 认证
│   │   └── xray.rs       # Xray 配置
│   ├── migrations/
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── views/
│   │   ├── App.vue
│   │   └── main.ts
│   └── package.json
└── docs/
    └── 开发日志.md
```

## 开发日志

详见 [docs/开发日志.md](docs/开发日志.md)

## License

MIT

## 致谢

- [3x-ui](https://github.com/vaxilu/x-ui) - 灵感来源
- [Xray-core](https://github.com/XTLS/Xray-core) - 代理核心
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
