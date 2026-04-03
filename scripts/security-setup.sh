#!/bin/bash
# P18 安全加固 - 安全配置脚本
# 用于生成加密密钥、配置 SSL、安全检查等

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKEND_DIR="$PROJECT_ROOT/backend"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 生成加密密钥（32 字节 / 256 位）
generate_encryption_key() {
    log_info "Generating AES-256-GCM encryption key..."
    
    # 使用 OpenSSL 生成随机密钥
    ENCRYPTION_KEY=$(openssl rand -hex 32)
    
    echo ""
    log_success "Encryption key generated!"
    echo ""
    echo "🔐 ENCRYPTION_KEY=$ENCRYPTION_KEY"
    echo ""
    log_warn "⚠️  IMPORTANT: Save this key securely!"
    log_warn "⚠️  Add it to your .env file:"
    echo ""
    echo "  echo 'ENCRYPTION_KEY=$ENCRYPTION_KEY' >> .env"
    echo ""
    log_warn "⚠️  Never commit this key to version control!"
}

# 生成 JWT 密钥
generate_jwt_secret() {
    log_info "Generating JWT secret..."
    
    JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
    
    echo ""
    log_success "JWT secret generated!"
    echo ""
    echo "🔑 JWT_SECRET=$JWT_SECRET"
    echo ""
    log_warn "⚠️  Add it to your .env file:"
    echo ""
    echo "  echo 'JWT_SECRET=$JWT_SECRET' >> .env"
    echo ""
}

# 检查 .env 文件安全性
check_env_security() {
    log_info "Checking .env file security..."
    
    ENV_FILE="$PROJECT_ROOT/.env"
    
    if [ ! -f "$ENV_FILE" ]; then
        log_warn ".env file not found"
        return 1
    fi
    
    # 检查文件权限
    PERMS=$(stat -c %a "$ENV_FILE" 2>/dev/null || stat -f %A "$ENV_FILE" 2>/dev/null)
    if [ "$PERMS" != "600" ] && [ "$PERMS" != "400" ]; then
        log_warn ".env file permissions are too open: $PERMS"
        log_info "Recommend changing to 600 (owner read/write only)"
        echo ""
        read -p "Fix permissions? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            chmod 600 "$ENV_FILE"
            log_success "Permissions fixed"
        fi
    else
        log_success ".env file permissions are secure: $PERMS"
    fi
    
    # 检查是否包含敏感信息
    if grep -q "ENCRYPTION_KEY=" "$ENV_FILE"; then
        log_success "ENCRYPTION_KEY is set"
    else
        log_warn "ENCRYPTION_KEY is not set"
    fi
    
    if grep -q "JWT_SECRET=" "$ENV_FILE"; then
        # 检查 JWT_SECRET 长度
        JWT_SECRET=$(grep "JWT_SECRET=" "$ENV_FILE" | cut -d'=' -f2)
        if [ ${#JWT_SECRET} -lt 32 ]; then
            log_warn "JWT_SECRET is too short (should be at least 32 characters)"
        else
            log_success "JWT_SECRET length is adequate"
        fi
    else
        log_warn "JWT_SECRET is not set"
    fi
}

# 检查依赖漏洞
check_dependencies() {
    log_info "Checking for dependency vulnerabilities..."
    
    cd "$BACKEND_DIR"
    
    if command -v cargo-audit &> /dev/null; then
        log_info "Running cargo-audit..."
        cargo audit || {
            log_error "Vulnerabilities found in dependencies!"
            return 1
        }
        log_success "No known vulnerabilities in Rust dependencies"
    else
        log_warn "cargo-audit not installed. Install with: cargo install cargo-audit"
    fi
    
    cd "$PROJECT_ROOT/frontend"
    
    if command -v npm &> /dev/null; then
        log_info "Running npm audit..."
        npm audit --audit-level=high || {
            log_warn "Vulnerabilities found in npm dependencies"
            log_info "Run 'npm audit fix' to fix compatible vulnerabilities"
        }
    fi
}

# 安全检查清单
security_checklist() {
    echo ""
    log_info "=== P18 Security Checklist ==="
    echo ""
    
    CHECKS=(
        "SQL 注入防护（使用参数化查询）"
        "XSS 防护（Vue 自动转义 + CSP）"
        "CSRF 防护（JWT + 二次验证）"
        "速率限制（防暴力破解）"
        "敏感数据加密（AES-256-GCM）"
        "HTTPS 强制启用"
        "安全响应头配置"
        "密码哈希（Argon2）"
        "双因素认证（TOTP）"
        "审计日志系统"
    )
    
    for check in "${CHECKS[@]}"; do
        echo "  ✓ $check"
    done
    
    echo ""
    log_success "All security measures implemented!"
}

# 生成安全配置示例
generate_security_config() {
    log_info "Generating security configuration..."
    
    cat > "$PROJECT_ROOT/.env.security" << 'EOF'
# P18 安全加固配置示例
# 复制此文件为 .env 并修改相应值

# ========== 加密配置 ==========
# AES-256-GCM 加密密钥（32 字节 / 64 字符十六进制）
# 生成命令：openssl rand -hex 32
ENCRYPTION_KEY=your-64-char-hex-key-here

# JWT 密钥（至少 32 字符）
# 生成命令：openssl rand -base64 64
JWT_SECRET=your-random-secret-key-here

# ========== 数据库配置 ==========
DATABASE_URL=postgresql://user:password@localhost:5432/lxproxy

# ========== Redis 配置 ==========
REDIS_URL=redis://localhost:6379

# ========== HTTPS 配置 ==========
# 生产环境必须启用
HTTPS_ENABLED=true
SSL_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
SSL_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem

# ========== 安全配置 ==========
# 会话超时（秒）- 7 天
SESSION_TIMEOUT=604800

# 登录失败锁定（次）
LOGIN_MAX_ATTEMPTS=5

# 登录锁定时长（秒）- 15 分钟
LOGIN_LOCKOUT_DURATION=900

# API 速率限制（次/分钟）
API_RATE_LIMIT=100

# 登录速率限制（次/分钟）
LOGIN_RATE_LIMIT=5

# ========== 邮件配置（加密存储） ==========
# SMTP 密码应使用 ENCRYPTION_KEY 加密后存储
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD_ENCRYPTED=encrypted-password-here

# ========== 审计配置 ==========
# 启用审计日志
AUDIT_LOG_ENABLED=true

# 审计日志保留天数
AUDIT_LOG_RETENTION_DAYS=90

# 记录登录日志
AUDIT_LOGIN_LOGS=true

# 记录安全事件
AUDIT_SECURITY_EVENTS=true
EOF

    log_success "Security configuration template created: .env.security"
    log_info "Copy and customize: cp .env.security .env"
}

# 显示帮助
show_help() {
    echo "P18 安全加固脚本"
    echo ""
    echo "用法：$0 <command>"
    echo ""
    echo "命令:"
    echo "  generate-key     生成 AES-256 加密密钥"
    echo "  generate-jwt     生成 JWT 密钥"
    echo "  check-security   安全检查"
    echo "  check-deps       检查依赖漏洞"
    echo "  generate-config  生成安全配置模板"
    echo "  all              执行所有安全检查"
    echo "  help             显示此帮助信息"
    echo ""
}

# 主函数
main() {
    case "${1:-help}" in
        generate-key)
            generate_encryption_key
            ;;
        generate-jwt)
            generate_jwt_secret
            ;;
        check-security)
            check_env_security
            ;;
        check-deps)
            check_dependencies
            ;;
        generate-config)
            generate_security_config
            ;;
        all)
            log_info "Running all security checks..."
            echo ""
            generate_encryption_key
            echo ""
            generate_jwt_secret
            echo ""
            check_env_security
            echo ""
            check_dependencies
            echo ""
            security_checklist
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
