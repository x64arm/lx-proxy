#!/bin/bash
# LX-Proxy API 集成测试脚本

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
TEST_USER="test_api_user"
TEST_PASS="test123456"

echo "🧪 LX-Proxy API 集成测试"
echo "================================"
echo "Base URL: $BASE_URL"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数
PASSED=0
FAILED=0

# 辅助函数
test_api() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_status="$5"
    
    echo -n "测试：$name ... "
    
    if [ "$method" == "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL$endpoint" ${4:+-H "Content-Type: application/json" -d "$4"})
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$BASE_URL$endpoint" ${4:+-H "Content-Type: application/json" -d "$4"})
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    
    if [ "$http_code" == "$expected_status" ]; then
        echo -e "${GREEN}✓ 通过${NC} ($http_code)"
        ((PASSED++))
    else
        echo -e "${RED}✗ 失败${NC} (期望：$expected_status, 实际：$http_code)"
        echo "响应：$body"
        ((FAILED++))
    fi
}

# 1. 健康检查
echo ""
echo "📌 1. 健康检查"
test_api "健康检查" "GET" "/health" "" "200"

# 2. 认证测试
echo ""
echo "📌 2. 认证测试"
test_api "管理员登录" "POST" "/api/auth/login" '{"username":"admin","password":"admin123"}' "200"

# 获取 Token
TOKEN=$(curl -s -X POST "$BASE_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"admin123"}' | jq -r '.token')

if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
    echo -e "${GREEN}✓ 获取 Token 成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 获取 Token 失败${NC}"
    ((FAILED++))
    exit 1
fi

# 3. 用户管理测试
echo ""
echo "📌 3. 用户管理测试"
test_api "获取用户列表" "GET" "/api/users" "" "200"

# 4. 入站配置测试
echo ""
echo "📌 4. 入站配置测试"
test_api "获取入站列表" "GET" "/api/inbounds" "" "200"

# 5. 流量统计测试
echo ""
echo "📌 5. 流量统计测试"
test_api "获取流量统计" "GET" "/api/traffic" "" "200"
test_api "获取流量汇总" "GET" "/api/traffic/summary" "" "200"

# 6. 系统配置测试
echo ""
echo "📌 6. 系统配置测试"
test_api "获取系统状态" "GET" "/api/system/status" "" "200"
test_api "获取系统配置" "GET" "/api/config" "" "200"

# 7. 邮件通知测试
echo ""
echo "📌 7. 邮件通知测试"
test_api "获取邮件配置状态" "GET" "/api/email/status" "" "200"

# 8. TOTP 测试
echo ""
echo "📌 8. 双因素认证测试"
test_api "获取 TOTP 状态（未登录）" "GET" "/api/totp/00000000-0000-0000-0000-000000000000/status" "" "401"

# 总结
echo ""
echo "================================"
echo "📊 测试结果总结"
echo "================================"
echo -e "通过：${GREEN}$PASSED${NC}"
echo -e "失败：${RED}$FAILED${NC}"
echo "总计：$((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 部分测试失败${NC}"
    exit 1
fi
