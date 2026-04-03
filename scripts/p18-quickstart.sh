#!/bin/bash
# P18 安全加固 - 快速部署指南
# 一键完成安全配置

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  P18 安全加固 - 快速部署指南          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# 步骤 1：生成密钥
echo -e "${YELLOW}[1/4] 生成加密密钥...${NC}"
ENCRYPTION_KEY=$(openssl rand -hex 32)
echo "✅ ENCRYPTION_KEY 已生成"

# 步骤 2：生成 JWT 密钥
echo -e "${YELLOW}[2/4] 生成 JWT 密钥...${NC}"
JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
echo "✅ JWT_SECRET 已生成"

# 步骤 3：更新 .env 文件
echo -e "${YELLOW}[3/4] 更新 .env 配置文件...${NC}"
ENV_FILE="$PROJECT_ROOT/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ .env 文件不存在，正在创建...${NC}"
    cp "$PROJECT_ROOT/.env.example" "$ENV_FILE" 2>/dev/null || true
fi

# 备份旧配置
if [ -f "$ENV_FILE" ]; then
    cp "$ENV_FILE" "$ENV_FILE.backup.$(date +%Y%m%d_%H%M%S)"
    echo "✅ 已备份旧配置"
fi

# 添加或更新密钥配置
if grep -q "^ENCRYPTION_KEY=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s|^ENCRYPTION_KEY=.*|ENCRYPTION_KEY=$ENCRYPTION_KEY|" "$ENV_FILE"
else
    echo "ENCRYPTION_KEY=$ENCRYPTION_KEY" >> "$ENV_FILE"
fi

if grep -q "^JWT_SECRET=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s|^JWT_SECRET=.*|JWT_SECRET=$JWT_SECRET|" "$ENV_FILE"
else
    echo "JWT_SECRET=$JWT_SECRET" >> "$ENV_FILE"
fi

echo "✅ .env 配置已更新"

# 步骤 4：设置文件权限
echo -e "${YELLOW}[4/4] 设置安全权限...${NC}"
chmod 600 "$ENV_FILE"
echo "✅ .env 文件权限已设置为 600（仅所有者可读写）"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✅ P18 安全加固配置完成！             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""

# 显示下一步操作
echo -e "${BLUE}📋 下一步操作：${NC}"
echo ""
echo "1. 重启后端服务以应用新配置："
echo -e "   ${YELLOW}cd $PROJECT_ROOT && docker-compose restart backend${NC}"
echo ""
echo "2. 配置 HTTPS（生产环境必须）："
echo -e "   ${YELLOW}参考文档：docs/HTTPS 部署配置.md${NC}"
echo ""
echo "3. 运行安全检查："
echo -e "   ${YELLOW}$PROJECT_ROOT/scripts/security-setup.sh check-security${NC}"
echo ""
echo "4. 查看完整的安全加固文档："
echo -e "   ${YELLOW}docs/P18-实施完成报告.md${NC}"
echo ""

# 安全提示
echo -e "${RED}⚠️  重要安全提示：${NC}"
echo ""
echo "• 请勿将 .env 文件提交到版本控制系统"
echo "• 定期更换密钥（建议每 90 天）"
echo "• 生产环境必须启用 HTTPS"
echo "• 定期检查依赖漏洞：cargo audit && npm audit"
echo ""

# 验证配置
echo -e "${BLUE}🔍 验证配置...${NC}"
echo ""

if grep -q "ENCRYPTION_KEY=$ENCRYPTION_KEY" "$ENV_FILE"; then
    echo -e "${GREEN}✅ ENCRYPTION_KEY 配置正确${NC}"
else
    echo -e "${RED}❌ ENCRYPTION_KEY 配置失败${NC}"
    exit 1
fi

if grep -q "JWT_SECRET=$JWT_SECRET" "$ENV_FILE"; then
    echo -e "${GREEN}✅ JWT_SECRET 配置正确${NC}"
else
    echo -e "${RED}❌ JWT_SECRET 配置失败${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 所有配置验证通过！${NC}"
echo ""
