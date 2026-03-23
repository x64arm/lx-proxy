# LX-Proxy API 文档

**版本：** 1.0.0  
**基础路径：** `/api`  
**认证方式：** Bearer Token (JWT)

---

## 📋 目录

1. [认证](#认证)
2. [用户管理](#用户管理)
3. [入站配置](#入站配置)
4. [流量统计](#流量统计)
5. [系统配置](#系统配置)
6. [邮件通知](#邮件通知)
7. [双因素认证](#双因素认证)

---

## 认证

### 用户登录
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}

# 响应
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "uuid",
    "username": "admin",
    "role": "admin"
  }
}
```

### 获取当前用户
```http
GET /api/auth/me
Authorization: Bearer {token}

# 响应
{
  "id": "uuid",
  "username": "admin",
  "role": "admin",
  "created_at": "2026-03-06T00:00:00Z"
}
```

### 登出
```http
POST /api/auth/logout
Authorization: Bearer {token}
```

---

## 用户管理

### 获取用户列表
```http
GET /api/users
Authorization: Bearer {token}

# 响应
{
  "users": [
    {
      "id": "uuid",
      "username": "admin",
      "role": "admin",
      "created_at": "2026-03-06T00:00:00Z"
    }
  ]
}
```

### 创建用户
```http
POST /api/users
Authorization: Bearer {token}
Content-Type: application/json

{
  "username": "newuser",
  "password": "password123",
  "role": "user"
}
```

### 更新用户
```http
PUT /api/users/{id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "username": "updateduser",
  "role": "admin"
}
```

### 删除用户
```http
DELETE /api/users/{id}
Authorization: Bearer {token}
```

---

## 入站配置

### 获取入站列表
```http
GET /api/inbounds
Authorization: Bearer {token}

# 查询参数
- enable: boolean (筛选启用/禁用)
- protocol: string (协议类型：vmess, vless, trojan, shadowsocks)
- user_id: uuid (按用户筛选)
```

### 创建入站配置
```http
POST /api/inbounds
Authorization: Bearer {token}
Content-Type: application/json

{
  "user_id": "uuid",
  "tag": "vmess-001",
  "protocol": "vmess",
  "port": 443,
  "settings": {...},
  "stream_settings": {...},
  "traffic_limit": 10737418240,  // 10GB
  "expire_at": "2026-12-31T23:59:59Z"
}
```

### 重置流量
```http
POST /api/inbounds/{id}/reset
Authorization: Bearer {token}
```

### 获取订阅链接
```http
GET /api/inbounds/{id}/links
Authorization: Bearer {token}

# 响应
{
  "subscription_url": "https://example.com/sub/uuid",
  "qr_code": "data:image/png;base64,..."
}
```

---

## 流量统计

### 获取所有流量统计
```http
GET /api/traffic
Authorization: Bearer {token}

# 查询参数
- inbound_id: uuid
- start_date: date (YYYY-MM-DD)
- end_date: date (YYYY-MM-DD)
- limit: integer (默认 30)

# 响应
[
  {
    "date": "2026-03-23",
    "inbound_id": "uuid",
    "inbound_name": "vmess-001",
    "upload": 1048576,
    "download": 10485760
  }
]
```

### 获取流量汇总
```http
GET /api/traffic/summary
Authorization: Bearer {token}

# 响应
{
  "total_upload": 1073741824,
  "total_download": 10737418240,
  "total_traffic": 11811160064
}
```

---

## 系统配置

### 获取系统状态
```http
GET /api/system/status
Authorization: Bearer {token}

# 响应
{
  "cpu_usage": 25.5,
  "memory_total": 8589934592,
  "memory_used": 4294967296,
  "memory_free": 4294967296,
  "uptime": 86400,
  "xray_running": true,
  "connections": 15
}
```

### 获取系统配置
```http
GET /api/config
Authorization: Bearer {token}
```

### 更新系统配置
```http
PUT /api/config
Authorization: Bearer {token}
Content-Type: application/json

{
  "web_title": "LX-Proxy",
  "timezone": "Asia/Shanghai",
  "language": "zh-CN"
}
```

---

## 邮件通知

### 发送测试邮件
```http
POST /api/email/test
Authorization: Bearer {token}
Content-Type: application/json

{
  "email": "user@example.com"
}

# 响应
200 OK
```

### 获取邮件配置状态
```http
GET /api/email/status
Authorization: Bearer {token}

# 响应
{
  "configured": true,
  "smtp_server": "smtp.gmail.com",
  "smtp_port": 587,
  "from_email": "noreply@lx-proxy.com"
}
```

---

## 双因素认证

### 初始化 TOTP 设置
```http
POST /api/totp/setup
Authorization: Bearer {token}
Content-Type: application/json

{
  "user_id": "uuid"
}

# 响应
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code_url": "otpauth://totp/LX-Proxy:admin?secret=...",
  "backup_codes": ["12345678", "87654321", ...],
  "message": "请使用 Authenticator 应用扫描二维码或手动输入密钥"
}
```

### 验证并启用 TOTP
```http
POST /api/totp/{user_id}/verify
Authorization: Bearer {token}
Content-Type: application/json

{
  "code": "123456"
}

# 响应
200 OK
```

### 获取 TOTP 状态
```http
GET /api/totp/{user_id}/status
Authorization: Bearer {token}

# 响应
{
  "enabled": true,
  "verified": true,
  "backup_codes_remaining": 8
}
```

### 禁用 TOTP
```http
POST /api/totp/{user_id}/disable
Authorization: Bearer {token}
Content-Type: application/json

{
  "password": "user-password"
}
```

### 备用代码登录
```http
POST /api/totp/backup-login
Content-Type: application/json

{
  "user_id": "uuid",
  "backup_code": "12345678"
}
```

---

## 错误响应

### 通用错误格式
```json
{
  "error": "错误信息",
  "code": "ERROR_CODE"
}
```

### 常见错误码
- `400` - 请求参数错误
- `401` - 未授权（Token 无效或过期）
- `403` - 禁止访问（权限不足）
- `404` - 资源不存在
- `409` - 资源冲突
- `500` - 服务器内部错误

---

## 环境变量

### 必需配置
```bash
# 服务器
HOST=0.0.0.0
PORT=8080

# 数据库
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/lx_proxy

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRATION_HOURS=24
```

### 可选配置
```bash
# Xray API
XRAY_API_URL=http://127.0.0.1:62780

# 邮件通知
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=your-email@gmail.com
SMTP_FROM_NAME=LX-Proxy
```

---

## 速率限制

- 认证接口：10 次/分钟
- 其他接口：100 次/分钟
- 邮件发送：5 次/小时

---

*最后更新：2026-03-23*
