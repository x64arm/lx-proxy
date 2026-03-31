#!/bin/bash
# P17 高可用部署 - 健康检查脚本
# 用于 Kubernetes 健康检查和生产环境监控

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
API_URL="${API_URL:-http://localhost:8080}"
TIMEOUT="${TIMEOUT:-5}"

echo "🏥 LX-Proxy Health Check"
echo "========================"
echo "API URL: $API_URL"
echo "Timeout: ${TIMEOUT}s"
echo ""

# 检查 HTTP 健康端点
check_http_health() {
    echo -n "Checking HTTP health endpoint... "
    
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $TIMEOUT "$API_URL/health" 2>/dev/null || echo "000")
    
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✅ OK${NC} (HTTP $response)"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC} (HTTP $response)"
        return 1
    fi
}

# 检查数据库连接
check_database() {
    echo -n "Checking database connection... "
    
    response=$(curl -s --max-time $TIMEOUT "$API_URL/health/db" 2>/dev/null || echo '{"status":"error"}')
    status=$(echo "$response" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
    
    if [ "$status" = "healthy" ]; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC} ($status)"
        return 1
    fi
}

# 检查 Redis 连接
check_redis() {
    echo -n "Checking Redis connection... "
    
    response=$(curl -s --max-time $TIMEOUT "$API_URL/health/redis" 2>/dev/null || echo '{"status":"error"}')
    status=$(echo "$response" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
    
    if [ "$status" = "healthy" ]; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC} ($status)"
        return 1
    fi
}

# 检查缓存状态
check_cache() {
    echo -n "Checking cache status... "
    
    response=$(curl -s --max-time $TIMEOUT "$API_URL/cache/health" 2>/dev/null || echo '{"status":"error"}')
    status=$(echo "$response" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
    
    if [ "$status" = "healthy" ]; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  WARNING${NC} ($status)"
        return 0  # 缓存不是关键依赖
    fi
}

# 获取系统信息
get_system_info() {
    echo ""
    echo "📊 System Information"
    echo "---------------------"
    
    response=$(curl -s --max-time $TIMEOUT "$API_URL/health" 2>/dev/null || echo '{}')
    
    echo "$response" | grep -o '"version":"[^"]*"' | cut -d'"' -f4 | xargs -I {} echo "Version: {}"
    echo "Uptime: $(curl -s --max-time $TIMEOUT "$API_URL/health" 2>/dev/null | grep -o '"uptime":[0-9.]*' | cut -d':' -f2 || echo 'N/A')s"
}

# 主函数
main() {
    local exit_code=0
    
    # 执行健康检查
    check_http_health || exit_code=1
    check_database || exit_code=1
    check_redis || exit_code=1
    check_cache
    
    # 获取系统信息
    get_system_info
    
    echo ""
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ All health checks passed${NC}"
    else
        echo -e "${RED}❌ Some health checks failed${NC}"
    fi
    
    exit $exit_code
}

# 运行主函数
main
