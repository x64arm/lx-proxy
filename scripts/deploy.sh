#!/bin/bash
# LX-Proxy 生产部署脚本
# 使用：./scripts/deploy.sh

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# 检查是否以 root 运行
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "请使用 root 用户运行此脚本"
        exit 1
    fi
}

# 检查系统
check_system() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$NAME
        VER=$VERSION_ID
        log_info "检测到系统：$OS $VER"
    else
        log_error "无法检测系统版本"
        exit 1
    fi
}

# 安装依赖
install_dependencies() {
    log_info "安装系统依赖..."
    
    if command -v apt-get &> /dev/null; then
        apt-get update
        apt-get install -y \
            curl \
            wget \
            git \
            ca-certificates \
            gnupg \
            lsb-release \
            ufw \
            fail2ban
    elif command -v yum &> /dev/null; then
        yum update -y
        yum install -y \
            curl \
            wget \
            git \
            ca-certificates \
            ufw \
            fail2ban
    else
        log_error "不支持的包管理器"
        exit 1
    fi
    
    log_success "依赖安装完成"
}

# 安装 Docker
install_docker() {
    log_info "安装 Docker..."
    
    if ! command -v docker &> /dev/null; then
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        rm get-docker.sh
        log_success "Docker 安装完成"
    else
        log_info "Docker 已安装"
    fi
    
    # 启动 Docker
    systemctl enable docker
    systemctl start docker
    
    # 安装 Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" \
            -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
        log_success "Docker Compose 安装完成"
    fi
}

# 配置防火墙
setup_firewall() {
    log_info "配置防火墙..."
    
    if command -v ufw &> /dev/null; then
        ufw --force reset
        ufw default deny incoming
        ufw default allow outgoing
        
        # SSH
        ufw allow 22/tcp comment 'SSH'
        
        # HTTP/HTTPS
        ufw allow 80/tcp comment 'HTTP'
        ufw allow 443/tcp comment 'HTTPS'
        
        # 管理面板（可选，建议限制 IP）
        # ufw allow from 192.168.1.0/24 to any port 8080
        ufw allow 8080/tcp comment 'LX-Proxy Panel'
        
        ufw --force enable
        log_success "防火墙配置完成"
    else
        log_warning "UFW 未安装，跳过防火墙配置"
    fi
}

# 克隆项目
clone_project() {
    log_info "克隆 LX-Proxy 项目..."
    
    if [ -d "lx-proxy" ]; then
        log_warning "lx-proxy 目录已存在"
        read -p "是否删除并重新克隆？(y/n): " confirm
        if [ "$confirm" = "y" ]; then
            rm -rf lx-proxy
        else
            log_info "使用现有目录"
        fi
    fi
    
    if [ ! -d "lx-proxy" ]; then
        git clone https://github.com/x64arm/lx-proxy.git
        cd lx-proxy
    else
        cd lx-proxy
        git pull
    fi
    
    log_success "项目克隆完成"
}

# 配置环境变量
setup_env() {
    log_info "配置环境变量..."
    
    # 生成随机 JWT_SECRET
    JWT_SECRET=$(openssl rand -hex 32)
    
    # 复制并配置 .env
    if [ ! -f ".env" ]; then
        cp .env.example .env
        sed -i "s/your-super-secret-jwt-key-change-this-in-production/$JWT_SECRET/" .env
        log_success ".env 配置完成"
    else
        log_warning ".env 已存在，跳过"
    fi
    
    # 配置后端环境变量
    if [ ! -f "backend/.env" ]; then
        cp backend/.env.example backend/.env
        log_success "backend/.env 配置完成"
    else
        log_warning "backend/.env 已存在，跳过"
    fi
}

# 启动服务
start_services() {
    log_info "启动服务..."
    
    # 构建并启动
    docker-compose up -d --build
    
    # 等待服务启动
    log_info "等待服务启动..."
    sleep 10
    
    # 检查服务状态
    docker-compose ps
    
    log_success "服务启动完成"
}

# 显示访问信息
show_info() {
    echo ""
    echo "================================"
    echo -e "${GREEN}✅ 部署完成！${NC}"
    echo "================================"
    echo ""
    echo "📌 访问信息："
    echo "   管理面板：http://$(hostname -I | awk '{print $1}'):8080"
    echo "   用户名：admin"
    echo "   密码：admin123"
    echo ""
    echo "⚠️  安全提示："
    echo "   1. 首次登录后请立即修改密码"
    echo "   2. 建议配置 HTTPS"
    echo "   3. 建议限制管理面板访问 IP"
    echo ""
    echo "📚 常用命令："
    echo "   查看日志：docker-compose logs -f"
    echo "   重启服务：docker-compose restart"
    echo "   停止服务：docker-compose down"
    echo "   更新版本：make update"
    echo ""
    echo "📖 文档："
    echo "   https://github.com/x64arm/lx-proxy/docs"
    echo ""
}

# 主函数
main() {
    echo ""
    echo "================================"
    echo "  LX-Proxy 生产部署脚本"
    echo "================================"
    echo ""
    
    check_root
    check_system
    install_dependencies
    install_docker
    setup_firewall
    clone_project
    setup_env
    start_services
    show_info
}

# 运行主函数
main
