# HTTPS 部署配置（P18 安全加固）

**更新时间：** 2026-04-01

---

## 📋 目录

1. [Nginx 反向代理配置](#nginx-反向代理配置)
2. [Docker Compose HTTPS 配置](#docker-compose-https-配置)
3. [Let's Encrypt 证书自动续期](#lets-encrypt-证书自动续期)
4. [HTTP 强制跳转 HTTPS](#http-强制跳转-https)
5. [HSTS 配置](#hsts-配置)

---

## Nginx 反向代理配置

### 1. 创建 Nginx 配置文件

```nginx
# /etc/nginx/sites-available/lx-proxy
server {
    listen 80;
    listen [::]:80;
    server_name your-domain.com;

    # HTTP 强制跳转 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name your-domain.com;

    # SSL 证书配置
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # SSL 优化配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;

    # HSTS（HTTP Strict Transport Security）
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;

    # 安全响应头
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # 日志配置
    access_log /var/log/nginx/lx-proxy-access.log;
    error_log /var/log/nginx/lx-proxy-error.log;

    # 前端静态文件
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # 后端 API
    location /api {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 增加超时时间（针对长时间运行的 API）
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket 支持
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    # 健康检查端点（公开访问）
    location /health {
        proxy_pass http://127.0.0.1:8080;
        access_log off;
    }
}
```

### 2. 启用配置

```bash
# 创建软链接
sudo ln -s /etc/nginx/sites-available/lx-proxy /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重载 Nginx
sudo systemctl reload nginx
```

---

## Docker Compose HTTPS 配置

### 1. 使用 Nginx Proxy Manager（推荐）

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  # Nginx Proxy Manager（自动 SSL）
  nginx-proxy-manager:
    image: 'jc21/nginx-proxy-manager:latest'
    restart: unless-stopped
    ports:
      - '80:80'
      - '443:443'
      - '81:81'  # 管理界面
    volumes:
      - ./data/npm-data:/data
      - ./data/letsencrypt:/etc/letsencrypt
    networks:
      - lx-proxy-network

  # LX-Proxy 后端
  lx-proxy-backend:
    image: lx-proxy-backend:latest
    restart: unless-stopped
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/lxproxy
      - JWT_SECRET=your-secret-key
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    networks:
      - lx-proxy-network
    expose:
      - '8080'

  # LX-Proxy 前端
  lx-proxy-frontend:
    image: lx-proxy-frontend:latest
    restart: unless-stopped
    networks:
      - lx-proxy-network
    expose:
      - '3000'

  # PostgreSQL
  db:
    image: postgres:15-alpine
    restart: unless-stopped
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
      - POSTGRES_DB=lxproxy
    volumes:
      - ./data/postgres:/var/lib/postgresql/data
    networks:
      - lx-proxy-network

  # Redis
  redis:
    image: redis:7-alpine
    restart: unless-stopped
    volumes:
      - ./data/redis:/data
    networks:
      - lx-proxy-network

networks:
  lx-proxy-network:
    driver: bridge
```

### 2. 通过 Web 界面配置 SSL

1. 访问 `http://your-server-ip:81`
2. 登录 Nginx Proxy Manager（默认：admin@example.com / changeme）
3. 添加 Proxy Host：
   - Domain: `your-domain.com`
   - Scheme: `http`
   - Forward Host: `lx-proxy-frontend`
   - Forward Port: `3000`
4. 在 SSL 标签页：
   - 选择 "Request a new SSL certificate"
   - 勾选 "Force SSL"
   - 勾选 "HTTP/2 Support"
   - 勾选 "HSTS Enabled"

---

## Let's Encrypt 证书自动续期

### 方法 1：Certbot（独立安装）

```bash
# 安装 Certbot
sudo apt update
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期测试
sudo certbot renew --dry-run
```

### 方法 2：Docker Certbot

```bash
# 创建证书目录
mkdir -p ./data/certbot/www
mkdir -p ./data/certbot/conf

# 首次获取证书
docker run --rm \
  -v ./data/certbot/conf:/etc/letsencrypt \
  -v ./data/certbot/www:/var/www/certbot \
  certbot/certbot \
  certonly --webroot \
  -w /var/www/certbot \
  -d your-domain.com \
  --email your-email@example.com \
  --agree-tos \
  --no-eff-email

# 创建续期脚本
cat > scripts/renew-cert.sh << 'EOF'
#!/bin/bash
docker run --rm \
  -v ./data/certbot/conf:/etc/letsencrypt \
  -v ./data/certbot/www:/var/www/certbot \
  certbot/certbot \
  renew --webroot \
  -w /var/www/certbot

# 重载 Nginx
docker-compose exec nginx nginx -s reload
EOF

chmod +x scripts/renew-cert.sh

# 添加 cron 任务（每月 1 号检查）
(crontab -l 2>/dev/null; echo "0 0 1 * * /path/to/scripts/renew-cert.sh") | crontab -
```

---

## HTTP 强制跳转 HTTPS

### Nginx 配置

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    # 强制跳转 HTTPS
    return 301 https://$server_name$request_uri;
}
```

### 验证跳转

```bash
# 测试 HTTP 跳转
curl -I http://your-domain.com

# 应该返回 301 跳转到 HTTPS
HTTP/1.1 301 Moved Permanently
Location: https://your-domain.com/
```

---

## HSTS 配置

### Nginx HSTS 响应头

```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
```

### HSTS 说明

- `max-age=31536000`：有效期 1 年（31536000 秒）
- `includeSubDomains`：包含所有子域名
- `preload`：允许加入浏览器 HSTS 预加载列表

### 提交到 HSTS 预加载列表

访问：https://hstspreload.org/

提交你的域名，Chrome/Firefox/Edge 等浏览器将强制使用 HTTPS。

---

## 安全验证

### 1. SSL 配置测试

```bash
# 使用 nmap 检查 SSL 配置
nmap --script ssl-enum-ciphers -p 443 your-domain.com

# 使用 OpenSSL 测试
openssl s_client -connect your-domain.com:443 -tls1_2
openssl s_client -connect your-domain.com:443 -tls1_3
```

### 2. 在线工具

- **SSL Labs:** https://www.ssllabs.com/ssltest/
- **Security Headers:** https://securityheaders.com/
- **Mozilla Observatory:** https://observatory.mozilla.org/

### 3. 预期结果

- SSL Labs 评分：**A+**
- Security Headers 评分：**A+**
- 强制 HTTPS 跳转：✅
- HSTS 启用：✅
- 安全响应头齐全：✅

---

## 故障排查

### 问题 1：证书续期失败

```bash
# 检查 Certbot 日志
sudo tail -f /var/log/letsencrypt/letsencrypt.log

# 手动续期
sudo certbot renew --force-renewal
```

### 问题 2：Nginx 无法启动

```bash
# 测试配置
sudo nginx -t

# 查看错误日志
sudo tail -f /var/log/nginx/error.log
```

### 问题 3：混合内容警告

浏览器控制台显示 "Mixed Content" 错误：

- 检查前端代码中是否有 `http://` 链接
- 将所有资源链接改为 `https://` 或相对路径
- 使用 CSP 升级不安全请求：

```nginx
add_header Content-Security-Policy "upgrade-insecure-requests" always;
```

---

## 最佳实践

1. ✅ **定期更新证书** - 设置自动续期，提前 30 天检查
2. ✅ **使用 TLS 1.3** - 禁用旧版本 TLS
3. ✅ **强加密套件** - 使用 ECDHE 密钥交换
4. ✅ **启用 HSTS** - 强制浏览器使用 HTTPS
5. ✅ **安全响应头** - 配置完整的安全头
6. ✅ **定期扫描** - 使用 SSL Labs 等工具定期检查

---

*参考文档：*
- [Nginx SSL 配置](https://nginx.org/en/docs/http/configuring_https_servers.html)
- [Let's Encrypt 官方文档](https://letsencrypt.org/docs/)
- [Mozilla SSL 配置生成器](https://ssl-config.mozilla.org/)
